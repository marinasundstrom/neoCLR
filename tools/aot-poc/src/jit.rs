//! Private, host-only scalar feasibility probe. No managed services or escaping code pointers.
use cranelift_codegen::{isa, settings, settings::Configurable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncOrDataId, Module};
use std::time::Instant;

type Error = Box<dyn std::error::Error>;

// JITModule requires explicit memory reclamation. This owner never exposes a callable
// outside execute(), including when compilation or finalization returns an error.
struct CodeOwner(Option<JITModule>);
impl Drop for CodeOwner {
    fn drop(&mut self) {
        if let Some(module) = self.0.take() {
            // SAFETY: execution is synchronous, pointers do not escape, and the call
            // has returned before this owner can be dropped. Partial builds never run.
            unsafe { module.free_memory() };
        }
    }
}

pub(super) fn execute(
    input: &neoclr::Module,
    root: &str,
    argument: i32,
) -> Result<serde_json::Value, Error> {
    if !cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        return Err("scalar JIT probe currently requires a macOS ARM64 host".into());
    }
    if input.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|op| matches!(op, neoclr::metadata::Instruction::String(_)))
    }) {
        return Err(
            "scalar JIT probe does not support string literals or external services".into(),
        );
    }
    let started = Instant::now();
    let mut settings = settings::builder();
    settings.set("is_pic", "false")?;
    let target = "aarch64-apple-darwin"
        .parse()
        .expect("fixed valid target triple");
    let isa = isa::lookup(target)?.finish(settings::Flags::new(settings))?;
    let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
    // Shared scalar validation rejects all external calls before finalization.
    let mut owner = CodeOwner(Some(JITModule::new(builder)));
    let module = owner.0.as_mut().expect("owned module");
    crate::compiler::compile_scalar(input, root, false, module)?;
    module.finalize_definitions()?;
    let Some(FuncOrDataId::Func(entry)) = module.get_name("neoclr_entry_v2") else {
        return Err("scalar JIT entry was not emitted".into());
    };
    let pointer = module.get_finalized_function(entry);
    let compile_seconds = started.elapsed().as_secs_f64();
    let mut value = 0i32;
    let started = Instant::now();
    // SAFETY: shared scalar lowering always exports this exact C ABI, including its
    // parameterless-root adapter. The aligned writable result lives through the call,
    // and CodeOwner retains finalized executable storage until after return.
    let entry: unsafe extern "C" fn(i32, *mut i32) -> i32 = unsafe { std::mem::transmute(pointer) };
    let status = unsafe { entry(argument, &mut value) };
    let invocation_seconds = started.elapsed().as_secs_f64();
    Ok(serde_json::json!({
        "profile": "macos-arm64-jit-scalar-v1",
        "status": status,
        "value": if status == 0 { Some(value) } else { None },
        "compileAndFinalizeSeconds": compile_seconds,
        "invocationSeconds": invocation_seconds,
    }))
}
