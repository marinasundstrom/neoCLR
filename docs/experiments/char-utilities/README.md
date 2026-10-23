# Char utilities — 2026-10-19

Development Raven APIs:

- `Char.TryFromString(string) -> Optional<char>` returns Some for exactly one Unicode
  17 extended grapheme and None for empty/multiple graphemes. Input is non-null;
  bytes are preserved. FromString remains the faulting alternative.
- `Char.Equals(char, StringComparison) -> bool` compares complete graphemes using
  Canonical, Ordinal byte identity or OrdinalIgnoreCase simple-fold scalar identity.
  Unknown modes fault, even when both arguments are equal. Default equality/hash
  remain canonical; explicit-policy comparisons do not change dictionary identity.

## Design comparison

[.NET Char.TryParse](https://learn.microsoft.com/en-us/dotnet/api/system.char.tryparse?view=net-10.0)
provides the non-throwing conversion role with Boolean/out and a single UTF-16 code
unit. neoCLR uses its existing Optional union and grapheme Char contract. This follows
[Swift Character's one-grapheme requirement](https://developer.apple.com/documentation/swift/character/init(_:)-6o1aq)
while exposing failed validation explicitly. The benefit is accepting composed and
decomposed graphemes, flags and emoji consistently; the cost is Unicode segmentation
rather than a code-unit length check. No parser overload family or null-as-text
convention is imported. Existing Raven union patterns extract the result.

The private Boolean service checks whether the first grapheme spans all the input,
without materializing an array or scanning every grapheme to compute Length.
Native input validation still checks UTF-8. Successful construction calls the
existing CharFromString service and uses an ordinary Some carrier. The interpreter
currently copies an owned Char buffer; the native representation can retain the
immutable text descriptor. No allocation-free construction guarantee is made.

Explicit equality compares Char storage directly through exact typed runtime
services. It avoids CharText/String intermediates and reuses the canonical,
ordinal and simple-fold kernels already used by String. These policies are not
cultural collation or compatibility normalization. A shared immutable Char byte
view is still a gap: interpreter Char owns String storage, and converting it to
StringValue currently copies. Do not describe that conversion as a borrowed view.

## Qualification

```
cargo test --release --features native-metadata --test native --test grapheme_chars
cargo test --release --manifest-path tools/aot-native-text/Cargo.toml validates_extended_graphemes_without_normalizing
python3 docs/experiments/char-utilities/check.py target/char-utilities-bundle-v1/bundle target/char-utilities-main-v2 --invalid target/char-utilities-invalid-v1 --output target/char-utilities-validation.json
python3 docs/experiments/char-utilities/measure.py target/char-utilities-allocation-v1 --output target/char-utilities-allocations.json
```

Five registry and five grapheme runtime tests pass, plus the native helper test.
The public consumer covers empty/multiple input, decomposed accents, emoji, flags,
CRLF/NUL/standalone marks, newer combining marks, exact byte retention, three
comparison policies, simple-fold limitations and invalid modes. An Optional<Char>
created from a dynamically allocated String survives return from its factory and
subsequent allocation/collection. See [six cross-engine executions](validation.json).

[Native allocation counters](allocations.json), 128 iterations per mode:

| Mode | Allocations | Reclaimed | Eager collections |
| --- | ---: | ---: | ---: |
| Baseline | 2 | 2 | 1552 |
| Three explicit comparisons | 2 | 2 | 1296 |
| TryFromString success | 2 | 2 | 2320 |
| TryFromString failure | 2 | 2 | 2320 |

These measured native workloads add no managed allocations. Collection counts also
include mode-dispatch branch differences; they are not isolated operation costs or
throughput measurements. The comparison path avoids String conversion; successful
interpreter Char construction still copies, and this test does not measure its
host allocations. No HTTP performance claim follows from these counters.

Artifacts: canonical-text-compiler-v1 (compiler code 2d22055dc, metadata adapter
e4709c451); char-utilities-libraries-v1 and bundle-v1; main-v2,
invalid-v1 and allocation-v1. Both API snapshots include the selected Char type's
new members with source summaries, parameters/results and limitations. Native
snapshot: char-utilities-api-audit-v1. Matching runtime/backend/helper builds are
required; Runtime Contract configuration and ordinary .NET behavior are unchanged.

Compiler bridge documentation: Raven codex/neoclr `03bcc23d6` (documentation only).
API snapshot checks and the 4,566-page website build pass. Publication is separate.
