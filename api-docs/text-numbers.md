# Casing and decimal reporting (development)

These APIs require matching development runtime, System library and compiler
references. They are not part of the published Preview 10 package.

[String](xref:System.String) provides `ToUpperInvariant()` and `ToLowerInvariant()`.
Both return String using **Unicode 17 full default casing**. They can expand text
and change byte, scalar and grapheme counts. Uppercasing `Straße ﬃ` yields `STRASSE FFI`;
lowercasing `ΟΣ` yields `ος`, and `İ` yields `i` plus a combining dot. Context uses
the original text. There is no normalization, locale selection or titlecasing.

The familiar .NET names do not promise identical results: full expansions and final
sigma differ from the tested .NET 10 invariant methods. Casing is also separate from
[StringComparer.OrdinalIgnoreCase](xref:System.StringComparer), which retains its
simple-fold equality/hash policy. Existing String references remain unchanged.
Grapheme segmentation still uses its documented Unicode 16 data.

[Int64](xref:System.Int64) provides:

| Member | Contract |
| --- | --- |
| `Parse(value: string)` | `Result<long, Int64ParseError>`; optional ASCII sign and one or more ASCII digits. Leading zeros and negative zero are allowed. |
| `ToString()` | ASCII decimal, negative sign only when needed, no grouping or leading zeros; round-trips every Int64 value. |
| `MinValue` | Static read-only property: -9223372036854775808. |
| `MaxValue` | Static read-only property: 9223372036854775807. |

The bounds are properties, not compile-time constant fields. No format strings,
styles or culture providers are supported. Unlike default .NET parsing, whitespace
is rejected. Separators, prefixes, NUL and non-ASCII digits are also invalid.

[Int64ParseError](xref:System.Int64ParseError) is a standard Raven union:
`InvalidFormat` means the whole input does not match the grammar; `Overflow` means
valid decimal text lies outside the range. Format validation comes first, so an
oversized number followed by `x` is InvalidFormat. Match cases through Raven patterns.
An uninitialized error union has no active value; Parse returns active errors only.

The executable report consumer is maintained in
`docs/experiments/casing-integer/Main.rvn` in the repository.
