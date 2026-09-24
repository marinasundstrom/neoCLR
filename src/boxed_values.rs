//! Bounded intrinsic Object dispatch for boxed values; never allocation identity.
use crate::{
    Fault, Value,
    metadata::{Function, Type},
    value::ObjectReference,
};

/// Int32 and Boolean have intrinsic value contracts. Other intrinsic types still
/// require their own validated equality/hash implementation.
pub(crate) fn dispatch(
    object: &ObjectReference,
    contract: &Function,
    arguments: &[Value],
) -> Result<Option<Value>, Fault> {
    if !matches!(object.concrete_type(), Type::Int32 | Type::Boolean)
        || contract.owner.as_ref() != Some(&Type::from_name("System.Object"))
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
        return Ok(Some(Value::Int32(value)));
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

// Only called after selecting a supported concrete type. Type identity is checked
// separately: Boolean true and Int32 one must never compare equal.
fn primitive_value(object: &ObjectReference) -> Result<i32, Fault> {
    match (object.concrete_type(), object.reference.read()?) {
        (Type::Int32, Value::Int32(value)) => Ok(value),
        (Type::Boolean, Value::Boolean(value)) => Ok(i32::from(value)),
        _ => Err(Fault::new("boxed primitive has an invalid payload")),
    }
}
