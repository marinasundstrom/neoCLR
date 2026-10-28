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
