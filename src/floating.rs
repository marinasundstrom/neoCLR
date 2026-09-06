//! Binary64 is this interpreter's CLI internal floating-point stack category.
use crate::{Fault, Value, metadata::Instruction as Op};

pub(crate) fn binary(op: &Op, left: f64, right: f64) -> Result<Value, Fault> {
    Ok(match op {
        Op::Add => Value::Double(left + right),
        Op::Sub => Value::Double(left - right),
        Op::Mul => Value::Double(left * right),
        Op::Divide => Value::Double(left / right),
        Op::Remainder => Value::Double(left % right),
        Op::Greater => Value::Boolean(left > right),
        Op::GreaterUnsigned => Value::Boolean(left.is_nan() || right.is_nan() || left > right),
        Op::Less => Value::Boolean(left < right),
        Op::LessUnsigned => Value::Boolean(left.is_nan() || right.is_nan() || left < right),
        _ => {
            return Err(Fault::new(
                "instruction does not accept floating-point operands",
            ));
        }
    })
}

pub(crate) fn convert(op: &Op, value: Value) -> Result<Value, Fault> {
    let unsigned = matches!(op, Op::ConvertFloatUnsigned);
    let narrow = matches!(op, Op::ConvertFloat32);
    macro_rules! cast {
        ($n:expr) => {
            if narrow { $n as f32 as f64 } else { $n as f64 }
        };
    }
    // Cast integers directly to the requested precision to avoid double rounding.
    let number = match value {
        Value::Double(n) if !unsigned => cast!(n),
        Value::Int32(n) if unsigned => cast!(n as u32),
        Value::Int32(n) => cast!(n),
        Value::Int64(n) if unsigned => cast!(n as u64),
        Value::Int64(n) => cast!(n),
        Value::IntPtr(n) if unsigned => cast!(n as usize),
        Value::IntPtr(n) => cast!(n),
        Value::UIntPtr(n) if unsigned => cast!(n),
        Value::UIntPtr(n) => cast!(n as isize),
        _ => return Err(Fault::new("unsupported floating-point conversion")),
    };
    Ok(Value::Double(number))
}

pub(crate) fn to_integer(op: &Op, number: f64) -> Result<Value, Fault> {
    // Deliberate policy for CLI-unspecified unchecked overflow/NaN results:
    // truncate toward zero, saturate to destination bounds, and map NaN to zero.
    // Rust's defined saturating float casts implement that policy (no unsafe cast).
    Ok(match op {
        Op::ConvertInt8 => Value::Int32(number as i8 as i32),
        Op::ConvertUInt8 => Value::Int32(number as u8 as i32),
        Op::ConvertInt16 => Value::Int32(number as i16 as i32),
        Op::ConvertUInt16 => Value::Int32(number as u16 as i32),
        Op::ConvertInt32 => Value::Int32(number as i32),
        Op::ConvertUInt32 => Value::Int32(number as u32 as i32),
        Op::ConvertInt64 => Value::Int64(number as i64),
        Op::ConvertUInt64 => Value::Int64(number as u64 as i64),
        Op::ConvertNativeInt => Value::IntPtr(number as isize),
        Op::ConvertNativeUInt => Value::UIntPtr(number as usize),
        _ => {
            return Err(Fault::new(
                "unsupported floating-point to integer conversion",
            ));
        }
    })
}
