//! Bounded intrinsic Object dispatch for boxed values; never allocation identity.
use crate::{
    Fault, Value,
    metadata::{Function, Type},
    value::ObjectReference,
};

/// Int32, Int64 and Boolean have intrinsic value contracts. Other intrinsic types still
/// require their own validated equality/hash implementation.
pub(crate) fn dispatch(
    object: &ObjectReference,
    contract: &Function,
    arguments: &[Value],
) -> Result<Option<Value>, Fault> {
    if !matches!(
        object.concrete_type(),
        Type::Int32 | Type::Int64 | Type::Boolean
    ) || contract.owner.as_ref() != Some(&Type::from_name("System.Object"))
        || contract
            .definition
            .as_ref()
            .is_none_or(|id| id.module != "System")
        || !contract.instance
        || !contract.is_virtual
        || contract.is_abstract
        || contract.receiver_byref
        || contract.no_result
        || !contract.generic_parameters.is_empty()
        || !contract.generic_arguments.is_empty()
    {
        return Ok(None);
    }
    let equality = contract.name == "System.Object.Equals"
        && contract.parameters == [Type::from_name("System.Object")]
        && contract.returns == Type::Boolean;
    let hashing = contract.name == "System.Object.GetHashCode"
        && contract.parameters.is_empty()
        && contract.returns == Type::Int32;
    if !equality && !hashing {
        return Ok(None);
    }
    let value = primitive_value(object)?;
    if hashing {
        return Ok(Some(Value::Int32(value.hash())));
    }
    let [other] = arguments else {
        return Err(Fault::new(
            "boxed primitive equality requires one Object argument",
        ));
    };
    let equal = match other {
        Value::NullObjectReference(_) => false,
        Value::ObjectReference(other) => {
            other.reference.assigned()?;
            if other.concrete_type() == object.concrete_type() {
                value == primitive_value(other)?
            } else {
                false
            }
        }
        _ => {
            return Err(Fault::new(
                "boxed primitive equality requires an Object argument",
            ));
        }
    };
    Ok(Some(Value::Boolean(equal)))
}

// Keep full payloads for equality; a hash can collide and must never be the
// compared value. Concrete type checks also distinguish Boolean/Int32/Int64.
#[derive(PartialEq, Eq)]
enum PrimitiveValue {
    Int32(i32),
    Int64(i64),
    Boolean(bool),
}

impl PrimitiveValue {
    fn hash(&self) -> i32 {
        match *self {
            Self::Int32(value) => value,
            Self::Int64(value) => (value as i32) ^ ((value >> 32) as i32),
            Self::Boolean(value) => i32::from(value),
        }
    }
}

fn primitive_value(object: &ObjectReference) -> Result<PrimitiveValue, Fault> {
    match (object.concrete_type(), object.reference.read()?) {
        (Type::Int32, Value::Int32(value)) => Ok(PrimitiveValue::Int32(value)),
        (Type::Int64, Value::Int64(value)) => Ok(PrimitiveValue::Int64(value)),
        (Type::Boolean, Value::Boolean(value)) => Ok(PrimitiveValue::Boolean(value)),
        _ => Err(Fault::new("boxed primitive has an invalid payload")),
    }
}
