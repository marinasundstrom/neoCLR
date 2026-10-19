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


### Public integration investigation (2026-10-19)

Metadata adapter commit `e4709c451` admits exact text Object override references;
Raven native documentation commit `b7d54e946` records the target-specific contract.
The in-progress Raven public consumer passes interpreter execution with canonical
String/Char equality, matching hashes, constant patterns, boxed values, explicit
ordinal policy, dictionary policies and preserved original UTF-8 bytes. All 37
updated boxed-object regressions pass. These results use development artifacts,
not a qualified release bundle; public library/backend edits remain in progress.

Native compilation then exposes a receiver-mode verification gap for String's
intrinsic by-value receiver. Both focused override regressions pass; the intrinsic String case covers the
dedicated receiver path, null checks and rejection of an unrelated override slot
under an explicitly selected Object root. With the receiver check repaired, native compilation reaches the primitive
wrapper gate, which rejects Char Object overrides. Native boxed Char adapters,
complete JIT/AOT consumers and API snapshots remain gates;
the interpreter result must not be presented as cross-engine completion.


### Qualified public canonical integration (2026-10-19)

The [canonical text fixture](../experiments/canonical-text/README.md) now passes
interpreter, JIT and AOT, including original-byte retention, typed/boxed equality
and hashes, constant patterns, canonical/ordinal dictionary policies and ordinal
JSON/SQL/path identities. Null String receivers fault normally in all three modes.
Native source override dispatch and GC registration precede projected null checks.
String ToString preserves its owner; Char ToString preserves original spelling.
HashCode.Add(string) no longer allocates an encoded array. These results supersede
the pending public/backend status above, without claiming an HTTP throughput gain.

This deliberately differs from .NET's default ordinal String equality, following
the author-selected Swift model. Equivalent spellings now identify the same key
under StringComparer.Default; ordinal identity remains explicit. Canonical order
is NFD scalar order, not linguistic collation. Stored bytes are never normalized.
Comparison still incurs Unicode traversal work; hashes are not persistent IDs.

A further record consumer exposes the existing native positional-record boundary:
generated equality/hash/format helpers are not exported. RecordGap.rvn preserves
that reproducer; this integration does not qualify generated record semantics.
The author next requests useful supported string manipulation methods. Trimming is
the proposed first slice, with explicit Unicode and grapheme-boundary semantics;
search positions, splitting and writer-oriented formatting remain follow-ups.


### Whitespace trimming (2026-10-19)

Add the familiar parameterless Trim, TrimStart and TrimEnd family. The policy is
Unicode 17 [White_Space](https://www.unicode.org/Public/17.0.0/ucd/PropList.txt),
all 25 scalars, including newlines and nonbreaking spaces. This follows the
[.NET whitespace contract](https://learn.microsoft.com/en-us/dotnet/api/system.string.trim)
and [Rust scalar-based trimming](https://doc.rust-lang.org/std/primitive.str.html#method.trim).
It is not protocol-specific ASCII whitespace validation. U+200B, U+FEFF and U+180E
are not whitespace under this pinned Unicode version.

Trimming classifies scalars, unlike grapheme indexing. In particular, a leading
space followed by a combining mark loses the space and keeps the mark. Choosing
to retain that entire grapheme would preserve a visual cluster but diverge from
whitespace removal in .NET/Rust. The scalar contract is explicit and tested; it
does not imply that arbitrary numeric byte offsets become text positions.
Char remains a grapheme. Retained bytes are neither normalized nor case folded.

Managed Raven code scans a borrowed ReadOnlySlice<byte> from either edge and uses
the existing checked UTF-8 slice service for materialization. No new InternalCall
or FFI provider is needed: the library owns classification, while existing runtime
code owns string allocation and scalar-boundary validation. The no-change path
returns the original owner, and all-whitespace input returns empty text. Changed
nonempty text copies retained bytes; the existing Value result transport may have
runtime overhead. No temporary byte/Char arrays or per-grapheme strings are built.
This retains compact independent results instead of keeping a potentially large
original owner alive. Custom trim sets, substring views and protocol whitespace
APIs remain separate decisions. See [qualification](../experiments/string-trimming/README.md).


### Corresponding .NET String operations: proposed priorities (2026-10-19)

The author asks which corresponding methods neoCLR can and should add, referencing
[.NET 10 String](https://learn.microsoft.com/en-us/dotnet/api/system.string?view=net-10.0).
This is a proposed sequence, not approval or implementation of every method.
Current String already has grapheme slicing, canonical equality/hash/order, ordinal
predicates/replacement, invariant casing, Join/Concat and whitespace trimming.

1. Add IsNullOrEmpty and IsNullOrWhiteSpace as intentional nullable-boundary
   conveniences; inspect bytes directly rather than materializing Trim results.
   Add Equals(other, comparison) and CompareTo for consistent explicit policy and
   ordering. CompareTo should accompany ComparableTo<string>, not just a name.
2. Unify Contains/StartsWith/EndsWith and eventual Replace around a documented
   comparison policy. Recommend canonical defaults consistent with text identity,
   plus explicit ordinal and simple-fold modes. Current Replace is ordinal despite
   its unqualified name; resolve this inconsistency deliberately, without quietly
   changing protocol callers. Canonical substring matching needs its own kernel
   and boundary rules; whole-string canonical comparison is insufficient.
3. Define typed text positions/match ranges before IndexOf/LastIndexOf or a
   Find/FindLast alternative. Prefer Option for absence and a range preserving the
   original matched spelling. Numeric grapheme indices must not receive byte offsets.
   Define whole-grapheme text matching versus explicit lower-level scalar/byte
   matching, foreign-owner positions and empty matches before implementation.
4. Add Insert, Remove and range replacement on the same position contract.
   Reuse current Range slicing instead of adding a duplicate Substring overload
   family. Result text can have different grapheme boundaries at concatenation joins.
5. Add lazy Split and line traversal over validated text views/ranges, with explicit
   materialization. Unlike [.NET String.Split](https://learn.microsoft.com/en-us/dotnet/api/system.string.split?view=net-10.0),
   avoid requiring a substring array. Define empty fields, limits, delimiter policy,
   CRLF handling and parent-storage retention. Do not promise zero total allocation
   while the iterator representation remains unspecified.
6. Add ReplaceLineEndings with an explicit replacement; if a default is offered,
   consider LF to match neoCLR StringBuilder.AppendLine. That is a deliberate
   divergence from .NET's environment newline default. Do not use it for HTTP framing.
7. Extend Join/Concat to suitable read-only views and sequences. Reference-element
   view support needs qualification before ReadOnlySlice<string> is advertised.
   Single-pass sources need a builder strategy; count/byte sizing must not enumerate
   a one-shot sequence twice. Share typed formatting with StringBuilder/writers
   before adding Format or TryFormat, avoiding mandatory object arrays and boxing.

Useful later: IndexOfAny/LastIndexOfAny with a deliberate grapheme/scalar set model;
custom-grapheme trimming; explicit Normalize/IsNormalized; PadLeft/PadRight.
Normalization returns new requested storage and never silently changes the source;
canonical comparison's NFD stream does not by itself implement NFC/NFKC output.
[Padding](https://learn.microsoft.com/en-us/dotnet/api/system.string.padleft?view=net-10.0)
needs a defined unit and boundary-merging behavior: grapheme counts are not terminal
columns, and repeating a grapheme can change segmentation at joins.

Do not prioritize Clone/Copy, redundant Parse/TryParse identity helpers, UTF-16
CopyTo/ToCharArray as the core transfer abstraction, pinning helpers or implicit
current-culture overloads. Explicit Char snapshots may still be useful; they are
not contiguous text storage. Prefer existing Utf8Bytes and encoding/writer APIs.
Fix eager grapheme/scalar array traversal alongside these additions. Qualify
cross-engine semantics, original-byte preservation and allocations per operation;
keep cultural collation and unmeasured performance claims out of the initial scope.


### Deferred string-as-view discussion (2026-10-19)

The author explores String itself as a view over byte storage, rather than adding
an analogue named StringSlice, and explicitly defers the decision until after the
utility methods. No representation change is adopted. A possible model is an
immutable UTF-8 owner plus byte offset/length; shared slicing can avoid byte copies
but retain a large owner. Mutable byte-array aliases must be excluded, and explicit
compaction would be needed. Whether that handle is a value type is a separate ABI,
boxing and GC question, not a consequence of sharing storage.
[Swift's UTF-8 implementation](https://www.swift.org/blog/utf8-string/) provides
precedent for a struct with shared backing storage; [Rust String](https://doc.rust-lang.org/std/string/struct.String.html)
separates owned text and borrowed str. These are alternatives to investigate,
not evidence that neoCLR has either representation. Existing String utilities
remain usable without changing their current ownership or null contracts.


### First utility additions (2026-10-19)

IsNullOrEmpty and IsNullOrWhiteSpace accept nullable text at an explicit validation
boundary. They inspect existing UTF-8 bytes without allocating a trimmed string.
Equals(other, comparison) reuses String.Compare's canonical, ordinal and simple-fold
policies, including unknown-mode rejection. CompareTo implements ComparableTo<string>
with canonical NFD ordering. Comparison arguments remain non-null, consistent with
neoCLR's explicit nullable-boundary conventions; no culture-sensitive order is implied.
These additions are independent of the deferred String representation decision.
See [the executable fixtures](../experiments/string-utilities/README.md).


### ReplaceLineEndings implementation (2026-10-19)

Follow [.NET's recognized newline set](https://learn.microsoft.com/en-us/dotnet/api/system.string.replacelineendings?view=net-10.0):
CRLF (one match), CR, LF, FF, NEL, LS and PS. VT is not included. Empty replacement
deletes matches; replacement text is never rescanned. Preserve every other UTF-8
byte, including decomposed spelling and embedded NUL. The parameterless overload
uses LF on all platforms, deliberately matching neoCLR StringBuilder.AppendLine
rather than .NET's host-specific newline. Costs: callers wanting CRLF must request
it explicitly; benefit: reproducible text independent of deployment OS. Protocol
parsers must retain their own framing rules.

The Raven surface calls a private runtime service because final immutable-string
allocation and source/replacement GC retention need runtime privileges. A two-pass
scan computes checked output size, returns the original owner when unchanged and
writes directly to the final result. Ordinary Raven chains of Replace would create
intermediate strings; byte-array staging would add an intermediate buffer. A separate
FFI allocator would still need a runtime-owned publication/GC adapter and add an ABI
boundary. Reconsider placement if a safe managed string-construction facility is
introduced. Interpreter and native kernels share the same documented match set;
backend binding is exact and must be used with a matching rebuilt library/bundle.

[Fixtures and allocation evidence](../experiments/string-lines/README.md) distinguish
managed native allocations from host Rust allocations and CPU/GC costs. No HTTP
throughput claim follows from avoiding intermediate strings.


### Canonical search prerequisite (2026-10-19)

The [search experiment](../experiments/string-search/README.md) applies Swift-like
whole-grapheme canonical matching to prefix, suffix and membership checks without
normalizing stored text or allocating substrings. It exposed an actual inconsistency:
Unicode 17 normalization could reorder newer marks across boundaries supplied by
the old Unicode 16 segmenter. Both interpreter and native helper now pin Unicode 17
segmentation; all 766 boundary cases and 20,034 normalization rows pass. This changes
counts/indexing/Char admission for affected text, while preserving original bytes.
Scalar category classification remains on its separate Unicode 16 table.

The search kernel remains experimental: public methods, explicit comparison modes
and native bindings are pending. A simple candidate scan uses bounded memory but
can repeat work for long shared prefixes; zero-allocation diagnostics do not imply
linear-time search or higher server throughput. The deferred storage/view decision
is unaffected.


### Char helper shortlist (2026-10-19; proposed)

The author asks whether helpers belong on Char. Current Raven Char provides
FromString, typed/boxed canonical equality, matching hashing, CompareTo and
ToString. Candidate additions are TryFromString returning Option<char>, Utf8Bytes,
non-eager scalar traversal and explicit-policy Equals, followed by a defined
whole-grapheme IsWhiteSpace. These are proposals, not implemented API contracts.

Unlike .NET's UTF-16 Char predicates, neoCLR's Char can contain multiple Unicode
scalars. UnicodeScalar already owns scalar category checks; assigning one category
to an arbitrary grapheme would require a deliberate rule. If casing is added, its
result should be string because full mappings may expand beyond one grapheme.
White-space classification must state whether every scalar must be whitespace;
a base space plus a combining mark is a useful counterexample. Borrowed byte and
scalar access must avoid creating an intermediate String/array and retain the
existing lifetime/immutability rules. The String representation decision remains
deferred. Public additions require separate source/API/three-engine qualification.


### Public search integration (2026-10-19)

The experimental kernel above is now shared by the interpreter and native text
helper, with public Raven Contains/StartsWith/EndsWith methods and explicit
StringComparison overloads. Canonical compares complete graphemes; Ordinal compares
bytes; OrdinalIgnoreCase applies the existing simple-fold scalar policy without
normalization or expansion. The latter two may match inside a grapheme. Empty
patterns match for valid policies; unknown modes fault before search. Preserved
named ordinal methods and Replace retain their existing behavior.

This deliberately follows Swift's canonical prefix/suffix semantics rather than
.NET's default ordinal Contains. The benefit is consistency with neoCLR Char and
String identity; costs are segmentation/normalization and potentially repeated
candidate-prefix scans. No cultural matching or linear-time guarantee is implied.
[Twelve cross-engine checks and allocation evidence](../experiments/string-search/README.md#public-integration-evidence)
qualify the additions without claiming HTTP throughput gains. Source methods remain
Raven; only private string access/search services use shared runtime kernels. No
public position type or storage/view representation change accompanies this work.


### Refined shared-storage String proposal (2026-10-19; not implemented)

The author clarifies that String remains an object: callers supply real backing
storage, while substring and split results allocate descriptors sharing the content.
This is independent of making String a value type. The assistant recommends a
prototype after the current helpers: immutable UTF-8 owner plus byte offset/length,
nested slices flattened onto that owner, distinct String object identity, and an
explicit independent-copy operation to release retained parent storage.

[Swift's Substring implementation/documentation](https://github.com/swiftlang/swift/blob/main/stdlib/public/core/Substring.swift)
provides a shipped comparison for storage sharing and retention; using String for
both full and partial windows would be a deliberate neoCLR choice. The historical
[Swift String manifesto](https://github.com/swiftlang/swift/blob/main/docs/StringManifesto.md)
discusses the same-type shared-storage alternative, but is design history rather
than the current API contract. Compared with copying .NET substrings, the benefit
is fewer byte copies; the costs include wrapper allocation, retained large buffers,
owner tracing, offset-aware native services and interop materialization where needed.

A ReadOnlyMemory<byte> is not sufficient proof of immutable storage if mutable
aliases survive. External buffers need copying, freezing or enforced ownership
transfer. Literal/interpolation/decoder constructors can still create storage
without asking ordinary callers to manage buffers. Grapheme slicing, canonical
comparison and original-byte preservation remain unchanged. Existing interpreter
StringValue shares whole StringData but has no window; native text layout/GC must
be coordinated. This remains a proposal, not approval or implementation of a layout
change, and it does not bypass the author's utility-first sequencing.


### First Char utilities implemented (2026-10-19)

The follow-up adds TryFromString with Option<char> and explicit-policy Equals,
qualified by the [Char utility consumer](../experiments/char-utilities/README.md).
Validation uses the first grapheme extent rather than constructing an eager Char
array; equality compares existing Char text without String conversion. Compared
with .NET Char.TryParse's one-code-unit Boolean/out contract, Option and complete
graphemes follow neoCLR's current language and text model. Non-null inputs keep
nullable handling explicit. Native diagnostic workloads add no managed allocations;
interpreter Char buffer copying remains a representation cost, not a solved issue.

The proposed Utf8Bytes, scalar traversal and whole-grapheme whitespace helpers
remain separate work. Inspection confirms Value::Char stores an owned Rust String;
CharText clones it into StringValue, so exposing that path as borrowed would be
misleading. The shared-storage proposal above could address this ownership issue,
but no String or Char layout change is adopted in this utility slice.

## Composite formatting and Formattable — 2026-10-19

The author explicitly defers shared-storage String views and directs the remaining
useful helpers, naming String.Format and Console Write/WriteLine format overloads.
After the assistant identified missing type-specific formatting, the author directed:
“Add a formattable interface then.” The implementation uses a shared Raven parser
and System.Formattable.ToString(string), with fixed-arity overloads avoiding argument
arrays. [Contracts, .NET comparison and validation](../experiments/composite-formatting/README.md)
record grapheme alignment, current limits and remaining allocation/provider gaps.
This does not change String storage or infer approval of culture-provider design.
