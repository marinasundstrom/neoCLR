# Provisional Task completion contracts

Development after Preview 8, 2026-09-19. This proof of concept is available in the
Raven library profile, not the published Preview 8 bundle. It provides the smallest
completion foundation for upcoming APIs. The September 21 development slice executes
compiler-generated async/await on neoCLR through an explicit TaskQueue. The public
namespace is now System.Tasks: completion is independent of threading. Rebuild
references and replace earlier System.Threading.Tasks imports.

The [Task model alignment assessment](task-model-alignment.md) records the next
contract. The core State/Outcome and producer cancellation slice is now implemented;
Cancelled awaits now propagate through named async functions; cancellation tokens
remain follow-up work. See the September 23 lowering slice below. Rebuild reference/library artifacts and callers together.

## Consumer and producer

The Raven-authored types live provisionally in System.Tasks:

| Type | Current surface | Responsibility |
| --- | --- | --- |
| Task<T> | State, Outcome, IsCompleted, IsCancelled, GetAwaiter(), GetResult(), OnCompleted(callback) | Observe completion or cancellation; register consumers. |
| Promise<T> | constructor(), constructor(queue), Task, Complete(value), Cancel() | Own completion authority; first completion wins. |
| TaskState | Pending, Completed, Cancelled | Normal enum for the public lifecycle. |
| TaskOutcome<T> | Completed(T), Cancelled | Union containing the terminal value or cancellation. |
| TaskQueue | Default, Run(callback), Post(callback), Drain() | Explicit single-invocation continuation dispatch for the PoC. |

Task is a stable reference: repeated reads of a source's Task property return the
same object. Completion publishes its value before enqueueing callbacks. Consumers
registered before completion are queued in registration order; late registrations
are also queued. Complete returns false after the first completion and cannot
replace its value. Cancel likewise returns false after either terminal transition.
IsCompleted is the provisional awaiter readiness flag: it is true for both terminal
states. State distinguishes Completed from Cancelled. Repeated GetResult calls return the stored value. Value payloads
follow ordinary value semantics; reference payloads retain their identity.

Outcome is None while pending, Some(Completed(value)) after completion, or
Some(Cancelled) after cancellation. No default T is created for a cancelled task.
Use ordinary nested patterns to inspect those cases. Cancel publishes its state
before enqueueing callbacks; late callbacks are queued too.

GetResult on a pending or cancelled task faults immediately; it never blocks.
Cancellation now propagates through await in named async functions: the enclosing
Task becomes Cancelled and the remaining body is skipped. Outcome and OnCompleted
also support explicit observation; see the September 23 lowering evidence below.
Calling Cancel on a producer completes that Task; this is distinct from a
future token source requesting cancellation of an operation. Expected API
failure is represented by an ordinary payload such as Result<T,E>. There is no
SetException, faulted-task state or special Result-aware completion path. Runtime
Faults remain terminal. Task<unit> uses the same implementation and storage as other
payload types. Ordinary Result patterns and propagation operate on the returned
value independently of Task.

The Task constructor and source's implementation methods are internal. Fields are
private. Both Raven compilation and direct guest-IL access checks enforce that
consumers cannot write completion state through their Task handle.

## Executable example

The exact sample is [library-tasks.rvn](experiments/raven-target/samples/library-tasks.rvn):

```raven
import System.*
import System.Tasks.*
import System.Console.*

func Main() {
    let queue = TaskQueue()
    let source = Promise<int>(queue)
    let task = source.Task

    task.OnCompleted(() => WriteLine(task.GetResult()))
    source.Complete(42)
    WriteLine("Completion queued")
    queue.Drain()
}
```

Output:

```text
Completion queued
42
```

A real API can return the Task while retaining its Promise for later
completion. This example chooses an explicit queue and drains it manually. Ordinary
Promise construction and async calls use the active queue or TaskQueue.Default;
the runtime drains default-queue work before returning from the invocation.
Run starts work in an active queue scope and then drains that queue. Drain processes queued batches and rejects
recursive pumping. Nested completion appends work instead of calling the next
consumer inside Complete. A callback Fault terminates execution; there is no
recovery or cleanup guarantee after it.

## Comparison and provisional choices

This slice reuses the [completion experiment's primary-source comparison](experiments/task-contract/README.md#comparison-and-tradeoffs)
and [async design assessment](async-api-design.md). .NET's
[TaskCompletionSource<T>](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.taskcompletionsource-1?view=net-10.0)
also separates producer control from the Task given to consumers. Its exception and thread-safe completion contracts are broader than this PoC.
This slice uses an explicit cancellation outcome without an exception channel.

Keeping producer/consumer separation gives upcoming APIs a common return contract.
Using ordinary Result payloads fits neoCLR's exception-free execution model, at the
cost of incompatibility with exception-based .NET Task APIs. A single explicit
queue makes ordering testable without a thread pool or context capture, but callers
must drive progress. This is not a proposed permanent scheduling API or a performance
claim. No ConfigureAwait policy is introduced.

TaskQueue, callback registration and nonblocking result access are provisional
supporting mechanisms. The builder and awaiter protocol is compiler-facing infrastructure added in the
September 21 slice. Runtime-owned suspension may replace later compiler machinery.
Cancellation requests and await propagation,
timeouts, concurrency combinators, cross-thread races, host completion after an invocation,
UI affinity, logical context and fairness remain open. An endlessly replenished
queue can starve its caller. APIs requiring those guarantees must wait for the
corresponding contracts.

## Implementation and validation

[Tasks.rvn](../runtime/raven/src/System/Tasks/Tasks.rvn) is the source of
truth. The producer and consumer compile together so internal access stays inside
the library. ArrayList<T> supplies an occupied result slot; the queue and registrations
also use ArrayList. This intentionally favors existing verified storage over a new
runtime primitive, with additional allocations and no performance claim. Task/source
cycles and queued closures use existing tracing GC. The September 21 slice adds
one bootstrap-only native service to obtain the nearest active TaskQueue.Run/Drain
receiver. It scans live library frames; the existing frame roots retain the queue.
It does not create a thread, scheduler or process-wide current context.

The bounded bridge validates Task signatures and preserves internal instance-member
visibility. Its Cecil audit normalizes nominal System.Void storage on both sides of
member resolution; no-result returns remain distinct. Callbacks currently retain
Func<System.Void> spelling: using Func<unit> in this source produced an extra discard
that the importer rejected. The IsCompleted getter uses a block because the pinned
SDK emitted ldnull for its expression-bodied call to the later generic producer.
These are recorded compiler/importer gaps, not preferred Raven style.

The library bootstrap uses Raven SDK 0.1.12-neoclr.15; application async emission
requires the current neoclr-branch compiler and matching bridge. Default .NET
builders remain independently tested. The source verifier covers immediate and
pending awaits, two suspensions with GC, generic unit results, Result propagation,
nested async calls, awaitless completion and queue scoping. Completion API and
access-boundary checks remain separate. This is a single-invocation PoC, not a
thread-safety or host-I/O guarantee.

Run the Raven checks with matching development artifacts:

```sh
python3 docs/experiments/task-contract/verify_tasks.py /path/to/Demo.rvnproj \
  --bridge /path/to/Probe.dll --system /path/to/System.neoil \
  --runtime /path/to/neoclr
cargo test --locked --test tasks
```

The saved project must use the rebuilt reference library. Regenerate bootstrap
snapshots through build_runtime_library.py when editing implementation sources;
never hand-edit generated neoIL.

## Generated async and queue selection

See [library-async.rvn](experiments/raven-target/samples/library-async.rvn) for the
complete runnable source. It starts PrintAnswer inside queue.Run, awaits a pending
Task<int>, prints “Suspended”, completes the producer and then resumes to print 42.
Ordinary async functions return Task<T>; use Task<unit> for no payload. Even an
awaitless async function uses the builder. It selects the nearest active queue or
the invocation default. The runtime dispatches default-queue work automatically
after the entry function returns; see the default-dispatch slice below. Explicit
queues still allow controlled Run/Drain for tests and custom orchestration.

Task completion queues registered continuations. The producer's queue controls
where its callbacks are drained; this is not UI affinity or automatic context flow.
Nested Run scopes on different queues select the nearest active queue, then restore
the outer scope naturally. Reentering Run or Drain on the same active queue faults
before running another callback. Cross-queue programs must drive their explicit
queues; the runtime drives the default queue. Tasks have cancellation state but no exception state. The opt-in
awaiter protocol propagates cancellation to an enclosing async method.

The provisional System.Runtime.CompilerServices protocol consists of
IAsyncStateMachine, ITaskAwaiter and AsyncTaskMethodBuilder<T>. The builder receives
reference state and awaiters by value, stores a Promise and registers MoveNext as a
continuation. Compared with .NET's generic by-reference builder protocol, this is
smaller for neoCLR's current object model but allocates heap state, including for
awaitless calls. These compiler-facing signatures may be replaced by runtime
suspension later. Task<Result<T,E>> has no special lowering: propagation returns
an ordinary Result payload through SetResult.

The bridge enables heap states and disables implicit exception capture for this
profile, preserves target builder metadata and admits same-module internal state
access. Clearing an awaiter materializes a typed default reference. Source exception
regions remain rejected. Hoisted aggregates without a default need further validation.
Generic async methods, async lambdas and broad async
iteration/disposal are outside the validated PoC. Two existing nested-lambda capture
problems observed during development are deferred compiler candidates; the runnable
sample uses a named async function and one ordinary callback.

```sh
python3 docs/experiments/task-contract/verify_async.py /path/to/Demo.rvnproj \
  --bridge /path/to/Probe.dll --system /path/to/System.neoil \
  --runtime /path/to/neoclr
```

The subsequent [isolated worker PoC](isolated-workers.md) runs on OS threads with
separate heaps. It returns Tasks owned by the caller and completes them through the explicit queue.
The queue may block while waiting for worker results; Task storage is not shared.

Shared neoCLR project props select RavenHeapAsyncStateMachines=true and
RavenCaptureAsyncExceptions=false in the matching development compiler. Build
the .rvnproj through MSBuild for the same policy used by the editor and bridge.
The published Preview 8 SDK does not implement these new project settings.

The intended public API will grow Promise-style composition and continuation
methods, using terminology appropriate to neoCLR rather than requiring .NET names.
Map and Then now provide these fundamental operators. OnCompleted remains
provisional protocol machinery. See the composition contract below and the
[alignment sequence](task-model-alignment.md).


## Core Task model migration — 2026-09-21

Rename TaskCompletionSource<T> to Promise<T> and producer calls from
TrySetResult(value) to Complete(value), and use Cancel()
for an operation's explicit cancelled completion. Both return bool and preserve the
first terminal outcome. TaskState is a normal enum; TaskOutcome<T> uses the same
bootstrap union metadata/storage protocol as Option and Result. The proposal's union
notation is conceptual; consumers use Raven case construction and nested patterns.
No Propagatable contract is attached to TaskOutcome: it does not overload `?`.

The pinned compiler mis-emits fully qualified nested case types in the new carrier's
constructor signatures as Object. Importing System.Tasks.TaskOutcome.* and using
unqualified constructor parameter types emits the correct signatures. This is a
bounded bootstrap workaround and a deferred general Raven compiler candidate; it
was reproduced with both the pinned bootstrap and installed development compiler.
The compiler repository was not modified in this slice.


For explicit cancellation, [library-task-outcomes.rvn](experiments/raven-target/samples/library-task-outcomes.rvn)
uses Promise<int>, queues an observer, cancels the producer and matches
Cancelled inside Some(outcome) when the queue drains. It prints `Cancelled`. Promise is producer
ownership, not an additional asynchronous result or a rejection/error channel.

The installed compiler's exhaustiveness analysis does not combine nested
Some(Completed(...)) and Some(Cancelled) match arms to cover Some. The sample first
extracts Some(outcome), then matches the outcome exhaustively. Nested `is` patterns
work directly. A nested match with no-result/Fault arms also produced incompatible
unit stacks in the importer; the sample instead computes its message with a
value-returning match. These are deferred Raven integration candidates, not new
restrictions on the Task model. Literal payload patterns can request unavailable
Object.Equals; extracted values use ordinary comparisons in the tests.

Validation for this core slice: 24 source contract scenarios, ten async regressions,
four worker scenarios, nine direct runtime checks and 263 signature checks pass.
The cancellation retention probe completed 38 garbage collections. Bootstrap
snapshot hashes, runtime API inventory/audit and website checks pass. This does not
validate cancelled await or multi-threaded Promise mutation.


## Task composition — 2026-09-21

Task operators use Raven extension declarations in System.Tasks. Import that
namespace to write `task.Map(transform)` and `task.Then(continuation)`. The
[composition sample](experiments/raven-target/samples/library-task-composition.rvn)
starts with a pending Promise<int>, maps 41 to 42, chains a Task<string>, and prints
42 when the queue drains. It needs no callback annotations.

| Operator | Completed input | Cancelled input |
| --- | --- | --- |
| Map<U>(Func<T,U>) -> Task<U> | Invoke the transform once; complete with its value. | Cancel the returned Task without invoking the transform. |
| Then<U>(Func<T,Task<U>>) -> Task<U> | Invoke the continuation once, then transfer the returned Task's eventual value or cancellation. | Cancel the returned Task without invoking the continuation. |

Both operators register queued callbacks, including when their input is already
terminal. Calling an operator does not execute user code inline. A new result stays
Pending until those callbacks run. Then also queues observation of an already
terminal inner Task. It flattens the asynchronous completion into Task<U>, not
Task<Task<U>>. Separate consumers can compose the same input independently.

The result Promise belongs to the input Task's queue. Map runs on that queue. Then
runs its first continuation there; observation of the inner Task runs on the inner
Task's queue. Completing the result enqueues its downstream observers on the
original queue. When these differ, the caller must drive both queues. This is
serialized single-invocation dispatch, not shared-state thread safety or automatic
context flow. No active TaskQueue.Run scope is required just to compose Tasks.

A Result.Error payload is passed to callbacks unchanged like any other T. Callback
Faults remain terminal execution failures; they do not create a Task fault/error
outcome. Cancellation travels downstream only: cancelling an input does not cancel
unrelated operations or introduce parent/sibling ownership. Tokens, structured
concurrency remain separate from these operators. Await propagation is implemented
in the following slice.

Implementation uses private generic continuation classes and ordinary managed
callbacks. Their references are retained by queued registrations and traced by GC.
The bounded importer admits parameterless, no-result callbacks on validated generic
library reference classes, preserving the constructed receiver while emitting one
adapter on the generic definition. This does not admit arbitrary generic application
methods, byref receivers or virtual generic callbacks. No Raven compiler or Runtime
Contract configuration change is part of this slice.

The installed compiler needs complete parameter/return annotations on block-bodied
callbacks whose generic result cannot be inferred. Supplying only the return type
also failed in this toolchain. The regression tests keep the required annotations;
ordinary expression callbacks and the user sample remain inferred. This is a
compiler limitation to revisit, not a requirement of Task composition.

Run the composition checks with the matching reference core, bridge and library:

```sh
python3 docs/experiments/task-contract/verify_composition.py /path/to/Demo.rvnproj \
  --bridge /path/to/Probe.dll --system /path/to/System.neoil \
  --runtime /path/to/neoclr
```

Compared with .NET ContinueWith's Task-valued callback, these operators receive the
completed value directly and propagate cancellation without calling user code.
This follows the selected Task model and reuses the
[recorded comparison](async-api-design.md#promise-style-composition-direction--2026-09-21).
It simplifies value pipelines but provides no scheduler-selection, fault-recovery or
multi-task orchestration API. Continuation objects and queueing add allocations and
dispatch work; no performance advantage is claimed.


## Cancelled awaits — 2026-09-23

Named async functions now propagate Task cancellation before consuming a value, on
both immediate and resumed awaits. The generated state machine clears its saved
awaiter, leaves the user body and calls the builder's SetCancelled. No default T,
Result.Error or exception is manufactured. Nested calls propagate the same outcome;
ordinary Result propagation still completes with a Result value.

This is selected explicitly with RavenPropagateAsyncCancellation=true, alongside
heap state machines and disabled exception capture in NeoCLR.Raven.props. The
provisional protocol adds bool IsCancelled to Task's awaiter surface and
SetCancelled() to AsyncTaskMethodBuilder<T>. The public State/Outcome model remains
the authoritative observation API. These hooks may change with runtime suspension.
The bridge admits stackless CLI leave instructions only in bodies without exception
handlers; source exception regions remain unsupported.

The compiler reports missing protocol members. Await inside for loops is rejected
with RAV2712: the probe exposed that their emitter-owned iterator state and disposal
are not yet suspension-aware. Raven use declarations also remain unsupported by
this target's Disposable contract (RAV1503); no cleanup support is claimed for them.
Cancellation exits participate in source-scope exit generation, but the supported
neoCLR subset does not yet include protected cleanup regions or async disposal.

Validation adds immediate/resumed cancellation for int, unit and Result<int,string>,
with nested async propagation and checks that later side effects never execute.
The existing successful awaits, Result propagation, GC retention and queue checks
remain regressions. Compiler tests cover explicit selection, project evaluation,
missing protocol diagnostics and the rejected for-loop shape. Ordinary .NET async
continues to use its existing exception-based protocol by default. This implements
the model's exception-free semantics at the cost of a target compiler contract;
it is not a performance claim. Reuse the primary-source comparison in
[the alignment assessment](task-model-alignment.md#comparison-and-acceptance-evidence).


## Default dispatch — 2026-09-23

Ordinary API implementations can construct Promise<T>() without a queue parameter.
It selects the nearest active TaskQueue.Run/Drain scope, falling back to the stable
TaskQueue.Default for this invocation. Async builders and Thread.Start/ThreadPool.Queue
use the same selection. Explicit Promise<T>(queue) remains available.

The runtime runs default-queue work automatically when the entry function returns.
Callbacks posted by callbacks are processed until the queue is empty, before the
invocation result is returned to the host. The original result is retained across
this dispatch. The default queue and any saved invocation result are GC roots while
work runs. Each isolated worker owns a separate default queue. Registration is
one-time, invocation-local state, not a mutable process-wide singleton.

[The runnable example](experiments/raven-target/samples/library-async-default-queue.rvn)
starts a worker from an async function and prints its result without constructing a
queue, passing one into a producer, or calling Drain. Tests cover parameterless
Promise construction, nested callbacks, cancellation and GC retention as well.

This is a minimal runtime dispatch loop, not a public TaskScheduler contract.
It is serialized; joining isolated workers can block the caller's dispatcher.
Automatic dispatch shares the invocation instruction budget and terminal Fault
behavior. An endlessly reposting callback exhausts that budget. An unresolved
Promise with no queued work does not keep an invocation alive: there is no external
I/O completion reactor yet. Explicit custom queues are still caller-driven, and
async entry-point signatures are not added by this slice. Source exceptions,
protected cleanup and shared guest mutation remain unsupported.

The author wants familiar .NET concepts adapted to neoCLR's framework style and
future runtime suspension. Reviewed 2026-09-23: .NET
[TaskScheduler.Current](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.taskscheduler.current?view=net-10.0)
falls back to its default scheduler outside a task, and
[TaskScheduler.Default](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.taskscheduler.default?view=net-10.0)
exposes the platform default. We reuse the ergonomic default-selection principle;
neoCLR's invocation-owned queue and serialized interpreter dispatch are provisional
implementation choices. They remove queue setup and pumping from normal callers,
at the cost of invocation-scoped lifetime and no independent host-event progress.
A scheduler abstraction may later fit runtime suspension, affinity or concurrency
policies; neither Task payloads nor Promise construction should freeze those choices.


## Explicit Result mapping — 2026-09-23

`Task<Result<T, E>>.MapResult(transform)` maps only the Ok payload, producing
`Task<Result<U, E>>`. Error retains the same error value and never invokes the
mapper. A cancelled input cancels the output without invoking it. Like Map, this
operator uses the input's dispatcher, queues observation even for an already
terminal input, and treats callback faults as terminal runtime faults.

This Raven extension in System.Tasks is an explicit convenience layer; ordinary
Task.Map continues to receive the entire Result. Compared with .NET Task and
exception-based completion, the benefit is visible separation of expected errors
from asynchronous completion. The cost is a distinct combinator and a Result
contract understood by API authors. It does not add a Task failure state.
See the [complete sample](experiments/raven-target/samples/library-task-result.rvn).
The provisional surface is intended to support API construction while scheduling,
cancellation tokens and runtime suspension continue to evolve.

A compiler limitation discovered here remains a general-fix candidate: invoking
generic Task.Map from an open extension over Task<Result<T,E>> encountered a cyclic
type-substitution emission error. The implementation uses an explicit continuation,
which also makes all outcome paths visible. Reproduce and validate the generic
projection issue independently on Raven's ordinary CLI target before integration
into Raven main; no neoCLR policy change is required by this helper.


## Awaited propagation and precedence — 2026-09-23

`let value = (await input)?` awaits a Task<Result<T,E>> and then propagates the
Result. Equivalent separate statements are `let result = await input` followed by
`let value = result?`. Error completes the outer Task with Error; a cancelled input
cancels it. Neither form introduces a Task failure state. Postfix propagation
currently binds before prefix await: `await input?` attempts propagation on the
Task and is rejected. An ergonomic shorthand is a future proposal, not shipped.

General Raven lowering now initializes the propagation operand temporary directly
when no catch conversion is required, avoiding an uninitialized union field in a
heap state machine. No new Runtime Contract option is needed. Protected exception
conversion remains unchanged; neoCLR's existing no-exception policy still applies.


Validation: six combined-expression cases cover immediate/resumed Ok, Error and
cancellation, alongside the two existing before/after-await propagation cases.
The general fix is independently tested on ordinary Raven CLI metadata contracts
(21 focused tests) and integrated into Raven main as c51c69bad; neoclr carries the
same lowering fix as e56fc1ddf (40 focused tests). The target-specific branch remains
separate. No parser or Task failure-state change was integrated into main.

## Scheduling and suspension exploration — 2026-09-23

The author clarified that TaskQueue was temporary scaffolding for building Task
interfaces/contracts. Keep using it while useful and update the model when concrete
needs arise, including runtime suspension. This is neither a commitment to preserve
TaskQueue nor a decision to replace it now. No public Scheduler API is selected.

Distinguish three responsibilities during that work: Task represents eventual
completion; scheduling determines where/when runnable work executes; continuation
dispatch determines where suspended work resumes. These are conceptual boundaries,
not three required public types. Task.Run should express concurrent work through
platform capabilities, whereas Thread remains an explicit native-thread API.
Supporting workers on another platform also needs capture/transport contracts;
choosing a scheduler alone does not make arbitrary closures transferable.

### Current behavior to reconsider

Promise captures a queue at construction. Its callbacks are posted to that queue.
AsyncTaskMethodBuilder creates its result Promise on the calling queue, but
AwaitOnCompleted registers MoveNext directly with the awaited task. Consequently,
an incomplete await resumes through the **awaited producer's queue**, even when
the async method's result belongs to a different calling queue. An already-completed
await may continue inline; this experiment specifically covers a pending worker.

[The affinity fixture](experiments/worker-task-cancellation/Affinity.rvn) starts a
worker on the default queue, then awaits it from an explicit queue. After entry
returns, the async body resumes without draining that explicit queue. Observation
of the async method's result requires an explicit drain of the calling queue. The
fixture records existing behavior so a future change can intentionally update the
expectation; it does not endorse this split as permanent affinity semantics.
Native notifications in this adapter are still driven only by the default queue.
Explicit-queue worker submission remains unsupported by the experiment's contract;
there is no newly implemented rejection or general custom-queue progress guarantee.

### Comparison and options

Reviewed 2026-09-23: Microsoft's
[ConfigureAwait FAQ](https://devblogs.microsoft.com/dotnet/configureawait-faq/)
explains that .NET Task awaits normally consider SynchronizationContext and then
a non-default TaskScheduler when arranging a pending continuation. neoCLR currently
has no equivalent await-site capture. Its simpler producer-queue behavior helped
bootstrap the APIs, but does not provide caller-affinity guarantees. This is a
missing design decision, not an improvement over .NET.

Possible next implementations include capturing a continuation destination at await,
using one runtime-owned execution context initially, or moving resumption ownership
into the runtime suspension model. Retaining the present behavior costs least now;
adding capture makes affinity explicit but needs lifetime/progress rules; runtime
ownership can centralize suspension/rooting but requires more machinery. No option
is selected merely by this comparison. A full public scheduler design needs broader
platform research before adoption.

Before a consumer requires these guarantees, use executable cases to settle:

- What owns a suspended operation, its retained roots and its continuation destination?
- How does native completion make it runnable without executing guest code on the
  producer thread or resuming it twice?
- What happens when completion races with cancellation or invocation shutdown?
- What keeps pending operations alive when no guest callbacks are ready, and how
  do limits, fairness and unsupported platform capabilities affect progress?

These questions guide the next need-driven slice. They do not require exposing
TaskQueue, Scheduler or a thread to every Storage caller.


### Runtime scheduling follow-up — 2026-09-24

The author revisited runtime async during socket integration. The
[runtime scheduling design](runtime-scheduling-design.md) recommends a private
invocation scheduler before extending the public socket bridge. TaskQueue remains
an adapter for existing behavior; neither a public Scheduler class nor permanent
producer-queue affinity is selected. Operation completion must make work runnable
independently of whether it is represented by today's generated callback or a future
runtime-owned frame. The affinity behavior recorded above remains implemented until
an explicit migration slice changes and validates it.

The first private native-host driver is now implemented: source polling uses one
rotating policy at idle and callback boundaries, worker completion publishes durable
wake hints, and pending roots are traced through the driver. TaskQueue affinity and
public Task behavior remain unchanged. See the
[implementation scope](runtime-scheduling-design.md#initial-native-host-driver--implemented-2026-09-24)
for polling fallbacks and the remaining ownership/migration gates.
