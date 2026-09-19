# Text abstraction: characters before encoding

Direction revised **2026-09-19**. The author selected grapheme-based ordinary String
length, indexing and iteration, with scalar and encoded access explicit. This
supersedes the earlier plan to expose scalar Char as String's ordinary element.
It does not claim that the revised contract is implemented.

## Public model and representation

String represents Unicode text. UTF-8 remains the canonical internal encoding;
it does not define public character boundaries or require an Encoding property
on String. Encoding conversion belongs at byte I/O and interop boundaries.

The assistant recommends Char represent exactly one extended grapheme cluster,
with a distinct explicitly named Unicode scalar type/view for parsers and Unicode
classification. A grapheme can contain multiple scalars, so this Char cannot retain
the current fixed-width integer representation. An owned text value or immutable
slice are implementation alternatives, not yet selected. The current scalar Char
implementation is provisional and must not be described as a grapheme value.

The intended direction is Length in graphemes, indexing returning a Char and
iteration yielding Char. A temporary indexer returning a one-grapheme String was
prototyped but is not being integrated as the public contract. Exact scalar type
names, character construction and classification rules remain design work.

## Unicode and .NET comparison

[Unicode UAX #29](https://www.unicode.org/reports/tr29/) defines extended grapheme
clusters as a practical approximation of user-perceived characters. They are not
identical to rendered glyphs or every writing system's intuitive character unit.
Both a precomposed é and e plus a combining accent form one cluster; many family
emoji sequences contain multiple scalars in one cluster. Boundaries depend on
context, so concatenation may join the last and first clusters of its inputs.

.NET keeps String/Char code-unit access separate from scalar traversal through
[String.EnumerateRunes](https://learn.microsoft.com/en-us/dotnet/api/system.string.enumeraterunes?view=net-10.0)
and text-element operations through
[StringInfo](https://learn.microsoft.com/en-us/dotnet/api/system.globalization.stringinfo?view=net-10.0).
Sources reviewed 2026-09-19. neoCLR's proposed default makes text-element handling
ordinary, at the cost of variable-size Char values and segmentation work.

Length and integer indexing must not promise constant-time access. Cursor-based
indices or iteration may avoid repeated scans; caching has memory/lifetime costs.
Normalization and equality are separate decisions: grapheme-based access does not
automatically equate canonically equivalent sequences. Choose and document the
Unicode segmentation version and upgrade behavior before publishing this contract.

## Work retained and next steps

The committed scalar implementation remains useful groundwork for decoding and
Unicode classification. A local experiment with unicode-segmentation 1.12.0
(Unicode 16) passed boundary tests for combining marks, emoji ZWJ sequences, flags,
skin tones, CRLF, empty strings and embedded NUL, plus invalid indices and snapshot
limits. It has not established a release dependency/version or a public Char model.

Next, define Char construction, value semantics, scalar access and pattern behavior;
then implement the minimal grapheme String surface against that contract. Preserve
explicit byte operations and keep specialized encoded string classes out of scope.
