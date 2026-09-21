# Provisional Task completion contracts

Development after Preview 8, 2026-09-19. This proof of concept is available in the
Raven library profile, not the published Preview 8 bundle. It provides the smallest
completion foundation for upcoming APIs. The September 21 development slice executes
compiler-generated async/await on neoCLR through an explicit TaskQueue. The public
namespace is now System.Tasks: completion is independent of threading. Rebuild
references and replace earlier System.Threading.Tasks imports.

## Consumer and producer

The Raven-authored types live provisionally in System.Tasks:

| Type | Current surface | Responsibility |
| --- | --- | --- |
| Task<T> | IsCompleted, GetAwaiter(), GetResult(), OnCompleted(callback) | Observe one eventual value; register consumers. |
| TaskCompletionSource<T> | constructor(queue), Task, TrySetResult(value) | Own completion authority; first completion wins. |
| TaskQueue | Run(callback), Post(callback), Drain() | Explicit single-invocation continuation dispatch for the PoC. |

Task is a stable reference: repeated reads of a source's Task property return the
same object. Completion publishes its value before enqueueing callbacks. Consumers
registered before completion are queued in registration order; late registrations
are also queued. TrySetResult returns false after the first completion and cannot
replace its value. Repeated GetResult calls return the stored value. Value payloads
follow ordinary value semantics; reference payloads retain their identity.

GetResult on a pending task faults immediately; it never blocks. Expected API
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
    let source = TaskCompletionSource<int>(queue)
    let task = source.Task

    task.OnCompleted(() => WriteLine(task.GetResult()))
    source.TrySetResult(42)
    WriteLine("Completion queued")
    queue.Drain()
}
```

Output:

```text
Completion queued
42
```

A real API can return the Task while retaining its TaskCompletionSource for later
completion. For this PoC, the caller owns and pumps the queue. There is no automatic host
progress. Run starts work in an active queue scope and then drains that queue. Drain processes queued batches and rejects
recursive pumping. Nested completion appends work instead of calling the next
consumer inside TrySetResult. A callback Fault terminates execution; there is no
recovery or cleanup guarantee after it.

## Comparison and provisional choices

This slice reuses the [completion experiment's primary-source comparison](experiments/task-contract/README.md#comparison-and-tradeoffs)
and [async design assessment](async-api-design.md). .NET's
[TaskCompletionSource<T>](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.taskcompletionsource-1?view=net-10.0)
also separates producer control from the Task given to consumers. Its exception,
cancellation and thread-safe completion states are broader than this PoC.

Keeping producer/consumer separation gives upcoming APIs a common return contract.
Using ordinary Result payloads fits neoCLR's exception-free execution model, at the
cost of incompatibility with exception-based .NET Task APIs. A single explicit
queue makes ordering testable without a thread pool or context capture, but callers
must drive progress. This is not a proposed permanent scheduling API or a performance
claim. No ConfigureAwait policy is introduced.

TaskQueue, callback registration and nonblocking result access are provisional
supporting mechanisms. The builder and awaiter protocol is compiler-facing infrastructure added in the
September 21 slice. Runtime-owned suspension may replace later compiler machinery. Cancellation,
timeouts, combinators, cross-thread races, host completion after an invocation,
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

## Generated async and explicit progress

See [library-async.rvn](experiments/raven-target/samples/library-async.rvn) for the
complete runnable source. It starts PrintAnswer inside queue.Run, awaits a pending
Task<int>, prints “Suspended”, completes the producer and then resumes to print 42.
Ordinary async functions return Task<T>; use Task<unit> for no payload. Even an
awaitless async function uses the builder and requires an active queue scope.
Starting async work outside Run or a drained callback faults. A method can remain
pending after Run returns and resume when later completion is followed by Drain.

Task completion queues registered continuations. The producer's queue controls
where its callbacks are drained; this is not UI affinity or automatic context flow.
Nested Run scopes on different queues select the nearest active queue, then restore
the outer scope naturally. Reentering Run or Drain on the same active queue faults
before running another callback. Cross-queue programs must drive every relevant
queue explicitly. A Task has no exception or cancellation state.

The provisional System.Runtime.CompilerServices protocol consists of
IAsyncStateMachine, ITaskAwaiter and AsyncTaskMethodBuilder<T>. The builder receives
reference state and awaiters by value, stores a TCS and registers MoveNext as a
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
