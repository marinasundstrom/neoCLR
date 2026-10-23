# Basic native collection experiment

The author requested continuing until basic GC also works for native compilation
(2026-10-08). This work extends the ARM64 AOT experiment; it does not change the
interpreter collector or establish a production native hosting ABI.

## Kernel foundation

`native-gc.c` implements nonmoving mark-and-sweep inside the host's bounded arena.
A 32-byte allocation descriptor records block span, logical payload size, allocation
kind and mark state. Reclaimed adjacent blocks coalesce and allocations reuse holes;
all payload addresses remain stable. Sweep and coalescing share one forward pass:
each block is reclaimed/poisoned once, adjacent free spans merge as they are visited,
and a trailing free run lowers the arena extent. No malloc or shared managed runtime is required.
Allocation itself never collects and publishes its output only on success.

Roots come from compiler-published argument/local slots, initialized stack snapshots,
constructor/result storage and the fault context. Heap membership is checked against
allocation payload ranges, so an interior reference keeps its owner alive. The tracer
does not follow borrowed pointers into native stack storage: their owning caller
locals/arguments or constructor storage must already be published. External byrefs,
escaped stack references and arbitrary native callbacks are outside this experiment.

Text and byte payloads are atomic. String arrays trace their initialized element slots.
Reference-object payload words are currently conservative candidates, including nested
value fields. This can retain an allocation when an integer resembles a heap address;
it cannot move objects or provide precise liveness. Initialized erased payloads can
also over-retain. Descriptors exclude headers/padding, bound all scans and distinguish
atomic byte values from String arrays even when their existing payload kind tags overlap.

An intrusive mark worklist uses aligned header pointers in the existing state word.
Each reachable allocation is queued once, without recursion or external allocation.
Failure clears temporary links before returning; no sweep occurs on an invalid array
descriptor. A per-collection 256-bucket sparse address index starts each candidate lookup at
the block containing its bucket boundary. Exact payload bounds still decide membership;
headers, padding and free spans are excluded. This costs about 2 KiB of bounded stack
storage and one linear index-building pass per collection. The index holds no addresses
between collections, allocates nothing and leaves the heap ABI/rooting policy unchanged.
Bucket-local scans remain linear; this is not a constant-time or production collector.
There are no generations, compaction, concurrent collection, finalizers or weak handles.
Fragmentation or a large live set can still exhaust the buffer. Thread-local counters
are cumulative diagnostics, not shared heap state; contexts and buffers remain isolated.
Entry reset/buffer release is not counted as traced reclamation, so subtracting cumulative
reclaimed allocations from allocations is not a live-object count across entries.

The initial kernel contract test runs with UndefinedBehaviorSanitizer and bounds checks.
AddressSanitizer on this host stalled during dyld/runtime initialization before `main`;
that unavailable check is not counted as passed.
It covers cycles, an interior-only root, String-array initialization markers, atomic
payloads, fault-message retention, free-block reuse, bounded exhaustion and buffer guards.
Generated-code safepoint integration and fixed-budget routing now also pass, as described below.

## Comparison and provisional choices

.NET's collector traces roots and supports generational collection and compaction;
this experiment only supplies a small nonmoving foundation, with higher retention and
fragmentation costs. See [Microsoft GC fundamentals](https://learn.microsoft.com/dotnet/standard/garbage-collection/fundamentals).
Conservative candidate scanning and allocation-kind distinctions are established
techniques in the [Boehm collector overview](https://www.hboehm.info/gc/gcdescr.html)
(primary sources reviewed 2026-10-08). This implementation does not embed that collector
or scan arbitrary machine stacks/registers. It relies on neoCLR's explicit published
storage and closed entry contract.

Collection runs at complete pre-operation boundaries, and audited allocation
helpers may also collect/retry while explicitly protecting their temporaries.
Unaudited native services remain noncollecting; copying/publishing result data
does not introduce collection points. The stress policy collects at every boundary;
precise object maps and an allocation-pressure schedule remain follow-up work. No speed,
pause-time or production-server claim follows from this design.


## Enabling compiled collection

Pass `--native-gc --reference-arena --compile-system` to the AOT tool with the usual
explicit System seed, dependency modules and service bindings. `--native-gc` is mutually
exclusive with diagnostic-only `--probe-stack-roots`; inspection and emission share
admission and report `capabilities.nativeGC`. GC images import distinct
`neoclr_gc_{enter,leave,stack_roots,transient}_v1` hooks. Link `root-probe.c`, `native-gc.c`
and `text-arena.c` with `-DNEOCLR_NATIVE_GC` together with the existing platform adapters.
Default arena images retain their existing layout and no-collection behavior.

The entry/context shape remains v4, but GC allocations have private in-buffer headers,
so a GC image and its adapters must be built together. `text.used` is the current heap
extent (including headers and interior free blocks), not cumulative allocation bytes or
peak live bytes. No payload moves. The collector never allocates outside that buffer.
The host must provide a valid aligned buffer/context and keep them alive for the invocation.
Concurrent mutation, reentry into the same context and native services calling back into
the same heap are unsupported. General host handles and reference-returning entrypoints
are not introduced: exported entrypoints still return Int32. Fault messages remain rooted
by their live context and expire on entry reset or buffer release as before.

Collection occurs after the compiler writes a complete pre-operation snapshot and retires
prior transient result views. Suspended callers still expose arguments, locals, operand
snapshots and constructor/pending-result storage. A stack borrow's owner storage is already
published by its active owning frame; a heap borrow recovers its allocation by range.
No borrowed pointee is read without that owner. Native wrappers expose argument copies,
and unaudited arena-only services never collect: temporary UTF-8 pointers, output
buffers and partially written objects must not be interrupted by this collector.
The explicitly audited stack-capture helper can retry its single output allocation
at a complete boundary; see the helper audit below. After an allocation,
its output reaches published storage before the next collection boundary. The compiler's
existing byref/output and escape restrictions continue to apply.

The private `neoclr_gc_collect_v1` is also usable by the test host after native frames
have returned. At that point only the fault context is a guest root. It is not an
implementation of the public Raven `System.Runtime.GC` API; binding that API is separate
work. Calls from arbitrary native services are not supported collection boundaries.

## Executable evidence

`native-gc.neoil` tests an object retained only through an interior output borrow, writes
a new dynamic String through that borrow, carries it through erased-value returns, and
creates 512 unreachable self-cycles per invocation. The ARM64 host uses only 2 KiB,
checks successful reentry and dynamic fault text after a host collection, then checks a
64-byte live-set exhaustion fault with unpublished result. Its user-fault rendering equals
the interpreter's. The C adapters run with undefined-behavior/bounds sanitizers; the
separate heap fixture checks initialization markers, coalescing/reuse and canary bounds.

Run the real Raven consumer with:

```sh
python3 docs/experiments/aot-console/verify_route_lifetime.py \
  --compiler /path/to/rvnc.dll --runtime target/debug/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --bundle /path/to/neoclr-native-poc --output /tmp/route-gc --native-gc
```

[Recorded validation](route-native-gc-validation.json) covers eight routing outcomes,
retained pattern/captures, exact output/fault parity, empty frame chains at host return,
and standalone executables with an empty environment and only libSystem dynamically
linked. With GC, 1,024 requests complete in 64 KiB; the old arena-only mode exhausted that
budget at 128 requests. The final heap extent is 1,648 bytes at every tested request count;
this is not a peak measurement. At 1,024 requests, 33,613 of 33,628 allocations have been
reclaimed across 278,666 stress collections. A 512-byte buffer still produces a clean
NativeMemoryLimitExceeded fault. These are bounded correctness/lifetime observations,
not throughput or latency results and not qualification of an HTTP listener/server.

Next: replace conservative object candidates with precise type maps, introduce a measured
allocation-pressure policy, and continue through the actual HTTP service's dependencies.
A production runtime also needs host handles, concurrency/transition contracts and stronger
fragmentation policy. Generated code now collects and reuses storage within this bounded experimental profile.


## Measured worklist slice — 2026-10-08

The author asks to evolve the POC through relevant benchmarks and examine improvements
for interpreter mode. The first measured change replaces repeated whole-heap mark
passes with a worklist, retaining the 32-byte header and existing collection schedule.
The interpreter already uses a pending-object vector (`src/gc.rs`); this slice transfers
that algorithmic approach into native mode with storage appropriate to a bounded heap.
It does not change interpreter behavior. Unlike .NET's generational/compacting collector,
this remains a whole-heap nonmoving collector; the benchmark is not a .NET comparison.

[Recorded samples and provenance](native-gc-worklist-validation.json) compare baseline
`9bb27be6` with identical Clang `-O2` adapters and the same generated routing object.
Two warm-up pairs precede seven measured pairs with alternating execution order:

| Workload | Baseline median | Worklist median | Interpretation |
| --- | ---: | ---: | --- |
| 100 collections, 1,024 reverse-linked nodes | 316.6 ms | 109.7 ms | 2.89× in an intentionally adverse graph; construction excluded |
| Raven routing, 1,024 requests, 64 KiB | 391.4 ms | 386.0 ms | About 1.4%; treat as effectively unchanged, not a demonstrated application gain |

Routing timings include process startup and captured output. Allocation, collection,
reclamation counts and final heap extent match; the graph harness verifies every link
and then complete reclamation. These are local ARM64 observations, not release performance
guarantees. Compile/startup versus execution, peak memory and pause distributions need
separate measurements before broader platform comparisons.

Reproduce after generating the routing object with `verify_route_lifetime.py --native-gc`
(using its compiler, runtime, AOT and bundle arguments):

```sh
SDKROOT=$(xcrun --show-sdk-path) python3 docs/experiments/aot-console/benchmark_native_gc.py \
  --baseline 9bb27be6 --route-object target/aot-route-gc-final/route.o \
  --output target/gc-worklist-rerun
```

The output directory must be new. Retain raw samples and input hashes for later slices.
Benchmark changes to tracing, allocation, scheduling and hot native services; keep routine
API validation focused when no performance question exists. Recheck output/fault and
memory-budget behavior alongside timings rather than accepting a speed-only result.

Interpreter portability review: its worklist and survivor-based allocation threshold
already cover these algorithmic ideas. Native header-pointer packing cannot be copied
into the interpreter's identity-indexed heap. Reusing interpreter tracing scratch buffers
is a possible future experiment, requiring allocation/pause measurements first. Shared
lifetime fixtures and workload inputs are useful across both modes now. Native scheduling
needs allocation-size/pressure information at safe boundaries: skipping collection merely
because no allocation happened since the previous one can miss newly dead roots and cause
avoidable exhaustion. Allocation services still must not collect with unpublished temporary
references. Precise object maps and pressure scheduling remain the next contract work.

## Host-held strong roots (2026-10-08)

The private C hosting experiment now provides create/replace/read/release operations
for strong roots, ahead of asynchronous socket completion. A host can retain an
allocation after its submitting guest frame leaves; the collector traces that allocation
and its reachable graph. Replacing or releasing a handle removes that edge for the next
collection. Null is allowed and still requires explicit release. This does not yet
provide callback invocation, task pumping or asynchronous socket services.

The comparison is .NET's [normal GCHandle](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.gchandletype?view=net-10.0)
(reviewed 2026-10-08), which retains an object held only by unmanaged code. The current
neoCLR heap does not move, so these private handles can return an allocation address;
this is not a pinning API or a stable ABI promise for a future moving collector. Keeping
roots in expired guest stack frames is invalid. A host registry makes ownership explicit
at the cost of required cleanup and possible retention leaks. No interpreter GC change
is needed for this native-host prerequisite.

The bounded contract is intentionally narrow:

- 256 handles per native thread, shared by its contexts; no malloc. IDs come from a
  process-wide monotonic atomic counter and are never reused. Quota/ID exhaustion returns
  NativeMemoryLimit (5). Invalid handles/arguments return RuntimeError (3).
- Only null or a live, untagged allocation base from that context is accepted. Interior
  addresses, tagged String views and image literals are rejected. Operations neither
  allocate nor collect; failure leaves output parameters unchanged.
- Handle lookup checks context and the creating thread's registry. This is not general
  context thread-ownership enforcement: callers must not transfer a context/buffer to
  another thread or destroy them while handles exist. Worker threads must post completion
  tokens back to the owning thread; they cannot execute guest code or collect its heap.
- Native GC entry checks run before heap reset, rejecting live handles (including null
  handles) or active guest frames for that context. Rejection sets fault code 3 without
  publishing a result or clearing the heap; ordinary entry fault reset still occurs.
  Release all handles before another entry. Context/result/buffer and host output storage
  must obey the existing non-overlap/lifetime requirements.

Registration validates the heap and scans allocation bases. Lookup and collection visit
an active linked list, avoiding a 256-slot scan on each collection when no handles exist.
Address validation/marking remains linear in heap blocks per root; the registry is not a
performance optimization. Measure populated-root cost when the asynchronous consumer
establishes a representative workload. General weak/pinned handles, teardown automation,
thread transfer and moving-collector integration remain open.

Focused validation on macOS ARM64 with the explicit Xcode SDK: all five `native_gc`
integration tests pass, including sanitized `host-roots-test.c`; the compiled
`native_gc_entry_rejects_live_host_handles_before_heap_reset` test and existing
`native_reference_arrays_preserve_identity_owners_and_faults` test pass. Coverage includes
parent/child retention with no guest frames, replacement/reclamation, null handles,
invalid/interior/freed pointers, stale/wrong-context/foreign-thread handles, globally
distinct tokens, pool exhaustion, out-of-order release, entry reset and buffer canaries.
These are correctness checks, not HTTP benchmark results. Generated GC-enabled objects
now require the matching `neoclr_gc_entry_check_v1` C helper when linking.

A follow-up aligns the published-frame hook with the compiler's 1,024-function
selection limit (the helper still used 512). The focused
`published_frame_function_bound_matches_native_selection` test checks ID 1023
enter/leave and rejection of ID 1024 in both GC and diagnostic builds. This fixes
a native helper admission mismatch; it does not broaden the compiler's limit.


## HTTP-driven address lookup optimization (2026-10-08)

A separate one-second CPU sample of the 32-request native HTTP consumer put 672 of
731 main-thread samples in the collector. The sparse index targets its repeated
whole-heap candidate searches. Three alternating before/after pairs, using the exact
same Raven artifact and native object, measured median throughput 20.08 versus 96.05
requests/s (median request latency 49.74 versus 10.43 ms). These short local samples
support this adapter optimization, not a general runtime or cross-platform ranking.
The collection-at-every-boundary stress policy remains unchanged.

Ten focused kernel tests pass, including a 4,096-object reverse chain across buckets,
interior roots, header/padding rejection and repeated reuse. Persistent HTTP passes
baseline, sanitized candidate and standalone candidate; all twelve single-request
HTTP mode/case checks also pass. A sanitized static-frame audit measures the collector
at 7,104 bytes (-O0) and 3,264 bytes (-O2), within the existing 192 KiB helper reserve;
it does not model libSystem or sanitizer internal frames.
[Raw evidence](../../../benchmarks/native-web/gc-index-validation.json) records inputs,
commands, samples and the test-driver stdout buffering correction.

The interpreter already resolves managed identities through its object map
(`src/gc.rs`); it does not perform these ambiguous native-address scans. Porting this
index would add cost without solving the same problem. Shared root/lifetime invariants
and the HTTP workload remain cross-cutting; no interpreter speedup is claimed.
The existing .NET/Boehm comparison above remains applicable: neither this lookup change
nor conservative tracing substitutes for precise descriptors or a GC scheduling policy.


## Conditional owner/value roots — private experiment, 2026-10-11

`neoclr_gc_collect_conditional_v1(context, frames, entries, count)` adds a bounded
collector primitive alongside ordinary collection. Each borrowed entry contains a
live allocation-base `owner`, a null/live allocation-base `value`, and an output
`retained` flag. A completely null entry is an inert tombstone. Foreign/interior
pointers, values without owners, null arrays with nonzero counts and counts above
256 reject before marking. The array is native caller-owned storage, must remain
valid, and must not be mutated or placed in reclaimable guest storage during the call.

Only independently marked owners activate their values. Ordinary tracing and
conditional activation repeat to a fixed point, so entry order does not affect
liveness and mutually dependent abandoned entries do not resurrect each other.
On successful collection, dead entries are cleared before sweeping; this prevents
later allocation-address reuse from accidentally reviving an old association.
Failure leaves entries/results and collection statistics unchanged. Existing
unconditional host handles, active frames, parked frames and fault roots retain
their current behavior.

This matches the dependent-lifetime principle of the .NET ConditionalWeakTable
comparison in the [scheduler design](../../runtime-scheduling-design.md#conditional-continuation-roots--collector-experiment-2026-10-11).
It does not claim that .NET Task uses that table. Ordinary managed ownership remains
preferable when it expresses the graph directly. Native retained stacks additionally
need precise reachability relationships to the managed operation.

This primitive does not persist/register entries: a caller must supply them at every
relevant collection. It is not wired into generators or scheduler policy. Generator
stacks/callbacks remain unconditional roots. Integration must account for live iterators,
outstanding advancement tasks, runnable callbacks and externally owned pending I/O;
then detach dead parked roots, native stack storage and operation handles without
running managed cleanup from GC. Explicit asynchronous disposal remains the mechanism
for deterministic cleanup.

Cost: at most 256 borrowed edges are rescanned after each marking wave, with no
additional heap allocation. Worst-case dependency chains can require many waves.
No performance improvement or production policy is claimed.

Validation: [sanitized collector/host-root contracts](../runtime-async/native-conditional-roots-validation.json)
cover owner retention, value-to-owner cycles, reverse-order chains, shared owners,
independently rooted values, dead-entry address reuse, null values, foreign/interior
references and failure after tracing a malformed dependent. The existing Windows
host validation already compiles this shared collector contract; no Windows execution
is claimed for this revision. The [generator regression](../runtime-async/conditional-gc-generator-validation.json)
passes interpreter, macOS ARM64 JIT and AOT using ordinary strong roots.
Reproduce the focused C checks with `runtime-async/validate_conditional_roots.py`
and a passing async-iteration host recipe.


## Managed ownership of parked frames — private experiment, 2026-10-11

`neoclr_gc_roots_park_owned_v1(context, owner, record)` parks the current frame
chain with a nonnull managed owner. The owner must be a live allocation base in
that context, the record must be zero-initialized and unregistered, and every
frame must belong to the same context. Invalid admission leaves the active chain
and record unchanged. Empty chains are allowed. The record and its frame storage
remain caller-owned native memory; neither may be placed in collectible storage.

Unlike the borrowed owner/value entries above, this association persists until
resume or discard and participates in **every** native collection, including
compiler snapshot safepoints. A marked owner activates its frame storage, transient
slots and spill lanes. Frames can discover further owners, so stack associations
and owner/value entries share the marking fixed point. A suspended frame's reference
back to its owner does not make an otherwise abandoned stack reachable.

After successful tracing, an unreachable owner's registration becomes expired:
its head and owner are cleared before sweeping. Expired registrations cannot resume
(the private resume API aborts misuse), remain registered until explicitly discarded,
and prevent context reset even though their frame chain is now empty. Address reuse
does not reactivate them. Failed tracing leaves registration state unchanged. Another
context's collection cannot expire the registration. Resume of a live registration
restores the active chain and clears its ownership association.

This extends the dependent-lifetime principle discussed above to native frame
storage; it is not a .NET Task API or a claim about CLR suspended-stack implementation.
The benefit is expressing reachability without making a suspended stack its own
permanent root. Costs include scanning activated frame chains during marking waves,
retaining native registration storage until its owner discards it, and requiring an
explicit ownership model for pending operations. No speedup is claimed.

[Sanitized tests](../runtime-async/owned-parked-roots-validation.json) cover automatic
safepoints, live resume, abandoned back-references, allocation-address reuse,
cross-context and mixed-frame rejection, empty chains, reverse-order dependencies
between stacks and conditional values, and failure atomicity. The shared native
collector test is also consumed by Windows validation; Windows was not run for this
slice. [Generator regression](../runtime-async/owned-parked-generator-validation.json)
passes interpreter, macOS ARM64 JIT and AOT with the existing strong-root behavior.

**Integration remains open:** the runtime-async carrier still uses ordinary strong
parking and strong callback handles. The collector does not release stack mappings,
operation slots or callbacks, and does not run managed cleanup. The next integration
must supply reachability from live iterators, outstanding advancement tasks and
external completion owners, then discard expired native resources. Merely switching
generator parking to this API would leave strong callback cycles or lose needed
work; it is not yet an end-to-end abandoned-generator solution.


## Dependent host handles — private experiment, 2026-10-11

`neoclr_gc_host_root_create_owned_v1(context, owner, value, output)` creates a
persistent dependent handle in the existing thread-affine handle table. The owner
must be a nonnull live allocation base in the same context; the value may be null
or another live allocation base. Admission and quota failures preserve the output.
Strong and dependent handles share the 256-entry per-thread limit and non-reused
identities. Existing strong-handle behavior is unchanged.

Every collection, including compiler snapshot safepoints, activates the value only
when the owner is independently marked. These associations share the fixed point
with parked frames and borrowed conditional roots. A value's back-reference to its
owner cannot keep an abandoned pair alive, but an independently rooted value can
retain its owner through ordinary managed references.

Successful collection clears both pointers and expires the handle before sweeping
an unreachable owner. Reads and replacement then reject without modifying outputs;
address reuse cannot revive the association. A live handle accepts value replacement
without changing its owner. Failed tracing leaves handle state unchanged. Expired
handles still consume quota and block entry reset until explicitly released. Release
also works for expired handles; the exclusive ownership check still counts them.
No release or managed cleanup callback is run from the collector.

**Dispatch requires strong ownership.** The callback readers reject dependent
handles even while live: a borrowed read does not protect the callback across a
subsequent safepoint. A dispatcher can read the value and create an ordinary strong
handle without an intervening collection, dispatch through that strong handle, then
release it. This makes the lifetime requirement explicit at the existing dispatch
boundary rather than allowing a callback to disappear during execution.

This is the same dependent-lifetime principle as the .NET ConditionalWeakTable
comparison above, applied to a private native handle table. It does not replace
managed callback storage or claim CLR Task implementation parity. Costs are two
pointers/lifetime flags in native bookkeeping and another bounded table scan per
marking wave. No performance improvement is claimed.

[Sanitized contracts](../runtime-async/owned-callback-roots-validation.json) cover
admission, live and abandoned receiver cycles, independent callback retention,
dispatch rejection and strong-handle admission, mixed handle/parked-stack activation
at automatic safepoints, cross-context isolation, address reuse, null values,
replacement and failure atomicity. The [generator regression](../runtime-async/owned-callback-generator-validation.json)
passes interpreter, macOS ARM64 JIT and AOT. No Windows execution is claimed.

The runtime-async carrier and managed provider still use strong callback handles.
Their integration must describe iterator/task/external completion ownership and
release expired native operations and stacks. This primitive does not yet enable
abandoned-generator reclamation.

## Indexed conditional-root validation — 2026-10-11

The [native async profile](../runtime-async/native-profiling.md) identifies collector
work as the largest pending-await cost. Collection now builds its existing sparse
address index before validating dependent handles and parked-stack owners. Exact
allocation-base checks reuse that index instead of walking the entire heap for each
owner/value. Conservative marking still admits valid interior payload pointers;
conditional owners and host values still require exact live allocation bases.

The index remains local to one collection, after full descriptor validation and
before any coalescing. No cached address survives a collection and no context,
metadata, root-lifetime or collection-frequency contract changes. This is a lookup
implementation optimization, not a new GC policy or a divergence from managed
ownership semantics. Like the existing nonmoving collector, its conservative object
scan differs from CLR's typed moving/generational implementation; this change does
not introduce compaction or generational collection.

A focused regression fills a heap across index buckets, frees intervening blocks,
checks 100 conditional edges and rejects 400 header/interior/foreign-boundary pointer
candidates, including payload bytes shaped like an allocation descriptor. Existing
conditional chains, cycles, rollback, host handles and 320 native owned suspensions
remain sanitizer-qualified. Matched benchmark evidence is linked from the
[profiling notes](../runtime-async/native-profiling.md).

## Trace each owned parked stack once — 2026-10-12

Native sampling identifies collection and frame marking as the remaining dominant
runtime-async costs. Conditional ownership previously rescanned every marked owner's
parked frames on every fixed-point pass. Frames cannot mutate during a collection,
so repeating a registration's scan cannot discover new references.

Each owned parked registration now carries private `traced` scratch, reset before
every collection's marking phase. The first independently marked owner activates
that registration once. Other owners discovered later still activate their stacks
on subsequent passes; multiple stacks sharing an owner each scan independently.
No marks survive as reachability evidence across collections, including after failure.
The field occupies existing alignment padding in the private registration on the
qualified 64-bit target. Hosts should still rebuild against matching private headers.

This preserves the existing .NET dependent-lifetime comparison above. It is an
internal tracing optimization, not an API divergence, generational collector or
change to collection frequency. Unlike reducing collection frequency, it removes
redundant work within the existing stress-at-safepoints behavior. The cost is one
private scratch flag per registration and a reset traversal. Concurrent mutation
or parallel marking would require revisiting the synchronization contract.

Sanitized tests cover reverse dependency order, shared owners, changed frame slots
on the next collection, failed tracing followed by retry, abandoned cycles and
owned native suspension/reclamation. Benchmark evidence is linked from the
[native profiling record](../runtime-async/native-profiling.md).

The same slice fuses heap-descriptor validation with address-index construction and
clears old marks while seeding intern roots. Every descriptor is still checked before
any marking mutation. Seeding only queues the current intern entry; transitive owner
marking starts after that pass, so later blocks cannot have fresh marks cleared.
The added pass test checks a malformed late descriptor without heap mutation,
multiple intern entries pointing forward in the heap, repeat collection, and release.
These changes remove two complete heap traversals without weakening admission.

### Native resource ownership registrations

The private `neoclr_gc_resource_attach_v1` / `neoclr_gc_resource_release_v1` hook
associates a caller-owned registration with a live managed allocation, without
rooting it. Successful collection releases registrations whose owners are unmarked.
Explicit release and collection clear/unlink before running trusted native cleanup,
which may free its registration. Failed tracing runs no cleanup.

This is not a managed finalizer facility. Cleanup must not block, perform I/O, call
managed code, allocate managed storage, reenter runtime services or fail. Registrations
are thread-affine, with a 1024-per-context quota. Entry reset and session ownership
admission reject outstanding registrations; adapters release them before teardown.
The [shared security foundation](../../shared-security-provider.md) records the
SafeHandle comparison, costs, focused evidence and remaining TLS adapter integration.

## HTTP-driven sweep optimization — 2026-10-13

[HTTP profiles](../../../benchmarks/http-server/profiling.md) identify collection as
the shared plain-text/JSON cost. Combining sweep and coalescing removes one heap
walk without changing collection frequency, allocation admission, roots, failure
atomicity, poisoning or native-resource cleanup order. This is an internal algorithm
optimization, with no native ABI, Raven bridge or public API change.

The [.NET comparison](../../runtime-gc.md#comparison-and-provisional-design-decision)
remains applicable: preserve observable reachability and explicit full collection;
this does not add CLR generations/compaction or claim competitive CLR performance.
Pressure-based pacing remains separate because allocation services cannot collect
while their native temporaries are unrooted.

[Sanitized validation](../../../benchmarks/http-server/gc-sweep-validation-20261013.json)
passes collector/host-root/profile tests and 320 owned native suspensions. The new
mixed-free-region test verifies merging old holes with newly dead blocks, live
payload survival, hole reuse without heap growth, tail trimming, exact reclamation
counts/bytes and poisoning. Existing descriptor-failure, conditional owner, cycle,
precise-boundary and retained-stack checks remain active. This is macOS ARM64
validation, not refreshed Windows or AddressSanitizer evidence.


## Collecting-helper audit — 2026-10-13

Runtime performance work covers allocation-heavy, reference-heavy and asynchronous
workloads, not only HTTP. Before changing native collection frequency, admit
allocation-pressure collection only where every live reference is accounted for.
The private rooted allocator's boundary identity check alone cannot prove this.

| Helper family | Live native state across allocation | Admission |
| --- | --- | --- |
| Current stack capture | Active frame records and immutable image names; no managed intermediate | Single allocation may collect/retry with current-context complete roots or quiescent host |
| Capture of ancestor context under a foreign frame | Frames outside the current-context boundary | Keep noncollecting allocation |
| GC-generated concat/join variants | Borrowed text inputs or parts array and separator | Protect values during allocation retry; parts tracing retains its text elements |
| Other arena-only text/object/array helpers | Caller operands, borrowed buffers and sometimes partially constructed outputs | Remain noncollecting; context-aware entry and temporary-root audits still needed |
| Multi-allocation producers and native callbacks | Intermediate objects and externally retained values | Remain noncollecting; publish each intermediate before any collecting call |

`native-gc-helper-test.c` forces capture retry in a 512-byte heap, verifies live
payload and Fault preservation, genuine exhaustion, impossible snapshots, foreign
context fallback and quiescent retry. Existing stack-capture tests cover truncation
and owned snapshots after source frames return. The collection policy at compiler
boundaries is unchanged. Broader collection pacing and collector replacement remain
separate milestones, evaluated against general runtime workloads as well as HTTP.

### Adaptive address index (2026-10-13)

The collector retains its 256-entry index for at most 2,048 validated spans. Larger
populations use a temporary 4,096-entry index through a non-inlined helper. This
reduces repeated header walks while retaining exact interior/base-pointer checks
and all validation, marking, conditional ownership and cleanup semantics. The
dense path costs another validated heap walk and 32 KiB of stack; no allocation
or collection pacing changes are involved. See the [graph matrix and tradeoffs](../../../benchmarks/gc-index/README.md).

### Allocation validation and first-fit search

Allocation now combines full span validation and first-fit selection in one walk.
Finding a hole does not terminate validation: a corrupt later block rejects the
call before heap/output mutation. Overflow and malformed-heap precedence remain
unchanged. [Allocation-only evidence](../../../benchmarks/gc-allocation/README.md)
measures the gain and records the remaining linear validation cost.

### Explicit helper temporary values

`neoclr_gc_allocate_rooted_values_v1` extends admitted allocation retry with up to
16 explicitly supplied temporary managed values. The collector sees a stack-local
root record only during retry collection; it is not added to the logical call
stack. No persistent handle, allocation or callback is involved. The caller must
still provide a complete current boundary and protect all other native temporaries.
The returned result must be published before another collecting call.

Native GC code generation selects `neoclr_string_concat_rooted_v1`,
`neoclr_string_join_parts_rooted_v1` and `neoclr_string_slice_utf8_rooted_v1`; their signatures retain the existing arena
argument and require its identity to match the active frame's context. Concatenation
roots both inputs; joining roots the parts array and separator, with normal array
tracing retaining initialized elements. All validate lengths before their sole
allocation and finish copying without another collection. UTF-8 slicing retains
the complete source object, including when the copied bytes begin inside it. Range
and UTF-8-boundary errors are returned before allocation and do not collect. They never retry a whole
operation. Non-GC code and existing native helper callers retain noncollecting v1
entry points. New generated native objects require a matching host with these
private symbols; old objects remain usable with the new host.

The native backend owns this specialization. Raven's Runtime Contract options,
CLI bridge metadata, service signatures and ordinary .NET lowering do not change.
No additional bridge encoding or metadata information is introduced or lost.
Pressure tests intentionally omit helper inputs from caller roots to check local
protection; compiled tests exercise both GC and non-GC paths and preserve interpreter
results/faults. General helper coverage and global pacing remain incomplete.

Remaining admission audit:

| Family | Required protection before opting in | Current policy |
| --- | --- | --- |
| Int32/Int64/UInt64 formatting | Native stack scratch; complete caller boundary; publish result before next allocation | Generated GC calls use rooted retry |
| Byte/scalar array allocation and reservation | No managed inputs; finish header, zeroing and initialization flags before publication | Generated GC calls use rooted retry |
| Reference/string/function arrays and reserved records | No managed inputs; finish headers, null payloads and flags before publication | Generated GC calls use rooted retry |
| Other scalar formatting | Complete caller boundary; audit intermediates | Noncollecting |
| Managed UTF-8 encode/decode | Retain source text or full byte-array owner through allocation/copy | Generated GC calls use rooted retry |
| Lexical paths and ordinal replacement | Retain every string read after allocation; no-op replacement preserves identity | Generated GC calls use rooted retry |
| Unicode casing, access/slicing and character assembly | Retain source text or owning array through allocation/copy | Generated GC calls use rooted retry |
| Unicode sequence construction | Retain source and partially initialized array; publish each element before the next allocation | Generated GC calls use rooted retry |
| Native process arguments | Temporary construction frame retains partial array; native argv/conversion scratch remains stable | Generated GC entry uses rooted retry |
| POSIX file input | Close descriptor before allocation; keep malloc-owned bytes alive; never replay read | Generated GC calls use rooted final decode; Windows binding remains excluded |
| Socket/TLS/crypto adapters | No managed allocation within adapters; preserve caller buffers, host roots and native resource ownership | Noncollecting by contract; no allocation retry to migrate |
| PostgreSQL | Managed Raven protocol over socket/encoding APIs; no native SQL allocator | Uses managed roots and underlying socket ownership |
| Culture/time-zone text | Retain captured native preferences/static version bytes through sole output allocation | Generated GC calls use rooted retry |
| Intern/object helpers | Audit intermediate managed values | Remaining allocation audit |
| Constructors / reflection invocation | Retain initialized receiver and every native result scratch until publication | Existing explicit generated roots; admission migration still needed |

These are coverage gaps, not permanent lifetime semantics. The existing .NET
allocation comparison remains applicable: collecting to satisfy an allocation is
normal, while exposing unrooted native intermediates to that collection is invalid.

UTF-8 slice follow-up (2026-10-13): the native pressure test includes multibyte
scalars, embedded NUL, a source omitted from caller roots, real live-heap exhaustion,
unchanged failure output, range-error precedence and no active caller boundary.
All 20 local GC/root executions pass with undefined-behavior/bounds sanitizers.
Compiled GC and non-GC consumers each compare 135 inputs with interpreter results
and faults, and the GC object imports the rooted symbol. This internal UTF-8 helper
does not change public character/grapheme indexing or slicing semantics. The
existing CLR comparison motivates protecting borrowed native references; this
slice adds no collector policy, moving objects or public API.

The [platform follow-up](https://github.com/marinasundstrom/neoCLR/actions/runs/38425599711)
at `38b2240d2` passes the 20 Windows GC/root executions and both platform socket
contracts. Reports are retained with the [allocation/admission evidence](../../../benchmarks/gc-allocation/README.md#utf-8-slice-admission-follow-up).

Integer formatting follow-up (2026-10-13): generated native-GC calls, including
boxed Int32 display, select the private `*_to_string_rooted_v1` helpers. Formatting
finishes in native stack scratch before the sole managed allocation; retry repeats
only allocation, and copying/publication cannot collect. Existing arena-only entry
points and non-GC emission remain noncollecting. A valid active boundary must own
the supplied arena; no boundary or a foreign arena returns RuntimeError. True live
exhaustion still faults and preserves the output. This follows CLR's requirement to
protect managed values across collecting allocations; there is no managed API,
formatting, fault-recovery or general pacing change. Newly generated GC objects
require a matching native host; existing objects remain compatible.

Local qualification passes all 22 native GC/root executions, relocated GC/non-GC
package checks and four focused compiled consumers (including 30 wide-integer
interpreter comparisons). [Retained evidence](../../validation/runtime-product/integer-formatting/README.md).
The hosted macOS/Windows gate also passes all 22 GC/root checks, socket contracts
and package checks; the evidence records its exact revision and scope.

Primitive-array follow-up (2026-10-13): generated native-GC byte allocation,
byte reservation and scalar-array construction select private rooted entry points.
They retry only storage allocation at the complete caller boundary, initialize the
header/payload/initialization map without reentry, and publish before any further
collection. Validation errors still precede allocation, and real live exhaustion
remains a terminal resource fault. Reserved elements remain unreadable until written;
ordinary arrays preserve zero/default initialization. Atomic GC storage prevents
pointer-shaped numeric payloads from retaining managed objects.

Existing native entry points, non-GC emission and UTF-8 encoding's borrowed-input
caller remain noncollecting. Reference/string/record arrays are outside this audit.
As with CLR array allocation, a collecting allocation requires the caller's managed
references to be visible; the private layout and native symbols are neoCLR details,
not a new language contract or public API. No Raven Runtime Contract, bridge
metadata or .NET behavior changes. Newly generated GC objects need matching native
hosts. General pacing remains disabled pending the remaining helper and idle audit.

Local qualification passes 23 GC/root checks, GC/non-GC relocated-package checks
and 107 compiled byte/scalar comparisons with interpreter results/faults.
[Retained evidence](../../validation/runtime-product/primitive-arrays/README.md).
The hosted macOS/Windows gate also passes all 23 GC/root executions, socket
contracts and relocated-package checks; macOS profiler attachment passes.

UTF-8 codec follow-up (2026-10-13): generated native-GC encode/decode calls use
private rooted symbols. Encoding retains its source String while constructing a
byte snapshot; decoding retains the complete source array while copying from its
interior. Validation finishes before the sole allocation. Reserved-slot failures
precede UTF-8 validation, and malformed UTF-8 returns the existing error without
collecting. Copying and result publication perform no further collecting calls.

The current nonmoving collector keeps these borrowed addresses stable. This does
not establish moving-GC pinning semantics. As with CLR native helpers, the owner of
a borrowed managed buffer must remain visible across a collecting allocation.
Raw native-buffer decoding and existing native callers retain noncollecting entry
points because their owner/lifetime contracts have not been audited. No encoding
format, snapshot mutability, public API, Raven metadata or collection policy changes.
New GC objects require the matching native helper symbols. Broader encoding,
reference-array construction and scheduler idle integration remain separate work.

Local qualification passes 24 GC/root checks, package checks and 14 focused
compiled tests. [Retained evidence](../../validation/runtime-product/utf8-codecs/README.md).
Hosted macOS ARM64 and Windows x64 qualification passes all 24 GC/root executions,
socket contracts and relocated-package checks; macOS profiler attachment also passes.
The retained evidence identifies the tested revision and source hashes.

Reference-array follow-up (2026-10-13): string/function arrays, nominal reference
arrays and reserved records now select rooted allocation in GC code. No managed
input is passed into construction; zeroing, layout headers and initialization flags
complete before publishing the result without reentry. Traced storage kinds remain
unchanged. Existing native symbols remain noncollecting. This follows the CLR
principle that managed references must be observable at collecting boundaries;
private neoCLR storage and helper symbols remain implementation details.
The pressure fixture checks referent retention through the newly published array,
context rejection, unchanged failure outputs, live exhaustion and final cleanup.
All 25 macOS/Windows GC/root checks, socket/package checks and 14 local compiled
array tests pass. See [retained qualification](../../validation/runtime-product/reference-arrays/README.md).

Borrowed path/replacement follow-up (2026-10-13): generated native-GC lexical path
combination and file-name extraction retain the string owners of copied byte ranges.
Ordinal replacement retains source, search and replacement strings through its
single allocation retry; all three are read during the copy pass. Validation and
no-op identity behavior precede allocation. Native entry points stay noncollecting.
The nonmoving collector preserves byte addresses; this is no moving-GC pinning
contract. Unix lexical path semantics are unchanged on both host platforms.

The remaining Unicode kernels in `string-unicode.c` are explicitly noncollecting:
casing reads its source in two passes; grapheme access/slicing copy an interior
range; `StringFromChars` reads an array and its elements; sequence construction must
retain both its source and a partially filled result across each element allocation.
The Rust visitor borrows source bytes during callbacks, so a future moving collector
would additionally need a stable-borrow/pinning contract. Other encoding kernels and
native file/network/TLS/SQL producers require their own owner/scratch audit; no
blanket change to the raw allocation service is safe. General pacing stays disabled.

Path/replacement qualification: all 26 GC/root checks, sockets and relocated-package
checks pass on macOS ARM64 and Windows x64. Four local compiled path/replacement
tests compare interpreter behavior, including GC/non-GC replacement and fault
cleanup. See [retained evidence](../../validation/runtime-product/borrowed-text/README.md).

Unicode follow-up (2026-10-13): rooted casing, grapheme access/slicing and character
assembly retain the complete source owner during their sole allocation retry.
Sequence construction publishes a temporary frame for its source and initialized
result-array pointer; each new grapheme is copied and inserted before another
allocation. All exits remove the frame, including partial-result exhaustion.
The current nonmoving collector keeps the Rust visitor's borrowed source stable;
a moving collector would require an explicit pinning or copied-buffer contract.
Like CLR native helpers, these adapters must report every live managed reference at
collecting boundaries. The private frame and UTF-8 layout remain neoCLR details.

The earlier noncollecting Unicode audit above is superseded for generated GC calls;
original native symbols remain noncollecting. Public UTF-8 and strict ASCII encodings
in `Encodings.rvn` already compose the audited managed UTF-8/byte-array operations;
there is no separate native ASCII codec needing a new allocation path. Incremental
managed encoder/decoder behavior retains its existing tests and is not redefined.
Native producer scratch/results and general pacing remain open. All 27 macOS/Windows GC/root checks and socket/package gates pass, alongside six
local compiled character tests. See [retained evidence](../../validation/runtime-product/unicode/README.md).

Native-producer follow-up (2026-10-13): process arguments now own a temporary frame
before guest startup, retaining the partial array through every native-byte decode.
The new decoder explicitly requires stable native-owned bytes; raw managed interior
pointers remain prohibited. This does not widen the old raw decoder's contract.
No input acquisition or external operation is replayed to retry an allocation.
See [process-entry ownership](../../native-entry-arguments.md).
File/network/TLS/SQL producers remain noncollecting pending their own result/handle
and cancellation audits. Their caller scratch and registered native resources cannot
be assumed to have the process-argument lifetime.

Process-argument qualification passes all 28 GC/root checks plus socket/package
checks on macOS ARM64 and Windows x64, including native UTF-16 conversion. Two
local compiled entry tests also pass. [Retained evidence](../../validation/runtime-product/native-arguments/README.md)
identifies the tested revision and scope; general pacing remains disabled.

Native-producer audit correction (2026-10-13): the earlier blanket pending entry for
file/network/TLS/SQL conflated distinct layers. Reviewed current implementations:

- `file-input.c` reads into native malloc/realloc storage and closes the descriptor
  before decoding. Its new generated-GC entry retries only managed output allocation.
  The native bytes remain owned until decoding returns; the managed path is no longer
  read after acquisition. Original native/non-GC calls remain noncollecting. The
  compiler already excludes this POSIX binding on Windows; this slice does not add
  Windows file input. Counted read/close tests establish no I/O replay on macOS.
- `socket-listener.c` allocates native operation buffers, copies sends, and retains
  receive destinations/callbacks via host roots. Completion copies into the existing
  managed array before releasing the buffer/root. Cancellation and disposal use the
  same release paths. It does not allocate a new managed result inside the adapter.
- `tls-managed.c` validates existing arrays, writes pump output into those arrays,
  and attaches native provider handles to managed owner tokens. Provider creation
  uses native allocation; resource attach failure releases the provider handle.
  Calls do not collect or invoke managed code while borrowing array bytes.
- `cryptography.c` synchronously fills caller-provided arrays and retains no borrowed
  pointers. It has no managed allocation needing retry.
- `runtime/raven/providers/System.Data.PostgreSql` implements the wire protocol in
  managed Raven over sockets and encoding. There is no native SQL result allocator.

This is a source-level ownership audit of unchanged socket/TLS/crypto/SQL behavior,
not fresh TLS/SQL workload qualification. Existing socket gates remain relevant;
TLS resource and provider tests remain in `scripts/test-native-security.py`.
Like CLR interop adapters, noncollecting native calls can borrow managed buffers
only while the runtime excludes collection/reentry; pending operations instead need
explicit ownership. Native allocation failure is not automatically a reason to run
the managed collector or replay an external operation. Culture/time-zone snapshots,
intern/object construction and actual scheduler idle integration remain separate
admission work before general pacing promotion.

Producer qualification passes all 29 GC/root executions and socket/package checks
on macOS ARM64 and Windows x64. The shared native-buffer decoder runs on both;
the counted POSIX file test runs only on macOS. Two local compiled file tests also
pass. [Retained evidence](../../validation/runtime-product/native-producers/README.md)
records the exact revision and platform scope.

Native preference follow-up (2026-10-13): generated GC calls use rooted culture-name
and time-zone-version output allocation. Culture lookup retains its Windows native
buffer or CoreFoundation snapshot through conversion and releases it on allocation
failure and success. No host query is repeated as part of allocation retry. The
pinned time-zone version borrows static native bytes. Existing native/non-GC entries
remain noncollecting, and lookup/offset APIs remain allocation-free. Linux culture
support requires an explicit implementation rather than falling through to macOS.
This preserves the .NET-style separation between host preference acquisition and
managed result ownership; private UTF-8 storage and rooted helper symbols are
neoCLR implementation details. Generated objects require matching native helpers.
Tests use real host preferences and the pinned time-zone library, force exhaustion
and retry, reject missing/foreign boundaries, preserve outputs and reclaim results.
The preference comparison assumes no concurrent user preference change during the
short test. All 30 local macOS ARM64 and hosted Windows x64 GC/root checks pass,
alongside local GC/non-GC generated execution and Windows socket/package checks.
The hosted macOS job remains queued; this is not a complete hosted matrix. See
[retained evidence](../../validation/runtime-product/native-preferences/README.md).

String interning follow-up (2026-10-13): generated GC bindings select a rooted
helper that explicitly retains the input while allocating its pool entry. Existing
entries remain collector roots until the admitted entry reset; equal byte strings
(including embedded NUL) return the existing identity. Lookup and quota failure
need no allocation boundary; a new entry requires the matching active context.
Allocation initializes the entry before publication without callbacks. The old raw
helper remains noncollecting. This preserves the CLR-style intern identity/lifetime
principle within neoCLR's existing bounded, per-entry pool rather than widening it
to a process-global pool. Quotas and fault behavior are unchanged. Forced-pressure
tests check a source absent from caller roots, retained live exhaustion, unchanged
failure outputs, duplicate lookup and entry reset. General pacing remains disabled.

Object construction follow-up (2026-10-13): generated GC object allocations now
use rooted retry. The helper has no managed input and completes the type header
and zeroed payload before returning, without collecting calls in between. The
IL-boundary spill retains pending constructor arguments and bound-function receivers
before the allocation. The generated constructor path publishes its new receiver
before entering managed constructor code; scalar boxing and callback descriptors
finish their scalar/reference writes without an intervening collection. Existing
raw/non-GC allocation remains noncollecting. Invalid arena/output arguments fault
without dereference. This follows CLR's requirement to report live references at
collecting allocation boundaries; neoCLR retains its own nonmoving arena/root ABI.
Pressure tests cover initialized payloads, receiver retention, exhaustion, unchanged
outputs and reclamation. A compiled consumer forces collection before and inside
a constructor, then forces callback-binding collection and reads its reference argument through
the bound receiver; the non-GC/interpreter controls
return the same result. This closes the named intern/object helper audit, not the
separate scheduler-idle or managed pacing qualification.

Intern/object qualification: all 32 local macOS ARM64 and hosted Windows x64
GC/root executions pass, along with Windows socket/package checks and the local
compiled consumers described above. The current hosted macOS job remains queued;
the preceding preference job was cancelled without steps. See [retained evidence](../../validation/runtime-product/native-object-lifetimes/README.md)
for exact revision, commands, hashes and scope.

## Scheduler idle admission — 2026-10-13

Native scheduler acquire now collects at its validated boundary before returning
Idle or Pending, before the host waits or exits a drain. It never collects while
transferring a ready callback into the delivery ticket. Pending operations retain
their handles; abandoned owned stacks and resources are reclaimed exactly once.
Invalid acquisitions neither collect nor replace the caller's ticket. Native hosts
must root managed values reused across acquire; the entry-delivery test now does
so explicitly. The lower-level poll/step and OS wait remain noncollecting.

This is runtime integration, shared by native JIT/AOT hosts, not a managed scheduler
API change. Unlike CLR's independently scheduled GC, this single-owner collector
uses a synchronous safe boundary. It guarantees cleanup before waiting at the cost
of a full collection on each non-ready acquire, including repeated pending polls;
further throttling needs a reliable reachability-change signal. The interpreter's
existing allocation-pressure/completion policy is unchanged. Local macOS socket
contracts cover pending timer retention, owned suspended-stack abandonment, idle
cleanup, fault delivery and invalid boundaries; hosted Windows qualification follows.

## Integrated opt-in pacing — 2026-10-13

Use matching current generated objects with audited rooted helper imports.
Build the root observer with `NEOCLR_NATIVE_GC` and `NEOCLR_GC_PACED` to select
the runtime-owned boundary policy. Stress collection remains the default for
installed JIT/AOT hosts; there is no new managed API or CLI option. This replaces
the isolated experiment's policy copy for opt-in builds, retaining all generated
root publication and validation. Collect after 16 allocations or 64 progressing
boundaries, on context changes or an unexpected collection epoch. Rooted allocation
retry handles large requests and fragmentation; explicit collection remains forced.
Scheduler Idle/Pending acquisition forces collection and resets the pacing budget.
Repeated pending polls still collect; these thresholds do not promise a wall-clock
latency bound or independently running collector. No concurrent execution is added.

The .NET comparison is allocation-pressure budgeting and safe-point root reporting,
not a copy of its generational/concurrent collector. The benefit is fewer redundant
full sweeps; costs are increased retained arena extent and synchronous pauses. The
16/64 thresholds are provisional. The interpreter already has an independent
allocation-pressure/completion policy; this change neither replaces it nor claims
equal collection timing across execution modes. Default promotion still requires
managed HTTP/runtime-async, sustained-load and JIT/AOT qualification.

The focused gate checks the actual paced root observer, pressure admission,
no-allocation reachability changes, explicit collection epochs, missing/foreign
boundaries and the real scheduler in both modes. The constructor/callback consumer
runs non-GC, stress-GC and paced-GC variants against the interpreter result. The
closed benchmark can now use `evaluate-rooted.py --integrated`; the historical
experiment remains reproducible without that flag. Platform evidence follows.

Retained-stack follow-up: acquisition can expire owned fibers after the async pump
has counted them. The macOS retained-stack carrier now reaps expired owners and
recomputes waiting/ready state after acquisition, avoiding a false unresolved-work
fault. A real suspended-body regression fails before this fix and passes afterward.
Its ownership and idle-drain tests now run in the macOS GC gate; the carrier is not
a Windows implementation. Shared scheduler/collector tests run on both systems.

[Qualification](../../validation/runtime-product/native-scheduler-pacing/README.md)
retains passing hosted macOS/Windows GC/root, stress/paced scheduler and package
checks. The integrated closed workloads improve 3.61–4.14× on those hosts with
approximately 23% larger peak arena extent and complete final reclamation. These
are not managed HTTP results; the separate retained-stack follow-up is identified
by its own revision and evidence.


## Root metadata search hints (2026-10-18)

The shared native root-publication adapter now keeps a bounded thread-local marker
search hint. Every publication still checks the current plan length and marker
bytes, all live storage/transient descriptors and fault contexts. No validated
pointer identity is cached; mutable metadata and reused image addresses remain
safe under the existing readable-string contract. Frame ABI, root ownership and
stress/paced collection policy are unchanged. Cost: 1 KiB per native thread.

[Serve validation and matched Go measurements](../../../benchmarks/http-server/serve/root-validation/README.md)
show 6.5% less CPU per completion in the clean 400/sec pair, with noisy repeats and
all admission failures retained. This is native GC infrastructure improvement,
not an HTTP-specific fast path; gains in other workloads remain unmeasured.
Interpreter-side GC is unchanged. Green Tea page-local tracing research is recorded
there as a separate possible direction, not an implemented collector replacement.


## Explicit runtime-owned allocation scope (2026-10-23)

The private acquire/release pair can lend an empty arena exclusively to one native
thread. While active, only runtime allocation/collection may modify its descriptors
or shape. A maintained cursor skips occupied prefixes and permits append without
a descriptor walk when no free blocks remain. The external checked allocator still
validates all headers before mutation. GC and release validate the entire heap;
shape changes are rejected. Release clears the scope even after reported corruption,
but does not release roots or repair the heap.

The scope adds five words of TLS bookkeeping, no per-object fields, and preserves
the context/payload/compiler ABI. It is explicitly opt-in in the source HTTP host
with NEOCLR_GC_OWNED_HEAP, for isolated per-thread arenas only; shared-object worker
GC remains on the checked path. See [contract, research comparison, tests and
measurements](../../../benchmarks/http-server/primitive-comparison/owned-heap/README.md).
