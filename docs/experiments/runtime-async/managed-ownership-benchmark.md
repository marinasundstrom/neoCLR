# Managed runtime-async ownership benchmark — 2026-10-11

Runtime async now uses the returned Task directly as the conditional owner of its
producer. This removes the per-body lifetime notification that caused a severe
completed-await regression in the first managed ownership integration. The generator
and HTTP/await lifetime suites pass with this correction.

## Matched results

These are compiler-state-machine versus retained-runtime lowerings **on neoCLR**,
not measurements of the .NET runtime. Apple M1, macOS ARM64, release hosts; each
case checks its result and fully tears down its fresh invocation. Loading and
compilation are excluded from the reported execution time. Two rounds reverse
lowering/engine order, each with four invocations and the first discarded as warmup:
six retained timing samples per cell. No CPU isolation or statistical performance
guarantee is claimed.

100 operations; median milliseconds (lower is better):

| Await state | Engine | Compiler state machine | Runtime async | Runtime / state machine |
| --- | --- | ---: | ---: | ---: |
| completed | interpreter | 1226.53 | 1119.53 | 0.91× |
| completed | jit | 15.44 | 24.13 | 1.56× |
| completed | aot | 16.10 | 24.97 | 1.55× |
| pending | interpreter | 2850.44 | 2037.22 | 0.71× |
| pending | jit | 34.20 | 61.75 | 1.81× |
| pending | aot | 32.23 | 58.35 | 1.81× |

[Full results, ranges, 10-operation controls and input hashes](task-owned-benchmark.json).
[Initial callback-adapter results](managed-ownership-callback-benchmark.json) are
preserved rather than overwritten. The initial completed-100 native ratio was
69–81×; direct Task ownership reduces it to about 1.55× in this matched run.
Interpreter runtime async is about 9% faster for completed-100 and 29% faster for
pending-100; native pending-100 remains about 1.8× slower. This closes the large
notification-retention regression, not the native performance/default gate.

## Reclamation and scope

Every interpreter invocation finishes with zero live managed objects. Every native
invocation reclaims all allocated GC blocks and releases all reserved stack mappings.
Native blocks and interpreter object counts are different units and must not be
compared directly. The native stack cache remains limited to eight mappings during
an invocation and is emptied at teardown. Stack high-water marks are per-process;
mapping/resumption counters are per-invocation deltas.

Runtime-async allocation counts at 100 operations:

| Await state | Engine | Allocated / reclaimed | Unit |
| --- | --- | ---: | --- |
| completed | interpreter | 3314 | managed objects |
| completed | jit | 3917 | native GC blocks |
| completed | aot | 3917 | native GC blocks |
| pending | interpreter | 5414 | managed objects |
| pending | jit | 6617 | native GC blocks |
| pending | aot | 6617 | native GC blocks |

The [generator suite](task-owned-generator-validation.json) separately qualifies
160 abandoned yielded generators, live task-only consumers, external completion,
cleanup and generic/delegated production in all three modes. The
[HTTP/await showcase](task-owned-showcase-validation.json) checks ordinary await,
HTTP client/server, timeout/cancellation/protocol handling and terminal Fault.
The benchmark above measures nested Task awaits; it is not generator-throughput,
HTTP/JSON throughput, multicore fairness or a comparison with Go/.NET/JavaScript.

## Subsequent profiling and optimization

[Indexed root-validation measurements](native-profiling.md#indexed-validation-result)
reduce native completed/pending ratios to about 1.28–1.29× / 1.46–1.48×
with unchanged collection frequency. The tables above remain the pre-optimization
checkpoint. The native performance/default gate remains open.

## Next performance gate

Profile remaining native carrier-entry/resumption and collector costs before
changing defaults. Native collection still occurs at published safepoints in this
profile; these results do not isolate GC policy from suspension overhead. Keep the
completed path as a regression control, preserve the compiler-state-machine baseline,
and add generator/HTTP workloads before generalizing. Windows runtime-async carrier
qualification and production operation metadata remain open.

Reproduce with `benchmark_modes.py`, using the pinned source recipe, qualified
native host recipe and release `runtime_async_interpreter` example recorded in
`task-owned-benchmark.json`. Exact commands, compiler/library hashes and artifact
hashes are retained there.
