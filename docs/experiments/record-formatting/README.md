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
