//! Identity of live heap objects, separate from value equality and byref locations.
use crate::{Fault, FaultCode, Value, metadata::Type, value::ObjectReference};

fn reference(value: &Value) -> Result<Option<&ObjectReference>, Fault> {
    match value {
        Value::NullObjectReference(_) => Ok(None),
        Value::ObjectReference(object) => {
            object.reference.assigned()?;
            if object.concrete_type() == Type::String {
                return Err(Fault::new("String object identity is not supported"));
            }
            Ok(Some(object))
        }
        Value::String(_) => Err(Fault::new("String object identity is not supported")),
        _ => Err(Fault::new("object identity requires an object reference")),
    }
}

pub(crate) fn reference_equals(left: &Value, right: &Value) -> Result<bool, Fault> {
    // Validate both operands even when the other is null. In particular, do not
    // accidentally promise stable String identity for an existing wrapper.
    Ok(match (reference(left)?, reference(right)?) {
        (None, None) => true,
        (Some(left), Some(right)) => left.reference.same_location(&right.reference),
        _ => false,
    })
}

pub(crate) fn equals(left: &Value, right: &Value) -> Result<bool, Fault> {
    reference(left)?.ok_or_else(|| {
        Fault::coded(
            FaultCode::NullReference,
            "Equals requires a non-null instance",
        )
    })?;
    // Default class/array equality cannot match intrinsic String contents. Do not
    // route this type mismatch through String's unsupported identity operation.
    if let Value::ObjectReference(other) = right {
        other.reference.assigned()?;
        if other.concrete_type() == Type::String {
            return Ok(false);
        }
    }
    reference_equals(left, right)
}

pub(crate) fn hash(value: &Value) -> Result<i32, Fault> {
    let object = reference(value)?.ok_or_else(|| {
        Fault::coded(
            FaultCode::NullReference,
            "GetHashCode requires a non-null instance",
        )
    })?;
    // Mix the stable execution-local allocation ID, never a native address or
    // mutable payload. This is not a unique ID, persisted format or security hash.
    let mut bits = (object.allocation_id() as u64).wrapping_add(0x9e3779b97f4a7c15);
    bits = (bits ^ (bits >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    bits = (bits ^ (bits >> 27)).wrapping_mul(0x94d049bb133111eb);
    bits ^= bits >> 31;
    Ok((bits ^ (bits >> 32)) as i32)
}
