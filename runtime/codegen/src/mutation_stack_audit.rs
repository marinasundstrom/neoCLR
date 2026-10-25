//! Test-only audit and opt-in guard removal for bounded direct stack arguments.
//! Reporting is read-only; elision requires the complete generated closure.
//! Address-taken and nonlocal functions are excluded. No cross-call fixed point.
use super::{single_edge_arguments, stack_destination};
use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{
    self, ExternalName, InstBuilder, InstructionData, MemFlags, Opcode, types,
};
use cranelift_module::{FuncId, Linkage, Module};
use std::collections::{HashMap, HashSet};

fn target(body: &ir::Function, reference: ir::FuncRef) -> Option<FuncId> {
    let ExternalName::User(name) = body.dfg.ext_funcs[reference].name else {
        return None;
    };
    let name = &body.params.user_named_funcs()[name];
    (name.namespace == 0).then(|| FuncId::from_u32(name.index))
}
fn parameter(
    body: &ir::Function,
    mut value: ir::Value,
    incoming: &HashMap<ir::Value, Option<ir::Value>>,
) -> Option<(usize, i64)> {
    let mut offset = 0i64;
    for _ in 0..body.dfg.num_values() {
        value = body.dfg.resolve_aliases(value);
        match body.dfg.value_def(value) {
            ir::ValueDef::Param(block, index) if Some(block) == body.layout.entry_block() => {
                return Some((index, offset));
            }
            ir::ValueDef::Param(_, _) => value = (*incoming.get(&value)?)?,
            ir::ValueDef::Result(inst, _) => match body.dfg.insts[inst] {
                InstructionData::BinaryImm64 {
                    opcode: Opcode::IaddImm,
                    arg,
                    imm,
                } => {
                    offset = offset.checked_add(i64::from(imm))?;
                    value = arg;
                }
                _ => return None,
            },
            _ => return None,
        }
    }
    None
}

pub(crate) fn report(
    module: &impl Module,
    bodies: &[(FuncId, &ir::Function)],
) -> serde_json::Value {
    analyze(module, bodies).0
}

fn analyze(
    module: &impl Module,
    bodies: &[(FuncId, &ir::Function)],
) -> (serde_json::Value, HashMap<FuncId, Vec<(ir::Inst, i64)>>) {
    let mut sites: HashMap<FuncId, Vec<(ir::Inst, i64)>> = HashMap::new();
    let mut calls: HashMap<FuncId, Vec<Vec<Option<u32>>>> = HashMap::new();
    let mut escaped = HashSet::new();
    for &(_, body) in bodies {
        let incoming = single_edge_arguments(body);
        for block in body.layout.blocks() {
            for inst in body.layout.block_insts(block) {
                match body.dfg.insts[inst] {
                    InstructionData::FuncAddr { func_ref, .. } => {
                        if let Some(id) = target(body, func_ref) {
                            escaped.insert(id);
                        }
                    }
                    InstructionData::Call {
                        opcode: Opcode::Call,
                        func_ref,
                        args,
                    } => {
                        if let Some(id) = target(body, func_ref) {
                            let arguments = args
                                .as_slice(&body.dfg.value_lists)
                                .iter()
                                .map(|&value| {
                                    let (slot, offset) = stack_destination(body, value, &incoming)?;
                                    let offset = u32::try_from(offset).ok()?;
                                    body.sized_stack_slots[slot].size.checked_sub(offset)
                                })
                                .collect();
                            calls.entry(id).or_default().push(arguments);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    let mut rows = Vec::new();
    for &(id, body) in bodies {
        let declaration = module.declarations().get_function_decl(id);
        let incoming = calls.get(&id);
        let excluded =
            declaration.linkage != Linkage::Local || escaped.contains(&id) || incoming.is_none();
        let mut proofs = HashMap::new();
        if !excluded {
            let incoming = incoming.unwrap();
            for index in 0..body.signature.params.len() {
                let sizes: Option<Vec<u32>> = incoming
                    .iter()
                    .map(|args| args.get(index).copied().flatten())
                    .collect();
                if let Some(sizes) = sizes {
                    proofs.insert(index, *sizes.iter().min().unwrap());
                }
            }
        }
        let incoming_edges = single_edge_arguments(body);
        let mut eligible = 0;
        let mut guarded = 0;
        for block in body.layout.blocks() {
            for inst in body.layout.block_insts(block) {
                let InstructionData::Call {
                    opcode: Opcode::Call,
                    func_ref,
                    args,
                } = body.dfg.insts[inst]
                else {
                    continue;
                };
                let Some(callee) = target(body, func_ref) else {
                    continue;
                };
                let name = module.declarations().get_function_decl(callee).name.as_deref();
                let raw = args.as_slice(&body.dfg.value_lists);
                let args = match (name, raw.len()) {
                    (Some("experiment_gc_store"), 4) => raw,
                    (Some("experiment_gc_store_borrowed"), 5) => &raw[1..],
                    _ => continue,
                };
                guarded += 1;
                let Some((index, offset)) = parameter(body, args[1], &incoming_edges) else {
                    continue;
                };
                let Some(&bytes) = proofs.get(&index) else {
                    continue;
                };
                let ir::ValueDef::Result(width, _) =
                    body.dfg.value_def(body.dfg.resolve_aliases(args[3]))
                else {
                    continue;
                };
                let InstructionData::UnaryImm {
                    opcode: Opcode::Iconst,
                    imm,
                } = body.dfg.insts[width]
                else {
                    continue;
                };
                let width = i64::from(imm);
                if matches!(width, 1 | 2 | 4 | 8)
                    && offset >= 0
                    && offset
                        .checked_add(width)
                        .is_some_and(|end| end <= i64::from(bytes))
                {
                    eligible += 1;
                    sites.entry(id).or_default().push((inst, width));
                }
            }
        }
        let mut proofs: Vec<_> = proofs.into_iter().map(|(parameter, bytes)| serde_json::json!({"parameter":parameter,"availableBytes":bytes})).collect();
        proofs.sort_by_key(|p| p["parameter"].as_u64());
        rows.push(serde_json::json!({"id":id.as_u32(),"name":declaration.name,"addressTaken":escaped.contains(&id),"local":declaration.linkage==Linkage::Local,"callSites":incoming.map_or(0,Vec::len),"stackParameters":proofs,"guardedStores":guarded,"candidateStores":eligible}));
    }
    (
        serde_json::json!({"scope":"Diagnostic only: directly anchored stack arguments across the supplied checked closure; no code changed or new contract admitted.","functions":rows}),
        sites,
    )
}

/// Test-only experiment. The caller must supply every generated function before
/// definition and must not embed function addresses in data. values.rs satisfies
/// this for its deferred live closure: all function addresses are explicit IR.
/// Local direct callees execute while their callers' stack slots remain alive.
/// Address-taken callbacks, loaded pointers and unknown callers stay guarded.
pub(crate) fn elide(
    module: &impl Module,
    bodies: &mut [(FuncId, cranelift_codegen::Context)],
) -> Result<usize, Box<dyn std::error::Error>> {
    let refs: Vec<_> = bodies.iter().map(|(id, ctx)| (*id, &ctx.func)).collect();
    crate::mutation_admission::check_call_closure(
        module,
        &refs,
        &crate::mutation_admission::root_providers(module),
    )?;
    let supplied: HashSet<_> = refs.iter().map(|(id, _)| *id).collect();
    if module
        .declarations()
        .get_functions()
        .any(|(id, decl)| decl.linkage != Linkage::Import && !supplied.contains(&id))
    {
        return Err("stack-argument elision requires every generated declaration".into());
    }
    let (_, sites) = analyze(module, &refs);
    let count = sites.values().map(Vec::len).sum();
    for (id, ctx) in bodies {
        for &(inst, width) in sites.get(id).into_iter().flatten() {
            let InstructionData::Call { args, .. } = ctx.func.dfg.insts[inst] else {
                unreachable!()
            };
            let raw = args.as_slice(&ctx.func.dfg.value_lists);
            // analyze admits only the exact four- or five-argument store ABIs.
            let args = if raw.len() == 5 { &raw[1..] } else { raw }.to_vec();
            let mut cursor = FuncCursor::new(&mut ctx.func).at_inst(inst);
            let bits = match width {
                1 => cursor.ins().ireduce(types::I8, args[2]),
                2 => cursor.ins().ireduce(types::I16, args[2]),
                4 => cursor.ins().ireduce(types::I32, args[2]),
                8 => args[2],
                _ => unreachable!(),
            };
            cursor.ins().store(MemFlags::new(), bits, args[1], 0);
            // Preserve the status SSA value and existing root-cleanup control flow.
            cursor.func.dfg.replace(inst).iconst(types::I32, 0);
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cranelift_codegen::ir::{
        AbiParam, InstBuilder, MemFlags, StackSlotData, StackSlotKind, types,
    };
    use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
    use cranelift_object::{ObjectBuilder, ObjectModule};

    fn audit(
        exported: bool,
        addressed: bool,
        dynamic: bool,
        sizes: &[u32],
        offset: i32,
    ) -> serde_json::Value {
        let isa = cranelift_codegen::isa::lookup("aarch64-apple-darwin".parse().unwrap())
            .unwrap()
            .finish(cranelift_codegen::settings::Flags::new(
                cranelift_codegen::settings::builder(),
            ))
            .unwrap();
        let mut module = ObjectModule::new(
            ObjectBuilder::new(isa, "audit", cranelift_module::default_libcall_names()).unwrap(),
        );
        let mut callee = module.make_context();
        callee.func.signature.params = vec![AbiParam::new(types::I64); 2];
        callee
            .func
            .signature
            .returns
            .push(AbiParam::new(types::I32));
        let id = module
            .declare_function(
                "callee",
                if exported {
                    Linkage::Export
                } else {
                    Linkage::Local
                },
                &callee.func.signature,
            )
            .unwrap();
        let context;
        {
            let mut fb = FunctionBuilderContext::new();
            let mut b = FunctionBuilder::new(&mut callee.func, &mut fb);
            let entry = b.create_block();
            b.append_block_params_for_function_params(entry);
            b.switch_to_block(entry);
            let args = b.block_params(entry).to_vec();
            context = args[0];
            let value = b.ins().iconst(types::I64, 123);
            b.ins().store(MemFlags::new(), value, args[1], offset);
            let zero = b.ins().iconst(types::I32, 0);
            b.ins().return_(&[zero]);
            b.seal_all_blocks();
            b.finalize();
        }
        super::super::lower(&mut module, &mut callee.func, context).unwrap();
        let mut caller = module.make_context();
        caller.func.signature = callee.func.signature.clone();
        let caller_id = module
            .declare_function("caller", Linkage::Export, &caller.func.signature)
            .unwrap();
        let reference = module.declare_func_in_func(id, &mut caller.func);
        {
            let mut fb = FunctionBuilderContext::new();
            let mut b = FunctionBuilder::new(&mut caller.func, &mut fb);
            let entry = b.create_block();
            b.append_block_params_for_function_params(entry);
            b.switch_to_block(entry);
            let args = b.block_params(entry).to_vec();
            for &size in sizes {
                let slot = b.create_sized_stack_slot(StackSlotData::new(
                    StackSlotKind::ExplicitSlot,
                    size,
                    3,
                ));
                let address = b.ins().stack_addr(types::I64, slot, 0);
                b.ins().call(reference, &[args[0], address]);
            }
            if dynamic {
                b.ins().call(reference, &[args[0], args[1]]);
            }
            if addressed {
                b.ins().func_addr(types::I64, reference);
            }
            let zero = b.ins().iconst(types::I32, 0);
            b.ins().return_(&[zero]);
            b.seal_all_blocks();
            b.finalize();
        }
        let before = (
            caller.func.display().to_string(),
            callee.func.display().to_string(),
        );
        let result = report(&module, &[(id, &callee.func), (caller_id, &caller.func)]);
        assert_eq!(
            before,
            (
                caller.func.display().to_string(),
                callee.func.display().to_string()
            )
        );
        let expected = result["functions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["candidateStores"].as_u64().unwrap() as usize)
            .sum::<usize>();
        let mut bodies = vec![(id, callee), (caller_id, caller)];
        assert_eq!(elide(&module, &mut bodies).unwrap(), expected);
        for (_, ctx) in &mut bodies {
            cranelift_codegen::verify_function(&ctx.func, module.isa()).unwrap();
        }
        // Applying the pass twice must not add stores or disturb cleanup branches.
        assert_eq!(elide(&module, &mut bodies).unwrap(), 0);
        let unchanged: Vec<_> = bodies
            .iter()
            .map(|(_, c)| c.func.display().to_string())
            .collect();
        module
            .declare_function("missing_body", Linkage::Local, &bodies[0].1.func.signature)
            .unwrap();
        assert!(
            elide(&module, &mut bodies)
                .unwrap_err()
                .to_string()
                .contains("every generated declaration")
        );
        assert_eq!(
            unchanged,
            bodies
                .iter()
                .map(|(_, c)| c.func.display().to_string())
                .collect::<Vec<_>>()
        );
        result["functions"][0].clone()
    }
    #[test]
    fn all_callers_must_provide_bounded_stack_storage() {
        let r = audit(false, false, false, &[32, 16], 8);
        assert_eq!(r["candidateStores"], 1);
        assert_eq!(
            r["stackParameters"],
            serde_json::json!([{"parameter":1,"availableBytes":16}])
        );
        assert_eq!(
            audit(false, false, false, &[32, 8], 8)["candidateStores"],
            0
        );
        assert_eq!(
            audit(false, false, false, &[32, 16], -1)["candidateStores"],
            0
        );
        assert_eq!(audit(false, false, true, &[32], 8)["candidateStores"], 0);
    }
    #[test]
    fn external_address_taken_and_unreferenced_functions_are_excluded() {
        for r in [
            audit(true, false, false, &[32], 8),
            audit(false, true, false, &[32], 8),
            audit(false, false, false, &[], 8),
        ] {
            assert_eq!(r["candidateStores"], 0);
            assert_eq!(r["stackParameters"], serde_json::json!([]));
        }
    }
    #[test]
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    fn jit_preserves_unaligned_bits_and_mixed_pointer_guards() {
        use cranelift_jit::{JITBuilder, JITModule};
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        unsafe extern "C" fn hook(_: u64, destination: *mut u8, bits: u64, width: u32) -> i32 {
            CALLS.fetch_add(1, Ordering::SeqCst);
            unsafe {
                std::ptr::copy_nonoverlapping(
                    bits.to_ne_bytes().as_ptr(),
                    destination,
                    width as usize,
                );
            }
            0
        }
        for width in [1, 2, 4, 8] {
            for mixed in [false, true] {
                for enabled in [false, true] {
                    let isa =
                        cranelift_codegen::isa::lookup("aarch64-apple-darwin".parse().unwrap())
                            .unwrap()
                            .finish(cranelift_codegen::settings::Flags::new(
                                cranelift_codegen::settings::builder(),
                            ))
                            .unwrap();
                    let mut builder =
                        JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
                    extern "C" fn acquire() -> u64 { 0x1234 }
                    unsafe extern "C" fn borrowed(handle: u64, context: u64, destination: *mut u8, bits: u64, width: u32) -> i32 {
                        assert_eq!(handle, 0x1234);
                        unsafe { hook(context, destination, bits, width) }
                    }
                    builder.symbol("experiment_gc_store", hook as *const u8);
                    builder.symbol("experiment_gc_store_state_borrow", acquire as *const u8);
                    builder.symbol("experiment_gc_store_borrowed", borrowed as *const u8);
                    let mut module = JITModule::new(builder);
                    let mut callee = module.make_context();
                    callee.func.signature.params = vec![AbiParam::new(types::I64); 2];
                    callee.func.signature.returns = vec![AbiParam::new(types::I32)];
                    let id = module
                        .declare_function("store", Linkage::Local, &callee.func.signature)
                        .unwrap();
                    let ty = match width {
                        1 => types::I8,
                        2 => types::I16,
                        4 => types::I32,
                        _ => types::I64,
                    };
                    let payload = 0xfedcba9876543210u64;
                    let context;
                    {
                        let mut fb = FunctionBuilderContext::new();
                        let mut b = FunctionBuilder::new(&mut callee.func, &mut fb);
                        let block = b.create_block();
                        b.append_block_params_for_function_params(block);
                        b.switch_to_block(block);
                        let args = b.block_params(block).to_vec();
                        context = args[0];
                        let bits = b.ins().iconst(ty, payload as i64);
                        b.ins().store(MemFlags::new(), bits, args[1], 1);
                        let status = b.ins().iconst(types::I32, 0);
                        b.ins().return_(&[status]);
                        b.seal_all_blocks();
                        b.finalize();
                    }
                    if crate::mutation_admission::borrowed_stores_enabled() {
                        // Isolated IR fixture: the first constant is the stand-in
                        // readiness point. Real generated consumers use linked frames.
                        let ready=callee.func.layout.first_inst(callee.func.layout.entry_block().unwrap()).unwrap();
                        super::super::lower_framed_borrowed(&mut module, &mut callee.func, context, ready).unwrap();
                    } else {
                        super::super::lower(&mut module, &mut callee.func, context).unwrap();
                    }
                    let mut caller = module.make_context();
                    caller.func.signature.params = vec![AbiParam::new(types::I64); 2];
                    caller.func.signature.returns = vec![AbiParam::new(types::I64)];
                    let entry = module
                        .declare_function("entry", Linkage::Export, &caller.func.signature)
                        .unwrap();
                    let target = module.declare_func_in_func(id, &mut caller.func);
                    {
                        let mut fb = FunctionBuilderContext::new();
                        let mut b = FunctionBuilder::new(&mut caller.func, &mut fb);
                        let block = b.create_block();
                        b.append_block_params_for_function_params(block);
                        b.switch_to_block(block);
                        let args = b.block_params(block).to_vec();
                        let slot = b.create_sized_stack_slot(StackSlotData::new(
                            StackSlotKind::ExplicitSlot,
                            16,
                            3,
                        ));
                        let address = b.ins().stack_addr(types::I64, slot, 0);
                        b.ins().call(target, &[args[0], address]);
                        if mixed {
                            b.ins().call(target, &args);
                        }
                        let loaded = b.ins().load(ty, MemFlags::new(), address, 1);
                        let result = if width == 8 {
                            loaded
                        } else {
                            b.ins().uextend(types::I64, loaded)
                        };
                        b.ins().return_(&[result]);
                        b.seal_all_blocks();
                        b.finalize();
                    }
                    let mut bodies = vec![(id, callee), (entry, caller)];
                    if enabled {
                        assert_eq!(elide(&module, &mut bodies).unwrap(), usize::from(!mixed));
                    }
                    for (id, ctx) in &mut bodies {
                        module.define_function(*id, ctx).unwrap();
                    }
                    module.finalize_definitions().unwrap();
                    let run: unsafe extern "C" fn(u64, *mut u8) -> u64 =
                        unsafe { std::mem::transmute(module.get_finalized_function(entry)) };
                    let mut bytes = [0xa5u8; 16];
                    CALLS.store(0, Ordering::SeqCst);
                    let mask = if width == 8 {
                        u64::MAX
                    } else {
                        (1u64 << (width * 8)) - 1
                    };
                    assert_eq!(unsafe { run(0, bytes.as_mut_ptr()) }, payload & mask);
                    assert_eq!(
                        CALLS.load(Ordering::SeqCst),
                        if mixed { 2 } else { usize::from(!enabled) }
                    );
                    if mixed {
                        assert_eq!(&bytes[1..1 + width], &payload.to_ne_bytes()[..width]);
                    }
                    assert_eq!(bytes[0], 0xa5);
                    assert_eq!(bytes[1 + width], 0xa5);
                    unsafe {
                        module.free_memory();
                    }
                }
            }
        }
    }
}
