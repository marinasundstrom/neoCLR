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


## Consistency recommendations and normalization foundation (2026-10-19)

The author invited suggestions to make the string model more consistent after
asking for the assistant's recommendation on canonical equality. The assistant
recommends the following coordinated contract; these recommendations are not yet
public API behavior:

- String and Char equality and hashes should recognize canonical equivalence.
  Boxed equality, compiler constants/patterns and collection policies must agree.
  Keep original UTF-8 bytes, with explicitly ordinal operations for protocols.
- Default text search and replacement should eventually follow the same canonical
  policy, while respecting grapheme boundaries. Do not silently give search a
  different equality relation. Exact byte search belongs to byte views or named
  ordinal APIs; case folding remains a separate policy.
- Search should return a position/range accepted directly by slicing. Prefer
  checked text positions over unlabelled byte offsets; specify owner validation
  and end positions before introducing this API. Existing grapheme integer indices
  remain a separate convenience with scanning costs.
- Construction joins text fragments, not independently fixed characters. A
  combining accent appended to `e` can change the last grapheme; builders must
  preserve this behavior and invalidate any cached boundaries accordingly.
- Explicit copies and borrowed/owned views should retain the Slice/Memory
  philosophy. A future substring view must state owner retention and lifetime
  costs; it is not implemented by this foundation.

Unlike .NET's ordinal String equality, the proposed default follows Swift's
canonical text identity. This improves consistency with neoCLR's grapheme Char,
while changing dictionary key identity and adding Unicode processing costs.
NFD scalar ordering is deterministic, but is not linguistic collation.
[Swift equality semantics](https://docs.swift.org/swift-book/LanguageGuide/StringsAndCharacters.html)
and [Unicode normalization](https://www.unicode.org/reports/tr15/)
provide the semantic basis. Compatibility normalization and visual similarity
are deliberately excluded.

The first implemented slice is a private, unwired Rust normalization kernel
(`src/string_normalization.rs`). It streams canonical decomposition and ordering,
including algorithmic Hangul, without a normalized String or heap scratch.
Short nonstarter runs use 32 fixed slots; longer runs rescan once per occupied
combining class (at most 255 passes). Scratch space is independent of input size;
long runs trade CPU for bounded memory. Eager normalized storage was rejected
because it changes observable bytes, and allocating normalization iterators do
not fit the current allocator-free native text helper. No runtime privileges
are needed by this pure UTF-8 kernel; service integration remains separate work.

Generation uses SHA-256-pinned Unicode 17 UnicodeData and NormalizationTest inputs:
`python3 scripts/generate-string-normalization.py target/unicode-17`. Normal
builds use checked-in outputs and do not fetch Unicode data. The existing Unicode
license covers both tables and the compact conformance fixture. Regeneration is
byte-identical after rustfmt.

Validation: `cargo test --release --test string_normalization` passes four tests,
including all 20,034 official conformance rows, short/long run boundaries,
canonical versus compatibility/case distinctions, and zero measured allocations
for comparison and hashing of long combining runs. An isolated `#![no_std]`
wrapper compiles both functions without an allocator. This establishes algorithm
and memory behavior, not runtime/HTTP throughput or public API integration.
Canonical public services, explicit ordinal preservation, boxed equality,
compiler behavior and three-engine consumers remain release prerequisites.


### Runtime service integration (2026-10-19)

The kernel is now connected to private `StringCompareCanonical`,
`StringHashCanonical`, `CharCompareCanonical` and `CharHashCanonical` services.
The interpreter and native backend use the same Unicode implementation. Char
services consume grapheme storage directly rather than allocating CharText
strings. Neither service changes or normalizes the original UTF-8 storage.
Native adapters validate UTF-8 and leave result slots untouched on failure;
comparison and hashing require no allocator and retain no pointers.

Raven declarations remain internal. Exact InternalCall signatures are admitted
under the existing StringOperations service and native `--bind-utf8-text`
capability. The native representation remains a borrowed immutable text descriptor
and an Int32 output, with checked status codes. String and Char share the native
kernel because their storage is UTF-8; their guest type identities remain distinct.
This private matched ABI is not the proposed future public hosting API. The native
metadata path uses the existing call contract, with no Raven compiler changes,
new Runtime Contract settings or changes to ordinary .NET behavior.

Public equality is deliberately still unchanged in this integration slice.
Before switching it, update the following together:

- String and Char direct equality/operators and content hashes, boxed overrides,
  compiler constant/pattern behavior and dictionary policies.
- StringComparer.Ordinal currently calls String.Equals; make it explicitly
  ordinal so that its equality continues matching its ordinal hash.
- HashCode.Add(string) currently allocates Utf8Encode output and hashes raw bytes;
  use the canonical hash service so containing value objects have matching hashes
  without that temporary array.
- Protocol/value types that depend on exact spelling must select ordinal equality
  and matching hashing explicitly. Audit these before changing defaults.

The website/API reference still describes existing public semantics. Private
services do not introduce a new public API to document or a shipped canonical
String contract. Release qualification remains pending public integration.


Service validation: six interpreter/kernel tests pass, including the complete
Unicode corpus and wrong-return-signature rejection. The native ABI test checks
unchanged original bytes, matching hashes and malformed-input/output validation.
The macOS ARM64 codegen consumer invokes all four services and preserves ordinal
inequality in both arena and native-GC AOT configurations. Commands:

```sh
cargo test --release --test canonical_text_services --test string_normalization
cargo test --release --manifest-path tools/aot-native-text/Cargo.toml canonical_tests
cargo test --release -p neoclr-codegen --test console canonical_text_services_match_interpreter_without_normalizing_storage
```

Native compilation used Xcode's clang and MacOSX26.2.sdk. Public Raven consumers,
JIT execution and release package qualification are not established by these
private service tests. The API reference fingerprint check passes unchanged.

Focused runtime Clippy (`--features native-metadata --test canonical_text_services
-- -D warnings`) passes. The additional strict codegen lint run reports 71 backend
diagnostics (including unused unit expressions, collapsible conditions and
argument-count limits); it is not a passing release gate. These findings require
a separate backend cleanup/qualification rather than being hidden with lint
suppression in the text-service change. Formatting and API fingerprint checks pass.


### Lessons from .NET, Swift and Rust (2026-10-19)

The author asks what neoCLR can learn from these three platforms. The following
is design guidance, not a claim that the public canonical integration is complete.

- **.NET:** retain explicit comparison policies and buffer-oriented APIs.
  [String comparison guidance](https://learn.microsoft.com/en-us/dotnet/standard/base-types/best-practices-strings)
  distinguishes linguistic operations from ordinal identifier matching;
  [memory views](https://learn.microsoft.com/en-us/dotnet/standard/memory-and-spans/)
  avoid copying underlying buffers. For neoCLR, keep exact protocol comparisons
  explicit and let parsing/formatting consume or fill slices directly. UTF-8
  storage deliberately differs from .NET's UTF-16 string representation; ordinal
  byte ordering must not be described as identical to UTF-16 code-unit ordering.
- **Swift:** retain grapheme-oriented text and canonical equality with matching
  hashes, while exposing storage separately. Its
  [String documentation](https://developer.apple.com/documentation/swift/string)
  describes canonical comparison and shared substring storage; its
  [UTF-8 implementation account](https://www.swift.org/blog/utf8-string/)
  illustrates why native UTF-8 helps interoperability. For neoCLR, preserve
  original bytes and avoid allocating normalized copies for comparisons. Shared
  substrings save copies but can retain a large owner; materialization must remain
  available. Grapheme counting/index traversal has a scanning cost, so checked
  positions and forward iteration deserve preference over repeated numeric indexing.
- **Rust:** learn from the distinction between an owned
  [String and borrowed str](https://doc.rust-lang.org/std/string/struct.String.html),
  valid UTF-8 invariants and checked scalar boundaries. A byte slice alone is not
  validated text; a scalar boundary is not necessarily a grapheme boundary.
  neoCLR should carry ownership through Memory and bound synchronous borrowing
  through Slice, rather than attempting to copy Rust's ownership system wholesale.

Recommended follow-ups are a consistent position contract for search and slicing,
explicit text-view materialization, and direct UTF-8 builder/writer formatting.
A Char remains a grapheme, not a fixed-size storage element: avoid allocating an
array of Char values simply to traverse text. Canonical equivalence, case folding
and locale-sensitive collation are separate policies. Exact JSON names, protocol
fields and metadata identities must keep their specified comparison contracts.

Validate ASCII, composed/decomposed accents, emoji sequences, long combining runs,
retained small substrings of large inputs, and builder fragments whose join changes
grapheme boundaries. Measure allocations, retained bytes and CPU separately;
fewer allocations alone do not establish higher HTTP throughput. Small-string
inlining or cached metadata remain measurement-driven options, not release scope.
