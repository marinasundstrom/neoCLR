# Bounded worker-to-owner completion delivery

Private POSIX experiment enabled by NEOCLR_WORKER_DELIVERY. Existing shared pool
captures/results stay rooted in their existing heap. A coalesced nonblocking pipe
wakes the owner's socket wait; bounded dispatch consumes tickets and completes
awaited operations on the owner. No foreign managed pointer travels through the
wake signal. The generic source is call-scoped and must outlive the pump.

The same native transport is intended for JIT/AOT services, but this slice does not
admit arbitrary capturing HTTP handlers or implement general managed Task.Run.
It preserves the existing pool's shared identity rather than replacing captures
with copies. The [.NET/Go comparison and task model](../../design/go-inspired-scheduling.md)
remain the semantic reference: Task.Run schedules; awaiting a pending Task parks
the awaiting execution. This is internal machinery, not a deliberate API divergence.

From the repository root on macOS with Xcode:

    SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk python3 docs/experiments/worker-delivery/validate.py --output target/worker-delivery-check
    SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk python3 docs/experiments/native-worker-pool/validate.py --worker-delivery --compiled-workers --trace-workers --local-queues --output target/worker-delivery-existing
    python3 docs/experiments/worker-delivery/profile.py --binaries target/worker-delivery-check --output target/worker-delivery-profiles

Output directories must be new. The trace-only binary writes trace.json in its
working directory; the validator isolates/renames it. Profiles are separate from
timed runs. [Results and limits](../../validation/worker-delivery-20261028/README.md)
retain all cohorts and normal-build assembly equivalence.

Descriptor consumers only poll; they must not drain or close the borrowed fd.
Dispatch notification is exactly once, but is not result consumption. If take
cannot allocate a consumer root, the owner must retain/retry that ticket explicitly;
a delivered ticket does not produce another notification. Closing a pool still
requires cooperative callbacks to return and can block; this change does not add
forced shutdown. Reentrant dispatch and destruction during dispatch stay rejected.

HTTP integration and a [fresh post-integration benchmark](http-plan.md) are the
next gate, not automatic consequences of the native timing gains.

## Generated closure delivery qualification

`generated.py --bundle <bundle> --libraries <matching-lib> --codegen <codegen>
--output <new-directory>` compiles `Generated.rvn` (the shared-pool fixture with
increased allocation pressure) and runs `generated-chain.c` under sync/concurrent
GC, optimized/UBSan.
Use the explicit SDKROOT above. The same AOT object is linked into all four cases;
UBSan covers the C adapter/runtime, not generated machine code.

The owner chains 64 awaited deliveries through a two-worker, one-ticket pool with
budget-one dispatch. Generated callbacks mutate an aliased cyclic graph, replace
edges, keep displaced locals alive, allocate 2,500 boxed values, and advance shared
statics. Every callback must allocate and collect. Successful submissions drop the
caller root: the ticket retains the closure until rooted result adoption. Job 33
faults deliberately; its separate recovery root permits subsequent jobs, because
failure discards result references. Collection precedes diagnostic copying/take.
Only a native status/opaque pool root reaches the adapter; the owner Task completes
with unit, never a foreign managed pointer.

This qualifies serialized generated callbacks on the pool heap, alongside a parked
owner on a different heap. It does not qualify parallel guest execution, managed
HTTP closures, Task fault mapping, cancellation of generated callbacks, socket
operations inside workers, or throughput. Existing native cancellation/backpressure
checks remain separate evidence. [Results](../../validation/worker-delivery-generated-20261028/README.md).

**Collector boundary:** the pool bridge borrows an arena without acquiring it as
an owned heap on the worker. Its generated callbacks therefore use synchronous
collection even in a concurrent-collector build. The test asserts zero worker
epochs and verifies actual owner epochs settle before teardown. Increasing from
500 to 2,500 allocations per callback did not activate worker epochs; this is an
ownership/admission limitation, not proof that the workload cannot benefit from
concurrent marking. Do not interpret collector build selection as concurrent
worker-heap qualification.

## Exclusive live-arena adoption prerequisite

A separate `NEOCLR_EXCLUSIVE_HEAP_ADOPTION` experiment validates and adopts an
existing unowned arena under a caller-proven exclusive lease. It rebuilds ownership
metadata, rejects current TLS roots/resources and releases/joins the marker before
handoff. [Contract, sync/concurrent checks and cost screen](../../validation/heap-adoption-20261028/README.md)
cover sequential native-thread transfer. Run `validate-adoption.py --output <new-dir>`
with the explicit SDKROOT above.

This primitive is not enabled in the generated worker bridge. That bridge still
lacks an admitted live-store-barrier compiler profile, and small-heap pressure can
prevent epochs even after ownership registration. The next prerequisite is a
restricted barrier-instrumented callback contract; copying HTTP captures or merely
registering their heap is insufficient. Persistent ownership metadata is a later
cost-reduction candidate, subject to the HTTP integration gate.

## Restricted generated owned callbacks

`NEOCLR_WORKER_OWNED_CALLBACKS` now connects the exclusive lease to the serialized
callback bridge. Its compiler profile instruments live stores and rejects owner-bound
async/socket/TLS services. The compiled image and collector contracts must match
before bootstrap; marker release precedes root withdrawal. Parallel participant
mode cannot be combined with this lease.

[Accepted generated checks](../../validation/worker-owned-callback-20261028/README.md)
run 64 chained deliveries and complete 64 worker epochs in the concurrent cases,
with shared cyclic captures/static state and fault recovery. Existing four-worker
cancellation/admission/teardown checks pass under both collectors. Use
`validate-owned-callbacks.py` with the matching compiler library test executable and
bootstrap selection command. This supersedes the earlier worker-fallback limit
only for this private profile; ordinary callbacks and HTTP owner heaps are unchanged.
The author's "good enough for now" scope defers metadata/queue micro-optimization
until useful HTTP integration and fresh measurements establish a need.
