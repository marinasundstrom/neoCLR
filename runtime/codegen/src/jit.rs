//! Private synchronous JIT probes. Generated code pointers never escape their owner.
use cranelift_codegen::{isa, settings, settings::Configurable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncOrDataId, Module};
use std::time::Instant;

#[path = "jit_lazy.rs"]
pub(crate) mod lazy;

type Error = Box<dyn std::error::Error>;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
unsafe extern "C" {
    fn neoclr_native_import_resolve_v1(
        library: *const std::ffi::c_char,
        symbol: *const std::ffi::c_char,
        output: *mut *mut std::ffi::c_void,
    ) -> i32;
}

#[cfg(windows)]
type EntryChar = u16;
#[cfg(not(windows))]
type EntryChar = std::ffi::c_char;

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

pub(super) fn host_target() -> Result<crate::compiler::Target, Error> {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Ok(crate::compiler::Target::MacosArm64)
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Ok(crate::compiler::Target::WindowsX64)
    } else {
        Err("JIT requires a macOS ARM64 or Windows x64 host".into())
    }
}

pub(super) fn execute(
    input: &neoclr::Module,
    root: &str,
    argument: i32,
    repetitions: usize,
    native_imports: bool,
    verification: Option<&neoclr::LoadedProgram>,
) -> Result<serde_json::Value, Error> {
    let target = host_target()?;
    if native_imports && target != crate::compiler::Target::MacosArm64 {
        return Err("native imports currently require macOS ARM64".into());
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
    let mut settings = crate::backend_settings();
    settings.set("is_pic", "false")?;
    settings.set("use_colocated_libcalls", "false")?;
    target.configure_stack(&mut settings)?;
    let isa = isa::lookup(target.triple().parse().expect("fixed target triple"))?
        .finish(settings::Flags::new(settings))?;
    #[allow(unused_mut)]
    let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
    // Bind only the shared resolver, not arbitrary process symbols. Its cache owns
    // foreign libraries for process lifetime, independently of generated JIT code.
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    if native_imports {
        builder.symbol(
            "neoclr_native_import_resolve_v1",
            neoclr_native_import_resolve_v1 as *const u8,
        );
    }
    // Shared validation rejects other external services before finalization.
    let mut owner = CodeOwner(Some(crate::jit_memory::new_module(builder)));
    let module = owner.0.as_mut().expect("owned module");
    if native_imports {
        crate::compiler::compile_scalar_with_imports(
            input,
            root,
            false,
            module,
            true,
            true,
            verification,
        )?;
    } else {
        crate::compiler::compile_scalar(input, root, false, module, true)?;
    }
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
        "profile": match target { crate::compiler::Target::MacosArm64 => "macos-arm64-jit-scalar-v1", crate::compiler::Target::WindowsX64 => "windows-x64-jit-scalar-v1" },
        "status": first["status"], "value": first["value"],
        "compileAndFinalizeSeconds": compile_seconds,
        "invocationSeconds": first["invocationSeconds"],
        "invocations": invocations,
    }))
}

type ManagedHost = unsafe extern "C" fn(*const u8, i32, i32, *const *const EntryChar) -> i32;

/// Invocation owner retains executable storage and its native host. Lazy mode
/// keeps lowering inputs on its compiler worker through host cleanup. No borrowed
/// compilation inputs escape preparation.
pub(super) struct PreparedManaged {
    worker: Option<lazy::Worker>,
    _code: CodeOwner,
    _library: std::sync::Arc<libloading::Library>,
    run: ManagedHost,
    entry: *const u8,
    with_args: i32,
    target: crate::compiler::Target,
    compile_seconds: f64,
}

#[cfg(test)]
pub(super) fn execute_managed(
    input: &neoclr::Module,
    root: &str,
    details: &crate::fault_details::Options,
    host: &std::path::Path,
    arguments: &[std::ffi::CString],
    target: crate::compiler::Target,
) -> Result<serde_json::Value, Error> {
    prepare_managed(input, root, details, host, target)?.invoke(arguments)
}

/// Prepare a synchronous managed invocation. Eager mode releases compilation
/// inputs; lazy mode owns the selected inputs on a dedicated compiler worker.
/// The host must reject outstanding roots before invocation returns.
pub(super) fn prepare_managed(
    input: &neoclr::Module,
    root: &str,
    details: &crate::fault_details::Options,
    host: &std::path::Path,
    target: crate::compiler::Target,
) -> Result<PreparedManaged, Error> {
    match std::env::var("NEOCLR_JIT_LAZY").as_deref() {
        Ok("1") => return lazy::prepare(input, root, details, host, target),
        Ok("0") | Err(_) => {}
        _ => return Err("NEOCLR_JIT_LAZY must be 0 or 1".into()),
    }
    prepare_inner(input, root, details, host, target)
}

fn new_managed_module(
    lazy: bool,
    details: &crate::fault_details::Options,
    host: &std::path::Path,
    target: crate::compiler::Target,
) -> Result<(CodeOwner, std::sync::Arc<libloading::Library>, ManagedHost), Error> {
    if target != host_target()? {
        return Err("JIT target must match the executing host".into());
    }
    if !details.native_gc
        || !details.reference_arena
        || !details.probe_stack_roots
        || !details.native_stack_budget
        || details.native_host_bootstrap
    {
        return Err("managed JIT requires a GC/stack host without bootstrap services".into());
    }
    // SAFETY: this explicit development host path is trusted native code, like a
    // statically linked AOT adapter. Keep the library alive longer than JIT code.
    let library = std::sync::Arc::new(unsafe { libloading::Library::new(host) }?);
    if details.experimental_live_gc_stores {
        // Check the explicit host before binding or executing generated code.
        unsafe { crate::mutation_admission::require_managed_host(&library) }?;
    }
    let run: libloading::Symbol<ManagedHost> = unsafe { library.get(b"neoclr_jit_run_v1") }?;
    let mut flags = crate::backend_settings();
    flags.set("is_pic", "false")?;
    flags.set("use_colocated_libcalls", "false")?;
    target.configure_stack(&mut flags)?;
    let isa = isa::lookup(target.triple().parse().expect("fixed triple"))?
        .finish(settings::Flags::new(flags))?;
    let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
    builder.symbol("neoclr_lazy_resolve_v1", lazy::resolve as *const u8);
    let symbols = library.clone();
    builder.symbol_lookup_fn(Box::new(move |name| unsafe {
        symbols
            .get::<*const u8>(name.as_bytes())
            .ok()
            .map(|symbol| *symbol)
    }));
    let run = *run;
    Ok((
        CodeOwner(Some(if lazy {
            crate::jit_memory::new_lazy_module(builder)
        } else {
            crate::jit_memory::new_module(builder)
        })),
        library,
        run,
    ))
}

fn validate_imports(module: &JITModule, library: &libloading::Library) -> Result<(), Error> {
    // Bind every declared import explicitly. Missing helpers fail before any guest
    // code runs; do not let default process-symbol lookup hide a host mismatch.
    for (_, declaration) in module.declarations().get_functions() {
        if declaration.name.as_deref() == Some("neoclr_lazy_resolve_v1") {
            continue;
        }
        if declaration.linkage == cranelift_module::Linkage::Import {
            let name = declaration.name.as_ref().ok_or("anonymous native import")?;
            let symbol: libloading::Symbol<*const u8> = unsafe { library.get(name.as_bytes()) }
                .map_err(|error| {
                    format!("native host is missing required symbol {name}: {error}")
                })?;
            let _ = *symbol;
        }
    }
    Ok(())
}

fn finalize_entry(module: &mut JITModule) -> Result<(*const u8, i32), Error> {
    module.finalize_definitions()?;
    let (entry, with_args) = match module.get_name("neoclr_entry_args_v1") {
        Some(FuncOrDataId::Func(id)) => (id, 1),
        _ => match module.get_name("neoclr_entry_v4") {
            Some(FuncOrDataId::Func(id)) => (id, 0),
            _ => return Err("managed JIT entry ABI missing".into()),
        },
    };
    let entry = module.get_finalized_function(entry);
    Ok((entry, with_args))
}

fn prepare_inner(
    input: &neoclr::Module,
    root: &str,
    details: &crate::fault_details::Options,
    host: &std::path::Path,
    target: crate::compiler::Target,
) -> Result<PreparedManaged, Error> {
    let started = Instant::now();
    let (mut owner, library, run) = new_managed_module(false, details, host, target)?;
    let module = owner.0.as_mut().expect("owned module");
    crate::compiler::values::compile_into(input, root, Some(details), target, module, true)?;
    validate_imports(module, &library)?;
    let (entry, with_args) = finalize_entry(module)?;
    Ok(PreparedManaged {
        worker: None,
        _code: owner,
        _library: library,
        run,
        entry,
        with_args,
        target,
        compile_seconds: started.elapsed().as_secs_f64(),
    })
}

impl PreparedManaged {
    /// Consumes the owner; code and native libraries outlive synchronous cleanup.
    pub(super) fn invoke(
        mut self,
        arguments: &[std::ffi::CString],
    ) -> Result<serde_json::Value, Error> {
        // The Windows AOT argument helper consumes UTF-16; other hosts consume UTF-8.
        // Keep backing storage alive through the synchronous host invocation.
        #[cfg(windows)]
        let wide_arguments: Vec<Vec<u16>> = arguments
            .iter()
            .map(|arg| Ok(arg.to_str()?.encode_utf16().chain([0]).collect()))
            .collect::<Result<_, std::str::Utf8Error>>()?;
        #[cfg(windows)]
        let argv: Vec<_> = wide_arguments.iter().map(|arg| arg.as_ptr()).collect();
        #[cfg(not(windows))]
        let argv: Vec<_> = arguments.iter().map(|arg| arg.as_ptr()).collect();
        let argc = i32::try_from(argv.len())?;
        let started = Instant::now();
        // SAFETY: only the validated entry ABI is passed. argv and the helper library
        // remain alive; the synchronous host must complete cleanup before returning.
        let _active = lazy::Active::enter(self.worker.as_ref())?;
        let exit_code = unsafe { (self.run)(self.entry, self.with_args, argc, argv.as_ptr()) };
        let invocation_seconds = started.elapsed().as_secs_f64();
        drop(_active);
        if let Some(worker) = &mut self.worker {
            worker.finish()?;
        }
        Ok(
            serde_json::json!({"profile": match self.target { crate::compiler::Target::MacosArm64 => "macos-arm64-jit-managed-v1", crate::compiler::Target::WindowsX64 => "windows-x64-jit-managed-v1" }, "guestExitCode": exit_code,
            "compileAndFinalizeSeconds": self.compile_seconds, "invocationSeconds": invocation_seconds,
            "lazyJit": self.worker.as_ref().map(lazy::Worker::report)}),
        )
    }
}
