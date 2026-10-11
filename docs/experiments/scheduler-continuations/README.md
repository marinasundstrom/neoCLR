# Continuations independent of the producer queue

Development experiment, 2026-10-11. This source-included internal awaitable separates
managed outcome storage from runtime submission of continuations. It does **not**
change public Task/Promise, introduce a public Scheduler API, or implement runtime
suspension. The existing Raven compiler still generates state machines.

`Completion.rvn` keeps a `Promise<int>` as its outcome store. Its awaiter implements
the existing `TaskAwaiter` contract, retains pending callbacks in a managed list and
submits them through `neoCLR.Runtime.SchedulerSubmit` on completion/cancellation.
Registration after completion also submits asynchronously. An ordinary completed
`await` uses IsCompleted/GetResult directly, avoiding registration and dispatch.

The producer is an explicit TaskQueue containing a callback that faults if pumped.
Success therefore demonstrates that these await registrations make progress without
that queue. Generated builders still use the existing queue adapter for their own
Task results. This fixture exercises a proposed separation, not a migrated library.

## Covered behavior

- Two pending generated state machines await one outcome; both receive its result.
- Await of a completed outcome completes synchronously.
- Explicit completion observation is deferred and delivered once.
- Cancellation resumes the generated state machine and marks its Task cancelled;
  a separate outcome observer checks it without awaiting a cancelled result.
- A producer fault after submitting a continuation remains terminal; the waiting
  state machine cannot resume after that fault. Invocation teardown owns pending roots.

Observers await explicit completion signals rather than relying on relative callback
order. Explicit OnCompleted calls are for deferred-observer/cancellation checks;
ordinary asynchronous flow uses await. Every producer queue contains the pump sentinel.

## Scoped observation cleanup

The internal `Subscribe` adapter returns a disposable `Continuation` for explicit
observation. Ordinary async code still uses await. `use observer = operation.Subscribe(...)`
withdraws the dormant callback at scope exit; early return is covered. Dispose is
idempotent and does not retract work already admitted to dispatch. It does not complete
or cancel the Task and does not make Task or Promise disposable.

Completion skips wrappers deliberately withdrawn through this adapter. It still faults
if a pending wrapper's runtime handle is unexpectedly unavailable; unknown/foreign
runtime handles must not be silently treated as an intentional managed withdrawal.
Submitted/withdrawn flags are owner-thread state, not a thread-safe transition protocol.
The outcome's list retains the small wrapper until completion, but the wrapper holds
no callback reference and cancellation releases the runtime-held callable immediately.

The executable checks 128 normal `use` exits on one pending operation (exceeding the
64-slot capacity without exhaustion), cleanup on return, repeated explicit disposal,
no reactivation, normal completion of a surviving waiter, and no retraction after
activation. The existing terminal-fault consumer remains valid. Terminal faults do not
unwind Raven `use` cleanup; runtime invocation teardown remains the root-release boundary
on that path. This is explicit observation lifetime management, not automatic cleanup
of unreachable/abandoned ordinary awaits, which remains open.

## Ownership and remaining design work

Each `Continuation` captures `SchedulerCapture()` and immediately registers its
callback with `SchedulerRegister`. The runtime retains the callable and reserves a
slot; the managed list holds only registration wrappers/handles. Completion calls
`SchedulerActivate`, transferring ownership into runnable dispatch exactly once.
`SchedulerCancel` releases a dormant callback and returns true once; after activation,
repeat cancellation or an unknown/foreign/expired handle returns false. It does not
retract already-runnable work. Completed explicit registration uses the same
register/activate path. A registration handle is private to its invocation.

Dormant registrations retain roots but do not keep an invocation pending. Teardown
abandons remaining dormant roots. The destination token itself remains non-owning;
the runtime registration is what retains the callback. This may retain abandoned
managed operations until explicit cancellation or teardown, so automatic abandonment
cleanup remains a production migration gate. Task still represents operation state;
Promise is its TaskCompletionSource-like producer. Registration transitions do not
complete/cancel Tasks; the lifecycle consumer checks this explicitly.

Interpreter and native scopes keep one lazy identity across dispatch turns. A foreign
or expired identity faults before admission; reopening a native scope at the same
address gets a different identity. Identities are private to the loaded runtime and
must not be persisted or transferred across processes/backends. They are not security
capabilities. The callback must still belong to the current invocation's heap.

This is captured **invocation** placement, not custom scheduler/context selection.
Cross-invocation and cross-thread completion remain unsupported and are rejected
rather than silently rerouted. A real library migration still needs completion
ownership, registration lifetime, race rules and deliberate cross-invocation handling.
It cannot simply replace every queue.Post with SchedulerSubmitTo.

Admission is bounded, per registration and may fault. It reserves capacity so later
activation needs no additional slot. Registering multiple waiters is not atomic;
rollback, overload/backpressure and cross-thread completion remain open. No new fairness, FIFO or stable ABI guarantee
is inferred. Runtime saved activations will require a different execution adapter;
callback/state-machine layout must not define their public contract.

The [.NET comparison](../../runtime-scheduling-design.md) separates outcome, scheduler
and await-site placement. This experiment tests independence from producer queues;
it does not implement .NET SynchronizationContext/TaskScheduler capture, ExecutionContext
flow or ConfigureAwait. The benefit under test is producer-independent progress; costs
include retained registration objects, runtime admissions and defined failure handling.
No speedup is claimed. Keep the [existing state-machine baseline](../../../benchmarks/scheduler/README.md)
unchanged until a production routing change can be measured with equivalent workloads.

## Reproduce and evidence

```sh
python3 docs/experiments/jit-async/verify.py \
  --bundle target/data-cases-bundle \
  --console-qualification target/jit-startup-validation/validation.json \
  --continuations --output target/scheduler-continuations-fresh
```

Use matching rebuilt runtime/backend binaries and the Xcode SDK environment described
in the scheduler-projection experiment. [Original invocation-only evidence](validation.json) predates destination capture.
[Captured-destination evidence](destinations-validation.json) predates runtime-owned
registrations. [Registration lifecycle evidence](registrations-validation.json) predates scoped
observation cleanup. [Scoped-disposal evidence](disposal-validation.json) records compiler,
bundle, shared source, runtime/backend and output hashes plus exact commands. Success
and terminal-fault cases pass in interpreter, JIT and AOT on macOS ARM64 (six executions).
Windows was not run. Runtime Contract configuration, compiler emission and class-library
binaries are unchanged; no new reference assembly/API snapshot is required.

[Native destination contracts](destination-contracts.json) additionally cover foreign,
expired/reused scopes, boundary/output rejection and rooted delivery/cleanup. Rust
scheduler tests cover equivalent destination identity and rejection-before-retention.
Exact reserved signature tests run in both interpreter and native binding layers.

[Native registration contracts](registration-contracts.json) check full-capacity
activation, cancellation and root release, dormant quiescence and scope teardown.
The real awaitable consumer additionally checks registration cancellation independently
of Task state and exits with an intentionally abandoned dormant registration.


The [conditional-root collector experiment](../../runtime-scheduling-design.md#conditional-continuation-roots--collector-experiment-2026-10-11)
provides graph-level retention/reclamation tests for a possible abandonment policy.
It is not connected to these registrations: the evidence above still uses strong
runtime roots, explicit observer disposal and invocation teardown.


## Managed pending ownership alternative — 2026-10-11

`Completion.rvn` now contains the common awaitable. Select exactly one implementation:
`RuntimeContinuation.rvn` preserves the existing runtime-owned dormant registrations;
`ManagedContinuation.rvn` retains pending callbacks in ordinary managed storage and
captures only the non-owning invocation destination. Submission transfers a callable to
runtime-owned runnable work and releases the managed reference. Disposal releases an
unsubmitted callback without changing the operation, and never retracts admitted work.
This is a source-included comparison, not a change to public Task/Promise.

Run the alternative with `--managed-continuations` in place of `--continuations` in the
command above. [Managed ownership evidence](managed-ownership-validation.json) includes
the same six lifecycle/fault executions in interpreter/JIT/AOT, plus one interpreter-only
explicit-GC consumer. [Runtime ownership regression evidence](runtime-ownership-validation.json)
confirms the original implementation still passes its six executions after source splitting.
The verifier records both implementation and common-source hashes; profiles are mutually
exclusive. These runs reuse the recorded development binaries; no collector changes are
needed for the managed alternative.

`ManagedOwnership.rvn` creates and abandons two batches of 128 pending generated awaits.
After explicit collection the second batch leaves exactly the first batch's live object
count. Separate cases prove that a reachable operation retains an unobserved state machine,
that admitted runtime work roots an otherwise unobserved producer/awaiter graph, and that
a pending one-shot Timer does the same. Explicit collection occurs before completion and
also after continuation admission. Discarding a returned Task does not cancel live work.

The explicit-GC consumer is **interpreter-only**: this backend rejects
`neoCLR.Runtime.GCHeapObjectCount` as an unsupported value member contract. It is not
silently replaced with a constant or described as native reclamation evidence. Native
GC service binding/diagnostic qualification remains a next gate, alongside socket/DNS
completion-source coverage. Existing native lifecycle runs establish execution behavior,
not the bounded retained-object assertion from this consumer. Windows was not run.

The initial managed ownership evidence above predates batch admission. That version
avoids dormant runtime slots and conditional collector edges, but loses registration-time
capacity reservation. In that version, completion can fault while admitting
a large fan-out, after publishing its outcome and possibly admitting earlier waiters.
The fixed 64-slot submission budget therefore needs a deliberate bulk-admission/draining
policy before public migration. Cross-thread completion, races and cross-invocation
routing remain open. Per-registration ArrayList storage is intentionally provisional;
no allocation or performance advantage is claimed. See the
[design comparison](../../runtime-scheduling-design.md#managed-pending-callbacks--ownership-comparison-2026-10-11).


## Completion batch admission — 2026-10-11

The managed variant now submits one rooted `ContinuationBatch.Run` callable for all
pending waiters. It first validates their captured destinations, then makes one runtime
admission, and only on success marks registrations submitted and detaches the operation's
pending list. Completion/cancellation publishes its outcome after that admission. Dispatch
is deferred on the same owner thread, so callbacks see the published outcome. Cancelling
or disposing an admitted observer cannot retract it, even before the batch begins.
Already-withdrawn observers are skipped; an entirely inactive list needs no queue slot.
The runtime-owned dormant-registration variant still activates its pre-reserved slots.

This closes partial fan-out admission caused by the 64-slot limit: 128 generated awaiters
and one explicit observer complete with 63 existing submissions occupying the queue.
The test awaits a separate signal for existing runnable work rather than assuming its
ordering relative to the batch. Another case delivers 128 cancellation observers after
outcome publication. A fault in a batch remains terminal: later callbacks do not execute,
and the existing host teardown checks release rooted batch state.

[Batch evidence](batch-validation.json) contains twelve interpreter/JIT/AOT executions
(shared lifecycle/fault, large fan-out/cancellation, in-batch fault) and the interpreter
explicit-GC ownership case. [Reserved-registration regression](batch-runtime-validation.json)
contains six executions of the original variant through the common completion adapter.
Use the same `--managed-continuations` and `--continuations` commands above. Native
reclamation counters and Windows remain unqualified. Await inside `for` is rejected by
the current compiler cancellation protocol; the fan-out test uses an explicit index loop.

This is a transitional managed adapter for runtime-owned runnable work, not a new native
batch ABI. The existing runtime root owns the batch's managed graph. All callbacks run in
one dispatch turn; incremental draining, fairness and admission when *every* runtime slot
is occupied remain open. A full queue rejects the single admission before the operation
publishes its outcome; this is not an infallible completion contract. Allocation failure
or terminal callback faults are not transactional rollback. Cross-thread publication is
unsupported; publishing after admission relies on owner-thread deferred dispatch.
No public Task/Promise/Scheduler API, Runtime Contract or compiler emission changes.


## Incremental batch turns — 2026-10-11

The managed batch adapter now invokes one observer per dispatch and resubmits itself
when entries remain. Runtime arbitration runs between those steps. `Yield.rvn` has the
first observer submit a marker; the second must see that marker. This exposes a native
slot-reuse problem: rotating operation-table indices could select a newly resubmitted
batch before older runnable work. Native submissions/activations now receive admission
order identities, and polling considers only the oldest ready submission while retaining
source rotation. No new public FIFO or time-slice guarantee is introduced.

The interpreter runnable deque now grows through fallible reservation instead of sharing
an artificial 64-entry quota with dormant registrations. Every submission reserves enough
spare space for all dormant registrations; registration likewise reserves its future slot.
Activation therefore does not allocate. Tests retain 256 submissions plus 64 dormant
registrations through collection, activate all 64 without capacity changes, and drain all
320 callbacks. Allocation failure remains a fault before admission. The internal dormant
registration quota stays 64.

This is intentionally an incomplete cross-mode capacity migration: native runnable work
still occupies its 64-entry I/O table. Its independent backlog and memory-budget policy
are next. Incremental native resubmission can still fail if a callback fills every slot;
public Task migration remains blocked on that ownership/admission issue. The adapter still
uses fn<Void> callbacks, not a native batch descriptor or runtime suspension. One callback
per turn adds dispatch/admission work; throughput and larger-quantum tradeoffs remain to
be measured before selecting production policy.

[Incremental execution evidence](incremental-validation.json) records 15 interpreter/JIT/AOT
runs (including the marker test) and the interpreter-only explicit-GC case.
[Original registration regression](incremental-runtime-validation.json) adds six executions.
[Native contracts](incremental-native-contracts.json) cover six instrumented source/dispatch
contracts. Seventeen Rust scheduler tests and five submission/timer consumer tests pass.
Reproduce with the same verifier profiles above; run `cargo test --lib scheduler::tests`
and `cargo test --test scheduler_submission --test timers` for interpreter contracts.
Windows and native reclamation diagnostics remain unqualified.


The yield fixture assigns the awaited result before comparing it: the current native
emitter rejects the direct `if await task != value` expression with a value-block exit
diagnostic. This source workaround does not change scheduling semantics; the general
compiler expression gap remains separate from this runtime slice.
