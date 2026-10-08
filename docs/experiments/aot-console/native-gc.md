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

A bounded fixed-point mark pass avoids recursion and external mark-stack allocation.
This is intentionally simple and can take quadratic or worse work as the heap grows.
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
