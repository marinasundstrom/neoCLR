//! Bounded intrinsic Object dispatch for boxed values; never allocation identity.
use crate::{
    Fault, Value,
    metadata::{Function, Type},
    value::ObjectReference,
};

/// Supported numeric types and Boolean have intrinsic value contracts. Other types still
/// require their own validated equality/hash implementation.
pub(crate) fn dispatch(
    object: &ObjectReference,
    contract: &Function,
    arguments: &[Value],
) -> Result<Option<Value>, Fault> {
    if !matches!(
        object.concrete_type(),
        Type::Int32 | Type::Int64 | Type::Boolean | Type::Single | Type::Double
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
    let display = contract.name == "System.Object.ToString"
        && contract.parameters.is_empty()
        && contract.returns == Type::String;
    if !equality && !hashing && !display {
        return Ok(None);
    }
    let value = primitive_value(object)?;
    if display {
        return Ok(value.display().map(Value::String));
    }
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
                value.equals(&primitive_value(other)?)
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
// compared value. Floating Object equality intentionally treats NaNs as equal.
enum PrimitiveValue {
    Int32(i32),
    Int64(i64),
    Boolean(bool),
    Single(f32),
    Double(f64),
}

impl PrimitiveValue {
    fn equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int32(a), Self::Int32(b)) => a == b,
            (Self::Int64(a), Self::Int64(b)) => a == b,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Single(a), Self::Single(b)) => a == b || (a.is_nan() && b.is_nan()),
            (Self::Double(a), Self::Double(b)) => a == b || (a.is_nan() && b.is_nan()),
            _ => false,
        }
    }

    fn display(&self) -> Option<String> {
        Some(match *self {
            Self::Int32(value) => value.to_string(),
            Self::Int64(value) => value.to_string(),
            Self::Boolean(value) => if value { "True" } else { "False" }.to_owned(),
            Self::Single(_) | Self::Double(_) => return None,
        })
    }

    fn hash(&self) -> i32 {
        match *self {
            Self::Int32(value) => value,
            Self::Int64(value) => (value as i32) ^ ((value >> 32) as i32),
            Self::Boolean(value) => i32::from(value),
            Self::Single(value) => {
                let bits = if value == 0.0 {
                    0
                } else if value.is_nan() {
                    0x7f80_0000
                } else {
                    value.to_bits()
                };
                bits as i32
            }
            Self::Double(value) => {
                let bits = if value == 0.0 {
                    0
                } else if value.is_nan() {
                    0x7ff0_0000_0000_0000
                } else {
                    value.to_bits()
                };
                (bits as i32) ^ ((bits >> 32) as i32)
            }
        }
    }
}

fn primitive_value(object: &ObjectReference) -> Result<PrimitiveValue, Fault> {
    match (object.concrete_type(), object.reference.read()?) {
        (Type::Int32, Value::Int32(value)) => Ok(PrimitiveValue::Int32(value)),
        (Type::Int64, Value::Int64(value)) => Ok(PrimitiveValue::Int64(value)),
        (Type::Boolean, Value::Boolean(value)) => Ok(PrimitiveValue::Boolean(value)),
        (Type::Single, Value::Single(value)) => Ok(PrimitiveValue::Single(value)),
        (Type::Double, Value::Double(value)) => Ok(PrimitiveValue::Double(value)),
        _ => Err(Fault::new("boxed primitive has an invalid payload")),
    }
}

#[cfg(test)]
mod tests {
    use super::PrimitiveValue;

    #[test]
    fn floating_object_nan_payloads_and_zero_hashes_are_canonical() {
        for bits in [0x7f80_0001, 0x7fc0_0000, 0xffc0_1234] {
            let value = PrimitiveValue::Single(f32::from_bits(bits));
            assert!(value.equals(&PrimitiveValue::Single(f32::NAN)));
            assert_eq!(value.hash(), 0x7f80_0000);
        }
        for bits in [
            0x7ff0_0000_0000_0001,
            0x7ff8_0000_0000_0000,
            0xfff8_0000_0000_1234,
        ] {
            let value = PrimitiveValue::Double(f64::from_bits(bits));
            assert!(value.equals(&PrimitiveValue::Double(f64::NAN)));
            assert_eq!(value.hash(), 0x7ff0_0000);
        }
        for (positive, negative) in [
            (PrimitiveValue::Single(0.0), PrimitiveValue::Single(-0.0)),
            (PrimitiveValue::Double(0.0), PrimitiveValue::Double(-0.0)),
        ] {
            assert!(positive.equals(&negative));
            assert_eq!(positive.hash(), 0);
            assert_eq!(negative.hash(), 0);
        }
        for (value, expected) in [
            (PrimitiveValue::Single(1.5), 0x3fc0_0000),
            (PrimitiveValue::Double(1.5), 0x3ff8_0000),
            (PrimitiveValue::Single(f32::from_bits(1)), 1),
            (PrimitiveValue::Double(f64::from_bits(1)), 1),
            (
                PrimitiveValue::Single(f32::NEG_INFINITY),
                0xff80_0000u32 as i32,
            ),
            (
                PrimitiveValue::Double(f64::NEG_INFINITY),
                0xfff0_0000u32 as i32,
            ),
        ] {
            assert_eq!(value.hash(), expected);
        }
        assert!(!PrimitiveValue::Single(f32::NAN).equals(&PrimitiveValue::Double(f64::NAN)));
        assert!(PrimitiveValue::Single(1.0).display().is_none());
        assert!(PrimitiveValue::Double(1.0).display().is_none());
    }
}
