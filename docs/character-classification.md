# Character classification

This slice adds familiar static System.Char predicates for token validation and
text processing, plus single-quoted Neo Char literals. Char remains a UTF-16 code
unit, including surrogate units; String remains valid UTF-8 text. A future Rune
API is needed to classify supplementary Unicode scalars as a single value.

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
| IsSurrogate / IsHighSurrogate / IsLowSurrogate | D800–DFFF / D800–DBFF / DC00–DFFF |
| IsAscii / IsAsciiDigit | 0000–007F / ASCII 0–9 |

No culture, normalization, case conversion, input mutation, retention or recoverable
errors are involved. IsDigit does not promise that Int32.Parse accepts that character:
the current parser intentionally accepts ASCII digits only. Use IsAsciiDigit for
ASCII numeric protocols. A combining mark is not a letter, and a numeric fraction
is not a decimal digit. Surrogates are not classified as the scalar their pair encodes.
String/index overloads are deferred until indexing units are settled.

These are library rules, not new CLR opcodes. Neo emits ordinary static calls;
all other frontends can use the same APIs. Simple range checks and category policy
live in platform IL. One validated InternalCall helper supplies a category lookup,
reported as the new CharacterClassification runtime service. Its internal Int32
category IDs follow .NET UnicodeCategory; this is not yet a public enum API.

Rather than tying behavior to Rust's changing Unicode version or its broader
Alphabetic/Numeric properties, use a checked-in range table generated from
[Unicode 16.0.0 data](https://www.unicode.org/Public/16.0.0/ucd/UnicodeData.txt).
This costs table space and explicit Unicode update maintenance but makes behavior
reproducible across hosts. No performance advantage is claimed. Lookup uses binary
search and no allocation, GC ownership or reference lifetime machinery. The pinned
.NET 10.0.100 probe agrees on all 65,536 category values (FNV-1a fingerprint
`9FB70257D9A6A292`, retained in a runtime unit test); comparison to another .NET/Unicode version
must account for assignments changing between Unicode releases.

## Neo literals

`'7'`, `'é'`, `'\n'` and `'\uD800'` produce Char values. Supported escapes are
`\0`, `\n`, `\r`, `\t`, `\b`, `\f`, `\v`, `\\`, `\'`, `\"` and exactly four
hexadecimal digits after `\u`. A literal must encode exactly one UTF-16 unit:
empty literals, multiple characters and supplementary scalars such as `'🌍'` are
rejected. Explicit surrogate escapes are permitted. This follows the useful bounded
part of [C# character literals](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/char); variable-width `\x` and eight-digit `\U` escapes
are not implemented. Literals lower to `ldc.i4` followed by `conv.u2`.

## Use and validation

```sh
cargo run --locked -- run examples/source/character-classification.neo
cargo test --locked --test character_classification --test native --test library
(cd docs/experiments/character-classification-dotnet && dotnet run)
python3 scripts/generate-char-categories.py /path/to/UnicodeData.txt
```

The sample distinguishes Unicode and ASCII digits, checks a letter and whitespace,
and demonstrates a surrogate code unit. Tests exercise static IL calls, serialized
artifacts, source literals and invalid literal diagnostics. Generation verifies the
source SHA-256, handles UnicodeData range records and defaults unassigned units to
Cn. Unicode data licensing is retained in `third-party/unicode/LICENSE.txt`.
