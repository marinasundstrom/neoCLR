# Character classification

**Current API (2026-09-19):** classification now lives on
`System.Text.UnicodeScalar`, accepting `uint` scalar values. Char is a grapheme text
value. The scalar-Char implementation notes below are historical; see
[the current text contract](design/text-abstraction.md).

The runtime now represents Char as a validated Unicode scalar stored in four bytes.
String remains native valid UTF-8. Supplementary characters are classified directly;
surrogates and values above U+10FFFF fault at Char value/storage boundaries.
This supersedes the original 2026-09-08 UTF-16 code-unit contract. Raven projects select this representation through the Unicode scalar Char contract.

## Contract and .NET comparison

Primary sources consulted 2026-09-08:
[Char](https://learn.microsoft.com/en-us/dotnet/api/system.char?view=net-10.0),
[IsDigit](https://learn.microsoft.com/en-us/dotnet/api/system.char.isdigit?view=net-10.0),
[IsLetter](https://learn.microsoft.com/en-us/dotnet/api/system.char.isletter?view=net-10.0),
[IsWhiteSpace](https://learn.microsoft.com/en-us/dotnet/api/system.char.iswhitespace?view=net-10.0).
Reuse their single-Char library contracts:

| Static method, each `(Char value) -> Boolean` | Meaning |
| --- | --- |
| IsDigit | Decimal digits (Nd), including non-ASCII digits |
| IsNumber | Nd, Nl and No; includes fractions and numeric symbols |
| IsLetter | Lu, Ll, Lt, Lm and Lo |
| IsLetterOrDigit | Letters or decimal digits |
| IsUpper / IsLower | Lu / Ll; titlecase is neither |
| IsWhiteSpace | Separators plus U+0009–000D and U+0085 |
| IsSeparator | Zs, Zl and Zp |
| IsControl | Cc |
| IsPunctuation | Pc, Pd, Ps, Pe, Pi, Pf and Po |
| IsSymbol | Sm, Sc, Sk and So |
| IsAscii / IsAsciiDigit | 0000–007F / ASCII 0–9 |

No culture, normalization, case conversion, input mutation, retention or recoverable
errors are involved. IsDigit does not promise that Int32.Parse accepts that character:
the current parser intentionally accepts ASCII digits only. Use IsAsciiDigit for
ASCII numeric protocols. A combining mark is not a letter, and a numeric fraction
is not a decimal digit. Surrogates cannot be supplied as Char values. The surrogate predicates have been removed.
String/index overloads are deferred until indexing units are settled.

These are library rules, not new CLR opcodes. Neo emits ordinary static calls;
all other frontends can use the same APIs. Simple range checks and category policy
live in platform IL. One validated InternalCall helper supplies a category lookup,
reported as the new CharacterClassification runtime service. Its internal Int32
category IDs follow .NET UnicodeCategory; this is not yet a public enum API.

Rather than tying behavior to Rust's changing Unicode version or its broader
Alphabetic/Numeric properties, use a checked-in range table generated from
[Unicode 16.0.0 data](https://www.unicode.org/Public/16.0.0/ucd/UnicodeData.txt).
The table now covers U+0000 through U+10FFFF. This costs table space and explicit Unicode update maintenance but makes behavior
reproducible across hosts. No performance advantage is claimed. Lookup uses binary
search and no allocation, GC ownership or reference lifetime machinery. The pinned
.NET 10.0.100 probe agrees on all 65,536 category values (FNV-1a fingerprint
`9FB70257D9A6A292`, retained in a runtime unit test); comparison to another .NET/Unicode version
must account for assignments changing between Unicode releases.

## Neo literals

`'7'`, `'é'`, `'🌍'`, `'\n'` and `'\U0001F600'` produce scalar Char values.
The Neo frontend accepts one scalar, common escapes, four-digit \u and eight-digit
\U escapes. Surrogates, out-of-range scalars and multi-scalar literals are rejected.
The supplementary value is preserved on the Int32 evaluation stack without a
16-bit conversion. This describes the historical Neo frontend; Raven's neoCLR target accepts the same scalar literal forms.

## Use and validation

```sh
cargo run --locked -- run examples/source/character-classification.neo
cargo test --locked --test character_classification --test native --test library
(cd docs/experiments/character-classification-dotnet && dotnet run)
python3 scripts/generate-char-categories.py /path/to/UnicodeData.txt
```

The sample distinguishes Unicode and ASCII digits, checks a letter and whitespace,
and classifies a supplementary symbol. Tests exercise static IL calls, serialized
artifacts, source literals and invalid literal diagnostics. Generation verifies the
source SHA-256, handles UnicodeData range records and defaults unassigned units to
Cn. Unicode data licensing is retained in `third-party/unicode/LICENSE.txt`.

## Scalar migration rationale — 2026-09-19

The author selected the original scalar-Char/UTF-8 direction. Compared with .NET
Char, this accepts supplementary values as one Char and excludes surrogate units,
closer to the validated-scalar role of .NET Rune. Char is still not a grapheme.
Costs include four-byte native layout, incompatible surrogate inputs and rebuilding
code that assumes 16-bit Char storage. The pinned Unicode category data is unchanged
in version; its coverage is extended. Numeric storage validates instead of truncating
to 16 bits, and native reads reject malformed scalar bit patterns.
