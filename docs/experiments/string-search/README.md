# Canonical search prerequisite — 2026-10-19

Status: **experimental kernel, not public String APIs**. `kernel.rs` implements
Contains/StartsWith/EndsWith semantics over borrowed UTF-8, comparing complete
extended graphemes with the existing allocation-free canonical comparison kernel.
Empty patterns match. Case and compatibility differences do not. Exact ordinal
predicates retain their existing ability to match inside a grapheme.

This follows Swift's documented character-by-character canonical prefix/suffix
model ([language guide](https://docs.swift.org/swift-book/LanguageGuide/StringsAndCharacters.html)),
extended here to substring membership. Unlike .NET's default ordinal Contains,
`e` does not match part of `e` plus combining acute. Unicode
[UAX #29](https://www.unicode.org/reports/tr29/) explains normalization-invariant
grapheme boundaries. No public search policy or position type is added yet.

## Bug found and fixed

Normalization conformance row 18051 failed with the previous Unicode 16
segmenter: `0061 0315 0300 05AE 1ACF 0062` and its canonically reordered spelling
were segmented differently. U+1ACF is a newer combining mark. Comparing complete
graphemes therefore disagreed with whole-string canonical equality.

Both interpreter and native text helper now pin `unicode-segmentation = 1.13.2`,
whose [release notes](https://github.com/unicode-rs/unicode-segmentation/blob/v1.13.2/README.md)
specify Unicode 17 support. This is a production prerequisite fix, independent of
the experimental search kernel. Newer marks can change String counts, indexing,
slicing and valid Char constructions. Bytes remain unchanged. Scalar category
classification still uses its separate Unicode 16 table; Raven literal diagnostics
still use host StringInfo. Rebuild native helpers/bundles before qualification;
existing built bundles are not upgraded by editing Cargo.lock.

## Evidence

```
cargo test --release --test string_search --test grapheme_segmentation --test string_normalization
cargo test --release --test grapheme_chars --test string_construction --test canonical_text_services
cargo test --release --manifest-path tools/aot-native-text/Cargo.toml
```

- Six search tests: canonical spelling, empty/overlapping matches, emoji/flag/CRLF
  boundaries, 20,034 normalization conformance rows, and a whole-range oracle
  covering 20,736 pairs of inputs assembled from boundary-sensitive fragments.
- All 766 Unicode 17 grapheme conformance rows pass forward and backward in the
  root crate and native helper. The same fixture is used by both test targets.
- Four normalization tests, 13 runtime construction/service tests and nine native
  helper tests pass, including new combining-mark Char admission.
- Long combining sequences (8,192 marks) exercise bounded normalization scratch.
  Allocator instrumentation observes **zero heap allocations** across Contains,
  StartsWith, EndsWith and a failed Contains call; setup allocations are excluded.
  Source bytes are unchanged. This is not a CPU, GC or HTTP throughput measurement.

Fixture: [Unicode GraphemeBreakTest 17.0.0](https://www.unicode.org/Public/17.0.0/ucd/auxiliary/GraphemeBreakTest.txt),
retained in `tests/fixtures/grapheme-break-17.txt` with trailing comment whitespace
removed. Upstream SHA-256
`e2d134d2c52919bace503ebb6a551c1855fe1a1faec18478c78fff254a1793ec`.
Retained fixture SHA-256: `88ead7e1c192b5d9767eb4c54cb62e177ab82afd972efc247e04599fab66b719`.
The original Unicode copyright/terms header is preserved; tests never fetch data.

## Costs and next integration gate

The kernel clones small iterator state and compares borrowed grapheme ranges; no
array, substring or normalized String is created. Contains uses a simple candidate
scan, with worst-case repeated prefix work proportional to text times pattern
length (plus segmentation/normalization costs). Long combining runs trade repeated
scans for bounded scratch. No linear-time or throughput claim is made.

Public Raven methods, explicit comparison overloads, runtime/native search service
bindings and interpreter/JIT/AOT consumers remain to implement. Search position
types and the deferred String-as-view representation are unchanged. This experiment
must not be listed as an available public String API.
