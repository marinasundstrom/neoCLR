//! Bootstrap floating operations behind signature-validated library calls.
use crate::{Fault, Value};

pub(crate) enum Operation {
    Abs,
    Sqrt,
    Floor,
    Ceiling,
    Truncate,
    Round,
    Exp,
    Log,
    Log10,
    Sin,
    Cos,
    Tan,
    Pow,
    Min,
    Max,
}
impl Operation {
    pub(crate) fn binding(name: &str) -> Option<(Self, usize)> {
        Some(match name {
            "neoCLR.Runtime.MathAbs" => (Self::Abs, 1),
            "neoCLR.Runtime.MathSqrt" => (Self::Sqrt, 1),
            "neoCLR.Runtime.MathFloor" => (Self::Floor, 1),
            "neoCLR.Runtime.MathCeiling" => (Self::Ceiling, 1),
            "neoCLR.Runtime.MathTruncate" => (Self::Truncate, 1),
            "neoCLR.Runtime.MathRound" => (Self::Round, 1),
            "neoCLR.Runtime.MathExp" => (Self::Exp, 1),
            "neoCLR.Runtime.MathLog" => (Self::Log, 1),
            "neoCLR.Runtime.MathLog10" => (Self::Log10, 1),
            "neoCLR.Runtime.MathSin" => (Self::Sin, 1),
            "neoCLR.Runtime.MathCos" => (Self::Cos, 1),
            "neoCLR.Runtime.MathTan" => (Self::Tan, 1),
            "neoCLR.Runtime.MathPow" => (Self::Pow, 2),
            "neoCLR.Runtime.MathMin" => (Self::Min, 2),
            "neoCLR.Runtime.MathMax" => (Self::Max, 2),
            _ => return None,
        })
    }
    pub(crate) fn invoke(&self, args: &[Value]) -> Result<Value, Fault> {
        let value = match (self, args) {
            (Self::Abs, [Value::Double(x)]) => x.abs(),
            (Self::Sqrt, [Value::Double(x)]) => x.sqrt(),
            (Self::Floor, [Value::Double(x)]) => x.floor(),
            (Self::Ceiling, [Value::Double(x)]) => x.ceil(),
            (Self::Truncate, [Value::Double(x)]) => x.trunc(),
            (Self::Round, [Value::Double(x)]) => x.round_ties_even(),
            (Self::Exp, [Value::Double(x)]) => x.exp(),
            (Self::Log, [Value::Double(x)]) => x.ln(),
            (Self::Log10, [Value::Double(x)]) => x.log10(),
            (Self::Sin, [Value::Double(x)]) => x.sin(),
            (Self::Cos, [Value::Double(x)]) => x.cos(),
            (Self::Tan, [Value::Double(x)]) => x.tan(),
            (Self::Pow, [Value::Double(x), Value::Double(y)]) => x.powf(*y),
            (Self::Min | Self::Max, [Value::Double(x), Value::Double(y)]) => {
                // Preserve .NET NaN propagation and ordering of opposite signed zeros.
                if x.is_nan() {
                    *x
                } else if y.is_nan() {
                    *y
                } else if x == y {
                    if x.is_sign_negative() == matches!(self, Self::Min) {
                        *x
                    } else {
                        *y
                    }
                } else if (*x < *y) == matches!(self, Self::Min) {
                    *x
                } else {
                    *y
                }
            }
            _ => return Err(Fault::new("invalid math arguments")),
        };
        Ok(Value::Double(value))
    }
}
