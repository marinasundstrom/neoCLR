# Acknowledged worker cancellation reaches Raven await

Development experiment, 2026-09-23. This connects per-job runtime cancellation to
Promise.Cancel in an isolated Raven worker adapter. It uses the existing native
notification path, real producer interpreters, ordinary Task awaits and managed GC.
The installed Thread/Task APIs and checked-in worker implementation are unchanged.

## Run

From a matching checkout and development toolchain:

```sh
python3 docs/experiments/delayed-copy/verify.py --toolchain-root /path/to/development/bundle --consumer-root docs/experiments/worker-task-cancellation --adapter-source docs/experiments/worker-task-cancellation/Workers.rvn
```

The harness compiles Workers.rvn against bootstrap declarations, replaces only its
fragments in a temporary System library, then compiles the consumer against the
normal reference core. Passing the adapter explicitly is essential: compiling the
consumer alone against the installed worker implementation does not run this test.

## What the product shows

Four consumers start before Main returns: cancelled/successful jobs on both dedicated
threads and the pool. Each consumer retains a four-byte destination. A cancelled
await must propagate before Apply mutates that destination. Successful siblings copy
`Hi` into the middle two bytes. An unrelated callback runs first. Allocation pressure
while consumers are pending exercises actual GC; all outcomes appear once, in any
host completion order, and the final guest live count is zero.

The test adapter automatically requests cancellation of a job whose input is
`"cancel"`; the designated producer loops in managed code until its runtime token is
observed. **That sentinel is fixture wiring, not a proposed API or installed Thread
behavior.** Requests happen immediately after notification registration. This does
not test a UI cancellation gesture, public tokens or arbitrary cancellation timing.
Instruction/time limits are test watchdogs; they do not establish cancellation latency.

WorkerCompletion calls JoinWorkerResult only after its notification dispatches.
String completes the Promise; acknowledged Void cancels it. Other native failures
fault before a value is returned. No task failure state, scheduler or suspension
mechanism is added. Normal managed callers cannot invoke the bootstrap services.

Additional fixtures check that an explicit producer Fault with the text
`execution cancelled` still has UserFault classification and never runs the consumer
continuation, and that the normal reference core rejects RuntimeServices calls.
This guards the distinction between runtime cancellation and guest-selected text.

## Comparison and limits

Reuse the [.NET cooperative cancellation comparison](../pending-read/README.md#net-comparison-and-alternatives)
and [per-job runtime contract](../../isolated-workers.md#per-job-cancellation-experiment-2026-09-23).
This implements acknowledgement-before-Task-cancellation using the existing Raven
state-machine and Promise model. It demonstrates missing neoCLR integration rather
than an advantage over .NET. Compared with the earlier deferred-discard adapter,
the managed producer is now requested to stop instead of being allowed to finish
normally. Native calls still cannot be interrupted, and shutdown has no time bound.
Owned string transport still costs copying/encoding; this is not a byte I/O backend.
Explicit TaskQueues and a public operation-cancellation handle remain open.

An async parameter originally named `state` collided with generated state-machine
field naming during neoIL verification. Naming it `destination` avoids the collision
in this tested consumer. This is a deferred compiler/importer integration issue,
not a reserved public naming rule. It needs a reduced general metadata test before
choosing a compiler or importer fix. No Raven repository changes were made here.

## Next slice

Choose and test queue affinity for the notification adapter before generalizing it:
construction, submission and completion must agree on the consumer queue, or reject
an unsupported explicit queue early. Keep producer cancellation wiring experimental
until a public request/ownership contract is selected. Byte payload accounting and
an actual filesystem producer follow those boundaries.
