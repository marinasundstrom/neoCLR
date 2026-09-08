//! Explicit runtime binding registry. A name alone never activates host dispatch.
use crate::{
    Fault, Value,
    metadata::{Function, Type},
};

pub(crate) enum Binding {
    LocalClock,
    Math(crate::math::Operation),
    Reflection(crate::reflection::Query),
    TypeName,
    TypeEquals,
    TypeArgumentCount,
    TypeArgument,
    ParseInt32,
    Int32ToString,
    WriteLine,
    CharCategory,
    StringConcat,
    StringByteCount,
    StringCompareOrdinal,
    StringContainsOrdinal,
    StringStartsWithOrdinal,
    StringEndsWithOrdinal,
    StringSliceUtf8,
    ErrorFromMessage,
    ErrorMessage,
    ReadAllText,
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
    if let Some((query, integer, returns)) = crate::reflection::Query::binding(&function.name) {
        let expected = if integer {
            vec![Type::RuntimeTypeHandle, Type::Int32]
        } else {
            vec![Type::RuntimeTypeHandle]
        };
        if function.parameters != expected || function.returns != returns {
            return Err(Fault::new("reflection binding signature mismatch"));
        }
        return Ok(Binding::Reflection(query));
    }
    let (binding, returns) = match (function.name.as_str(), function.parameters.as_slice()) {
        ("neoCLR.Runtime.LocalClock", []) => {
            (Binding::LocalClock, Type::Array(Box::new(Type::Int32)))
        }
        ("neoCLR.Runtime.ParseInt32", [Type::String]) => (Binding::ParseInt32, Type::Value),
        ("neoCLR.Runtime.Int32ToString", [Type::Int32]) => (Binding::Int32ToString, Type::String),
        ("neoCLR.Runtime.WriteLine", [Type::String]) => (Binding::WriteLine, Type::Void),
        ("neoCLR.Runtime.CharCategory", [Type::Char]) => (Binding::CharCategory, Type::Int32),
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
        ("neoCLR.Runtime.StringByteCount", [Type::String]) => {
            (Binding::StringByteCount, Type::Int32)
        }
        ("neoCLR.Runtime.StringSliceUtf8", [Type::String, Type::Int32, Type::Int32]) => {
            (Binding::StringSliceUtf8, Type::Value)
        }
        ("neoCLR.Runtime.ErrorFromMessage", [Type::String]) => {
            (Binding::ErrorFromMessage, Type::Error)
        }
        ("neoCLR.Runtime.ErrorMessage", [Type::Error]) => (Binding::ErrorMessage, Type::String),
        ("neoCLR.Runtime.ReadAllText", [Type::String, Type::Int32]) => {
            (Binding::ReadAllText, Type::Value)
        }
        ("neoCLR.Runtime.ConsoleReadByte", []) => (Binding::ConsoleReadByte, Type::Value),
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
        module: &crate::Module,
        limits: &crate::Limits,
        output: &mut Vec<String>,
        console: Option<&dyn crate::Console>,
    ) -> Result<Value, Fault> {
        if let Self::Math(operation) = self {
            return operation.invoke(&args);
        }
        if let Self::Reflection(query) = self {
            return query.invoke(module, &args, limits);
        }
        match (self, args.as_slice()) {
            (Self::LocalClock, []) => crate::clock::read_local(),
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
            (Self::CharCategory, [Value::Char(value)]) => {
                let ranges = crate::char_categories::RANGES;
                let index = ranges.partition_point(|(end, _)| end < value);
                Ok(Value::Int32(i32::from(ranges[index].1)))
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
                // .NET ordinal ordering compares UTF-16 units, not UTF-8 bytes or scalars.
                let order = left.encode_utf16().cmp(right.encode_utf16());
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
            (Self::ErrorFromMessage, [Value::String(message)])
            | (Self::ErrorMessage, [Value::Error(message)]) => {
                let mut text = String::new();
                text.try_reserve_exact(message.len())
                    .map_err(|_| Fault::new("error text allocation failed"))?;
                text.push_str(message);
                Ok(if matches!(self, Self::ErrorFromMessage) {
                    Value::Error(text)
                } else {
                    Value::String(text)
                })
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
        assert_eq!(ranges.last().unwrap().0, u16::MAX);
        assert!(ranges.windows(2).all(|pair| pair[0].0 < pair[1].0));
        let mut hash = 14_695_981_039_346_656_037u64;
        for value in 0..=u16::MAX {
            let index = ranges.partition_point(|(end, _)| *end < value);
            hash = (hash ^ u64::from(ranges[index].1)).wrapping_mul(1_099_511_628_211);
        }
        assert_eq!(hash, 0x9FB70257D9A6A292);
    }
}
