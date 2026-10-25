//! Read-only diagnostic of directly anchored stack arguments in the live closure.
//! This does not admit stores or change emitted code. All callers must be present;
//! address-taken and nonlocal functions are excluded. No cross-call fixed point.
use super::{single_edge_arguments, stack_destination};
use cranelift_codegen::ir::{self, ExternalName, InstructionData, Opcode};
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
                if module
                    .declarations()
                    .get_function_decl(callee)
                    .name
                    .as_deref()
                    != Some("experiment_gc_store")
                {
                    continue;
                }
                guarded += 1;
                let args = args.as_slice(&body.dfg.value_lists);
                if args.len() != 4 {
                    continue;
                }
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
                }
            }
        }
        let mut proofs: Vec<_> = proofs.into_iter().map(|(parameter, bytes)| serde_json::json!({"parameter":parameter,"availableBytes":bytes})).collect();
        proofs.sort_by_key(|p| p["parameter"].as_u64());
        rows.push(serde_json::json!({"id":id.as_u32(),"name":declaration.name,"addressTaken":escaped.contains(&id),"local":declaration.linkage==Linkage::Local,"callSites":incoming.map_or(0,Vec::len),"stackParameters":proofs,"guardedStores":guarded,"candidateStores":eligible}));
    }
    serde_json::json!({"scope":"Diagnostic only: directly anchored stack arguments across the supplied checked closure; no code changed or new contract admitted.","functions":rows})
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
}
