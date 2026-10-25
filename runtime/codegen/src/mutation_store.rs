//! Experimental scalar-store lowering. Not admission for concurrent execution:
//! native calls, construction and scheduler paths require separate qualification.
use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{self, AbiParam, InstBuilder, MemFlags, Opcode, types};
use cranelift_module::{Linkage, Module};
use std::collections::HashMap;

type Error = Box<dyn std::error::Error>;

#[derive(Debug, PartialEq)]
pub(crate) struct Lowered {
    pub stores: usize,
    pub stack_stores: usize,
}

// Preserve provenance only across a unique incoming edge. Count edges, not
// predecessor blocks: both arms of a branch can pass different addresses.
fn single_edge_arguments(func: &ir::Function) -> HashMap<ir::Value, Option<ir::Value>> {
    let mut incoming = HashMap::new();
    for block in func.layout.blocks() {
        for inst in func.layout.block_insts(block) {
            for edge in func.dfg.insts[inst]
                .branch_destination(&func.dfg.jump_tables, &func.dfg.exception_tables)
            {
                let target = edge.block(&func.dfg.value_lists);
                if Some(target) == func.layout.entry_block() {
                    continue;
                }
                for (&parameter, argument) in func
                    .dfg
                    .block_params(target)
                    .iter()
                    .zip(edge.args(&func.dfg.value_lists))
                {
                    incoming
                        .entry(parameter)
                        .and_modify(|value| *value = None)
                        .or_insert(argument.as_value());
                }
            }
        }
    }
    incoming
}

// Resolve compiler-owned addresses, checked constant offsets and unique-edge
// transfers. Loaded pointers, entry parameters, joins and cycles remain unknown.
fn stack_destination(
    func: &ir::Function,
    mut value: ir::Value,
    incoming: &HashMap<ir::Value, Option<ir::Value>>,
) -> Option<(ir::StackSlot, i64)> {
    let mut displacement = 0i64;
    // A single-edge cycle with no originating stack address must not establish
    // a proof or hang compilation. Every successful chain visits fewer values.
    for _ in 0..func.dfg.num_values() {
        value = func.dfg.resolve_aliases(value);
        let inst = match func.dfg.value_def(value) {
            ir::ValueDef::Result(inst, _) => inst,
            ir::ValueDef::Param(_, _) => {
                value = (*incoming.get(&value)?)?;
                continue;
            }
            _ => return None,
        };
        match func.dfg.insts[inst] {
            ir::InstructionData::StackLoad {
                opcode: Opcode::StackAddr,
                stack_slot,
                offset,
            } => {
                return Some((stack_slot, displacement.checked_add(i64::from(offset))?));
            }
            ir::InstructionData::BinaryImm64 {
                opcode: Opcode::IaddImm,
                arg,
                imm,
            } => {
                displacement = displacement.checked_add(i64::from(imm))?;
                value = arg;
            }
            _ => return None,
        }
    }
    None
}

/// Run before frame-return cleanup instrumentation so the new fault return also
/// unlinks roots. The hook is noncollecting, including overflow cancellation.
/// This pass alone does not admit callees/providers or enable concurrent GC.
pub(crate) fn lower(
    module: &mut impl Module,
    func: &mut ir::Function,
    context: ir::Value,
) -> Result<Lowered, Error> {
    lower_impl(module, func, context, None, false)
}

/// The caller identifies the instruction completing frame linkage and saved-state
/// initialization. Every fallible store and every return must be dominated by it.
/// Invoke before return cleanup; this does not admit native writers or callees.
pub(crate) fn lower_framed(
    module: &mut impl Module,
    func: &mut ir::Function,
    context: ir::Value,
    frame_ready: ir::Inst,
) -> Result<Lowered, Error> {
    lower_impl(module, func, context, Some(frame_ready), false)
}

fn lower_impl(
    module: &mut impl Module,
    func: &mut ir::Function,
    context: ir::Value,
    frame_ready: Option<ir::Inst>,
    borrowed: bool,
) -> Result<Lowered, Error> {
    let entry = func.layout.entry_block().ok_or("missing entry block")?;
    if func.signature.returns.len() != 1 || func.signature.returns[0].value_type != types::I32 {
        return Err("mutation lowering requires an I32 status return".into());
    }
    if func.dfg.value_type(context) != types::I64
        || !matches!(func.dfg.value_def(context), ir::ValueDef::Param(block, _) if block == entry)
    {
        return Err("mutation context must be an I64 entry parameter".into());
    }
    if borrowed {
        let ready = frame_ready.ok_or("store borrow requires a linked frame")?;
        let cfg = cranelift_codegen::flowgraph::ControlFlowGraph::with_function(func);
        if func.layout.inst_block(ready) != Some(entry) || cfg.pred_iter(entry).next().is_some() {
            return Err("store borrow requires a non-reentrant entry frame".into());
        }
    }
    let dominance = if let Some(ready) = frame_ready {
        if func.layout.inst_block(ready).is_none() {
            return Err("frame-ready instruction is not in the function".into());
        }
        let cfg = cranelift_codegen::flowgraph::ControlFlowGraph::with_function(func);
        Some(cranelift_codegen::dominator_tree::DominatorTree::with_function(func, &cfg))
    } else {
        None
    };
    let require_frame = |inst| -> Result<(), Error> {
        if let (Some(ready), Some(dominance)) = (frame_ready, &dominance) {
            if inst == ready || !dominance.dominates(ready, inst, &func.layout) {
                return Err(
                    "frame initialization must dominate fallible stores and returns".into(),
                );
            }
        }
        Ok(())
    };
    let incoming = single_edge_arguments(func);
    let mut stores = Vec::new();
    let mut stack_stores = 0;
    // Validate the entire input before changing any instruction or adding imports.
    for block in func.layout.blocks() {
        for inst in func.layout.block_insts(block) {
            let instruction = &func.dfg.insts[inst];
            if instruction.opcode() == Opcode::Return {
                require_frame(inst)?;
            }
            if instruction.opcode() == Opcode::StackStore {
                stack_stores += 1;
                continue;
            }
            if !instruction.opcode().can_store() {
                continue;
            }
            let ir::InstructionData::Store {
                opcode,
                args,
                offset,
                flags,
            } = *instruction
            else {
                return Err(format!("unadmitted memory writer: {}", instruction.opcode()).into());
            };
            if flags.explicit_endianness().is_some() {
                return Err("explicit-endian stores require separate mutation lowering".into());
            }
            let ty = func.dfg.value_type(args[0]);
            if !matches!(
                ty,
                types::I8 | types::I16 | types::I32 | types::I64 | types::F32 | types::F64
            ) {
                return Err(format!("unadmitted mutation value type: {ty}").into());
            }
            if func.dfg.value_type(args[1]) != types::I64 {
                return Err("mutation destination must be I64".into());
            }
            let width = match opcode {
                Opcode::Store => ty.bytes(),
                Opcode::Istore8 => 1,
                Opcode::Istore16 => 2,
                Opcode::Istore32 => 4,
                _ => return Err(format!("unadmitted memory writer: {opcode}").into()),
            };
            if let Some((slot, displacement)) = stack_destination(func, args[1], &incoming) {
                let start = displacement.checked_add(i64::from(offset));
                let end = start.and_then(|start| start.checked_add(i64::from(width)));
                if !matches!((start, end), (Some(start), Some(end))
                    if start >= 0 && end <= i64::from(func.sized_stack_slots[slot].size))
                {
                    return Err("mutation store exceeds proven stack slot".into());
                }
                stack_stores += 1;
                continue;
            }
            require_frame(inst)?;
            stores.push((inst, args, i64::from(offset), ty, width));
        }
    }
    let result = Lowered {
        stores: stores.len(),
        stack_stores,
    };
    if stores.is_empty() {
        return Ok(result);
    }
    // Tests opt in explicitly only after establishing a same-thread activation.
    // No managed escape: the handle is passed only to the borrowed-store hook.
    let handle = if borrowed {
        let mut signature = module.make_signature();
        signature.returns.push(AbiParam::new(types::I64));
        let acquire = module.declare_function("experiment_gc_store_state_borrow", Linkage::Import, &signature)?;
        let acquire = module.declare_func_in_func(acquire, func);
        let mut cursor = FuncCursor::new(func).at_inst(frame_ready.unwrap());
        cursor.next_inst();
        let call = cursor.ins().call(acquire, &[]);
        Some(cursor.func.dfg.inst_results(call)[0])
    } else { None };
    let mut signature = module.make_signature();
    signature
        .params
        .extend([types::I64, types::I64, types::I64, types::I32].map(AbiParam::new));
    signature.returns.push(AbiParam::new(types::I32));
    if borrowed { signature.params.insert(0, AbiParam::new(types::I64)); }
    let hook = module.declare_function(if borrowed { "experiment_gc_store_borrowed" } else { "experiment_gc_store" }, Linkage::Import, &signature)?;
    let hook = module.declare_func_in_func(hook, func);
    let failure = func.dfg.make_block();
    let status = func.dfg.append_block_param(failure, types::I32);
    func.layout.append_block(failure);
    FuncCursor::new(func)
        .at_bottom(failure)
        .ins()
        .return_(&[status]);
    for (inst, args, offset, ty, width) in stores {
        let previous = func.layout.inst_block(inst).unwrap();
        let continuation = func.dfg.make_block();
        func.layout.split_block(continuation, inst);
        func.layout.remove_inst(inst);
        let mut cursor = FuncCursor::new(func).at_bottom(previous);
        let destination = cursor.ins().iadd_imm(args[1], offset);
        let bits = if ty == types::F32 {
            let bits = cursor.ins().bitcast(types::I32, MemFlags::new(), args[0]);
            cursor.ins().uextend(types::I64, bits)
        } else if ty == types::F64 {
            cursor.ins().bitcast(types::I64, MemFlags::new(), args[0])
        } else if ty.bits() < 64 {
            cursor.ins().uextend(types::I64, args[0])
        } else {
            args[0]
        };
        let width = cursor.ins().iconst(types::I32, i64::from(width));
        let call = if let Some(handle) = handle {
            cursor.ins().call(hook, &[handle, context, destination, bits, width])
        } else {
            cursor.ins().call(hook, &[context, destination, bits, width])
        };
        let status = cursor.func.dfg.inst_results(call)[0];
        cursor
            .ins()
            .brif(status, failure, &[status.into()], continuation, &[]);
    }
    Ok(result)
}

#[cfg(test)]
#[path = "mutation_store_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "mutation_stack_audit.rs"]
pub(crate) mod stack_audit;
