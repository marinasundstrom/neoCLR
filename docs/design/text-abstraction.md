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

## Separate the text and encoding contracts

**Author clarification, 2026-09-27:** metadata encoding, runtime representation and
class-library encoding defaults are different concerns. A character is a logical
value in the system; it need not carry or expose an encoding. Callers choose an
encoding when crossing a byte boundary, or use the encoding specified by that API.
UTF-8 is the preferred default where appropriate, not the meaning of every text API.

| Concern | Contract owner | Consequence for the public API |
| --- | --- | --- |
| Source/artifact and metadata serialization | The source or artifact format and its reader/writer | Names and literals need a defined serialized representation. Its encoded units do not define guest Char, String.Length or a codec's public buffer units. Do not infer a metadata-format change from a library default. |
| Logical String and Char values | The language/platform text contract | String denotes Unicode text; today's Char denotes one extended grapheme cluster. A scalar denotes one Unicode scalar value. Neither logical unit is a UTF-8 byte or UTF-16 code unit. |
| Runtime storage | The runtime implementation | Current storage uses UTF-8. Layout, sharing and caching can evolve while preserving public text behavior. Storage does not require callers to encode/decode ordinary String values. |
| Text-to-byte and byte-to-text boundaries | The specific library API, protocol or caller's explicit choice | An API may accept an encoding, document a UTF-8 default, or mandate an encoding. These are different policies. String-to-String operations do not need an encoding parameter. |
| Foreign ABI / opaque data | The interop or resource boundary | Convert explicitly to required bytes/code units and define invalid-input handling. Preserve data outside Unicode text in a suitable boundary representation rather than silently replacing it. |
| Interpretation of text | The selected comparison, segmentation, normalization or casing policy | These operate on logical text and Unicode rules. UTF-8 storage or output defaults do not choose culture, case equivalence, normalization or user-visible character boundaries. |

Encoding-specific operations remain useful **explicit projections**. GetUtf8ByteCount,
SliceUtf8 and Utf8.Encode/Decode have UTF-8 in their contract, regardless of how a
future runtime stores the String. Likewise, the currently selected scalar ordinal
ordering is an observable comparison contract to preserve, not permission for an
internal-storage change to change sorting.

A high-level writer can accept String or Char and perform its documented default
encoding internally; users need not manually create bytes for every write. An
explicit codec can select another boundary encoding without changing the input
text's identity. Metadata readers independently decode the representation required
by their artifact format. None of these operations means that Char itself is
“a UTF-8 character”.

This refines the review below: **encoding-independent text input is appropriate for
builders and codecs**. The warning concerns confusing grapheme counts with byte or
scalar progress in low-level conversion buffers. It does not mean every text API
must expose scalar arrays, UTF-8 storage or encoding choices. A scalar-oriented core
can support a simpler String/Char-facing API.

The word “Unicode character” alone does not resolve the scalar-versus-grapheme
choice. This clarification preserves the currently selected grapheme Char; it does
not silently change it to a scalar, code point or encoded unit.

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

**Review conclusion:** keep immutable Unicode text, grapheme Char and familiar
String operations, with UTF-8 as the current storage choice. Text-facing System.Text
APIs may consume String/Char; their lower-level conversion contracts must distinguish
byte/scalar progress from grapheme counts. Sequence<char> is not a universal encoded
buffer contract. Metadata encoding and boundary defaults remain independent.
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

The major problem would be **conflating these layers or hiding their differences**, for example calling an
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

### Foundation scope and Swift direction — author clarification

The author confirms that Char should hold a user-perceived character, not an
encoding unit, and identifies Swift as the closer direction for the text API.
Use Swift as the primary comparison for character/scalar/encoding views and text
navigation; retain .NET as an ergonomic and capability comparison, not a required
surface shape. This refines the earlier .NET-first framing for this theme.
A grapheme approximates a perceived character; it is not necessarily one rendered
glyph or one display cell.

The immediate goal is **encoding/decoding foundations and possibly a small
StringBuilder**, sufficient to build later APIs. It is not the full System.Text
API. Public scalar values, general range machinery, casing, normalization and
formatting families must not become prerequisites without a concrete dependency.
Existing Utf8 conversion and the chunk experiment already work without a new
public scalar type. Keep such types experimental until a consumer needs them.

[Swift's String/Character guide](https://raw.githubusercontent.com/swiftlang/swift-book/main/TSPL.docc/LanguageGuide/StringsAndCharacters.md)
provides the reference for character collections, separate Unicode representations,
append operations and value semantics. A neoCLR builder would supply controlled
mutable accumulation beside the existing immutable String. Compare that with
Swift-style string construction before adding a separate public type; Swift's
copy-on-write implementation is not automatically a neoCLR requirement.

Swift inspiration does not silently adopt canonical-equivalence equality, mutable
String value semantics or every indexing rule. Existing exact equality/hashing and
String/Sequence consumers remain unchanged. Those divergences need explicit
review before any future migration. The detailed inventory below is a menu of
later capabilities, not the scope of this foundation milestone.

The author's supplied ChatGPT proposal is a useful comparison, not an adopted
specification. Encoding-neutral text, explicit views, source-bound positions and
strict decoding fit this direction. Removing Char, removing Length and demoting
graphemes are alternative design choices, not Unicode necessities. Keep them open
for reasoned review rather than inferring a migration from the pasted text. Canonical
equality does not require storage normalization: Swift's canonical comparisons do
not by themselves imply rewriting encoded data. A source-bound position also needs
a declared boundary view (grapheme versus scalar), source validation and retention
rules; making its offset opaque does not settle those questions.

### Open naming discussion: Text versus String — 2026-09-27

**Input, not a rename decision.** The author adds a proposal favoring `Text` for an
encoding-independent Unicode value, with explicit scalar, grapheme and encoding
views. Its suggested vocabulary is Text / Rune / Bytes / Encoding, and it considers
moving encoding APIs out of System.Text so that System.Text could name the value
type. The claimed readability benefit is a hypothesis; no usability comparison or
compiler migration has been performed.

`Text` foregrounds meaning rather than storage, and pairs naturally with ReadText,
WriteText and explicit encoded views. However, Unicode text also includes machine
syntax, control characters and non-display data; “human-readable” must not become a
validation requirement. `Bytes` likewise needs an ownership/mutability contract if
introduced as a type, not merely a nicer spelling for byte arrays. Neither Rune
nor Bytes is approved by this vocabulary discussion.

The proposal's premise that the value has no default element type differs from
neoCLR's current `String : Sequence<char>` contract and the author's grapheme Char
direction. We must evaluate those semantics independently of spelling. Retaining
a character view or default character iteration does not make text encoding-bound.
Swift itself retains the name String alongside grapheme Character and explicit
Unicode views. Conversely, renaming to Text would not by itself remove indexing,
choose count units, settle equality or change any complexity guarantee.

Primary comparisons (reviewed 2026-09-27): [C# string](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/reference-types#the-string-type)
is a language alias for System.String; this illustrates the distinction between
source spelling and library type identity. [Swift String/Character](https://raw.githubusercontent.com/swiftlang/swift-book/main/TSPL.docc/LanguageGuide/StringsAndCharacters.md)
illustrates that familiar String naming can coexist with an encoding-independent
character model. Neither precedent chooses neoCLR's vocabulary for us.

| Alternative | Potential benefit | Cost and question to resolve |
| --- | --- | --- |
| Keep String/string; describe it as Unicode text | Familiar .NET and Swift vocabulary, unchanged type identity and consumers | Explicit views and documentation must counter code-unit/indexing assumptions. |
| Explore Text as a source alias or preferred spelling for the same value | Test clearer vocabulary without introducing a second text representation | Requires Raven name-resolution/tooling investigation; not known to work as a built-in alias today. Two spellings can confuse signatures, diagnostics and documentation. |
| Rename the public core type to Text | One consistent text-oriented vocabulary across APIs | Coordinate literal/interpolation typing, compiler projection, bridge bindings, metadata identity, introspection, tools, samples and references. Decide whether a legacy alias is useful; do not assume a global textual replacement is sufficient. |
| Add a distinct Text wrapper beside String | Could express an additional invariant if a real consumer needs one | Without such an invariant it duplicates the same abstraction, adds conversions and complicates equality/overloads. Not the current recommendation. |

The migration cost here is more than programmer habit. For example, the existing
[application bridge](../experiments/raven-target/OpaqueLibrary.cs) explicitly matches
System.String and its opaque representation. Preview status makes breaking changes
possible, but does not remove this coordination work. These are engineering costs,
not a requirement to preserve .NET compatibility at the expense of a better contract.

Namespace placement is a separate choice. Using System.Text as a type would require
reviewing the existing System.Text namespace, imports, documentation paths and
compiler resolution. System.Encoding and System.Globalization are candidate
organizational names, not selected destinations. Keeping a compact System.Text
namespace does not commit us to implementing .NET's entire namespace.

**Disposition:** keep Text as an open naming candidate and current String/Char
behavior intact while the foundation work proceeds. If pursued, compare a tiny
reader/decoder/construction example under both vocabularies, with names held separate
from changes to iteration/indexing. Then test the selected spelling through Raven
literals, interpolation, generated metadata, bridge import and introspection. Do not
make a broad rename or namespace reorganization a prerequisite for basic encoding.

### High-level text API intent — author clarification

The author clarifies the purpose of the naming discussion: a high-level API for
handling text, with names that challenge developers' inherited assumptions, while
keeping the design small. C#-style string/char concepts could remain in lower-level
APIs. This is stronger than a cosmetic rename proposal, but does not select exact
type names, authorize a second representation or decide the fate of current APIs.

Evaluate `Text` as the application-facing abstraction: ordinary text construction,
character access where useful, search and extraction should not require callers to
reason about an encoding. Scalar and encoded operations should be explicit when a
parser, protocol or interoperability boundary needs them. Naming is intended to
signal these contracts; whether it actually helps developers remains to be tested
with small consumer examples. Do not add parallel type families merely to make a
layer diagram look complete.

Compared with .NET's familiar String/Char surface, the proposed benefit is making
text operations and representation-specific operations easier to distinguish. The
cost is new vocabulary and potentially conversions or overlapping APIs if both
surfaces are independently exposed. Prefer one text value with deliberate views
or projections unless a distinct invariant justifies another type. A high-level
abstraction need not add a wrapper allocation or duplicate text storage.

The lower-level suggestion needs a precise unit: C# char is a UTF-16 code unit;
neoCLR's current Char is a grapheme. Neither naming nor layering makes those
interchangeable, and UTF-8 bytes are another distinct representation. If a C#-like
UTF-16 surface is useful for interop, name its unit explicitly and decide how
unpaired surrogates cross the valid-text boundary. Do not silently repurpose the
existing Char or add a full compatibility layer without a consumer.

**Scope guard:** begin with a coherent vocabulary for text values, basic
encoding/decoding and minimal construction. Views or search ranges enter only as
needed by concrete consumers. Text/Rune/Bytes, a renamed namespace hierarchy and
separate high/low type families are options, not a required package. The earlier
String/Char contract remains implemented behavior; the author has clarified the
design objective, not approved an exact replacement surface.

### Consumer API sketch: identical behavior, two vocabularies

**Experimental, 2026-09-27.** The author asks for the next safe step, then directs
continuation of a small sketch tested against decoding, construction and extraction.
The [executable consumer](../experiments/text-boundaries/Consumer.rvn) uses existing
String plus application-only helpers. The Text spelling below is a design sketch,
not a compiled alias, new runtime type or claim of measured developer preference.
Both columns intentionally describe the same behavior. Existing library String is
real; the additional builders, decoder surface and extraction operation are proposals.

| Operation | Familiar String vocabulary | Text-oriented vocabulary | Common contract |
| --- | --- | --- | --- |
| Incremental decode | UTF-8 decoder produces string chunks | UTF-8 decoder produces Text chunks | Accept byte input with final-input flag and named output budget. Return text, consumed bytes and status; failures retain progress and byte error location. |
| Construction | StringBuilder: Append(string), Build → string, Clear | TextBuilder: Append(Text), Build → Text, Clear | Append valid text; Build preserves an immutable snapshot. Reuse cannot mutate prior results. Appends can combine into one grapheme. |
| Delimiter extraction | AfterOrdinal(string, delimiter) → Option<string> | AfterOrdinal(Text, delimiter) → Option<Text> | First exact delimiter match; Some(empty) differs from None. No caller byte arithmetic. Prototype empty delimiter matches the start; a public API must document that choice. |
| Explicit representation | UTF-8 encode(string) → bytes | UTF-8 encode(Text) → bytes | Encoding conversion is explicit. Renaming the input does not change validity, ownership or byte count. |

These are signature descriptions, not Raven declarations. `TextAccumulator` in the
fixture models builder semantics with existing concatenation. It does not claim
amortized append performance or prescribe the eventual public implementation.

The tested consumer assembles `name=` then `e`, saves a snapshot, appends a combining
acute accent, and extracts the value after `=`. The saved snapshot stays `name=e`;
the extracted value is one grapheme `é`. Clear/reuse leaves the built report intact.
A second path decodes bytes for `e` plus an incomplete combining scalar, then the
remaining byte. Joining the returned chunks yields the same text. Codecs need not
buffer a whole grapheme to return useful valid text.

The extraction fixture also searches `e` inside `é` and returns the remaining
combining mark. That is intentional **ordinal** behavior, not a recommended implicit
character-search policy. Swift-like character-boundary search would need different
acceptance rules. Naming the value Text cannot settle that choice; retain an explicit
ordinal name for this experiment and do not ship a generic Find/After default yet.

**Assessment:** Text foregrounds application intent, while String preserves familiar
vocabulary. The examples do not demonstrate that either spelling improves usability.
They do demonstrate that application code can build and extract text without exposing
byte indexes, and that lower-level decoding still needs byte progress. No public
Rune/Bytes wrapper or opaque position type is required by these three consumers.
One text representation is sufficient for this sketch.

**Smallest foundation recommendation:** retain current whole-value UTF-8 conversion;
add one stateful decoder role with explicit progress/finalization for a chunked reader.
Keep output immutable and make strict errors visible. Decide its output-budget unit,
retry behavior, bounds and allocation policy before promoting the experimental
signature. Then implement minimal construction only if the report consumer warrants
it. Keep search policy and the String/Text naming decision open; neither blocks the
codec contract. Reuse the earlier .NET Decoder.Convert and Swift text comparisons;
this consumer is behavioral evidence, not performance or usability evidence.

### Internal reader integration — development

StreamReader.ReadToEnd now consumes the bounded UTF-8 conversion role internally.
It reads at most 256 bytes, retries conversion of an unconsumed suffix before reading
again, and carries at most three incomplete bytes between conversions. A complete
scalar is never split across decoder output chunks, but graphemes may be. Complete
output is validated by the existing strict Utf8 codec in bounded groups, not decoded
one scalar at a time. The reader still accumulates one String and returns no prefix
on failure. Its existing 65536-byte limit, BOM preservation and input ownership stay
in place. ReadLine is unchanged.

Malformed input can fail before EOF now, changing failure ordering and cursor
advancement relative to Preview 10; [the public reader contract](../../api-docs/streams.md#text-readers)
records those details. No public decoder types or new representation are introduced.
The internal helper exposes consumed bytes and output exhaustion to its one caller;
its Result uses the existing InvalidUtf8Error. Rich error offsets and prefix recovery
from the prototype remain candidates for a later public decoder, not shipped behavior.

The application prototype's rich result carrier is not directly admitted as a private
library dependency by the current bridge (readonly storage/private union restrictions).
Keep the private helper minimal rather than changing compiler admission or making
experimental carrier types public to bypass that restriction. This limits reuse;
it does not select the eventual public contract. The helper is synchronous and
single-consumer, has no callbacks, and relies on its caller's checked ranges.

### Later Encoder abstraction — author direction

The author explicitly requests an Encoder abstraction later, adapted to neoCLR's
string/text direction rather than copied from .NET. This is a future capability,
not authorization to implement the full Encoding/Encoder/Decoder class hierarchy.
.NET's encoder role is the comparison baseline; its UTF-16 char-buffer inputs are
not neoCLR's public text unit.

A candidate Encoder consumes valid text (currently String) and produces explicitly
encoded bytes. Public spelling may later be Text without changing that role.
Incremental calls need final-input, output-exhaustion, ownership and progress rules.
Do not report consumed graphemes if conversion may stop inside a multi-scalar Char:
choose whole-input acceptance with bounded owned carry, or a resumable position with
a precisely named boundary. The choice remains open. Encoding selection, BOM policy
and unmappable text for restricted encodings must be explicit; UTF-8's valid-text
path does not need a UTF-16 surrogate-repair policy.

The first internal reader decoder is evidence for bounded conversion, not a base
class or a public Encoder signature. Its fixed UTF-8 buffers and progress properties
are implementation details. Avoid shaping future public APIs around their incidental
limits. Introduce the abstraction with one actual writer/encoding consumer after
the corresponding contracts are settled; no new character representation or
namespace reorganization is implied.

### Shared encoding APIs (development)

`System.Text.Encoding` is the reusable conversion policy: `Encode(string)` returns
`Result<Sequence<byte>, EncodingError>` and `CreateDecoder()` creates independent
state. `Encodings.Utf8` and `Encodings.Ascii` supply the first implementations.
`Decoder.Decode(byte[], offset, count, final)` returns valid text. Success accepts
all offered bytes; an incomplete UTF-8 scalar is copied into decoder-owned carry
and may produce empty text. The caller may immediately reuse its array. Invalid
ranges and counts above 65536 fail before mutation and can be retried. Malformed
input or finalization ends the decoder; later calls return Finished. Built-ins
never replace invalid/unrepresentable content. UTF-8 preserves U+FEFF; no codec
sniffing, BOM insertion or automatic BOM removal occurs.

`StreamReader(input, encoding[, leaveOpen])` and
`StreamWriter(output, encoding[, leaveOpen])` select this shared interface. Existing
constructors keep UTF-8 defaults. Each reader owns its decoder; the encoding itself
can be reused. ReadLine recognizes LF and CRLF in decoded text and retains decoded
suffixes supplied by a codec. Built-in codecs do not read ahead across lines.
WriteLine encodes text plus LF together. Writer results count actual encoded bytes;
strict built-in preflight finishes before output callbacks; the writer now drains
a bounded byte buffer before requesting further encoded bytes. Partial writes
are retried. Conversion errors return InvalidEncoding, while malformed UTF-8 reads
retain InvalidUtf8. Stream failures retain their existing cases.

`maxUtf8Bytes` remains the returned text's UTF-8 size, with a separate source-read
ceiling of the same number (plus one overflow byte for ReadToEnd, or up to two line
terminator bytes for ReadLine). These measures coincide for valid built-in input;
custom codecs must respect both. Future expanding/stateful codecs need explicit
source/output quota design. Errors return no partial text and may advance input.
Writer input and encoded output are each limited to 65536 bytes; WriteLine includes
LF in both limits. Runtime allocation/instruction budgets still apply, so the API
ceiling is not an allocation guarantee. Conversion does not add async behavior.

Compared with .NET Encoding/Decoder, the policy/factory roles are familiar but the
conversion boundary returns valid Unicode text instead of a UTF-16 char buffer.
This avoids exposing storage units through character APIs, at the cost of owned
result allocations and no destination-capacity/progress API. Strict ASCII avoids
silent replacement at the cost of handling conversion failures. Whole-value Encode remains available alongside the development incremental
Encoder described below. Flush forwards stream flushing; explicit Finish finalizes
a StreamWriter encoder. HTTP can reuse the
conversion policy later, but charset selection, protocol validation and framing
remain HTTP concerns. This work does not reopen the completed HTTP POC.

Focused consumers: [EncodingMain.rvn](../experiments/text-boundaries/EncodingMain.rvn)
and [verification](../experiments/text-boundaries/verify_encoding.py), plus the
existing reader regression fixtures. No website build or full suite is required.

### Selected stream encoding with UTF-8 default — author requirement

The author clarifies that **StreamReader and StreamWriter must accept a selected
encoding, with UTF-8 as the default**. This is an intended capability, not just a
possible extra codec. At the time of this requirement, production constructors were UTF-8-only; the private
reader integration implements part of the default path and does not complete this
requirement. The next foundation step must establish that selection contract before
presenting either stream adapter as complete.

.NET's reader/writer encoding selection is the ergonomic baseline. Adapt the role:
one reusable encoding choice can create separate per-reader decoding and per-writer
encoding state. Do not share mutable conversion state across streams. “Encoder” in
this discussion denotes the requested encoding abstraction; exact Encoding/Encoder/
Decoder type names and whether a combined contract is preferable remain open.
A read adapter needs byte-to-text conversion; a write adapter needs text-to-byte
conversion. The author has not requested one specific class hierarchy.

The default constructors should continue to select UTF-8. Explicit selection must
flow into conversion rather than be accepted and ignored; validate the design with
a second codec or a distinct test implementation. Reader/writer ownership and close
behavior must remain clear. Finalizing conversion at EOF/close and ordinary Flush
are separate concerns for stateful codecs; specify those semantics before exposing
custom implementations. Keep BOM handling explicit rather than inferring detection
from encoding selection.

Existing `ReadToEnd(maxUtf8Bytes)` and writer byte-count results need deliberate
unit review. For another encoding, source-byte quotas, UTF-8 size of internal text
and encoded bytes written are different quantities. Preserve or explicitly migrate
the existing contract; never reinterpret a parameter silently. Valid text remains
the public input/output value, so choosing UTF-16 must not redefine Char as a code
unit. Strict invalid/unrepresentable-data behavior also belongs in the selected
encoding contract.

Keep this bounded: selection, independent conversion state, precise progress/limits,
and one reader plus one writer consumer. General code-page registries, provider
hierarchies, automatic detection and a full System.Text clone are not implied.

### Shared encoding selection probe — UTF-8 and strict ASCII

**Author choice, 2026-09-27:** start with UTF-8 and strict ASCII, rather than the
assistant's suggested UTF-16 little-endian. The author stresses that this is shared
infrastructure for many APIs, including HTTP. The assistant initially considered
built-in modes, then chose to test an interface rather than prescribe a closed enum
that consumers would switch over. No enum or public selection API was implemented.

[Selection.rvn](../experiments/text-boundaries/Selection.rvn) is an executable,
application-only contract probe. It provides one encoding interface for whole-text
Encode and CreateDecoder; each factory call returns independent mutable decoding
state. The same selected-reader and selected-writer implementations use either codec,
with UTF-8 default constructors. They do not inspect codec names or contain encoding
switches. Text is still the existing valid String; no UTF-16 char array is introduced.
Exact public names and compiler reference projection remain unselected.

The decoder variant accepts the complete offered byte range on success and owns any
incomplete UTF-8 carry. That is an explicit alternative to exposing low-level output
buffer exhaustion: this high-level operation returns an owned String, including empty
text when more input is needed. Final input succeeds once or returns a typed error;
subsequent use returns Finished. Range/limit errors are retryable, malformed input is
terminal. Failure does not expose prefix recovery or a consumed count. These simpler
semantics do not replace the separate bounded-buffer experiment's contract.

ASCII decoding rejects bytes above 127; encoding rejects unrepresentable text before
any output call. NUL and DEL are valid ASCII values; ASCII conversion is not a
printability or protocol-grammar validator. In contrast, default
[.NET ASCIIEncoding](https://learn.microsoft.com/en-us/dotnet/api/system.text.asciiencoding?view=net-10.0)
uses replacement for unrepresentable characters, with configurable fallback available.
Strict typed failure avoids silent data loss at the cost of explicit error handling;
we do not need to copy that fallback class hierarchy. Source reviewed 2026-09-27.

Writer results count actual encoded bytes, and short writes are retried. Encoded data
is snapshotted before invoking output callbacks. WriteLine encodes text plus LF through
the selected codec; Flush forwards to the stream and does not finalize a codec. The
prototype uses stateless whole-value encoding, so it does **not** prove future stateful
Encoder flush/close behavior. One selected encoding value can be reused; decoder
state is never shared between readers.

The fixture uses 256 offered bytes per decoder call and 4096 total/source/text bytes,
plus a four-byte reader/writer scratch buffer. These are experiment limits, not public
platform recommendations. For valid UTF-8 and ASCII, encoded bytes and UTF-8 text
bytes have equal lengths, so preserving maxUtf8Bytes is demonstrable. This equality
must not become a generic encoding-interface assumption: other encodings need
separate source-byte and decoded-text quotas. The probe checks both counts but uses
the same numeric limit for each; it does not establish a policy for expanding codecs.

Two focused runs prove default/explicit selection, strict ASCII rejection in both
directions, independent UTF-8 carry after caller mutation, finalization, retryable
invalid ranges, byte limits, empty EOF, partial writes, encoded-byte counts, newline
encoding and ownership close. It omits ReadLine, leaveOpen overloads, destination
fault injection and stateful encoders. These are explicit integration tasks before
promoting the contract into production StreamReader/StreamWriter; existing constructors
were UTF-8-only at the probe stage. Public integration is now recorded above.
The probe itself did not change .NET/Raven references or runtime APIs.

**HTTP dependency:** keep conversion in the shared text foundation. A future HTTP
text-body adapter selects an encoding according to media type/charset policy and
keeps byte-body access available; codec defaults do not silently become protocol
rules. [RFC 9110, Content-Type](https://www.rfc-editor.org/rfc/rfc9110.html#name-content-type)
is the primary basis for representation metadata (reviewed 2026-09-27). Transfer
framing, compression, header validation and content-length accounting stay in their
own HTTP layers. ASCII includes control bytes, so choosing ASCII cannot establish
safe field syntax. This records a dependency, not new HTTP implementation scope.

**Probe follow-up (now implemented above):** promote the shared encoding/decoder roles with matching
reference metadata and both stream adapters, retaining UTF-8 defaults and adding
strict ASCII. Cover line reads and ownership compatibility, choose typed conversion
errors without silently changing existing cases, and make quotas explicit. Add a
stateful Encoder when its writer consumer requires it; avoid a registry or general
inheritance hierarchy solely for .NET surface parity.

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
| **Implemented in development** | Encoding | A small selectable codec contract now supports UTF-8 and author-selected strict ASCII. UTF-16 remains a possible later .NET/native interchange codec, needing endianness/BOM/surrogate-error policy. Strict ASCII validation/conversion is useful for machine protocols; it need not imply a full Encoding subclass family. [.NET Encoding](https://learn.microsoft.com/en-us/dotnet/api/system.text.encoding?view=net-10.0) is a role comparison, not a class-shape mandate. |
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

**Current experiment:** [text boundaries](../experiments/text-boundaries/README.md)
uses delimiter extraction and split UTF-8 input to examine units, ownership and
conversion progress. These are application types, not public System APIs. Scalar
and range experiments inform future views; they do not mandate introducing them
before basic encoding or text construction.

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

**Next bounded implementation recommendation:** the internal UTF-8 reader integration
is complete and shared Encoding/Decoder selection is integrated in both stream
adapters with UTF-8 defaults and strict ASCII. Focused integration covers
line reads, ownership and error/limit semantics. A builder remains a companion
candidate; no complete search API, new scalar public type or String/Text rename is
required first. Unicode-version alignment and comparison naming remain separate.

The proposed improvements over .NET are clearer unit boundaries, fewer accidental
culture defaults, valid-text invariants and coherent typed failures. The present
probes support the identified contract mismatches, **not** a proven usability or
performance superiority. Developer feedback and the boundary consumers must test
that hypothesis. Full culture collation, normalization, Unicode security profiles,
regex design and broad encoding coverage remain open work, not hidden blockers for
every small API. No benchmark is necessary until a relevant performance claim or
regression is part of the selected task.

### Bounded report construction evaluation — 2026-09-27

**Scenario and hypothesis.** A report consumer appends a name, decoded text and
status lines, then retains the result while reusing the accumulator. Test whether
a minimal managed builder provides useful explicit quotas and snapshot semantics,
and whether deferred balanced concatenation is worth promoting over existing
String.Concat. This is an evaluation of the next roadmap candidate, not a new
public type or an author decision on naming.

**Comparison (primary sources retrieved 2026-09-27).** .NET's
[StringBuilder](https://learn.microsoft.com/en-us/dotnet/api/system.text.stringbuilder?view=net-10.0)
provides mutable construction and immutable ToString snapshots. Its length/capacity
units follow .NET char, and documented MaxCapacity behavior is not a general hard
input quota. Its guidance also recommends measuring whether a builder helps.
Swift's [String](https://developer.apple.com/documentation/swift/string) offers
value-semantic mutation backed by copy-on-write storage and amortized append growth.
Neither API spelling grants those implementation properties to neoCLR. The
[.NET semantic fixture](../experiments/text-boundaries/BuilderBaseline.cs) confirms
snapshot/reuse behavior and distinct UTF-16, UTF-8 and grapheme counts on the same
report; there is no cross-runtime timing comparison.

**Prototype contract.** [ReportTextBuilder](../experiments/text-boundaries/Builder.rvn)
is an ordinary application reference type with Append(string), AppendLine(string),
Build(), Clear(), Utf8ByteCount and MaxUtf8Bytes. The caller explicitly selects a
0–65536 UTF-8 byte quota. Unsupported construction limits fault; append overflow is
a typed LimitExceeded and leaves existing content unchanged. AppendLine preflights
text plus LF as one operation. Host resource faults are not transactional guarantees.
No mutable character indexing, capacity property, formatting overload family or
implicit encoding conversion is introduced. Scalar and Char helpers can be added
only with a consumer; the fixture accepts valid strings from the existing Decoder.

Build returns immutable text, caches repeated materialization and compacts retained
fragments. Appending a combining mark can merge with an earlier grapheme; byte counts
are additive while character counts are not. Clear releases fragments and cached
text while preserving the quota. Empty appends add no retained fragment. The managed
implementation keeps immutable parts and combines adjacent pairs in balanced rounds.
For n equal-sized pieces, this changes concatenation's summed copied length from
quadratic growth to O(total bytes × log n), but adds guest collection operations,
reference objects and intermediate strings. That is algorithm analysis, not measured
native allocation evidence. Runtime source at `0a14989b`,
[src/native.rs](../../src/native.rs), shows StringConcat allocating a fresh payload
and copying both inputs; immutable payload sharing alone does not make append cheap.

**Evidence and decision.** The focused contracts pass for empty/zero/exact bounds,
atomic line rejection, odd fragment counts/order, repeated Build, retained snapshots,
clear/reuse, supplementary text and split decoded combining sequences. The diagnostic
checks identical final bytes for both strategies at 8 and 1024 pieces, with three
fresh invocations each and alternating order. At 1024 × 16 bytes, ordinary bounded
concatenation took **51–58 ms**, versus **561–574 ms** for the managed builder on
macOS arm64. At eight pieces the ranges were 0–2 and 5–6 ms. Millisecond resolution
limits interpretation of the small case. Measurements include guest construction,
checks and captured output, but exclude load/verification; they are not a warmed
throughput study. Managed GC counts do not measure native String payload bytes.
See [raw evidence](../experiments/text-boundaries/builder-validation.json).

The explicit quota and snapshot contract is useful, but this implementation is not
justified as the public builder. Keep ordinary concatenation for the tested report;
retain the bounded construction contract as a candidate. Do not optimize this probe
or add a new native buffer/ownership facility solely to obtain a favorable result.
A future consumer that needs bulk construction can justify evaluating a one-pass
join or native append buffer, including snapshot lifetimes and resource enforcement.
That would be separate runtime work. A thin bounded concatenation wrapper is also
an option if callers need quotas without claiming faster assembly.

**Next bounded candidate.** Return to the author-requested incremental Encoder role:
first settle text-input progress, output-byte capacity and completion semantics with
a writer consumer. This remains a follow-up, not authorization for a .NET fallback
hierarchy or new HTTP feature. The shared Encoding/Decoder APIs remain implemented;
public builder promotion and String/Text naming remain deferred.

### Encoder progress and writer evaluation — 2026-09-27

**Problem.** The current Encoding.Encode returns all encoded bytes at once. A
writer-facing Encoder needs bounded output, observable finalization and a clear
answer to what input it has accepted. String/Char must not acquire UTF-16 buffer
semantics to provide this. The [application probe](../experiments/text-boundaries/Encoder.rvn)
now exercises a candidate with a real OutputStream consumer; it is not yet a public
System.Text API or a production StreamWriter change.

**.NET baseline.** [Encoder.Convert](https://learn.microsoft.com/en-us/dotnet/api/system.text.encoder.convert?view=net-10.0)
(primary documentation retrieved 2026-09-27) separates input consumed, output written
and completion, with final-input signaling and retained state. Its source is a char
buffer; those units cannot be relabeled as neoCLR graphemes. The
[.NET semantic fixture](../experiments/text-boundaries/EncoderBaseline.cs) consumes
five UTF-16 units to produce nine UTF-8 bytes for `é😀é`. A one-byte destination
cannot hold the first encoded scalar and is rejected by the tested .NET 11 encoder.
This is a behavioral comparison, not a performance comparison or a criticism of
.NET's different contract.

**Candidate: whole-text acceptance followed by byte draining.**

- `Accept(text, final)` accepts the complete immutable String on success. It retains
  that value until drained; the caller may reassign its variable. There is no
  character-consumption count or caller-managed source offset. Only one accepted
  chunk can be pending. A nonfinal pending chunk rejects another Accept with Busy.
- `Drain(output, offset, count)` returns one immutable step containing BytesWritten
  and a standard union state: NeedsOutput, Ready or Finished. Writes affect only
  that many bytes in the offered range. A positive capacity makes progress for
  pending built-in output; zero capacity can return NeedsOutput with zero bytes.
- Ready means the accepted chunk is fully drained and another may be accepted.
  Finished means final input **and every final byte** have drained. An accepted
  final chunk rejects further input even while its bytes remain pending. Empty
  final input is meaningful; draining it may finish immediately or produce bytes.
- Byte fragments need not be independently valid text: even a one-byte destination
  works by splitting encoded scalars across drains. This simplifies byte sinks but
  requires downstream text consumers to decode incrementally rather than treat
  every output fragment as a complete UTF-8 string.

The prototype uses a 4096-byte input quota and a 4096-byte maximum drain request;
these are experimental bounds, not a new platform limit. Invalid ranges/capacities
fail before state or destination mutation. Unsupported text and oversized input
fail before acceptance and are retryable. UTF-8 accepts valid String directly;
strict ASCII preflights the entire text before accepting it, including non-ASCII
content beyond the first internal chunk. No replacement fallback is added.

Internally the provider retains the immutable source and a pending encoded chunk
of at most 32 bytes, using scalar-safe String slices for conversion. It does not allocate an encoded
copy of the entire input. This is bounded encoded buffering, **not constant total
memory**: the source and destination remain owned values, conversions allocate,
and normal host budgets still apply. ASCII performs a validation pass followed by
conversion. No throughput or allocation reduction has been benchmarked or claimed.
The provider deliberately supports only UTF-8/ASCII. Arbitrary Encoding.Encode
implementations must not be assumed composable across chunks or stateful sessions.

**Writer/lifetime evidence.** The consumer pumps into a four-byte buffer, retries
one-byte short writes, and drains again only after all returned bytes have been
written. Counts represent actual bytes sent. An output error can follow partial
output; the writer then rejects further Write/Finish operations because it cannot
safely replay the accepted text. Expected ASCII preflight failure leaves it reusable.

Finish explicitly accepts final empty input and drains it; repeated successful
Finish calls emit nothing. Flush only invokes OutputStream.Flush. Close retains
its existing ownership role and does not silently finalize conversion. A synthetic
trailer provider proves that final output may need several drains and may fail;
it is a protocol test double, not a new supported character encoding. Callers must
Finish, then Flush if required, and observe both results before Close. Close/failed
writers do not promise immediate collection of retained encoder state.

**Validation.** Three focused executable runs pass for owned input, Busy/Finished,
zero and one-byte capacities, untouched destination sentinels, invalid range retry,
strict rejection after a valid prefix, source limits, scalar boundaries at the
internal chunk edge, cross-input grapheme composition, short writes, leaveOpen,
final bytes, idempotent Finish, Flush/Close separation and final-output errors.
The .NET semantic comparison passes separately. See [evidence](../experiments/text-boundaries/encoder-validation.json).
No benchmark, full suite, website build, compiler change or public API addition.

**Recommendation and integration boundary.** Prefer this acceptance/drain role to
exposing grapheme progress or raw UTF-8 source offsets. Its costs are retained input,
two-stage usage and byte fragments that can split a scalar. The alternative is a
resumable source-bound position with stricter lifetime/identity machinery; that is
unnecessary for this bounded writer. Keep names and prototype bounds provisional.

The next bounded implementation should add the public Encoder factory alongside
Encoding.CreateDecoder, promote a coherent progress/error result, and connect it
to StreamWriter with explicit Finish. Adding Encoding.CreateEncoder requires a
migration for custom development Encoding implementations; do not silently wrap any
arbitrary whole-value codec as an incremental one. Initially Finish can live on
StreamWriter; extending the general TextWriter interface is separate work for a
consumer needing that capability. Preserve UTF-8 defaults, strict ASCII preflight,
WriteLine atomic preflight, encoded-byte counts and ownership, refresh matching
reference/API artifacts, and reuse these focused consumers. HTTP charset policy,
other encodings and public builder promotion remain outside this slice.

### Public Encoder integration (development) — 2026-09-27

The acceptance/drain contract is now implemented in System.Text. Encoding adds
CreateEncoder; Encoder exposes Accept/Drain; EncoderProgress has read-only
BytesWritten/State properties; EncoderState is a standard Raven union. EncodingError
adds Busy without reordering its existing cases. Factories return independent state.
The built-ins accept up to 65536 UTF-8 input bytes per chunk and drain requests up
to 65536 bytes, retaining the source and at most 256 pending encoded bytes. The
prototype's 4096/32-byte bounds are not the public contract. No String/Char semantic
change, fallback replacement, native opcode or compiler option is introduced.

StreamWriter now drives an encoder with a 256-byte output buffer, completes short
writes before draining again, and exposes Finish on the concrete writer. Finish
emits final bytes and returns their count; repeat success returns zero. Flush stays
stream-only and Close stays ownership-only. Existing UTF-8 defaults and leaveOpen
constructors remain. WriteLine preflights text plus LF as one input. Strict ASCII
rejects unsupported input before output; positive-capacity built-in drains make
progress even when a scalar spans several byte fragments.

Material development migration: custom Encoding implementations must add
CreateEncoder, returning independent sessions. Do not wrap arbitrary Encode calls
as a chunk-wise codec. After an encoder-drain or output error, subsequent writes
and Finish return InvalidEncoding; there is no replay of partly consumed input.
Source-limit and ASCII preflight errors remain retryable. Custom output exceeding
65536 bytes can be detected after partial output, but the writer never sends beyond
the per-call ceiling. This differs from the previous whole-value snapshot check.
Observe Finish and Flush before Close when final conversion output matters.

EncoderProgress rejects negative counts and inactive EncoderState values. Its public
surface is immutable; private storage follows the bridge's mutable-field profile.
Internal encoder helper methods follow its existing public-instance dependency
profile, without exposing the provider class. No importer admission broadening is
needed for these implementation details. An explicit union-case match on the state
getter establishes an active return for Raven's union-flow checks.

Validation: four public Encoder/writer runs and three existing encoding/line runs
pass against matching rebuilt reference/library artifacts. They include an
independent factory session, one-byte drains, the 256-byte scalar boundary, custom
final output, strict line rejection, partial-write failure, and a 65536-byte write
with oversized input/line rejection before output. Larger boundary fixtures use
the existing measure_async host limits; runtime defaults are unchanged. See
[public evidence](../experiments/text-boundaries/public-encoder-validation.json).
No full suite, website build or performance claim. This completes the bounded
UTF-8/strict-ASCII encoding foundation; broader codecs, general TextWriter completion,
HTTP charset policy and public builder promotion remain separately scoped work.
