# Managed garbage collection

Garbage collection is part of NeoCLR's normal managed execution model. Its main
improvement over CLR is explicit value versus reference semantics, with checked
reference lifetimes; programmers do not take responsibility for managed memory
merely by choosing references. Ordinary values copy their contents. Explicit managed
heap allocation provides shared identity and automatic memory management.
Developers choose scoped value storage for method/block-local work or managed heap
storage when an object needs an independent lifetime. Either permits references,
subject to the owner's lifetime; forming a reference does not promote a local. The
prototype currently enforces frames, with lexical block lifetime enforcement pending.

The prototype now implements a single-threaded, nonmoving mark-and-sweep collector.
It traces the existing heap.new/Ref encoding; that encoding remains transitional.
The selected public direction is T&/ByRef for both frame-backed references and
managed heap references, without an ownership wrapper. Heap-backed ByRef, interior
heap references and construction directly into heap destinations remain future work.

## Roots and collection boundaries

Before a heap.new instruction reaches an allocation threshold, the runtime traces
all initialized arguments, locals and evaluation stacks in every active frame.
The pending allocation operand is still on the stack at this point. Inline record
fields and System.Value payloads are traversed, and heap references are followed
transitively. Cycles are supported: reachable cycles survive and unreachable cycles
are reclaimed. Collection does not require guest retain/release or invalidation.

Current managed slot references always address active frame cells, which are scanned
independently; they cannot be hidden in heap fields or erased values. Extending ByRef
to heap storage must add owner-root tracing, including for interior references and
interface receivers. A heap field reference must keep its entire allocation alive.
The runtime's prohibition on returning an address into the current frame remains.

Collection runs between instructions, before allocation operands are popped. The
initial threshold is the smaller of 64 objects and the heap_objects limit. After
collection it becomes twice the survivor count, with a minimum of 64 and a maximum
of that limit. The limit is checked after collection and bounds live object count,
not lifetime allocation count or allocated bytes. A live local remains a root until
replacement or frame exit; this prototype has no compiler-derived last-use maps.

On successful execution completion, another collection preserves only objects
reachable from the returned value. Execution.heap is a read-only ManagedHeap
snapshot: get(identity) resolves live objects, len() counts them, and collection and
reclamation counters are available. Identities increase monotonically and are never
reused within an execution; gaps are expected. Counter exhaustion faults. References
are meaningful only within their execution; persistent host root registration and
passing heap references between executions are not implemented.

Fault or cancellation drops execution storage without running guest callbacks.
Native pointer allocations are tracked separately and are not traced as managed
references. Existing heap.alloc/free and pointer operations provide the explicit
low-level path for native interop. Copying a raw pointer does not keep managed
storage alive; a future managed/native bridge needs explicit pinning or handles.

## Monitoring the collector

Run `cargo run -- run examples/features.neoil --gc-stats` for an end-of-execution
report on stderr, separate from the program's normal stdout:

```text
GC: allocated=1 live=0 peak=1 collections=1 reclaimed=1
```

Hosts can read `execution.heap.statistics()` for a typed `GcStatistics` snapshot:

| Counter | Meaning |
| --- | --- |
| allocated_objects | Total successful managed heap allocations in this execution |
| live_objects | Objects remaining in the heap after final collection |
| peak_objects | Maximum objects resident at once, including garbage awaiting collection |
| collections | Completed collections, including final collection even if the heap was empty |
| reclaimed_objects | Total objects removed by collection |

Counts exclude inline values, host buffers and native pointer allocations; they are
not byte measurements. Each execution starts fresh counters. The initial report is
available on successful completion, not continuously during execution or after a
terminal Fault. It does not invoke extra collections or alter guest output.

The next monitoring layer should provide collection events with reasons, before/after
counts and pause durations including root scanning; allocation rates and byte counts
require a defined managed storage-size model. Hosts will also need snapshots during
long-running programs and diagnostics on Fault/cancellation. Define bounded buffering
and observer behavior before callbacks are permitted around GC safepoints. Later
heap graph/root inspection can explain why particular allocations remain reachable.

## Cleanup and future work

Becoming unreachable makes heap storage eligible for collection; it does not promise
immediate reclamation at the last reference loss. Dispose and Close remain the
protocols for timely external-resource cleanup. Frame-owned value cleanup is a
separate lifetime contract. Guest destructors, finalizers, resurrection and automatic
scope cleanup are not implemented; the collector does not invoke Dispose or Close.

CLR likewise traces roots to find reachable objects and automatically collects
unreachable storage; see Microsoft's [GC fundamentals](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/fundamentals).
This initial collector does not implement CLR's generational or compacting policies.
Moving collection, weak references, concurrent execution, static/host roots, native
pinning and finalization need separate contracts and tests before being exposed.
These are collector improvements, not prerequisites for ordinary managed allocation.

Next, unify heap roots with T& while preserving checked frame provenance, then settle
explicit heap construction using familiar MSIL initialization/construction patterns.
See the [managed heap strategy](managed-heap-strategy.md). Changing an instruction's
result from Ref<T> to T& requires a coordinated preview format transition.
