# Strings, characters and encoding

A String is Unicode text. A Character is one grapheme cluster—a practical approximation
of a character as a reader perceives it. UTF-8 is the internal encoding; converting
between text and bytes is an explicit operation.

The development API names the grapheme type `Character` (previously `Char`).
Character literals infer it, and explicit annotations use `Character` after
`import System.*`. The neoCLR target treats `char` as an ordinary identifier,
not a type alias. Raven targeting .NET retains its `char` keyword and UTF-16 type.
Rebuild libraries and consumers together when migrating.

## Count and iterate characters

`Length` counts grapheme clusters, and string iteration yields Character values:

```raven
{{GRAPHEME_COUNTS}}
```

This prints 4 characters, 12 Unicode scalars and 37 UTF-8 bytes.

```raven
{{GRAPHEME_ITERATION}}
```

Graphemes can contain combining marks or several emoji scalars. They are not always
single displayed glyphs. Counting scans text, and concatenation can change boundaries.
[Complete character sample](/samples/library-grapheme-strings.rvn).

## Convert text and bytes

Use `Encodings.Utf8` or `Encodings.Ascii` through the Encoding interface. Encode and
Decode return Results with typed errors. UTF-8 decoding rejects malformed or
truncated input; ASCII rejects values it cannot represent rather than substituting.

```raven
{{UTF8_ROUNDTRIP}}
```

`RoundTrip("café 🌍")` prints `Round trip`. StreamReader and StreamWriter accept
encoding policies and default to UTF-8. Incremental Decoder and Encoder instances
support bounded progress across buffers. In development, `Decoder.Decode` accepts
`ReadOnlySlice<byte>` and `Encoder.Drain` accepts `Slice<byte>` directly. Pass an
array or an array window; codecs do not retain borrowed input or destinations.
Incomplete decoded bytes are copied into owned state. See [stream and encoding contracts](/docs/streams.html).

## Index, slice and compare

String indexing addresses graphemes. Bracket access with Index or Range is a
convenience that can fault on invalid bounds; validate inputs or use the checked
operations described in the [sequence-access guide](/docs/sequence-access.html).

In the development API, String and Character equality recognizes canonical Unicode 17
equivalence: precomposed é and e followed by a combining accent compare equal and
have matching hashes. Their original UTF-8 bytes remain unchanged. Comparisons
and hashing do not allocate normalized copies.

`StringComparer.Default` follows this text policy. `StringComparer.Ordinal`
compares exact UTF-8 and remains appropriate for exact identifiers and protocol
fields. `OrdinalIgnoreCase` applies Unicode simple folding without normalization
or language-specific rules. Canonical equivalence, case folding and linguistic
collation are separate operations.

Borrow `Utf8Bytes` to process storage without an encoding array. A byte window is
not automatically valid text, and a Unicode scalar boundary is not necessarily a
grapheme boundary. Traverse graphemes sequentially when possible: repeatedly
locating the nth grapheme can require repeated scans.

These are deliberate differences from .NET's UTF-16 Char and string indexing model.
Development grapheme segmentation now uses Unicode 17 alongside canonical
comparison. This changes boundaries for some newer combining marks; stored bytes
remain unchanged. Scalar category classification still uses Unicode 16.
Case conversion, folding and segmentation have their own versioned Unicode rules;
consult the [String](xref:System.String) and [Character](/docs/api/System/Character/index.html) contracts when
text boundaries or cross-platform comparison must be exact.

## Work with one character

Development `Character.TryFromString(text)` returns `Some(character)` for one complete
grapheme and `None` for empty text or multiple graphemes. It preserves the original
UTF-8 spelling. `Character.Equals(other, comparison)` also supports explicit Canonical,
Ordinal and OrdinalIgnoreCase policies, comparing character storage directly.
For example, composed and decomposed accents match canonically but differ ordinally.

Character represents a grapheme, while `UnicodeScalar` owns scalar category checks.
Successful Character/Option construction can allocate under the current representation;
a borrowed Character byte view remains future work. See the [Character reference](/docs/api/System/Character/index.html).

## Search text

Development `Contains`, `StartsWith` and `EndsWith` compare complete graphemes by
canonical equivalence. They also accept an explicit `StringComparison` policy:

```raven
let text = "Aé👩‍💻🇸🇪\u0000Z"
let equivalent = text.Contains("é") // true
let wholeCharacter = text.Contains("e") // false
let exact = text.Contains("e", StringComparison.Ordinal) // true
let folded = "xΣKẞy".Contains("ςkß", StringComparison.OrdinalIgnoreCase) // true
```

These calls come from the tested search consumer. Ordinal matching uses exact bytes;
OrdinalIgnoreCase uses Unicode simple folding without normalization or expansion.
Both can match inside a grapheme. Empty patterns match; unknown comparison modes
fault. The predicates borrow existing bytes and create no temporary strings or
arrays. Long shared prefixes can require repeated comparisons; fewer allocations
alone do not establish faster HTTP throughput. See the [String reference](xref:System.String)
for overloads and contracts.

## Build text

Use concatenation for small expressions and `StringBuilder` for incremental output.
Append and AppendLine add text; ToString returns an independent immutable snapshot.
AppendLine uses LF, while wire protocols such as HTTP need explicit CRLF.
String.Join places a separator between array elements, including empty elements.
Both builder and conversion APIs enforce explicit size limits.

## Read further

- [Encoding and streams](/docs/streams.html) — ownership, partial progress and errors.
- [StringBuilder](xref:System.Text.StringBuilder) and [StringComparer](xref:System.StringComparer).
- [Object equality contracts](/docs/objects.html) — identity, boxing and content equality.
- [Globalization](/features/globalization/) — culture-sensitive operations.

<a id="characters"></a>

<a id="conversion"></a>

<a id="try"></a>

<a id="direction"></a>

<a id="feedback"></a>

<a id="char-through-object-development"></a>

<a id="string-through-object-development"></a>

<a id="constructing-and-indexing-text-development"></a>

<a id="explicit-interning-development"></a>

<a id="explicit-comparison-policies-development"></a>

<a id="encoding-foundations-development"></a>

<a id="unicode-casing-and-decimal-reports-development"></a>

In development, `String.Utf8Bytes` also works in native JIT/AOT: it borrows immutable
UTF-8 storage without an encoding array and retains managed text through collection.
Its indices count bytes; Character and ordinary string indexing remain grapheme-based.
Use `ToArray()` only when an independent mutable byte array is needed.


### Whitespace trimming (development)

`Trim()`, `TrimStart()` and `TrimEnd()` remove Unicode 17 whitespace scalars from
the requested edges. They scan borrowed UTF-8 bytes, reuse unchanged strings and
copy retained text only when needed. Original spelling and interior spaces remain
unchanged. Combining marks are retained: removing a leading space can leave its
following mark at the start. Indexing still uses graphemes. Protocol parsers should
keep their own whitespace rules instead of using general Unicode trimming.


Development null checks include `String.IsNullOrEmpty(text)` and
`String.IsNullOrWhiteSpace(text)`, which inspect text without materializing trimmed
copies. `Equals(other, comparison)` provides explicit equality policies;
`CompareTo(other)` and `ComparableTo<string>` use the same canonical order as
`String.Compare`. Use the matching StringComparer when choosing dictionary identity.


Development `ReplaceLineEndings()` produces LF consistently across platforms;
pass a replacement explicitly for another convention. CRLF is treated as one
newline. The implementation preserves all other bytes, reuses unchanged strings
and writes changed native output directly into its final allocation. This is a
text utility, not an HTTP framing operation.

## Composite formatting (development)

`String.Format` and the matching `Console.Write`/`WriteLine` overloads accept indexed
placeholders, alignment and escaped braces. `System.Formattable` lets a type interpret
its own format string through `ToString(format: string, culture: Culture)`. Alignment counts graphemes;
null arguments produce empty text. Types without Formattable use ordinary ToString.

One-, two- and three-argument overloads avoid an argument array; a params overload
handles larger lists. This is not allocation-free: boxing, text fragments and final
output can allocate. Formatted output currently has the builder's 65536-byte limit;
custom providers and primitive numeric-format integration remain gaps. Console output
uses its synchronous text writer. String storage/view
changes remain deferred.

Culture-first `String.Format(culture, format, ...)` overloads pass the selected culture
to every Formattable item. Existing overloads resolve one current-culture snapshot
when needed. Date, TimeOfDay, LocalDateTime and Duration implement the interface;
see [time formatting](/features/time/) for the supported development specifiers.
Implementations of the earlier one-argument Formattable contract must add the culture
parameter and consumers must be recompiled. No full locale-pattern support is implied.
