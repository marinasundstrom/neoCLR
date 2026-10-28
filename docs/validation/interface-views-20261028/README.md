# Reuse successful interface views within immutable metadata scopes - 2026-10-28

The preceding selection-scan profile identifies transitive interface traversal at 9.78% inclusive sampled cycles and assignability at 14.97%; these overlap. This experiment reuses successful transitive interface lists within the existing immutable module scope, rather than caching an assignment answer or method target.

## Contract and alternatives

The key is the complete structural owner type, including generic arguments and symbolic Self, within the exact borrowed module. Preserve traversal order, diamond deduplication, inheritance substitutions, array projections and errors. Only successful results are retained; callers receive their own vector. Errors still take the original traversal, and variance, method/signature, access and implementation validation still run in their existing callers.

The cache is thread-local, released with the outer immutable scope, and restored across nested modules or unwinding. Mutation between scopes builds fresh state. It accepts at most 1,024 owner keys and 4,096 top-level interface entries; these are entry limits, not a byte budget for recursively structured types. Reaching either cap leaves traversal as the fallback. No cache is populated outside a matching scope. This adds temporary storage and hashing/cloning work, so measurements must justify it.

CoreCLR stores directly and indirectly implemented interfaces in MethodTable interface maps and distinguishes instantiated interfaces for closed generic types. See the primary [MethodTable declaration](https://github.com/dotnet/runtime/blob/main/src/coreclr/vm/methodtable.h), reviewed 28 October 2026. The analogous design principle is sharing stable type relationships; neoCLR uses temporary immutable scopes rather than persistent loader-owned MethodTables. This changes no managed API or intended .NET assignment/boxing semantics. A permanent module cache would require a stronger mutation/lifetime contract; caching final assignability would also risk conflating validation and variant ambiguity. Both are deferred.

## Validation and measurement method

New differential cases compare scoped and unscoped traversal for constructed types, inherited diamonds, Self, arrays, missing types and cycles. They check nested-module isolation, mutation between scopes, successful/failed retention and cap fallback. Existing scope unwind/thread isolation tests exercise the same lifetime machinery. Runtime, interface/constraint/variance/reflection and codegen tests are recorded separately below.

Baseline is `67e111ab1`, including the preceding per-pass boxing scan. Reuse the exact hash-verified guest, Preview 18 libraries and sync/concurrent hosts from [that cohort](../selection-box-scan-20261028/README.md). Both variants use production-default codegen. Run three alternating pairs per eager/lazy and sync/concurrent mode, JSON pipeline, four clients, depth 16, 16 warmups and 4,096 measured responses. Keep memory diagnostics and CPU profiling separate from timing. No task-owned compiler or profiler overlaps timed runs; shared desktop activity remains uncontrolled. These measurements do not establish production capacity or a collector-default recommendation.

## Correctness results

All 209 codegen tests pass (35 manual probes ignored), as do 103 focused contract tests across interface inheritance/variance, class/default/explicit/boxed interfaces, constraints, Self and reflection hierarchy. The broader runtime run passes 379 tests with five ignored and one new fixture failure: it redeclared the assembler-provided Iterable type. Removing that duplicate declaration fixes the fixture; all seven lookup tests pass in the final rerun. The implementation did not change for this correction. Retain both logs rather than presenting the original full run as entirely green.

The previous lookup lifetime tests cover nested scopes, unwind restoration, mutation, generic arity and thread isolation. The new tests compare full closure results and diagnostics before/after caching, including successful empty lists and uncached errors. Managed method bodies, source validation and native publication contracts are unchanged.

## Short paired results

All 24 timed invocations pass (98,688 responses including warmup), following eight smoke invocations (384 responses). Every lazy process compiles 870 methods and leaves 811 uncompiled. Three-round medians:

| Mode / GC | READY before / after (s) | Selection + verification before / after (s) | Warm req/s before / after |
| --- | ---: | ---: | ---: |
| Eager / sync | 10.942 / 10.689 | 6.614 / 6.156 | 1,922 / 1,667 |
| Lazy / sync | 8.010 / 7.547 | 6.774 / 6.310 | 2,142 / 2,134 |
| Eager / concurrent | 13.429 / 13.131 | 6.639 / 6.239 | 1,747 / 1,779 |
| Lazy / concurrent | 7.875 / 7.831 | 6.504 / 6.285 | 1,510 / 1,465 |

Eight of twelve readiness pairs improve. Group medians improve 2.3%, 5.8%, 2.2% and 0.6%; the lazy/concurrent gain is particularly small and paired variation is substantial. Warm throughput is mixed. The 13.3% lower eager/sync group median triggers a separate longer comparison; no warm gain or universal startup gain is claimed.

## Memory diagnostics

All eight memory invocations pass (32,896 responses). Live malloc bytes and block counts match after invocation for every pair. These are allocator snapshots, not peak memory or managed-heap attribution.

| Mode / GC | Reserved after invocation before / after (MiB) |
| --- | ---: |
| Eager / sync | 956 / 972 |
| Lazy / sync | 872 / 864 |
| Eager / concurrent | 1048 / 1088 |
| Lazy / concurrent | 952 / 968 |

The cache trades temporary metadata storage for fewer traversals. Entry caps do not predict allocator reservation or prove lower peak RSS; memory-pressure qualification remains separate.

## Longer throughput check and CPU profile

Three additional eager/sync pairs each measure 65,536 requests (393,312 checked responses including warmup). Median throughput is 1,890.9 / 1,886.1 req/s (-0.25%). Paired differences are +1.68%, +0.84% and -0.25%; invocation CPU medians are 34.377 / 34.334 seconds. The short-run 13.3% regression is not reproduced. This does not prove exact throughput equivalence, and both cohorts remain visible.

Separate ten-second lazy/concurrent startup captures contain 18,019 baseline and 16,276 candidate samples. Interface closure inclusive share falls 9.35% / 2.56%; the underlying traversal is 8.01% / 0.41%, and assignability is 14.06% / 8.89%. Shares overlap and capture durations are not equal-work phase measurements. Remaining self samples include hashing (6.40%), type equality (5.15%), allocator free (5.02%) and dispatch target discovery (4.68%).

Retain the bounded cache: semantic differential/contract tests pass, preparation medians improve modestly, traversal samples fall, post-invocation live allocations match, and the longer comparison does not reproduce the short throughput gap. No warm-throughput, general memory or special concurrent-GC advantage is claimed. Total HTTP validation is 525,280 checked responses across short, smoke, memory and longer runs.

## Reproduction and continuation

The source and result archives are hashed in `evidence-index.json`. Rebuild the release codegen runner at `67e111ab1`, retain it as `baseline-runner`, then apply the candidate patch and build `indexed-runner`. Use the Xcode SDK in `environment.json`. `measure.py`, `long-command.json` and `long-measure.py` record the local commands. The unchanged guest/library metadata and native host reconstruction are preserved in the preceding selection-scan evidence; verify shared input hashes before reuse. CPU-only trace tables are archived; full traces remain local. Script paths under `/tmp` are provenance and must be adjusted when replaying elsewhere.

The author subsequently directed exploration of Go-inspired lightweight execution contexts, worker queues, work stealing, parallelism and suspension without blocked workers. That becomes the next author-directed investigation, rather than another metadata optimization by default. Adoption remains conditional on measured benefit and complexity. Preserve the lazy-JIT and sync/concurrent GC comparisons, and resolve the existing one-mutator-per-heap constraint before claiming parallel managed execution.

All 26 website contract tests and the 6,137-page build pass, with existing API-description warnings. Updated tables await the next manual publication; no deployment or push was performed.
