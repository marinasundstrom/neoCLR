//! Unicode 17 full default casing, independent of locale and host Unicode data.
use crate::string_casing_data::{CASE_IGNORABLE, CASED, LOWER, UPPER};

fn has_property(value: char, ranges: &[(u32, u32)]) -> bool {
    let value = value as u32;
    let index = ranges.partition_point(|&(start, _)| start <= value);
    index > 0 && value <= ranges[index - 1].1
}

pub(crate) fn convert(text: &str, uppercase: bool) -> String {
    let mappings = if uppercase { UPPER } else { LOWER };
    let mut result = String::with_capacity(text.len());
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
            result.push('ς');
        } else if let Ok(index) =
            mappings.binary_search_by_key(&(value as u32), |&(source, _)| source)
        {
            result.push_str(mappings[index].1);
        } else {
            result.push(value);
        }
        if !has_property(value, CASE_IGNORABLE) {
            preceded_by_cased = has_property(value, CASED);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pinned_mapping_and_property_range() {
        for (uppercase, table) in [(false, LOWER), (true, UPPER)] {
            let mut previous = None;
            for &(source, expected) in table {
                assert!(previous.is_none_or(|value| value < source));
                previous = Some(source);
                let source = char::from_u32(source).unwrap().to_string();
                assert_eq!(convert(&source, uppercase), expected);
            }
        }
        for table in [CASED, CASE_IGNORABLE] {
            for (index, &(start, end)) in table.iter().enumerate() {
                assert!(start <= end && (index == 0 || table[index - 1].1 < start));
                assert!(has_property(char::from_u32(start).unwrap(), table));
                assert!(has_property(char::from_u32(end).unwrap(), table));
            }
        }
    }

    #[test]
    fn full_default_casing_context_and_expansion() {
        for (input, lower, upper) in [
            ("", "", ""),
            ("Straße ﬃ", "straße ﬃ", "STRASSE FFI"),
            ("İıI", "i\u{307}ıi", "İII"),
            ("ΟΣ", "ος", "ΟΣ"),
            ("ΟΣΑ", "οσα", "ΟΣΑ"),
            ("Σ", "σ", "Σ"),
            ("AΣ\u{301}", "aς\u{301}", "AΣ\u{301}"),
            ("AΣ\u{301}B", "aσ\u{301}b", "AΣ\u{301}B"),
            ("A.Σ", "a.ς", "A.Σ"),
            ("A Σ", "a σ", "A Σ"),
            ("𐐀𐐨👩‍💻", "𐐨𐐨👩‍💻", "𐐀𐐀👩‍💻"),
            ("é e\u{301}\0", "é e\u{301}\0", "É E\u{301}\0"),
        ] {
            assert_eq!(convert(input, false), lower, "lower {input:?}");
            assert_eq!(convert(input, true), upper, "upper {input:?}");
        }
    }
}
