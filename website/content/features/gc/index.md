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


## Native collection and suspended operations

Development ARM64 JIT/AOT builds can link a non-moving native collector. The
runtime-async profile binds `GC.Collect()` and preserves published roots from
suspended native stacks. Other public GC counters and controls are not yet bound
in this profile.

The collector conservatively scans object payloads, which can retain extra objects.
Conditional owner/value roots let live tasks and external completions retain their
suspended producers without keeping abandoned pending cycles alive. Runnable work
remains rooted until dispatch. Collection releases unreachable frames and native
stack mappings; it does not execute managed cleanup. Dispose resources explicitly.

These are development contracts with target-specific qualification. See the
[native collector contract](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/aot-console/native-gc.md)
for supported profiles, ownership rules and correctness evidence.

## Inspect the native runtime

A provisional profiler combines symbolized macOS sampling with collector timing,
allocation/live-payload counters and bounded pause histograms. An external client
can read copied GC snapshots from an opt-in native host on macOS ARM64. Snapshots
indicate stale data when no new collection boundary has been reached.

The terminal debugger runs in-process. Standalone tool packaging, native stepping
and logical async-stack inspection remain incomplete. These tools help investigate
the runtime; they are not installed, process-wide telemetry services.
Windows x64 also has diagnostic HTTP runs separating collection, socket polling
and scheduler wait intervals; remote profiler attachment remains macOS-specific.
[Profiling workflow](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/runtime-async/native-profiling.md).

[GC benchmarks](/benchmarks/gc/) retain application throughput comparisons,
allocation/reclamation evidence and measurement limits for macOS and Windows.

Native JIT/AOT development builds adapt the collector's address index for dense
heaps. The [GC benchmark report](/benchmarks/gc/#dense-object-graphs-2026-10-13)
records general object-graph gains, small/sparse controls and memory costs. This
changes lookup work, not managed lifetime semantics or collection frequency.

Native allocation now combines full heap validation with free-space selection in
one walk. Audited concat/join and UTF-8 slicing helpers can retain their inputs while collecting and
retrying an output allocation. Integer formatting can also retry its sole output
allocation after formatting into native stack storage. Byte and scalar array
construction can retry allocation before initializing and publishing the array.
Managed UTF-8 encoding/decoding retain their source text or array through allocation
retry. Reference-array construction also supports retry before publishing initialized
storage. Lexical path and ordinal replacement helpers retain their string inputs
through retry. Unicode helpers also retain source strings and partially constructed
sequences. Process startup retains partially built argument arrays during native
text conversion. Broader collection pacing remains experimental;
see the [allocation benchmark and qualification](/benchmarks/gc/#allocation-throughput-2026-10-13).
