# Concurrent GC optimization workload inventory (2026-10-25)

Author direction: optimize request throughput, CPU and allocations together, using
relevant ordinary samples and existing/new profiling tools, not just HTTP handlers.
The [required matrix](../concurrent-gc-investigation/README.md#required-workload-matrix)
remains authoritative; this inventory connects the next work to existing consumers.
These are execution/profiling starting points, not claims of complete coverage.

| Family | Existing source or harness | Current candidate evidence / next action |
| --- | --- | --- |
| Plaintext HTTP | Full retained primitive-comparison object; `measure-plaintext.py --sample plain` | Live JIT/AOT correctness and matched cache controls pass; saturated throughput and worker scaling remain open. |
| Reflection JSON HTTP | `benchmarks/http-server/primitive-comparison/transient-handoff/JsonServer.rvn` | Live JIT/AOT correctness and matched cache controls pass; keep parser/serializer and transport costs distinguishable. |
| Tasks and collections | `docs/experiments/raven-target/samples/library-tasks.rvn`, `application-order-collections.rvn`; retained wrapped artifacts in gc-mode-comparison | Fresh live JIT output/cleanup checks pass. Each starts one epoch and cancels at shutdown; batch sustained work before claiming GC performance. |
| Arrays and callbacks | Same sample directory: `library-arrays.rvn`, `library-array-tour.rvn`, `library-array-queries.rvn`, `library-array-callbacks.rvn` | Select representative element layouts, mutation and callback paths, rebuild against a coherent bundle, then qualify outputs and sustained JIT/AOT execution. |
| Strings and UTF-8 | `library-strings.rvn`, `library-grapheme-strings.rvn` and expected output | Separate allocation-heavy transformations from comparison/read paths; measure both short-lived and retained data. |
| Reflection/metadata | `library-reflection.rvn`, `library-managed-array-metadata.rvn`; bottleneck-review startup tools | Keep metadata load/selection/JIT compilation distinct from managed execution and GC; qualify current compiled consumers first. |
| Async and failures | `library-async.rvn`, `library-async-default-queue.rvn`, `library-async-cancellation.rvn`, `library-task-propagation.rvn`; state-integration tools | Extend existing lifecycle checks into sustained suspension, cancellation and cleanup profiles without dropping root/owner invariants. |
| Standalone JSON DOM/stream/composition | Transient-handoff corpus and its documented JSON admission gap | Rebuild/repair the current consumer coherently; report rejection as a coverage gap, never as zero cost. |
| Object graphs/retention | concurrent-live lifetime/native GC fixtures; `examples/class_construction.neoil` | Native differential and mutation checks pass with/without the cache. Add managed sustained construction, survival and mutation work; native protocol checks are not application throughput measurements. |

Available profiling entry points include `gc-mode-comparison/run.py`,
`lookup-profile/corpus.py`, `state-integration/counts.py`, the bottleneck-review
startup/diagnostic tools and concurrent pause/phase instrumentation. Reuse their
counter definitions and output checks, but rebuild hosts with the qualified live
providers and owner-thread reporting. Do not mix ordinary unguarded generated code
with a live collector. Keep the synchronous collector as a separate comparison.

For each selected workload, first establish output and cleanup equivalence, then
batch enough managed work to complete collections and separate execution from
startup. Preserve workload semantics and label any console-output suppression or
other benchmark adaptation. Capture work units, allocations/counts/bytes, CPU,
retained/peak memory, completed/cancelled epochs and outer pause distributions.
Check whether profiling overhead changes throughput; do not infer GC pauses from
client latency or raw collection-core maxima. Use repeatable quiet runs for capacity
claims, with compilation complete before measured execution.

The current allocation totals are unchanged by the range cache. Its improvement is
lookup/CPU work, not an allocation improvement. Remaining out-of-arena store calls,
repeated publications and metadata work should be assessed across these families
before choosing the next general runtime optimization.

Follow-up: [sustained ordinary measurements](../concurrent-workloads/README.md) now
cover tasks, collections and array tour with matched sync/live AOT allocation and
CPU evidence. The eight-sample build/pilot matrix retains shared helper gaps and
a cancellation batching failure; those workloads are not performance-qualified.
