//! Explicit native service implementations after original-scope verification and
//! closed-world selection. Ordinary managed wrappers are never replaced.
use neoclr::metadata::{Instruction as Op, Type};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;

pub fn user_fault(input: &mut neoclr::Module, selection: &Value) -> Result<Value, Error> {
    let mut bindings = vec![];
    for row in selection["functions"]
        .as_array()
        .ok_or("missing selection inventory")?
    {
        let legacy =
            row["name"] == "neoCLR.Runtime.Fault" && row["definition"]["module"] == "System";
        let source_owned = row["name"] == "neoCLR.Runtime.Fail";
        if !legacy && !source_owned {
            continue;
        }
        let f = &mut input.functions[row["compiledIndex"]
            .as_u64()
            .ok_or("missing compiled index")? as usize];
        if f.name != row["name"].as_str().unwrap()
            || f.owner.is_some()
            || f.instance
            || f.receiver_byref
            || f.receiver_readonly
            || f.parameters != [Type::String]
            || f.returns != Type::Void
            || f.no_result != source_owned
            || f.impl_flags != 0x1000
            || f.pinvoke.is_some()
            || !f.body.is_empty()
            || !f.locals.is_empty()
            || f.is_virtual
            || f.is_override
            || f.is_abstract
            || !f.generic_parameters.is_empty()
            || !f.generic_arguments.is_empty()
            || !f.generic_constraints.is_empty()
            || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty()
            || !f.out_when_true.is_empty()
            || !f.readonly_parameters.is_empty()
        {
            return Err("native UserFault binding requires exact neoCLR.Runtime.Fault(String) -> Void or neoCLR.Runtime.Fail(String) -> noresult InternalCall contract".into());
        }
        // The supplied String remains a normal argument. Diagnostic lowering uses
        // the explicit binding index to capture that message at the terminal site.
        f.impl_flags = 0;
        f.body = vec![Op::Fault("native UserFault service".into())];
        bindings.push(json!({"definition": row["definition"], "name": row["name"],
            "compiledIndex": row["compiledIndex"], "implementation": "terminal-user-fault-v1",
            "status": 4, "diagnostics": "UTF-8 message and managed frames; caller-owned ABI v3"}));
    }
    Ok(json!(bindings))
}

/// Bind only the verified reserved service, never its public managed wrapper.
pub fn console_read_byte(
    input: &mut neoclr::Module,
    selection: &Value,
) -> Result<Vec<Value>, Error> {
    let mut bindings = vec![];
    for row in selection["functions"]
        .as_array()
        .ok_or("missing selection inventory")?
    {
        if row["name"] != "neoCLR.Runtime.ConsoleReadByte" {
            continue;
        }
        let f = &mut input.functions[row["compiledIndex"]
            .as_u64()
            .ok_or("missing compiled index")? as usize];
        if f.name != "neoCLR.Runtime.ConsoleReadByte"
            || f.owner.is_some()
            || f.instance
            || f.receiver_byref
            || f.receiver_readonly
            || !f.parameters.is_empty()
            || f.returns != Type::Value
            || f.no_result
            || f.impl_flags != 0x1000
            || f.pinvoke.is_some()
            || !f.body.is_empty()
            || !f.locals.is_empty()
            || f.is_virtual
            || f.is_override
            || f.is_abstract
            || !f.generic_parameters.is_empty()
            || !f.generic_arguments.is_empty()
            || !f.generic_constraints.is_empty()
            || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty()
            || !f.out_when_true.is_empty()
            || !f.readonly_parameters.is_empty()
        {
            return Err("native byte input binding requires exact neoCLR.Runtime.ConsoleReadByte() -> System.Value InternalCall contract".into());
        }
        // A verifiable private placeholder. Only this reported definition is lowered
        // to the native service; no opcode or guest name is an intrinsic by itself.
        f.impl_flags = 0;
        f.body = vec![Op::Void, Op::PackValue(Type::Void), Op::Return];
        bindings.push(json!({"definition": row["definition"], "name": row["name"],
            "compiledIndex": row["compiledIndex"], "implementation": "console-read-byte-v1",
            "symbol": "neoclr_console_read_byte_v1"}));
    }
    Ok(bindings)
}

/// Output service used by the ordinary string, Boolean and empty-line wrappers.
pub fn console_write_line(
    input: &mut neoclr::Module,
    selection: &Value,
) -> Result<Vec<Value>, Error> {
    let mut bindings = vec![];
    for row in selection["functions"]
        .as_array()
        .ok_or("missing selection inventory")?
    {
        if row["name"] != "neoCLR.Runtime.WriteLine" {
            continue;
        }
        let f = &mut input.functions[row["compiledIndex"]
            .as_u64()
            .ok_or("missing compiled index")? as usize];
        if f.name != "neoCLR.Runtime.WriteLine"
            || f.owner.is_some()
            || f.instance
            || f.receiver_byref
            || f.receiver_readonly
            || f.parameters != [Type::String]
            || f.returns != Type::Void
            || f.impl_flags != 0x1000
            || f.pinvoke.is_some()
            || !f.body.is_empty()
            || !f.locals.is_empty()
            || f.is_virtual
            || f.is_override
            || f.is_abstract
            || !f.generic_parameters.is_empty()
            || !f.generic_arguments.is_empty()
            || !f.generic_constraints.is_empty()
            || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty()
            || !f.out_when_true.is_empty()
            || !f.readonly_parameters.is_empty()
        {
            return Err("native line output binding requires exact neoCLR.Runtime.WriteLine(String) -> Void/noresult InternalCall contract".into());
        }
        f.impl_flags = 0;
        f.body = if f.no_result {
            vec![Op::Return]
        } else {
            vec![Op::Void, Op::Return]
        };
        bindings.push(json!({"definition": row["definition"], "name": row["name"],
            "compiledIndex": row["compiledIndex"], "implementation": "console-write-line-v1",
            "symbol": "neoclr_console_write_line_utf8_v1"}));
    }
    Ok(bindings)
}

/// Explicit formatting producer for invocation-owned String data.
pub fn int32_to_string(input: &mut neoclr::Module, selection: &Value) -> Result<Vec<Value>, Error> {
    let mut bindings = vec![];
    for row in selection["functions"]
        .as_array()
        .ok_or("missing selection inventory")?
    {
        if row["name"] != "neoCLR.Runtime.Int32ToString" {
            continue;
        }
        let f = &mut input.functions[row["compiledIndex"]
            .as_u64()
            .ok_or("missing compiled index")? as usize];
        if f.name != "neoCLR.Runtime.Int32ToString"
            || f.owner.is_some()
            || f.instance
            || f.receiver_byref
            || f.receiver_readonly
            || f.parameters != [Type::Int32]
            || f.returns != Type::String
            || f.no_result
            || f.impl_flags != 0x1000
            || f.pinvoke.is_some()
            || !f.body.is_empty()
            || !f.locals.is_empty()
            || f.is_virtual
            || f.is_override
            || f.is_abstract
            || !f.generic_parameters.is_empty()
            || !f.generic_arguments.is_empty()
            || !f.generic_constraints.is_empty()
            || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty()
            || !f.out_when_true.is_empty()
            || !f.readonly_parameters.is_empty()
        {
            return Err("native formatting binding requires exact neoCLR.Runtime.Int32ToString(Int32) -> String InternalCall contract".into());
        }
        f.impl_flags = 0;
        f.body = vec![Op::String(String::new()), Op::Return];
        bindings.push(json!({"definition": row["definition"], "name": row["name"],
            "compiledIndex": row["compiledIndex"], "implementation": "int32-to-string-v1",
            "symbol": "neoclr_int32_to_string_v1", "storage": "caller-owned-text-arena-v4"}));
    }
    Ok(bindings)
}

/// UTF-8 grapheme characters retain immutable text; validation uses the same
/// pinned Unicode segmentation dependency as the interpreter, statically linked.
pub fn character_text(input: &mut neoclr::Module, selection: &Value) -> Result<Vec<Value>, Error> {
    let mut bindings = vec![];
    for row in selection["functions"].as_array().ok_or("missing selection inventory")? {
        let (parameter, result, implementation) = match row["name"].as_str() {
            Some("neoCLR.Runtime.CharFromString") => (Type::String, Type::Char, "char-from-string-v1"),
            Some("neoCLR.Runtime.CharText") => (Type::Char, Type::String, "char-text-v1"),
            _ => continue,
        };
        let f = &mut input.functions[row["compiledIndex"].as_u64().ok_or("missing compiled index")? as usize];
        if f.name != row["name"].as_str().unwrap()
            || f.owner.is_some() || f.instance || f.receiver_byref || f.receiver_readonly
            || f.parameters != [parameter] || f.returns != result || f.no_result
            || f.impl_flags != 0x1000 || f.pinvoke.is_some() || !f.body.is_empty()
            || !f.locals.is_empty() || f.is_virtual || f.is_override || f.is_abstract
            || !f.generic_parameters.is_empty() || !f.generic_arguments.is_empty()
            || !f.generic_constraints.is_empty() || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty() || !f.out_when_true.is_empty() || !f.readonly_parameters.is_empty()
        { return Err("native character binding requires exact CharFromString(String) -> Char or CharText(Char) -> String InternalCall contract".into()); }
        f.impl_flags = 0;
        if result == Type::Char {
            f.locals = vec![Type::Char];
            f.body = vec![Op::LocalAddress(0), Op::InitializeObject(Type::Char), Op::Load(0), Op::Return];
        } else {
            f.body = vec![Op::String(String::new()), Op::Return];
        }
        bindings.push(json!({"definition": row["definition"], "name": row["name"],
            "compiledIndex": row["compiledIndex"], "implementation": implementation,
            "validationSymbol": if result == Type::Char { Some("neoclr_is_single_grapheme_v1") } else { None },
            "storage": "borrowed immutable image or invocation text; no allocation"}));
    }
    Ok(bindings)
}

/// Explicit wide integer formatting and ARM64 native-width conversion.
pub fn integer_text(input: &mut neoclr::Module, selection: &Value) -> Result<Vec<Value>, Error> {
    let mut bindings = vec![];
    for row in selection["functions"].as_array().ok_or("missing selection inventory")? {
        let (parameter, result, implementation) = match row["name"].as_str() {
            Some("neoCLR.Runtime.Int64ToString") => (Type::Int64, Type::String, "int64-to-string-v1"),
            Some("neoCLR.Runtime.UInt64ToString") => (Type::UInt64, Type::String, "uint64-to-string-v1"),
            Some("neoCLR.Runtime.IntPtrToInt64") => (Type::IntPtr, Type::Int64, "native-integer-to64-v1"),
            Some("neoCLR.Runtime.UIntPtrToUInt64") => (Type::UIntPtr, Type::UInt64, "native-integer-to64-v1"),
            _ => continue,
        };
        let f = &mut input.functions[row["compiledIndex"].as_u64().ok_or("missing compiled index")? as usize];
        if f.name != row["name"].as_str().unwrap()
            || f.owner.is_some() || f.instance || f.receiver_byref || f.receiver_readonly
            || f.parameters != [parameter] || f.returns != result || f.no_result
            || f.impl_flags != 0x1000 || f.pinvoke.is_some() || !f.body.is_empty()
            || !f.locals.is_empty() || f.is_virtual || f.is_override || f.is_abstract
            || !f.generic_parameters.is_empty() || !f.generic_arguments.is_empty()
            || !f.generic_constraints.is_empty() || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty() || !f.out_when_true.is_empty() || !f.readonly_parameters.is_empty()
        { return Err("native integer text binding requires exact reserved Int64/UInt64 formatting or native-integer conversion InternalCall contract".into()); }
        f.impl_flags = 0;
        f.body = if result == Type::String {
            vec![Op::String(String::new()), Op::Return]
        } else {
            vec![Op::Int64(0), Op::Return]
        };
        bindings.push(json!({"definition": row["definition"], "name": row["name"],
            "compiledIndex": row["compiledIndex"], "implementation": implementation,
            "symbol": match implementation {
                "int64-to-string-v1" => Some("neoclr_int64_to_string_v1"),
                "uint64-to-string-v1" => Some("neoclr_uint64_to_string_v1"),
                _ => None,
            }, "storage": if result == Type::String { "caller-owned-text-arena-v4" } else { "64-bit target bit-preserving conversion" }}));
    }
    Ok(bindings)
}

/// Raw stream services keep recoverable I/O/range errors in managed Result wrappers.
pub fn console_stream_output(input: &mut neoclr::Module, selection: &Value) -> Result<Vec<Value>, Error> {
    let mut bindings = vec![];
    for row in selection["functions"].as_array().ok_or("missing selection inventory")? {
        let name = row["name"].as_str().unwrap();
        let (parameters, implementation, symbol) = match name {
            "neoCLR.Runtime.ConsoleWriteBytes" => (vec![Type::Boolean, Type::ArrayRef(Box::new(Type::Byte)), Type::Int32, Type::Int32], "console-write-bytes-v1", "neoclr_console_write_bytes_v1"),
            "neoCLR.Runtime.ConsoleFlush" => (vec![Type::Boolean], "console-flush-v1", "neoclr_console_flush_v1"),
            _ => continue,
        };
        let f = &mut input.functions[row["compiledIndex"].as_u64().ok_or("missing compiled index")? as usize];
        if f.name != name || f.owner.is_some() || f.instance || f.receiver_byref || f.receiver_readonly
            || f.parameters != parameters || f.returns != Type::Value || f.no_result
            || f.impl_flags != 0x1000 || f.pinvoke.is_some() || !f.body.is_empty() || !f.locals.is_empty()
            || f.is_virtual || f.is_override || f.is_abstract || !f.generic_parameters.is_empty()
            || !f.generic_arguments.is_empty() || !f.generic_constraints.is_empty()
            || !f.interface_implementations.is_empty() || !f.out_parameters.is_empty()
            || !f.out_when_true.is_empty() || !f.readonly_parameters.is_empty() {
            return Err("native stream output binding requires exact ConsoleWriteBytes(Boolean, arrayref<Byte>, Int32, Int32) or ConsoleFlush(Boolean) -> Value InternalCall contract".into());
        }
        f.impl_flags = 0;
        f.body = vec![Op::Int(0), Op::PackValue(Type::Int32), Op::Return];
        bindings.push(json!({"definition":row["definition"],"name":row["name"],"compiledIndex":row["compiledIndex"],"implementation":implementation,"symbol":symbol}));
    }
    Ok(bindings)
}
