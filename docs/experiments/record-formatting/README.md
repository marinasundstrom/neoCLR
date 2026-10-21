# Native scalar record formatting — 2026-10-20

This development slice addresses the generated `ToString()` gap recorded in the
[release review](../../validation/release-readiness-20261020/README.md). The retained
[baseline](baseline.txt) fails on the preceding matched compiler with an unregistered
self-dependency. It does not establish release readiness.

## Contract and implementation

Nongeneric source positional record structs containing `int`, `long`, `bool`,
`string` (including null), `Char`, and nested supported source record structs now
have an emitted `ToString()` override. Typed and boxed calls use the same body.
Output follows Raven's existing `Name { Member = value }` display syntax, including
quoted strings/characters, escaped backslashes/quotes, literal `null` and Boolean
`True`/`False`. Text retains its original UTF-8 representation; equality policy does
not normalize displayed values. This is diagnostic display, not serialization.

The native emitter builds a bound body against the selected core's StringBuilder,
String.Replace, Object.ReferenceEquals and typed scalar ToString methods. Nested
records append through public getters into the same builder. There is no generated
component boxing, reflection, CLR helper dependency or intermediate nested record
string. Output strings, scalar formatting strings, escaping replacements and builder
storage still allocate. The existing builder's 65,536 UTF-8-byte limit applies;
exceeding it faults. Avoiding intermediate nested strings also increases generated
body size for repeated nested shapes; no code-size or throughput improvement is claimed.

[.NET records](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/record)
similarly provide generated named-member display using a builder. Raven already
quotes string/Char values and displays null explicitly; this slice preserves those
language semantics rather than switching to C# display spelling. It avoids the
shared Raven CLR formatter's Object conversion for statically known components.
Ordinary Raven/.NET synthesis is unchanged.

## Boundaries

A subsequent [union component slice](../record-union-formatting/README.md) extends
this qualification. The scope below describes the original scalar cohort.

Generic records, record classes, union/enum/imported-value and floating-point
components remain unqualified for native generated formatting. Calls to un-emitted
source record formatting now report the missing scalar/text-helper contract rather
than an unrelated dependency error. Missing core formatting helpers also reject
cleanly. Empty positional record syntax currently fails earlier with `Unsupported
node kind: RecordDeclaration`; this separate frontend gap was observed while
constructing the fixture and is not repaired here. Formatting support is independent
of equality/hash admission. Public ConsoleKeyInfo still contains unqualified
components, so this change does not add its ToString override or change public
class-library API signatures. Internal scalar TerminalKeyStep gains the override.

## Validation

Compiler implementation: Raven `7062ed558` on `codex/neoclr`. The tested artifact is labeled `cf66a235c-dirty-record-formatting`, built before that commit; libraries and
bundle are freshly generated from that compiler. Runtime/backend are reused from
the preceding record-component cohort. Exact executable hashes and commands are in
[validation.json](validation.json). The compiler source-core regression separately
checks rejection when native text helpers are absent; [compiler-probe.txt](compiler-probe.txt).

The executable checks integer limits, nested records, null and ordinary text,
Boolean display, grapheme text, escaped quotes/backslashes, ordinal preservation of
decomposed Unicode and boxed dispatch. `check.py` runs interpreter/JIT/AOT and an
unsupported imported Index component compile. The allocation diagnostic compares
128 nested display operations with an equivalent hand-written single builder;
[allocation-results.json](allocation-results.json) records measured managed counts
and reclamation, not HTTP throughput or host allocator activity. Baseline: 2 allocations; generated and manual: 770 each (six additional managed allocations per iteration), all reclaimed. Collections were 8 / 8,968 / 8,328 respectively; this does not show CPU parity or improved throughput.

```sh
python3 docs/experiments/record-formatting/check.py target/record-format-bundle-v1/bundle target/record-format-app-v2 --output docs/experiments/record-formatting/validation.json
python3 docs/experiments/record-formatting/measure.py target/record-format-allocation-v1 --output docs/experiments/record-formatting/allocation-results.json
```

## Imported value formatting follow-up — 2026-10-21

Native generated display now calls a public, concrete, parameterless String-returning
`ToString` declared by an imported value type. Admission and emission share that
method selection. This follows .NET's use of component display contracts without
inspecting private fields or guessing that a type is a record. There is no new
Runtime Contract configuration, metadata encoding or ABI. Ordinary Raven/.NET
synthesis is unchanged. The existing equality/hash admission remains independent.

The generated call uses the value receiver directly, without adding a component
Object box. The component's returned string and existing builder still allocate;
this change makes no allocation-count or throughput claim. Ref-like values and
inherited Object fallback are not admitted by this rule. Generic owners, enum and
floating components, and record classes remain unqualified. In particular the
floating repro rejects at equality synthesis before formatting.

Raven `bf541d49c` on `codex/neoclr` supplies the compiler fix. All nine executions
(three tests across interpreter, JIT and native) pass. The built-in
`record-formatting` suite covers Index and Range, typed/boxed record
calls, nested records, Option payloads, null, scalar display and ordinal preservation
of decomposed UTF-8. See [the execution report](imported-validation-20261021.json),
[the current gap audit](gap-audit-20261021.json), and
[the missing-helper compiler regression](imported-source-core-20261021.txt).
The original Unsupported Index fixture is now a generic-owner rejection fixture;
the earlier dated validation remains a historical result. Libraries/runtime/backend
are reused from the method-constraint cohort; only the compiler is rebuilt. This is
development evidence, not clean extracted-package qualification.

## Enum component follow-up — 2026-10-21

Native nongeneric record-struct display now supports Int32-backed enum components,
including imported enums and closed-union payloads. It calls the existing ordinary
Object display contract, which owns alias selection, flag decomposition and numeric
fallback. No copied enum tables, separate formatter or runtime service is introduced.
This follows .NET general enum display; Raven's record punctuation is unchanged.

The current Object contract requires a temporary enum box per formatted component.
That is a known allocation cost, not an allocation optimization. Named-value display
reuses image text; flags and numeric fallback retain their existing result allocation.
A future borrowed-value display contract could remove the box without duplicating
formatting policy. Imported values with their own direct formatter remain unboxed.
Runtime Contract configuration, native metadata and ordinary .NET generation are unchanged.

The five built-in record-formatting tests cover names/aliases, flag decomposition,
zero, unnamed and negative values, imported enums, union payloads, boxed record
calls and retention across collection, alongside earlier imported/scalar cases.
All 15 executions pass with Raven `f13bf8de1` on `codex/neoclr`, together with
the source-core missing-helper regression. [Execution evidence](enum-validation-20261021.json)
records exact artifacts and results.
The preceding gap audit is the enum rejection baseline. Floating components,
generic owners and record classes remain gaps. This is development qualification,
not a matched release bundle or a performance result.

## Floating component follow-up — 2026-10-21

Native nongeneric positional record structs now admit Single and Double components
for generated equality, hashing and display. This uses existing typed primitive
Equals/ToString and ordinary Object hashing, rather than record-specific floating
algorithms. As with .NET record component equality, NaNs compare equal and signed
zeros compare equal with matching hashes. Floating operators themselves still use
IEEE comparisons, so a NaN does not compare equal through primitive `==`.

The generated typed equality/display calls do not box their floating components.
Hashing currently uses the existing Object fallback and therefore boxes each floating
component. Record-to-Object or interface conversion also boxes, and formatting results
and the builder allocate. No allocation reduction or throughput improvement is claimed.
No primitive API, Runtime Contract configuration, metadata encoding or ABI changes.
Ordinary Raven/.NET emission remains unchanged.

Built-in tests exercise both widths, NaNs, signed zeros, infinities, unequal finite
components, default values, boxed/interface dispatch, nested records, union payloads,
hash agreement and invariant text including negative zero. Prior imported/scalar/enum
cases run alongside them. All 24 executions (eight tests across interpreter, JIT and
native) pass with Raven `4b61e2737` on `codex/neoclr`, as does the source-core
missing-contract regression. [Execution evidence](floating-validation-20261021.json)
records compiler/runtime identities and results. The preceding gap audit retains the
original equality-synthesis rejection as a baseline. Generic record formatting and
record classes remain gaps; clean release-package qualification remains separate.
