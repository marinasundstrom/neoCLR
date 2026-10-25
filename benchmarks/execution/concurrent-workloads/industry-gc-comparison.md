# Industry GC mechanisms and neoCLR priorities

2026-10-25. Author-directed review of primary sources, read on this date. These are
design lessons and proposed priorities, not implemented neoCLR features or promises
to reproduce other runtimes' performance. The concurrent-default objective remains.

## Shipped mechanisms

**.NET:** CoreCLR uses thread-affine allocation contexts to avoid common-path locks,
and generations to collect only part of the heap. Its design note dates from 2015;
numerical sizing examples are not current tuning promises.
[CoreCLR GC design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/garbage-collection.md).
Background generation-2 collection coexists with synchronous generation-0/1
collections. Concurrency does not replace cheap young collections.
[Background GC](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/background-gc).
DATAS adjusts allocation budgets and heap count to workload size and throughput,
making the memory/CPU tradeoff explicit.
[DATAS](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/datas).

**Go:** The runtime describes precise, non-generational, non-compacting concurrent
mark/sweep, with size-segregated per-P allocation. Sweeping proceeds by span in the
background or in response to allocation, instead of requiring a whole-heap owner
sweep at each mark termination. [Runtime algorithm](https://go.dev/src/runtime/mgc.go).
Its guide explains collection frequency as a CPU/memory tradeoff; the conceptual
guide identifies its description as Go 1.19-era, not a complete account of newer
scanning internals. [GC guide](https://go.dev/doc/gc-guide).
Green Tea queues pages and tracks seen/scanned objects, accumulating nearby work for
memory-order scanning. Better locality and smaller worklists reduce overhead;
sparse workloads can benefit less. [Green Tea design](https://go.dev/blog/greenteagc).
Go 1.26 enabled it by default. Reported GC-overhead reductions are not equivalent
HTTP-throughput gains; newer x86 vector results do not prove gains on our ARM64 host.
[Go 1.26 release notes](https://go.dev/doc/go1.26).

**Java G1:** Regions, generational evacuation and remembered sets limit collection
scope. SATB preserves a logical snapshot using barriers; it does not require copying
every object header. The tradeoffs include barrier/metadata costs and floating garbage.
[G1 design](https://www.oracle.com/java/technologies/g1gc.html).
Adaptive IHOP uses observed marking duration and allocation to time collection starts.
[Current G1 guide](https://docs.oracle.com/en/java/javase/26/gctuning/garbage-first-g1-garbage-collector1.html).
JDK 26 also ships reduced application/collector synchronization; this review verifies
the shipped feature, not its exact barrier implementation.
[JDK 26 release notes](https://www.oracle.com/java/technologies/javase/26-relnote-issues.html).

**Java ZGC:** Oracle's JDK 26 guide describes generational low-latency collection with
adaptive generations and GC thread counts. Concurrent work requires memory headroom
for ongoing allocation. Low pauses do not establish lowest CPU or highest throughput.
[ZGC, chapter 9](https://docs.oracle.com/en/java/javase/26/gctuning/hotspot-virtual-machine-garbage-collection-tuning-guide.pdf).
Detailed barrier/relocation adoption is outside this review: direct JEP retrieval
was unavailable, and secondary accounts are not treated as specifications.

## What our evidence says

The [begin/marker profile](begin-phases-results.md) validates 6,240 HTTP responses
and nine standalone runs. Plaintext starts about 0.47 epochs/response and JSON about
0.92 in these small-heap fixtures. Begin/marker CPU is respectively 11.7/17.8 ms and
24.0/31.1 ms per 1,040 responses. Capture/dispatch consumes over half of begin CPU.
We should test whether this frequency and metadata traffic are necessary.

Generated collector inspection shows capture allocating snapshot/gate/header/offset/
pending/mark/expected storage and a sparse index, then joining before disposal.
Native allocation volume has not yet been measured. Header identity checks currently
provide safety: removing snapshots without replacing that proof is incorrect.
A logical SATB snapshot also does not by itself legalize racy C/Rust payload reads.

The [owner profile](stable-byref-pause-results.md) attributes roughly 93% of JSON
finish CPU to final collection. This makes reclamation structure relevant, not just
additional marker threads. Go's span sweeping is a useful comparison, but adapting
it requires explicit sweep/allocation ownership and safe free-list publication.

## Proposed sequence, not a new implementation contract

1. **Pacing with a memory budget.** Compare current triggers with live-size,
   allocation-rate and measured-cost budgets; isolate idle epoch starts. Preserve
   allocation-pressure recovery and shutdown. Report CPU/request, epochs/request,
   throughput, p99 and RSS/arena peaks together; do not hide a larger-heap tradeoff.
2. **Persistent metadata and cheaper capture.** Measure native allocation volume,
   then evaluate reusable owner-held snapshot buffers and stable block/region indexes.
   Reset only after marker join, cap retained capacity, and test growth, shrinkage,
   cancellation, reuse and exhaustion. This is our inference, not a claim that other
   collectors use neoCLR's copied-header representation.
3. **Reduce finish work structurally.** Explore incremental/lazy sweep over owned
   spans and avoid repeated index/identity work only with a lifetime proof. Compare
   with existing owner-thread reclamation. Keep direct seeding optional: prior weak
   whole-workload gains do not become strong merely because its phase is prominent.
4. **Precise tracing and locality.** Profile pointer/scalar filtering and marker
   queue locality, then consider page/span batches and shared layout descriptions
   across JIT, AOT and interpreter paths. Use generic element/layout contracts,
   not record-specific semantics. Bound batch work to preserve latency.
5. **Generations remain on the table.** Measure survival/lifetimes and design complete
   old-to-young tracking across compiled, interpreted, native, array and bulk-copy
   writes. Moving young objects additionally requires relocation-aware roots and
   interior pointers; current cached byrefs assume nonmoving collection. Compare a
   nonmoving generational alternative, including fragmentation/allocation costs.

.NET remains the public semantic reference; Go is especially relevant internally
because it supports nonmoving concurrent collection. Shared-heap participant,
barrier and safepoint requirements remain mandatory. This does not prioritize a
new thread pool or compaction ahead of measured pacing/capture costs.

No cross-language throughput ranking is established. Match hardware/core limits,
HTTP semantics and memory budgets, retain synchronous controls, and include plaintext,
JSON, standalone lifetimes and larger retained graphs. Measure native allocation,
owner/marker CPU, pauses and lookup/scan work. The near-term recommendation is pacing
and metadata lifetime/capture, followed by reclamation structure; generations remain
an explicit design direction rather than an abandoned option.
