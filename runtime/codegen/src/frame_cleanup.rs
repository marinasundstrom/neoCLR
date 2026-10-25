//! Shared return instrumentation for compiler-published GC frames.
use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{self, InstBuilder, StackSlot, types};
use cranelift_module::{FuncId, Module};

pub(crate) fn finish(
    module: &mut impl Module,
    func: &mut ir::Function,
    frame: StackSlot,
    leave: FuncId,
    state_handle: bool,
) {
    let returns: Vec<_> = func
        .layout
        .blocks()
        .flat_map(|block| func.layout.block_insts(block))
        .filter(|&inst| func.dfg.insts[inst].opcode() == ir::Opcode::Return)
        .collect();
    let leave = module.declare_func_in_func(leave, func);
    for inst in returns {
        let mut cursor = FuncCursor::new(func).at_inst(inst);
        let address = cursor.ins().stack_addr(types::I64, frame, 0);
        if state_handle {
            let state = cursor.ins().stack_load(types::I64, frame, 104);
            cursor.ins().call(leave, &[state, address]);
        } else {
            cursor.ins().call(leave, &[address]);
        }
    }
}
