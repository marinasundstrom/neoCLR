# Text abstraction: characters before encoding

Current contract and design review, revised **2026-09-27**. The author approved graphemes as the
default and then directed implementation of the Swift-inspired model. The guiding
goal is an improved .NET-like experience suited to modern computing, with familiar
names and explicit lower-level operations. The grapheme/sequence surface is in
Preview 10; explicit comparison modes are later development additions. The review
below proposes further changes, without implementing or approving them.

## Public contract

- String is immutable Unicode text. UTF-8 is the canonical internal representation
  and the default explicit byte conversion, not the public character unit.
- Char is exactly one extended grapheme cluster. Raven character literals accept
  combining sequences and emoji sequences; `Char.FromString(text)` validates a
  dynamic value and faults if it is empty or contains more than one cluster.
- `String.Length` counts graphemes. String implements `Sequence<char>` (and thus
  Collection/Iterable); ordinary iteration and the read-only integer indexer yield
  Char. Count is an explicit Collection member, not a public String property.
- `Char.ToString()` preserves its original text. `Equals` and `CompareTo` are
  ordinal, consistent with String; neither silently normalizes text.
- `String.GetScalars()` returns `Sequence<uint>` of Unicode scalar values, independent
  of encoding. `System.Text.UnicodeScalar` contains the classification predicates
  formerly on Char. Category-based predicates reject surrogate/out-of-range values;
  ASCII predicates simply test the numeric ASCII range.
- UTF-8 encoding/decoding and explicitly named byte operations remain available.
  Possible future Utf8String and AsciiString types could expose encoding-specific
  operations and guarantees. They would complement the neutral String/Char API,
  not define its character semantics. No such types or general Encoding hierarchy
  are introduced in this slice.

The author explicitly affirmed that Char is not a numeric format. Its backing
storage does not justify numeric casts; their absence is intentional API design.

The scalar view uses uint for this small preview surface, not a new scalar value
type. This is provisional and may become a dedicated type with construction-time
validation. Char's scalar values are available through `character.ToString().GetScalars()`.

## Representation, safety and costs

The runtime stores Char as owned, immutable, validated UTF-8 text. Copying a Char
copies its text; it has no public fixed byte width or native inline layout. Managed
fields, arrays, byrefs and defaults work with the value. The default is the NUL
character, not an empty or invalid character. Numeric conversions and arithmetic
are rejected by Raven; direct IL cannot store an integer in a Char slot. Native
integer-pointer access and `sizeof Char` are unsupported. Array payload limits
include the owned bytes of each character, not only the value header.

Length requires segmentation, and iteration currently creates a snapshot. These
choices cost scanning, allocation and copying. They are a correctness-first preview
implementation, not a performance claim. A position/cursor API and more efficient
iteration may follow. The later sequence slice added integer indexing by scanning
from the start; it does not promise constant-time random access. Repeated indexed
traversal can therefore take quadratic work. This is a source-level complexity
observation, not a benchmark or a reason to optimize before selecting contracts.

Segmentation uses unicode-segmentation **1.12.0**, pinned to **Unicode 16**, matching
the classification data version. The later simple-fold table uses Unicode 17; the
versions are currently mixed, as reviewed below. Upgrading the rules can change boundaries and
counts. Raven diagnoses literals using the host .NET StringInfo rules; the target
runtime revalidates every constructed character with its pinned rules. Host Unicode
versions may differ; matching literal diagnostics to the pinned target rules remains
a tooling limitation to resolve before a stable language contract.

A cluster approximates a perceived character, not necessarily one rendered glyph.
Joining strings can change the boundary at their join: counts are not generally
additive. Precomposed é and e plus an accent both have Length 1 but remain distinct
under ordinal equality. Normalization, culture-sensitive comparison and collation
are separate future design decisions. Development OrdinalIgnoreCase uses simple
folding without normalization; see the [comparison contract](../ordinal-text.md#explicit-comparison-modes-development).

## Comparisons and migration

Sources reviewed 2026-09-19:

- [.NET Unicode model](https://learn.microsoft.com/en-us/dotnet/standard/base-types/character-encoding-introduction):
  Char is a UTF-16 code unit, Rune a scalar and StringInfo exposes text elements.
  neoCLR retains familiar String/Char names but chooses graphemes for ordinary
  character operations. This reduces accidental splitting of text at the cost of
  variable-size values, segmentation work and incompatibility with numeric Char.
- [Swift String and Character](https://docs.swift.org/swift-book/LanguageGuide/StringsAndCharacters.html):
  the main precedent for grapheme characters and separate scalar/encoding views.
  Swift's string positions illustrate why an integer array index is not the only
  way to navigate text. We have not adopted Swift's whole comparison/API contract.
- [Rust strings](https://doc.rust-lang.org/book/ch08-02-strings.html): explicit byte
  and scalar iteration and absence of integer character indexing make units/costs
  clear. Its scalar `char` alone would not deliver the chosen perceived-character
  default.
- [Unicode UAX #29](https://www.unicode.org/reports/tr29/): the normative segmentation
  model and its limits. The runtime version is pinned above, not automatically
  whatever Unicode revision this link currently describes.

This supersedes the intermediate four-byte scalar Char implementation. Rebuild the
compiler, core metadata, runtime library and executable together. Use
`RavenGraphemeChar=true` instead of `RavenUnicodeScalarChar=true`; the old compiler
option remains available for the earlier experimental target only. Move numeric
classification to UnicodeScalar and replace character/integer casts with explicit
scalar traversal. No neoCLR policy is integrated into Raven main.

The archived Neo bootstrap profile retains its borrowed collection contracts and
omits the new String collection methods. Its non-collection String/Char operations
come from the same Raven sources through generated bootstrap fragments. The Raven
profile exposes the complete contract above.

## Executable evidence

[The Raven sample](../experiments/raven-target/samples/library-grapheme-strings.rvn)
compiles, imports, verifies and runs on neoCLR: `Aé👨‍👩‍👧‍👦🇸🇪` has 4 graphemes,
12 scalars and 37 UTF-8 bytes. It exercises literals, matching, arrays, direct and
interface iteration and explicit classification. Native tests cover invalid
construction/storage, default initialization, CRLF/NUL, combining sequences, emoji,
flags, skin tones and snapshot element/byte limits. Compiler semantic tests cover
valid/invalid literals, prohibited numeric operations and unchanged .NET defaults.

## System.Text foundation review — 2026-09-27

**Review conclusion:** keep immutable valid UTF-8 text, grapheme Char and familiar
String operations, but do not build the selected System.Text capabilities around
`Sequence<char>`. A grapheme is suitable for character-facing operations; it is
not the fixed-size processing unit needed by encoders, buffers or parser positions.
The user identifies future System.Text work as dependent on getting these crucial
String interfaces right first, then clarifies that the goal is to select useful
capabilities to bring over, not copy the entire namespace.

The following are **assistant recommendations**, not newly implemented APIs or
blanket author approval. The [library tracker](../tracking/library-data.md#string-design-review-before-further-expansion)
owns sequencing. This review does not reopen HTTP or require a whole-runtime rewrite.

### Is the current String model a problem?

**Assessment: viable, with significant API consequences; no evidence from this
review justifies abandoning it.** UTF-8 storage, valid-text invariants, grapheme Char,
exact equality and collection conformance are separate choices. We can keep the
first three while improving interfaces around them. Calling the API modern means
consistent units, explicit transformations, useful typed failures and composable
incremental processing—not reproducing the newest overload list.

| Impact | Inherent to the choice, or current implementation? | Severity and response |
| --- | --- | --- |
| UTF-8 storage | Variable-width scalar encoding is inherent. ASCII occupies one byte; other scalar widths vary. UTF-16 also has variable-width supplementary scalars. | **Acceptable tradeoff.** Suitable for UTF-8 file/protocol boundaries; no universal memory/speed advantage is claimed. UTF-16/native interoperability needs conversion and explicit error rules. |
| Grapheme Char | A perceived character can contain many scalars and has no fixed encoded size. This is inherent, even with better storage. | **Foundational constraint.** Char cannot be a native fixed-width cell or the universal encoder buffer unit. Supply scalar/byte processing separately. If fixed-size Char ABI compatibility becomes a requirement, reconsider the choice. |
| Length and integer indexing | Counting/locating graphemes requires segmentation unless indexes/counts are stored. The current implementation rescans; repeated indexing can be quadratic. | **Ergonomic risk, not a correctness defect.** Do not imply array costs. Prefer iteration/positions for traversal. Caching/cursors are possible implementation choices, with memory/lifetime costs; benchmark only a selected workload. |
| Character copies and iteration | Char currently owns text, and enumeration creates snapshots. These allocations are implementation choices; variable-size character content is inherent. | **Potential practical cost.** Do not make whole-string materialization a requirement for codecs. No measured regression or need for a storage rewrite is established here. |
| Search units versus Char indexing | Ordinal search may end inside a grapheme; the existing byte slice allows scalar boundaries. The conflict is in a future ambiguous index API, not invalid text today. | **Must resolve before IndexOf-like APIs.** Return a range with clear units/boundaries, or use delimiter operations. Do not return a byte/scalar offset and let callers treat it as a Char ordinal. |
| Concatenation and streaming | Appended scalars can merge graphemes across the join. This is inherent Unicode behavior, not an implementation bug. | **Must shape builders and decoders.** Valid text chunks may split a larger grapheme. Character-stream finalization and limits belong to a separate layer; total grapheme count is not additive. |
| Valid-text restriction | Some foreign byte strings or UTF-16 sequences do not represent valid Unicode text. String deliberately cannot preserve them as text. | **Real interoperability limitation.** Preserve opaque bytes/foreign code units in boundary types when lossless round-trip matters; never silently replace data to fit String. It does not require weakening String itself. |
| Equality and user expectations | Exact equality distinguishes canonical spellings; current simple folding differs from .NET's OrdinalIgnoreCase. Neither choice follows automatically from UTF-8. | **Policy/discoverability risk.** Keep exact default identity, offer explicit transformations when needed, and settle truthful fold naming. Grapheme Char alone does not solve human search or Unicode identifier security. |
| Unicode data versions | Grapheme boundaries/classification/folding depend on versioned rules; mixed 16/17 data is current integration debt. | **Fix before widening Unicode behavior.** Document one target version and validate upgrades plus host-literal differences. Do not persist grapheme indexes or folded-key assumptions across versions without an application policy. |

Rust's [OsString](https://doc.rust-lang.org/std/ffi/struct.OsString.html) illustrates
why lossless operating-system strings can need a domain separate from Unicode text.
This review does not audit neoCLR filesystem marshalling or claim it already has
that solution. A raw filename round-trip requirement should trigger that boundary
audit. Similarly, explicit decoder fallback in [.NET Encoding.GetString](https://learn.microsoft.com/en-us/dotnet/api/system.text.encoding.getstring?view=net-10.0)
is a conversion policy, not a reason to treat replacement as lossless.

Three alternatives were considered:

- **Current grapheme Char plus explicit scalar/byte facilities — recommended.** Keeps
  the selected user-facing model and fixes the missing lower-level vocabulary.
  Its costs remain segmentation, variable-size values and extra API concepts.
- **Scalar Char with a separate Grapheme type/view.** Simplifies scanner/classifier
  signatures and gives a fixed scalar value, but still does not give constant-time
  UTF-8 indexing or solve perceived-character operations. It would break current
  literals, collections, storage/projections and consumers. Reconsider if the boundary
  prototype cannot serve core workloads without pervasive awkward conversions.
- **UTF-16 code-unit Char/String semantics.** Simplifies copying .NET buffer signatures
  and some interop, but conflicts with the selected platform direction and restores
  surrogate/code-unit concerns. Choose only if .NET binary/ABI compatibility becomes
  a primary product requirement; familiarity alone is insufficient reason.

The major problem would be **hiding these differences**, for example calling an
encoder output count “chars written” when it actually counts bytes, or using a
String sequence as if it were a fixed-width buffer. The model itself supports the
selected modern API portfolio provided these boundaries are made explicit.

### What the current code and probes establish

The [small executable corpus](../experiments/text-review/README.md) passes on the
current neoCLR/Raven toolchain and was also run in .NET, Swift, Rust and Go. Its text
`Aé👨‍👩‍👧‍👦🇸🇪` has 4 graphemes, 12 scalars, 37 UTF-8 bytes and, in the foreign
probes, 18 UTF-16 code units. These are different useful measurements, not competing
answers to one encoding-independent buffer-length question.

Two findings matter more than adding convenience methods:

- neoCLR indexes `é` as one Char, but `StartsWithOrdinal("e")` succeeds and
  `SliceUtf8(1, 2)` returns the combining mark alone. Exact search is useful, but a
  future search result cannot always be a grapheme integer index. Scalar-boundary
  text ranges and whole-character ranges must not be interchangeable by accident.
- Joining `e` and a combining mark produces one grapheme from two input Chars.
  A builder cannot calculate final character Length by summing appended Char counts.
  Likewise a decoder chunk ending with `e` is not proof of a completed grapheme:
  later scalars can extend it. Valid UTF-8 chunks and complete-character chunks
  require different contracts.

Source inspection also confirms that grapheme indexing/counting scan, scalar and
character iteration materialize snapshots, and StreamWriter returns a UTF-8 byte
count despite accepting String. Existing LINQ operators generally use iterators;
there is no evidence here that they all require random access or need rewriting.
Older design prose incorrectly said the integer indexer was still absent. This
review corrects that prose rather than treating it as an implementation gap.

### Framework lessons, with their limits

Primary sources reviewed 2026-09-27; local tool versions and outputs are in the
[probe record](../experiments/text-review/results.json).

| Reference | Established behavior / status | Lesson for neoCLR |
| --- | --- | --- |
| [.NET text model](https://learn.microsoft.com/en-us/dotnet/standard/base-types/character-encoding-introduction) | String/Char use UTF-16 units; Rune and StringInfo address scalars and text elements. | Preserve discoverable roles, not their unit assumptions. Our Char cannot be substituted into every .NET signature. |
| [.NET comparison guidance](https://learn.microsoft.com/en-us/dotnet/standard/base-types/best-practices-strings) and [runtime discussion #43956](https://github.com/dotnet/runtime/issues/43956) | Explicit modes are recommended. The issue author reports unintended globalization dependencies and proposes documentation/analyzer/default-experience changes; the issue remains open, without design consensus. | Choose a consistent explicit-policy story for new neoCLR search APIs. This is evidence of a usability problem, not evidence that culture-aware comparison is wrong or that the proposed .NET changes shipped. |
| [Swift String/Character guide](https://raw.githubusercontent.com/swiftlang/swift-book/main/TSPL.docc/LanguageGuide/StringsAndCharacters.md) | Grapheme characters, separate encoding/scalar views, positional indexes and canonical-equivalence equality. The probe's prefix within a grapheme is false, and composed/decomposed equality is true. | Learn the views/positions model. Do not claim we copied Swift semantics: neoCLR exact equality and ordinal prefix behavior intentionally differ. |
| [Rust str](https://doc.rust-lang.org/std/primitive.str.html) | Valid UTF-8, explicit bytes/scalars, checked byte-range access and named ASCII-insensitive comparison. | Make units and failed boundaries visible. Borrowed Rust lifetimes cannot simply be copied into a managed runtime. The probe exercises stable APIs, not every experimental API listed in current docs. |
| [Go strings](https://pkg.go.dev/strings) | EqualFold uses simple Unicode folding; Cut provides a useful delimiter-oriented operation. The local fold examples agree with neoCLR. | Consumer operations can avoid exposing indexes for common parsing tasks. Go is a comparison point, not a specification for neoCLR's validity, ownership or Unicode versions. |
| [Cysharp ZString](https://github.com/Cysharp/ZString) | Independent .NET library with UTF-8 and UTF-16 builders. Mutable value builders require disposal and careful copying; its README documents these obligations. | Text construction and UTF-8 output can be library concerns. Reuse the separation of text construction from output encoding, not pooled mutable-value lifetime hazards. Its performance claims are not neoCLR evidence. |

The .NET [Utf8String proposal #933](https://github.com/dotnet/runtime/issues/933)
is closed as not planned; it is not a shipped contract. Its separate byte/scalar
views remain an alternative worth studying, but adding a second public UTF-8 string
is not justified when neoCLR String already stores valid UTF-8. The closure alone
also does not prove a native UTF-8 design unsuitable for another runtime.

Use .NET 10 as the executed baseline, not as a frozen claim about future .NET.
The [.NET 11 Preview 7 notes](https://github.com/dotnet/core/blob/main/release-notes/11.0/preview/preview7/libraries.md#ordinal-casing-apis)
describe ordinal casing helpers aligned with ordinal comparison. Those are preview
additions, not part of our executed .NET 10 comparison. Any later casing proposal
must revisit that baseline before claiming a missing .NET capability.

### Keep, change or defer

| Area | Recommendation | Benefit, cost and compatibility |
| --- | --- | --- |
| String value and storage | **Keep** immutable, valid Unicode text with canonical UTF-8 storage. Preserve exact content through construction, concatenation, encoding and interning. Null remains distinct from empty; existing operations require non-null text. | Avoids malformed internal text and accidental data rewriting. Validation and foreign UTF-16 conversion still cost work. Nullable-storage enforcement is separate runtime work. |
| Char and Length | **Keep** grapheme Char and grapheme Length for familiar character-facing use. Document that a cluster is not a glyph, display width or fixed-size unit. | Reduces accidental splitting in ordinary display manipulation. Costs variable-size values and segmentation; no claim that all text processing should use graphemes. |
| Collection interfaces | **Keep for now** String's existing Sequence<char> compatibility, with documented scanning. **Change future dependence:** lower-level text APIs must consume explicit byte/scalar contracts, not generic character random access. | Avoids an immediate broad break. A later view/cursor design may replace this interface if a real consumer proves it misleading; no O(1) promise is added to Sequence. |
| Scalar boundary | **Change before broad System.Text:** add a validated scalar value contract and explicit scalar traversal/construction. Candidate spelling `UnicodeScalar` requires resolving the existing static helper name; exact spelling remains open. | Encoders/classifiers share one validity rule and cannot accidentally admit surrogates. This needs ordinary value/reference projection and migration from uint consumers; a new keyword/opcode is not presumed. |
| Search and slicing | **Change before adding position-returning APIs:** separate scalar-boundary text positions/ranges from grapheme ordinals and explicitly named byte ranges. Prefer search-to-slice round trips, or delimiter operations that avoid positions. | Prevents unit confusion and unrepresentable matches. Source-bound positions need ownership, default-value and retention rules; these remain prototype questions. Keep existing SliceUtf8 behavior. |
| Default equality | **Keep** exact equality and consistent hashing; normalization must be explicit. **Defer** canonical-equivalence comparers until a consumer chooses the normalization policy. | Preserves data/key identity and avoids hidden work. Visually equivalent input can remain different; callers need explicit normalization for suitable domains. Swift-style default equality is a deliberate rejected alternative for now. |
| Case-insensitive policy | **Change recommended before wider adoption:** name the present relation explicitly, provisionally `UnicodeSimpleFold`, across StringComparison/StringComparer, and avoid additional shortcuts until naming settles. | The current OrdinalIgnoreCase name invites assumptions our Kelvin/long-s/sharp-S behavior does not meet. UTF-8 does not force these differences. Renaming breaks development callers but leaves Preview 10 unaffected. Exact .NET behavior is an alternative if compatibility proves more valuable; no rename is performed here. |
| Casing/normalization | **Defer implementation**, but reserve String-to-String results and explicit policy. Do not promise Char-to-Char casing or replacement preserving counts. | Expanding/contextual transformations cannot generally fit one input/output Char. Display casing, key folding and normalization solve different problems. |
| Unicode version | **Change before new families:** adopt one documented target Unicode baseline across segmentation, classification and folding; keep host literal validation differences visible. | Reduces contradictory classification/folding results. Aligning to 17 requires updating and qualifying segmentation/classification; moving folding back to 16 is an alternative with changed key behavior. Neither upgrade is done by this review. |
| Errors | **Keep** typed malformed-input/range failures and terminal misuse/resource faults. **Change upcoming conversion results:** carry input byte position and distinguish incomplete input from invalid input. | Improves diagnostics and streaming recovery. Absolute versus chunk-relative position and partial-progress semantics must be specified before use. |
| Specialized text/zero-copy | **Defer** Utf8String/AsciiString, public borrowed spans, pooling and layout optimization. | Current valid String and explicit bytes cover the baseline. Avoid lifetime/aliasing machinery without a consumer; prototype retention and allocations before asserting benefits. |

Simple folding is not normalization, lowercasing or a security identifier policy.
The [Unicode mapping data](https://www.unicode.org/Public/17.0.0/ucd/CaseFolding.txt)
separates simple, full and tailored mappings. The existing [comparison evidence](../ordinal-text.md#explicit-comparison-modes-development)
remains valid; this review revisits how to present that relation, not its tested
implementation. A future ASCII-only policy belongs to protocol consumers, not a
silent shortcut for all Unicode comparisons.

### What to bring over from System.Text

This is the recommended portfolio, **not an implementation commitment**. Inventory
source: [.NET System.Text](https://learn.microsoft.com/en-us/dotnet/api/system.text?view=net-10.0).
Use .NET’s useful capabilities while selecting neoCLR contracts independently.
Some current documentation also exposes preview members; no newer type is assumed
part of the tested .NET 10 baseline merely because it appears on that page.

| Disposition | .NET capability | Proposed neoCLR scope and reason |
| --- | --- | --- |
| **Keep and develop** | UTF8Encoding’s conversion role | Existing Utf8.Encode/Decode already covers whole values. Add incremental progress and useful error positions for bounded file/protocol input; keep strict decoding as the default. A configurable UTF8Encoding class is not required just to reproduce the name. |
| **Bring over, adapted** | Rune | A validated scalar value and its classification/conversion operations, building on the existing UnicodeScalar helper. Needed by codecs and scanners independently of grapheme Char. [.NET Rune](https://learn.microsoft.com/en-us/dotnet/api/system.text.rune?view=net-10.0) is the semantic baseline; type name and migration remain to be decided. |
| **Bring over, bounded** | StringBuilder | Append text/Char/scalars, materialize String, clear/reuse and explicit limits for a report consumer. Defer arbitrary insertion, replacement, formatting overload families and mutable character indexing. A minimal append-only builder need not wait for every search-position decision. |
| **Bring over the role** | Encoder/Decoder | Stateful incremental conversion with explicit byte/scalar progress, final input, invalid data and destination exhaustion. Start with UTF-8; implement only the state needed by that codec, not an inheritance hierarchy first. |
| **Adapt when a second codec is needed** | Encoding | A small selectable codec contract for a real interop consumer. UTF-16 is the first candidate for .NET/native interchange, with endianness/BOM/surrogate-error policy. Strict ASCII validation/conversion is useful for machine protocols; it need not imply a full Encoding subclass family. [.NET Encoding](https://learn.microsoft.com/en-us/dotnet/api/system.text.encoding?view=net-10.0) is a role comparison, not a class-shape mandate. |
| **Bring over selectively after foundations** | Ascii helpers; normalization and casing capabilities | Explicit ASCII operations for protocol code; whole-String normalization/casing for an identified input/display scenario. Keep normalization opt-in and distinguish it from key folding. Do not wait for complete globalization to provide ASCII operations. |
| **Defer until repeated-format use exists** | CompositeFormat and specialized formatting integration | Begin with existing formatting/interpolation capabilities and the builder’s concrete report. Parsed format caching, providers and compiler-specific handlers need separate evidence; no automatic C# handler port to Raven. |
| **Leave out of the initial portfolio** | EncodingProvider, EncodingInfo registries and code-page packages; UTF-7 and broad UTF-32 support | No current consumer justifies discovery/registration or that coverage. Add a codec when a file/protocol requires it, not to complete an inventory. |
| **Do not copy the hierarchy** | Encoder/Decoder fallback and fallback-buffer classes | Keep typed errors; consider an explicit replacement policy when required. Do not silently adopt best-fit replacement or exception-based fallback machinery. Custom user callbacks are a later extension with its own safety/lifetime rules. |
| **Do not duplicate existing roles** | StringInfo/text-element helpers and separate UTF-8 string storage | String/Char already provide grapheme access; StringInfo is actually in System.Globalization. Reuse that foundation instead of adding a second façade solely for familiarity. A new Utf8String needs a guarantee the existing String cannot provide. |

Regular expressions, JSON and escaping-related subnamespaces are **separate
capability reviews**, not implicitly included by the phrase System.Text. Existing
neoCLR JSON stays in place. TextReader/TextWriter live in System.IO and remain
integration consumers, not namespace-copying targets. The initial useful portfolio
is therefore **scalar support, a small builder, better UTF-8 conversion and the
incremental encoder/decoder roles**; other entries need their own consumer.

### Crucial interfaces required by System.Text

Treat these as a **contract checklist**, not a promised type inventory. Namespaces
and one-for-one .NET overload parity are not the target. Existing System.IO readers
and writers, System.Data.Json and HTTP consumers participate without being renamed
or reopened by this review.

| Consumer family | String-side dependency to settle first | Boundary that must remain explicit |
| --- | --- | --- |
| StringBuilder and formatting | Append valid text/scalars/characters; immutable materialization; size limits and count units | Byte capacity and grapheme Length are different. Appending can resegment the boundary. A managed mutable reference builder is the first candidate; avoid copyable pooled-value ownership. |
| Encoding, Encoder and Decoder | Valid scalar input and valid-text output; byte traversal; progress/result contract | Incomplete scalar bytes, final input, invalid sequences, destination full, and consumed/produced units. Do not use grapheme counts as buffer capacity or UTF-16 char counts under a familiar name. |
| TextReader/TextWriter | Chunk ownership, EOF, newline policy, decoding errors and operation-result units | Existing Write returns bytes; future return types/docs must make that explicit. A valid text chunk need not end at a grapheme boundary in the combined stream. |
| Search, splitting and replacement | Exact versus folded matching; typed ranges; output resegmentation | Search results must select the original source text, even when transformed matching changes byte lengths. Empty matches and overlapping matches need rules. |
| Unicode classification, casing and normalization | Valid scalar traversal, target data version and String-producing transformations | Grapheme classification needs a separately chosen meaning; it is not “classify the first scalar”. Normalizing chunks separately is not generally equivalent to normalizing their concatenation. |
| Escaping, parsing, regular expressions and serialization | Stable position/error units, output assembly, scalar versus character semantics | Machine grammar and user-visible characters differ. This is dependency analysis, not a commitment to implement every subnamespace next or replace the existing JSON API. |

.NET's [Decoder.Convert](https://learn.microsoft.com/en-us/dotnet/api/system.text.decoder.convert?view=net-10.0)
separates progress, completion and final input; preserve those roles while replacing
its UTF-16 buffer assumptions. [Rust Utf8Error](https://doc.rust-lang.org/std/str/struct.Utf8Error.html)
shows useful error-position/incomplete-sequence information. These are precedents,
not a ready-made neoCLR signature. Prefer an ordinary Raven result/status type to
loosely coupled integer outputs if it keeps progress states coherent.

Unicode [UAX #29 revision 47](https://www.unicode.org/reports/tr29/tr29-47.html)
provides segmentation rules, not a fixed byte size for a character or an assurance
that chunk ends are character boundaries. Repeated extending scalars illustrate
why a grapheme stream needs its own buffering/limit/finalization policy. Keep that
layer separate from an incremental UTF-8 decoder. Similarly, a small slice retaining
an entire large source is an ownership cost even in a GC-managed runtime.

### Recommended next bounded step and acceptance

**First:** a String/System.Text boundary prototype and contract decision, rather
than StringBuilder or the entire Encoding hierarchy. Resolve scalar type identity,
traversal/construction, positions versus grapheme ordinals, progress/error units and
ownership using two tiny consumers: delimiter extraction with search-to-slice
round trips, and UTF-8 conversion split inside a multibyte scalar. This is a proposed
next task; this review contains no implementation of those interfaces.

The prototype should demonstrate that:

1. A search for the combining mark in `é` can select that exact source range without
   misusing a grapheme index; wrong-source and invalid-boundary positions are rejected.
2. Split UTF-8 bytes produce “need more input” before final input, and a typed error
   with a defined byte offset when truncated at final input. Destination exhaustion
   preserves unambiguous progress. Scalars, not graphemes, bound the conversion unit.
3. Reading chunks `e` then a combining mark preserves all scalars; grapheme iteration
   over the combined text still yields one Char. A grapheme stream is not inferred
   from a String chunk stream.
4. Materialized output survives source/builder mutation and disposal; slices/cursors
   have explicit source retention. Limits count named units and do not use a capacity
   hint as a security quota.
5. Existing String/Sequence consumers have a documented keep-or-migrate path; exact
   equality and hash coherence remain intact. Runtime enforcement, metadata and
   Raven projection agree on any new value/range type, including invalid defaults.

**Then:** qualify the chosen Unicode baseline and settle comparison naming before
expanding dependent APIs. Do not make unfinished regex, collation, general codec
registration or zero-copy work prerequisites for the selected portfolio. Next implement a minimal builder against those contracts,
followed by incremental conversion and reader/writer integration. StringBuilder's
familiar construction role is useful; [.NET's implementation and full surface](https://learn.microsoft.com/en-us/dotnet/api/system.text.stringbuilder?view=net-10.0)
are not prerequisites. Each family needs its own bounded consumer, not one giant
System.Text implementation commit.

The proposed improvements over .NET are clearer unit boundaries, fewer accidental
culture defaults, valid-text invariants and coherent typed failures. The present
probes support the identified contract mismatches, **not** a proven usability or
performance superiority. Developer feedback and the boundary consumers must test
that hypothesis. Full culture collation, normalization, Unicode security profiles,
regex design and broad encoding coverage remain open work, not hidden blockers for
every small API. No benchmark is necessary until a relevant performance claim or
regression is part of the selected task.
