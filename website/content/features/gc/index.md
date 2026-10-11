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


## Native compilation experiment

Development ARM64 AOT builds can opt into `--native-gc`, which links a basic nonmoving
collector into the executable. The tested Raven routing workload completes 1,024 requests
within 64 KiB while preserving a pattern and captures across requests. No shared managed
runtime is needed. The separate macOS ARM64 runtime-async POC now binds
`System.Runtime.GC.Collect()` in JIT/AOT and preserves published roots from suspended
native stacks. Other public GC controls/counters are not yet bound in this native
profile; native GC metric parity remains open.

The initial collector scans object payloads conservatively and collects at every published
operation boundary. Conservative scanning can retain extra objects, and frequent collection
can perform excessive work; precise maps and pressure scheduling remain next. An in-header
mark worklist avoids repeated heap traversal passes. Local measurements improve an adverse
graph but show no material routing speedup. General host handles, concurrent
collection and full HTTP-server qualification remain open.
[Experimental contract and evidence](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/aot-console/native-gc.md).


A private native collector experiment now tests conditional owner/value roots:
reachable owners retain their dependents, while abandoned cycles can be reclaimed.
Opt-in owned parked-frame registrations now apply this relationship at automatic
collection safepoints, preventing an abandoned stack from rooting itself. Dependent
host handles participate in the same graph and require strong ownership for dispatch. This is
now integrated into the source-local Raven runtime-async provider. In the scoped
interpreter and macOS ARM64 JIT/AOT experiment, live consumer tasks and external
completions retain suspended producers; abandoned pending cycles release frames,
operation slots and native stack mappings. Runnable work stays rooted until dispatch.
Collection does not execute managed cleanup: use explicit asynchronous disposal for
resources. This remains experimental, with Windows and performance qualification open.
[Development contract and evidence](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/aot-console/native-gc.md#conditional-ownervalue-roots--private-experiment-2026-10-11).


A private native profiling runner now combines symbolized macOS sampling with
opt-in collector timing and checked heap/stack teardown. Profiler and debugger
remain separate tools; native stepping and logical async-stack inspection are
follow-up work. Profiling identified repeated conditional-root validation searches,
which now reuse the collector's address index without changing collection frequency.
[Profiling workflow and measurements](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/runtime-async/native-profiling.md).

**Development follow-up:** native collection now avoids rescanning an activated
parked stack within one collection and combines two redundant heap walks. Descriptor
validation, conditional ownership and collection frequency remain unchanged. Local
AOT comparisons use identical guest objects and verify full heap/stack reclamation;
these results do not qualify a new collector policy or establish Windows performance.
The profiling record above retains the benchmark and correctness evidence.
