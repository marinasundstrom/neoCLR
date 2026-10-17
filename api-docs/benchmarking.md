# Raven benchmark helpers

<a id="raven-benchmark-helpers-development"></a>

`NeoClr.Benchmarking` is source-included repository tooling, not a released runtime
library or installed benchmark SDK. Its types are outside the runtime reference
assembly/RavenDoc selection; this manual page covers that independent surface.
There is no dependency on `NeoClr.Testing`.

## Attributes

All three attributes derive from `System.Attribute` and target methods.

- `BenchmarkAttribute()` or `BenchmarkAttribute(description: string)` marks an
  accessible nongeneric module function returning the `int` correctness token.
  `Description: string { get; set; }` defaults to empty. Named Description is
  supported. The marker may occur once; it does not alter the qualified selector.
- Repeatable `ArgumentsAttribute` supplies one complete row per annotation.
  Constructors accept `(int)`, `(string)`, `(bool)`, `(int, int)`, `(int, int, int)`,
  `(string, string)` or `(string, int)`. Types and arity must match exactly. Null,
  general attribute arrays, Type values and arbitrary combinations remain gaps.
- Repeatable `ArgumentsSourceAttribute(methodName: string)` names an accessible
  parameterless nongeneric module function returning `T[]` for a benchmark taking
  one `T`. Unqualified names resolve in the benchmark's module; qualified names
  resolve in the same assembly. Sources must return 1–10,000 rows deterministically.
  Provider arguments, sequences, async providers and tuple spreading remain gaps.

These names follow [BenchmarkDotNet](https://benchmarkdotnet.org/articles/features/parameterization.html).
They are distinct from similarly named testing attributes. No TUnit or
BenchmarkDotNet library dependency is introduced.

## Discovery and worker

Metadata discovery emits typed Raven registrations without invoking attributes,
sources or benchmark bodies. Invalid signatures/data and duplicate rows fail and
remove stale outputs. Source templates are marked `dynamic: true` in the inventory.
Runtime registration calls sources once, capturing each input for reuse during
warmup and measurement. Mutable workloads must account for this reuse; reset and
lifecycle hooks are not implemented.

The assembly-internal `BenchmarkSuite.Add(name: string, operation: () -> int)`
registers a unique nonempty selector and callback. Each invocation must return `1`;
any other value or fault fails the process. Inline selectors append `(values)` and
source selectors append `/Qualified.Provider[index]` with zero-based indices.
Source row identities therefore depend on order.

`BenchmarkSuite.Run(arguments: string[])` takes an exact selector, warmup count
(0–10,000), and operation count (1–10,000). Unknown selectors or invalid counts fault.
Alternatively `--list` emits registered names between `NEOBENCH/1 CASES` and
`NEOBENCH/1 END`. The preparation tool uses listing outside measurement for source rows.
Normal worker output has READY, BEGIN, END and checksum lines. Registration precedes
READY, warmups precede BEGIN; the host measures external batch time, including pipe
transport and correctness checks. This is not internal operation timing.

See the [tested source](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/framework/Discovery.rvn)
and [runner configuration and limitations](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/framework/README.md).

See the [benchmarking project guide](https://github.com/marinasundstrom/neoCLR/blob/main/tools/benchmarking/README.md)
for preparation, runner commands, configuration and result interpretation.
