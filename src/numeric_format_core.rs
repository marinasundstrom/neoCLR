//! Shared bounded floating formatting; no temporary heap allocation.
use core::fmt::{self, Write};
pub(crate) struct Buffer { pub bytes: [u8; 1024], pub len: usize }
impl Buffer {
    fn new() -> Self { Self { bytes: [0; 1024], len: 0 } }
    fn text(&self) -> &str { core::str::from_utf8(&self.bytes[..self.len]).unwrap() }
    fn byte(&mut self, b: u8) -> Result<(), fmt::Error> {
        if self.len == self.bytes.len() { return Err(fmt::Error); }
        self.bytes[self.len] = b; self.len += 1; Ok(())
    }
}
impl Write for Buffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if s.len() > self.bytes.len() - self.len { return Err(fmt::Error); }
        self.bytes[self.len..self.len+s.len()].copy_from_slice(s.as_bytes()); self.len += s.len(); Ok(())
    }
}
/// Code retains case. Precision -1 means omitted; caller validates standard syntax.
pub(crate) fn format(value: f64, single: bool, code: u8, precision: i32, swedish: bool) -> Result<Buffer, fmt::Error> {
    let upper = code.to_ascii_uppercase();
    if !matches!(upper, b'G'|b'R'|b'F'|b'N'|b'E') || !(-1..=99).contains(&precision) { return Err(fmt::Error); }
    let mut raw = Buffer::new();
    if !value.is_finite() {
        raw.write_str(if value.is_nan() { "NaN" } else if value.is_sign_negative() { "-Infinity" } else { "Infinity" })?;
        return Ok(raw);
    }
    if matches!(upper, b'F'|b'N') {
        let p = if precision < 0 { 2 } else { precision as usize };
        if single { write!(raw, "{:.*}", p, value as f32)?; } else { write!(raw, "{:.*}", p, value)?; }
    } else {
        let mut scientific = Buffer::new();
        let p = if upper == b'E' { Some(if precision < 0 { 6 } else { precision as usize }) }
            else if upper == b'G' && precision > 0 { Some(precision as usize - 1) } else { None };
        match (single, p) {
            (true, Some(p)) => write!(scientific, "{:.*e}", p, value as f32)?,
            (false, Some(p)) => write!(scientific, "{:.*e}", p, value)?,
            (true, None) => write!(scientific, "{:e}", value as f32)?,
            (false, None) => write!(scientific, "{:e}", value)?,
        }
        let (mantissa, exponent) = scientific.text().split_once('e').ok_or(fmt::Error)?;
        let exponent: i32 = exponent.parse().map_err(|_| fmt::Error)?;
        let negative = mantissa.starts_with('-');
        let mut digits = [0u8; 110]; let mut count = 0;
        for b in mantissa.bytes().filter(|b| b.is_ascii_digit()) { digits[count] = b; count += 1; }
        if upper != b'E' { while count > 1 && digits[count-1] == b'0' { count -= 1; } }
        if negative { raw.byte(b'-')?; }
        let limit = if upper == b'G' && precision > 0 { precision } else if single { 9 } else { 17 };
        if upper == b'E' || exponent < -4 || exponent >= limit {
            raw.byte(digits[0])?;
            if count > 1 { raw.byte(b'.')?; for &b in &digits[1..count] { raw.byte(b)?; } }
            raw.byte(if code.is_ascii_lowercase() { b'e' } else { b'E' })?;
            raw.byte(if exponent < 0 { b'-' } else { b'+' })?;
            write!(raw, "{:0width$}", exponent.unsigned_abs(), width=if upper == b'E' { 3 } else { 2 })?;
        } else {
            let point = exponent + 1;
            if point <= 0 {
                raw.write_str("0.")?;
                for _ in 0..-point { raw.byte(b'0')?; }
                for &b in &digits[..count] { raw.byte(b)?; }
            } else {
                for index in 0..(point as usize).max(count) {
                    if index == point as usize { raw.byte(b'.')?; }
                    raw.byte(if index < count { digits[index] } else { b'0' })?;
                }
            }
        }
    }
    if !swedish && upper != b'N' { return Ok(raw); }
    let mut output = Buffer::new();
    let start = usize::from(raw.bytes[0] == b'-');
    let end = raw.bytes[..raw.len].iter().position(|b| !b.is_ascii_digit() && *b != b'-').unwrap_or(raw.len);
    for (i, &b) in raw.bytes[..raw.len].iter().enumerate() {
        if upper == b'N' && i > start && i < end && (end-i)%3 == 0 {
            output.write_str(if swedish { "\u{a0}" } else { "," })?;
        }
        output.byte(if swedish && b == b'.' { b',' } else { b })?;
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn text(v:f64,s:bool,c:u8,p:i32,sv:bool)->String { let b=format(v,s,c,p,sv).unwrap(); b.text().into() }
    #[test] fn deterministic_finite_roundtrips() {
        let mut bits = 0x123456789abcdef0u64;
        for _ in 0..4096 {
            bits = bits.wrapping_mul(6364136223846793005).wrapping_add(1);
            let double = f64::from_bits(bits);
            if double.is_finite() {
                assert_eq!(text(double, false, b'R', -1, false).parse::<f64>().unwrap().to_bits(), bits);
            }
            let single = f32::from_bits(bits as u32);
            if single.is_finite() {
                assert_eq!(text(single as f64, true, b'G', -1, false).parse::<f32>().unwrap().to_bits(), bits as u32);
            }
        }
    }
    #[test] fn formats_boundaries_and_rounding() {
        assert_eq!(text(2.5,false,b'F',0,false),"2");
        assert_eq!(text(3.5,false,b'F',0,false),"4");
        assert_eq!(text(-0.0,false,b'G',-1,false),"-0");
        assert_eq!(text(1234.5,false,b'N',2,true),"1\u{a0}234,50");
        assert_eq!(text(0.1,true,b'G',-1,false),"0.1");
        assert_eq!(text(1234.5,false,b'E',2,false),"1.23E+003");
        assert_eq!(text(0.00001,false,b'g',-1,false),"1e-05");
        assert_eq!(text(1e16,false,b'G',-1,false),"10000000000000000");
        assert_eq!(text(1e17,false,b'G',-1,false),"1E+17");
        for v in [f64::MIN_POSITIVE,f64::from_bits(1),f64::MAX,-f64::MAX,0.1,-0.0,1.2345678901234567] {
            assert_eq!(text(v,false,b'R',-1,false).parse::<f64>().unwrap().to_bits(),v.to_bits());
        }
        for v in [f32::MIN_POSITIVE,f32::from_bits(1),f32::MAX,-f32::MAX,0.1,-0.0] {
            assert_eq!(text(v as f64,true,b'R',-1,false).parse::<f32>().unwrap().to_bits(),v.to_bits());
        }
        assert!(format(f64::MAX,false,b'N',99,true).unwrap().len < 1024);
    }
}
