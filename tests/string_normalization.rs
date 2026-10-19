#[path = "../src/string_normalization.rs"]
mod string_normalization;
#[path = "../src/string_normalization_data.rs"]
mod string_normalization_data;
use std::cmp::Ordering;
use string_normalization::{Canonical, compare, hash};

fn text(column: &str) -> String {
    column
        .split_whitespace()
        .map(|scalar| char::from_u32(u32::from_str_radix(scalar, 16).unwrap()).unwrap())
        .collect()
}
fn nfd(input: &str) -> String {
    Canonical::new(input)
        .map(|scalar| char::from_u32(scalar).unwrap())
        .collect()
}
#[test]
fn unicode_17_normalization_conformance() {
    let mut count = 0;
    for line in include_str!("fixtures/normalization-17.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let columns: Vec<_> = line.split(';').map(text).collect();
        assert_eq!(columns.len(), 5);
        for column in &columns[..3] {
            assert_eq!(nfd(column), columns[2], "row {count}");
            assert_eq!(compare(column, &columns[2]), Ordering::Equal, "row {count}");
            assert_eq!(hash(column), hash(&columns[2]), "row {count}");
        }
        for column in &columns[3..] {
            assert_eq!(nfd(column), columns[4], "row {count}");
        }
        count += 1;
    }
    assert_eq!(count, 20034);
}
#[test]
fn long_runs_are_stable_and_reset_at_starters() {
    for length in [31, 32, 33, 4096] {
        let marks = "\u{301}\u{315}\u{300}\u{323}".repeat(length);
        let expected = format!(
            "{}{}{}",
            "\u{323}".repeat(length),
            "\u{301}\u{300}".repeat(length),
            "\u{315}".repeat(length)
        );
        assert_eq!(nfd(&marks), expected);
        assert_eq!(
            nfd(&format!("a{marks}b{marks}")),
            format!("a{expected}b{expected}")
        );
        assert_eq!(hash(&marks), hash(&expected));
    }
}
#[test]
fn canonical_is_not_compatibility_case_or_visual_equality() {
    for (left, right) in [
        ("é", "e\u{301}"),
        ("Å", "Å"),
        ("각", "각"),
        ("\0é", "\0e\u{301}"),
    ] {
        assert_eq!(compare(left, right), Ordering::Equal);
        assert_eq!(hash(left), hash(right));
    }
    for (left, right) in [
        ("A", "a"),
        ("A", "А"),
        ("ﬁ", "fi"),
        ("①", "1"),
        ("é", "e"),
        ("\u{301}\u{300}", "\u{300}\u{301}"),
    ] {
        assert_ne!(compare(left, right), Ordering::Equal);
    }
    assert_eq!(compare("", ""), Ordering::Equal);
    assert_eq!(compare("a", "ab"), Ordering::Less);
    let ordinal = b"ASCII".iter().fold(2166136261u32, |value, byte| {
        (value ^ u32::from(*byte)).wrapping_mul(16777619)
    });
    assert_eq!(hash("ASCII"), ordinal as i32);
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
fn comparison_and_hashing_allocate_nothing_including_long_runs() {
    let left = format!("é{}", "\u{315}\u{323}".repeat(4096));
    let right = format!(
        "e\u{301}{}{}",
        "\u{323}".repeat(4096),
        "\u{315}".repeat(4096)
    );
    ALLOCATIONS.with(|count| count.set(Some(0)));
    let comparison = compare(std::hint::black_box(&left), std::hint::black_box(&right));
    let hashes = (
        hash(std::hint::black_box(&left)),
        hash(std::hint::black_box(&right)),
    );
    let allocations = ALLOCATIONS.with(|count| count.replace(None).unwrap());
    assert_eq!(comparison, Ordering::Equal);
    assert_eq!(hashes.0, hashes.1);
    assert_eq!(allocations, 0);
}
