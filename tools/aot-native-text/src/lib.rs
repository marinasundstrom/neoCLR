//! Allocation-free native text validation. The Unicode dependency is deliberately
//! pinned to the interpreter's version; no shared managed runtime is linked.
#![cfg_attr(not(test), no_std)]
use unicode_segmentation::UnicodeSegmentation;

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    unsafe extern "C" { fn abort() -> !; }
    // An internal implementation failure, never a guest validation outcome.
    unsafe { abort() }
}

fn single_grapheme(bytes: &[u8]) -> bool {
    let Ok(text) = core::str::from_utf8(bytes) else { return false };
    let mut graphemes = text.graphemes(true);
    graphemes.next().is_some() && graphemes.next().is_none()
}

/// Returns 1 only for one valid extended grapheme; 0 for empty/invalid/multiple.
///
/// # Safety
/// `bytes` must point to `length` readable immutable bytes. No pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_is_single_grapheme_v1(bytes: *const u8, length: usize) -> i32 {
    if bytes.is_null() || length > isize::MAX as usize { return 0; }
    let bytes = unsafe { core::slice::from_raw_parts(bytes, length) };
    i32::from(single_grapheme(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_extended_graphemes_without_normalizing() {
        for text in ["\0", "A", "å", "e\u{301}", "👨‍👩‍👧‍👦", "🇸🇪", "\r\n"] {
            assert!(single_grapheme(text.as_bytes()), "{text:?}");
        }
        for bytes in [b"".as_slice(), b"ab", b"\xff", "🇸🇪🇳🇴".as_bytes()] {
            assert!(!single_grapheme(bytes));
        }
    }
}

#[path = "../../../src/string_casing_data.rs"]
mod string_casing_data;
#[path = "../../../src/string_casing_kernel.rs"]
mod string_casing_kernel;
#[path = "../../../src/string_case_folding.rs"]
mod string_case_folding;
#[path = "../../../src/string_comparison.rs"]
mod string_comparison;

unsafe fn input<'a>(bytes: *const u8, length: usize) -> Option<&'a str> {
    if bytes.is_null() || length > isize::MAX as usize { return None; }
    core::str::from_utf8(unsafe { core::slice::from_raw_parts(bytes, length) }).ok()
}

/// Private two-pass casing kernel. A null output measures; a nonnull output has
/// capacity bytes. No pointers are retained. Invalid UTF-8 is rejected.
///
/// # Safety
/// Input must be readable for length bytes. A nonnull output must be writable for
/// capacity bytes and disjoint from input and written. Written must be aligned and
/// writable. On insufficient capacity, output may contain a partial mapping.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_unicode_case_v1(bytes: *const u8, length: usize, upper: i32,
    output: *mut u8, capacity: usize, written: *mut usize) -> i32 {
    let Some(text) = (unsafe { input(bytes, length) }) else { return 3 };
    if written.is_null() || !(0..=1).contains(&upper) { return 3; }
    let mut size = 0usize;
    let mut failed = false;
    string_casing_kernel::append_mapped(text, upper != 0, |part| {
        let Some(end) = size.checked_add(part.len()) else { failed = true; return };
        if !output.is_null() {
            if end > capacity { failed = true; return; }
            unsafe { core::ptr::copy_nonoverlapping(part.as_ptr(), output.add(size), part.len()); }
        }
        size = end;
    });
    if failed { return 3; }
    unsafe { *written = size; }
    0
}

/// Compares validated text using the interpreter's pinned simple folding.
///
/// # Safety
/// Inputs must be readable for their lengths. Output must be aligned, writable,
/// and disjoint from both input ranges. No pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_unicode_compare_v1(left: *const u8, left_len: usize,
    right: *const u8, right_len: usize, output: *mut i32) -> i32 {
    let (Some(left), Some(right)) = (unsafe { input(left, left_len) }, unsafe { input(right, right_len) }) else { return 3 };
    if output.is_null() { return 3; }
    unsafe { *output = string_comparison::compare_ignore_case(left, right) as i32; }
    0
}

/// Visits grapheme byte ranges or Unicode scalar values in one linear pass.
/// The callback must not retain borrowed pointers or reenter guest execution.
///
/// # Safety
/// Input must be readable for length bytes. Visitor must be a valid function
/// accepting state, and state must satisfy its contract for every invocation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_unicode_visit_v1(bytes: *const u8, length: usize, scalars: i32,
    visitor: unsafe extern "C" fn(usize, usize, *mut core::ffi::c_void) -> i32,
    state: *mut core::ffi::c_void) -> i32 {
    let Some(text) = (unsafe { input(bytes, length) }) else { return 3 };
    if scalars == 1 {
        for (at, value) in text.chars().enumerate() {
            let status = unsafe { visitor(at, value as usize, state) };
            if status != 0 { return status; }
        }
    } else if scalars == 0 {
        for (offset, part) in text.grapheme_indices(true) {
            let status = unsafe { visitor(offset, part.len(), state) };
            if status != 0 { return status; }
        }
    } else { return 3; }
    0
}

/// Hashes validated UTF-8 with the same content policy as the interpreter.
/// # Safety
/// Input must be readable for length bytes; output must be aligned, writable,
/// and disjoint from input. No pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_unicode_hash_v1(bytes: *const u8, length: usize,
    ignore_case: i32, output: *mut i32) -> i32 {
    let Some(text) = (unsafe { input(bytes, length) }) else { return 3 };
    if output.is_null() || !(0..=1).contains(&ignore_case) { return 3; }
    unsafe { *output = if ignore_case == 0 { string_comparison::hash_ordinal(text) }
        else { string_comparison::hash_ignore_case(text) }; }
    0
}

#[cfg(test)]
mod hash_tests {
    use super::*;
    fn hash(text: &str, policy: i32) -> i32 {
        let mut value = 0;
        assert_eq!(unsafe { neoclr_unicode_hash_v1(text.as_ptr(), text.len(), policy, &mut value) }, 0);
        value
    }

    #[test]
    fn content_hashes_preserve_utf8_nul_and_simple_fold_contracts() {
        assert_eq!(hash("", 0), 2166136261u32 as i32);
        assert_eq!(hash("hello", 0), 0x4f9f2cabu32 as i32);
        for (left, right) in [("Å", "å"), ("Σ", "ς"), ("𐐀", "𐐨"), ("A\0B", "a\0b")] {
            assert_eq!(hash(left, 1), hash(right, 1));
            assert_ne!(hash(left, 0), hash(right, 0));
        }
        assert_ne!(hash("ß", 1), hash("ss", 1));
        assert_ne!(hash("é", 0), hash("e\u{301}", 0));
        let long = "雪".repeat(128);
        assert_eq!(hash(&long, 0), string_comparison::hash_ordinal(&long));
        let mut output = 123;
        assert_eq!(unsafe { neoclr_unicode_hash_v1([255].as_ptr(), 1, 0, &mut output) }, 3);
        assert_eq!(output, 123);
        assert_eq!(unsafe { neoclr_unicode_hash_v1(b"a".as_ptr(), 1, 2, &mut output) }, 3);
        assert_eq!(output, 123);
    }
}

// The interpreter and native adapters share grammar and rounding, not transport.
#[path = "../../../src/numeric_parse_core.rs"]
mod numeric_parse;

unsafe fn parse_number(text: *const u8, output: *mut u64, kind: numeric_parse::Kind) -> i32 {
    if text.is_null() || output.is_null() { return 3; }
    let length = unsafe { text.cast::<u64>().read() };
    if length > isize::MAX as u64 { return 3; }
    let bytes = unsafe { core::slice::from_raw_parts(text.add(8), length as usize) };
    let Ok(text) = core::str::from_utf8(bytes) else { return 3; };
    use numeric_parse::ParsedNumber as Parsed;
    // Private AOT erased tags, synchronized with value_profile::erased_tag.
    let (tag, payload) = match numeric_parse::parse(text, kind) {
        Parsed::Int32(n) => (1, n as u32 as u64),
        Parsed::Byte(n) => (2, n as u64),
        Parsed::Boolean(n) => (3, u64::from(n)),
        Parsed::UInt64(n) => (6, n),
        Parsed::SByte(n) => (7, n as u8 as u64),
        Parsed::Int16(n) => (8, n as u16 as u64),
        Parsed::UInt16(n) => (9, n as u64),
        Parsed::UInt32(n) => (10, n as u64),
        Parsed::Single(n) => (11, n.to_bits() as u64),
        Parsed::Double(n) => (12, n.to_bits()),
    };
    unsafe { output.write(tag); output.add(1).write(payload); }
    0
}

macro_rules! parser_adapter {
    ($symbol:ident, $kind:ident) => {
        /// Parse a native immutable UTF-8 string into private erased result lanes.
        /// # Safety
        /// `text` addresses its readable length header and bytes; `output` addresses
        /// two writable u64 lanes. Both pointers are aligned and live for this call.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $symbol(text: *const u8, output: *mut u64) -> i32 {
            unsafe { parse_number(text, output, numeric_parse::Kind::$kind) }
        }
    };
}
parser_adapter!(neoclr_parse_sbyte_v1, SByte);
parser_adapter!(neoclr_parse_byte_v1, Byte);
parser_adapter!(neoclr_parse_int16_v1, Int16);
parser_adapter!(neoclr_parse_uint16_v1, UInt16);
parser_adapter!(neoclr_parse_uint32_v1, UInt32);
parser_adapter!(neoclr_parse_uint64_v1, UInt64);
parser_adapter!(neoclr_parse_single_v1, Single);
parser_adapter!(neoclr_parse_double_v1, Double);
parser_adapter!(neoclr_parse_boolean_v1, Boolean);

#[cfg(test)]
mod numeric_adapter_tests {
    use super::*;
    type Parser = unsafe extern "C" fn(*const u8, *mut u64) -> i32;

    #[test]
    fn publishes_exact_erased_kinds_and_float_bits() {
        let cases: [(Parser, &str, [u64; 2]); 12] = [
            (neoclr_parse_sbyte_v1, "-128", [7, 128]),
            (neoclr_parse_byte_v1, "1", [2, 1]),
            (neoclr_parse_byte_v1, "-1", [1, 2]),
            (neoclr_parse_int16_v1, "-32768", [8, 32768]),
            (neoclr_parse_uint16_v1, "65535", [9, 65535]),
            (neoclr_parse_uint32_v1, "4294967295", [10, u32::MAX as u64]),
            (neoclr_parse_uint64_v1, "18446744073709551615", [6, u64::MAX]),
            (neoclr_parse_single_v1, "-0", [11, (-0.0f32).to_bits() as u64]),
            (neoclr_parse_single_v1, "1e39x", [1, 1]),
            (neoclr_parse_double_v1, "-Infinity", [12, f64::NEG_INFINITY.to_bits()]),
            (neoclr_parse_double_v1, "1e999", [1, 2]),
            (neoclr_parse_boolean_v1, "TrUe", [3, 1]),
        ];
        for (parse, text, expected) in cases {
            let mut storage = vec![0u64; 1 + text.len().div_ceil(8)];
            storage[0] = text.len() as u64;
            unsafe { core::ptr::copy_nonoverlapping(text.as_ptr(), storage.as_mut_ptr().add(1).cast(), text.len()); }
            let mut output = [u64::MAX; 2];
            assert_eq!(unsafe { parse(storage.as_ptr().cast(), output.as_mut_ptr()) }, 0);
            assert_eq!(output, expected, "{text}");
        }
    }

    #[test]
    fn null_and_invalid_utf8_fault_without_publishing() {
        let mut output = [99, 98];
        assert_eq!(unsafe { neoclr_parse_single_v1(core::ptr::null(), output.as_mut_ptr()) }, 3);
        assert_eq!(output, [99, 98]);
        let invalid = [1u64, 255];
        assert_eq!(unsafe { neoclr_parse_double_v1(invalid.as_ptr().cast(), output.as_mut_ptr()) }, 3);
        assert_eq!(output, [99, 98]);
    }
}
