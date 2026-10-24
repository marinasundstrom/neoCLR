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
            module.define_function(id, &mut ctx).unwrap();
        }
        let bytes = module.finish().emit().unwrap();
        let object = object::File::parse(bytes.as_slice()).unwrap();
        assert!(object.symbols().any(|s| s.is_undefined()
            && s.name().unwrap_or("").trim_start_matches('_') == "experiment_gc_store"));
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
    type Hook = unsafe extern "C" fn(*mut std::ffi::c_void, *mut u8, u64, u32) -> i32;
    type Writer = unsafe extern "C" fn(*mut std::ffi::c_void, *mut u8, u64) -> i32;
    type Run = unsafe extern "C" fn(Writer) -> i32;
    let hook: libloading::Symbol<Hook> = unsafe { library.get(b"experiment_gc_store") }.unwrap();
    let run: libloading::Symbol<Run> = unsafe { library.get(b"native_store_consumer") }.unwrap();
    let mut builder = JITBuilder::with_isa(
        isa(target(), false),
        cranelift_module::default_libcall_names(),
    );
    builder.symbol("experiment_gc_store", *hook as *const u8);
    let mut code = Code(Some(JITModule::new(builder)));
    let module = code.0.as_mut().unwrap();
    let (id, mut ctx, context) = function(module, "live_writer", types::I64);
    lower(module, &mut ctx.func, context).unwrap();
    module.define_function(id, &mut ctx).unwrap();
    module.finalize_definitions().unwrap();
    let call: Writer = unsafe { std::mem::transmute(module.get_finalized_function(id)) };
    assert_eq!(unsafe { run(call) }, 0);
}
