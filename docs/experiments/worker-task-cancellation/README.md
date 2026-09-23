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

## Queue ownership characterization

Affinity.rvn starts a worker from the default queue and awaits its pending Task
inside an explicit caller queue. The async body resumes through the producer's
default queue; observing the async method's result then requires draining the
caller queue. The harness verifies the exact output in affinity.expected.txt.
The async body keeps its bookkeeping in a synchronous owner method. An initial
version with a nested callback capturing async parameters failed strict import
with a state-machine/receiver stack-type mismatch. This remains an unreduced
compiler/importer investigation. A nested Main callback also observed a null captured
result Task; registration now uses a separate synchronous helper. Neither issue is
claimed fixed or treated as a language restriction. See the
[integration findings](../../integration/README.md#queue-ownership-fixture-findings-2026-09-23).
This demonstrates current behavior, not a recommended affinity contract. No API or
installed implementation changes are made. Explicit-queue worker submission is
still outside the experiment's supported cases; no early rejection was added.

TaskQueue remains scaffolding that can evolve when requirements justify it. See
[scheduling and suspension exploration](../../task-contracts.md#scheduling-and-suspension-exploration--2026-09-23)
for the .NET comparison, alternatives and unresolved ownership/progress rules.

## Next slice

Use a concrete pending-operation case to decide continuation ownership/progress
when needed. Keep producer cancellation wiring experimental until a public
request/ownership contract is selected. Byte payload accounting and an actual
filesystem producer follow those boundaries.
