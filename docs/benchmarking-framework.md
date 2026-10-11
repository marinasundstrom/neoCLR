# Benchmarking framework direction

Author direction, 2026-10-11: some benchmarks should run outside the process, and
neoCLR should develop its own BenchmarkDotNet equivalent. This is an implementation
direction; the contracts below are provisional, not shipped APIs. Complete the
current compiler fix and execution-mode measurements before expanding the framework.

## Reference and separation of responsibilities

BenchmarkDotNet generates an isolated project for runtime settings and launches
benchmark processes. Inside those processes, it distinguishes operations from
iterations and has pilot and warmup stages. Its public model includes benchmark
attributes, jobs, parameters and result exporters. See the primary references:
[execution model](https://benchmarkdotnet.org/articles/guides/how-it-works.html) and
[public overview](https://benchmarkdotnet.org/articles/overview.html), read 2026-10-11.

Process isolation and in-process timing are compatible: an external controller can
launch an isolated worker that times repeated operations internally. Use that for
serialization and collection microbenchmarks. Measure process startup externally,
and use external peers for HTTP latency/throughput. Do not subtract startup from
an unrelated process total to infer individual operation costs.

The proposed Raven contract should feel familiar: benchmark attributes and optional
setup/cleanup, explicit jobs and parameters, filters, raw measurements and summaries.
Like the test framework, support module functions and manual registration; grouping
in classes should be optional. Attribute discovery must work with explicit retention
under AOT; generated registration can follow later. These are proposals, not final
attribute/type names or an API-compatibility promise with BenchmarkDotNet.

## First implementation slices

1. Keep build/launch, transport and measurement protocols separate. Extract the
   current external HTTP controller into a reusable runner once the JSON workload
   defines the second consumer. Each mode is a separate job/process with explicit
   interpreter, JIT or AOT selection and no silent fallback.
2. Provide a Raven worker for registered synchronous/async operations, setup,
   correctness validation and observable result consumption. First validate an
   appropriate monotonic, high-resolution clock contract across all modes; wall
   clock time is not a substitute. Keep console output outside timed iterations.
3. Add explicit warmup/iteration counts, multiple process launches, parameters and
   raw JSON export. Report startup and operation timing separately, include sample
   counts and dispersion, and record compiler/runtime/source/artifact provenance.
4. Add attribute discovery/filtering and validate with node and typed JSON workloads,
   including nested objects, arrays, lists/sequences and string-keyed maps. Measure
   serialization and deserialization separately with equivalent input data.
5. Add calibrated operation counts and overhead investigation, then allocation/GC
   diagnostics only when supported by reliable runtime counters. Do not invent
   allocation figures, confidence intervals or automatic stability guarantees.

A fixed-count first version is simpler to inspect but less adaptive than
BenchmarkDotNet; short operations may be dominated by clock/harness overhead.
External orchestration gives isolation and cross-mode comparability at the cost of
process startup and deployment complexity. Record both costs instead of claiming
an improvement. Retain all raw samples and flag contention; repeat noisy runs on a
quiet machine before publication. Tests must cover failure/cleanup, selection,
iteration accounting and output integrity as well as measured workload correctness.

## Current evidence

The [HTTP workbench](../benchmarks/jit-http/README.md) implements external process
orchestration, fresh builds, warmup, repeated launches, correctness checks, mode
rotation and raw timing/provenance output. It is development tooling, not yet a
Raven benchmarking library. Its first run observed background CPU contention.
The [initial worker and runner](../benchmarks/framework/README.md) now provide manual
synchronous registration, exact-name filtering, bounded counts, isolated jobs,
protocol/checksum validation, input hashes and raw JSON reports. HTTP shares the
process transport. The [JSON node workload](../benchmarks/json/README.md) separates
serialization and deserialization of a nested Unicode document with arrays/nulls.
This first version measures external batch latency, including transport and
correctness checks; it does not implement internal operation timing.

Attribute-driven module-function discovery now uses native metadata introspection
and generates typed registrations for AOT reachability. Final assembly rediscovery
checks the generated contract; manual registration remains available. The current
worker signature remains parameterless `int`, with token `1` for success.

Typed JSON objects and collection mapping remain next, followed by a qualified
monotonic clock and internal measurement, async callbacks and setup/cleanup. These are still gaps relative to BenchmarkDotNet, not new
permanent semantics. This work stays within the benchmark gate before schedulers
and runtime-managed suspension.
