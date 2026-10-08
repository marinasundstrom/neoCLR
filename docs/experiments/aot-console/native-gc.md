# Basic native collection experiment

The author requested continuing until basic GC also works for native compilation
(2026-10-08). This work extends the ARM64 AOT experiment; it does not change the
interpreter collector or establish a production native hosting ABI.

## Kernel foundation

`native-gc.c` implements nonmoving mark-and-sweep inside the host's bounded arena.
A 32-byte allocation descriptor records block span, logical payload size, allocation
kind and mark state. Reclaimed adjacent blocks coalesce and allocations reuse holes;
all payload addresses remain stable. No malloc or shared managed runtime is required.
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

Collection runs only at complete pre-operation boundaries, never in allocation
services or between result writes. That avoids collecting native service temporaries
and partly initialized data. The initial stress policy collects at every such boundary;
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
and their services never collect: temporary UTF-8 pointers, output buffers and partially
written objects therefore cannot be interrupted by this collector. After an allocation,
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
