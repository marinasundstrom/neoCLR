# Generational concurrent GC — design input, 2026-10-25

Status: exploratory architecture, not implemented behavior or a replacement for
the active measured optimization work. The author supplied an external ChatGPT
recommendation for thread-local allocation buffers (TLABs), copying young
collections, concurrent old-generation marking, card barriers and safepoints.
The existing intention to qualify concurrent GC as the default remains; this input
does not establish that every proposed mechanism is necessary or faster.

## Industry baseline

[CoreCLR's GC design at .NET 10.0.0](https://github.com/dotnet/runtime/blob/v10.0.0/docs/design/coreclr/botr/garbage-collection.md)
separates allocation from collection. Thread allocation contexts allow allocations
without locking until refill, with explicit zeroing and heap-walkability rules.
[.NET background GC](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/background-gc)
performs generation-2 work in the background while allowing foreground young
collections that suspend managed threads. These are implementation precedents,
not new public API semantics or proof that identical policies suit neoCLR.
Sources retrieved 2026-10-25; the design document describes historical details too,
so its example buffer sizes are not adopted as tuning constants.

## Mechanisms and prerequisites

| Mechanism | Potential benefit | Cost and required evidence |
| --- | --- | --- |
| Per-worker allocation regions | Amortize allocation synchronization and bookkeeping | Refill/publication protocol, unused tails, thread exit, pressure accounting, collector visibility of partly initialized objects; measure refills and CPU/allocation |
| Young-generation collection | Avoid repeatedly tracing long-lived survivors | Old-to-young remembered sets, promotion and survival policy; first measure survival and bytes scanned/reclaimed by age |
| Copying nursery | Cheap reclamation of mostly dead regions and locality | Exact writable references or explicit pinning/handles, interior references, suspended/native roots, identity and forwarding; validate relocation before performance claims |
| Concurrent old marking | Overlap long-lived tracing with application work | Correct marking barriers, termination and allocation visibility, CPU and memory headroom; retain current mutation/liveness controls |
| Parallel young collection | Reduce pause duration for larger young heaps | Work distribution and synchronization can outweigh tracing on small heaps; qualify serial first, then compare scaling |

Card marking and concurrent-marking barriers have different obligations. A dirty
card can identify possible old-to-young references, but is not automatically a
replacement for the current deletion/snapshot protocol. A combined barrier must
preserve both invariants, including bulk copies, arrays and native writes.

TLABs also do not by themselves permit concurrent execution. The current
[requester-local allocator](../../benchmarks/gc-suite/concurrency/local-allocation.md)
removes a service round trip while retaining the exclusive execution lease and
pool gate. It is not a TLAB implementation. Independent HTTP heap owners are a
different architecture from shared-heap participants; neither result proves the
other scales. Allocation regions could first reduce bookkeeping under existing
ownership, but multicore allocation claims require safe concurrent mutators.

## Bounded progression and alternatives

1. Finish attributing runtime root/queue/resumption and allocation/boundary work.
   The [fiber profile](../../benchmarks/gc-suite/concurrency/fiber-results.md)
   places raw transfers at roughly 1–5.5% of measured invocation CPU. It does not
   establish the cost of surrounding services. Preserve this baseline.
2. Measure survival, retained graph size, scanning productivity, allocation/refill
   costs and pause distribution across the existing non-HTTP and HTTP samples.
   Few allocations with frequent writes must remain in the matrix: generations
   cannot be assumed to solve the observed store-admission overhead.
3. Prototype allocation regions separately from a generation policy. Keep a
   synchronized refill/collection slow path and explicit publication rules; test
   pressure, thread exit, interrupted initialization and cross-worker references.
4. Compare a nonmoving young-generation experiment with a copying nursery after
   the relocation inventory is qualified. Nonmoving generations avoid relocation
   initially but still need remembered sets and may retain fragmentation costs.
   Retaining the current nonmoving collector is the control, not a failed option.
5. Combine qualified mechanisms with existing concurrent marking. Add parallelism
   where measured pause work warrants it; defer concurrent relocation until a
   demonstrated fragmentation or pause problem justifies that complexity.

All stages compare each collector with its own baseline and include sync controls,
JIT/AOT correctness, interpreter applicability, throughput, total CPU, memory and
pauses. No equal-memory or equal-worker assumption is required, but configurations
must be disclosed. Generations are not a prerequisite for continuing the present
concurrent collector improvements. “Generational concurrent GC” is an accurate
target label only with an explicit proposed qualifier until generations exist.

This extends the existing [relocation prerequisites](allocation-metadata-runtime-comparison.md)
without claiming an implemented runtime change, a throughput advantage or default
qualification. No public API, compiler bridge or website capability changes arise
from this research note.
