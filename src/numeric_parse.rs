//! Interpreter transport for the shared allocation-free primitive parser.
use crate::Value;
#[path = "numeric_parse_core.rs"]
mod core;
pub(crate) use core::Kind;

pub(crate) fn parse(text: &str, kind: Kind) -> Value {
    use core::ParsedNumber as Parsed;
    let value = match core::parse(text, kind) {
        Parsed::SByte(n) => Value::SByte(n),
        Parsed::Byte(n) => Value::Byte(n),
        Parsed::Int16(n) => Value::Int16(n),
        Parsed::UInt16(n) => Value::UInt16(n),
        Parsed::UInt32(n) => Value::UInt32(n),
        Parsed::UInt64(n) => Value::UInt64(n),
        Parsed::Single(n) => Value::Single(n),
        Parsed::Double(n) => Value::Double(n),
        Parsed::Boolean(n) => Value::Boolean(n),
        Parsed::Int32(n) => Value::Int32(n),
    };
    Value::Erased(Box::new(value))
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
