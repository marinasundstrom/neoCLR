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
