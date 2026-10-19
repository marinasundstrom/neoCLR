# String API review — 2026-10-19

Source review at neoCLR 76b72a296; proposals below are not shipped APIs.
The author explicitly reaffirms Swift as the semantic model for String. This
qualifies the general .NET API baseline: familiar operations remain useful
comparisons, but UTF-16 indexing and legacy overload families do not define text.
The separate [StringBuilder review](string-builder-api-review-20261019.md) covers
construction. Existing [text research](text-abstraction.md) remains relevant;
this review does not silently replace previously recorded choices.

## Current API and implementation

[Source](../../runtime/raven/src/System/String.rvn) and
[runtime services](../../src/native.rs) establish the following inventory.

| Area | Implemented | Gap or cost |
| --- | --- | --- |
| Characters | Length, int/Index indexing, Get, Range slicing, Slice | Grapheme counts and indexed lookups scan. Slicing copies; repeated indexed traversal can be quadratic. |
| Views | Utf8Bytes: ReadOnlySlice<byte> | Borrowed immutable bytes are distinct from character positions; no general substring text view. |
| Traversal | Sequence<char>, GetIterator, GetScalars | Iterator creation snapshots graphemes; scalar access also creates an array. This is an implementation cost, not required by Unicode semantics. |
| Search | ContainsOrdinal, StartsWithOrdinal, EndsWithOrdinal | No IndexOf, LastIndexOf or match-range result. Exact matches may occur within a grapheme. |
| Replacement | Replace(string, string) | Exact nonoverlapping UTF-8 matches, empty search faults; no cultural or canonical matching. No-match/equal-replacement path reuses immutable text in the interpreter. |
| Comparison | Equals/operators, CompareOrdinal, explicit comparison modes | Exact default identity differs from Swift canonical equivalence. Ignore-case uses documented simple folding, not general linguistic search. |
| Construction | Character-sequence constructor, Concat, Join, Intern | Character constructor snapshots. Join currently requires a whole string array; selected windows need a suitable view contract. |
| Transformation | ToUpperInvariant, ToLowerInvariant | No general trim/split family or composite Format. Do not infer their absence is a deliberate semantic contract. |
| Legacy byte operations | GetUtf8ByteCount, SliceUtf8 (obsolete) | Internal callers remain. Migrate through explicit byte/encoding contracts before removal. |

## Swift comparison and decisions needed

Swift exposes grapheme characters, position-based indexing and Unicode views.
Its equality recognizes canonical equivalence, and substrings can share storage.
These are separate capabilities; UTF-8 storage alone does not provide them.
[Swift language guide](https://docs.swift.org/swift-book/LanguageGuide/StringsAndCharacters.html)
and [String reference](https://developer.apple.com/documentation/swift/string)
are the semantic sources. Swift's [UTF-8 implementation article](https://www.swift.org/blog/utf8-string/)
illustrates storage options, not a requirement to reproduce its ABI or interop.

1. **Settle text positions before IndexOf.** Prefer a typed position/range that can
   be consumed directly by text slicing. A byte offset must never masquerade as
   the existing integer grapheme index. Specify ownership, foreign-string checks,
   end positions and grapheme-boundary behavior. Alternatively, an explicitly
   grapheme-ordinal result is simpler but rescans and cannot describe every exact
   byte match. This decision applies to search, replace, split and substring APIs.
2. **Review canonical equality as a coordinated contract.** Current exact equality
   was previously deliberate. Swift alignment means reviewing String and Char
   equality, hashing, dictionary keys, constant folding, patterns and native/interpreter
   parity together. Keep explicit ordinal operations available for protocols.
   Do not normalize stored bytes merely to obtain equivalent comparison. The cost
   includes Unicode normalization work and data; the benefit is consistent text
   identity across canonical spellings. No behavior changes in this review.
3. **Prioritize allocation-free traversal.** A cursor retaining immutable text can
   advance through boundaries without materializing the complete Char array.
   Char values may still allocate under the current representation; distinguish
   eliminating the eager container from eliminating all character allocations.
4. **Add operations from real consumers.** Search/ranges and trimming/splitting are
   stronger immediate gaps than copying a Format overload family. Splitting can
   return views, but retained parent storage and explicit owned copies must be
   documented. Array/Memory slices are not automatically valid text slices.
5. **Share formatting machinery.** Investigate typed writer-oriented formatting
   and interpolation lowering. String-producing convenience APIs should materialize
   only the final result; avoid object arrays and boxing as the mandatory core.
   Locale, formatting errors and resource limits need explicit contracts.

.NET remains useful for comparison of operation roles, not index units. Its
[String API](https://learn.microsoft.com/en-us/dotnet/api/system.string?view=net-10.0)
is not a checklist to import wholesale. A source-language surface can remain
familiar while using Swift-like text semantics and neoCLR lifetime rules.

## Validation required for implementation

Use ASCII, composed/decomposed accents, combining-mark-only matches, emoji ZWJ
sequences, regional indicators, CRLF, empty/end positions and foreign-owner indices.
Exercise equality/hash consistency and dictionary lookup before changing identity.
Measure traversal allocations and repeated search/slicing separately from HTTP
throughput. Qualify interpreter/JIT/AOT and compiler folding with matching Unicode
data. Existing tests and source inspection support this audit; no new benchmark or
new API execution is claimed.
