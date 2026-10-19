//! Experimental canonical whole-grapheme search; no public/runtime binding yet.
//! Borrow existing UTF-8 and clone iterator state, never normalized text.
use crate::string_normalization::compare;
use core::cmp::Ordering;
use unicode_segmentation::UnicodeSegmentation;

fn matches<'a, 'b>(
    mut text: impl Iterator<Item = &'a str>,
    pattern: impl Iterator<Item = &'b str>,
) -> bool {
    for expected in pattern {
        let Some(actual) = text.next() else {
            return false;
        };
        if compare(actual, expected) != Ordering::Equal {
            return false;
        }
    }
    true
}

pub fn starts_with(text: &str, pattern: &str) -> bool {
    matches(text.graphemes(true), pattern.graphemes(true))
}

pub fn ends_with(text: &str, pattern: &str) -> bool {
    matches(text.graphemes(true).rev(), pattern.graphemes(true).rev())
}

pub fn contains(text: &str, pattern: &str) -> bool {
    if pattern.is_empty() {
        return true;
    }
    let pattern = pattern.graphemes(true);
    let mut candidates = text.graphemes(true);
    loop {
        // Keep the original iterator's context: slicing at arbitrary scalar
        // offsets would change segmentation (for example, regional indicators).
        if matches(candidates.clone(), pattern.clone()) {
            return true;
        }
        if candidates.next().is_none() {
            return false;
        }
    }
}
