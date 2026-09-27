# Garbage collection

**Development API.** `System.Runtime.GC` describes and influences the managed heap
for the current execution. Use matching development runtime and compiler artifacts.

The runtime uses a non-moving tracing collector. Collection happens automatically
under allocation pressure and at execution completion; applications can also request
one synchronous full collection.

```raven
import System.Runtime.GC

let collections = GC.CollectionCount
GC.Collect()
Check(GC.CollectionCount == collections + 1)
```

This excerpt is checked in the executable
[GC consumer](https://github.com/marinasundstrom/neoCLR/tree/main/docs/experiments/runtime-gc).
`Check` faults when its Boolean argument is false. `Collect()` preserves reachable
objects, including scheduler state and objects under construction. It does not change
host limits or promise an operating-system memory reduction.

| Read-only long property | Meaning |
| --- | --- |
| `CollectionCount` | Completed collections in this execution |
| `AllocatedObjectCount` | Tracked objects allocated since execution began |
| `HeapObjectCount` | Retained objects, including garbage not yet collected |
| `PeakHeapObjectCount` | Highest retained object count observed |
| `ReclaimedObjectCount` | Objects reclaimed by completed collections |
| `HeapObjectLimit` | Host-configured object limit |

Queries do not trigger collection or allocate tracked objects. Counts exclude inline
values, native memory and untracked string buffers; they are not byte measurements.
Each isolated worker has its own heap. Values saturate at `long.MaxValue` if needed.
Separate reads are not an atomic snapshot across intervening allocations or collections.

`GC.KeepAlive(reference)` expresses reachability through that call and accepts null.
It does not pin an object, manage a native resource or promise immediate reclamation
afterwards. The interpreter may retain local references until their frame returns.

.NET's `System.GC` offers generation and byte-based diagnostics and more collection
modes. neoCLR exposes the supported object counters under the author-selected
`System.Runtime` namespace. Generations, finalizers, no-GC regions and tuning are
future collector work. See the [GC reference](xref:System.Runtime.GC).
