# Text abstraction: characters before encoding

Development contract, revised **2026-09-19**. The author approved graphemes as the
default and then directed implementation of the Swift-inspired model. The guiding
goal is an improved .NET-like experience suited to modern computing, with familiar
names and explicit lower-level operations. This is next-preview development, not
part of published Preview 7.

## Public contract

- String is immutable Unicode text. UTF-8 is the canonical internal representation
  and the default explicit byte conversion, not the public character unit.
- Char is exactly one extended grapheme cluster. Raven character literals accept
  combining sequences and emoji sequences; `Char.FromString(text)` validates a
  dynamic value and faults if it is empty or contains more than one cluster.
- `String.Length` counts graphemes. String implements `Iterable<char>`; ordinary
  iteration yields Char. There is no integer string indexer in this slice.
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
iteration may follow; ordinary integer indexing is intentionally deferred.

Segmentation uses unicode-segmentation **1.12.0**, pinned to **Unicode 16**, matching
the classification data version. Upgrading the rules can change boundaries and
counts. Raven diagnoses literals using the host .NET StringInfo rules; the target
runtime revalidates every constructed character with its pinned rules. Host Unicode
versions may differ; matching literal diagnostics to the pinned target rules remains
a tooling limitation to resolve before a stable language contract.

A cluster approximates a perceived character, not necessarily one rendered glyph.
Joining strings can change the boundary at their join: counts are not generally
additive. Precomposed é and e plus an accent both have Length 1 but remain distinct
under ordinal equality. Normalization, culture-sensitive comparison and collation
are separate future design decisions.

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
