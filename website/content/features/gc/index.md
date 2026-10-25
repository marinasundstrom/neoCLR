# Garbage collection

Inspect the managed heap and request collection through `System.Runtime.GC`.

neoCLR uses a non-moving tracing collector. It reclaims unreachable objects under
allocation pressure and when an execution completes. `GC.Collect()` also requests
one synchronous full collection, preserving reachable objects and task state.

## Concurrent collection direction

Our intention is to make concurrent GC the general-purpose default, with better
overall application performance. The current default remains synchronous while
we qualify the candidate across HTTP and non-HTTP workloads. Synchronous collection
will remain useful for small, resource-constrained or single-thread environments
where a collector worker and extra memory headroom may be poor tradeoffs.

The native candidate overlaps marking with application execution. Root publication,
mutation barriers and owner-thread reclamation preserve its lifetime boundaries;
sweeping is not yet concurrent. We tune each collector independently and track
both its own performance history and comparisons with the other collector.

[Current results](/benchmarks/gc/) · [Benchmarking and profiling procedure](/benchmarks/gc/procedure/)

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

Native allocation combines full heap validation with free-space selection in
one walk. Audited concat/join and UTF-8 slicing helpers can retain their inputs while collecting and
retrying an output allocation. Integer formatting can also retry its sole output
allocation after formatting into native stack storage. Byte and scalar array
construction can retry allocation before initializing and publishing the array.
Managed UTF-8 encoding/decoding retain their source text or array through allocation
retry. Reference-array construction also supports retry before publishing initialized
storage. Lexical path and ordinal replacement helpers retain their string inputs
through retry. Unicode helpers also retain source strings and partially constructed
sequences. Process startup retains partially built argument arrays during native
text conversion. The development POSIX file-input adapter can retry final managed
allocation after the file is closed. Culture and time-zone text producers retain
native data while retrying their output allocation. Broader collection pacing remains experimental;
see the [allocation benchmark and qualification](/benchmarks/gc/#allocation-throughput-2026-10-13).

Development builds also preserve string-intern inputs through allocation retry,
including pool identity and entry-reset lifetime. Object construction retains
arguments and receivers at collecting allocation boundaries and initializes payloads
before publication. This does not change the public
API or enable general collection pacing.

Native scheduler acquisition collects before returning idle or pending, retaining
waiting operations and reclaiming abandoned owners. Development host builds can
opt into bounded collection pacing; installed JIT/AOT hosts retain stress collection
as the default while managed-workload qualification continues. Short local
plain-text and JSON HTTP checks reclaim every allocation under both policies;
pacing greatly reduces full-collection counts, with modestly higher peak arena
extent. Sustained and concurrent load remain qualification work.


## Development: select the collector

The development CLI accepts `--gc mark-sweep` with interpreter or JIT execution.
This names the existing synchronous collector and preserves the default behavior.
It is the only public choice; unknown collectors are rejected. Native JIT/AOT
compilation accepts the same choice with `--native-gc` and `--reference-arena`.
Collector selection and collection pacing are separate policies.

## Experimental concurrent collection

An isolated AOT HTTP host can trace a private heap snapshot on a background worker.
The owner thread captures the bounded snapshot, validates the completed trace and
its invalidation epoch, then reclaims objects. Mutation or stale results can force
fallback; allocation pressure retains synchronous collection. Shutdown joins the
worker before releasing its storage. This is not an entirely concurrent collector:
capture, validation and reclamation still involve the owner thread.

The experiment now runs real asynchronous HTTP Serve operations. Qualification
covers live reclamation, suspended requests, allocation pressure, cancellation,
malformed-input recovery and shutdown. It is **experimental**, not selectable in
installed runtimes, and does not change the synchronous production default.
Concurrent HTTPS and JIT coverage remain open.

A 24-trial macOS ARM64 comparison used the same memory-view guest and libraries,
a 1 MiB arena, 16 server lanes and 64 load-generator slots. Connections closed after
each response. The measured outcomes depend on response size and offered load:

| Workload | Paced synchronous GC | Experimental concurrent GC |
| --- | --- | --- |
| 10-byte responses, 600 offered requests/s | 483.37 completed/s | 458.40 completed/s, about 5% lower |
| 8 KiB responses, 200 offered requests/s | 111.48 completed/s | 122.58 completed/s, about 10% higher |
| 8 KiB responses, 100 offered requests/s | Both sustain the offered rate; p99 22.28–23.65 ms | p99 9.62–9.64 ms, about 58% lower |

CPU usage helps diagnose collector work and resource costs; it is not by itself a
reason to reject this unoptimized experiment. These local AOT plain-HTTP results
are not production capacity, HTTPS results or evidence of a universal advantage.
The cohort predates the latest content-constructor migration; its numbers are
retained as that cohort's evidence, not relabeled as a measurement of current HEAD.

[Methodology, all trials and limitations](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/http-server/serve/memory-gc-20261019/README.md) ·
[Ownership and fallback checks](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/concurrent-gc-http/README.md)

Further work targets snapshot and validation costs, pacing, and broader workload
qualification. HTTP parsing, task completion, encoding and transport must also be
measured: collection is not assumed to be the entire server bottleneck.
