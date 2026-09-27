//! Strict culture-independent parsing used by the concrete primitive Parse methods.
use crate::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    SByte,
    Byte,
    Int16,
    UInt16,
    UInt32,
    UInt64,
    Single,
    Double,
    Boolean,
}

fn decimal_float(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut i = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let mut digits = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
        digits += 1;
    }
    if bytes.get(i) == Some(&b'.') {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return false;
    }
    if matches!(bytes.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(bytes.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if start == i {
            return false;
        }
    }
    i == bytes.len()
}

pub(crate) fn parse(text: &str, kind: Kind) -> Value {
    // Int32 status 1 = InvalidFormat, 2 = Overflow. None of these kinds is Int32.
    let payload = if kind == Kind::Boolean {
        if text.eq_ignore_ascii_case("true") {
            Value::Boolean(true)
        } else if text.eq_ignore_ascii_case("false") {
            Value::Boolean(false)
        } else {
            Value::Int32(1)
        }
    } else if matches!(kind, Kind::Single | Kind::Double) {
        let special = match text {
            "NaN" => Some(f64::NAN),
            "Infinity" | "+Infinity" => Some(f64::INFINITY),
            "-Infinity" => Some(f64::NEG_INFINITY),
            _ => None,
        };
        if let Some(value) = special {
            if kind == Kind::Single {
                Value::Single(value as f32)
            } else {
                Value::Double(value)
            }
        } else if !decimal_float(text) {
            Value::Int32(1)
        } else if kind == Kind::Single {
            match text.parse::<f32>() {
                Ok(value) if value.is_finite() => Value::Single(value),
                _ => Value::Int32(2),
            }
        } else {
            match text.parse::<f64>() {
                Ok(value) if value.is_finite() => Value::Double(value),
                _ => Value::Int32(2),
            }
        }
    } else {
        let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            Value::Int32(1)
        } else {
            macro_rules! signed {
                ($ty:ty, $variant:ident) => {
                    match text.parse::<$ty>() {
                        Ok(n) => Value::$variant(n),
                        Err(_) => Value::Int32(2),
                    }
                };
            }
            macro_rules! unsigned {
                ($ty:ty, $variant:ident) => {
                    if text.starts_with('-') {
                        if digits.bytes().all(|b| b == b'0') {
                            Value::$variant(0)
                        } else {
                            Value::Int32(2)
                        }
                    } else {
                        signed!($ty, $variant)
                    }
                };
            }
            match kind {
                Kind::SByte => signed!(i8, SByte),
                Kind::Byte => unsigned!(u8, Byte),
                Kind::Int16 => signed!(i16, Int16),
                Kind::UInt16 => unsigned!(u16, UInt16),
                Kind::UInt32 => unsigned!(u32, UInt32),
                Kind::UInt64 => unsigned!(u64, UInt64),
                _ => unreachable!(),
            }
        }
    };
    Value::Erased(Box::new(payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn payload(text: &str, kind: Kind) -> Value {
        match parse(text, kind) {
            Value::Erased(value) => *value,
            _ => unreachable!(),
        }
    }
    #[test]
    fn integer_ranges_grammar_and_unambiguous_byte_status() {
        for (kind, minimum, maximum, below, above) in [
            (Kind::SByte, "-128", "127", "-129", "128"),
            (Kind::Byte, "0", "255", "-1", "256"),
            (Kind::Int16, "-32768", "32767", "-32769", "32768"),
            (Kind::UInt16, "0", "65535", "-1", "65536"),
            (Kind::UInt32, "0", "4294967295", "-1", "4294967296"),
            (
                Kind::UInt64,
                "0",
                "18446744073709551615",
                "-1",
                "18446744073709551616",
            ),
        ] {
            for text in [minimum, maximum, "+0", "-0", "00000001"] {
                assert!(
                    !matches!(payload(text, kind), Value::Int32(_)),
                    "{kind:?} {text}"
                );
            }
            for text in [below, above] {
                assert_eq!(payload(text, kind), Value::Int32(2));
            }
            for text in [
                "",
                "+",
                " 1",
                "1 ",
                "１２",
                "1\0",
                "1.0",
                "0x10",
                "999999999999999999999999x",
            ] {
                assert_eq!(payload(text, kind), Value::Int32(1), "{kind:?} {text:?}");
            }
        }
        assert_eq!(payload("1", Kind::Byte), Value::Byte(1));
        assert_eq!(payload("2", Kind::Byte), Value::Byte(2));
    }
    #[test]
    fn floating_syntax_specials_rounding_and_range() {
        for kind in [Kind::Single, Kind::Double] {
            for text in [
                ".5",
                "1.",
                "-0",
                "+1.25e-2",
                "NaN",
                "Infinity",
                "+Infinity",
                "-Infinity",
                "1e-9999",
            ] {
                assert!(
                    !matches!(payload(text, kind), Value::Int32(_)),
                    "{kind:?} {text}"
                );
            }
            assert_eq!(payload("1e9999", kind), Value::Int32(2));
            for text in [
                "", ".", "+.", "1e", "1e+", "nan", "inf", "1_000", " 1", "1,5", "1e9999x",
            ] {
                assert_eq!(payload(text, kind), Value::Int32(1));
            }
        }
        assert!(
            matches!(payload("-0", Kind::Single), Value::Single(n) if n == 0.0 && n.is_sign_negative())
        );
        assert!(
            matches!(payload("-0", Kind::Double), Value::Double(n) if n == 0.0 && n.is_sign_negative())
        );
        assert_eq!(payload("16777217", Kind::Single), Value::Single(16777216.0));
        assert_eq!(payload("3.4028236e38", Kind::Single), Value::Int32(2));
        assert_eq!(
            payload("1.7976931348623159e308", Kind::Double),
            Value::Int32(2)
        );
    }
    #[test]
    fn boolean_is_not_numeric_or_whitespace_tolerant() {
        assert_eq!(payload("TrUe", Kind::Boolean), Value::Boolean(true));
        assert_eq!(payload("FALSE", Kind::Boolean), Value::Boolean(false));
        for text in ["1", "0", " true", "false ", "true\0", ""] {
            assert_eq!(payload(text, Kind::Boolean), Value::Int32(1));
        }
    }
}
