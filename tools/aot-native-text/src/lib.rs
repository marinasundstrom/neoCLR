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
