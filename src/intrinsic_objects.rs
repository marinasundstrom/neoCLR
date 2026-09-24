//! Bounded Object value dispatch for intrinsic strings and boxed values.
use crate::{
    Fault, Value,
    metadata::{Function, Type},
    value::ObjectReference,
};

/// Strings and supported boxed primitives have intrinsic content/value contracts.
/// Other types still require their own validated equality/hash implementation.
pub(crate) fn dispatch(
    object: &ObjectReference,
    contract: &Function,
    arguments: &[Value],
) -> Result<Option<Value>, Fault> {
    if !matches!(
        object.concrete_type(),
        Type::Int32
            | Type::Int64
            | Type::Boolean
            | Type::Single
            | Type::Double
            | Type::Char
            | Type::String
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
    let value = intrinsic_value(object)?;
    if display {
        return Ok(value.display().map(Value::String));
    }
    if hashing {
        return Ok(Some(Value::Int32(value.hash())));
    }
    let [other] = arguments else {
        return Err(Fault::new(
            "intrinsic Object equality requires one Object argument",
        ));
    };
    let equal = match other {
        Value::NullObjectReference(_) => false,
        Value::ObjectReference(other) => {
            other.reference.assigned()?;
            if other.concrete_type() == object.concrete_type() {
                value.equals(&intrinsic_value(other)?)
            } else {
                false
            }
        }
        _ => {
            return Err(Fault::new(
                "intrinsic Object equality requires an Object argument",
            ));
        }
    };
    Ok(Some(Value::Boolean(equal)))
}

// Keep full payloads for equality; a hash can collide and must never be the
// compared value. Floating Object equality intentionally treats NaNs as equal.
enum IntrinsicValue {
    Int32(i32),
    Int64(i64),
    Boolean(bool),
    Single(f32),
    Double(f64),
    Char(String),
    String(crate::StringValue),
}

impl IntrinsicValue {
    fn equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int32(a), Self::Int32(b)) => a == b,
            (Self::Int64(a), Self::Int64(b)) => a == b,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Char(a), Self::Char(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Single(a), Self::Single(b)) => a == b || (a.is_nan() && b.is_nan()),
            (Self::Double(a), Self::Double(b)) => a == b || (a.is_nan() && b.is_nan()),
            _ => false,
        }
    }

    fn display(&self) -> Option<crate::StringValue> {
        Some(match *self {
            Self::Int32(value) => value.to_string().into(),
            Self::Int64(value) => value.to_string().into(),
            Self::Boolean(value) => if value { "True" } else { "False" }.into(),
            Self::Char(ref value) => value.clone().into(),
            Self::String(ref value) => value.clone(),
            Self::Single(_) | Self::Double(_) => return None,
        })
    }

    fn hash(&self) -> i32 {
        match *self {
            Self::Int32(value) => value,
            Self::Int64(value) => (value as i32) ^ ((value >> 32) as i32),
            Self::Boolean(value) => i32::from(value),
            // Same UTF-8 FNV-1a component hash used by HashCode.Add(string).
            Self::Char(ref value) => value.as_bytes().iter().fold(2166136261u32, |hash, byte| {
                (hash ^ u32::from(*byte)).wrapping_mul(16777619)
            }) as i32,
            Self::String(ref value) => value.as_bytes().iter().fold(2166136261u32, |hash, byte| {
                (hash ^ u32::from(*byte)).wrapping_mul(16777619)
            }) as i32,
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

fn intrinsic_value(object: &ObjectReference) -> Result<IntrinsicValue, Fault> {
    match (object.concrete_type(), object.reference.read()?) {
        (Type::Int32, Value::Int32(value)) => Ok(IntrinsicValue::Int32(value)),
        (Type::Int64, Value::Int64(value)) => Ok(IntrinsicValue::Int64(value)),
        (Type::Boolean, Value::Boolean(value)) => Ok(IntrinsicValue::Boolean(value)),
        (Type::Single, Value::Single(value)) => Ok(IntrinsicValue::Single(value)),
        (Type::Double, Value::Double(value)) => Ok(IntrinsicValue::Double(value)),
        (Type::Char, Value::Char(value)) => Ok(IntrinsicValue::Char(value)),
        (Type::String, Value::String(value)) => Ok(IntrinsicValue::String(value)),
        _ => Err(Fault::new("intrinsic Object has an invalid payload")),
    }
}

#[cfg(test)]
mod tests {
    use super::IntrinsicValue;

    #[test]
    fn string_object_display_retains_text_owner() {
        let text = crate::StringValue::from("display 👩‍💻");
        let value = IntrinsicValue::String(text.clone());
        let display = value.display().unwrap();
        assert_eq!(display, text);
        assert_eq!(display.as_ptr(), text.as_ptr());
    }

    #[test]
    fn floating_object_nan_payloads_and_zero_hashes_are_canonical() {
        for bits in [0x7f80_0001, 0x7fc0_0000, 0xffc0_1234] {
            let value = IntrinsicValue::Single(f32::from_bits(bits));
            assert!(value.equals(&IntrinsicValue::Single(f32::NAN)));
            assert_eq!(value.hash(), 0x7f80_0000);
        }
        for bits in [
            0x7ff0_0000_0000_0001,
            0x7ff8_0000_0000_0000,
            0xfff8_0000_0000_1234,
        ] {
            let value = IntrinsicValue::Double(f64::from_bits(bits));
            assert!(value.equals(&IntrinsicValue::Double(f64::NAN)));
            assert_eq!(value.hash(), 0x7ff0_0000);
        }
        for (positive, negative) in [
            (IntrinsicValue::Single(0.0), IntrinsicValue::Single(-0.0)),
            (IntrinsicValue::Double(0.0), IntrinsicValue::Double(-0.0)),
        ] {
            assert!(positive.equals(&negative));
            assert_eq!(positive.hash(), 0);
            assert_eq!(negative.hash(), 0);
        }
        for (value, expected) in [
            (IntrinsicValue::Single(1.5), 0x3fc0_0000),
            (IntrinsicValue::Double(1.5), 0x3ff8_0000),
            (IntrinsicValue::Single(f32::from_bits(1)), 1),
            (IntrinsicValue::Double(f64::from_bits(1)), 1),
            (
                IntrinsicValue::Single(f32::NEG_INFINITY),
                0xff80_0000u32 as i32,
            ),
            (
                IntrinsicValue::Double(f64::NEG_INFINITY),
                0xfff0_0000u32 as i32,
            ),
        ] {
            assert_eq!(value.hash(), expected);
        }
        assert!(!IntrinsicValue::Single(f32::NAN).equals(&IntrinsicValue::Double(f64::NAN)));
        assert!(IntrinsicValue::Single(1.0).display().is_none());
        assert!(IntrinsicValue::Double(1.0).display().is_none());
    }
}
