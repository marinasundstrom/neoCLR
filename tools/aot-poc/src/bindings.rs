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
            Some("neoCLR.Runtime.ParseInt32") => (Type::String, Type::Value, "parse-int32-v1"),
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
        { return Err("native integer text binding requires exact reserved Int32 parsing, Int64/UInt64 formatting or native-integer conversion InternalCall contract".into()); }
        f.impl_flags = 0;
        f.body = if result == Type::String {
            vec![Op::String(String::new()), Op::Return]
        } else if result == Type::Value {
            vec![Op::Void, Op::PackValue(Type::Void), Op::Return]
        } else {
            vec![Op::Int64(0), Op::Return]
        };
        bindings.push(json!({"definition": row["definition"], "name": row["name"],
            "compiledIndex": row["compiledIndex"], "implementation": implementation,
            "symbol": match implementation {
                "parse-int32-v1" => Some("neoclr_parse_int32_v1"),
                "int64-to-string-v1" => Some("neoclr_int64_to_string_v1"),
                "uint64-to-string-v1" => Some("neoclr_uint64_to_string_v1"),
                _ => None,
            }, "storage": if result == Type::String { "caller-owned-text-arena-v4" } else if result == Type::Value { "allocation-free erased Int32 or Byte parse error" } else { "64-bit target bit-preserving conversion" }}));
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

/// UTF-8-native text operations; ordinary String wrappers remain CIL.
pub fn utf8_text(input: &mut neoclr::Module, selection: &Value) -> Result<Vec<Value>, Error> {
    let mut bindings = vec![];
    for row in selection["functions"].as_array().ok_or("missing selection inventory")? {
        let name = row["name"].as_str().unwrap();
        let (parameters, result, implementation, symbol) = match name {
            "neoCLR.Runtime.StringContainsOrdinal" => (vec![Type::String, Type::String], Type::Boolean, "string-contains-ordinal-v1", "neoclr_string_contains_ordinal_v1"),
            "neoCLR.Runtime.StringStartsWithOrdinal" => (vec![Type::String, Type::String], Type::Boolean, "string-starts-with-ordinal-v1", "neoclr_string_starts_with_ordinal_v1"),
            "neoCLR.Runtime.StringEndsWithOrdinal" => (vec![Type::String, Type::String], Type::Boolean, "string-ends-with-ordinal-v1", "neoclr_string_ends_with_ordinal_v1"),
            "neoCLR.Runtime.StringJoinParts" => (vec![Type::ArrayRef(Box::new(Type::String)), Type::Int32, Type::String, Type::Int32], Type::String, "string-join-parts-v1", "neoclr_string_join_parts_v1"),
            "neoCLR.Runtime.StringConcat" => (vec![Type::String, Type::String], Type::String, "string-concat-v1", "neoclr_string_concat_v1"),
            "neoCLR.Runtime.Utf8Decode" => (vec![Type::ArrayRef(Box::new(Type::Byte))], Type::Value, "utf8-decode-v1", "neoclr_utf8_decode_v1"),
            "neoCLR.Runtime.Utf8Encode" => (vec![Type::String], Type::Array(Box::new(Type::Byte)), "utf8-encode-v1", "neoclr_utf8_encode_v1"),
            "neoCLR.Runtime.StringByteCount" => (vec![Type::String], Type::Int32, "string-byte-count-v1", "neoclr_string_byte_count_v1"),
            "neoCLR.Runtime.StringSliceUtf8" => (vec![Type::String, Type::Int32, Type::Int32], Type::Value, "string-slice-utf8-v1", "neoclr_string_slice_utf8_v1"),
            _ => continue,
        };
        let f = &mut input.functions[row["compiledIndex"].as_u64().ok_or("missing compiled index")? as usize];
        if f.name != name || f.owner.is_some() || f.instance || f.receiver_byref || f.receiver_readonly
            || f.parameters != parameters || f.returns != result || f.no_result
            || f.impl_flags != 0x1000 || f.pinvoke.is_some() || !f.body.is_empty() || !f.locals.is_empty()
            || f.is_virtual || f.is_override || f.is_abstract || !f.generic_parameters.is_empty()
            || !f.generic_arguments.is_empty() || !f.generic_constraints.is_empty()
            || !f.interface_implementations.is_empty() || !f.out_parameters.is_empty()
            || !f.out_when_true.is_empty() || !f.readonly_parameters.is_empty() {
            return Err("native UTF-8 binding requires exact StringByteCount(String) -> Int32 or StringSliceUtf8(String, Int32, Int32) -> Value or Utf8Encode(String) -> Byte[] or Utf8Decode(arrayref<Byte>) -> Value or StringConcat(String, String) -> String or ordinal String predicates(String, String) -> Boolean InternalCall contract".into());
        }
        f.impl_flags = 0;
        // A verifier-valid nonreturning placeholder needs no unsupported array
        // constructor; only the reported InternalCall is lowered to native code.
        f.body = if matches!(result, Type::Array(_)) { vec![Op::Branch(0)] }
            else if result == Type::Boolean { vec![Op::Bool(false), Op::Return] }
            else if result == Type::String { vec![Op::String(String::new()), Op::Return] }
            else if result == Type::Value { vec![Op::String(String::new()), Op::PackValue(Type::String), Op::Return] }
            else { vec![Op::Int(0), Op::Return] };
        bindings.push(json!({"definition":row["definition"],"name":row["name"],"compiledIndex":row["compiledIndex"],"implementation":implementation,"symbol":symbol}));
    }
    Ok(bindings)
}

/// Bounded synchronous listener lifecycle, with resources owned by a host scope.
pub fn socket_listener(input: &mut neoclr::Module, selection: &Value) -> Result<Vec<Value>, Error> {
    let mut bindings = vec![];
    for row in selection["functions"].as_array().ok_or("missing selection inventory")? {
        let (parameters, implementation, symbol) = match row["name"].as_str() {
            Some("neoCLR.Runtime.SocketListen") => (vec![Type::String, Type::Int32, Type::Int32], "socket-listen-v1", "neoclr_socket_listen_v1"),
            Some("neoCLR.Runtime.SocketLocalPort") => (vec![Type::Int64], "socket-local-port-v1", "neoclr_socket_local_port_v1"),
            Some("neoCLR.Runtime.SocketClose") => (vec![Type::Int64], "socket-close-v1", "neoclr_socket_close_v1"),
            _ => continue,
        };
        let f = &mut input.functions[row["compiledIndex"].as_u64().ok_or("missing compiled index")? as usize];
        if f.name != row["name"].as_str().unwrap()
            || f.owner.is_some() || f.instance || f.receiver_byref || f.receiver_readonly
            || f.parameters != parameters || f.returns != Type::Value || f.no_result
            || f.impl_flags != 0x1000 || f.pinvoke.is_some() || !f.body.is_empty()
            || !f.locals.is_empty() || f.is_virtual || f.is_override || f.is_abstract
            || !f.generic_parameters.is_empty() || !f.generic_arguments.is_empty()
            || !f.generic_constraints.is_empty() || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty() || !f.out_when_true.is_empty() || !f.readonly_parameters.is_empty()
        { return Err("native socket listener binding requires exact reserved SocketListen/SocketLocalPort/SocketClose InternalCall contract".into()); }
        f.impl_flags = 0;
        f.body = vec![Op::Void, Op::PackValue(Type::Void), Op::Return];
        bindings.push(json!({"definition": row["definition"], "name": row["name"],
            "compiledIndex": row["compiledIndex"], "implementation": implementation, "symbol": symbol,
            "storage": "explicit host socket scope; 64 listeners; no guest references or callbacks retained"}));
    }
    Ok(bindings)
}

pub fn socket_accept(input: &mut neoclr::Module, selection: &Value) -> Result<Vec<Value>, Error> {
    let mut bindings = vec![];
    for row in selection["functions"].as_array().ok_or("missing selection inventory")? {
        let (parameters, result, implementation, symbol) = match row["name"].as_str() {
            Some("neoCLR.Runtime.SocketAccept") => (vec![Type::Int64, neoclr::assembler::parse_type("fn<Void>").expect("fixed callback signature")], Type::Value, "socket-accept-v1", "neoclr_socket_accept_v1"),
            Some("neoCLR.Runtime.SocketConnectResult") => (vec![Type::Int64], Type::Value, "socket-connect-result-v1", "neoclr_socket_connect_result_v1"),
            Some("neoCLR.Runtime.SocketCancel") => (vec![Type::Int64], Type::Boolean, "socket-cancel-v1", "neoclr_socket_cancel_v1"),
            _ => continue,
        };
        let f = &mut input.functions[row["compiledIndex"].as_u64().ok_or("missing compiled index")? as usize];
        if f.name != row["name"].as_str().unwrap()
            || f.owner.is_some() || f.instance || f.receiver_byref || f.receiver_readonly
            || f.parameters != parameters || f.returns != result || f.no_result
            || f.impl_flags != 0x1000 || f.pinvoke.is_some() || !f.body.is_empty()
            || !f.locals.is_empty() || f.is_virtual || f.is_override || f.is_abstract
            || !f.generic_parameters.is_empty() || !f.generic_arguments.is_empty()
            || !f.generic_constraints.is_empty() || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty() || !f.out_when_true.is_empty() || !f.readonly_parameters.is_empty()
        { return Err("native socket accept binding requires exact reserved Accept/ConnectResult/Cancel InternalCall contract".into()); }
        f.impl_flags = 0;
        f.body = if result == Type::Boolean { vec![Op::Bool(false), Op::Return] }
            else { vec![Op::Void, Op::PackValue(Type::Void), Op::Return] };
        bindings.push(json!({"definition": row["definition"], "name": row["name"],
            "compiledIndex": row["compiledIndex"], "implementation": implementation, "symbol": symbol,
            "storage": "explicit host socket scope; 64 accept operations; rooted callbacks; owner-thread deferred polling"}));
    }
    Ok(bindings)
}

pub fn socket_transfer(input: &mut neoclr::Module, selection: &Value) -> Result<Vec<Value>, Error> {
    let mut bindings = vec![];
    for row in selection["functions"].as_array().ok_or("missing selection inventory")? {
        let (parameters, result, implementation, symbol) = match row["name"].as_str() {
            Some("neoCLR.Runtime.SocketReceive") => (vec![Type::Int64, Type::ArrayRef(Box::new(Type::Byte)), Type::Int32, Type::Int32, neoclr::assembler::parse_type("fn<Void>").expect("fixed callback signature")], Type::Value, "socket-receive-v1", "neoclr_socket_receive_v1"),
            Some("neoCLR.Runtime.SocketSend") => (vec![Type::Int64, Type::ArrayRef(Box::new(Type::Byte)), Type::Int32, Type::Int32, neoclr::assembler::parse_type("fn<Void>").expect("fixed callback signature")], Type::Value, "socket-send-v1", "neoclr_socket_send_v1"),
            Some("neoCLR.Runtime.SocketTransferResult") => (vec![Type::Int64], Type::Value, "socket-transfer-result-v1", "neoclr_socket_transfer_result_v1"),
            Some("neoCLR.Runtime.SocketDeadlineAfter") => (vec![Type::Int32], Type::Int64, "socket-deadline-after-v1", "neoclr_socket_deadline_after_v1"),
            Some("neoCLR.Runtime.SocketDeadlineExpired") => (vec![Type::Int64], Type::Boolean, "socket-deadline-expired-v1", "neoclr_socket_deadline_expired_v1"),
            Some("neoCLR.Runtime.SocketReceiveUntil") => (vec![Type::Int64, Type::ArrayRef(Box::new(Type::Byte)), Type::Int32, Type::Int32, Type::Int64, neoclr::assembler::parse_type("fn<Void>").expect("fixed callback signature")], Type::Value, "socket-receive-until-v1", "neoclr_socket_receive_until_v1"),
            Some("neoCLR.Runtime.SocketSendUntil") => (vec![Type::Int64, Type::ArrayRef(Box::new(Type::Byte)), Type::Int32, Type::Int32, Type::Int64, neoclr::assembler::parse_type("fn<Void>").expect("fixed callback signature")], Type::Value, "socket-send-until-v1", "neoclr_socket_send_until_v1"),
            _ => continue,
        };
        let f = &mut input.functions[row["compiledIndex"].as_u64().ok_or("missing compiled index")? as usize];
        if f.name != row["name"].as_str().unwrap()
            || f.owner.is_some() || f.instance || f.receiver_byref || f.receiver_readonly
            || f.parameters != parameters || f.returns != result || f.no_result
            || f.impl_flags != 0x1000 || f.pinvoke.is_some() || !f.body.is_empty()
            || !f.locals.is_empty() || f.is_virtual || f.is_override || f.is_abstract
            || !f.generic_parameters.is_empty() || !f.generic_arguments.is_empty()
            || !f.generic_constraints.is_empty() || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty() || !f.out_when_true.is_empty() || !f.readonly_parameters.is_empty()
        { return Err("native socket transfer binding requires exact reserved Receive/Send/TransferResult InternalCall contract".into()); }
        f.impl_flags = 0;
        f.body = if result == Type::Boolean { vec![Op::Bool(false), Op::Return] }
            else if result == Type::Int64 { vec![Op::Int64(0), Op::Return] }
            else { vec![Op::Void, Op::PackValue(Type::Void), Op::Return] };
        bindings.push(json!({"definition": row["definition"], "name": row["name"],
            "compiledIndex": row["compiledIndex"], "implementation": implementation, "symbol": symbol,
            "storage": "explicit host socket scope; 64 shared socket operations; rooted callbacks; owner-thread deferred polling"}));
    }
    Ok(bindings)
}

/// Reference-arena identity uses the same verified primitive as CIL ref.equal.
pub fn object_reference_equals(input: &mut neoclr::Module, selection: &Value) -> Result<Vec<Value>, Error> {
    let mut bindings = vec![];
    for row in selection["functions"].as_array().ok_or("missing selection inventory")? {
        if row["name"] != "neoCLR.Runtime.ObjectReferenceEquals" { continue; }
        let f = &mut input.functions[row["compiledIndex"].as_u64().ok_or("missing compiled index")? as usize];
        let object = Type::Named("System.Object".into());
        if f.name != row["name"].as_str().unwrap()
            || f.owner.is_some() || f.instance || f.receiver_byref || f.receiver_readonly
            || f.parameters != [object.clone(), object] || f.returns != Type::Boolean || f.no_result
            || f.impl_flags != 0x1000 || f.pinvoke.is_some() || !f.body.is_empty()
            || !f.locals.is_empty() || f.is_virtual || f.is_override || f.is_abstract
            || !f.generic_parameters.is_empty() || !f.generic_arguments.is_empty()
            || !f.generic_constraints.is_empty() || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty() || !f.out_when_true.is_empty() || !f.readonly_parameters.is_empty()
        { return Err("native identity binding requires exact ObjectReferenceEquals(Object,Object) -> Boolean InternalCall contract".into()); }
        f.impl_flags = 0;
        f.body = vec![Op::Arg(0), Op::Arg(1), Op::ReferenceEqual, Op::Return];
        bindings.push(json!({"definition": row["definition"], "name": row["name"],
            "compiledIndex": row["compiledIndex"], "implementation": "object-reference-equals-intrinsic-v1",
            "semantics": "reference identity including null; no content comparison or allocation"}));
    }
    Ok(bindings)
}

#[cfg(test)]
mod identity_tests {
    use super::*;

    #[test]
    fn identity_binding_requires_exact_reserved_contract() {
        let mut source = neoclr::assemble(".module Identity\n.type class System.Object\n.end\n.function neoCLR.Runtime.ObjectReferenceEquals(System.Object left, System.Object right) -> Boolean\nldc.bool false\nret\n.end\n").unwrap();
        source.functions[0].impl_flags = 0x1000;
        source.functions[0].body.clear();
        let inventory = json!({"functions":[{"name":"neoCLR.Runtime.ObjectReferenceEquals","compiledIndex":0}]});
        for change in 0..6 {
            let mut input = source.clone();
            match change {
                0 => input.functions[0].impl_flags = 0,
                1 => input.functions[0].body = vec![Op::Bool(false), Op::Return],
                2 => input.functions[0].parameters[0] = Type::String,
                3 => input.functions[0].returns = Type::Int32,
                4 => input.functions[0].receiver_byref = true,
                _ => input.functions[0].out_parameters = vec![0],
            }
            assert!(object_reference_equals(&mut input, &inventory).is_err());
        }
        assert_eq!(object_reference_equals(&mut source, &inventory).unwrap().len(), 1);
        assert_eq!(source.functions[0].impl_flags, 0);
        assert!(matches!(source.functions[0].body[2], Op::ReferenceEqual));
    }
}

#[cfg(test)]
mod deadline_tests {
    use super::*;
    #[test]
    fn deadlines_require_exact_reserved_signatures_and_internal_bodies() {
        for (name, signature) in [
            ("SocketDeadlineAfter", "Int32 milliseconds) -> Int64"),
            ("SocketDeadlineExpired", "Int64 deadline) -> Boolean"),
            ("SocketReceiveUntil", "Int64 socket, arrayref<Byte> buffer, Int32 offset, Int32 count, Int64 deadline, fn<Void> callback) -> Value"),
            ("SocketSendUntil", "Int64 socket, arrayref<Byte> buffer, Int32 offset, Int32 count, Int64 deadline, fn<Void> callback) -> Value"),
        ] {
            let source = neoclr::assemble(&format!(".module Contract\n.function neoCLR.Runtime.{name}({signature}\n.methodimpl InternalCall\n.end")).unwrap();
            let inventory = json!({"functions":[{"name":format!("neoCLR.Runtime.{name}"),"compiledIndex":0}]});
            for change in 0..6 {
                let mut input = source.clone();
                match change {
                    0 => input.functions[0].impl_flags = 0,
                    1 => input.functions[0].body = vec![Op::Void, Op::Return],
                    2 => input.functions[0].parameters[0] = Type::String,
                    3 => input.functions[0].returns = Type::Void,
                    4 => input.functions[0].no_result = true,
                    _ => input.functions[0].out_parameters = vec![0],
                }
                assert!(socket_transfer(&mut input, &inventory).is_err(), "{name}: mutation {change}");
            }
            assert_eq!(socket_transfer(&mut source.clone(), &inventory).unwrap().len(), 1);
        }
    }
}
