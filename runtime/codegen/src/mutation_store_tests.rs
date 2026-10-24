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
#[test]
fn managed_arrays_emit_grouped_stores() {
    let source = include_str!("../../../benchmarks/execution/concurrent-store/arrays.neoil");
    emit_managed(source);
    let input = neoclr::assemble(&source.replace("stelem Node", "stelem fn<Int32,Int32>")).unwrap();
    let mut module = ObjectModule::new(ObjectBuilder::new(
        isa(target(), true), "reject-array", cranelift_module::default_libcall_names(),
    ).unwrap());
    let error = crate::compiler::values::compile_into(
        &input, "Main", Some(&managed_options()), crate::compiler::Target::MacosArm64,
        &mut module, false,
    ).unwrap_err();
    assert!(error.to_string().contains("qualification for callable arrays"));
}
#[test]
fn managed_text_arrays_emit_grouped_stores() {
    emit_managed(include_str!("../../../benchmarks/execution/concurrent-store/text-arrays.neoil"));
}
#[test]
fn managed_statics_emit_guarded_publication() {
    emit_managed(include_str!("../../../benchmarks/execution/concurrent-store/statics.neoil"));
}
fn managed_options_for(source: &str) -> crate::fault_details::Options {
    let mut options = managed_options();
    if source.starts_with(".module LiveNativeText") {
        options.string_concat.push(0);
        options.string_byte_count.push(1);
        options.string_compare_canonical.push(2);
        options.string_compare_ignore_case.push(3);
        options.string_starts_with_ordinal.push(4);
    }
    options
}
#[test]
fn managed_native_text_emits_guarded_wrappers() {
    let source = include_str!("../../../benchmarks/execution/concurrent-store/native-text.neoil");
    emit_managed(source);
    let input = managed_input(source);
    let mut options = managed_options_for(source);
    options.string_concat.clear();
    options.string_upper.push(0);
    let mut module = ObjectModule::new(ObjectBuilder::new(
        isa(target(), true), "reject-provider", cranelift_module::default_libcall_names(),
    ).unwrap());
    let error = crate::compiler::values::compile_into(
        &input, "Main", Some(&options), crate::compiler::Target::MacosArm64, &mut module, false,
    ).unwrap_err();
    assert!(error.to_string().contains("unaudited native replacement bodies"));
}
fn managed_input(source: &str) -> neoclr::metadata::Module {
    use neoclr::metadata::{Instruction as Op, StaticField, Type, Visibility};
    let mut input = neoclr::assemble(source).unwrap();
    if source.starts_with(".module LiveNativeText") {
        // Exercise production binding validation before value-profile lowering.
        let rows: Vec<_> = input.functions.iter().take(5).enumerate().map(|(i, f)| {
            serde_json::json!({"name": f.name, "compiledIndex": i,
                "definition": {"module": "LiveNativeText", "index": i}})
        }).collect();
        let bindings = crate::bindings::utf8_text(
            &mut input, &serde_json::json!({"functions": rows}),
        ).unwrap();
        assert_eq!(bindings.len(), 5);
        // This direct backend fixture lacks the production closed-world name
        // projection. Give the validated replacement placeholders private names
        // so verifier loading cannot collide with bundled System declarations.
        for (index, name) in [(0, "LiveNativeText.Concat"), (1, "LiveNativeText.ByteCount"), (2, "LiveNativeText.Canonical"), (3, "LiveNativeText.IgnoreCase"), (4, "LiveNativeText.StartsWith")] {
            let original = input.functions[index].name.clone();
            input.functions[index].name = name.into();
            for function in &mut input.functions {
                for op in &mut function.body {
                    if let Op::Call(target) = op {
                        if target.name == original { target.name = name.into(); }
                    }
                }
            }
        }
    }
    if source.starts_with(".module LiveStatics") || source.starts_with(".module LiveInitializers") {
        // Static-field syntax is not exposed by the text assembler. Use the same
        // metadata construction as the existing static-storage compiler tests.
        for index in [1, 2] {
            input.types[index].static_fields.push(StaticField {
                name: "Value".into(), ty: Type::Named("Node".into()),
                visibility: Visibility::Public, readonly: false,
            });
        }
        let first = Type::Named("First".into());
        let second = Type::Named("Second".into());
        let writer = input.functions.iter().position(|f| f.name == "Write").unwrap();
        input.functions[writer].body = vec![
            Op::Arg(2), Op::SetStaticField { owner: first.clone(), index: 0 },
            Op::Arg(2), Op::SetStaticField { owner: second.clone(), index: 0 },
            Op::Arg(0), Op::StaticField { owner: first, index: 0 }, Op::StoreObject(Type::Named("Node".into())),
            Op::Arg(1), Op::StaticField { owner: second, index: 0 }, Op::StoreObject(Type::Named("Node".into())),
            Op::Int(0), Op::Return,
        ];
    }
    if source.starts_with(".module LiveInitializers") {
        let owner = Type::Named("First".into());
        input.types[1].static_fields.push(StaticField {
            name: "Count".into(), ty: Type::Int32,
            visibility: Visibility::Public, readonly: false,
        });
        let initializer = input.functions.iter().position(|f| f.name == "First..cctor").unwrap();
        input.functions[initializer].body = vec![
            Op::StaticField { owner: owner.clone(), index: 1 }, Op::Int(1), Op::Add,
            Op::SetStaticField { owner: owner.clone(), index: 1 },
            Op::Call(neoclr::assembler::parse_function_ref("First::Reenter()").unwrap()), Op::Pop,
            Op::Return,
        ];
        for name in ["Count", "Fail"] {
            input.types[2].static_fields.push(StaticField {
                name: name.into(), ty: Type::Int32,
                visibility: Visibility::Public, readonly: false,
            });
        }
        let owner = Type::Named("Second".into());
        let initializer = input.functions.iter().position(|f| f.name == "Second..cctor").unwrap();
        input.functions[initializer].body = vec![
            Op::StaticField { owner: owner.clone(), index: 1 }, Op::Int(1), Op::Add,
            Op::SetStaticField { owner: owner.clone(), index: 1 },
            Op::StaticField { owner, index: 2 }, Op::BranchFalse(10),
            Op::Int(i32::MAX), Op::Int(1), Op::AddChecked, Op::Pop, Op::Return,
        ];
    }
    input
}
#[test]
fn managed_initializers_emit_guarded_stores() {
    emit_managed(include_str!("../../../benchmarks/execution/concurrent-store/initializers.neoil"));
}
fn emit_managed(source: &str) {
    let input = managed_input(source);
    for enabled in [false, true] {
        let mut module = ObjectModule::new(
            ObjectBuilder::new(
                isa(target(), true),
                "managed",
                cranelift_module::default_libcall_names(),
            )
            .unwrap(),
        );
        let mut options = managed_options_for(source);
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
            let mut exports = vec![(input.functions.iter().position(|f| f.name == "Write").unwrap(), "live_managed_writer")];
            if source.starts_with(".module LiveNativeText") {
                exports.extend([(0, "live_native_concat"), (1, "live_native_count"), (2, "live_native_canonical"), (3, "live_native_ignore_case"), (4, "live_native_starts_with")]);
            }
            for (index, name) in exports {
                let Some(cranelift_module::FuncOrDataId::Func(writer)) =
                    module.get_name(&format!("neoclr_value_{index}"))
                else {
                    panic!("writer")
                };
                let signature = module
                    .declarations()
                    .get_function_decl(writer)
                    .signature
                    .clone();
                let export = module
                    .declare_function(name, Linkage::Export, &signature)
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
        }
        let bytes = module.finish().emit().unwrap();
        if enabled {
            if let Some(path) = std::env::var_os("NEOCLR_MUTATION_MANAGED_OBJECT") {
                std::fs::write(path, &bytes).unwrap();
            }
        }

        let object = object::File::parse(bytes.as_slice()).unwrap();
        if source.starts_with(".module LiveStatics") || source.starts_with(".module LiveInitializers") {
            assert_eq!(object.symbols().any(|s| s.is_undefined()
                && s.name().unwrap_or("").trim_start_matches('_') == "experiment_gc_static_storage_rooted_v1"), enabled);
        }
        if source.contains("stelem") && !source.starts_with(".module LivePrimitiveArrays") {
            let helper = if source.contains("stelem String") {
                "experiment_gc_store_text"
            } else { "experiment_gc_store_record" };
            assert_eq!(object.symbols().any(|s| s.is_undefined()
                && s.name().unwrap_or("").trim_start_matches('_') == helper), enabled);
        }
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
    let mut options = managed_options_for(source);
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
#[test]
#[ignore = "requires array host built by validate-arrays.py"]
fn managed_arrays_run_on_live_collector() {
    run_managed(include_str!("../../../benchmarks/execution/concurrent-store/arrays.neoil"));
}
#[test]
#[ignore = "requires text-array host built by validate-text-arrays.py"]
fn managed_text_arrays_run_on_live_collector() {
    run_managed(include_str!("../../../benchmarks/execution/concurrent-store/text-arrays.neoil"));
}
#[test]
#[ignore = "requires static host built by validate-statics.py"]
fn managed_statics_run_on_live_collector() {
    run_managed(include_str!("../../../benchmarks/execution/concurrent-store/statics.neoil"));
}
#[test]
#[ignore = "requires initializer host built by validate-initializers.py"]
fn managed_initializers_run_on_live_collector() {
    run_managed(include_str!("../../../benchmarks/execution/concurrent-store/initializers.neoil"));
}
#[test]
#[ignore = "requires native-text host built by validate-native-text.py"]
fn managed_native_text_runs_on_live_collector() {
    run_managed(include_str!("../../../benchmarks/execution/concurrent-store/native-text.neoil"));
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
    let input = managed_input(source);
    crate::compiler::values::compile_into(
        &input,
        "Main",
        Some(&managed_options_for(source)),
        crate::compiler::Target::MacosArm64,
        module,
        true,
    )
    .unwrap();
    module.finalize_definitions().unwrap();
    let Some(cranelift_module::FuncOrDataId::Func(id)) = module.get_name(&format!("neoclr_value_{}", input.functions.iter().position(|f| f.name == "Write").unwrap())) else {
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
    if source.starts_with(".module LiveNativeText") {
        type NativeRun = unsafe extern "C" fn(*const u8, *const u8, *const u8, *const u8, *const u8) -> i32;
        let run: libloading::Symbol<NativeRun> = unsafe { library.get(b"native_text_alias_consumer") }.unwrap();
        let pointers: Vec<_> = (0..5).map(|index| {
            let Some(cranelift_module::FuncOrDataId::Func(id)) = module.get_name(&format!("neoclr_value_{index}")) else { panic!("native wrapper") };
            module.get_finalized_function(id)
        }).collect();
        assert_eq!(unsafe { run(pointers[0], pointers[1], pointers[2], pointers[3], pointers[4]) }, 0);
    }
}

fn byte_view_input() -> (neoclr::Module, crate::fault_details::Options) {
    use neoclr::metadata::{MemoryViewKind, Type};
    let mut input = neoclr::assemble(".module LiveByteViews\n.function Main(Int32 value) -> Int32\nldarg value\nret\n.end").unwrap();
    let mut main = input.functions.pop().unwrap();
    main.definition = None;
    for (kind, name) in [(MemoryViewKind::Slice, "ByteSlice"), (MemoryViewKind::ReadOnlySlice, "ReadOnlyByteSlice"), (MemoryViewKind::Memory, "ByteMemory"), (MemoryViewKind::ReadOnlyMemory, "ReadOnlyByteMemory")] {
        let mut ty = crate::memory_views::test_module(kind, Type::Byte).types.remove(0);
        ty.name = name.into();
        ty.definition = None;
        input.types.push(ty);
    }
    let slice = Type::Named("ByteSlice".into());
    let readonly = Type::Named("ReadOnlyByteSlice".into());
    let bytes = Type::ArrayRef(Box::new(Type::Byte));
    let contracts = [
        ("neoCLR.Runtime.ViewCreateSlice", vec![bytes.clone()], slice.clone(), false),
        ("neoCLR.Runtime.ViewCreateSlice", vec![bytes.clone(), Type::Int32, Type::Int32], slice.clone(), false),
        ("neoCLR.Runtime.ViewSlice", vec![slice.clone(), Type::Int32, Type::Int32], slice.clone(), false),
        ("neoCLR.Runtime.ViewGet", vec![slice.clone(), Type::Int32], Type::Byte, false),
        ("neoCLR.Runtime.ViewSet", vec![slice.clone(), Type::Int32, Type::Byte], Type::Void, true),
        ("neoCLR.Runtime.ViewLength", vec![slice], Type::Int32, false),
        ("neoCLR.Runtime.ViewUtf8", vec![Type::String], readonly, false),
        ("neoCLR.Runtime.ViewCreateMemory", vec![bytes.clone()], Type::Named("ByteMemory".into()), false),
        ("neoCLR.Runtime.ViewCreateReadOnlyMemory", vec![bytes], Type::Named("ReadOnlyByteMemory".into()), false),
    ];
    let mut rows = vec![];
    for (index, (name, parameters, returns, no_result)) in contracts.into_iter().enumerate() {
        // Private compiled names avoid rebinding bundled System. The real binding
        // validator checks each reserved source name and closed view signature.
        let compiled = format!("LiveByteViews.Service{index}");
        input.functions.push(serde_json::from_value(serde_json::json!({
            "name":compiled,"parameters":parameters,"returns":returns,
            "no_result":no_result,"impl_flags":4096
        })).unwrap());
        rows.push(serde_json::json!({"name":name,"compiledName":compiled,"compiledIndex":index}));
    }
    assert_eq!(crate::memory_views::bind(&mut input, &serde_json::json!({"functions":rows}), true).unwrap().len(), 9);
    input.functions.push(main);
    let mut options = managed_options();
    options.byte_memory_create = vec![0, 1, 7, 8];
    options.byte_memory_slice = vec![2];
    options.byte_view_get = vec![3];
    options.byte_view_set = vec![4];
    options.byte_memory_length = vec![5];
    options.byte_view_utf8 = vec![6];
    (input, options)
}

fn export_byte_views(module: &mut ObjectModule) {
    for index in 0..9 {
        let Some(cranelift_module::FuncOrDataId::Func(id)) = module.get_name(&format!("neoclr_value_{index}")) else { panic!("view body") };
        let signature = module.declarations().get_function_decl(id).signature.clone();
        let export = module.declare_function(&format!("live_view_{index}"), Linkage::Export, &signature).unwrap();
        let mut context = module.make_context();
        context.func.signature = signature;
        let callee = module.declare_func_in_func(id, &mut context.func);
        let mut fb = FunctionBuilderContext::new();
        let mut b = FunctionBuilder::new(&mut context.func, &mut fb);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let args = b.block_params(entry).to_vec();
        let call = b.ins().call(callee, &args);
        let status = b.inst_results(call)[0];
        b.ins().return_(&[status]);
        b.seal_all_blocks();
        b.finalize();
        module.define_function(export, &mut context).unwrap();
    }
}

#[test]
fn managed_byte_views_emit_guarded_results() {
    let (input, mut options) = byte_view_input();
    for enabled in [false, true] {
        options.experimental_live_gc_stores = enabled;
        let mut module = ObjectModule::new(ObjectBuilder::new(isa(target(), true), "views", cranelift_module::default_libcall_names()).unwrap());
        crate::compiler::values::compile_into(&input,"Main",Some(&options),crate::compiler::Target::MacosArm64,&mut module,false).unwrap();
        export_byte_views(&mut module);
        let bytes = module.finish().emit().unwrap();
        let object = object::File::parse(bytes.as_slice()).unwrap();
        assert_eq!(object.symbols().any(|s| s.is_undefined() && s.name().unwrap_or("").trim_start_matches('_') == "experiment_gc_store"), enabled);
        if enabled {
            if let Some(path) = std::env::var_os("NEOCLR_MUTATION_MANAGED_OBJECT") { std::fs::write(path, bytes).unwrap(); }
        }
    }
}

#[test]
#[ignore = "requires managed byte-view host built by validate-managed-byte-views.py"]
fn managed_byte_views_run_on_live_collector() {
    let library = std::sync::Arc::new(unsafe { libloading::Library::new(std::env::var_os("NEOCLR_MUTATION_TEST_HOST").expect("host")) }.unwrap());
    unsafe { crate::mutation_admission::require_managed_host(&library) }.unwrap();
    let mut builder = JITBuilder::with_isa(isa(target(), false), cranelift_module::default_libcall_names());
    let resolver = library.clone();
    builder.symbol_lookup_fn(Box::new(move |name| unsafe { resolver.get::<unsafe extern "C" fn()>(name.as_bytes()).ok().map(|f| *f as *const u8) }));
    let mut code = Code(Some(JITModule::new(builder)));
    let module = code.0.as_mut().unwrap();
    let (input, options) = byte_view_input();
    crate::compiler::values::compile_into(&input,"Main",Some(&options),crate::compiler::Target::MacosArm64,module,true).unwrap();
    module.finalize_definitions().unwrap();
    let pointers: Vec<_> = (0..9).map(|index| {
        let Some(cranelift_module::FuncOrDataId::Func(id)) = module.get_name(&format!("neoclr_value_{index}")) else { panic!("view body") };
        module.get_finalized_function(id)
    }).collect();
    type Run = unsafe extern "C" fn(*const *const u8) -> i32;
    let run: libloading::Symbol<Run> = unsafe { library.get(b"managed_byte_view_consumer") }.unwrap();
    assert_eq!(unsafe { run(pointers.as_ptr()) }, 0);
}

#[test]
fn managed_primitive_arrays_emit_guarded_stores() {
    emit_managed(include_str!("../../../benchmarks/execution/concurrent-store/primitive-arrays.neoil"));
}
#[test]
#[ignore = "requires primitive-array host built by validate-primitive-arrays.py"]
fn managed_primitive_arrays_run_on_live_collector() {
    run_managed(include_str!("../../../benchmarks/execution/concurrent-store/primitive-arrays.neoil"));
}

fn scoped_slice_input() -> (neoclr::Module, crate::fault_details::Options) {
    let mut input = neoclr::assemble(r#"
.module LiveScopedSlice
.references ()
.function neoCLR.Runtime.StringSliceUtf8(String,Int32,Int32) -> Value
.methodimpl InternalCall
.end
.function Main(Int32 value) -> Int32
ldarg value
ret
.end
"#).unwrap();
    let rows = serde_json::json!({"functions": [{"name": input.functions[0].name,
        "compiledIndex": 0, "definition": {"module": "LiveScopedSlice", "index": 0}}]});
    assert_eq!(crate::bindings::utf8_text(&mut input, &rows).unwrap().len(), 1);
    input.functions[0].name = "ScopedSlice".into();
    let mut options = managed_options();
    options.string_slice_utf8.push(0);
    (input, options)
}

#[test]
fn managed_scoped_slice_emits_guarded_result() {
    let (input, mut options) = scoped_slice_input();
    for enabled in [false, true] {
        options.experimental_live_gc_stores = enabled;
        let mut module = ObjectModule::new(ObjectBuilder::new(isa(target(), true), "slice", cranelift_module::default_libcall_names()).unwrap());
        crate::compiler::values::compile_into(&input,"Main",Some(&options),crate::compiler::Target::MacosArm64,&mut module,false).unwrap();
        let Some(cranelift_module::FuncOrDataId::Func(id)) = module.get_name("neoclr_value_0") else { panic!("slice") };
        let signature = module.declarations().get_function_decl(id).signature.clone();
        let export = module.declare_function("live_scoped_slice", Linkage::Export, &signature).unwrap();
        let mut context = module.make_context();
        context.func.signature = signature;
        let callee = module.declare_func_in_func(id, &mut context.func);
        let mut fb = FunctionBuilderContext::new();
        let mut b = FunctionBuilder::new(&mut context.func, &mut fb);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let args = b.block_params(entry).to_vec();
        let call = b.ins().call(callee, &args);
        let status = b.inst_results(call)[0];
        b.ins().return_(&[status]);
        b.seal_all_blocks();
        b.finalize();
        module.define_function(export, &mut context).unwrap();
        let bytes = module.finish().emit().unwrap();
        let object = object::File::parse(bytes.as_slice()).unwrap();
        let imports: Vec<_> = object.symbols().filter(|s| s.is_undefined()).filter_map(|s| s.name().ok()).collect();
        assert_eq!(imports.iter().any(|s| s.contains("experiment_gc_string_slice")), enabled);
        assert_eq!(imports.iter().any(|s| s.contains("experiment_gc_store")), enabled);
        if enabled {
            if let Some(path) = std::env::var_os("NEOCLR_MUTATION_MANAGED_OBJECT") { std::fs::write(path, bytes).unwrap(); }
        }
    }
}

#[test]
#[ignore = "requires scoped-slice host built by validate-scoped-slice.py"]
fn managed_scoped_slice_runs_on_live_collector() {
    let library = std::sync::Arc::new(unsafe { libloading::Library::new(std::env::var_os("NEOCLR_MUTATION_TEST_HOST").expect("host")) }.unwrap());
    unsafe { crate::mutation_admission::require_managed_host(&library) }.unwrap();
    let mut builder = JITBuilder::with_isa(isa(target(), false), cranelift_module::default_libcall_names());
    let resolver = library.clone();
    builder.symbol_lookup_fn(Box::new(move |name| unsafe { resolver.get::<unsafe extern "C" fn()>(name.as_bytes()).ok().map(|f| *f as *const u8) }));
    let mut code = Code(Some(JITModule::new(builder)));
    let module = code.0.as_mut().unwrap();
    let (input, options) = scoped_slice_input();
    crate::compiler::values::compile_into(&input,"Main",Some(&options),crate::compiler::Target::MacosArm64,module,true).unwrap();
    module.finalize_definitions().unwrap();
    let Some(cranelift_module::FuncOrDataId::Func(id)) = module.get_name("neoclr_value_0") else { panic!("slice") };
    type Run = unsafe extern "C" fn(*const u8) -> i32;
    let run: libloading::Symbol<Run> = unsafe { library.get(b"scoped_slice_consumer") }.unwrap();
    assert_eq!(unsafe { run(module.get_finalized_function(id)) }, 0);
}
