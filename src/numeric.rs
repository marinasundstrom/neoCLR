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
                Op::Greater => return Ok(Value::Boolean(left > right)),
                Op::GreaterUnsigned => return Ok(Value::Boolean(unsigned_left > unsigned_right)),
                Op::Less => return Ok(Value::Boolean(left < right)),
                Op::LessUnsigned => return Ok(Value::Boolean(unsigned_left < unsigned_right)),
                Op::BitAnd => Some(left & right),
                Op::BitOr => Some(left | right),
                Op::BitXor => Some(left ^ right),
                Op::Remainder => left.checked_rem(right),
                Op::RemainderUnsigned => unsigned_left
                    .checked_rem(unsigned_right)
                    .map(|n| n as $signed),
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
                    if right == 0
                        && matches!(
                            op,
                            Op::Divide | Op::DivideUnsigned | Op::Remainder | Op::RemainderUnsigned
                        )
                    {
                        "division by zero"
                    } else {
                        concat!(stringify!($variant), " overflow")
                    },
                )
            })
        }};
    }
    if let (Value::Double(left), Value::Double(right)) = (&left, &right) {
        return crate::floating::binary(op, *left, *right);
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
    if let Value::Double(value) = value {
        return crate::floating::to_integer(op, value);
    }
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
        Op::LoadIndirectFloat32 | Op::StoreIndirectFloat32 => {
            (matches!(target, T::Single), T::Single)
        }
        Op::LoadIndirectFloat64 | Op::StoreIndirectFloat64 => {
            (matches!(target, T::Double), T::Double)
        }
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

pub(crate) fn unary(op: &Op, value: Value) -> Result<Value, Fault> {
    macro_rules! apply {
        ($n:expr, $variant:ident) => {
            Ok(Value::$variant(match op {
                Op::BitNot => !$n,
                Op::Negate => $n.wrapping_neg(),
                _ => return Err(Fault::new("invalid unary integer operation")),
            }))
        };
    }
    match value {
        Value::Double(n) if matches!(op, Op::Negate) => Ok(Value::Double(-n)),
        Value::Int32(n) => apply!(n, Int32),
        Value::Int64(n) => apply!(n, Int64),
        Value::IntPtr(n) => apply!(n, IntPtr),
        Value::UIntPtr(n) => apply!(n, UIntPtr),
        _ => Err(Fault::new("unary operation requires integer")),
    }
}

pub(crate) fn shift(op: &Op, value: Value, count: Value) -> Result<Value, Fault> {
    let count = match count {
        Value::Int32(n) => n as u32,
        Value::IntPtr(n) => n as u32,
        Value::UIntPtr(n) => n as u32,
        _ => return Err(Fault::new("shift count requires Int32 or native integer")),
    };
    macro_rules! apply {
        ($n:expr, $signed:ty, $unsigned:ty, $variant:ident) => {{
            let n = $n as $signed;
            // Explicit prototype rule for CLI-unspecified out-of-range counts.
            let count = count & (<$signed>::BITS - 1);
            let bits = match op {
                Op::ShiftLeft => n.wrapping_shl(count),
                Op::ShiftRight => n.wrapping_shr(count),
                Op::ShiftRightUnsigned => ((n as $unsigned) >> count) as $signed,
                _ => return Err(Fault::new("invalid shift operation")),
            };
            Ok(Value::$variant(bits as _))
        }};
    }
    match value {
        Value::Int32(n) => apply!(n, i32, u32, Int32),
        Value::Int64(n) => apply!(n, i64, u64, Int64),
        Value::IntPtr(n) => apply!(n, isize, usize, IntPtr),
        Value::UIntPtr(n) => apply!(n, isize, usize, UIntPtr),
        _ => Err(Fault::new("shift requires integer value")),
    }
}

pub(crate) fn branch_condition(op: &Op, left: Value, right: Value) -> Result<bool, Fault> {
    if matches!(op, Op::BranchEqual(_) | Op::BranchNotEqual(_)) {
        if left.ty() != right.ty() {
            return Err(Fault::new("equality branch requires matching types"));
        }
        return Ok((left == right) == matches!(op, Op::BranchEqual(_)));
    }
    let floating = matches!((&left, &right), (Value::Double(_), Value::Double(_)));
    // Ordered >= and <= must reject NaN, while their .un forms accept it.
    // Invert the opposite comparison with the appropriate unordered behavior.
    let (comparison, invert) = match op {
        Op::BranchGreater(_) => (Op::Greater, false),
        Op::BranchGreaterUnsigned(_) => (Op::GreaterUnsigned, false),
        Op::BranchLess(_) => (Op::Less, false),
        Op::BranchLessUnsigned(_) => (Op::LessUnsigned, false),
        Op::BranchGreaterEqual(_) if floating => (Op::LessUnsigned, true),
        Op::BranchGreaterEqual(_) => (Op::Less, true),
        Op::BranchGreaterEqualUnsigned(_) if floating => (Op::Less, true),
        Op::BranchGreaterEqualUnsigned(_) => (Op::LessUnsigned, true),
        Op::BranchLessEqual(_) if floating => (Op::GreaterUnsigned, true),
        Op::BranchLessEqual(_) => (Op::Greater, true),
        Op::BranchLessEqualUnsigned(_) if floating => (Op::Greater, true),
        Op::BranchLessEqualUnsigned(_) => (Op::GreaterUnsigned, true),
        _ => return Err(Fault::new("invalid comparison branch")),
    };
    let Value::Boolean(value) = binary(&comparison, left, right)? else {
        return Err(Fault::new("comparison did not produce Boolean"));
    };
    Ok(value != invert)
}
