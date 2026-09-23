//! Bounded intrinsic Object dispatch for boxed values; never allocation identity.
use crate::{
    Fault, Value,
    metadata::{Function, Type},
    value::ObjectReference,
};

/// Int32 is the first intrinsic value contract. Named structs and other intrinsic
/// types still require their own validated equality/hash implementation.
pub(crate) fn dispatch(
    object: &ObjectReference,
    contract: &Function,
    arguments: &[Value],
) -> Result<Option<Value>, Fault> {
    if object.concrete_type() != Type::Int32
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
    let Value::Int32(value) = object.reference.read()? else {
        return Err(Fault::new("boxed Int32 has an invalid payload"));
    };
    if hashing {
        return Ok(Some(Value::Int32(value)));
    }
    let [other] = arguments else {
        return Err(Fault::new(
            "boxed Int32 equality requires one Object argument",
        ));
    };
    let equal = match other {
        Value::NullObjectReference(_) => false,
        Value::ObjectReference(other) => {
            other.reference.assigned()?;
            if other.concrete_type() == Type::Int32 {
                let Value::Int32(other) = other.reference.read()? else {
                    return Err(Fault::new("boxed Int32 has an invalid payload"));
                };
                value == other
            } else {
                false
            }
        }
        _ => {
            return Err(Fault::new(
                "boxed Int32 equality requires an Object argument",
            ));
        }
    };
    Ok(Some(Value::Boolean(equal)))
}
