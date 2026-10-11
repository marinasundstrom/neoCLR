//! Private synchronous JIT probes. Generated code pointers never escape their owner.
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
    repetitions: usize,
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
    settings.set("use_colocated_libcalls", "false")?;
    let target = "aarch64-apple-darwin"
        .parse()
        .expect("fixed valid target triple");
    let isa = isa::lookup(target)?.finish(settings::Flags::new(settings))?;
    let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
    // Shared scalar validation rejects all external calls before finalization.
    let mut owner = CodeOwner(Some(JITModule::new(builder)));
    let module = owner.0.as_mut().expect("owned module");
    crate::compiler::compile_scalar(input, root, false, module, true)?;
    module.finalize_definitions()?;
    let Some(FuncOrDataId::Func(entry)) = module.get_name("neoclr_entry_v2") else {
        return Err("scalar JIT entry was not emitted".into());
    };
    let pointer = module.get_finalized_function(entry);
    let compile_seconds = started.elapsed().as_secs_f64();
    // SAFETY: shared lowering exports this exact ABI. The owner retains executable
    // storage across every synchronous call; no callable escapes this scope.
    let entry: unsafe extern "C" fn(i32, *mut i32) -> i32 = unsafe { std::mem::transmute(pointer) };
    let mut invocations = Vec::with_capacity(repetitions);
    for _ in 0..repetitions {
        let mut value = 0i32;
        let started = Instant::now();
        let status = unsafe { entry(argument, &mut value) };
        invocations.push(serde_json::json!({
            "status": status,
            "value": if status == 0 { Some(value) } else { None },
            "invocationSeconds": started.elapsed().as_secs_f64(),
        }));
    }
    let first = invocations
        .first()
        .ok_or("at least one invocation is required")?;
    Ok(serde_json::json!({
        "profile": "macos-arm64-jit-scalar-v1",
        "status": first["status"], "value": first["value"],
        "compileAndFinalizeSeconds": compile_seconds,
        "invocationSeconds": first["invocationSeconds"],
        "invocations": invocations,
    }))
}

/// Synchronous development host: code, native helpers and the invocation heap do
/// not escape this call. The host must reject outstanding roots before returning.
pub(super) fn execute_managed(
    input: &neoclr::Module,
    root: &str,
    details: &crate::fault_details::Options,
    host: &std::path::Path,
    arguments: &[std::ffi::CString],
) -> Result<serde_json::Value, Error> {
    if !cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        return Err("managed JIT probe currently requires macOS ARM64".into());
    }
    if !details.native_gc
        || !details.reference_arena
        || !details.probe_stack_roots
        || !details.native_stack_budget
        || details.native_host_bootstrap
        || !details.socket_listen.is_empty()
    {
        return Err(
            "managed JIT requires a GC/stack host without socket or bootstrap services"
                .into(),
        );
    }
    let started = Instant::now();
    // SAFETY: this explicit development host path is trusted native code, like a
    // statically linked AOT adapter. Keep the library alive longer than JIT code.
    let library = std::sync::Arc::new(unsafe { libloading::Library::new(host) }?);
    type Host = unsafe extern "C" fn(*const u8, i32, i32, *const *const std::ffi::c_char) -> i32;
    let run: libloading::Symbol<Host> = unsafe { library.get(b"neoclr_jit_run_v1") }?;
    let mut flags = settings::builder();
    flags.set("is_pic", "false")?;
    flags.set("use_colocated_libcalls", "false")?;
    let isa = isa::lookup("aarch64-apple-darwin".parse().expect("fixed triple"))?
        .finish(settings::Flags::new(flags))?;
    let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
    let symbols = library.clone();
    builder.symbol_lookup_fn(Box::new(move |name| unsafe {
        symbols
            .get::<*const u8>(name.as_bytes())
            .ok()
            .map(|symbol| *symbol)
    }));
    let mut owner = CodeOwner(Some(JITModule::new(builder)));
    let module = owner.0.as_mut().expect("owned module");
    crate::compiler::values::compile_into(
        input,
        root,
        Some(details),
        crate::compiler::Target::MacosArm64,
        module,
        true,
    )?;
    // Bind every declared import explicitly. Missing helpers fail before any guest
    // code runs; do not let default process-symbol lookup hide a host mismatch.
    for (_, declaration) in module.declarations().get_functions() {
        if declaration.linkage == cranelift_module::Linkage::Import {
            let symbol: libloading::Symbol<*const u8> = unsafe {
                library.get(
                    declaration
                        .name
                        .as_ref()
                        .ok_or("anonymous native import")?
                        .as_bytes(),
                )
            }?;
            let _ = *symbol;
        }
    }
    module.finalize_definitions()?;
    let (entry, with_args) = match module.get_name("neoclr_entry_args_v1") {
        Some(FuncOrDataId::Func(id)) => (id, 1),
        _ => match module.get_name("neoclr_entry_v4") {
            Some(FuncOrDataId::Func(id)) => (id, 0),
            _ => return Err("managed JIT entry ABI missing".into()),
        },
    };
    let entry = module.get_finalized_function(entry);
    let compile_seconds = started.elapsed().as_secs_f64();
    let argv: Vec<_> = arguments.iter().map(|arg| arg.as_ptr()).collect();
    let argc = i32::try_from(argv.len())?;
    let started = Instant::now();
    // SAFETY: only the validated entry ABI is passed. argv and the helper library
    // remain alive; the synchronous host must complete cleanup before returning.
    let exit_code = unsafe { run(entry, with_args, argc, argv.as_ptr()) };
    Ok(
        serde_json::json!({"profile": "macos-arm64-jit-managed-v1", "guestExitCode": exit_code,
        "compileAndFinalizeSeconds": compile_seconds, "invocationSeconds": started.elapsed().as_secs_f64()}),
    )
}
