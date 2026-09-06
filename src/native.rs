//! Explicit runtime binding registry. A name alone never activates host dispatch.
use crate::{
    Fault, Value,
    metadata::{Case, Function, Type},
};

pub(crate) enum Binding {
    ParseInt32,
    Int32ToString,
    WriteLine,
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
            _ => Err(Fault::new("invalid native arguments")),
        }
    }
}
