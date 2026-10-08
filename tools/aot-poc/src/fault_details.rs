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
    pub user_faults: Vec<usize>,
    pub console_read_byte: Vec<usize>,
    pub console_write_line: Vec<usize>,
    pub console_write_bytes: Vec<usize>,
    pub console_flush: Vec<usize>,
    pub int32_to_string: Vec<usize>,
    pub string_byte_count: Vec<usize>,
    pub string_slice_utf8: Vec<usize>,
    pub char_from_string: Vec<usize>,
    pub char_text: Vec<usize>,
    pub int64_to_string: Vec<usize>,
    pub uint64_to_string: Vec<usize>,
    pub native_integer_to64: Vec<usize>,
    pub reference_arena: bool,
    pub object_base: Option<usize>,
    pub frame_names: HashMap<usize, String>,
}
impl Options {
    pub fn from_report(report: Option<&serde_json::Value>) -> Self {
        Self {
            reference_arena: report.is_some_and(|r| r["referenceArena"] == true),
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
    defaults: [DataId; 7],
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
    defaults: [ir::Value; 7],
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
            b.ins().select(index, self.defaults[6], message)
        };
        b.ins().store(MemFlags::new(), status, self.context, 0);
        b.ins().store(MemFlags::new(), message, self.context, 8);
        b.ins().jump(append, &[]);
        b.switch_to_block(append);
        if !self.capture_frame {
            return;
        }
        // The acyclic 128-function profile bounds live frames. Retain a bounds guard
        // as a second line of defense at the native storage boundary.
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
