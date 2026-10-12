# Strings, characters and encoding

A String is Unicode text. A Char is one grapheme cluster—a practical approximation
of a character as a reader perceives it. UTF-8 is the internal encoding; converting
between text and bytes is an explicit operation.

## Count and iterate characters

`Length` counts grapheme clusters, and string iteration yields Char values:

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
support bounded progress across buffers. See [stream and encoding contracts](/docs/streams.html).

## Index, slice and compare

String indexing addresses graphemes. Bracket access with Index or Range is a
convenience that can fault on invalid bounds; validate inputs or use the checked
operations described in the [sequence-access guide](/docs/sequence-access.html).

Equality is ordinal and does not normalize Unicode: precomposed é and an e followed
by a combining accent remain different values. `StringComparer.Ordinal` supplies
consistent comparison, equality and hashing for maps and sets. OrdinalIgnoreCase
uses Unicode simple folding without language-specific rules or normalization.

These are deliberate differences from .NET's UTF-16 Char and string indexing model.
Case conversion, folding and segmentation have their own versioned Unicode rules;
consult the [String](xref:System.String) and [Char](xref:System.Char) contracts when
text boundaries or cross-platform comparison must be exact.

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
