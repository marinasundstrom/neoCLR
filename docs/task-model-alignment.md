# Aligning the Task implementation with the model

Assessment and implementation order, **2026-09-21**. The author directed that the
[Task model proposal](proposals/task-model.md) take priority over adapting Raven's
existing async lowering. This note records that direction and the remaining work;
it does not announce implementation of the proposal.

## Contract before lowering

The public model belongs to System.Tasks and is independent of Raven:

| Surface | Required meaning |
| --- | --- |
| TaskState | Normal enum: Pending, Completed, Cancelled. No scheduler states. |
| TaskOutcome<T> | Union: Completed(T) or Cancelled. |
| Task<T>.State | The externally observable lifecycle. |
| Task<T>.Outcome | None while pending; Some of the terminal outcome afterward. |
| Producer completion | Complete(value) or Cancel(), with only the first terminal transition succeeding. Exact producer API naming remains provisional. |
| CancellationToken | A request that an operation may observe; requesting cancellation alone does not complete a Task. |
| Map | Transform an ordinary completed T; propagate cancellation without invoking the transform. |
| Then | Chain a T-to-Task<U> continuation and flatten its completion; propagate cancellation at either stage. |

State and Outcome must describe the same completion, including during callbacks.
Publish one terminal outcome before dispatching any observers. Do not introduce an
independent optional Result property or manufacture a default T for cancellation.
Consumers cannot mutate completion state. Repeated observations preserve the value's
ordinary copy/reference semantics.

Result<T,E> is an ordinary payload. Completed(Error(error)) remains Completed;
Task operators must neither unwrap it nor skip their callbacks because it is Error.
Faults terminate execution outside this outcome model. There is no Faulted state,
exception storage or recovery operator. Use the existing Raven Error case spelling
in implementation samples; the proposal's conceptual Err spelling does not rename
Result.

## Assessment baseline and subsequent progress

The following gap analysis records the state before implementation. The first core
slice now adds TaskState, TaskOutcome<T>, State/Outcome and Complete/Cancel; see
[current contracts](task-contracts.md). Composition, tokens and cancelled-await
propagation remain outstanding.

## Gaps identified at assessment

The [validated completion PoC](task-contracts.md) supports pending/completed values,
queued callbacks and generated async, but does not implement the proposed public
State/Outcome or Map/Then surface. TaskQueue and the builder/awaiter protocol are
provisional mechanisms, not requirements imposed on future runtimes.

The cancellation work in progress has not been regenerated, validated end to end,
or installed. Adding IsCanceled, TrySetCanceled and a builder cancellation method
alone would not implement this model. State must be a real enum and Outcome a real
union with compiler-visible case metadata, rather than unrelated boolean flags or
a fabricated successful payload. The bootstrap bridge must admit those contracts
and their patterns in addition to the runtime implementation.

The draft worker path also checks the cancellation request after a string callback
returns and discards its value if the request is set. That is not sufficient
acknowledgement of cancellation: a callback may ignore a request and complete
normally. Replace that behavior with an explicit operation-level terminal decision.
Checking IsCancellationRequested is itself only observation; it must not silently
change a subsequently returned successful value into cancellation. A worker adapter
may decline to start work after observing a request, but once started the operation
must choose how to terminate. The exact worker acknowledgement API remains open.
Keep the runtime host's execution-abort/Fault mechanism distinct from guest Task
cancellation.

## Implementation slices

1. **Task state and outcomes.** Implement the enum, union, immutable consumer view
   and single-terminal-transition producer contract. Regenerate matching reference
   metadata and Raven library artifacts. Validate int, unit, Option, Result and
   reference payloads, duplicate transitions and private completion authority.
2. **Composition.** Implement Map and Then against the completion contract without
   async syntax. Use the current explicit queue as provisional dispatch: callbacks
   are queued even when registered after completion. Test pending/completed inputs,
   both cancellation paths in Then, non-invocation of skipped callbacks, ordinary
   Result.Error values, callback Faults and retention through GC. Cross-queue work
   must state which queues the caller must pump; do not imply automatic progress.
3. **Cancellation requests and workers.** Connect tokens to cooperative operation
   decisions while keeping request and terminal state separate. Validate ignored
   requests, cancellation before execution, explicit termination after a request,
   completion winning over a later request, and worker cleanup. Registration,
   linking and shared guest objects need not be added to establish the core model.
4. **Raven integration.** Lower await onto those tested semantics. Preserve ordinary
   Result propagation and .NET behavior; document unsupported constructs explicitly.
   Rebuild and install a matching development toolchain only after source scenarios
   and emitted artifacts pass. Update the website's implemented-status claims then.

Each slice needs its own changelog, tests and commit. This order establishes the
Task model without making the existing compiler protocol permanent. It does not
promise a production scheduler, thread-safe guest Task storage, structured
concurrency or runtime-owned suspension.

## Raven lowering boundary

Inspection of Raven **a7b728aa4** on its neoclr branch found that
`AsyncLowerer.LowerAwaitExpression` stores an awaiter, tests completion, registers
resumption when pending and calls GetResult on the common immediate/resume path.
Disabling exception capture removes the catch wrapper; it does not add cancellation
propagation. A cancelled task cannot yield a T, so an unconditional GetResult call
cannot implement the proposal without introducing an unwanted exception or Fault.

The neoCLR path needs a terminal-outcome decision before consuming T:

- Completed(value): continue the user body with that value.
- Cancelled: complete the enclosing Task as Cancelled and stop the user body.
- Pending: retain state and register resumption, without publishing an outcome.

Apply the same decision to immediate and resumed awaits. Release saved awaiter/state
references appropriately and audit cleanup/early-return paths; a cancellation branch
must not bypass required normal cleanup. Unsupported cleanup forms need diagnostics,
not silently omitted behavior. Cancellation and `?` stay independent: the former
terminates asynchronous computation; the latter is ordinary value propagation.

The recommendation is an explicit target-specific async lowering policy, sharing
state/liveness machinery where semantics match. A separate lowering implementation
may be clearer if completion and cleanup paths diverge substantially; duplicating
all of AsyncLowerer now would create two implementations to maintain before the
boundary is understood. Do not infer the policy from a class name, the absence of
SetException, or the payload being Result. Keep the target policy on Raven's neoclr
branch and independently extract any general metadata/lowering fixes for main.
No Raven code was changed by this assessment.

## Comparison and acceptance evidence

Reuse the primary-source comparisons in [async design](async-api-design.md#primary-comparisons-and-remaining-research)
and the [completion experiment](experiments/task-contract/README.md#comparison-and-tradeoffs).
.NET TaskCompletionSource supplies the useful producer/consumer separation, but its
exception/cancellation completion protocol cannot simply be copied into an
exception-free target. The explicit Outcome adds inspectable cancellation at the
cost of new metadata, library API and compiler integration. Promise-style chaining
provides composition vocabulary without importing rejection or JavaScript scheduling.
This is an API/semantic choice, not a measured performance improvement.

Acceptance must demonstrate that requesting cancellation does not itself finish an
operation; a cancelled await skips later side effects both before and after
suspension; and Result.Error completes normally through the same generic path as
other values. Inspect emitted artifacts for exception dependencies and test ordinary
.NET awaits separately. Library composition tests should establish the model before
compiler tests establish Raven's syntax over it. Fault cleanup guarantees and
cross-thread races remain outside the single-queue PoC's evidence.
