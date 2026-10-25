//! Published frames for diagnostic observation or explicit nonmoving GC boundaries.
use super::{
    Error, gc_layout, gc_points,
    profile::{Profile, Stacks, Ty},
};
use cranelift_codegen::ir::{self, AbiParam, InstBuilder, StackSlot, types};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::{DataDescription, DataId, FuncId, Linkage, Module};
use std::collections::HashMap;

struct Storage {
    data: DataId,
    length: usize,
    lanes: Vec<(bool, usize, usize, u32, u32)>,
}
struct Transient {
    data: Option<DataId>,
    length: usize,
    lanes: Vec<(usize, u32)>,
}
struct Point {
    exclusive: bool,
    data: DataId,
    length: usize,
    stack_lanes: usize,
    spills: Vec<usize>,
}
pub(super) struct Probes {
    workers: bool,
    state_handle: bool,
    callback: FuncId,
    enter: FuncId,
    leave: FuncId,
    points: HashMap<(usize, usize), Point>,
    storage: HashMap<usize, Storage>,
    transient: HashMap<(usize, usize, u32), Transient>,
    publish: FuncId,
}
impl Probes {
    pub(super) fn supports_store_borrow(&self) -> bool {
        self.state_handle && !self.workers
    }

    pub fn prepare(
        module: &mut impl Module,
        p: &Profile<'_>,
        flows: &[Stacks],
        details: Option<&crate::fault_details::Options>,
    ) -> Result<Self, Error> {
        let gc = details.is_some_and(|d| d.native_gc);
        let workers = details.is_some_and(|d| d.native_worker_safepoints);
        let state_handle = gc && !workers;
        if workers {
            let d = details.unwrap();
            for (i, f) in p.input.functions.iter().enumerate() {
                if (gc_points::native_body(p, i, details)
                    || f.is_internal_call()
                    || f.pinvoke.is_some())
                    && !d.user_faults.contains(&i)
                    && !d.int32_to_string.contains(&i)
                    && !d.string_concat.contains(&i)
                    && !d.int32_boxes.contains_key(&i)
                    && !d.scalar_box_queries.contains_key(&i)
                {
                    return Err(format!(
                        "worker safepoints reject unaudited native body: {}",
                        f.name
                    )
                    .into());
                }
            }
        }
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
        if workers {
            sig.params.push(AbiParam::new(types::I32));
        }
        if state_handle {
            sig.params.insert(0, AbiParam::new(types::I64));
        }
        let callback = module.declare_function(
            if workers {
                "neoclr_gc_worker_read_phase_map_v1"
            } else if state_handle {
                "neoclr_gc_state_stack_v1"
            } else if gc {
                "neoclr_gc_stack_map_v1"
            } else {
                "neoclr_probe_stack_roots_v2"
            },
            Linkage::Import,
            &sig,
        )?;
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
        if state_handle {
            enter_sig.returns.push(AbiParam::new(types::I64));
        }
        let enter = module.declare_function(
            if state_handle {
                "neoclr_gc_state_enter_v1"
            } else if gc {
                // This table is emitted once in the prologue and remains fixed
                // until frame leave, including while a cooperative stack parks.
                "neoclr_gc_enter_map_v1"
            } else {
                "neoclr_probe_enter_v3"
            },
            Linkage::Import,
            &enter_sig,
        )?;
        let mut leave_sig = module.make_signature();
        leave_sig.params.push(AbiParam::new(types::I64));
        if state_handle {
            leave_sig.params.push(AbiParam::new(types::I64));
        }
        let leave = module.declare_function(
            if state_handle {
                "neoclr_gc_state_leave_v1"
            } else if gc {
                "neoclr_gc_leave_v1"
            } else {
                "neoclr_probe_leave_v1"
            },
            Linkage::Import,
            &leave_sig,
        )?;
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
        if state_handle {
            sig.params.insert(0, AbiParam::new(types::I64));
        }
        let publish = module.declare_function(
            if state_handle {
                "neoclr_gc_state_transient_v1"
            } else if gc {
                "neoclr_gc_transient_map_v1"
            } else {
                "neoclr_probe_transient_v2"
            },
            Linkage::Import,
            &sig,
        )?;
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
                        if !gc {
                            entries.push(serde_json::json!({"argument": argument, "index": index,
                                "lane": lane, "readBytes": width, "flags": flags, "layout": gc_layout::layout(p, ty)}));
                        }
                    }
                }
            }
            let diagnostic =
                serde_json::json!({"schema":"neoclr-probe-storage-v1", "entries":entries});
            let (bytes, length) = execution_slots(&diagnostic, STORAGE_MAP_MAGIC, lanes.len(), gc)?;
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
                let candidate = gc_points::point_at(p, i, pc, stack)?;
                let exclusive = gc_points::worker_exclusive(op, candidate.is_some());
                let plan = if let Some(plan) = candidate {
                    plan
                } else if workers {
                    gc_points::worker_point(p, stack)?
                } else {
                    continue;
                };
                use neoclr::metadata::Instruction as Op;
                let phases: Vec<(u32, Ty)> = match op {
                    Op::Construct(target) => {
                        let ty = p.ty(target.owner.as_ref().unwrap())?;
                        vec![(1, ty.clone()), (2, ty)]
                    }
                    Op::Call(target) | Op::CallVirtual(target)
                        if crate::selection::callable_invoke(target).is_some() =>
                    {
                        let shape = crate::selection::callable_invoke(target).unwrap();
                        if shape.no_result {
                            vec![]
                        } else {
                            let t = p.ty(&shape.returns)?;
                            vec![(3, t.clone()), (2, t)]
                        }
                    }
                    Op::Call(target) | Op::CallVirtual(target) => p.results[p.callee(target)?]
                        .clone()
                        .map(|t| vec![(3, t.clone()), (2, t)])
                        .unwrap_or_default(),
                    _ => vec![],
                };
                for (phase, ty) in phases {
                    let wide = p.pointer_lanes(&ty);
                    let lanes: Vec<_> = gc_layout::seed_lanes(p, &ty)
                        .into_iter()
                        .map(|lane| (lane, if wide[lane] { 8 } else { 4 }))
                        .collect();
                    // The pre-operation boundary clears this frame's transient roots.
                    // A pre-call table points at scratch storage throughout the call:
                    // returning changes its values, not the descriptor addresses.
                    // Diagnostic probes retain all phase events; GC needs neither an
                    // empty table nor an identical post-success republication.
                    if gc && (phase == 2 || lanes.is_empty()) {
                        transient.insert(
                            (i, pc, phase),
                            Transient {
                                data: None,
                                length: 0,
                                lanes: vec![],
                            },
                        );
                        continue;
                    }
                    let diagnostic = if gc {
                        serde_json::Value::Null
                    } else {
                        gc_layout::layout(p, &ty)
                    };
                    let (bytes, length) =
                        execution_slots(&diagnostic, TRANSIENT_MAP_MAGIC, lanes.len(), gc)?;
                    let data = module.declare_data(
                        &format!("neoclr_transient_{i}_{pc}_{phase}"),
                        Linkage::Local,
                        false,
                        false,
                    )?;
                    let mut description = DataDescription::new();
                    description.define(bytes.into_boxed_slice());
                    module.define_data(data, &description)?;
                    transient.insert(
                        (i, pc, phase),
                        Transient {
                            data: Some(data),
                            length,
                            lanes,
                        },
                    );
                }
                // Native GC executes the published slot/lane tables; the rich
                // inspection plan is diagnostic data, not a tracing program.
                // Keep a versioned lane-count header in the execution image.
                let (bytes, length) = if gc {
                    // Versioned execution header; explicit published lanes carry
                    // roots. Rich spill/layout metadata remains in inspection.
                    let bytes = native_root_map(plan["stackLanes"].as_u64().unwrap() as u32);
                    (bytes, 8)
                } else {
                    let mut bytes = serde_json::to_vec(&plan)?;
                    let length = bytes.len();
                    bytes.push(0);
                    (bytes, length)
                };
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
                        exclusive,
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
            workers,
            state_handle,
            callback,
            enter,
            leave,
            points,
            storage,
            transient,
            publish,
        })
    }
    // The collector-visible prefix remains 104 bytes. The trailing handle is
    // compiler-owned native state, never a managed root or part of frame traversal.
    pub fn frame_bytes(&self) -> u32 {
        if self.state_handle { 112 } else { 104 }
    }
    // GC publication keeps frame, storage and instruction operands explicit at each emission site.
    #[allow(clippy::too_many_arguments)]
    pub fn enter(
        &self,
        module: &mut impl Module,
        b: &mut FunctionBuilder<'_>,
        frame: StackSlot,
        context: ir::Value,
        function: usize,
        table: StackSlot,
        arguments: &[StackSlot],
        locals: &[StackSlot],
    ) -> ir::Inst {
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
        let call = b.ins().call(
            enter,
            &[address, context, function, table, count, plan, length],
        );
        if self.state_handle {
            let state = b.inst_results(call)[0];
            b.ins().stack_store(state, frame, 104)
        } else {
            call
        }
    }
    // Lowering emits returns in many fault branches. Instrument the completed IR
    // so all returns, including arithmetic/null faults, share the same cleanup.
    pub fn finish(&self, module: &mut impl Module, func: &mut ir::Function, frame: StackSlot) {
        crate::frame_cleanup::finish(module, func, frame, self.leave, self.state_handle);
    }
    pub fn transient_bytes(&self, function: usize) -> Option<u32> {
        self.transient
            .iter()
            .filter(|((i, _, _), _)| *i == function)
            .map(|(_, t)| (t.lanes.len().max(1) * 16) as u32)
            .max()
    }
    // GC publication keeps frame, storage and instruction operands explicit at each emission site.
    #[allow(clippy::too_many_arguments)]
    pub fn publish(
        &self,
        module: &mut impl Module,
        b: &mut FunctionBuilder<'_>,
        function: usize,
        pc: usize,
        phase: u32,
        frame: StackSlot,
        table: StackSlot,
        storage: ir::Value,
    ) {
        let t = &self.transient[&(function, pc, phase)];
        let Some(data) = t.data else {
            return;
        };
        for (index, &(lane, width)) in t.lanes.iter().enumerate() {
            let pointer = b.ins().iadd_imm(storage, (lane * 8) as i64);
            b.ins().stack_store(pointer, table, (index * 16) as i32);
            let size = b.ins().iconst(types::I32, width as i64);
            let flag = b.ins().iconst(types::I32, i64::from(width == 4));
            b.ins().stack_store(size, table, (index * 16 + 8) as i32);
            b.ins().stack_store(flag, table, (index * 16 + 12) as i32);
        }
        let state = self
            .state_handle
            .then(|| b.ins().stack_load(types::I64, frame, 104));
        let frame = b.ins().stack_addr(types::I64, frame, 0);
        let phase = b.ins().iconst(types::I32, phase as i64);
        let table = b.ins().stack_addr(types::I64, table, 0);
        let count = b.ins().iconst(types::I32, t.lanes.len() as i64);
        let data = module.declare_data_in_func(data, b.func);
        let plan = b.ins().global_value(types::I64, data);
        let length = b.ins().iconst(types::I32, t.length as i64);
        let callback = module.declare_func_in_func(self.publish, b.func);
        if let Some(state) = state {
            b.ins()
                .call(callback, &[state, frame, phase, table, count, plan, length]);
        } else {
            b.ins()
                .call(callback, &[frame, phase, table, count, plan, length]);
        }
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
    // GC publication keeps frame, storage and instruction operands explicit at each emission site.
    #[allow(clippy::too_many_arguments)]
    pub fn emit(
        &self,
        module: &mut impl Module,
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
        let state = self
            .state_handle
            .then(|| b.ins().stack_load(types::I64, frame, 104));
        let frame = b.ins().stack_addr(types::I64, frame, 0);
        let pc = b.ins().iconst(types::I32, pc as i64);
        let address = b.ins().stack_addr(types::I64, slot, 0);
        let count = b.ins().iconst(types::I32, point.stack_lanes as i64);
        let data = module.declare_data_in_func(point.data, b.func);
        let descriptor = b.ins().global_value(types::I64, data);
        let length = b.ins().iconst(types::I32, point.length as i64);
        let callback = module.declare_func_in_func(self.callback, b.func);
        let mut args = vec![frame, pc, address, count, descriptor, length];
        if let Some(state) = state {
            args.insert(0, state);
        }
        if self.workers {
            args.push(b.ins().iconst(types::I32, i64::from(point.exclusive)));
        }
        b.ins().call(callback, &args);
    }
}

// Current native targets are little-endian ARM64/x64. Keep this version/layout
// matched with neoclr_gc_root_map; do not serialize a host-sized Rust structure.
const ROOT_MAP_MAGIC: u32 = 0x3143474e;
const STORAGE_MAP_MAGIC: u32 = 0x3153474e;
const TRANSIENT_MAP_MAGIC: u32 = 0x3154474e;

fn native_count_map(magic: u32, count: u32) -> Vec<u8> {
    let mut bytes = magic.to_le_bytes().to_vec();
    bytes.extend_from_slice(&count.to_le_bytes());
    bytes
}

fn native_root_map(lanes: u32) -> Vec<u8> {
    native_count_map(ROOT_MAP_MAGIC, lanes)
}

fn execution_slots(
    plan: &serde_json::Value,
    magic: u32,
    count: usize,
    native_gc: bool,
) -> Result<(Vec<u8>, usize), Error> {
    if native_gc {
        Ok((native_count_map(magic, count.try_into()?), 8))
    } else {
        let mut bytes = serde_json::to_vec(plan)?;
        let length = bytes.len();
        bytes.push(0);
        Ok((bytes, length))
    }
}

#[cfg(test)]
mod execution_metadata_tests {
    use super::{STORAGE_MAP_MAGIC, TRANSIENT_MAP_MAGIC, execution_slots, native_root_map};
    use serde_json::json;

    #[test]
    fn native_map_has_version_and_exact_lane_count() {
        assert_eq!(native_root_map(8192), [0x4e, 0x47, 0x43, 0x31, 0, 32, 0, 0]);
    }

    #[test]
    fn storage_and_transient_maps_keep_counts_and_diagnostic_layouts() {
        let layout =
            json!({"fields": [{"offset": 0, "readBytes": 4}, {"offset": 8, "readBytes": 8}]});
        for (magic, tag) in [(STORAGE_MAP_MAGIC, b"NGS1"), (TRANSIENT_MAP_MAGIC, b"NGT1")] {
            for count in [0, 2, 129] {
                let (bytes, length) = execution_slots(&layout, magic, count, true).unwrap();
                assert_eq!(length, 8);
                assert_eq!(bytes.len(), 8);
                assert_eq!(&bytes[..4], tag);
                assert_eq!(&bytes[4..], &(count as u32).to_le_bytes());
                let (bytes, length) = execution_slots(&layout, magic, count, false).unwrap();
                assert_eq!(bytes.len(), length + 1);
                assert_eq!(bytes[length], 0);
                assert_eq!(
                    serde_json::from_slice::<serde_json::Value>(&bytes[..length]).unwrap(),
                    layout
                );
            }
        }
    }
}
