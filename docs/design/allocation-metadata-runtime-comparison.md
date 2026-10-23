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
