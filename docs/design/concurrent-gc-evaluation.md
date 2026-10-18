# Concurrent GC alternatives and evaluation

Author-directed investigation, 2026-10-18. **Design comparison, not shipped behavior.**
The author requested continuing through concurrent GC implementation, then explicitly
asked to learn from collectors beyond .NET without choosing one solely by precedent.
The current native collector remains synchronous. Interpreter collection uses its own
Rust heap; observable lifetime/API semantics must remain consistent, while collector
mechanisms need not match. The unfinished Promise result-storage experiment is parked.

## Primary sources and transferable ideas

| Approach | What the source establishes | Implication for neoCLR (our assessment) |
| --- | --- | --- |
| [.NET background GC](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/background-gc) | Background generation-2 collection can overlap application execution; foreground ephemeral collections still suspend it. | Preserve familiar blocking explicit Collect semantics. Generations and CLR internals are not prerequisites for our implementation. |
| [Go collector](https://go.dev/src/runtime/mgc.go), [write barrier](https://go.dev/src/runtime/mbarrier.go) | Concurrent, nonmoving mark/sweep uses mutation barriers, root coordination and allocation assists. Its hybrid barrier protects overwritten and introduced references. | A nonmoving concurrent tracer fits existing stable native addresses better than relocation. Audit stores, bulk copies and runtime roots before trusting concurrent reads. Assists can move latency back into allocation. |
| [Green Tea](https://go.dev/blog/greenteagc) | Page-based work queues and separate seen/scanned state batch nearby objects, improving locality; unfavorable sparse-page cases remain. [Go 1.26 notes](https://go.dev/doc/go1.26#runtime) confirm it became the default, beyond the original Go 1.25 experiment. | Evaluate page/region queues separately from concurrent correctness. Our variable-size arena and conservative object-word scanning differ from Go's size classes and pointer maps; importing vector scanning is not a drop-in change. |
| [Shenandoah developer documentation](https://wiki.openjdk.org/pages/viewpage.action?pageId=35127335) | Historical implementation documentation describes concurrent compaction and barrier costs, root-sensitive pauses and heap headroom requirements. This page is historical, not a current-version qualification. | Relocation would require updated roots, interior pointers and native borrows. A smaller pause is not sufficient if barriers or memory contention slow HTTP. |
| [LXR, PLDI 2022 extended paper](https://arxiv.org/html/2210.17175v1) | Research combines Immix block/line allocation, coalescing reference counting, occasional concurrent tracing of cycles and limited stop-the-world copying. | Consider region allocation and reducing total tracing as independent alternatives. Reference-count/remembered-set metadata and barrier coverage are substantial new machinery. Published workload wins do not predict neoCLR results. |
| [Opportunistic ZGC, September 2026 preprint v2](https://arxiv.org/html/2609.15558v2) | Research adjusts collection scheduling using CPU availability to trade unused compute for smaller heaps. This is a preprint policy experiment, not evidence that stock ZGC implements that policy. | Measure host saturation and worker CPU alongside pauses. Aggressively collecting while Serve already saturates its available CPU may worsen admissions. |

Sources reviewed on the entry date. Direct OpenJDK JEP pages could not be retrieved;
no claims about their current status are inferred from search snippets. The comparisons
above use accessible primary runtime sources, developer documentation and author papers.
The prior [.NET/Go research](../runtime-gc.md) remains relevant, not exclusive.

## Candidate boundaries

1. **Barrier-based, nonmoving concurrent tracing:** closest long-term candidate to
   our address-stable heap. Requires a complete reference-write contract across
   interpreter, native generation, arrays/value-record copies, helpers and root updates.
   C memory-model races must be solved as well as graph reachability: adding a log
   does not by itself make simultaneous ordinary reads/writes valid.
2. **Immutable snapshot tracing:** at an owner-thread safe point, copy heap state and
   root values; trace the copy in a worker while the original remains mutable. Apply
   reclamation only on the owner, retain new allocations, and prevent address reuse
   until the cycle is finished or discarded. This is a bounded experimental way to
   test actual overlap without missing barriers. It adds copying pauses proportional
   to snapshot size, scratch memory, floating garbage and worker coordination. It is
   not the barrier-based SATB technique merely because both preserve a beginning state.
3. **Concurrent sweep after synchronous marking:** simpler tracing correctness, but
   only helps the sweep fraction. Allocation/free-list ownership and resource cleanup
   still need coordination. Measure that fraction before adopting it.
4. **Locality, allocation and pacing changes:** region queues, faster free-space lookup,
   and fewer redundant cycles can reduce work even without concurrent execution.
   Compare independently to avoid attributing scheduling/policy changes to concurrency.

A native worker inside the runtime is the first implementation boundary to investigate:
it needs heap layout, published roots and shutdown privileges. A standalone FFI library
would still need those contracts, expanding the private ABI. A separate process adds
shared-memory and pointer/address coordination without establishing collector safety.
Public GC APIs need no speculative concurrent or generation switches for this experiment.

## Safety and measurement gate

Keep the synchronous full collector as oracle and exact explicit/pressure fallback.
Tests must cover changing and deleting edges during a cycle, new objects referring to
old objects, cycles, arrays and embedded references, parked frames, conditional owners,
host roots, interned/static/fault roots, native resource cleanup and invocation teardown.
Forced worker delay must demonstrate mutator progress before worker completion. Verify
that reclamation never runs user/resource callbacks on the worker. Bound scratch
memory and queued cycles; exercise overflow, thread-start failure and repeated entry.

For each candidate record snapshot/root pauses, publication/reclamation pauses, worker
CPU, application CPU, assists/waits, cycles, retained heap and scratch memory. Separate
instrumented correctness/diagnostics from sustained load. Include mutation-heavy graphs,
large stable live sets, sparse pointer chains, allocation churn and low-memory pressure.
Compare unchanged Serve and Go at fixed offered rates in reversed order, keeping every
admission miss and generator lag failure. Report total process CPU, including the worker;
state available CPU/core budgets so an extra collector core is not a hidden advantage.
Shorter GC pauses alone do not establish better request latency or a higher sustained rate.

## Current status

Stall traces are committed in `ca78fc42b`; they do not establish that GC execution is
responsible for every long wall interval. A local snapshot prototype is incomplete and
unvalidated. No collector design or new default has been selected. Continue with a
bounded executable concurrency experiment and correctness evidence before capacity
comparison or promotion. Review website behavior claims when implementation is validated.


### Subsequent author constraint: selectable memory management

The author requests separately includable implementations and activation of an included
memory manager, not necessarily GC. The immediate scope was subsequently narrowed to
[named GC selection](memory-management-providers.md), followed by a practical concurrent
proof of concept. General provider machinery is deferred. The unvalidated snapshot
scaffold stays isolated; build-time experimental macros alone do not satisfy runtime
selection.

### Snapshot proof-of-concept result

The isolated [executable experiment](../experiments/concurrent-gc-snapshot/README.md)
demonstrates owner progress during an outstanding worker job and exposes a retention
flaw: promoting conditional roots keeps an unreachable owner alive across repeated
cycles. The synchronous oracle expires it and cleans up its native resource once.
UBSan/bounds reproduces both observations; ThreadSanitizer is unavailable on this host
(even an empty control crashes). The prototype is not integrated or selectable.
Phase-cost measurements should guide the next practical offload experiment; this
result establishes no throughput improvement or complete concurrent-GC safety.


### Walker integration checkpoint

The [isolated worker/walker experiment](../experiments/concurrent-gc-walker/README.md)
now preserves conditional host-root pairs and matches synchronous collection on 101
protected-heap graph cases plus focused root/array/error checks. It reuses the current
walker and does not apply results to the live heap. Parked capture and mutation/handle
publication safety remain unresolved; this result is not qualification of concurrent
reclamation or a measured performance improvement.
