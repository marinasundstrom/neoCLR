# Garbage collection

Inspect the managed heap and request collection through `System.Runtime.GC`.

neoCLR uses a non-moving tracing collector. It reclaims unreachable objects under
allocation pressure and when an execution completes. `GC.Collect()` also requests
one synchronous full collection, preserving reachable objects and task state.

## Inspect and collect

This excerpt comes from the executable GC consumer. `MakeGarbage()` allocates two
objects in a helper that returns before collection; `Check` faults on a failed check.

```raven
{{GC_SAMPLE}}
```

The complete [downloadable example](../../samples/runtime-gc.rvn) includes the model,
helpers and imports. Follow the [development setup guide](../../try/#development)
to compile and run it.

## Understand the counters

All counters are read-only `long` properties for the current execution. Isolated
workers have separate heaps and counters.

| Property | What it measures |
| --- | --- |
| `CollectionCount` | Completed collections |
| `AllocatedObjectCount` | Tracked object allocations since execution began |
| `HeapObjectCount` | Retained objects, including uncollected garbage |
| `PeakHeapObjectCount` | Highest retained object count observed |
| `ReclaimedObjectCount` | Objects removed by completed collections |
| `HeapObjectLimit` | Host-configured object limit |

Queries do not allocate tracked objects or trigger collection. The counts concern
objects, not bytes: inline values, native allocations and untracked string buffers
are excluded. Separate reads are not an atomic snapshot across intervening execution.

## Express reference lifetime

`GC.KeepAlive(reference)` keeps a reference reachable through that call and accepts
null. It does not pin memory or own native resources. The interpreter may retain
locals until their frame returns, so becoming unused does not promise immediate
reclamation.

Explicit collection preserves frame and scheduler roots, interior references and
objects under construction. It cannot raise the heap limit and does not guarantee
that memory is returned to the operating system. Frequent forced collections may do
unnecessary work; automatic collection remains enabled.

.NET's `System.GC` offers generation and byte-based diagnostics and collection modes.
neoCLR's current API exposes its supported object counters under `System.Runtime`.
Generations, byte accounting, finalizers, tuning and no-GC regions are future work.

[API guide](../../docs/gc.html) · [GC reference](xref:System.Runtime.GC)
