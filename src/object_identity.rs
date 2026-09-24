//! Identity of live heap objects, separate from value equality and byref locations.
use crate::{Fault, FaultCode, Value, metadata::Type, value::ObjectReference};

enum Identity<'a> {
    Object(&'a ObjectReference),
    Text(crate::StringValue),
}
fn reference(value: &Value) -> Result<Option<Identity<'_>>, Fault> {
    match value {
        Value::NullObjectReference(_) => Ok(None),
        Value::ObjectReference(object) => {
            object.reference.assigned()?;
            if object.concrete_type() == Type::String {
                let Value::String(text) = object.reference.read()? else {
                    return Err(Fault::new("String wrapper has invalid storage"));
                };
                return Ok(Some(Identity::Text(text)));
            }
            Ok(Some(Identity::Object(object)))
        }
        Value::String(text) => Ok(Some(Identity::Text(text.clone()))),
        _ => Err(Fault::new("object identity requires an object reference")),
    }
}

pub(crate) fn reference_equals(left: &Value, right: &Value) -> Result<bool, Fault> {
    Ok(match (reference(left)?, reference(right)?) {
        (None, None) => true,
        (Some(Identity::Object(left)), Some(Identity::Object(right))) => {
            left.reference.same_location(&right.reference)
        }
        (Some(Identity::Text(left)), Some(Identity::Text(right))) => left.same_owner(&right),
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
    reference_equals(left, right)
}

pub(crate) fn hash(value: &Value) -> Result<i32, Fault> {
    let object = reference(value)?.ok_or_else(|| {
        Fault::coded(
            FaultCode::NullReference,
            "GetHashCode requires a non-null instance",
        )
    })?;
    Ok(match object {
        Identity::Object(object) => mix_hash(object.allocation_id() as u64),
        Identity::Text(text) => text.identity_hash(),
    })
}

// Hashes may collide and are neither persisted IDs nor native addresses.
pub(crate) fn mix_hash(seed: u64) -> i32 {
    let mut bits = seed.wrapping_add(0x9e3779b97f4a7c15);
    bits = (bits ^ (bits >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    bits = (bits ^ (bits >> 27)).wrapping_mul(0x94d049bb133111eb);
    bits ^= bits >> 31;
    (bits ^ (bits >> 32)) as i32
}
