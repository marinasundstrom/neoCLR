# Allocation bookkeeping and metadata: Java, Go and .NET

Research date: 2026-10-23. Author direction: minimize unnecessary metadata
loading/resolution and object headers; compare Java, Go and .NET. The continuing
HTTP priority is throughput with low CPU/request, with memory savings measured
separately. This is a design comparison and proposed next slice, not an integrated
allocator redesign.

## What the runtimes do

| Runtime | Per-object / shared data | Common allocation path | Costs and constraints |
| --- | --- | --- | --- |
| Java HotSpot | Object header identifies shared class metadata; compact headers pack identity/state. Class metadata lives in native metaspace. | Thread-local allocation buffer (TLAB): check remaining space, advance top; refill/slow path when exhausted. | Header decoding, compressed-class limits, unused TLAB tails, collector coordination. |
| Go | Size-class spans and allocation bitmaps; pointer maps/type information vary by object size and whether pointers are present. | Per-P span cache, free-slot bitmap; refill through central structures. | Size rounding, span/bitmap memory, refill synchronization. P means scheduler processor, not one OS thread. |
| .NET CoreCLR | MethodTable supplies shared size/type information; preceding header carries synchronization state. Execution metadata is distinct from reflection data. | Thread allocation contexts amortize allocation and synchronization; slow paths replenish space or collect. | Context slack, shared metadata lifetime and collector coordination; layout differs by architecture. |
| neoCLR native today | 32-byte private descriptor plus the ordinary object's eight-byte identity word; specialized payloads have their own layouts. | Full descriptor validation and first-fit search on allocation; exact host-root admission also walks the arena. | Simple bounded nonmoving heap and strong corruption checks, but repeated work grows with block count. |

### Java / HotSpot

The JDK 26 GC guide documents compact headers reducing 96/128-bit headers to
64 bits when enabled, with a four-million-loaded-class limit. This is a versioned
supported configuration, not a claim about every JVM or current default. Class
metadata is managed separately in native metaspace. The JDK 25 mark-word source
shows packed class identity, hash, age and lock-related state. Rich class data is
not copied into each object.

The TLAB implementation checks available space and advances its top pointer.
Buffer retirement/refill must preserve heap parsability; local allocation does
not mean the resulting objects are confined to that thread.

Sources: [JDK 26 GC guide](https://docs.oracle.com/en/java/javase/26/gctuning/other-considerations.html),
[JDK 25 mark word](https://github.com/openjdk/jdk/blob/jdk-25-ga/src/hotspot/share/oops/markWord.hpp),
[TLAB allocation](https://github.com/openjdk/jdk/blob/master/src/hotspot/share/gc/shared/threadLocalAllocBuffer.inline.hpp),
[TLAB lifecycle](https://github.com/openjdk/jdk/blob/master/src/hotspot/share/gc/shared/threadLocalAllocBuffer.hpp).
The OpenJDK JEP page itself returned HTTP 403 during this investigation; the
comparison relies on the accessible official guide and runtime sources.

### Go

Go's allocator uses size-class spans, free bitmaps and per-P caches. A cache hit
can allocate without taking a lock; span refills amortize central synchronization.
Object location/size information can therefore be recovered from shared span
structure instead of walking every preceding allocation.

Go is not simply “headerless.” The documented heap-map implementation puts pointer
bitmaps in spans for small objects; larger scannable objects use type metadata,
sometimes in a word before the object and sometimes in the span itself. Pointer-free
allocations need neither pointer bitmap nor associated tracing type metadata.
This specialization is particularly relevant to HTTP byte/text buffers.

Sources: [allocator](https://go.dev/src/runtime/malloc.go),
[pointer maps and allocation headers](https://go.dev/src/runtime/mbitmap.go).
These are implementation sources retrieved on the research date, not Go language
layout guarantees. No claim is made that Go's slot bounds equal neoCLR's exact
logical payload bounds.

### .NET / CoreCLR

CoreCLR derives object size from MethodTable constants and, for variable-sized
objects, a component count. Ordinary objects carry a MethodTable pointer and
preceding synchronization header; this is different from neoCLR's explicit
per-allocation span/size/kind bookkeeping. Runtime type structures support fast
execution access without materializing reflection views. The design explicitly
calls for minimal type loading and GC/stack-walk access without allocation or locks.

Allocation contexts reserve regions for a thread, amortizing synchronization and
keeping normal allocation local. The key lesson is to resolve shared size/tracing
data once and maintain allocation state, rather than rediscovering it per object.
Exact sizes, layouts and policies remain runtime/version-specific.

Sources: [object layout and size](https://github.com/dotnet/runtime/blob/main/src/coreclr/vm/object.h),
[type-system design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/type-system.md),
[allocator design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/garbage-collection.md).

## neoCLR evidence and decision

A tested 24-byte descriptor packed kind into aligned span bits. It preserved exact
payload sizes, full-width worklist state, managed layout, tracing and validation.
It reduced peak arena extent by 6.3%, but one/four-owner HTTP throughput fell
2.4%/5.1%, and CPU/request rose 2.6%/4.9%. Native GC, forced-GC HTTP and compiled
shared-worker tests passed. The implementation was reverted; its patch, adjusted
fixtures, commands and results are retained in the
[compact-header experiment](../../benchmarks/http-server/primitive-comparison/compact-header/README.md).

This rejects that particular tradeoff for the current workload, not compact headers
in principle. The larger structural difference is our repeated descriptor walks.
A smaller header leaves their algorithmic cost intact. Benchmark results alone do
not isolate the regression to a particular instruction.

## Proposed next bounded implementation

Start with runtime-owned allocation bookkeeping, preserving the public value-object
contracts and current payload addresses. Do not begin by copying a whole JVM, Go
allocator or CLR collector.

1. Define a private heap-owner contract covering arena identity, initialization,
   reset, allocation, reclamation and worker handoff. The existing mutable arena
   and host API cannot silently acquire a “trusted forever” validation cache.
2. Maintain an allocation/base index through these transitions. Prototype an owned
   append path and a reclamation/free-space index that do not rediscover every
   earlier block. Compare a region/size-class map with an exact-base side table;
   budget the extra memory and invalidation work explicitly.
3. Preserve strict checked admission for existing external/mutable inputs and
   malformed trailing-header tests. Any faster internal entry point needs an
   explicit ownership guarantee and corruption/invalidation tests, not merely an
   address-based cache. Collection still needs a coherent heap snapshot.
4. Separately prototype image-owned type execution descriptors: fixed size,
   reference-field map and variable-layout discriminator, indexed by existing type
   identity. Keep names, attributes and reflection-only data out of this path.
   Validate identity, bounds and image lifetime at registration. The compiler/native
   bridge change needs matched Raven/neoCLR documentation and integration tests.
5. After the allocation-path experiment, revisit descriptor fields. Derive fixed
   object sizes from type descriptors or span classes only where doing so preserves
   exact interior-pointer, padding and allocation-info behavior. Keep variable byte
   payload lengths explicit where necessary.

These steps are proposals, not a commitment to size classes, moving GC, generational
collection or a specific binary metadata ABI. Owner-local allocation must remain
compatible with shared-object GC safepoints; a local fast path alone does not solve
cross-worker tracing or make parallel HTTP heaps safe to share.

Acceptance: corruption/failure atomicity, reset/reuse, intern/resource lifetimes,
interior roots, cross-worker collection and forced-GC HTTP all pass. Compare the
same ten-byte plain HTTP workload at one/four owners, forward/reverse, recording
CPU/request, throughput, tails, side-metadata footprint and peak arena extent.
Keep changes only when the measured balance supports the author's priorities.


## Implemented first slice (2026-10-23 follow-up)

The explicit thread-bound scope and a conservative first-free search cursor are
now implemented as private native APIs, opt-in in the source HTTP host. Acquisition
requires an empty arena; allocation/sweep maintain the cursor; GC/release validate
all descriptors. The scope's caller promises exclusive runtime descriptor writes.
This is not a general side index or shared-worker ownership protocol. The checked
path remains the default, including for shared-object workers.

Paired one/four-owner HTTP throughput rises 18.0%/15.7% and CPU/response falls
15.2%/14.6%; four-owner tails remain mixed. Native GC, repeated forced-GC HTTP and
checked-path shared-worker regression tests pass. See the
[owned-heap integration](../../benchmarks/http-server/primitive-comparison/owned-heap/README.md).
The broader steps above remain proposals where this bounded implementation does
not cover them.


The next bounded follow-up adds a 64-entry positive exact-base cache to the owned
scope. It avoids heap walks for already proven bases without a heap-wide bitmap
or object-header expansion. Allocation/checks populate it, sweep/release invalidate
it, and misses retain full validation. The additional fixed TLS cost is 512 bytes
on the tested 64-bit target. This is a neoCLR-specific implementation informed by
the general execution-metadata principle, not an imitation of a particular CLR/
HotSpot/Go cache. Paired throughput improves 19.0%/10.7% with one/four owners,
CPU/response falls 13.8%/12.1%; desktop drift limits precision. See
[evidence](../../benchmarks/http-server/primitive-comparison/owned-bases/README.md).


## Generated-frame ownership follow-up (2026-10-23)

The first root-metadata ownership slice uses the existing frame lifetime, avoiding
a new image registry. neoCLR's native compiler initializes each storage descriptor
once in the prologue; a distinct entry validates it and asserts exclusive fixed
metadata until leave/parked-stack expiry. Slot values stay mutable and traced;
ordinary native and transient descriptors retain their checks. This follows the
compiler/runtime cooperation described in
[CoreCLR stack walking](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/stackwalking.md),
without adopting its per-method unwind/GC format. Costs are entry-time validation,
a per-ancestor marker check and a new required runtime symbol for regenerated
objects. No frame size or allocation growth. Shared binary descriptors and image
registration remain proposals. See [validation and measurements](../../benchmarks/http-server/primitive-comparison/owned-root-storage/README.md).

## Standard runtime diagnostics proposal — 2026-10-23

Author suggestion: add standard monitoring/profiling to help detect runtime issues.
This is a proposed follow-up to HTTP publication attribution, not an implemented
public API or a replacement for the throughput objective.

Existing private facilities include GC statistics/profile snapshots, async-runtime
statistics, compiled-worker statistics and optional publication-site counting.
Several are thread-local; reading the main thread does not produce process totals.
The immediate need is consistent scope, units, lifetime and aggregation, followed
by timing evidence that distinguishes CPU work from waits.

Use .NET's separation of runtime metrics and diagnostic event collection as the
API/design reference: [EventCounters](https://learn.microsoft.com/dotnet/core/diagnostics/event-counters)
and [EventPipe](https://learn.microsoft.com/dotnet/core/diagnostics/eventpipe).
Go similarly separates [runtime statistics, profiling and tracing](https://go.dev/doc/diagnostics).
These sources support the separation of tools; they do not establish neoCLR's
implementation costs or prescribe its event transport. Full EventPipe compatibility
is not proposed for the first slice.

Proposed increments:

1. Define a private, versioned runtime snapshot with explicit process/worker/heap
   scope. Start with allocation bytes/count, heap use, GC count/pause distribution,
   active workers, queued/completed work and I/O completions. Distinguish gauges,
   cumulative counters and rates derived by the reader. An unsupported measurement
   must be marked unavailable rather than reported as zero. Preserve completed
   worker totals and avoid counting a shared heap once per worker.
2. Add opt-in bounded event recording for GC start/end, queue enqueue/start/finish,
   worker wait/wake and I/O wait/completion. Use monotonic timestamps, worker IDs
   and numeric operation IDs. Report dropped events explicitly. Compute queue/wait
   durations without adding clocks to every instruction or byte operation.
3. Support CPU sampling and generated-code symbol mapping using platform profilers
   first. The view must include generated code, native C/Rust runtime and OS waits;
   publication counts alone cannot identify CPU bottlenecks. Cooperative-only
   sampling risks bias toward safepoints. Portable sampling/unwinding remains a
   separate design task, not an assumption about current compiler support.

Keep updates local to their owning thread where practical. A collector must use
an explicit synchronized snapshot/publication protocol; unsynchronized reads of
another thread's ordinary counters are not safe. Avoid a global lock or contended
atomic increment per allocation/request. Bound recording storage, defer names and
formatting to export, and do no managed allocation or file I/O in diagnostic hooks.
Do not record request payloads/URLs as labels. Runtime-toggleable light monitoring
and compile-time detailed instrumentation have different disabled-cost tradeoffs;
measure both before selecting defaults. A dropped diagnostic event must not abort
an application, unlike the deliberately strict benchmark attribution fixture.

First bounded implementation candidate: aggregate existing GC/async/worker counts
at safe owner boundaries into a versioned snapshot and integrate it with the HTTP
benchmark reporter. Retain exact units and explicit unavailable fields. Then add
queue/wait timing where CPU profiles show a need. Test concurrent snapshot safety,
thread exit, shared-heap accounting, overflow/drop behavior and unchanged application
results. Compare disabled and enabled CPU/request, throughput and memory against
the same plain-text fixture; do not select an arbitrary overhead budget or claim
zero cost before measuring. Native/public API exposure and tooling command names
remain undecided; any public surface requires matching reference documentation.

### First implemented slice — 2026-10-23

Private `runtime/native/diagnostics/snapshot.h` now provides versioned owner-thread
cumulative counters, availability, checked interval subtraction and checked merging.
HTTP's opt-in snapshot build captures before/after each isolated application
invocation and merges copies on the coordinator only after workers join. Export
retains individual intervals and marks unavailable metrics null. This is a bounded
counter foundation analogous to the role of .NET runtime metrics, not EventPipe
transport/API compatibility. It adds no generated-code instrumentation or managed
wrappers and does not change the compiler bridge.

Scope is intentionally narrower than the proposal: no live cross-thread collector,
shared-heap aggregate, heap gauges, queue timings, pause histograms or CPU sampler.
Peak values are not summed. Counters have no guaranteed owner identity in the data;
callers must preserve ownership/reset constraints. HTTP's isolated invocation
adapter establishes those constraints; general shared-heap hosts remain unqualified.

Eight optimized/sanitized source configurations and seven HTTP measurement/smoke
runs pass. The disabled worker object is byte-identical to its prior source.
Basic snapshots show +0.22% mean CPU/response and -2.84% throughput in short paired
four-owner trials; variation prevents a precise overhead conclusion. GC timers
remain separately opt-in. No faster-server claim follows from these diagnostics.
See [contracts, measurements and limitations](../../benchmarks/http-server/primitive-comparison/runtime-snapshot/README.md).

### Native stack attribution follow-up — 2026-10-23

The [macOS stack-profile tool](../../benchmarks/http-server/primitive-comparison/stack-profile/README.md)
now captures checked one/four-owner HTTP loads and maps generated leaf/caller symbols
using byte-reproduced compiler output. All thread self counts reconcile, including
small leaves and recursion. Coordinator waits are separate. Six parser/name tests
pass. This uses an OS diagnostic as proposed; it is wall-clock sampling including
blocked threads, not a portable on-CPU profiler or managed async-causality trace.

A useful distinction emerges: most memory-clearing self samples are called from
makecontext, while the allocator has very few direct clearing samples. Allocation,
TLS access and root publication remain leading locations. Investigate completed
async context reuse with independent safety/performance qualification; do not infer
that cached stack mappings already avoid context initialization, or that sampled
proportions predict an attainable throughput gain. No runtime change is made here.

### Completed async context experiment — 2026-10-23

Implement opt-in macOS `NEOCLR_ASYNC_CONTEXT_REUSE`: retain successful completed
contexts at stable addresses, reset invocation ownership/state and inherit the new
caller's signal mask/FP environment. Suspended, expired and faulting bodies are not
reused. Cached contexts are not live work; shutdown releases them. Default runtime
objects remain byte-identical; Windows reuse is rejected pending qualification.

Compared with [.NET state-machine async](https://devblogs.microsoft.com/dotnet/how-async-await-really-works/),
this improves neoCLR's existing retained-stack implementation without claiming the
same representation or ExecutionContext support. Costs include more lifecycle state
and retained native stack contents; cache mapping bounds remain unchanged. Fourteen
native test executions, forced-GC HTTP and paired one/four-owner trials pass. Mean
throughput improves 11.7%/14.5% and CPU/response falls 10.6%/13.4%. A separate profile
no longer finds the makecontext clearing hotspot. Keep this opt-in until broader
async consumers/platforms are qualified. See [contracts and evidence](../../benchmarks/http-server/primitive-comparison/context-reuse/README.md).

Follow-up compiled-consumer qualification passes 40 invocations across baseline/reuse
and optimized/sanitized builds: nested awaits, cancellation and scoped disposal,
retained objects through GC above the cache bound, operation lifetime and terminal
fault teardown/re-entry. This strengthens lifecycle evidence without changing the
representation, default or previous performance results; shared-heap async and other
platforms remain outside this qualification. The linked experiment retains the runner
and exact report.

Following those checks, select reuse as the development macOS ARM64 default with
`NEOCLR_ASYNC_CONTEXT_REUSE=0` as the diagnostic fallback. Optimized object identity
proves both paths are unchanged from the qualified implementation; repeat lifecycle,
compiled-consumer and forced-GC HTTP checks pass. This is a platform-specific policy
selection, not a new .NET semantic divergence or cross-platform qualification. The
retention costs and representation tradeoffs above still apply. Windows remains on
its existing carrier. Historical opt-in reports remain historical evidence.


### Broader post-integration investigation — 2026-10-23

Fresh stack attribution, allocation-search counters and 28 paired capacity trials
identify costly repeated first-fit search and continued publication/TLS metadata work.
An owned-heap free-space index is the leading allocator follow-up; ThinLTO has a
modest confirmed benefit and compiled-consumer checks. Larger GC budgets and blind
append-first allocation expose tracing/search/memory tradeoffs, not defaults to adopt.
The author's generational-GC question remains an open design track alongside the
existing concurrent-GC direction. Compare nonmoving generations and copying nurseries
with explicit barrier/relocation prerequisites; measure survival first. Full results,
primary-source comparisons, failed/variable screening evidence and priorities are in
[the cost investigation](../../benchmarks/http-server/primitive-comparison/cost-investigation/README.md).


### Owned free-space index integrated — 2026-10-23

Implement the bounded allocation-index follow-up using an address-ordered list in
unused free-descriptor words. Compared with the .NET/Go/HotSpot approaches discussed
above, this keeps neoCLR's existing nonmoving first-fit contract rather than adopting
size classes, per-thread bump regions or generational lifetime assumptions. It removes
most live-block searches without adding descriptor bytes or auxiliary allocations.
Costs are link maintenance and reconciliation during existing full validation walks;
fragmented heaps still have linear free-run search. External/shared heaps retain their
checked path. Differential placement/extent tests, corruption checks, independent
threaded owners, shared-worker regressions and forced-GC HTTP pass. Mean one/four-owner
throughput rises 21.7%/18.7%, with 17.9%/16.1% lower CPU/response in short paired desktop
trials. No public API or semantic divergence from the existing contract is introduced.
See [implementation and evidence](../../benchmarks/http-server/primitive-comparison/free-index/README.md).


### Post-index profile and rejected shortcuts — 2026-10-23

Fresh source-matched profiling puts root publication, TLS and metadata validation
above allocation search. A direct compact-plan marker-prefix check regresses the
one-owner HTTP case; combining retained stack bounds reduces ARM64 TLS resolution
sites but does not improve end-to-end results reliably. Revert both and retain all
16 capacity trials. Strengthened mutable-plan and stack lifetime/isolation tests pass
on restored code. This supports retaining explicit validation contracts while
investigating larger reductions in publication frequency or image-owned typed maps,
rather than caching validity by pointer. Reuse the CLR metadata/effect-analysis
comparisons above; no immutable image contract is implemented by these experiments.
See [full evidence](../../benchmarks/http-server/primitive-comparison/stack-bounds/README.md).
