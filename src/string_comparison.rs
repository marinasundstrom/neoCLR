//! Locale-independent Unicode 17 default simple case folding for string policies.
use core::cmp::Ordering;

fn fold(value: char) -> char {
    let mappings = crate::string_case_folding::MAPPINGS;
    match mappings.binary_search_by_key(&(value as u32), |&(source, _)| source) {
        Ok(index) => char::from_u32(mappings[index].1).expect("valid generated case-fold scalar"),
        Err(_) => value,
    }
}

pub(crate) fn compare_ignore_case(left: &str, right: &str) -> Ordering {
    left.chars().map(fold).cmp(right.chars().map(fold))
}

pub(crate) fn hash_ordinal(value: &str) -> i32 {
    value.bytes().fold(2166136261u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(16777619)
    }) as i32
}

pub(crate) fn hash_ignore_case(value: &str) -> i32 {
    let mut hash = 2166136261u32;
    let mut buffer = [0u8; 4];
    for scalar in value.chars().map(fold) {
        for byte in scalar.encode_utf8(&mut buffer).bytes() {
            hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
        }
    }
    hash as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mapping_is_idempotent_and_hash_consistent() {
        let mut previous = None;
        for &(source, target) in crate::string_case_folding::MAPPINGS {
            assert!(previous.is_none_or(|value| value < source));
            previous = Some(source);
            let source = char::from_u32(source).unwrap();
            let target = char::from_u32(target).unwrap();
            assert_eq!(fold(source), target);
            assert_eq!(fold(target), target);
            let left = format!("prefix{source}suffix");
            let right = format!("PREFIX{target}SUFFIX");
            assert_eq!(compare_ignore_case(&left, &right), Ordering::Equal);
            assert_eq!(hash_ignore_case(&left), hash_ignore_case(&right));
        }
    }

    #[test]
    fn simple_folding_edges_and_order() {
        for (left, right) in [
            ("", ""),
            ("A", "a"),
            ("Å", "å"),
            ("Σ", "ς"),
            ("𐐀", "𐐨"),
            ("ẞ", "ß"),
            ("K", "k"),
            ("ſ", "S"),
            ("A\0B", "a\0b"),
        ] {
            assert_eq!(compare_ignore_case(left, right), Ordering::Equal);
            assert_eq!(hash_ignore_case(left), hash_ignore_case(right));
        }
        for (left, right) in [("ß", "ss"), ("İ", "i"), ("ı", "I"), ("é", "é")] {
            assert_ne!(compare_ignore_case(left, right), Ordering::Equal);
        }
        for (left, right) in [
            ("", "a"),
            ("A", "aa"),
            ("a", "B"),
            ("[", "a"),
            ("\u{e000}", "\u{10000}"),
        ] {
            assert_eq!(compare_ignore_case(left, right), Ordering::Less);
            assert_eq!(compare_ignore_case(right, left), Ordering::Greater);
        }
    }
}
