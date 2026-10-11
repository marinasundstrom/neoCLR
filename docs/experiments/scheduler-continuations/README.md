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

## Ownership and remaining design work

Each `Continuation` captures `SchedulerCapture()` at registration and holds that
opaque destination plus its callback. The managed list retains these registrations
until completion; `SchedulerSubmitTo` verifies the destination before transferring
callback ownership into runtime admission. Completed explicit registration follows
the same capture/submit path. The token retains no runtime or managed root itself.

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

Admission is bounded, per callback and may fault. More than available capacity is not
an atomic fan-out operation; rollback, overload/backpressure, unlinking cancellation
and cross-thread completion remain open. No new fairness, FIFO or stable ABI guarantee
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
[Captured-destination evidence](destinations-validation.json) records compiler,
bundle, shared source, runtime/backend and output hashes plus exact commands. Success
and terminal-fault cases pass in interpreter, JIT and AOT on macOS ARM64 (six executions).
Windows was not run. Runtime Contract configuration, compiler emission and class-library
binaries are unchanged; no new reference assembly/API snapshot is required.

[Native destination contracts](destination-contracts.json) additionally cover foreign,
expired/reused scopes, boundary/output rejection and rooted delivery/cleanup. Rust
scheduler tests cover equivalent destination identity and rejection-before-retention.
Exact reserved signature tests run in both interpreter and native binding layers.
