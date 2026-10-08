//! Opt-in diagnostic frame chain. Incomplete roots: collection remains forbidden.
use super::{
    Error, gc_layout, gc_points,
    profile::{Profile, Stacks, Ty},
};
use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{self, AbiParam, InstBuilder, StackSlot, types};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::{DataDescription, DataId, FuncId, Linkage, Module};
use cranelift_object::ObjectModule;
use std::collections::HashMap;

struct Storage {
    data: DataId,
    length: usize,
    lanes: Vec<(bool, usize, usize, u32, u32)>,
}
struct Transient {
    data: DataId,
    length: usize,
    lanes: Vec<(usize, u32)>,
}
struct Point {
    data: DataId,
    length: usize,
    stack_lanes: usize,
    spills: Vec<usize>,
}
pub(super) struct Probes {
    callback: FuncId,
    enter: FuncId,
    leave: FuncId,
    points: HashMap<(usize, usize), Point>,
    storage: HashMap<usize, Storage>,
    transient: HashMap<(usize, usize, u32), Transient>,
    publish: FuncId,
}
impl Probes {
    pub fn prepare(
        module: &mut ObjectModule,
        p: &Profile<'_>,
        flows: &[Stacks],
        details: Option<&crate::fault_details::Options>,
    ) -> Result<Self, Error> {
        let mut sig = module.make_signature();
        sig.params.extend(
            [
                types::I64,
                types::I32,
                types::I64,
                types::I32,
                types::I64,
                types::I32,
            ]
            .map(AbiParam::new),
        );
        let callback =
            module.declare_function("neoclr_probe_stack_roots_v2", Linkage::Import, &sig)?;
        let mut enter_sig = module.make_signature();
        enter_sig.params.extend(
            [
                types::I64,
                types::I64,
                types::I32,
                types::I64,
                types::I32,
                types::I64,
                types::I32,
            ]
            .map(AbiParam::new),
        );
        let enter =
            module.declare_function("neoclr_probe_enter_v3", Linkage::Import, &enter_sig)?;
        let mut leave_sig = module.make_signature();
        leave_sig.params.push(AbiParam::new(types::I64));
        let leave =
            module.declare_function("neoclr_probe_leave_v1", Linkage::Import, &leave_sig)?;
        let mut sig = module.make_signature();
        sig.params.extend(
            [
                types::I64,
                types::I32,
                types::I64,
                types::I32,
                types::I64,
                types::I32,
            ]
            .map(AbiParam::new),
        );
        let publish =
            module.declare_function("neoclr_probe_transient_v2", Linkage::Import, &sig)?;
        let mut transient = HashMap::new();
        let mut points = HashMap::new();
        let mut storage = HashMap::new();
        for (i, f) in p.input.functions.iter().enumerate() {
            let native = gc_points::native_body(p, i, details);
            let mut lanes = vec![];
            let mut entries = vec![];
            for (argument, types) in [(true, &p.args[i]), (false, &p.locals[i])] {
                // Replacement native bodies never initialize the IL body's locals.
                if native && !argument {
                    continue;
                }
                for (index, ty) in types.iter().enumerate() {
                    let wide = p.pointer_lanes(ty);
                    for lane in gc_layout::seed_lanes(p, ty) {
                        let width = if wide[lane] { 8 } else { 4 };
                        let flags = if matches!(ty, super::profile::Ty::Address(_)) {
                            2
                        } else if width == 4 {
                            1
                        } else {
                            0
                        };
                        lanes.push((argument, index, lane, width, flags));
                        entries.push(serde_json::json!({"argument": argument, "index": index,
                            "lane": lane, "readBytes": width, "flags": flags, "layout": gc_layout::layout(p, ty)}));
                    }
                }
            }
            let mut bytes = serde_json::to_vec(
                &serde_json::json!({"schema":"neoclr-probe-storage-v1", "entries":entries}),
            )?;
            let length = bytes.len();
            bytes.push(0);
            let data = module.declare_data(
                &format!("neoclr_root_storage_{i}"),
                Linkage::Local,
                false,
                false,
            )?;
            let mut description = DataDescription::new();
            description.define(bytes.into_boxed_slice());
            module.define_data(data, &description)?;
            storage.insert(
                i,
                Storage {
                    data,
                    length,
                    lanes,
                },
            );
            if native {
                continue;
            }
            for (pc, op) in f.body.iter().enumerate() {
                let Some(stack) = &flows[i][pc] else {
                    continue;
                };
                let Some(plan) = gc_points::point(p, op, stack)? else {
                    continue;
                };
                use neoclr::metadata::Instruction as Op;
                let phases: Vec<(u32, Ty)> = match op {
                    Op::Construct(target) => {
                        let ty = p.ty(target.owner.as_ref().unwrap())?;
                        vec![(1, ty.clone()), (2, ty)]
                    }
                    Op::Call(target) | Op::CallVirtual(target) => p.results[p.callee(target)?]
                        .clone()
                        .map(|t| vec![(3, t.clone()), (2, t)])
                        .unwrap_or_default(),
                    _ => vec![],
                };
                for (phase, ty) in phases {
                    let mut bytes = serde_json::to_vec(&gc_layout::layout(p, &ty))?;
                    let length = bytes.len();
                    bytes.push(0);
                    let data = module.declare_data(
                        &format!("neoclr_transient_{i}_{pc}_{phase}"),
                        Linkage::Local,
                        false,
                        false,
                    )?;
                    let mut description = DataDescription::new();
                    description.define(bytes.into_boxed_slice());
                    module.define_data(data, &description)?;
                    let wide = p.pointer_lanes(&ty);
                    let lanes = gc_layout::seed_lanes(p, &ty)
                        .into_iter()
                        .map(|lane| (lane, if wide[lane] { 8 } else { 4 }))
                        .collect();
                    transient.insert(
                        (i, pc, phase),
                        Transient {
                            data,
                            length,
                            lanes,
                        },
                    );
                }
                let mut bytes = serde_json::to_vec(&plan)?;
                let length = bytes.len();
                bytes.push(0);
                let data = module.declare_data(
                    &format!("neoclr_root_probe_{i}_{pc}"),
                    Linkage::Local,
                    false,
                    false,
                )?;
                let mut description = DataDescription::new();
                description.define(bytes.into_boxed_slice());
                module.define_data(data, &description)?;
                points.insert(
                    (i, pc),
                    Point {
                        data,
                        length,
                        stack_lanes: plan["stackLanes"].as_u64().unwrap() as usize,
                        spills: plan["requiredSpillLanes"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_u64().unwrap() as usize)
                            .collect(),
                    },
                );
            }
        }
        Ok(Self {
            callback,
            enter,
            leave,
            points,
            storage,
            transient,
            publish,
        })
    }
    pub fn enter(
        &self,
        module: &mut ObjectModule,
        b: &mut FunctionBuilder<'_>,
        frame: StackSlot,
        context: ir::Value,
        function: usize,
        table: StackSlot,
        arguments: &[StackSlot],
        locals: &[StackSlot],
    ) {
        let address = b.ins().stack_addr(types::I64, frame, 0);
        let storage = &self.storage[&function];
        for (n, &(argument, index, lane, width, flags)) in storage.lanes.iter().enumerate() {
            let slot = if argument {
                arguments[index]
            } else {
                locals[index]
            };
            let pointer = b.ins().stack_addr(types::I64, slot, (lane * 8) as i32);
            b.ins().stack_store(pointer, table, (n * 16) as i32);
            let width = b.ins().iconst(types::I32, width as i64);
            let flags = b.ins().iconst(types::I32, flags as i64);
            b.ins().stack_store(width, table, (n * 16 + 8) as i32);
            b.ins().stack_store(flags, table, (n * 16 + 12) as i32);
        }
        let table = b.ins().stack_addr(types::I64, table, 0);
        let count = b.ins().iconst(types::I32, storage.lanes.len() as i64);
        let data = module.declare_data_in_func(storage.data, b.func);
        let plan = b.ins().global_value(types::I64, data);
        let length = b.ins().iconst(types::I32, storage.length as i64);
        let function = b.ins().iconst(types::I32, function as i64);
        let enter = module.declare_func_in_func(self.enter, b.func);
        b.ins().call(
            enter,
            &[address, context, function, table, count, plan, length],
        );
    }
    // Lowering emits returns in many fault branches. Instrument the completed IR
    // so all returns, including arithmetic/null faults, share the same cleanup.
    pub fn finish(&self, module: &mut ObjectModule, func: &mut ir::Function, frame: StackSlot) {
        let returns: Vec<_> = func
            .layout
            .blocks()
            .flat_map(|block| func.layout.block_insts(block))
            .filter(|&inst| func.dfg.insts[inst].opcode() == ir::Opcode::Return)
            .collect();
        let leave = module.declare_func_in_func(self.leave, func);
        for inst in returns {
            let mut cursor = FuncCursor::new(func).at_inst(inst);
            let address = cursor.ins().stack_addr(types::I64, frame, 0);
            cursor.ins().call(leave, &[address]);
        }
    }
    pub fn transient_bytes(&self, function: usize) -> Option<u32> {
        self.transient
            .iter()
            .filter(|((i, _, _), _)| *i == function)
            .map(|(_, t)| (t.lanes.len().max(1) * 16) as u32)
            .max()
    }
    pub fn publish(
        &self,
        module: &mut ObjectModule,
        b: &mut FunctionBuilder<'_>,
        function: usize,
        pc: usize,
        phase: u32,
        frame: StackSlot,
        table: StackSlot,
        storage: ir::Value,
    ) {
        let t = &self.transient[&(function, pc, phase)];
        for (index, &(lane, width)) in t.lanes.iter().enumerate() {
            let pointer = b.ins().iadd_imm(storage, (lane * 8) as i64);
            b.ins().stack_store(pointer, table, (index * 16) as i32);
            let size = b.ins().iconst(types::I32, width as i64);
            let flag = b.ins().iconst(types::I32, i64::from(width == 4));
            b.ins().stack_store(size, table, (index * 16 + 8) as i32);
            b.ins().stack_store(flag, table, (index * 16 + 12) as i32);
        }
        let frame = b.ins().stack_addr(types::I64, frame, 0);
        let phase = b.ins().iconst(types::I32, phase as i64);
        let table = b.ins().stack_addr(types::I64, table, 0);
        let count = b.ins().iconst(types::I32, t.lanes.len() as i64);
        let data = module.declare_data_in_func(t.data, b.func);
        let plan = b.ins().global_value(types::I64, data);
        let length = b.ins().iconst(types::I32, t.length as i64);
        let callback = module.declare_func_in_func(self.publish, b.func);
        b.ins()
            .call(callback, &[frame, phase, table, count, plan, length]);
    }
    pub fn table_bytes(&self, function: usize) -> u32 {
        (self.storage[&function].lanes.len().max(1) * 16) as u32
    }
    pub fn storage_bytes(&self, function: usize) -> Option<u32> {
        self.points
            .iter()
            .filter(|((i, _), _)| *i == function)
            .map(|(_, point)| (point.stack_lanes.max(1) * 8) as u32)
            .max()
    }
    pub fn emit(
        &self,
        module: &mut ObjectModule,
        b: &mut FunctionBuilder<'_>,
        function: usize,
        pc: usize,
        slot: StackSlot,
        frame: StackSlot,
        stack: &[ir::Value],
    ) {
        let Some(point) = self.points.get(&(function, pc)) else {
            return;
        };
        assert_eq!(point.stack_lanes, stack.len());
        // Every snapshot lane is initialized. Numeric holes are zero, never roots.
        let zero = b.ins().iconst(types::I64, 0);
        for lane in 0..point.stack_lanes {
            b.ins().stack_store(zero, slot, (lane * 8) as i32);
        }
        for &lane in &point.spills {
            let value = stack[lane];
            let value = if b.func.dfg.value_type(value) == types::I64 {
                value
            } else {
                b.ins().uextend(types::I64, value)
            };
            b.ins().stack_store(value, slot, (lane * 8) as i32);
        }
        let frame = b.ins().stack_addr(types::I64, frame, 0);
        let pc = b.ins().iconst(types::I32, pc as i64);
        let address = b.ins().stack_addr(types::I64, slot, 0);
        let count = b.ins().iconst(types::I32, point.stack_lanes as i64);
        let data = module.declare_data_in_func(point.data, b.func);
        let descriptor = b.ins().global_value(types::I64, data);
        let length = b.ins().iconst(types::I32, point.length as i64);
        let callback = module.declare_func_in_func(self.callback, b.func);
        b.ins()
            .call(callback, &[frame, pc, address, count, descriptor, length]);
    }
}
