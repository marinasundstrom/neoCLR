# String search — 2026-10-19

Status: **implemented development APIs, qualified on interpreter/JIT/AOT**.
String.Contains/StartsWith/EndsWith and their StringComparison overloads use the
shared [search kernel](../../../src/string_search.rs) over borrowed UTF-8.
The default Canonical policy compares complete
extended graphemes with the existing allocation-free canonical comparison kernel.
Empty patterns match. Case and compatibility differences do not. Exact ordinal
predicates retain their existing ability to match inside a grapheme.

This follows Swift's documented character-by-character canonical prefix/suffix
model ([language guide](https://docs.swift.org/swift-book/LanguageGuide/StringsAndCharacters.html)),
extended here to substring membership. Unlike .NET's default ordinal Contains,
`e` does not match part of `e` plus combining acute. Unicode
[UAX #29](https://www.unicode.org/reports/tr29/) explains normalization-invariant
grapheme boundaries. No public search position type is added.

## Bug found and fixed

Normalization conformance row 18051 failed with the previous Unicode 16
segmenter: `0061 0315 0300 05AE 1ACF 0062` and its canonically reordered spelling
were segmented differently. U+1ACF is a newer combining mark. Comparing complete
graphemes therefore disagreed with whole-string canonical equality.

Both interpreter and native text helper now pin `unicode-segmentation = 1.13.2`,
whose [release notes](https://github.com/unicode-rs/unicode-segmentation/blob/v1.13.2/README.md)
specify Unicode 17 support. This is a production prerequisite fix, independent of
the search kernel. Newer marks can change String counts, indexing,
slicing and valid Char constructions. Bytes remain unchanged. Scalar category
classification still uses its separate Unicode 16 table; Raven literal diagnostics
still use host StringInfo. Rebuild native helpers/bundles before qualification;
existing built bundles are not upgraded by editing Cargo.lock.

## Prerequisite evidence

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

## Costs and remaining scope

The kernel clones small iterator state and compares borrowed grapheme ranges; no
array, substring or normalized String is created. Contains uses a simple candidate
scan, with worst-case repeated prefix work proportional to text times pattern
length (plus segmentation/normalization costs). Long combining runs trade repeated
scans for bounded scratch. No linear-time or throughput claim is made.

Search position types and the deferred String-as-view representation are unchanged.
Replace remains ordinal; this addition does not silently change its callers.


## Public integration evidence

The runtime library is authored in Raven; no legacy neoIL library mirror was added.
The three methods default to Canonical. Explicit Ordinal reuses the existing byte
services; OrdinalIgnoreCase uses Unicode 17 simple folding of scalars without
normalization or expansion. Ordinal modes can match inside graphemes. Every valid
mode accepts empty patterns, while invalid modes fault even for empty text.

```
cargo test --release --features native-metadata --test string_search --test native
cargo test --release --manifest-path tools/aot-native-text/Cargo.toml search_tests
python3 docs/experiments/string-search/check.py target/string-search-bundle-v1/bundle target/string-search-main-v1 --invalid target/string-search-invalid-v1 --output target/string-search-validation.json
python3 docs/experiments/string-search/measure.py target/string-search-allocation-v1 --output target/string-search-allocations.json
```

Nine search/folding tests and five registry tests pass. The native ABI test checks
all six operation/policy combinations and rejected UTF-8, pointers and tags without
publishing output. [Twelve public executions](validation.json) pass: the main
consumer and each invalid-policy method under interpreter, JIT and AOT. The main
consumer covers canonical spellings, new combining marks, exact byte retention,
emoji/flags/CRLF, empty/overlapping matches, explicit modes and unchanged named
ordinal methods.

The [allocation consumer](Allocation.rvn) performs six searches per iteration for
128 iterations. [Counters](allocations.json): baseline/check both allocate and reclaim
2 managed objects; eager collections rise from 517 to 3,077. Thus the searches add
no managed allocations in this workload, but call-boundary GC work still exists.
The kernel tests separately observe no Rust heap allocations. No throughput claim.

Artifacts: compiler code 2d22055dc plus metadata adapter e4709c451 in
canonical-text-compiler-v1; string-search-libraries-v2 and string-search-bundle-v1;
main/invalid/allocation-v1 consumers. Documentation-only source comment additions
are built separately in string-search-libraries-v3 for the API snapshot; runtime
method bodies are unchanged from the qualified library. Rebuild all runtime/backend
components for these new private service bindings. The native source fingerprint
includes string_search.rs, and search-only bindings select the Unicode archive.

Compiler-side bridge documentation is recorded in Raven codex/neoclr commit
`c74aea8e1`; no compiler source, Runtime Contract option or ordinary .NET behavior
changed. API snapshots include all six overloads and their parameter/result/remarks
contracts. Website publication remains a separate operation.
