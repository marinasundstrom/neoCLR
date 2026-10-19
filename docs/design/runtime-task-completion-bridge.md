# Runtime task completion bridge — investigation direction

Author direction, 2026-10-19: HTTP should not allocate Promises indiscriminately.
The author asks about a ValueTask-like design and wants a different bridge for
runtime operations. This document records the next investigation, not a shipped
API or an approved representation change. The author subsequently clarifies that
Promise optimization remains useful, while its allocation pattern should not be
the primary HTTP mechanism; they explicitly question whether Promise is actually
the cause of low throughput. The profiles establish a contributor, not the dominant
capacity limit. Measure saturation and the complete proposed bridge before
attributing a capacity ceiling to this pattern.

## Current costs and ownership

Task<T> currently references Promise<T>. Promise owns settlement, result storage,
callbacks and the Task wrapper. Socket transfer operations add a completion object
holding a Promise and a native operation handle. RuntimeAwaitProvider.Body<T> also
owns a Promise, and awaiting a pending Task creates a CompletionSignal plus an
owned runtime handle. The conventional AsyncTaskMethodBuilder<T> owns a Promise.
These are concrete layers, not a requirement that public Promise back every await.

A pending operation does require completion state, retained buffers and reachable
continuations. Promise is useful as an explicitly producer-controlled public
adapter, including callback-to-awaitable adaptation. Eliminating its wrapper does
not eliminate that underlying state. A bridge should combine ownership with the
operation state rather than rename Promise to another equally allocating wrapper.

## Desired direction and alternatives

Prefer ordinary async methods and HTTP/socket operations to use runtime-managed
completion state. Keep public Promise for explicit producer control. Preserve
Task's existing multiple-reader/repeated-await behavior unless a separate contract
is explicitly chosen. Continue using Memory/ReadOnlyMemory for retained transfers;
borrowed Slice values must not escape into pending operation storage.

Investigate these stages independently:

1. Consolidate completion state and remove the Task/Promise pair from core-generated
   async paths. Audit immediate results/errors/cancellation as well as pending work.
   Existing Task.FromResult still allocates through Promise; replacing call-site
   spelling alone is not an optimization.
2. Let a socket operation's runtime completion record project an awaitable directly.
   Avoid a fresh managed producer wrapper and avoid an extra completion-signal
   adapter where the runtime already knows which frame/continuation to resume.
3. Add an allocation-free already-completed representation if measured synchronous
   completion frequency justifies it. Compare a unified Task representation against
   a distinct value awaitable; do not introduce two APIs merely to mirror .NET.
4. Consider reuse only after defining ownership, generation validation and release.
   A stale awaitable must never observe a reused operation's new result. Repeatable
   results may need retained storage even after native handles can be released.

Potential impacts include Raven's explicit Runtime Contract, async lowering,
RuntimeAwait provider, native metadata and JIT/AOT bindings, interpreter execution,
GC ownership, scheduler dispatch and class-library I/O. Keep Raven main's ordinary
.NET behavior separate. Compiler changes require both repositories' integration
documentation and validation; this note changes neither compiler nor contract.

## Semantic and performance gates

Test immediate success/error/cancellation, genuine suspension, completion before
registration, duplicate notification, cancellation acknowledgement races, short
transfers/EOF, deadlines, close/disposal and scheduler affinity. Retain pending
owners, reference/value/union results and buffers across collection. Include
repeated awaits, multiple waiters, late observers, abandoned operations and cleanup.
Never resume a freed frame or recycle state while an observer can still read it.

Use interpreter/JIT/AOT consumers plus both collector qualifications. Count every
allocation in the full operation path, not merely public Promise constructor calls.
Then run uninstrumented HTTP comparisons with all misses and tail latency retained.
Measure native handles and retained bytes too: moving allocations to native memory
is not the same as eliminating them.

The [Promise storage experiment](../../benchmarks/http-server/serve/promise-result-20261019/README.md)
shows why representation matters: inline generic results reduced allocations but
regressed bulk latency; retaining an array initialization marker avoided that large
regression in the tested pairs. This does not rule out inline completed results,
but precise GC layout and large-value copy costs need explicit evaluation.

## .NET reference and deliberate choices

.NET's [ValueTask design](https://devblogs.microsoft.com/dotnet/understanding-the-whys-whats-and-whens-of-valuetask/)
uses inline successful results and can reference reusable asynchronous sources.
It explains the allocation benefit and the stricter consumption constraints of
reusable sources, contrasting them with Task's shareable, repeatable result.
That is a design precedent, not a reason to duplicate legacy overload layers or
silently impose one-shot semantics on neoCLR Task.

A unified small awaitable could simplify API selection, but may enlarge every
return value and complicate sharing/reuse. A separate value type makes restrictions
explicit, but adds API choices and conversions. A reference Task directly backed by
operation state keeps familiar sharing at the cost of some allocation. Select
among these using actual completion patterns and lifetime requirements; no universal
winner or zero-allocation pending-operation promise is established yet.
