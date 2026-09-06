//! Checked numeric conversion preserves integer precision and validates before casting.
use crate::{
    Fault, Value,
    metadata::{Instruction as Op, Type},
};

pub(crate) fn convert(op: &Op, value: Value) -> Result<Value, Fault> {
    let (target, source_unsigned) = match op {
        Op::CheckedInt8 => (Type::SByte, false),
        Op::CheckedUInt8 => (Type::Byte, false),
        Op::CheckedInt16 => (Type::Int16, false),
        Op::CheckedUInt16 => (Type::UInt16, false),
        Op::CheckedInt32 => (Type::Int32, false),
        Op::CheckedUInt32 => (Type::UInt32, false),
        Op::CheckedInt64 => (Type::Int64, false),
        Op::CheckedUInt64 => (Type::UInt64, false),
        Op::CheckedNativeInt => (Type::IntPtr, false),
        Op::CheckedNativeUInt => (Type::UIntPtr, false),
        Op::CheckedInt8Unsigned => (Type::SByte, true),
        Op::CheckedUInt8Unsigned => (Type::Byte, true),
        Op::CheckedInt16Unsigned => (Type::Int16, true),
        Op::CheckedUInt16Unsigned => (Type::UInt16, true),
        Op::CheckedInt32Unsigned => (Type::Int32, true),
        Op::CheckedUInt32Unsigned => (Type::UInt32, true),
        Op::CheckedInt64Unsigned => (Type::Int64, true),
        Op::CheckedUInt64Unsigned => (Type::UInt64, true),
        Op::CheckedNativeIntUnsigned => (Type::IntPtr, true),
        Op::CheckedNativeUIntUnsigned => (Type::UIntPtr, true),
        _ => return Err(Fault::new("invalid checked conversion instruction")),
    };
    let (bits, signed) = match target {
        Type::SByte => (8, true),
        Type::Byte => (8, false),
        Type::Int16 => (16, true),
        Type::UInt16 => (16, false),
        Type::Int32 => (32, true),
        Type::UInt32 => (32, false),
        Type::Int64 => (64, true),
        Type::UInt64 => (64, false),
        Type::IntPtr => (isize::BITS, true),
        Type::UIntPtr => (usize::BITS, false),
        _ => return Err(Fault::new("invalid checked conversion target")),
    };
    let lower = if signed { -(1i128 << (bits - 1)) } else { 0 };
    let upper_exclusive = 1i128 << (bits - u32::from(signed));
    let overflow = || Fault::new(format!("checked conversion overflow to {target:?}"));
    let number = match value {
        Value::Int32(n) if source_unsigned => n as u32 as i128,
        Value::Int32(n) => n as i128,
        Value::Int64(n) if source_unsigned => n as u64 as i128,
        Value::Int64(n) => n as i128,
        Value::IntPtr(n) if source_unsigned => n as usize as i128,
        Value::IntPtr(n) => n as i128,
        Value::UIntPtr(n) if source_unsigned => n as i128,
        Value::UIntPtr(n) => n as isize as i128,
        Value::Double(n) => {
            let truncated = n.trunc();
            // Power-of-two bounds are exactly representable in binary64, unlike
            // i64::MAX/u64::MAX. Test the exclusive upper bound before any cast.
            if !truncated.is_finite()
                || truncated < lower as f64
                || truncated >= upper_exclusive as f64
            {
                return Err(overflow());
            }
            truncated as i128
        }
        _ => return Err(Fault::new("checked conversion requires numeric value")),
    };
    if number < lower || number >= upper_exclusive {
        return Err(overflow());
    }
    Ok(match target {
        Type::SByte | Type::Byte | Type::Int16 | Type::UInt16 | Type::Int32 | Type::UInt32 => {
            Value::Int32(number as i32)
        }
        Type::Int64 | Type::UInt64 => Value::Int64(number as i64),
        Type::IntPtr => Value::IntPtr(number as isize),
        Type::UIntPtr => Value::UIntPtr(number as usize),
        _ => return Err(Fault::new("invalid checked conversion target")),
    })
}
