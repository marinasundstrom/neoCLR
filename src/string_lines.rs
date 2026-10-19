//! Unicode newline sequences shared by the string line-ending operation.
//! Valid UTF-8 input; scanning non-leading bytes cannot match these prefixes.
pub(crate) fn newline_width(bytes: &[u8]) -> usize {
    match bytes {
        [13, 10, ..] => 2,
        [10 | 12 | 13, ..] => 1,
        [0xc2, 0x85, ..] => 2,
        [0xe2, 0x80, 0xa8 | 0xa9, ..] => 3,
        _ => 0,
    }
}
