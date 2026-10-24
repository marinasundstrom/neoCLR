//! Experimental scalar-store lowering. Not admission for concurrent execution:
//! native calls, construction and scheduler paths require separate qualification.
use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{self, types, AbiParam, InstBuilder, MemFlags, Opcode};
use cranelift_module::{Linkage, Module};

type Error = Box<dyn std::error::Error>;

#[derive(Debug, PartialEq)]
pub(crate) struct Lowered {
    pub stores: usize,
    pub stack_stores: usize,
}

/// Run before frame-return cleanup instrumentation so the new fault return also
/// unlinks roots. The hook is noncollecting, including overflow cancellation.
/// This pass alone does not admit callees/providers or enable concurrent GC.
pub(crate) fn lower(
    module: &mut impl Module,
    func: &mut ir::Function,
    context: ir::Value,
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
    let mut stores = Vec::new();
    let mut stack_stores = 0;
    // Validate the entire input before changing any instruction or adding imports.
    for block in func.layout.blocks() {
        for inst in func.layout.block_insts(block) {
            let instruction = &func.dfg.insts[inst];
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
    let mut signature = module.make_signature();
    signature
        .params
        .extend([types::I64, types::I64, types::I64, types::I32].map(AbiParam::new));
    signature.returns.push(AbiParam::new(types::I32));
    let hook = module.declare_function("experiment_gc_store", Linkage::Import, &signature)?;
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
        let call = cursor
            .ins()
            .call(hook, &[context, destination, bits, width]);
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
