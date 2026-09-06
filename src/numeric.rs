//! Integer operations use opcode signedness, independently of signature spelling.
use crate::{Fault, Value, metadata::Instruction as Op};

pub(crate) fn binary(op: &Op, left: Value, right: Value) -> Result<Value, Fault> {
    macro_rules! calculate {
        ($left:expr, $right:expr, $signed:ty, $unsigned:ty, $variant:ident) => {{
            let left = $left as $signed;
            let right = $right as $signed;
            let unsigned_left = left as $unsigned;
            let unsigned_right = right as $unsigned;
            let result = match op {
                Op::Less => return Ok(Value::Boolean(left < right)),
                Op::LessUnsigned => return Ok(Value::Boolean(unsigned_left < unsigned_right)),
                Op::Add => Some(left.wrapping_add(right)),
                Op::Sub => Some(left.wrapping_sub(right)),
                Op::Mul => Some(left.wrapping_mul(right)),
                Op::AddChecked => left.checked_add(right),
                Op::SubChecked => left.checked_sub(right),
                Op::MulChecked => left.checked_mul(right),
                Op::AddCheckedUnsigned => unsigned_left
                    .checked_add(unsigned_right)
                    .map(|n| n as $signed),
                Op::SubCheckedUnsigned => unsigned_left
                    .checked_sub(unsigned_right)
                    .map(|n| n as $signed),
                Op::MulCheckedUnsigned => unsigned_left
                    .checked_mul(unsigned_right)
                    .map(|n| n as $signed),
                Op::Divide => left.checked_div(right),
                Op::DivideUnsigned => unsigned_left
                    .checked_div(unsigned_right)
                    .map(|n| n as $signed),
                _ => return Err(Fault::new("invalid integer operation")),
            };
            result.map(|n| Value::$variant(n as _)).ok_or_else(|| {
                Fault::new(
                    if right == 0 && matches!(op, Op::Divide | Op::DivideUnsigned) {
                        "division by zero"
                    } else {
                        concat!(stringify!($variant), " overflow")
                    },
                )
            })
        }};
    }
    match (left, right) {
        (Value::Int32(a), Value::Int32(b)) => calculate!(a, b, i32, u32, Int32),
        (Value::Int64(a), Value::Int64(b)) => calculate!(a, b, i64, u64, Int64),
        (Value::IntPtr(a), Value::IntPtr(b)) => calculate!(a, b, isize, usize, IntPtr),
        (Value::UIntPtr(a), Value::UIntPtr(b)) => calculate!(a, b, isize, usize, UIntPtr),
        _ => Err(Fault::new(
            "integer operation requires matching integer types",
        )),
    }
}

pub(crate) fn convert(op: &Op, value: Value) -> Result<Value, Fault> {
    // Widening interpretation is selected by the conversion opcode.
    let unsigned = matches!(op, Op::ConvertNativeUInt | Op::ConvertUInt64);
    let bits = match value {
        Value::Int32(n) if unsigned => n as u32 as u64,
        Value::Int32(n) => n as i64 as u64,
        Value::Int64(n) => n as u64,
        Value::IntPtr(n) if unsigned => n as usize as u64,
        Value::IntPtr(n) => n as i64 as u64,
        Value::UIntPtr(n) if unsigned => n as u64,
        Value::UIntPtr(n) => n as isize as i64 as u64,
        Value::Pointer(p) if matches!(op, Op::ConvertNativeInt | Op::ConvertNativeUInt) => {
            p.address as u64
        }
        _ => return Err(Fault::new("unsupported integer conversion")),
    };
    Ok(match op {
        Op::ConvertInt8 => Value::Int32(bits as i8 as i32),
        Op::ConvertUInt8 => Value::Int32(bits as u8 as i32),
        Op::ConvertInt16 => Value::Int32(bits as i16 as i32),
        Op::ConvertUInt16 => Value::Int32(bits as u16 as i32),
        Op::ConvertInt32 | Op::ConvertUInt32 => Value::Int32(bits as i32),
        Op::ConvertInt64 | Op::ConvertUInt64 => Value::Int64(bits as i64),
        Op::ConvertNativeInt => Value::IntPtr(bits as isize),
        Op::ConvertNativeUInt => Value::UIntPtr(bits as usize),
        _ => return Err(Fault::new("invalid integer conversion")),
    })
}

/// Validate an indirect instruction's storage family and select its interpretation.
pub(crate) fn indirect_type(
    op: &Op,
    target: &crate::metadata::Type,
) -> Result<crate::metadata::Type, Fault> {
    use crate::metadata::Type as T;
    let (valid, read_type) = match op {
        Op::LoadIndirectInt8 | Op::LoadIndirectUInt8 | Op::StoreIndirectInt8 => (
            matches!(target, T::Byte | T::SByte),
            if matches!(op, Op::LoadIndirectUInt8) {
                T::Byte
            } else {
                T::SByte
            },
        ),
        Op::LoadIndirectInt16 | Op::LoadIndirectUInt16 | Op::StoreIndirectInt16 => (
            matches!(target, T::Int16 | T::UInt16 | T::Char),
            if matches!(op, Op::LoadIndirectUInt16) {
                T::UInt16
            } else {
                T::Int16
            },
        ),
        Op::LoadIndirectInt32 | Op::LoadIndirectUInt32 | Op::StoreIndirectInt32 => {
            (matches!(target, T::Int32 | T::UInt32), T::Int32)
        }
        Op::LoadIndirectInt64 | Op::StoreIndirectInt64 => {
            (matches!(target, T::Int64 | T::UInt64), T::Int64)
        }
        Op::LoadIndirectNative | Op::StoreIndirectNative => {
            (matches!(target, T::IntPtr | T::UIntPtr), target.clone())
        }
        _ => return Err(Fault::new("invalid indirect instruction")),
    };
    if valid {
        Ok(read_type)
    } else {
        Err(Fault::new("memory pointer type mismatch"))
    }
}
