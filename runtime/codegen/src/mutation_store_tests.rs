use super::*;
use cranelift_codegen::settings::Configurable;
use cranelift_codegen::{isa, settings};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_object::{ObjectBuilder, ObjectModule};
use object::{Object, ObjectSymbol};

fn target() -> &'static str {
    if cfg!(target_os = "windows") {
        "x86_64-pc-windows-msvc"
    } else {
        "aarch64-apple-darwin"
    }
}
fn isa(triple: &str, pic: bool) -> std::sync::Arc<dyn isa::TargetIsa> {
    let mut flags = settings::builder();
    flags
        .set("is_pic", if pic { "true" } else { "false" })
        .unwrap();
    isa::lookup(triple.parse().unwrap())
        .unwrap()
        .finish(settings::Flags::new(flags))
        .unwrap()
}
fn function(
    module: &mut impl Module,
    name: &str,
    ty: ir::Type,
) -> (
    cranelift_module::FuncId,
    cranelift_codegen::Context,
    ir::Value,
) {
    let mut ctx = module.make_context();
    ctx.func
        .signature
        .params
        .extend([types::I64; 3].map(AbiParam::new));
    ctx.func.signature.returns.push(AbiParam::new(types::I32));
    let id = module
        .declare_function(name, Linkage::Export, &ctx.func.signature)
        .unwrap();
    let mut fb = FunctionBuilderContext::new();
    let context;
    {
        let mut b = FunctionBuilder::new(&mut ctx.func, &mut fb);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let args = b.block_params(entry).to_vec();
        context = args[0];
        let value = match ty {
            types::F64 => b.ins().bitcast(types::F64, MemFlags::new(), args[2]),
            types::F32 => {
                let bits = b.ins().ireduce(types::I32, args[2]);
                b.ins().bitcast(types::F32, MemFlags::new(), bits)
            }
            types::I64 => args[2],
            _ => b.ins().ireduce(ty, args[2]),
        };
        b.ins().store(MemFlags::new(), value, args[1], 8);
        b.ins().store(MemFlags::new(), value, args[1], 16);
        let zero = b.ins().iconst(types::I32, 0);
        b.ins().return_(&[zero]);
        b.seal_all_blocks();
        b.finalize();
    }
    (id, ctx, context)
}
#[repr(C)]
struct State {
    calls: u32,
    reject_at: u32,
    widths: [u32; 2],
    bits: [u64; 2],
}
unsafe extern "C" fn hook(context: *mut State, destination: *mut u8, bits: u64, width: u32) -> i32 {
    let state = unsafe { &mut *context };
    let i = state.calls as usize;
    state.calls += 1;
    state.widths[i] = width;
    state.bits[i] = bits;
    if state.calls == state.reject_at {
        return 3;
    }
    // Test host is little-endian (macOS ARM64 or Windows x64).
    let bytes = bits.to_le_bytes();
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), destination, width as usize);
    }
    0
}
struct Code(Option<JITModule>);
impl Drop for Code {
    fn drop(&mut self) {
        if let Some(module) = self.0.take() {
            // No generated pointer escapes the test or survives execution.
            unsafe { module.free_memory() };
        }
    }
}
#[test]
fn scalar_bits_and_fault_exit_execute_in_jit() {
    let mut builder = JITBuilder::with_isa(
        isa(target(), false),
        cranelift_module::default_libcall_names(),
    );
    builder.symbol("experiment_gc_store", hook as *const u8);
    let mut code = Code(Some(JITModule::new(builder)));
    let module = code.0.as_mut().unwrap();
    let cases = [
        types::I8,
        types::I16,
        types::I32,
        types::I64,
        types::F32,
        types::F64,
    ];
    let mut ids = vec![];
    for ty in cases {
        let (id, mut ctx, context) = function(module, &format!("mutation_{ty}"), ty);
        assert_eq!(
            lower(module, &mut ctx.func, context).unwrap(),
            Lowered {
                stores: 2,
                stack_stores: 0
            }
        );
        assert!(!ctx
            .func
            .layout
            .blocks()
            .flat_map(|b| ctx.func.layout.block_insts(b))
            .any(|i| ctx.func.dfg.insts[i].opcode() == Opcode::Store));
        module.define_function(id, &mut ctx).unwrap();
        ids.push(id);
    }
    module.finalize_definitions().unwrap();
    for (ty, id) in cases.into_iter().zip(ids) {
        let entry = module.get_finalized_function(id);
        let call: unsafe extern "C" fn(*mut State, *mut u8, u64) -> i32 =
            unsafe { std::mem::transmute(entry) };
        for bits in [
            0,
            u64::MAX,
            0x8000000000000000,
            0x80000000,
            0x7ff8000000000123,
            0x7fc00123,
        ] {
            for reject_at in [0, 1, 2] {
                let mut state = State {
                    calls: 0,
                    reject_at,
                    widths: [0; 2],
                    bits: [0; 2],
                };
                let mut output = [0xa5u8; 32];
                let status = unsafe { call(&mut state, output.as_mut_ptr(), bits) };
                assert_eq!(status, if reject_at == 0 { 0 } else { 3 });
                let calls = if reject_at == 1 { 1 } else { 2 };
                assert_eq!(state.calls, calls);
                for i in 0..calls as usize {
                    assert_eq!(state.widths[i], ty.bytes());
                    let mask = if ty.bits() == 64 {
                        u64::MAX
                    } else {
                        (1u64 << ty.bits()) - 1
                    };
                    assert_eq!(state.bits[i], bits & mask);
                }
                let mut expected = [0xa5u8; 32];
                for (i, at) in [8, 16].into_iter().enumerate() {
                    if reject_at == 0 || (i as u32) + 1 < reject_at {
                        expected[at..at + ty.bytes() as usize]
                            .copy_from_slice(&bits.to_le_bytes()[..ty.bytes() as usize]);
                    }
                }
                assert_eq!(output, expected);
            }
        }
    }
}
#[test]
fn emits_aot_hook_import_for_both_targets() {
    for triple in ["aarch64-apple-darwin", "x86_64-pc-windows-msvc"] {
        let mut module = ObjectModule::new(
            ObjectBuilder::new(
                isa(triple, true),
                "mutation",
                cranelift_module::default_libcall_names(),
            )
            .unwrap(),
        );
        for ty in [
            types::I8,
            types::I16,
            types::I32,
            types::I64,
            types::F32,
            types::F64,
        ] {
            let (id, mut ctx, context) = function(&mut module, &format!("mutation_{ty}"), ty);
            lower(&mut module, &mut ctx.func, context).unwrap();
            crate::mutation_admission::check_call_closure(
                &module,
                &[(id, &ctx.func)],
                &live_providers(&module),
            )
            .unwrap();
            module.define_function(id, &mut ctx).unwrap();
        }
        let (id, mut ctx, context, frame, _, ready) = framed_function(&mut module, false);
        lower_framed(&mut module, &mut ctx.func, context, ready).unwrap();
        finish_native_frame(&mut module, &mut ctx.func, frame);
        crate::mutation_admission::check_call_closure(
            &module,
            &[(id, &ctx.func)],
            &live_providers(&module),
        )
        .unwrap();
        module.define_function(id, &mut ctx).unwrap();
        let bytes = module.finish().emit().unwrap();
        let object = object::File::parse(bytes.as_slice()).unwrap();
        let imports: std::collections::BTreeSet<_> = object
            .symbols()
            .filter(|symbol| symbol.is_undefined())
            .map(|symbol| symbol.name().unwrap().trim_start_matches('_').to_owned())
            .collect();
        assert_eq!(
            imports,
            ["experiment_gc_store", "frame_enter", "frame_leave"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        );
        if triple == target() {
            if let Some(path) = std::env::var_os("NEOCLR_MUTATION_TEST_OBJECT") {
                std::fs::write(path, &bytes).unwrap();
            }
        }
    }
}
#[test]
fn rejects_unadmitted_store_before_editing() {
    let mut module = ObjectModule::new(
        ObjectBuilder::new(
            isa(target(), false),
            "reject",
            cranelift_module::default_libcall_names(),
        )
        .unwrap(),
    );
    let (_, mut ctx, context) = function(&mut module, "reject", types::I64);
    let insts: Vec<_> = ctx
        .func
        .layout
        .blocks()
        .flat_map(|b| ctx.func.layout.block_insts(b))
        .filter(|&i| ctx.func.dfg.insts[i].opcode() == Opcode::Store)
        .collect();
    if let ir::InstructionData::Store { flags, .. } = &mut ctx.func.dfg.insts[insts[1]] {
        *flags = flags.with_endianness(ir::Endianness::Big);
    }
    let before = ctx.func.display().to_string();
    assert!(lower(&mut module, &mut ctx.func, context)
        .unwrap_err()
        .to_string()
        .contains("explicit-endian"));
    assert_eq!(ctx.func.display().to_string(), before);
}

#[test]
#[ignore = "requires NEOCLR_MUTATION_TEST_HOST built by concurrent-store/validate.py"]
fn native_collector_jit_consumer() {
    let path = std::env::var_os("NEOCLR_MUTATION_TEST_HOST").expect("native host fixture");
    // The fixture is built from this checkout; its library outlives generated code.
    let library = unsafe { libloading::Library::new(path) }.unwrap();
    unsafe { crate::mutation_admission::require_host(&library) }.unwrap();
    type Hook = unsafe extern "C" fn(*mut std::ffi::c_void, *mut u8, u64, u32) -> i32;
    type Writer = unsafe extern "C" fn(*mut std::ffi::c_void, *mut u8, u64) -> i32;
    type Run = unsafe extern "C" fn(Writer) -> i32;
    let hook: libloading::Symbol<Hook> = unsafe { library.get(b"experiment_gc_store") }.unwrap();
    let run: libloading::Symbol<Run> = unsafe { library.get(b"native_store_consumer") }.unwrap();
    type Enter =
        unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    type Leave = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void);
    let enter: libloading::Symbol<Enter> = unsafe { library.get(b"frame_enter") }.unwrap();
    let leave: libloading::Symbol<Leave> = unsafe { library.get(b"frame_leave") }.unwrap();
    let mut builder = JITBuilder::with_isa(
        isa(target(), false),
        cranelift_module::default_libcall_names(),
    );
    builder.symbol("experiment_gc_store", *hook as *const u8);
    builder.symbol("frame_enter", *enter as *const u8);
    builder.symbol("frame_leave", *leave as *const u8);
    let mut code = Code(Some(JITModule::new(builder)));
    let module = code.0.as_mut().unwrap();
    let (id, mut ctx, context, frame, _, ready) = framed_function(module, false);
    lower_framed(module, &mut ctx.func, context, ready).unwrap();
    finish_native_frame(module, &mut ctx.func, frame);
    crate::mutation_admission::check_call_closure(
        module,
        &[(id, &ctx.func)],
        &live_providers(module),
    )
    .unwrap();
    module.define_function(id, &mut ctx).unwrap();
    module.finalize_definitions().unwrap();
    let call: Writer = unsafe { std::mem::transmute(module.get_finalized_function(id)) };
    assert_eq!(unsafe { run(call) }, 0);
}

// This mirrors argument/local copies emitted before a GC frame is linked: they
// use generic stores through stack_addr, not the dedicated stack_store opcode.
fn stack_copy(
    module: &mut impl Module,
    displacement: i64,
) -> (
    cranelift_module::FuncId,
    cranelift_codegen::Context,
    ir::Value,
) {
    let (id, mut ctx, context) = function(module, "stack_copy", types::I64);
    let stores: Vec<_> = ctx
        .func
        .layout
        .blocks()
        .flat_map(|block| ctx.func.layout.block_insts(block))
        .filter(|&inst| ctx.func.dfg.insts[inst].opcode() == Opcode::Store)
        .collect();
    let slot = ctx.func.create_sized_stack_slot(ir::StackSlotData::new(
        ir::StackSlotKind::ExplicitSlot,
        32,
        3,
    ));
    let mut cursor = FuncCursor::new(&mut ctx.func).at_inst(stores[0]);
    let base = cursor.ins().stack_addr(types::I64, slot, 4);
    let address = cursor.ins().iadd_imm(base, displacement);
    if let ir::InstructionData::Store { args, .. } = &mut cursor.func.dfg.insts[stores[0]] {
        args[1] = address;
    }
    let mut cursor = FuncCursor::new(&mut ctx.func).at_inst(stores[1]);
    let loaded = cursor.ins().load(types::I64, MemFlags::new(), address, 8);
    if let ir::InstructionData::Store { args, .. } = &mut cursor.func.dfg.insts[stores[1]] {
        args[0] = loaded;
    }
    (id, ctx, context)
}

#[test]
fn proven_stack_copy_stays_direct_and_byref_remains_checked() {
    let mut builder = JITBuilder::with_isa(
        isa(target(), false),
        cranelift_module::default_libcall_names(),
    );
    builder.symbol("experiment_gc_store", hook as *const u8);
    let mut code = Code(Some(JITModule::new(builder)));
    let module = code.0.as_mut().unwrap();
    let (id, mut ctx, context) = stack_copy(module, 4);
    assert_eq!(
        lower(module, &mut ctx.func, context).unwrap(),
        Lowered {
            stores: 1,
            stack_stores: 1
        }
    );
    module.define_function(id, &mut ctx).unwrap();
    module.finalize_definitions().unwrap();
    let call: unsafe extern "C" fn(*mut State, *mut u8, u64) -> i32 =
        unsafe { std::mem::transmute(module.get_finalized_function(id)) };
    for reject_at in [0, 1] {
        let mut state = State {
            calls: 0,
            reject_at,
            widths: [0; 2],
            bits: [0; 2],
        };
        let mut output = [0xa5; 32];
        let bits = 0xfedcba9876543210u64;
        assert_eq!(
            unsafe { call(&mut state, output.as_mut_ptr(), bits) },
            if reject_at == 0 { 0 } else { 3 }
        );
        assert_eq!(state.calls, 1);
        assert_eq!(state.bits[0], bits);
        let mut expected = [0xa5; 32];
        if reject_at == 0 {
            expected[16..24].copy_from_slice(&bits.to_le_bytes());
        }
        assert_eq!(output, expected);
    }
}

#[test]
fn invalid_proven_stack_extent_rejected_without_edits() {
    for displacement in [-13, 13] {
        let mut module = ObjectModule::new(
            ObjectBuilder::new(
                isa(target(), true),
                "bounds",
                cranelift_module::default_libcall_names(),
            )
            .unwrap(),
        );
        let (_, mut ctx, context) = stack_copy(&mut module, displacement);
        let before = ctx.func.display().to_string();
        assert!(lower(&mut module, &mut ctx.func, context)
            .unwrap_err()
            .to_string()
            .contains("exceeds proven stack slot"));
        assert_eq!(ctx.func.display().to_string(), before);
    }
}

#[repr(C)]
struct FramedState {
    stores: State,
    enters: u32,
    leaves: u32,
    active: bool,
}
unsafe extern "C" fn frame_enter(frame: *mut usize, state: *mut FramedState) -> *mut FramedState {
    let state = unsafe { &mut *state };
    assert!(!state.active);
    state.active = true;
    state.enters += 1;
    unsafe { frame.write(state as *mut FramedState as usize) };
    state
}
unsafe extern "C" fn frame_leave(frame: *mut usize) {
    let state = unsafe { &mut *(frame.read() as *mut FramedState) };
    assert!(state.active);
    state.active = false;
    state.leaves += 1;
}
unsafe extern "C" fn state_frame_leave(state: *mut FramedState, frame: *mut usize) {
    assert_eq!(unsafe { frame.read() }, state as usize);
    unsafe { frame_leave(frame) };
}
fn framed_function(
    module: &mut impl Module,
    late: bool,
) -> (
    cranelift_module::FuncId,
    cranelift_codegen::Context,
    ir::Value,
    ir::StackSlot,
    ir::Inst,
    ir::Inst,
) {
    let (id, mut ctx, context) = function(module, "framed", types::I64);
    let stores: Vec<_> = ctx
        .func
        .layout
        .blocks()
        .flat_map(|b| ctx.func.layout.block_insts(b))
        .filter(|&i| ctx.func.dfg.insts[i].opcode() == Opcode::Store)
        .collect();
    let frame = ctx.func.create_sized_stack_slot(ir::StackSlotData::new(
        ir::StackSlotKind::ExplicitSlot,
        112,
        3,
    ));
    let mut signature = module.make_signature();
    signature.params.extend([types::I64; 2].map(AbiParam::new));
    signature.returns.push(AbiParam::new(types::I64));
    let enter = module
        .declare_function("frame_enter", Linkage::Import, &signature)
        .unwrap();
    let enter = module.declare_func_in_func(enter, &mut ctx.func);
    let mut cursor = FuncCursor::new(&mut ctx.func).at_inst(stores[usize::from(late)]);
    let address = cursor.ins().stack_addr(types::I64, frame, 0);
    let call = cursor.ins().call(enter, &[address, context]);
    let state = cursor.func.dfg.inst_results(call)[0];
    let ready = cursor.ins().stack_store(state, frame, 104);
    (id, ctx, context, frame, call, ready)
}
#[test]
fn framed_fault_and_normal_returns_use_production_cleanup() {
    for state_handle in [false, true] {
        let mut builder = JITBuilder::with_isa(
            isa(target(), false),
            cranelift_module::default_libcall_names(),
        );
        builder.symbol("experiment_gc_store", hook as *const u8);
        builder.symbol("frame_enter", frame_enter as *const u8);
        builder.symbol(
            "frame_leave",
            if state_handle {
                state_frame_leave as *const u8
            } else {
                frame_leave as *const u8
            },
        );
        let mut code = Code(Some(JITModule::new(builder)));
        let module = code.0.as_mut().unwrap();
        let (id, mut ctx, context, frame, _, ready) = framed_function(module, false);
        assert_eq!(
            lower_framed(module, &mut ctx.func, context, ready)
                .unwrap()
                .stores,
            2
        );
        let mut signature = module.make_signature();
        signature.params.extend(vec![
            AbiParam::new(types::I64);
            if state_handle { 2 } else { 1 }
        ]);
        let leave = module
            .declare_function("frame_leave", Linkage::Import, &signature)
            .unwrap();
        crate::frame_cleanup::finish(module, &mut ctx.func, frame, leave, state_handle);
        module.define_function(id, &mut ctx).unwrap();
        module.finalize_definitions().unwrap();
        let call: unsafe extern "C" fn(*mut FramedState, *mut u8, u64) -> i32 =
            unsafe { std::mem::transmute(module.get_finalized_function(id)) };
        for reject_at in [0, 1, 2] {
            let mut state = FramedState {
                stores: State {
                    calls: 0,
                    reject_at,
                    widths: [0; 2],
                    bits: [0; 2],
                },
                enters: 0,
                leaves: 0,
                active: false,
            };
            let mut output = [0; 32];
            assert_eq!(
                unsafe { call(&mut state, output.as_mut_ptr(), 42) },
                if reject_at == 0 { 0 } else { 3 }
            );
            assert_eq!((state.enters, state.leaves, state.active), (1, 1, false));
            assert_eq!(state.stores.calls, if reject_at == 1 { 1 } else { 2 });
        }
    }
}
#[test]
fn frame_initialization_must_dominate_stores_and_returns() {
    for bypass in [false, true] {
        let mut module = ObjectModule::new(
            ObjectBuilder::new(
                isa(target(), true),
                "framed",
                cranelift_module::default_libcall_names(),
            )
            .unwrap(),
        );
        let (_, mut ctx, context, _, call, ready) = framed_function(&mut module, !bypass);
        if bypass {
            let entry = ctx.func.layout.entry_block().unwrap();
            // Branch around frame initialization, leaving an unframed return.
            let initialized = ctx.func.dfg.make_block();
            ctx.func.layout.split_block(initialized, call);
            let unframed = ctx.func.dfg.make_block();
            ctx.func.layout.append_block(unframed);
            let condition = ctx.func.dfg.block_params(entry)[2];
            FuncCursor::new(&mut ctx.func).at_bottom(entry).ins().brif(
                condition,
                initialized,
                &[],
                unframed,
                &[],
            );
            let mut cursor = FuncCursor::new(&mut ctx.func).at_bottom(unframed);
            let zero = cursor.ins().iconst(types::I32, 0);
            cursor.ins().return_(&[zero]);
        }
        let before = ctx.func.display().to_string();
        assert!(lower_framed(&mut module, &mut ctx.func, context, ready)
            .unwrap_err()
            .to_string()
            .contains("must dominate"));
        assert_eq!(ctx.func.display().to_string(), before);
    }
}

fn finish_native_frame(module: &mut impl Module, func: &mut ir::Function, frame: ir::StackSlot) {
    let mut signature = module.make_signature();
    signature.params.extend([types::I64; 2].map(AbiParam::new));
    let leave = module
        .declare_function("frame_leave", Linkage::Import, &signature)
        .unwrap();
    crate::frame_cleanup::finish(module, func, frame, leave, true);
}

fn live_providers(module: &impl Module) -> Vec<(&'static str, ir::Signature)> {
    let make = |parameters: &[ir::Type], returns: &[ir::Type]| {
        let mut signature = module.make_signature();
        signature
            .params
            .extend(parameters.iter().copied().map(AbiParam::new));
        signature
            .returns
            .extend(returns.iter().copied().map(AbiParam::new));
        signature
    };
    vec![
        (
            "experiment_gc_store",
            make(
                &[types::I64, types::I64, types::I64, types::I32],
                &[types::I32],
            ),
        ),
        (
            "frame_enter",
            make(&[types::I64, types::I64], &[types::I64]),
        ),
        ("frame_leave", make(&[types::I64, types::I64], &[])),
    ]
}

#[test]
fn call_closure_requires_transitive_bodies_and_exact_providers() {
    let mut module = ObjectModule::new(
        ObjectBuilder::new(
            isa(target(), true),
            "closure",
            cranelift_module::default_libcall_names(),
        )
        .unwrap(),
    );
    let (callee, mut callee_ctx, context) = function(&mut module, "callee", types::I64);
    lower(&mut module, &mut callee_ctx.func, context).unwrap();
    let (caller, mut caller_ctx, context) = function(&mut module, "caller", types::I64);
    lower(&mut module, &mut caller_ctx.func, context).unwrap();
    let entry = caller_ctx.func.layout.entry_block().unwrap();
    let args = caller_ctx.func.dfg.block_params(entry).to_vec();
    let target = module.declare_func_in_func(callee, &mut caller_ctx.func);
    let first = caller_ctx.func.layout.first_inst(entry).unwrap();
    let call = FuncCursor::new(&mut caller_ctx.func)
        .at_inst(first)
        .ins()
        .call(target, &args);
    let providers = live_providers(&module);
    let check = crate::mutation_admission::check_call_closure;
    assert!(check(&module, &[(caller, &caller_ctx.func)], &providers)
        .unwrap_err()
        .to_string()
        .contains("no supplied body"));
    let bodies = [(caller, &caller_ctx.func), (callee, &callee_ctx.func)];
    check(&module, &bodies, &providers).unwrap();
    assert!(check(&module, &bodies, &[])
        .unwrap_err()
        .to_string()
        .contains("no admitted provider"));
    let mut wrong = live_providers(&module);
    wrong[0].1.params[3].value_type = types::I64;
    assert!(check(&module, &bodies, &wrong)
        .unwrap_err()
        .to_string()
        .contains("signature mismatch"));
    assert!(check(
        &module,
        &[(callee, &callee_ctx.func), (callee, &callee_ctx.func)],
        &providers
    )
    .unwrap_err()
    .to_string()
    .contains("duplicate generated"));
    if let ir::InstructionData::Call { opcode, .. } = &mut caller_ctx.func.dfg.insts[call] {
        *opcode = Opcode::ReturnCall;
    }
    assert!(check(
        &module,
        &[(caller, &caller_ctx.func), (callee, &callee_ctx.func)],
        &providers
    )
    .unwrap_err()
    .to_string()
    .contains("tail"));
}

#[test]
fn call_closure_rejects_indirect_calls_and_forged_site_signatures() {
    let mut module = ObjectModule::new(
        ObjectBuilder::new(
            isa(target(), true),
            "calls",
            cranelift_module::default_libcall_names(),
        )
        .unwrap(),
    );
    let (id, mut ctx, context) = function(&mut module, "calls", types::I64);
    lower(&mut module, &mut ctx.func, context).unwrap();
    let entry = ctx.func.layout.entry_block().unwrap();
    let call = ctx
        .func
        .layout
        .block_insts(entry)
        .find(|&i| ctx.func.dfg.insts[i].opcode() == Opcode::Call)
        .unwrap();
    let ir::InstructionData::Call { func_ref, .. } = ctx.func.dfg.insts[call] else {
        unreachable!()
    };
    let signature = ctx.func.dfg.ext_funcs[func_ref].signature;
    ctx.func.dfg.signatures[signature].params[3].value_type = types::I64;
    assert!(crate::mutation_admission::check_call_closure(
        &module,
        &[(id, &ctx.func)],
        &live_providers(&module)
    )
    .unwrap_err()
    .to_string()
    .contains("call-site signature"));
    let args = ctx.func.dfg.block_params(entry).to_vec();
    let signature = ctx.func.signature.clone();
    let signature = ctx.func.import_signature(signature);
    let first = ctx.func.layout.first_inst(entry).unwrap();
    FuncCursor::new(&mut ctx.func)
        .at_inst(first)
        .ins()
        .call_indirect(signature, context, &args);
    assert!(crate::mutation_admission::check_call_closure(
        &module,
        &[(id, &ctx.func)],
        &live_providers(&module)
    )
    .unwrap_err()
    .to_string()
    .contains("indirect"));
}

#[test]
#[ignore = "requires NEOCLR_MUTATION_TEST_HOST_CASES built by concurrent-store/validate.py"]
fn incompatible_live_hosts_rejected_before_binding() {
    let directory = std::path::PathBuf::from(
        std::env::var_os("NEOCLR_MUTATION_TEST_HOST_CASES").expect("host cases"),
    );
    for name in ["missing", "legacy", "partial"] {
        let library =
            unsafe { libloading::Library::new(directory.join(format!("{name}.dylib"))) }.unwrap();
        assert!(
            unsafe { crate::mutation_admission::require_host(&library) }.is_err(),
            "{name}"
        );
    }
    let scalar_host =
        unsafe { libloading::Library::new(directory.join("noboundary.dylib")) }.unwrap();
    unsafe { crate::mutation_admission::require_host(&scalar_host) }.unwrap();
    assert!(unsafe { crate::mutation_admission::require_managed_host(&scalar_host) }.is_err());
}

fn managed_options() -> crate::fault_details::Options {
    crate::fault_details::Options {
        native_gc: true,
        reference_arena: true,
        probe_stack_roots: true,
        experimental_live_gc_stores: true,
        ..Default::default()
    }
}
#[test]
fn ordinary_value_pipeline_lowers_only_when_opted_in() {
    emit_managed(include_str!("../../../benchmarks/execution/concurrent-store/managed.neoil"));
}
#[test]
fn managed_construction_emits_guarded_stores() {
    emit_managed(include_str!("../../../benchmarks/execution/concurrent-store/construction.neoil"));
}
fn emit_managed(source: &str) {
    let input = neoclr::assemble(source).unwrap();
    for enabled in [false, true] {
        let mut module = ObjectModule::new(
            ObjectBuilder::new(
                isa(target(), true),
                "managed",
                cranelift_module::default_libcall_names(),
            )
            .unwrap(),
        );
        let mut options = managed_options();
        options.experimental_live_gc_stores = enabled;
        crate::compiler::values::compile_into(
            &input,
            "Main",
            Some(&options),
            crate::compiler::Target::MacosArm64,
            &mut module,
            false,
        )
        .unwrap();
        if enabled {
            let Some(cranelift_module::FuncOrDataId::Func(writer)) =
                module.get_name("neoclr_value_1")
            else {
                panic!("writer")
            };
            let signature = module
                .declarations()
                .get_function_decl(writer)
                .signature
                .clone();
            let export = module
                .declare_function("live_managed_writer", Linkage::Export, &signature)
                .unwrap();
            let mut context = module.make_context();
            context.func.signature = signature;
            let writer = module.declare_func_in_func(writer, &mut context.func);
            let mut fb = FunctionBuilderContext::new();
            {
                let mut b = FunctionBuilder::new(&mut context.func, &mut fb);
                let entry = b.create_block();
                b.append_block_params_for_function_params(entry);
                b.switch_to_block(entry);
                let arguments = b.block_params(entry).to_vec();
                let call = b.ins().call(writer, &arguments);
                let status = b.inst_results(call)[0];
                b.ins().return_(&[status]);
                b.seal_all_blocks();
                b.finalize();
            }
            module.define_function(export, &mut context).unwrap();
        }
        let bytes = module.finish().emit().unwrap();
        if enabled {
            if let Some(path) = std::env::var_os("NEOCLR_MUTATION_MANAGED_OBJECT") {
                std::fs::write(path, &bytes).unwrap();
            }
        }

        let object = object::File::parse(bytes.as_slice()).unwrap();
        assert_eq!(
            object.symbols().any(|s| s.is_undefined()
                && s.name().unwrap_or("").trim_start_matches('_') == "experiment_gc_store"),
            enabled
        );
    }
    let mut module = ObjectModule::new(
        ObjectBuilder::new(
            isa(target(), true),
            "reject",
            cranelift_module::default_libcall_names(),
        )
        .unwrap(),
    );
    let mut options = managed_options();
    options.native_worker_safepoints = true;
    assert!(crate::compiler::values::compile_into(
        &input,
        "Main",
        Some(&options),
        crate::compiler::Target::MacosArm64,
        &mut module,
        false
    )
    .unwrap_err()
    .to_string()
    .contains("owned published frames"));
}
#[test]
#[ignore = "requires NEOCLR_MUTATION_TEST_HOST built by concurrent-store/validate.py"]
fn managed_pipeline_runs_on_live_collector() {
    run_managed(include_str!("../../../benchmarks/execution/concurrent-store/managed.neoil"));
}
#[test]
#[ignore = "requires construction host built by validate-construction.py"]
fn managed_construction_runs_on_live_collector() {
    run_managed(include_str!("../../../benchmarks/execution/concurrent-store/construction.neoil"));
}
fn run_managed(source: &str) {
    let path = std::env::var_os("NEOCLR_MUTATION_TEST_HOST").expect("host");
    let library = std::sync::Arc::new(unsafe { libloading::Library::new(path) }.unwrap());
    unsafe { crate::mutation_admission::require_managed_host(&library) }.unwrap();
    let mut builder = JITBuilder::with_isa(
        isa(target(), false),
        cranelift_module::default_libcall_names(),
    );
    let resolver = library.clone();
    builder.symbol_lookup_fn(Box::new(move |name| unsafe {
        resolver
            .get::<unsafe extern "C" fn()>(name.as_bytes())
            .ok()
            .map(|symbol| *symbol as *const u8)
    }));
    let mut code = Code(Some(JITModule::new(builder)));
    let module = code.0.as_mut().unwrap();
    let input = neoclr::assemble(source).unwrap();
    crate::compiler::values::compile_into(
        &input,
        "Main",
        Some(&managed_options()),
        crate::compiler::Target::MacosArm64,
        module,
        true,
    )
    .unwrap();
    module.finalize_definitions().unwrap();
    let Some(cranelift_module::FuncOrDataId::Func(id)) = module.get_name("neoclr_value_1") else {
        panic!("managed writer")
    };
    type Writer = unsafe extern "C" fn(
        *mut std::ffi::c_void,
        *mut std::ffi::c_void,
        u64,
        *mut i32,
        *mut std::ffi::c_void,
    ) -> i32;
    type Run = unsafe extern "C" fn(Writer) -> i32;
    let run: libloading::Symbol<Run> = unsafe { library.get(b"native_managed_consumer") }.unwrap();
    let call: Writer = unsafe { std::mem::transmute(module.get_finalized_function(id)) };
    assert_eq!(unsafe { run(call) }, 0);
}
