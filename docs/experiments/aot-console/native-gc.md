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

The initial kernel contract test runs with UndefinedBehaviorSanitizer and bounds checks.
AddressSanitizer on this host stalled during dyld/runtime initialization before `main`;
that unavailable check is not counted as passed.
It covers cycles, an interior-only root, String-array initialization markers, atomic
payloads, fault-message retention, free-block reuse, bounded exhaustion and buffer guards.
Generated-code safepoint integration and the fixed-budget routing acceptance remain next.

## Comparison and provisional choices

.NET's collector traces roots and supports generational collection and compaction;
this experiment only supplies a small nonmoving foundation, with higher retention and
fragmentation costs. See [Microsoft GC fundamentals](https://learn.microsoft.com/dotnet/standard/garbage-collection/fundamentals).
Conservative candidate scanning and allocation-kind distinctions are established
techniques in the [Boehm collector overview](https://www.hboehm.info/gc/gcdescr.html)
(primary sources reviewed 2026-10-08). This implementation does not embed that collector
or scan arbitrary machine stacks/registers. It relies on neoCLR's explicit published
storage and closed entry contract.

Collection is planned only at complete pre-operation boundaries, never in allocation
services or between result writes. That avoids collecting native service temporaries
and partly initialized data. The initial stress policy will collect at every such boundary;
precise object maps and an allocation-pressure schedule remain follow-up work. No speed,
pause-time or production-server claim follows from this design.
