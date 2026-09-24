# Runtime scheduling and async suspension

**2026-09-24 · Internal implementation direction; public API remains exploratory.**

The author asked whether ongoing socket work accounts for runtime-owned async and
whether a scheduler is needed instead of task queues. The recommendation is **yes to
an internal scheduling responsibility now**, without introducing a public Scheduler
class or committing to .NET's scheduler API. TaskQueue remains a compatibility adapter
while current Raven state machines are supported. Runtime suspension remains the target;
we should not make that future change depend on a particular callback representation.

This document refines the socket integration sequence in the
[platform roadmap](platform-roadmap.md). It does not require implementing full runtime
suspension before TCP echo. The initial private native-host completion driver is now
implemented; runtime suspension and a public scheduler are not.

## Separate responsibilities

| Responsibility | Meaning | Current implementation / next boundary |
| --- | --- | --- |
| Task / Promise | Observable eventual outcome and producer authority | Preserve Task/Result/cancellation semantics while changing execution machinery. A Task is not a thread or necessarily a runnable job. |
| I/O operation | Resource, buffer, backend request, terminal completion and cancellation acknowledgement | Retain native/managed ownership independently of the consumer's continuation representation. |
| Scheduler | Admit runnable work, arbitrate completion sources, decide the next dispatch, park/wake or yield to the host, and determine quiescence | Private invocation-owned component; current VM code implements parts of this implicitly. |
| Resumption target | Where runnable work may execute | Initially the owning invocation. Future UI/affinity or isolate targets need explicit lifetime and capability rules. |
| Suspended execution | Saved execution state and its GC roots | Generated receiver/delegate today; runtime-owned frame/continuation later. Scheduling must not depend on builder or MoveNext names. |
| TaskQueue | Existing callback storage and explicit queue APIs | Transitional adapter, not the definition of all scheduling or all async work. |

Task.Run submits work for concurrent execution through platform capabilities. It is
separate from awaiting an I/O operation or making an already-suspended computation
runnable. A single-thread event loop can support async progress without CPU parallelism.
A worker-backed Task.Run additionally needs transferable captures/results or a supported
shared-memory model. An internal scheduler cannot make arbitrary closures transferable.
Thread remains explicit and platform-dependent; WebAssembly workers are not silently
relabelled as threads. No Task.Run overload or capture policy is selected by this slice.

## What the current implementation proves, and what it does not

The [TCP VM adapter](experiments/socket-completion/VM-INTEGRATION.md) checks real input,
managed-buffer roots, default-queue callback delivery, busy-queue progress, a pending
worker and terminal cleanup. Its receive binding and socket poller exist only in test
builds. Production worker waits now return control to the invocation every bounded
wait step. This is useful integration evidence, not a complete scheduler.

Current limitations are architectural, not just missing method names:

- The VM recognizes TaskQueue.Drain and Func<Void> return boundaries by System metadata.
  This compatibility behavior must not become the only place a future frame can resume.
- The initial source-order mismatch is resolved by the shared driver below. Its
  cooperative source rotation does not establish preemption or latency guarantees.
- Normal worker library adapters still have queued joins; the delayed-copy notification
  adapter is experimental. Merely adding a scheduler does not make those joins nonblocking.
- Promise captures its queue. A pending await registers MoveNext with the awaited Task,
  so it currently resumes on the producer's queue even if the result Promise belongs
  to the caller's queue. The [affinity fixture](task-contracts.md#scheduling-and-suspension-exploration--2026-09-23)
  records that mismatch; it is not an intended permanent contract.
- No supported runtime representation of suspended interpreter frames exists yet.

## First internal contract

These are proposed implementation rules and validation gates, not new public types.

1. Each invocation owns its scheduling state and all runnable/parked guest roots.
   Backend threads produce owned completion data and wake notifications; they cannot
   run guest code or mutate guest arrays. Initial dispatch stays on the invocation's
   owning thread. Multiple invocations and isolates remain separate ownership domains.
2. An admitted operation retains its buffer/resource state and a resumption registration.
   The registration identifies its owner and destination, independently of whether its
   runnable body is a generated callback or a future saved frame. Do not introduce a
   fake runtime frame type merely to fill out an interface before suspension exists.
3. Completion becomes runnable exactly once on the owner. Move all needed roots into
   ready ownership before releasing pending ownership; move them into active execution
   before removing ready ownership. Account for pending, ready and active resources,
   including completed results awaiting consumption. Completing I/O cannot bypass quotas.
4. Cancellation requests do not establish backend quiescence. An in-flight native
   operation retains resources until the backend acknowledges it or completion wins.
   Late events cannot resume retired work. Close and invocation shutdown use the same
   ownership rules; a noncooperative backend needs a documented shutdown strategy.
5. Source arbitration has one policy at idle and dispatch boundaries. Start by polling
   sources in rotating order, admitting a bounded amount of completion work per turn,
   and executing a bounded batch of ready work before checking completions again.
   This is cooperative fairness, not a latency promise or preemption. Existing callbacks
   must return; an instruction budget/cancellation check still bounds guest execution.
6. An empty ready queue is not completion while live operations can make progress.
   Wake notifications must be durable across the check-for-work / park boundary.
   Native blocking hosts may park; browser/embedded hosts must be able to return a
   pending execution to their event loop. The current 1/10 ms waits are transitional,
   not a portable scheduler contract. Do not use a blocking wait on the browser thread.
7. Continuation destination belongs to the awaiting execution, independently of the
   producer's location. Adopt the owning invocation as the initial supported target.
   Existing explicit TaskQueue behavior must receive a deliberate migration test and
   documentation update; do not silently change it in a socket backend. Whether public
   queue affinity survives, changes or is deprecated remains open.
8. Already-completed awaits may continue inline under the current language behavior.
   External pending completion must enqueue resumption rather than invoke guest code
   from a producer. Specify reentrancy separately from context propagation. Logical
   context, UI affinity and custom scheduler selection are distinct future decisions.

The names above describe responsibilities; they do not prescribe one public class per
row, an ambient scheduler, a thread pool, a work-stealing algorithm or a new namespace.

## Runtime suspension migration

Scheduling chooses runnable work; suspension determines what state is saved and how it
can safely resume. The interpreter should eventually report continue, suspend, return
or Fault to its execution driver. The suspended-state owner must preserve the needed
frames, PC, arguments, locals, evaluation state and logical context, with exact GC roots.
Byrefs, stack storage, cleanup and native-call boundaries require independent verifier
and lifetime rules before they can cross suspension. A callback ticket is not proof that
arbitrary interpreter frames can already be suspended.

The current adapter can turn an admitted runnable continuation into TaskQueue.Post/Drain
frames. A later adapter can enter a retained runtime frame. The I/O producer should
publish the same operation outcome in either case. Test the two adapters against the
same ordering, cancellation and root-transfer requirements when both actually exist.
Keep compiler-generated state machines and builders until that replacement is validated;
they can then be deprecated/removed deliberately. Public Task/Result behavior should not
inherit their storage or builder protocols.

## .NET comparison and tradeoffs

Primary sources reviewed 2026-09-24:

- [.NET 10 TaskScheduler](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.taskscheduler?view=net-10.0)
  provides task scheduling and permits custom implementations. We need its separation
  of policy from asynchronous results; we have not selected its Task-based public
  work-item/customization surface.
- [.NET 10 SynchronizationContext](https://learn.microsoft.com/en-us/dotnet/api/system.threading.synchronizationcontext?view=net-10.0)
  and the [ConfigureAwait FAQ](https://devblogs.microsoft.com/dotnet/configureawait-faq/)
  distinguish continuation placement from Task scheduling and logical context flow.
  An explicit initial owner avoids copying the whole ambient-capture surface now,
  but does not solve UI affinity or supply equivalent compatibility automatically.
- The [.NET runtime-async specification draft](https://github.com/dotnet/runtime/blob/main/docs/design/specs/runtime-async.md)
  describes runtime-supported suspension and constraints on preserved execution state.
  It is explicitly a design draft, not evidence here of a released .NET feature or a
  specification neoCLR has adopted. Its relevant lesson is that replacing compiler
  state machines still requires ownership and resumption rules; it does not remove
  scheduling responsibilities.

Keeping only TaskQueue is cheapest immediately, but extends dependencies on callback
and library names and leaves wakeup/affinity implicit. Copying .NET TaskScheduler plus
SynchronizationContext gives familiar extension points but commits a broad public
model before portable host requirements are known. A private execution scheduler with
an existing-queue adapter adds internal lifecycle machinery now while leaving that
public choice open. Runtime suspension first would avoid some transitional machinery,
but delays the application POC and still requires the same I/O ownership work.

The private scheduler is the recommended next step. Its cost is a temporary adapter and
new state/lifecycle tests; no performance or allocation improvement is claimed. Existing
GC and socket experiments are reusable evidence, not sufficient proof of the full model.

## Next implementation slices

1. **Initial native-host extraction complete:** one private source-arbitration and
   waiting boundary now serves idle and callback dispatch. TaskQueue remains the
   adapter. Focused rotation/wakeup checks and real guest TCP/worker progress cases
   pass. OS socket readiness and nonblocking host yielding remain backend follow-ups;
   there is no public scheduler API.
2. Make continuation destination and operation ownership explicit at that boundary.
   Cover both GC paths, pending-to-ready-to-active root transfer, bounded completed
   outcomes, cancellation/close orderings and explicit-queue migration behavior. Use
   real I/O and actual guest callbacks, not only synthetic scheduler tokens.
3. Attach reusable sockets and Task<Result<...>> producers through that boundary,
   refresh public reference metadata/docs, and run Raven TCP echo. Record how a future
   runtime-suspension adapter will replace the callback side without changing the
   transport producer. Full frame suspension is a later independently validated slice.

## Initial native-host driver — implemented 2026-09-24

[src/scheduler.rs](../src/scheduler.rs) now owns the worker registry, completion-source
arbitration and idle waiting for each invocation. The VM delegates root collection at
both GC paths and polls the same driver at queue quiescence and the existing safe
callback-return boundary. One completion is admitted per boundary. Source order rotates
after each successful delivery; an idle source cannot hold up a ready peer. Source lists
use stack storage rather than allocating a list on every poll.

Worker result publication or sender disconnection signals a shared wake latch after
the channel sender is dropped. Outcomes remain in their channels; the latch is only
a coalesced hint. A signal arriving between an empty poll and parking remains set,
and parking checks that state under the same mutex used by signal publication. The
condition variable may wake spuriously, so progress is always polled again. This is
a native-host implementation choice, not a browser parking contract.

The driver retains a 10 ms timeout because host cancellation has no wake subscription
and the test-only TCP adapter has no OS readiness notification. Thus neither native
socket readiness registration nor fully event-driven cancellation is claimed. The
worker registry's old wait loop remains only as a unit-test helper; production waiting
now belongs to the scheduler. Private worker submission/join access still uses its
existing registry; no generic public work-submission API has been added.

Root transfer into actual TaskQueue.Post frames and existing callback semantics remain
unchanged. Runnable work is still represented by a callback Value. No synthetic saved
frame variant is introduced before runtime suspension can implement it. The new shared
wake state adds a per-invocation allocation and synchronization cost; no speedup claim
is made. Pending/ready quotas, await-site affinity, nonblocking host yielding and real
suspended-frame ownership remain follow-up work.

Validation commands:

```sh
cargo test --lib scheduler::tests
cargo test --lib socket_vm_probe
cargo test --lib workers::tests
cargo test --test workers
```

Four focused scheduler cases check rotation, completion between poll and park, coalesced
signals retaining multiple outcomes, and a cross-thread wake. The real-TCP VM fixture
checks both collection paths, busy queues, progress alongside a pending worker and
terminal teardown. Existing worker tests cover output budgets, cancellation, default
versus explicit queues and callback/root ownership. A separate worker reply test
checks that publication/disconnection is observable before the durable wake. The next implementation slice is
explicit operation/resumption ownership and its affinity migration gates; the current
producer-queue affinity has not changed.

Checked locally on Darwin arm64: 4 scheduler cases, 5 real-TCP VM cases,
11 worker-registry cases (including the reply-order check), and 16 worker integration
cases pass. The combined website build validates 523 pages.
