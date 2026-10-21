# Positional declarations and patterns — 2026-10-21

The author requested one Raven contract across .NET and neoCLR, covering reference,
value and derived types, named and unnamed forms. The monetary consumer is renamed
from MonetaryAmount to Money without a compatibility alias.

## Shared language contract

- `let money = Money(...)` constructs the record normally.
- `let Money(amount, currency) = money` and `let (amount, currency) = money`
  bind components without else when the static contract guarantees the match.
- `let Money(amount, Currency.SEK) = money else { return ... }` constrains a
  component. Named patterns on an object/base input additionally test the type.
- Refutable bindings need an exiting else; an empty failure block does not
  establish definite assignment. RAV1620 diagnoses refutable named declarations
  without else. A plain tuple deconstruction needs a static Deconstruct contract;
  it does not guess a runtime type from object.
- Accessible inherited Deconstruct methods participate in ordinary member lookup.
- Qualified readonly static members are evaluated runtime equality patterns, not
  constants or captures. Equality uses Object.Equals, as existing explicit `==`
  patterns do. A getter need not be pure; evaluate it once when its component is
  reached. Such patterns do not add constant exhaustiveness coverage. Bare locals,
  mutable members and instance properties still require explicit comparison.

Raven main `d39ef018a` owns these rules. All 88 focused semantic/runtime tests pass
on .NET; the pre-change assignment baseline was 52 passing tests. Coverage includes
record structs/classes, tuple bindings, derived/base/object input, null/wrong-type
failure, component mismatches, once-only property evaluation and typed/boxed record
struct equality. The latter exposed an existing missing generated method body;
record struct equality now avoids reference-only guards and checks boxed type
before extracting the value. No monetary type name is special-cased.

## Native integration

`f5daaaea1` deliberately carries the main commit to codex/neoclr; `ddd62881f`
implements native lowering. Native lowering
uses existing type-test/conversion/unboxing instructions and declared Deconstruct
calls. Derived assignment receivers use ordinary implicit reference conversion.
Runtime equality patterns use ordinary Object equality with null handling; value
boxing is currently required on this path. This is semantic parity, not an
allocation/performance optimization. No new Runtime Contract setting, IL opcode,
CLI encoding or runtime money representation is introduced.

The built-in `deconstruction` suite exercises the requested reference/value/derived
matrix and production Money/Currency APIs. The `currency` suite verifies the renamed
record's equality/hash/display, formatting/parsing and interface behavior.

The rebuilt five-library cohort identifies its compiler as
`f5daaaea1-native-pattern-working`: those tested backend changes are committed as
`ddd62881f`. API snapshots were refreshed from that exact renamed native library
cohort. The API fingerprint check passes; the website builds and validates 5,216
pages, and the local Money reference responds with HTTP 200. Existing unrelated
missing API descriptions remain reported by the website checker.


## Execution evidence

[The built-in runner report](report.json) records all 30 passing executions:
four deconstruction and six currency tests under each of native, JIT and
interpreter. Input, compiler, runtime and library fingerprints are included.
This complements the 88 focused .NET tests; neither result substitutes for the
other target. No throughput or allocation benchmark claim is made.
