//! First-call machine-code compilation. The worker owns every compiler object;
//! the mutator waits with its published root chain intact and never runs the
//! compiler on its bounded managed stack. No managed pointers cross the channel.
use super::*;
use cranelift_codegen::{
    Context,
    ir::{AbiParam, InstBuilder, MemFlags, types},
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{FuncId, Linkage};
use std::{
    cell::{Cell, RefCell},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
};

pub(super) struct Pending {
    id: FuncId,
    context: Option<Context>,
    failed: bool,
    stack_budget: bool,
    trampoline_bytes: u64,
    slot: Box<AtomicUsize>,
}
thread_local! {
    static CAPTURE: RefCell<Option<Vec<Pending>>> = const { RefCell::new(None) };
    static ACTIVE: Cell<*const Worker> = const { Cell::new(std::ptr::null()) };
}

pub(super) struct Capture(bool);
impl Capture {
    pub(super) fn new(enabled: bool) -> Self {
        if enabled {
            CAPTURE.with(|c| {
                assert!(c.borrow().is_none());
                *c.borrow_mut() = Some(Vec::new());
            });
        }
        Self(enabled)
    }
    pub(super) fn finish(self) -> Vec<Pending> {
        if self.0 {
            CAPTURE.with(|c| c.borrow_mut().take().unwrap())
        } else {
            Vec::new()
        }
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        if self.0 {
            CAPTURE.with(|c| {
                c.borrow_mut().take();
            });
        }
    }
}
pub(crate) fn capture(id: FuncId, context: &mut Context, stack_budget: bool) -> bool {
    CAPTURE.with(|c| {
        let mut capture = c.borrow_mut();
        let Some(bodies) = capture.as_mut() else {
            return false;
        };
        bodies.push(Pending {
            id,
            context: Some(std::mem::replace(context, Context::new())),
            failed: false,
            stack_budget,
            trampoline_bytes: 0,
            slot: Box::new(AtomicUsize::new(0)),
        });
        true
    })
}

pub(super) fn emit_trampolines(
    module: &mut JITModule,
    pending: &mut [Pending],
) -> Result<(), Error> {
    let mut resolver_sig = module.make_signature();
    resolver_sig.params.push(AbiParam::new(types::I64));
    resolver_sig.returns.push(AbiParam::new(types::I64));
    let resolver =
        module.declare_function("neoclr_lazy_resolve_v1", Linkage::Import, &resolver_sig)?;
    for (index, body) in pending.iter_mut().enumerate() {
        let signature = module
            .declarations()
            .get_function_decl(body.id)
            .signature
            .clone();
        // All managed lowering uses the status/out-buffer ABI. Fail closed if a
        // new lowering introduces a different ABI rather than inventing a result.
        if signature.returns != [AbiParam::new(types::I32)] {
            return Err("lazy JIT requires the managed status/out-buffer ABI".into());
        }
        let original = body.id;
        body.id = module.declare_function(
            &format!("neoclr_lazy_body_{index}"),
            Linkage::Local,
            &signature,
        )?;
        let mut context = module.make_context();
        context.func.signature = signature.clone();
        let mut fb = FunctionBuilderContext::new();
        {
            let mut b = FunctionBuilder::new(&mut context.func, &mut fb);
            let entry = b.create_block();
            let cold = b.create_block();
            let ready = b.create_block();
            let failed = b.create_block();
            b.append_block_params_for_function_params(entry);
            b.append_block_param(ready, types::I64);
            b.switch_to_block(entry);
            let args = b.block_params(entry).to_vec();
            let address = b
                .ins()
                .iconst(types::I64, (&*body.slot as *const AtomicUsize) as i64);
            let pointer = b
                .ins()
                .atomic_load(types::I64, MemFlags::trusted(), address);
            b.ins().brif(pointer, ready, &[pointer.into()], cold, &[]);
            b.switch_to_block(cold);
            let id = b.ins().iconst(types::I64, index as i64);
            let resolver = module.declare_func_in_func(resolver, b.func);
            let call = b.ins().call(resolver, &[id]);
            let pointer = b.inst_results(call)[0];
            b.ins().brif(pointer, ready, &[pointer.into()], failed, &[]);
            b.switch_to_block(failed);
            let status = b.ins().iconst(types::I32, 1);
            b.ins().return_(&[status]);
            b.switch_to_block(ready);
            let pointer = b.block_params(ready)[0];
            let signature = b.import_signature(signature);
            let call = b.ins().call_indirect(signature, pointer, &args);
            let result = b.inst_results(call).to_vec();
            b.ins().return_(&result);
            b.seal_all_blocks();
            b.finalize();
        }
        crate::compiler::prepare_calls(&mut context, true);
        module.define_function(original, &mut context)?;
        crate::compiler::values::check_frame(module, &context, body.stack_budget)?;
        body.trampoline_bytes = crate::compiler::values::frame_charge(
            &context,
            module.isa().default_call_conv() == cranelift_codegen::isa::CallConv::WindowsFastcall,
        )?;
    }
    Ok(())
}

enum Request {
    Compile(usize, mpsc::SyncSender<usize>),
    Stop,
}
pub(super) struct Worker {
    requests: mpsc::Sender<Request>,
    thread: Option<thread::JoinHandle<()>>,
    error: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    stats: std::sync::Arc<std::sync::Mutex<Stats>>,
}
#[derive(Default)]
struct Stats {
    total: usize,
    compiled: usize,
    lowered: usize,
    seconds: f64,
    lowering_seconds: f64,
    codegen_seconds: f64,
    module_preparation_seconds: f64,
    trampoline_seconds: f64,
    entry_finalization_seconds: f64,
}
impl Worker {
    pub(super) fn report(&self) -> serde_json::Value {
        let s = self.stats.lock().unwrap();
        serde_json::json!({"selectedMethods": s.total, "compiledMethods": s.compiled, "loweredMethods": s.lowered,
            "uncompiledMethods": s.total - s.compiled, "firstCallCompilationSeconds": s.seconds,
            "firstCallLoweringSeconds": s.lowering_seconds,
            "firstCallCodegenAndPublishSeconds": s.codegen_seconds,
            "modulePreparationSeconds": s.module_preparation_seconds,
            "trampolinePreparationSeconds": s.trampoline_seconds,
            "entryFinalizationSeconds": s.entry_finalization_seconds})
    }
    pub(super) fn finish(&mut self) -> Result<(), Error> {
        let _ = self.requests.send(Request::Stop);
        if let Some(thread) = self.thread.take() {
            if thread.join().is_err() {
                return Err("lazy JIT worker could not complete cleanup".into());
            }
        }
        self.check_error()
    }
    fn check_error(&self) -> Result<(), Error> {
        if let Some(error) = self.error.lock().unwrap().as_ref() {
            return Err(error.clone().into());
        }
        Ok(())
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.requests.send(Request::Stop);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
pub(super) struct Active;
impl Active {
    pub(super) fn enter(worker: Option<&Worker>) -> Result<Self, Error> {
        ACTIVE.with(|a| {
            if !a.get().is_null() {
                return Err("nested managed JIT invocation is unsupported".into());
            }
            a.set(worker.map_or(std::ptr::null(), |w| w));
            Ok(Self)
        })
    }
}
impl Drop for Active {
    fn drop(&mut self) {
        ACTIVE.with(|a| a.set(std::ptr::null()));
    }
}

pub(super) unsafe extern "C" fn resolve(index: usize) -> usize {
    // Never unwind through generated frames. Channel failures produce the normal
    // managed failure status, with the compiler error surfaced after host cleanup.
    std::panic::catch_unwind(|| {
        ACTIVE.with(|a| {
            let pointer = a.get();
            if pointer.is_null() {
                return 0;
            }
            // SAFETY: Active borrows the invocation-owned worker until host cleanup;
            // callbacks are admitted only on that same mutator thread.
            let worker = unsafe { &*pointer };
            let (send, recv) = mpsc::sync_channel(0);
            if worker.requests.send(Request::Compile(index, send)).is_err() {
                return 0;
            }
            recv.recv().unwrap_or(0)
        })
    })
    .unwrap_or(0)
}

pub(super) fn prepare(
    input: &neoclr::Module,
    root: &str,
    details: &crate::fault_details::Options,
    host: &std::path::Path,
    target: crate::compiler::Target,
) -> Result<PreparedManaged, Error> {
    let started = Instant::now();
    let input = input.clone();
    let root = root.to_owned();
    let details = details.clone();
    let host = host.to_owned();
    let (requests, receive) = mpsc::channel();
    let (ready, prepared) = mpsc::sync_channel(0);
    let error = std::sync::Arc::new(std::sync::Mutex::new(None));
    let worker_error = error.clone();
    let stats = std::sync::Arc::new(std::sync::Mutex::new(Stats::default()));
    let worker_stats = stats.clone();
    let thread = thread::Builder::new()
        .name("neoclr-jit".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<(), Error> {
                    let (mut owner, library, run) =
                        super::new_managed_module(&details, &host, target)?;
                    let mut driver = Driver {
                        ready: &ready,
                        receive,
                        library,
                        run,
                        stats: worker_stats,
                        error: worker_error.clone(),
                        preparation_started: Instant::now(),
                    };
                    crate::compiler::values::compile_lazy(
                        &input,
                        &root,
                        Some(&details),
                        target,
                        owner.0.as_mut().unwrap(),
                        &mut driver,
                    )
                }));
            let failure = match result {
                Ok(Ok(())) => None,
                Ok(Err(error)) => Some(error.to_string()),
                Err(_) => Some("lazy JIT compiler worker panicked".into()),
            };
            if let Some(message) = failure {
                *worker_error.lock().unwrap() = Some(message.clone());
                let _ = ready.send(Err(message));
            }
        })?;
    let worker = Worker {
        requests,
        thread: Some(thread),
        error,
        stats,
    };
    let (entry, run, with_args, library) = prepared
        .recv()
        .map_err(|_| "lazy JIT preparation worker failed")??;
    Ok(PreparedManaged {
        worker: Some(worker),
        _code: CodeOwner(None),
        _library: library,
        run,
        entry: entry as *const u8,
        with_args,
        target,
        compile_seconds: started.elapsed().as_secs_f64(),
    })
}

type Ready = Result<(usize, ManagedHost, i32, std::sync::Arc<libloading::Library>), String>;
struct Driver<'a> {
    preparation_started: Instant,
    ready: &'a mpsc::SyncSender<Ready>,
    receive: mpsc::Receiver<Request>,
    library: std::sync::Arc<libloading::Library>,
    run: ManagedHost,
    stats: std::sync::Arc<std::sync::Mutex<Stats>>,
    error: std::sync::Arc<std::sync::Mutex<Option<String>>>,
}
impl crate::compiler::values::LazyDriver<JITModule> for Driver<'_> {
    fn run(
        &mut self,
        module: &mut JITModule,
        methods: &[FuncId],
        lower: &mut dyn FnMut(&mut JITModule, usize) -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.stats.lock().unwrap().module_preparation_seconds =
            self.preparation_started.elapsed().as_secs_f64();
        let started = Instant::now();
        let mut pending: Vec<_> = methods
            .iter()
            .map(|&id| Pending {
                id,
                context: None,
                failed: false,
                stack_budget: true,
                trampoline_bytes: 0,
                slot: Box::new(AtomicUsize::new(0)),
            })
            .collect();
        emit_trampolines(module, &mut pending)?;
        self.stats.lock().unwrap().trampoline_seconds = started.elapsed().as_secs_f64();
        let started = Instant::now();
        super::validate_imports(module, &self.library)?;
        let (entry, with_args) = super::finalize_entry(module)?;
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total = methods.len();
            stats.entry_finalization_seconds = started.elapsed().as_secs_f64();
        }
        if self
            .ready
            .send(Ok((
                entry as usize,
                self.run,
                with_args,
                self.library.clone(),
            )))
            .is_err()
        {
            return Ok(());
        }
        let mut terminal_failure = false;
        while let Ok(request) = self.receive.recv() {
            let Request::Compile(index, response) = request else {
                break;
            };
            // A panic must not unwind the module/slots while the mutator is
            // waiting inside a published trampoline. Keep all executable storage
            // alive until Stop, and refuse further compilation after any failure.
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || -> Result<usize, Error> {
                    if terminal_failure {
                        return Err("lazy compilation is terminally failed".into());
                    }
                    let body = pending.get_mut(index).ok_or("invalid lazy method index")?;
                    let published = body.slot.load(Ordering::Acquire);
                    if published != 0 {
                        return Ok(published);
                    }
                    if body.failed {
                        return Err("lazy method compilation previously failed".into());
                    }
                    body.failed = true;
                    let started = Instant::now();
                    #[cfg(test)]
                    if std::env::var("NEOCLR_TEST_LAZY_FAIL_AFTER")
                        .ok()
                        .and_then(|v| v.parse::<usize>().ok())
                        == Some(self.stats.lock().unwrap().compiled)
                    {
                        return Err("injected lazy publication failure".into());
                    }
                    #[cfg(test)]
                    if std::env::var("NEOCLR_TEST_LAZY_PANIC_AFTER")
                        .ok()
                        .and_then(|v| v.parse::<usize>().ok())
                        == Some(self.stats.lock().unwrap().compiled)
                    {
                        panic!("injected lazy compiler panic");
                    }
                    let capture = Capture::new(true);
                    lower(module, index)?;
                    let mut captured = capture.finish();
                    if captured.len() != 1 || captured[0].id != methods[index] {
                        return Err("lazy lowering must supply exactly the requested method".into());
                    }
                    self.stats.lock().unwrap().lowered += 1;
                    let lowering_seconds = started.elapsed().as_secs_f64();
                    let codegen_started = Instant::now();
                    let mut context = captured.pop().unwrap().context.unwrap();
                    super::validate_imports(module, &self.library)?;
                    module.define_function(body.id, &mut context)?;
                    crate::compiler::values::check_frame(module, &context, body.stack_budget)?;
                    let bytes = crate::compiler::values::frame_charge(
                        &context,
                        module.isa().default_call_conv()
                            == cranelift_codegen::isa::CallConv::WindowsFastcall,
                    )?;
                    if bytes + body.trampoline_bytes > 65536 {
                        return Err(
                            "lazy JIT body plus trampoline exceeds the 64 KiB stack charge".into(),
                        );
                    }
                    module.finalize_definitions()?;
                    let pointer = module.get_finalized_function(body.id) as usize;
                    body.slot.store(pointer, Ordering::Release);
                    let mut stats = self.stats.lock().unwrap();
                    stats.compiled += 1;
                    stats.seconds += started.elapsed().as_secs_f64();
                    stats.lowering_seconds += lowering_seconds;
                    stats.codegen_seconds += codegen_started.elapsed().as_secs_f64();
                    Ok(pointer)
                },
            ))
            .unwrap_or_else(|_| Err("lazy compiler panicked before publication".into()));
            let pointer = match result {
                Ok(pointer) => pointer,
                Err(error) => {
                    terminal_failure = true;
                    self.error
                        .lock()
                        .unwrap()
                        .get_or_insert_with(|| format!("lazy JIT method {index}: {error}"));
                    0
                }
            };
            let _ = response.send(pointer);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64")
    ))]
    fn trampoline_preserves_register_and_stack_arguments_and_failure_status() {
        let target = host_target().unwrap();
        let mut flags = crate::backend_settings();
        flags.set("is_pic", "false").unwrap();
        flags.set("use_colocated_libcalls", "false").unwrap();
        target.configure_stack(&mut flags).unwrap();
        let isa = isa::lookup(target.triple().parse().unwrap())
            .unwrap()
            .finish(settings::Flags::new(flags))
            .unwrap();
        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        builder.symbol("neoclr_lazy_resolve_v1", resolve as *const u8);
        let mut owner = CodeOwner(Some(crate::jit_memory::new_module(builder)));
        let module = owner.0.as_mut().unwrap();
        let mut signature = module.make_signature();
        signature.params.extend([AbiParam::new(types::I64); 12]);
        signature.returns.push(AbiParam::new(types::I32));
        let id = module
            .declare_function("test_lazy_entry", Linkage::Export, &signature)
            .unwrap();
        let mut context = module.make_context();
        context.func.signature = signature;
        let mut fb = FunctionBuilderContext::new();
        {
            let mut b = FunctionBuilder::new(&mut context.func, &mut fb);
            let entry = b.create_block();
            b.append_block_params_for_function_params(entry);
            b.switch_to_block(entry);
            let params = b.block_params(entry).to_vec();
            let mut sum = params[0];
            for &p in &params[1..] {
                sum = b.ins().iadd(sum, p);
            }
            let sum = b.ins().ireduce(types::I32, sum);
            b.ins().return_(&[sum]);
            b.seal_all_blocks();
            b.finalize();
        }
        let capture_guard = Capture::new(true);
        assert!(capture(id, &mut context, true));
        let mut pending = capture_guard.finish();
        assert!(!capture(id, &mut Context::new(), false));
        emit_trampolines(module, &mut pending).unwrap();
        module.finalize_definitions().unwrap();
        type Entry =
            unsafe extern "C" fn(u64, u64, u64, u64, u64, u64, u64, u64, u64, u64, u64, u64) -> i32;
        let entry: Entry = unsafe { std::mem::transmute(module.get_finalized_function(id)) };
        // An unbound cold callback must fail without calling an invalid address.
        assert_eq!(unsafe { entry(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12) }, 1);
        let body = &mut pending[0];
        let mut context = body.context.take().unwrap();
        module.define_function(body.id, &mut context).unwrap();
        module.finalize_definitions().unwrap();
        body.slot.store(
            module.get_finalized_function(body.id) as usize,
            Ordering::Release,
        );
        for _ in 0..8 {
            assert_eq!(unsafe { entry(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12) }, 78);
        }
    }

    #[test]
    fn preparation_does_not_lower_unused_bodies_with_either_collector() {
        use cranelift_object::{ObjectBuilder, ObjectModule};
        struct Inspect;
        impl crate::compiler::values::LazyDriver<ObjectModule> for Inspect {
            fn run(
                &mut self,
                module: &mut ObjectModule,
                methods: &[FuncId],
                lower: &mut dyn FnMut(&mut ObjectModule, usize) -> Result<(), Error>,
            ) -> Result<(), Error> {
                assert_eq!(methods.len(), 3);
                CAPTURE.with(|capture| {
                    let bodies = capture.borrow();
                    assert!(
                        bodies
                            .as_ref()
                            .unwrap()
                            .iter()
                            .all(|body| !methods.contains(&body.id)),
                        "no selected method may be lowered during preparation"
                    );
                });
                // Write calls Put, but Put must remain deferred rather than
                // recursively lowering the entire reachable call graph.
                lower(module, 1)?;
                CAPTURE.with(|capture| {
                    let bodies = capture.borrow();
                    let methods_lowered: Vec<_> = bodies
                        .as_ref()
                        .unwrap()
                        .iter()
                        .filter(|body| methods.contains(&body.id))
                        .map(|body| body.id)
                        .collect();
                    assert_eq!(methods_lowered, [methods[1]]);
                });
                Ok(())
            }
        }
        let input = neoclr::assemble(include_str!(
            "../../../benchmarks/execution/concurrent-store/managed.neoil"
        ))
        .unwrap();
        for live in [false, true] {
            let isa = isa::lookup("aarch64-apple-darwin".parse().unwrap())
                .unwrap()
                .finish(settings::Flags::new(crate::backend_settings()))
                .unwrap();
            let mut module = ObjectModule::new(
                ObjectBuilder::new(
                    isa,
                    "lazy_lowering",
                    cranelift_module::default_libcall_names(),
                )
                .unwrap(),
            );
            let details = crate::fault_details::Options {
                native_gc: true,
                reference_arena: true,
                probe_stack_roots: true,
                experimental_live_gc_stores: live,
                ..Default::default()
            };
            let capture = Capture::new(true);
            crate::compiler::values::compile_lazy(
                &input,
                "Main",
                Some(&details),
                crate::compiler::Target::MacosArm64,
                &mut module,
                &mut Inspect,
            )
            .unwrap();
            capture.finish();
        }
    }

    #[test]
    fn abandoned_capture_does_not_affect_the_next_compilation() {
        {
            let _capture = Capture::new(true);
        }
        assert!(!capture(FuncId::from_u32(0), &mut Context::new(), false));
    }
}
