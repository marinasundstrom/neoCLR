//! Allocation-free Unicode 17 casing shared by interpreter and native text services.
use crate::string_casing_data::{CASE_IGNORABLE, CASED, LOWER, UPPER};

pub(crate) fn has_property(value: char, ranges: &[(u32, u32)]) -> bool {
    let value = value as u32;
    let index = ranges.partition_point(|&(start, _)| start <= value);
    index > 0 && value <= ranges[index - 1].1
}

pub(crate) fn append_mapped(text: &str, uppercase: bool, mut append: impl FnMut(&str)) {
    let mappings = if uppercase { UPPER } else { LOWER };
    let mut preceded_by_cased = false;
    for (offset, value) in text.char_indices() {
        // Context is evaluated in the original text, skipping Case_Ignorable.
        let final_sigma = !uppercase
            && value == 'Σ'
            && preceded_by_cased
            && !text[offset + value.len_utf8()..]
                .chars()
                .find(|&c| !has_property(c, CASE_IGNORABLE))
                .is_some_and(|c| has_property(c, CASED));
        if final_sigma {
            append("ς");
        } else if let Ok(index) =
            mappings.binary_search_by_key(&(value as u32), |&(source, _)| source)
        {
            append(mappings[index].1);
        } else {
            append(value.encode_utf8(&mut [0u8; 4]));
        }
        if !has_property(value, CASE_IGNORABLE) {
            preceded_by_cased = has_property(value, CASED);
        }
    }
}
