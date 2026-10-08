//! Opt-in, synchronous read-only snapshots. This is deliberately not root registration.
use super::{
    Error, gc_points,
    profile::{Profile, Stacks},
};
use cranelift_codegen::ir::{self, AbiParam, InstBuilder, StackSlot, types};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::{DataDescription, DataId, FuncId, Linkage, Module};
use cranelift_object::ObjectModule;
use std::collections::HashMap;

struct Point {
    data: DataId,
    length: usize,
    stack_lanes: usize,
    spills: Vec<usize>,
}
pub(super) struct Probes {
    callback: FuncId,
    points: HashMap<(usize, usize), Point>,
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
                types::I32,
                types::I32,
                types::I64,
                types::I32,
                types::I64,
                types::I32,
            ]
            .map(AbiParam::new),
        );
        let callback =
            module.declare_function("neoclr_probe_stack_roots_v1", Linkage::Import, &sig)?;
        let mut points = HashMap::new();
        for (i, f) in p.input.functions.iter().enumerate() {
            if gc_points::native_body(p, i, details) {
                continue;
            }
            for (pc, op) in f.body.iter().enumerate() {
                let Some(stack) = &flows[i][pc] else {
                    continue;
                };
                let Some(plan) = gc_points::point(p, op, stack)? else {
                    continue;
                };
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
        Ok(Self { callback, points })
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
        let function = b.ins().iconst(types::I32, function as i64);
        let pc = b.ins().iconst(types::I32, pc as i64);
        let address = b.ins().stack_addr(types::I64, slot, 0);
        let count = b.ins().iconst(types::I32, point.stack_lanes as i64);
        let data = module.declare_data_in_func(point.data, b.func);
        let descriptor = b.ins().global_value(types::I64, data);
        let length = b.ins().iconst(types::I32, point.length as i64);
        let callback = module.declare_func_in_func(self.callback, b.func);
        b.ins().call(
            callback,
            &[function, pc, address, count, descriptor, length],
        );
    }
}
