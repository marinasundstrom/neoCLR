//! Strict culture-independent parsing used by the concrete primitive Parse methods.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ParsedNumber {
    SByte(i8), Byte(u8), Int16(i16), UInt16(u16), UInt32(u32), UInt64(u64),
    Single(f32), Double(f64), Boolean(bool), Int32(i32),
}

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

pub(crate) fn parse(text: &str, kind: Kind) -> ParsedNumber {
    // Int32 status 1 = InvalidFormat, 2 = Overflow. None of these kinds is Int32.
    if kind == Kind::Boolean {
        if text.eq_ignore_ascii_case("true") {
            ParsedNumber::Boolean(true)
        } else if text.eq_ignore_ascii_case("false") {
            ParsedNumber::Boolean(false)
        } else {
            ParsedNumber::Int32(1)
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
                ParsedNumber::Single(value as f32)
            } else {
                ParsedNumber::Double(value)
            }
        } else if !decimal_float(text) {
            ParsedNumber::Int32(1)
        } else if kind == Kind::Single {
            match text.parse::<f32>() {
                Ok(value) if value.is_finite() => ParsedNumber::Single(value),
                _ => ParsedNumber::Int32(2),
            }
        } else {
            match text.parse::<f64>() {
                Ok(value) if value.is_finite() => ParsedNumber::Double(value),
                _ => ParsedNumber::Int32(2),
            }
        }
    } else {
        let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            ParsedNumber::Int32(1)
        } else {
            macro_rules! signed {
                ($ty:ty, $variant:ident) => {
                    match text.parse::<$ty>() {
                        Ok(n) => ParsedNumber::$variant(n),
                        Err(_) => ParsedNumber::Int32(2),
                    }
                };
            }
            macro_rules! unsigned {
                ($ty:ty, $variant:ident) => {
                    if text.starts_with('-') {
                        if digits.bytes().all(|b| b == b'0') {
                            ParsedNumber::$variant(0)
                        } else {
                            ParsedNumber::Int32(2)
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
    }
}
