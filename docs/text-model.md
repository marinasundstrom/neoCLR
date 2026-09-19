# Text model and initial String API

The selected direction is native, well-formed UTF-8 with Unicode scalar semantics,
following the [String proposal](proposals/string-api.md). The author confirmed on
2026-09-19 that UTF-16 is not neoCLR's native text model or a compatibility constraint.
The interpreter already stores String as valid UTF-8. Strict byte conversion and
UTF-8 ordinal ordering are implemented; scalar-valued Char, scalar indexing and
iteration still require coordinated runtime/compiler work. Do not describe those
remaining changes as complete or preserve UTF-16 merely for .NET compatibility.

Future foreign-encoding adapters, if needed, belong at explicit interop boundaries.
They do not change the native model. A general Encoding API and specialized
Utf8String remain outside the minimal slice.

Separate the concepts through types and APIs:

- Byte holds an encoded octet, not necessarily a complete character.
- Char is intended to hold a validated Unicode scalar value under the proposal.
- System.Char currently still stores a UTF-16 code unit; this is migration debt,
  not a retained compatibility goal. A separate Rune is not required by this direction.
- Text-element APIs should handle grapheme clusters for user-facing operations.

UTF-8 represents a scalar using one to four bytes. A displayed text element can
contain multiple scalars. Byte counts, scalar counts, and text-element counts are
different operations. See [Unicode's encoding FAQ](https://unicode.org/faq/utf_bom.html)
and [.NET's Unicode model](https://learn.microsoft.com/en-us/dotnet/standard/base-types/character-encoding-introduction).

String.Length, general indexing, search offsets, and migration adapters still need explicit
contracts. SliceUtf8 explicitly uses UTF-8 byte offsets, as documented below. Familiar
.NET signatures must not quietly switch from UTF-16 offsets to byte offsets. Byte
counting is now explicit; scalar enumeration and a UTF-16 compatibility view remain
possible extensions.

A valid-text constructor should validate bytes and expose decoding failure as Result.
Raw or invalid data should remain bytes; lossy replacement should be explicitly
requested. UTF-16 conversion also needs a policy for unpaired surrogates, which Char
can represent but Unicode scalar values cannot. Native APIs need explicit encoding,
length, and termination contracts. UTF-8 is not a reason to assume every foreign
function accepts UTF-8 or requires null termination.

Normalization, comparison/culture behavior, grapheme segmentation, and the exact
Rune API remain future decisions. Choosing an encoding alone does not settle them.

## Implemented String members

System.String is a runtime-represented primitive type with ordinary platform-library
methods. The current methods are compiled by the same assembler as application code:

| Member | Contract |
| --- | --- |
| static Concat(String left, String right) -> String | Produce an owned concatenation, preserving all text including embedded NUL |
| instance Equals(String other) -> Boolean | Exact ordinal equality; no culture, case folding, or normalization |
| instance IsEmpty: Boolean | True only for the valid empty String |
| instance GetUtf8ByteCount() -> Int32 | Count encoded UTF-8 bytes, not UTF-16 units, scalars, or graphemes |
| instance SliceUtf8(Int32 byteStart, Int32 byteLength) -> System.Result<String,System.Text.Utf8SliceError> | Copy a valid UTF-8 byte range into an owned String |

Concat, byte count, and slicing forward through validated InternalCall declarations.
Equals and IsEmpty execute ordinary IL. No new IL opcode, array representation, or
special member dispatch is required. The runtime-service report classifies the three
helpers as StringOperations; slicing also requires ValueStorage for explicit carrier storage.

SliceUtf8 accepts an empty range at any code-point boundary, including the end of the
string. Negative arguments or a range beyond the encoded byte length return
an ordinary Utf8SliceError.OutOfRange case. Both endpoints must be code-point boundaries;
otherwise the error case is Utf8SliceError.InvalidBoundary, including an empty range
inside a multi-byte encoding. Success is Ok(String), including Ok(""). These checks
do not require grapheme boundaries: a combining mark can be sliced independently.

Empty is not absent, and an embedded NUL is ordinary content, not a terminator.
No invalid-text constructor or native string marshalling is added. String values remain
owned interpreter values, and copies/results do not expose shared writable byte storage.
This is not a permanent allocation ABI or a requirement for implicit GC.

A byte count outside Int32 range, concatenation size overflow, or failure of explicit
helper allocation reservation produces a terminal Fault. Range and encoding-boundary
errors are ordinary Result values. Host input validation and execution-limit Faults keep
their existing contracts and stack diagnostics. String memory has no separate guest
quota yet; catastrophic host allocation failure elsewhere is not guaranteed recoverable.

`cargo run -- run examples/strings.neoil` prints Hello, neoCLR!, reports 10 UTF-8 bytes
for "café 🌍", prints the globe slice, and handles an invalid boundary without terminating.


The non-generic System.Text.Utf8SliceError carrier directly nests OutOfRange and
InvalidBoundary, with constructors, IsOutOfRange/IsInvalidBoundary properties, checked
GetOutOfRange/GetInvalidBoundary accessors, and ToString. Range validation precedes
boundary validation: a request extending beyond the string is OutOfRange even when its
start lies inside a UTF-8 sequence. Case extraction on the wrong variant produces a Fault.

The internal StringSliceUtf8 helper now returns System.Value containing an erased String
on success, Byte 1 for OutOfRange, or Byte 2 for InvalidBoundary. Library IL constructs
the public error and Result cases; no message comparison or union-specific IL is used.
Unknown payload types/statuses Fault. The helper no longer constructs bootstrap unions.

Reassemble applications and System together. Replace ldcase/is.case after SliceUtf8 with
ordinary Result and error-case accessors, using the specific error type in signatures.
There is no parallel Typed method, and JSON format 4 does not imply library ABI stability.

## Explicit ordinal comparison and matching

[Ordinal text operations](ordinal-text.md) now provide String.CompareOrdinal and
readonly ContainsOrdinal, StartsWithOrdinal and EndsWithOrdinal. Ordering follows
.NET UTF-16 code units despite UTF-8 storage; matching is exact and case-sensitive.
These Boolean predicates expose no indexes and do not settle general indexing or
Length. Culture, normalization and case folding remain future work.

[Character classification](character-classification.md) adds familiar System.Char predicates and
Neo character literals, including Unicode IsDigit and explicit IsAsciiDigit.

The experimental [Raven String projection](raven-string-api.md) now includes
SliceUtf8 with typed Result handling and propagation over its existing error contract.

## Modern text and encoding review (2026-09-13)

The author reaffirmed UTF-8 text and requested modern String, character and encoding
APIs while preserving useful .NET ergonomics. This is a review of the public contract,
not a change to Char's UTF-16 meaning, CompareOrdinal's ordering or existing slicing.

The proposed first slice is validated Unicode scalar construction and iteration,
followed by explicit byte-to-text decoding and text-to-byte encoding with Result
errors. Names and signatures remain to be designed. Compare a narrow UTF-8/UTF-16 API
with a general Encoding abstraction; do not build a codec framework before a consumer
needs it. Prefer strict validation by default, explicit lossy replacement, and clear
consumed/written counts for eventual streaming or destination-buffer APIs. Define BOM
handling, embedded NUL, incomplete input and unpaired UTF-16 surrogate behavior.

Separate byte offsets, scalar iteration and grapheme-oriented editing. A scalar is
not necessarily a user-perceived character. Defer general culture/normalization and
grapheme APIs until required; do not claim they follow automatically from UTF-8.
Use owned copies first where necessary; zero-copy text views depend on the memory-view
lifetime contract. Measure transcoding and allocation costs rather than promising that
UTF-8 improves every workload.

Validation should cover ASCII, supplementary scalars, combining sequences, embedded
NUL, malformed/truncated UTF-8 and unpaired UTF-16 surrogates, plus a Raven file-reading
example. Keep migration offsets explicit; no silent reinterpretation of .NET indexes.

Primary comparison refreshed 2026-09-13: [.NET's encoding guide](https://learn.microsoft.com/en-us/dotnet/standard/base-types/character-encoding-introduction)
already distinguishes Char, Rune and text elements; those are useful precedents.
[UTF8Encoding](https://learn.microsoft.com/en-us/dotnet/api/system.text.utf8encoding.-ctor?view=net-10.0)
provides configurable invalid-input handling. Our proposed difference is a focused
UTF-8-first surface with Result-based failures, not invention of Unicode-aware APIs.

## Minimal Unicode-centred model proposal (2026-09-15)

The author proposed reducing the model to one semantic foundation: Unicode text,
with canonical UTF-8 String storage and explicit UTF-8/UTF-16 representation views.
ASCII is a Unicode subset rather than a parallel text system; Latin-1, Windows-1252,
Shift-JIS and similar formats remain codecs exposed through Encoding. The following
examples are design notation only, not claims that the current Raven compiler accepts
these declarations or literals:

```text
Unicode
  ├─ String / Char       semantic text values
  ├─ UTF-8               Byte representation
  └─ UTF-16              UInt16 representation
```

The proposed core would therefore define `Char` as one Unicode scalar value, permit
examples such as `Char('😀')`, and make `AsciiChar`/`AsciiString` constrained
representation types. Lossless widening would be available from an ASCII subset to
UTF-8 and then to String; narrowing would validate explicitly, for example
`AsciiString.From(text) -> Result<AsciiString, AsciiError>`. `Encoding` would translate
between external bytes and Unicode String rather than add formats to String itself.

This is a substantive reopening of the current contract, not an implementation update.
The implemented neoCLR `Char` and the documented .NET-compatible comparison currently
use UTF-16 code-unit semantics, while the earlier review kept `Rune` and `Char`
separate. The proposal's main benefit is a small, coherent semantic core: ordinary text
operations need not expose encoding families, and ASCII guarantees can be carried by
types. Its costs are compatibility and migration impact for existing `Char` literals,
predicates, ordering and metadata; a scalar is also not necessarily a user-perceived
grapheme. A constrained ASCII type adds conversion and generic/reflection surface, and
canonical UTF-8 does not remove the need for UTF-16 copies at .NET boundaries.

At this earlier review, alternatives remained open: retain the familiar .NET split (`Char` plus `Rune`),
make String scalar-oriented while retaining a UTF-16-compatible Char view, or adopt the
minimal scalar model above. The author’s later 2026-09-19 clarification selects the
scalar model and native UTF-8; the alternatives are retained here as decision history.
Before the Char migration, compare
the alternatives against current .NET `Char`/`Rune`/encoding contracts and the CLI
metadata representation, then validate literals, indexing/iteration, malformed input,
unpaired UTF-16 surrogates, ASCII narrowing, generic constraints, reflection and
interop. Existing String slicing and ordinal comparison must not silently change.

## Consolidated String proposal: semantic text and representation views

The author refined the preceding proposal into a more complete String direction. The
goal is to retain familiar .NET ergonomics—`String`, `Char`, PascalCase and ordinary
text operations—while making Unicode semantics explicit instead of exposing UTF-16
storage accidentally. This section is the preferred proposal for further review; it
does not change the current implementation.

`String` is immutable Unicode text and, semantically, a sequence of Unicode scalar
values (`String = sequence of Char`). Well-formed UTF-8 is its canonical storage
representation, but storage does not become part of ordinary String operations. String
has no `Encoding` property and is not parameterized as `String<Encoding>`.

`Char` is one Unicode scalar value. Surrogates cannot be valid Char values, so a
supplementary scalar such as `😀` is one Char even though its UTF-8 and UTF-16 encoded
forms use multiple code units. Unicode determines the meaning of Char; the selected
encoding determines only its representation. Character classification belongs on Char
(`IsLetter`, `IsDigit`, `Category` and related properties), while casing,
normalization and other potentially context- or culture-sensitive transformations
belong on String. A scalar is still distinct from a user-perceived grapheme; grapheme
segmentation remains a separate API concern.

When representation matters, use explicit encoded-string types rather than adding
storage details to String:

| Abstraction | Meaningful elements | Typical operations |
| --- | --- | --- |
| `String` | `Char` / Unicode scalar | Text semantics and ordinary search/editing |
| `Utf8String` | UTF-8 `Byte` units | Byte length, byte access, bytes and UTF-8 slicing |
| `Utf16String` | UTF-16 `UInt16` units | UTF-16 code-unit access and interop |
| `AsciiString` | `AsciiChar` | Protocol/token operations with an ASCII invariant |

`Utf8String` may be a zero-copy view over canonical String storage if the eventual
memory-view lifetime contract permits it; that is an optimization hypothesis, not an
ABI promise. `Length` means the number of elements in the current abstraction: String
counts Char values, while Utf8String counts UTF-8 code units. `AsciiChar` and
`AsciiString` carry stronger range invariants. In particular, ASCII `IsDigit` means
exactly `'0'..'9'`, whereas Char `IsDigit` follows Unicode classification. Explicit
constructors such as `AsciiString.From(text) -> Result<AsciiString, AsciiError>` are
required for narrowing; lossless widening can be provided where the invariant is
known.

`Encoding` describes transformation rules, not a property of text or a replacement
for encoded-string types. It can expose UTF-8, UTF-16, UTF-32, ASCII, Latin-1 and
other codecs through operations such as `Encoding.Utf8.Encode(text)` and
`Encoding.Utf16.Decode(bytes)`. UTF-8 is both the canonical/default String storage
and one member of the general encoding abstraction. Ordinary text APIs should accept
and return String; encoded types and Bytes belong at explicit representation and I/O
boundaries.

The error model follows the rest of neoCLR: successful searches return Boolean, absent
positions use `Option<Int>`, expected decoding, encoding and parsing failures use
typed `Result`, and `Fault` is reserved for failures where normal execution cannot
reasonably continue. This removes sentinel indexes, `TryParse` pairs and exceptions
from normal recoverable cases without making every text operation fallible.

The resulting separation is:

```text
TEXT:       String ──elements──> Char ──Unicode properties──> classification
STORAGE:    String ──canonical──> UTF-8 ──view──> Utf8String / Bytes
            UTF-16 ──view──> Utf16String / UInt16
            ASCII ──subset──> AsciiString / AsciiChar
ENCODING:   String <── Encode / Decode ──> Bytes
```

This proposal is deliberately different from the current UTF-16 code-unit Char
contract and from the existing UTF-8 byte-offset slicing API. Before implementation,
define migration for Char literals, predicates, ordering, indexing and metadata;
decide whether representation views are owned values or lifetime-bounded views; and
validate malformed/truncated input, unpaired UTF-16 surrogates, embedded NUL, ASCII
narrowing, generic constraints, reflection, native interop and allocation behavior.
Do not silently reinterpret existing String offsets or claim that the Raven-shaped
examples in this proposal are currently valid Raven source.

Strict byte conversion is now available through [System.Text.Utf8](raven-string-api.md#strict-utf-8-conversion-2026-09-19); broader text-model proposals above remain provisional.
