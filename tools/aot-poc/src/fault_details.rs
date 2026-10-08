//! Private ABI v3 diagnostic capture. The caller owns storage; image data owns text.
use cranelift_codegen::ir::{self, InstBuilder, MemFlags, condcodes::IntCC, types};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::{DataDescription, DataId, Linkage, Module};
use cranelift_object::ObjectModule;
use neoclr::metadata::Instruction as Op;
use std::collections::HashMap;
type Error = Box<dyn std::error::Error>;

#[derive(Default)]
pub struct Options {
    pub probe_stack_roots: bool,
    pub native_gc: bool,
    pub native_stack_budget: bool,
    pub user_faults: Vec<usize>,
    pub console_read_byte: Vec<usize>,
    pub console_write_line: Vec<usize>,
    pub console_write_bytes: Vec<usize>,
    pub console_flush: Vec<usize>,
    pub entry_task_drain: Vec<usize>,
    pub task_queue_register: Vec<usize>,
    pub task_queue_default: Vec<usize>,
    pub task_queue_current: Vec<usize>,
    pub task_queue_run: Option<usize>,
    pub task_queue_drain: Option<usize>,
    pub socket_listen: Vec<usize>,
    pub socket_local_port: Vec<usize>,
    pub socket_close: Vec<usize>,
    pub socket_accept: Vec<usize>,
    pub socket_connect_result: Vec<usize>,
    pub socket_cancel: Vec<usize>,
    pub socket_receive: Vec<usize>,
    pub socket_send: Vec<usize>,
    pub socket_deadline_after: Vec<usize>,
    pub socket_deadline_expired: Vec<usize>,
    pub socket_receive_until: Vec<usize>,
    pub socket_send_until: Vec<usize>,

    pub socket_transfer_result: Vec<usize>,
    pub parse_int32: Vec<usize>,
    pub int32_to_string: Vec<usize>,
    pub string_compare_ordinal: Vec<usize>,
    pub file_input: Vec<usize>,
    pub file_output: Vec<usize>,
    pub path_combine: Vec<usize>,
    pub path_file_name: Vec<usize>,
    pub string_contains_ordinal: Vec<usize>,
    pub string_starts_with_ordinal: Vec<usize>,
    pub string_ends_with_ordinal: Vec<usize>,
    pub string_join_parts: Vec<usize>,
    pub string_concat: Vec<usize>,
    pub string_byte_count: Vec<usize>,
    pub utf8_decode: Vec<usize>,
    pub utf8_encode: Vec<usize>,
    pub string_slice_utf8: Vec<usize>,
    pub char_from_string: Vec<usize>,
    pub char_text: Vec<usize>,
    pub int64_to_string: Vec<usize>,
    pub uint64_to_string: Vec<usize>,
    pub native_integer_to64: Vec<usize>,
    pub reference_arena: bool,
    pub int32_boxes: HashMap<usize, usize>,
    pub boxed_int32_display: bool,
    pub empty_record_boxes: HashMap<usize, usize>,
    pub object_base: Option<usize>,
    pub array_backing: Option<usize>,
    pub reference_array_backings: Vec<usize>,
    pub string_dispatch: HashMap<usize, usize>,
    pub primitive_receivers: Vec<usize>,
    pub string_interfaces: Option<Vec<usize>>,
    pub object_display: HashMap<usize, Vec<(usize, usize)>>,
    pub frame_names: HashMap<usize, String>,
}
impl Options {
    pub fn from_report(report: Option<&serde_json::Value>) -> Self {
        Self {
            primitive_receivers: report.and_then(|r| r["primitiveInstanceProjections"].as_array()).into_iter().flatten()
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            probe_stack_roots: false,
            native_gc: false,
            native_stack_budget: false,
            string_dispatch: report.and_then(|r| r["stringInterfaceDispatch"].as_array()).into_iter().flatten()
                .filter_map(|r| Some((r["contractCompiledIndex"].as_u64()? as usize, r["functionCompiledIndex"].as_u64()? as usize))).collect(),
            string_interfaces: report.and_then(|r| r["stringInterfaceViews"].as_array())
                .map(|rows| rows.iter().filter_map(|i| i.as_u64().map(|i| i as usize)).collect()),
            object_display: report.and_then(|r| r["objectDisplayDispatch"].as_array()).into_iter().flatten()
                .filter_map(|r| Some((r["contractCompiledIndex"].as_u64()? as usize, r["targets"].as_array()?.iter()
                    .filter_map(|t| Some((t["typeCompiledIndex"].as_u64()? as usize, t["functionCompiledIndex"].as_u64()? as usize))).collect()))).collect(),
            boxed_int32_display: report.is_some_and(|r| r["boxedInt32Display"] == true),
            int32_boxes: report.and_then(|r| r["int32Boxes"].as_array()).into_iter().flatten()
                .filter_map(|r| Some((r["helperCompiledIndex"].as_u64()? as usize, r["typeCompiledIndex"].as_u64()? as usize))).collect(),
            empty_record_boxes: report.and_then(|r| r["emptyRecordBoxes"].as_array()).into_iter().flatten()
                .filter_map(|r| Some((r["helperCompiledIndex"].as_u64()? as usize, r["typeCompiledIndex"].as_u64()? as usize))).collect(),
            reference_arena: report.is_some_and(|r| r["referenceArena"] == true),
            reference_array_backings: report.and_then(|r| r["referenceArrayBackingProjections"].as_array()).into_iter().flatten()
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            array_backing: report.and_then(|r| r["arrayBackingProjection"]["compiledIndex"].as_u64()).map(|i| i as usize),
            object_base: report.and_then(|r| r["objectBaseProjection"]["compiledIndex"].as_u64()).map(|i| i as usize),
            frame_names: report
                .and_then(|r| r["functions"].as_array())
                .into_iter()
                .flatten()
                .filter_map(|r| {
                    Some((
                        r["compiledIndex"].as_u64()? as usize,
                        r["name"].as_str()?.to_owned(),
                    ))
                })
                .collect(),
            socket_accept: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "socket-accept-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            socket_connect_result: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "socket-connect-result-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            socket_cancel: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "socket-cancel-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            socket_deadline_after: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "socket-deadline-after-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            socket_deadline_expired: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "socket-deadline-expired-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            socket_receive_until: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "socket-receive-until-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            socket_send_until: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "socket-send-until-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            socket_receive: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "socket-receive-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            socket_transfer_result: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "socket-transfer-result-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            socket_send: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "socket-send-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            entry_task_drain: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "entry-task-drain-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            task_queue_register: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "task-queue-register-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            task_queue_default: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "task-queue-default-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            task_queue_current: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "task-queue-current-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            task_queue_run: report.and_then(|r| r["taskQueueFrames"]["Run"].as_u64()).map(|i| i as usize),
            task_queue_drain: report.and_then(|r| r["taskQueueFrames"]["Drain"].as_u64()).map(|i| i as usize),
            socket_listen: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "socket-listen-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            socket_local_port: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "socket-local-port-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            socket_close: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "socket-close-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            console_write_line: report
                .and_then(|r| r["nativeBindings"].as_array())
                .into_iter().flatten()
                .filter(|r| r["implementation"] == "console-write-line-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize))
                .collect(),
            console_write_bytes: report
                .and_then(|r| r["nativeBindings"].as_array())
                .into_iter().flatten()
                .filter(|r| r["implementation"] == "console-write-bytes-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize))
                .collect(),
            console_flush: report
                .and_then(|r| r["nativeBindings"].as_array())
                .into_iter().flatten()
                .filter(|r| r["implementation"] == "console-flush-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize))
                .collect(),
            utf8_decode: report
                .and_then(|r| r["nativeBindings"].as_array())
                .into_iter().flatten()
                .filter(|r| r["implementation"] == "utf8-decode-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize))
                .collect(),
            utf8_encode: report
                .and_then(|r| r["nativeBindings"].as_array())
                .into_iter().flatten()
                .filter(|r| r["implementation"] == "utf8-encode-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize))
                .collect(),
            file_output: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "file-write-utf8-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            file_input: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "file-read-utf8-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            path_combine: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "path-combine-unix-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            path_file_name: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "path-file-name-unix-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            string_compare_ordinal: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "string-compare-ordinal-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            string_contains_ordinal: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "string-contains-ordinal-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            string_starts_with_ordinal: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "string-starts-with-ordinal-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            string_ends_with_ordinal: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "string-ends-with-ordinal-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            string_join_parts: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "string-join-parts-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            string_concat: report
                .and_then(|r| r["nativeBindings"].as_array())
                .into_iter().flatten()
                .filter(|r| r["implementation"] == "string-concat-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize))
                .collect(),
            string_byte_count: report
                .and_then(|r| r["nativeBindings"].as_array())
                .into_iter().flatten()
                .filter(|r| r["implementation"] == "string-byte-count-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize))
                .collect(),
            string_slice_utf8: report
                .and_then(|r| r["nativeBindings"].as_array())
                .into_iter().flatten()
                .filter(|r| r["implementation"] == "string-slice-utf8-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize))
                .collect(),
            parse_int32: report.and_then(|r| r["nativeBindings"].as_array()).into_iter().flatten()
                .filter(|r| r["implementation"] == "parse-int32-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize)).collect(),
            int32_to_string: report
                .and_then(|r| r["nativeBindings"].as_array())
                .into_iter().flatten()
                .filter(|r| r["implementation"] == "int32-to-string-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize))
                .collect(),
            char_from_string: report
                .and_then(|r| r["nativeBindings"].as_array())
                .into_iter().flatten()
                .filter(|r| r["implementation"] == "char-from-string-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize))
                .collect(),
            char_text: report
                .and_then(|r| r["nativeBindings"].as_array())
                .into_iter().flatten()
                .filter(|r| r["implementation"] == "char-text-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize))
                .collect(),
            int64_to_string: report
                .and_then(|r| r["nativeBindings"].as_array())
                .into_iter().flatten()
                .filter(|r| r["implementation"] == "int64-to-string-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize))
                .collect(),
            uint64_to_string: report
                .and_then(|r| r["nativeBindings"].as_array())
                .into_iter().flatten()
                .filter(|r| r["implementation"] == "uint64-to-string-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize))
                .collect(),
            native_integer_to64: report
                .and_then(|r| r["nativeBindings"].as_array())
                .into_iter().flatten()
                .filter(|r| r["implementation"] == "native-integer-to64-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize))
                .collect(),
            console_read_byte: report
                .and_then(|r| r["nativeBindings"].as_array())
                .into_iter().flatten()
                .filter(|r| r["implementation"] == "console-read-byte-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize))
                .collect(),
            user_faults: report
                .and_then(|r| r["nativeBindings"].as_array())
                .into_iter()
                .flatten()
                .filter(|r| r["implementation"] == "terminal-user-fault-v1")
                .filter_map(|r| r["compiledIndex"].as_u64().map(|i| i as usize))
                .collect(),
        }
    }
}
pub struct Data {
    frames: Vec<DataId>,
    messages: HashMap<(usize, usize), DataId>,
    defaults: [DataId; 8],
}
fn literal(module: &mut ObjectModule, name: &str, text: &str) -> Result<DataId, Error> {
    let id = module.declare_data(name, Linkage::Local, false, false)?;
    let mut bytes = (text.len() as u64).to_le_bytes().to_vec();
    bytes.extend_from_slice(text.as_bytes());
    let mut data = DataDescription::new();
    data.define(bytes.into_boxed_slice());
    data.set_align(8);
    module.define_data(id, &data)?;
    Ok(id)
}
impl Data {
    pub fn new(
        module: &mut ObjectModule,
        input: &neoclr::Module,
        options: &Options,
    ) -> Result<Self, Error> {
        let mut frames = vec![];
        let mut messages = HashMap::new();
        for (i, f) in input.functions.iter().enumerate() {
            let name = options
                .frame_names
                .get(&i)
                .map_or(f.name.as_str(), String::as_str);
            frames.push(literal(module, &format!("fault_frame_{i}"), name)?);
            for (pc, op) in f.body.iter().enumerate() {
                if let Op::Fault(text) = op {
                    messages.insert(
                        (i, pc),
                        literal(module, &format!("fault_message_{i}_{pc}"), text)?,
                    );
                }
            }
        }
        let defaults = [
            literal(
                module,
                "fault_divide",
                neoclr::FaultCode::DivideByZero.standard_message().unwrap(),
            )?,
            literal(
                module,
                "fault_overflow",
                neoclr::FaultCode::ArithmeticOverflow
                    .standard_message()
                    .unwrap(),
            )?,
            literal(
                module,
                "fault_runtime",
                neoclr::FaultCode::RuntimeError.standard_message().unwrap(),
            )?,
            literal(module, "fault_native_memory",
                neoclr::FaultCode::NativeMemoryLimitExceeded.standard_message().unwrap())?,
            literal(module, "fault_null_reference", neoclr::FaultCode::NullReference.standard_message().unwrap())?,
            literal(module, "fault_array_limit", neoclr::FaultCode::ArrayLimitExceeded.standard_message().unwrap())?,
            literal(module, "fault_index", neoclr::FaultCode::IndexOutOfRange.standard_message().unwrap())?,
            literal(module, "fault_stack", neoclr::FaultCode::StackOverflow.standard_message().unwrap())?,
        ];
        Ok(Self {
            frames,
            messages,
            defaults,
        })
    }
    pub fn site(
        &self,
        module: &mut ObjectModule,
        b: &mut FunctionBuilder<'_>,
        context: ir::Value,
        i: usize,
        pc: usize,
    ) -> Site {
        let mut address = |id| {
            let global = module.declare_data_in_func(id, b.func);
            b.ins().global_value(types::I64, global)
        };
        let frame = address(self.frames[i]);
        let defaults = self.defaults.map(&mut address);
        let message = self.messages.get(&(i, pc)).map(|id| address(*id));
        Site {
            context,
            frame,
            pc,
            defaults,
            message,
            capture_frame: true,
        }
    }
}
pub struct Site {
    context: ir::Value,
    frame: ir::Value,
    pc: usize,
    defaults: [ir::Value; 8],
    pub message: Option<ir::Value>,
    pub capture_frame: bool,
}
impl Site {
    pub fn record(&self, b: &mut FunctionBuilder<'_>, status: ir::Value) {
        let count = b.ins().load(types::I32, MemFlags::new(), self.context, 4);
        let previous = b.ins().load(types::I32, MemFlags::new(), self.context, 0);
        let first = b.ins().icmp_imm(IntCC::Equal, previous, 0);
        let initialize = b.create_block();
        let append = b.create_block();
        b.ins().brif(first, initialize, &[], append, &[]);
        b.switch_to_block(initialize);
        let message = if let Some(message) = self.message {
            message
        } else {
            let overflow = b.ins().icmp_imm(IntCC::Equal, status, 2);
            let other = b.ins().select(overflow, self.defaults[1], self.defaults[2]);
            let divide = b.ins().icmp_imm(IntCC::Equal, status, 1);
            let message = b.ins().select(divide, self.defaults[0], other);
            let memory = b.ins().icmp_imm(IntCC::Equal, status, 5);
            let message = b.ins().select(memory, self.defaults[3], message);
            let null = b.ins().icmp_imm(IntCC::Equal, status, 6);
            let message = b.ins().select(null, self.defaults[4], message);
            let array = b.ins().icmp_imm(IntCC::Equal, status, 7);
            let message = b.ins().select(array, self.defaults[5], message);
            let index = b.ins().icmp_imm(IntCC::Equal, status, 8);
            let message = b.ins().select(index, self.defaults[6], message);
            let stack = b.ins().icmp_imm(IntCC::Equal, status, 9);
            b.ins().select(stack, self.defaults[7], message)
        };
        b.ins().store(MemFlags::new(), status, self.context, 0);
        b.ins().store(MemFlags::new(), message, self.context, 8);
        b.ins().jump(append, &[]);
        b.switch_to_block(append);
        if !self.capture_frame {
            return;
        }
        // Fault capture is bounded independently of the live native call stack.
        let room = b.ins().icmp_imm(
            IntCC::UnsignedLessThan,
            count,
            neoclr::StackTrace::MAX_FRAMES as i64,
        );
        let write = b.create_block();
        let done = b.create_block();
        let truncated = b.create_block();
        b.ins().brif(room, write, &[], truncated, &[]);
        b.switch_to_block(truncated);
        let one = b.ins().iconst(types::I32, 1);
        b.ins().store(MemFlags::new(), one, self.context, 16);
        b.ins().jump(done, &[]);
        b.switch_to_block(write);
        let offset = b.ins().uextend(types::I64, count);
        let offset = b.ins().imul_imm(offset, 16);
        let frame = b.ins().iadd(self.context, offset);
        b.ins().store(MemFlags::new(), self.frame, frame, 24);
        let pc = b.ins().iconst(types::I32, self.pc as i64);
        b.ins().store(MemFlags::new(), pc, frame, 32);
        let zero = b.ins().iconst(types::I32, 0);
        b.ins().store(MemFlags::new(), zero, frame, 36);
        let count = b.ins().iadd_imm(count, 1);
        b.ins().store(MemFlags::new(), count, self.context, 4);
        b.ins().jump(done, &[]);
        b.switch_to_block(done);
    }
}
pub fn reset(b: &mut FunctionBuilder<'_>, context: ir::Value) {
    let zero = b.ins().iconst(types::I64, 0);
    b.ins().store(MemFlags::new(), zero, context, 0);
    b.ins().store(MemFlags::new(), zero, context, 8);
    b.ins().store(MemFlags::new(), zero, context, 16);
}
