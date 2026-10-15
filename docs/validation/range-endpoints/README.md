# Native range endpoint carry — 2026-10-15

Raven main `c16fca24c` is carried deliberately to `codex/neoclr` as `bf5e0abe8`.
The branches remain separate. Structural-type work is parked.

## Qualified behavior

- `..` excludes the endpoint; `..=` includes it in supported native integer loops.
  The consumer checks ascending and descending traversal and stored half-open bounds.
- Existing `System.Index`/`System.Range` library ABI and metadata are unchanged.
- Native inclusive stored/sliced Range construction rejects with NEOMETA001 and
  a checked-Index-conversion explanation, rather than dropping inclusion.
- Shared range/syntax/portable-contract selection: 169 passing tests on .NET 11.
  Generator/build and native-enabled compiler builds pass.
- Fresh System.Runtime, System.Data, System.Networking, System.Web and
  System.Data.PostgreSql libraries build from migrated sources.
- [Consumer source](Main.rvn) returns 42 in interpreter, macOS ARM64 AOT and managed
  JIT. AOT runs with undefined/bounds sanitizers. JIT uses the existing native GC
  host infrastructure; it is not sanitizer-instrumented. This consumer allocates
  no arena storage and makes no throughput claim.

[Execution evidence](results.json) records commands and binary/source fingerprints.
The library build report is retained at
`target/range-endpoints-libraries-20261015-final/build-evidence.json`; its declared
working-tree compiler was subsequently committed as `bf5e0abe8`. The native project
and executable outputs are in `target/range-endpoints-consumer-20261015`.
To rebuild the consumer, create a normal executable Raven project importing the
fresh library directory's `NeoCLR.ClassLibrary.props`, include Main.rvn and run
`dotnet <native-enabled rvnc.dll> neoclr --project <project.rvnproj>`.
The execution commands in the evidence specify the matching seed and modules.

## Source migration

The syntax-aware migration reads pre-change sources: former `..<` becomes `..`,
while inclusive loops/comprehensions/patterns become `..=`. Stored half-open values,
slicing, sequence segments/rest patterns, spreads and strings preserve their meanings.
Active library, test, benchmark and experiment consumers are updated. Archived
`benchmarks/framework/data-evidence/Registry.rvn` remains unchanged as historical
benchmark evidence. The fresh library build and bounded consumer are the executable
qualification; this does not claim every historical experiment was rerun.

## Remaining work

1. Implement native checked inclusive Index conversion with once-only source-order
   evaluation and native Fault transport. Keep negative distances, `^0`, `^1` and
   integer overflow consistent with the shared language rule.
2. Support native range patterns: current lowering produces BoundAndPattern and
   BoundComparisonPattern, which the portable backend rejects explicitly.
3. Implement overflow-safe integer advancement and then generic ordering/stepping
   capabilities. Existing loop arithmetic at integer limits is not qualified here.

The broader compiler selection exposed an independent readonly-field defensive-copy
failure (3 rather than 2). The same source reproduces on Raven main `c16fca24c`;
track it as a general main-first candidate, not as passing range evidence. The initial
fixture migration also accidentally touched a spread; that fixture was restored
before the final passing selection. A stale .NET 10 output was rebuilt explicitly
before native rejection qualification. Native evidence is macOS ARM64 only.
