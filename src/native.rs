//! Explicit runtime binding registry. A name alone never activates host dispatch.
use crate::{
    Fault, Value,
    metadata::{Case, Function, Type},
};

pub(crate) enum Binding {
    ParseInt32,
    Int32ToString,
    WriteLine,
    StringConcat,
    StringByteCount,
    StringSliceUtf8,
    ErrorFromMessage,
    ErrorMessage,
    ReadAllText,
}

pub(crate) fn bind(function: &Function) -> Result<Binding, Fault> {
    if !function.is_internal_call() || function.instance || function.owner.is_some() {
        return Err(Fault::new("native binding requires InternalCall metadata"));
    }
    let (binding, returns) = match (function.name.as_str(), function.parameters.as_slice()) {
        ("neoCLR.Runtime.ParseInt32", [Type::String]) => (
            Binding::ParseInt32,
            Type::Result(Box::new(Type::Int32), Box::new(Type::Error)),
        ),
        ("neoCLR.Runtime.Int32ToString", [Type::Int32]) => (Binding::Int32ToString, Type::String),
        ("neoCLR.Runtime.WriteLine", [Type::String]) => (Binding::WriteLine, Type::Void),
        ("neoCLR.Runtime.StringConcat", [Type::String, Type::String]) => {
            (Binding::StringConcat, Type::String)
        }
        ("neoCLR.Runtime.StringByteCount", [Type::String]) => {
            (Binding::StringByteCount, Type::Int32)
        }
        ("neoCLR.Runtime.StringSliceUtf8", [Type::String, Type::Int32, Type::Int32]) => (
            Binding::StringSliceUtf8,
            Type::Result(Box::new(Type::String), Box::new(Type::Error)),
        ),
        ("neoCLR.Runtime.ErrorFromMessage", [Type::String]) => {
            (Binding::ErrorFromMessage, Type::Error)
        }
        ("neoCLR.Runtime.ErrorMessage", [Type::Error]) => (Binding::ErrorMessage, Type::String),
        ("neoCLR.Runtime.ReadAllText", [Type::String, Type::Int32]) => (
            Binding::ReadAllText,
            Type::Result(Box::new(Type::String), Box::new(Type::Error)),
        ),
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
        output: &mut Vec<String>,
    ) -> Result<Value, Fault> {
        match (self, args.as_slice()) {
            (Self::ReadAllText, [Value::String(path), Value::Int32(max_bytes)]) => {
                crate::file_io::read_all_text(path, *max_bytes)
            }
            (Self::ParseInt32, [Value::String(text)]) => Ok(match text.parse::<i32>() {
                Ok(n) => Value::result(Value::Int32(n), Type::Int32, Type::Error, Case::Ok),
                Err(_) => Value::result(
                    Value::Error("InvalidInt32".into()),
                    Type::Int32,
                    Type::Error,
                    Case::Err,
                ),
            }),
            (Self::Int32ToString, [Value::Int32(number)]) => Ok(Value::String(number.to_string())),
            (Self::WriteLine, [Value::String(text)]) => {
                output.push(text.clone());
                Ok(Value::Void)
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
                let error = |message: &str| {
                    Value::result(
                        Value::Error(message.into()),
                        Type::String,
                        Type::Error,
                        Case::Err,
                    )
                };
                let (Ok(start), Ok(length)) = (usize::try_from(*start), usize::try_from(*length))
                else {
                    return Ok(error("ArgumentOutOfRange"));
                };
                let Some(end) = start.checked_add(length).filter(|end| *end <= value.len()) else {
                    return Ok(error("ArgumentOutOfRange"));
                };
                let Some(slice) = value.get(start..end) else {
                    return Ok(error("InvalidUtf8Boundary"));
                };
                let mut result = String::new();
                result
                    .try_reserve_exact(slice.len())
                    .map_err(|_| Fault::new("string allocation failed"))?;
                result.push_str(slice);
                Ok(Value::result(
                    Value::String(result),
                    Type::String,
                    Type::Error,
                    Case::Ok,
                ))
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
