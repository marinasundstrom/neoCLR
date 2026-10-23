# Native record union display — 2026-10-20

Extends [scalar record display](../record-formatting/README.md) to closed declared
union components with supported payloads. This is a native compiler change on
Raven `codex/neoclr`, not an ABI change or release-package qualification.

## Contract and comparison

Preserve Raven's existing generated union display: `Optional.Some(42)`, `Optional.None`,
`Choice.Pair(Left=7, Right=null)`, and `<Uninitialized>` when no case is active.
Single payloads omit their parameter name; multiple payloads use public property
names. Nested unions and supported source record structs reuse the enclosing builder.
Quoted/escaped text, null handling and original UTF-8 bytes retain the scalar contract.
Case checks and payload reads use public pattern/getter contracts, as equality/hash
already do; generated code does not inspect private discriminants or payload offsets.

The .NET comparison remains ordinary record-generated ToString and virtual dispatch,
with typed component calls. Raven unions have a language-specific display contract;
this change implements that existing syntax without introducing a runtime calling
convention or a generic CLR union API. Shared Raven/.NET union synthesis in
`SynthesizedMethodBodyFactory.Unions.cs` is the spelling reference. See the
[scalar review](../record-formatting/README.md#contract-and-implementation) for the
.NET primary source and the existing quoted-text divergence. The author's reaffirmed
[.NET runtime/ABI default](../../design-research.md) applies unchanged.

No per-component Object box or intermediate union string is introduced. Result,
scalar-text and builder storage still allocate; output retains the existing 65,536
UTF-8-byte limit. Inlining nested display increases generated code size, and pattern
checks add work. Allocation counts alone do not establish CPU or throughput parity.

## Limits

Unions must have declared cases, public matching payload getters and recursively
supported payload types. Enum/imported/floating-point payloads, generic record
owners, record classes and recursive display shapes remain gaps. ConsoleKeyInfo
still has an unsupported ConsoleKey enum component, so this slice does not add its
ToString override or alter public class-library signatures. General standalone
union ToString emission is outside this slice. Default carriers remain visibly
distinct from active empty cases; no semantic or storage ABI change is made.

## Evidence

[baseline.txt](baseline.txt) records rejection by the preceding scalar-formatting
compiler. [Program.rvn](Program.rvn) checks Some/None/default, nested options,
multiple payloads, nullable text, original decomposed Unicode and boxed dispatch.
[validation.json](validation.json) retains interpreter/JIT/AOT results, an unsupported
Optional<Index> diagnostic, exact commands and artifact hashes. The source-core probe
also retains existing native equality/hash and missing-format-helper regression
coverage: [compiler-probe.txt](compiler-probe.txt).

Compiler implementation: Raven `a0e62cdb6` on `codex/neoclr`.
The tested compiler is labeled `7062ed558-dirty-record-union-formatting`, built before
the commit. Fresh class libraries and a matched bundle use that compiler; runtime
and backend are reused from scalar formatting. [allocation-results.json](allocation-results.json)
compares 128 generated union displays with equivalent hand-written builder output.
Baseline uses 2 managed allocations; generated and manual workloads both use 770,
all reclaimed (six additional allocations per iteration). Collection counts are
8 / 9,224 / 8,328 respectively, so allocation parity is not a speed claim.
No HTTP or GC-throughput claim is inferred. Website prose is updated; no public
class-library signature changed, so the API snapshot is unchanged. No website
build or publication is performed for this compiler-feature prose update.

```sh
python3 docs/experiments/record-union-formatting/check.py target/record-union-format-bundle-v2/bundle target/record-union-format-app-v2 --output docs/experiments/record-union-formatting/validation.json
python3 docs/experiments/record-union-formatting/measure.py target/record-union-format-allocation-v2 --output docs/experiments/record-union-formatting/allocation-results.json
```
