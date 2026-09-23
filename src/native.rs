//! Explicit runtime binding registry. A name alone never activates host dispatch.
use crate::{
    Fault, Value,
    metadata::{Function, Type},
};
use unicode_segmentation::UnicodeSegmentation;

pub(crate) enum Binding {
    EnvironmentArguments,
    EnvironmentCurrentDirectory,
    EnvironmentVariable,
    PathCombine,
    PathGetFileName,
    UnixTimeTicks,
    UnixTimeToLocal,
    Math(crate::math::Operation),
    Reflection(crate::reflection::Query),
    ObjectTypeHandle,
    ExecutingAssembly,
    CurrentTaskQueue,
    DefaultTaskQueue,
    RegisterDefaultTaskQueue,
    StartWorker(bool),
    JoinWorker,
    NotifyWorker,
    AssemblyInfo(crate::assembly_info::Query),
    TypeName,
    TypeEquals,
    TypeArgumentCount,
    TypeArgument,
    ParseInt32,
    Int32ToString,
    WriteLine,
    Fault,
    CharCategory,
    Utf8Encode,
    Utf8Decode,
    StringConcat,
    StringGraphemeCount,
    CharFromString,
    CharText,
    StringGraphemes,
    StringScalars,
    StringByteCount,
    StringCompareOrdinal,
    StringContainsOrdinal,
    StringStartsWithOrdinal,
    StringEndsWithOrdinal,
    StringSliceUtf8,
    FileResource(crate::file_streams::Operation),
    ReadAllText,
    WriteAllText,
    ConsoleReadByte,
}

pub(crate) fn bind(function: &Function) -> Result<Binding, Fault> {
    if !function.is_internal_call() || function.instance || function.owner.is_some() {
        return Err(Fault::new("native binding requires InternalCall metadata"));
    }
    if let Some((operation, arity)) = crate::math::Operation::binding(&function.name) {
        if function.parameters != vec![Type::Double; arity] || function.returns != Type::Double {
            return Err(Fault::new("math binding signature mismatch"));
        }
        return Ok(Binding::Math(operation));
    }
    if let Some((query, count, result)) = crate::assembly_info::Query::binding(&function.name) {
        if function.parameters != vec![Type::String; count]
            || function.returns != crate::assembler::parse_type(result)?
        {
            return Err(Fault::new("assembly service signature mismatch"));
        }
        return Ok(Binding::AssemblyInfo(query));
    }
    if let Some((query, integer, returns)) = crate::reflection::Query::binding(&function.name) {
        let expected = if integer {
            vec![Type::RuntimeTypeHandle, Type::Int32]
        } else {
            vec![Type::RuntimeTypeHandle]
        };
        // The Raven model uses TypeInfo throughout; the historical Neo profile
        // retains Type. Admit only the corresponding exact result signature.
        fn info_result(ty: &Type) -> Type {
            match ty {
                Type::Named(name) if name == "System.Type" => {
                    Type::from_name("System.Introspection.TypeInfo")
                }
                Type::Array(element) => Type::Array(Box::new(info_result(element))),
                Type::Constructed {
                    definition,
                    arguments,
                } => Type::Constructed {
                    definition: definition.clone(),
                    arguments: arguments.iter().map(info_result).collect(),
                },
                _ => ty.clone(),
            }
        }
        if function.parameters != expected
            || (function.returns != returns && function.returns != info_result(&returns))
        {
            return Err(Fault::new("reflection binding signature mismatch"));
        }
        return Ok(Binding::Reflection(query));
    }
    let (binding, returns) = match (function.name.as_str(), function.parameters.as_slice()) {
        ("neoCLR.Runtime.EnvironmentArguments", []) => (
            Binding::EnvironmentArguments,
            Type::Array(Box::new(Type::String)),
        ),
        ("neoCLR.Runtime.EnvironmentCurrentDirectory", []) => {
            (Binding::EnvironmentCurrentDirectory, Type::Value)
        }
        ("neoCLR.Runtime.EnvironmentVariable", [Type::String]) => {
            (Binding::EnvironmentVariable, Type::Value)
        }
        ("neoCLR.Runtime.PathCombine", [Type::String, Type::String]) => {
            (Binding::PathCombine, Type::String)
        }
        ("neoCLR.Runtime.PathGetFileName", [Type::String]) => {
            (Binding::PathGetFileName, Type::String)
        }
        ("neoCLR.Runtime.UnixTimeToLocal", [Type::Int64]) => {
            (Binding::UnixTimeToLocal, Type::Array(Box::new(Type::Int32)))
        }
        ("neoCLR.Runtime.UnixTimeTicks", []) => (Binding::UnixTimeTicks, Type::Int64),
        ("neoCLR.Runtime.ParseInt32", [Type::String]) => (Binding::ParseInt32, Type::Value),
        ("neoCLR.Runtime.Int32ToString", [Type::Int32]) => (Binding::Int32ToString, Type::String),
        ("neoCLR.Runtime.Fault", [Type::String]) => (Binding::Fault, Type::Void),
        ("neoCLR.Runtime.WriteLine", [Type::String]) => (Binding::WriteLine, Type::Void),
        ("neoCLR.Runtime.CharCategory", [Type::UInt32]) => (Binding::CharCategory, Type::Int32),
        ("neoCLR.Runtime.Utf8Encode", [Type::String]) => {
            (Binding::Utf8Encode, Type::Array(Box::new(Type::Byte)))
        }
        ("neoCLR.Runtime.Utf8Decode", [Type::ArrayRef(element)]) if **element == Type::Byte => {
            (Binding::Utf8Decode, Type::Value)
        }
        ("neoCLR.Runtime.StringConcat", [Type::String, Type::String]) => {
            (Binding::StringConcat, Type::String)
        }
        ("neoCLR.Runtime.StringCompareOrdinal", [Type::String, Type::String]) => {
            (Binding::StringCompareOrdinal, Type::Int32)
        }
        ("neoCLR.Runtime.StringContainsOrdinal", [Type::String, Type::String]) => {
            (Binding::StringContainsOrdinal, Type::Boolean)
        }
        ("neoCLR.Runtime.StringStartsWithOrdinal", [Type::String, Type::String]) => {
            (Binding::StringStartsWithOrdinal, Type::Boolean)
        }
        ("neoCLR.Runtime.StringEndsWithOrdinal", [Type::String, Type::String]) => {
            (Binding::StringEndsWithOrdinal, Type::Boolean)
        }
        ("neoCLR.Runtime.StringGraphemeCount", [Type::String]) => {
            (Binding::StringGraphemeCount, Type::Int32)
        }
        ("neoCLR.Runtime.CharFromString", [Type::String]) => (Binding::CharFromString, Type::Char),
        ("neoCLR.Runtime.CharText", [Type::Char]) => (Binding::CharText, Type::String),
        ("neoCLR.Runtime.StringGraphemes", [Type::String]) => {
            (Binding::StringGraphemes, Type::Array(Box::new(Type::Char)))
        }
        ("neoCLR.Runtime.StringScalars", [Type::String]) => {
            (Binding::StringScalars, Type::Array(Box::new(Type::UInt32)))
        }
        ("neoCLR.Runtime.StringByteCount", [Type::String]) => {
            (Binding::StringByteCount, Type::Int32)
        }
        ("neoCLR.Runtime.StringSliceUtf8", [Type::String, Type::Int32, Type::Int32]) => {
            (Binding::StringSliceUtf8, Type::Value)
        }
        ("neoCLR.Runtime.FileOpenRead", [Type::String]) => (
            Binding::FileResource(crate::file_streams::Operation::OpenRead),
            Type::Value,
        ),
        ("neoCLR.Runtime.FileOpenWrite", [Type::String]) => (
            Binding::FileResource(crate::file_streams::Operation::OpenWrite),
            Type::Value,
        ),
        ("neoCLR.Runtime.FileCreateNew", [Type::String]) => (
            Binding::FileResource(crate::file_streams::Operation::CreateNew),
            Type::Value,
        ),
        ("neoCLR.Runtime.FileReadChunk", [Type::Int32, Type::Int32]) => (
            Binding::FileResource(crate::file_streams::Operation::Read),
            Type::Value,
        ),
        (
            "neoCLR.Runtime.FileReadInto",
            [
                Type::Int32,
                Type::ArrayRef(element),
                Type::Int32,
                Type::Int32,
            ],
        ) if **element == Type::Byte => (
            Binding::FileResource(crate::file_streams::Operation::ReadInto),
            Type::Value,
        ),
        (
            "neoCLR.Runtime.FileWriteChunk",
            [
                Type::Int32,
                Type::ArrayRef(element),
                Type::Int32,
                Type::Int32,
            ],
        ) if **element == Type::Byte => (
            Binding::FileResource(crate::file_streams::Operation::Write),
            Type::Value,
        ),
        ("neoCLR.Runtime.FileFlush", [Type::Int32]) => (
            Binding::FileResource(crate::file_streams::Operation::Flush),
            Type::Value,
        ),
        ("neoCLR.Runtime.FileClose", [Type::Int32]) => (
            Binding::FileResource(crate::file_streams::Operation::Close),
            Type::Value,
        ),
        ("neoCLR.Runtime.FilePosition", [Type::Int32]) => (
            Binding::FileResource(crate::file_streams::Operation::Position),
            Type::Value,
        ),
        ("neoCLR.Runtime.FileSeek", [Type::Int32, Type::Int64]) => (
            Binding::FileResource(crate::file_streams::Operation::Seek),
            Type::Value,
        ),
        ("neoCLR.Runtime.StorageList", [Type::String, Type::Int32]) => (
            Binding::FileResource(crate::file_streams::Operation::List),
            Type::Value,
        ),
        ("neoCLR.Runtime.StorageKind", [Type::String]) => (
            Binding::FileResource(crate::file_streams::Operation::Kind),
            Type::Value,
        ),
        ("neoCLR.Runtime.StorageCreateDirectory", [Type::String]) => (
            Binding::FileResource(crate::file_streams::Operation::CreateDirectory),
            Type::Value,
        ),
        ("neoCLR.Runtime.WriteAllText", [Type::String, Type::String, Type::Int32]) => {
            (Binding::WriteAllText, Type::Int32)
        }
        ("neoCLR.Runtime.ReadAllText", [Type::String, Type::Int32]) => {
            (Binding::ReadAllText, Type::Value)
        }
        ("neoCLR.Runtime.ConsoleReadByte", []) => (Binding::ConsoleReadByte, Type::Value),
        (
            "neoCLR.Runtime.StartWorker",
            [
                Type::Constructed {
                    definition,
                    arguments,
                },
                Type::String,
            ],
        ) if definition == "System.Func" && arguments == &[Type::String, Type::String] => {
            (Binding::StartWorker(false), Type::Int32)
        }
        (
            "neoCLR.Runtime.QueueWorker",
            [
                Type::Constructed {
                    definition,
                    arguments,
                },
                Type::String,
            ],
        ) if definition == "System.Func" && arguments == &[Type::String, Type::String] => {
            (Binding::StartWorker(true), Type::Int32)
        }
        ("neoCLR.Runtime.JoinWorker", [Type::Int32]) => (Binding::JoinWorker, Type::String),
        ("neoCLR.Runtime.NotifyWorker", [Type::Int32, callback])
            if *callback == crate::assembler::parse_type("System.Func<Void>")? =>
        {
            (Binding::NotifyWorker, Type::Void)
        }
        ("neoCLR.Runtime.DefaultTaskQueue", []) => (
            Binding::DefaultTaskQueue,
            Type::from_name("System.Tasks.TaskQueue"),
        ),
        ("neoCLR.Runtime.RegisterDefaultTaskQueue", [queue])
            if queue == &Type::from_name("System.Tasks.TaskQueue") =>
        {
            (Binding::RegisterDefaultTaskQueue, Type::Void)
        }
        ("neoCLR.Runtime.CurrentTaskQueue", []) => (
            Binding::CurrentTaskQueue,
            Type::from_name("System.Tasks.TaskQueue"),
        ),
        ("neoCLR.Runtime.ExecutingAssembly", []) => (
            Binding::ExecutingAssembly,
            Type::from_name("System.Introspection.AssemblyInfo"),
        ),
        ("neoCLR.Runtime.ObjectTypeHandle", [Type::Named(name)]) if name == "System.Object" => {
            (Binding::ObjectTypeHandle, Type::RuntimeTypeHandle)
        }
        ("neoCLR.Runtime.TypeName", [Type::RuntimeTypeHandle]) => (Binding::TypeName, Type::String),
        ("neoCLR.Runtime.TypeEquals", [Type::RuntimeTypeHandle, Type::RuntimeTypeHandle]) => {
            (Binding::TypeEquals, Type::Boolean)
        }
        ("neoCLR.Runtime.TypeArgumentCount", [Type::RuntimeTypeHandle]) => {
            (Binding::TypeArgumentCount, Type::Int32)
        }
        ("neoCLR.Runtime.TypeArgument", [Type::RuntimeTypeHandle, Type::Int32]) => {
            (Binding::TypeArgument, Type::RuntimeTypeHandle)
        }
        _ => {
            return Err(Fault::new(format!(
                "no runtime binding for {}({:?})",
                function.name, function.parameters
            )));
        }
    };
    if function.returns != returns {
        return Err(Fault::new(format!(
            "runtime binding return type mismatch for {}",
            function.name
        )));
    }
    Ok(binding)
}

impl Binding {
    pub(crate) fn invoke(
        &self,
        args: Vec<Value>,
        executing_assembly: Option<&str>,
        module: &crate::Module,
        limits: &crate::Limits,
        output: &mut Vec<String>,
        options: &crate::ExecutionOptions,
    ) -> Result<Value, Fault> {
        let console = options.console.as_deref();
        if let Self::Math(operation) = self {
            return operation.invoke(&args);
        }
        if let Self::AssemblyInfo(query) = self {
            return query.invoke(module, &args, limits);
        }
        if let Self::ExecutingAssembly = self {
            return crate::assembly_info::assembly_value(
                module,
                executing_assembly.ok_or_else(|| {
                    Fault::new("ExecutingAssembly requires source origin metadata")
                })?,
            );
        }
        if let Self::Reflection(query) = self {
            return query.invoke(module, &args, limits);
        }
        match (self, args.as_slice()) {
            (Self::ObjectTypeHandle, [value]) => {
                let concrete = match value {
                    Value::ObjectReference(object) => {
                        object.reference.assigned()?;
                        object.concrete_type()
                    }
                    Value::String(_) => Type::String,
                    Value::NullObjectReference(_) => {
                        return Err(Fault::coded(
                            crate::FaultCode::NullReference,
                            "GetType requires a non-null instance",
                        ));
                    }
                    _ => return Err(Fault::new("GetType requires an object reference")),
                };
                Ok(Value::RuntimeTypeHandle(Box::new(
                    crate::type_identity::describe_loaded(module, &concrete)?,
                )))
            }
            (Self::EnvironmentArguments, []) => Ok(Value::Array {
                element: Type::String,
                elements: options
                    .arguments
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect(),
            }),
            (Self::EnvironmentCurrentDirectory, []) => {
                let payload = std::env::current_dir()
                    .ok()
                    .and_then(|p| p.into_os_string().into_string().ok())
                    .map(Value::String)
                    .unwrap_or(Value::Int32(1));
                Ok(Value::Erased(Box::new(payload)))
            }
            (Self::EnvironmentVariable, [Value::String(name)]) => {
                let payload = if name.is_empty() || name.contains(['=', '\0']) {
                    Value::Int32(1)
                } else {
                    match std::env::var(name) {
                        Ok(value) => Value::String(value),
                        Err(std::env::VarError::NotPresent) => Value::Void,
                        Err(std::env::VarError::NotUnicode(_)) => Value::Int32(1),
                    }
                };
                Ok(Value::Erased(Box::new(payload)))
            }
            (Self::PathCombine, [Value::String(left), Value::String(right)]) => {
                Ok(Value::String(crate::path::combine(left, right)))
            }
            (Self::PathGetFileName, [Value::String(path)]) => {
                Ok(Value::String(crate::path::file_name(path)))
            }
            (Self::UnixTimeToLocal, [Value::Int64(ticks)]) => crate::clock::local_at(*ticks),
            (Self::UnixTimeTicks, []) => crate::clock::read_instant(),
            (Self::TypeName, [Value::RuntimeTypeHandle(handle)]) => {
                Ok(Value::String(handle.name.clone()))
            }
            (
                Self::TypeEquals,
                [
                    Value::RuntimeTypeHandle(left),
                    Value::RuntimeTypeHandle(right),
                ],
            ) => Ok(Value::Boolean(left.identity == right.identity)),
            (Self::TypeArgumentCount, [Value::RuntimeTypeHandle(handle)]) => {
                Ok(Value::Int32(handle.generic_arguments.len() as i32))
            }
            (Self::TypeArgument, [Value::RuntimeTypeHandle(handle), Value::Int32(index)]) => {
                let argument = usize::try_from(*index)
                    .ok()
                    .and_then(|index| handle.generic_arguments.get(index))
                    .ok_or_else(|| Fault::new("generic argument index out of range"))?;
                Ok(Value::RuntimeTypeHandle(Box::new(argument.clone())))
            }
            (Self::ConsoleReadByte, []) => {
                // Byte = data, Void = EOF; Int32 1 = Unavailable, 2 = ReadFailed.
                // A worker's bounded output sink does not provide console input.
                let console = if crate::workers::is_worker() {
                    None
                } else {
                    console
                };
                let payload = match console {
                    None => Value::Int32(1),
                    Some(console) => match console.read_byte() {
                        Ok(Some(byte)) => Value::Byte(byte),
                        Ok(None) => Value::Void,
                        Err(_) => Value::Int32(2),
                    },
                };
                Ok(Value::Erased(Box::new(payload)))
            }
            (
                Self::WriteAllText,
                [
                    Value::String(path),
                    Value::String(text),
                    Value::Int32(limit),
                ],
            ) => Ok(Value::Int32(crate::file_io::write_all_text(
                path, text, *limit,
            ))),
            (Self::ReadAllText, [Value::String(path), Value::Int32(max_bytes)]) => {
                crate::file_io::read_all_text(path, *max_bytes)
            }
            (Self::ParseInt32, [Value::String(text)]) => {
                // Validate the whole grammar first: malformed text wins over overflow.
                // Byte 1 = InvalidFormat, Byte 2 = Overflow; Int32 = success.
                let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
                let payload = if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                    Value::Byte(1)
                } else {
                    match text.parse::<i32>() {
                        Ok(n) => Value::Int32(n),
                        Err(_) => Value::Byte(2),
                    }
                };
                Ok(Value::Erased(Box::new(payload)))
            }
            (Self::Int32ToString, [Value::Int32(number)]) => Ok(Value::String(number.to_string())),
            (Self::Fault, [Value::String(message)]) => {
                Err(Fault::coded(crate::FaultCode::UserFault, message))
            }
            (Self::WriteLine, [Value::String(text)]) => {
                if let Some(console) = console {
                    console
                        .write_line(text)
                        .map_err(|_| Fault::new("console output failed"))?;
                } else {
                    output.push(text.clone());
                }
                Ok(Value::Void)
            }
            (Self::CharCategory, [Value::UInt32(value)]) => {
                char::from_u32(*value).ok_or_else(|| Fault::new("invalid Unicode scalar"))?;
                let ranges = crate::char_categories::RANGES;
                let index = ranges.partition_point(|(end, _)| end < value);
                Ok(Value::Int32(i32::from(ranges[index].1)))
            }
            (Self::Utf8Encode, [Value::String(text)]) => {
                crate::reflection::array("Byte", text.bytes().map(|b| Ok(Value::Byte(b))), limits)
            }
            (Self::Utf8Decode, [Value::ObjectReference(bytes)]) => {
                let Value::Array {
                    element: Type::Byte,
                    elements,
                } = bytes.reference.read()?
                else {
                    return Err(Fault::new("UTF-8 decoding requires a byte array"));
                };
                let mut buffer = Vec::new();
                buffer
                    .try_reserve_exact(elements.len())
                    .map_err(|_| Fault::new("UTF-8 decoding allocation failed"))?;
                for value in elements {
                    let Value::Byte(byte) = value else {
                        return Err(Fault::new("UTF-8 decoding requires initialized bytes"));
                    };
                    buffer.push(byte);
                }
                let payload = match String::from_utf8(buffer) {
                    Ok(text) => Value::String(text),
                    Err(_) => Value::Byte(1),
                };
                Ok(Value::Erased(Box::new(payload)))
            }
            (Self::StringConcat, [Value::String(left), Value::String(right)]) => {
                let length = left
                    .len()
                    .checked_add(right.len())
                    .ok_or_else(|| Fault::new("string size overflow"))?;
                let mut value = String::new();
                value
                    .try_reserve_exact(length)
                    .map_err(|_| Fault::new("string allocation failed"))?;
                value.push_str(left);
                value.push_str(right);
                Ok(Value::String(value))
            }
            (Self::StringCompareOrdinal, [Value::String(left), Value::String(right)]) => {
                // Valid UTF-8 byte order agrees with Unicode scalar order.
                let order = left.as_bytes().cmp(right.as_bytes());
                Ok(Value::Int32(match order {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                }))
            }
            (Self::StringContainsOrdinal, [Value::String(value), Value::String(pattern)]) => {
                Ok(Value::Boolean(value.contains(pattern.as_str())))
            }
            (Self::StringStartsWithOrdinal, [Value::String(value), Value::String(pattern)]) => {
                Ok(Value::Boolean(value.starts_with(pattern.as_str())))
            }
            (Self::StringEndsWithOrdinal, [Value::String(value), Value::String(pattern)]) => {
                Ok(Value::Boolean(value.ends_with(pattern.as_str())))
            }
            (Self::StringGraphemeCount, [Value::String(value)]) => {
                let count = i32::try_from(value.graphemes(true).count())
                    .map_err(|_| Fault::new("String grapheme count exceeds Int32 range"))?;
                Ok(Value::Int32(count))
            }
            (Self::CharFromString, [Value::String(value)]) => {
                let character = Value::Char(value.clone());
                character.initialized()?;
                Ok(character)
            }
            (Self::CharText, [Value::Char(value)]) => Ok(Value::String(value.clone())),
            (Self::StringGraphemes, [Value::String(value)]) => crate::reflection::array(
                "Char",
                value.graphemes(true).map(|g| Ok(Value::Char(g.into()))),
                limits,
            ),
            (Self::StringScalars, [Value::String(value)]) => crate::reflection::array(
                "UInt32",
                value.chars().map(|c| Ok(Value::UInt32(c as u32))),
                limits,
            ),
            (Self::StringByteCount, [Value::String(value)]) => {
                let length = i32::try_from(value.len())
                    .map_err(|_| Fault::new("UTF-8 byte count exceeds Int32 range"))?;
                Ok(Value::Int32(length))
            }
            (
                Self::StringSliceUtf8,
                [
                    Value::String(value),
                    Value::Int32(start),
                    Value::Int32(length),
                ],
            ) => {
                // Explicit internal statuses: 1 = OutOfRange, 2 = InvalidBoundary.
                let error = |status| Value::Erased(Box::new(Value::Byte(status)));
                let (Ok(start), Ok(length)) = (usize::try_from(*start), usize::try_from(*length))
                else {
                    return Ok(error(1));
                };
                let Some(end) = start.checked_add(length).filter(|end| *end <= value.len()) else {
                    return Ok(error(1));
                };
                let Some(slice) = value.get(start..end) else {
                    return Ok(error(2));
                };
                let mut result = String::new();
                result
                    .try_reserve_exact(slice.len())
                    .map_err(|_| Fault::new("string allocation failed"))?;
                result.push_str(slice);
                Ok(Value::Erased(Box::new(Value::String(result))))
            }
            _ => Err(Fault::new("invalid native arguments")),
        }
    }
}

#[cfg(test)]
mod character_tests {
    #[test]
    fn bmp_categories_match_pinned_dotnet_10_probe() {
        // FNV-1a over Char.GetUnicodeCategory for all 65,536 UTF-16 units.
        // docs/experiments/character-classification-dotnet, SDK 10.0.100.
        let ranges = crate::char_categories::RANGES;
        assert_eq!(ranges.last().unwrap().0, 0x10FFFF);
        assert!(ranges.windows(2).all(|pair| pair[0].0 < pair[1].0));
        let mut hash = 14_695_981_039_346_656_037u64;
        for value in 0..=u16::MAX {
            let index = ranges.partition_point(|(end, _)| *end < u32::from(value));
            hash = (hash ^ u64::from(ranges[index].1)).wrapping_mul(1_099_511_628_211);
        }
        assert_eq!(hash, 0x9FB70257D9A6A292);
    }
}
