#[path = "../docs/experiments/string-search/kernel.rs"]
mod search;
#[path = "../src/string_normalization.rs"]
#[allow(dead_code)]
mod string_normalization;
#[path = "../src/string_normalization_data.rs"]
mod string_normalization_data;

#[test]
fn canonical_spelling_matches_without_changing_storage() {
    for (original, equivalent) in [
        ("é", "e\u{301}"),
        ("Å", "Å"),
        ("각", "각"),
        ("a\u{315}\u{323}", "a\u{323}\u{315}"),
        ("\0é", "\0e\u{301}"),
    ] {
        for (left, right) in [(original, equivalent), (equivalent, original)] {
            assert!(search::starts_with(&format!("{left}!"), right));
            assert!(search::ends_with(&format!("!{left}"), right));
            assert!(search::contains(&format!("!{left}!"), right));
        }
    }
}

#[test]
fn never_matches_partial_graphemes() {
    for (whole, part) in [
        ("e\u{301}", "e"),
        ("e\u{301}", "\u{301}"),
        ("👩‍💻", "👩"),
        ("👩‍💻", "💻"),
        ("👍🏽", "👍"),
        ("🇸🇪", "🇸"),
        ("🇸🇪", "🇪"),
        ("\r\n", "\r"),
        ("\r\n", "\n"),
    ] {
        assert!(!search::starts_with(whole, part), "{whole:?}/{part:?}");
        assert!(!search::ends_with(whole, part), "{whole:?}/{part:?}");
        assert!(!search::contains(whole, part), "{whole:?}/{part:?}");
        assert!(!search::contains(&format!("!{whole}!"), part));
        // Explicit ordinal search intentionally has a different boundary policy.
        assert!(whole.contains(part));
    }
    assert!(!search::contains("🇦🇧🇨🇩", "🇧🇨"));
    assert!(search::contains("🇦🇧🇨🇩", "🇨🇩"));
    assert!(search::ends_with("🇦🇧🇨", "🇨"));
    assert!(!search::ends_with("🇦🇧🇨🇩", "🇩"));
}

#[test]
fn empty_overlap_and_non_equivalent_cases() {
    for text in ["", "ascii", "é", "👩‍💻", "\r\n"] {
        assert!(search::contains(text, ""));
        assert!(search::starts_with(text, ""));
        assert!(search::ends_with(text, ""));
        assert!(search::contains(text, text));
        assert!(search::starts_with(text, text));
        assert!(search::ends_with(text, text));
    }
    assert!(search::contains("aaaab", "aaab"));
    for (text, pattern) in [
        ("", "a"),
        ("a", "aa"),
        ("é", "É"),
        ("ﬁ", "fi"),
        ("①", "1"),
        ("ß", "ss"),
        ("A", "А"),
    ] {
        assert!(!search::contains(text, pattern));
        assert!(!search::starts_with(text, pattern));
        assert!(!search::ends_with(text, pattern));
    }
}

#[test]
fn normalization_conformance_pairs_match_as_whole_text() {
    let decode = |column: &str| -> String {
        column
            .split_whitespace()
            .map(|scalar| char::from_u32(u32::from_str_radix(scalar, 16).unwrap()).unwrap())
            .collect()
    };
    let mut rows = 0;
    for line in include_str!("fixtures/normalization-17.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let columns: Vec<_> = line.split(';').map(decode).collect();
        for original in &columns[..3] {
            for (left, right) in [(original, &columns[2]), (&columns[2], original)] {
                assert!(search::starts_with(left, right), "row {rows}");
                assert!(search::ends_with(left, right), "row {rows}");
                assert!(search::contains(left, right), "row {rows}");
            }
        }
        rows += 1;
    }
    assert_eq!(rows, 20034);
}

#[test]
fn matches_whole_range_oracle() {
    use unicode_segmentation::UnicodeSegmentation;
    // The oracle allocates every original-boundary range and compares it as a
    // whole String; the kernel must reach the same answer without those copies.
    let pieces = [
        "", "a", "é", "e\u{301}", "\u{301}", "\r", "\n", "🇦", "🇧", "👩", "\u{200d}", "💻",
    ];
    let mut cases = Vec::new();
    for left in pieces {
        for right in pieces {
            cases.push(format!("{left}{right}"));
        }
    }
    for text in &cases {
        let mut boundaries: Vec<_> = text
            .grapheme_indices(true)
            .map(|(offset, _)| offset)
            .collect();
        boundaries.push(text.len());
        for pattern in &cases {
            let mut expected = (false, false, false);
            for (index, &start) in boundaries.iter().enumerate() {
                for &end in &boundaries[index..] {
                    if string_normalization::compare(&text[start..end], pattern)
                        == std::cmp::Ordering::Equal
                    {
                        expected.0 = true;
                        expected.1 |= start == 0;
                        expected.2 |= end == text.len();
                    }
                }
            }
            assert_eq!(
                (
                    search::contains(text, pattern),
                    search::starts_with(text, pattern),
                    search::ends_with(text, pattern)
                ),
                expected,
                "{text:?}/{pattern:?}"
            );
        }
    }
}

struct CountingAllocator;
thread_local! {
    static ALLOCATIONS: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}
unsafe impl std::alloc::GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        ALLOCATIONS.with(|count| {
            if let Some(value) = count.get() {
                count.set(Some(value + 1));
            }
        });
        unsafe { std::alloc::System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        unsafe { std::alloc::System.dealloc(ptr, layout) }
    }
}
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

#[test]
fn long_combining_runs_search_without_heap_allocation() {
    let original = format!("é{}", "\u{315}\u{323}".repeat(4096));
    let equivalent = format!(
        "e\u{301}{}{}",
        "\u{323}".repeat(4096),
        "\u{315}".repeat(4096)
    );
    let text = format!("{original}!{original}");
    let before = text.clone();
    ALLOCATIONS.with(|count| count.set(Some(0)));
    let input = std::hint::black_box(text.as_str());
    let pattern = std::hint::black_box(equivalent.as_str());
    let results = (
        search::contains(input, pattern),
        search::starts_with(input, pattern),
        search::ends_with(input, pattern),
        search::contains(input, "missing"),
    );
    let allocations = ALLOCATIONS.with(|count| count.replace(None).unwrap());
    assert_eq!(results, (true, true, true, false));
    assert_eq!(allocations, 0);
    assert_eq!(text.as_bytes(), before.as_bytes());
}
