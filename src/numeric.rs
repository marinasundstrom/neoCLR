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
        (Value::IntPtr(a), Value::IntPtr(b)) => calculate!(a, b, isize, usize, IntPtr),
        (Value::UIntPtr(a), Value::UIntPtr(b)) => calculate!(a, b, isize, usize, UIntPtr),
        _ => Err(Fault::new(
            "integer operation requires matching integer types",
        )),
    }
}

pub(crate) fn convert(op: &Op, value: Value) -> Result<Value, Fault> {
    // conv.i sign-extends Int32; conv.u zero-extends its 32-bit representation.
    // Native values and pointers preserve all native bits. conv.i4 truncates.
    let bits = match value {
        Value::Int32(n) if matches!(op, Op::ConvertNativeUInt) => n as u32 as usize,
        Value::Int32(n) => n as isize as usize,
        Value::IntPtr(n) => n as usize,
        Value::UIntPtr(n) => n,
        Value::Pointer(p) if !matches!(op, Op::ConvertInt32) => p.address,
        _ => return Err(Fault::new("unsupported integer conversion")),
    };
    Ok(match op {
        Op::ConvertNativeInt => Value::IntPtr(bits as isize),
        Op::ConvertNativeUInt => Value::UIntPtr(bits),
        Op::ConvertInt32 => Value::Int32(bits as i32),
        _ => return Err(Fault::new("invalid integer conversion")),
    })
}
