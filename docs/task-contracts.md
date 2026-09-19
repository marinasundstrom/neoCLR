# Provisional Task completion contracts

Development after Preview 8, 2026-09-19. This proof of concept is available in the
Raven library profile, not the published Preview 8 bundle. It provides the smallest
completion foundation for upcoming APIs. It does not yet execute compiler-generated
async/await on neoCLR.

## Consumer and producer

The Raven-authored types live provisionally in System.Threading.Tasks:

| Type | Current surface | Responsibility |
| --- | --- | --- |
| Task<T> | IsCompleted, GetResult(), OnCompleted(callback) | Observe one eventual value; register consumers. |
| TaskCompletionSource<T> | constructor(queue), Task, TrySetResult(value) | Own completion authority; first completion wins. |
| TaskQueue | Post(callback), Drain() | Explicit single-invocation continuation dispatch for the PoC. |

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
import System.Threading.Tasks.*
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
completion. For this PoC, the caller owns and pumps the queue. There is no ambient
scheduler or automatic host progress. Drain processes queued batches and rejects
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
supporting mechanisms. State-machine builders and awaiters are not added in this
slice. Runtime-owned suspension may replace later compiler machinery. Cancellation,
timeouts, combinators, cross-thread races, host completion after an invocation,
UI affinity, logical context and fairness remain open. An endlessly replenished
queue can starve its caller. APIs requiring those guarantees must wait for the
corresponding contracts.

## Implementation and validation

[Tasks.rvn](../runtime/raven/src/System/Threading/Tasks/Tasks.rvn) is the source of
truth. The producer and consumer compile together so internal access stays inside
the library. ArrayList<T> supplies an occupied result slot; the queue and registrations
also use ArrayList. This intentionally favors existing verified storage over a new
runtime primitive, with additional allocations and no performance claim. Task/source
cycles and queued closures use existing tracing GC. No native service was added.

The bounded bridge validates Task signatures and preserves internal instance-member
visibility. Its Cecil audit normalizes nominal System.Void storage on both sides of
member resolution; no-result returns remain distinct. Callbacks currently retain
Func<System.Void> spelling: using Func<unit> in this source produced an extra discard
that the importer rejected. The IsCompleted getter uses a block because the pinned
SDK emitted ldnull for its expression-bodied call to the later generic producer.
These are recorded compiler/importer gaps, not preferred Raven style.

Validation uses Raven SDK 0.1.12-neoclr.15, the current development bridge and rebuilt
library. Four Rust tests check identity, generic unit storage, invariance and direct
IL access restrictions. Fifteen Raven scenarios cover values, callbacks, duplicate
completion, Result/unit, GC retention, terminal faults and source-level access
rejection. The GC scenarios observed 38 collections after the API returned and 39 while
retaining a reference payload. The exact example above is
compiled and executed separately, including its saved .rvnproj build for VS Code.
All 257 signature checks and 22 selected Rust Task/collection/UTF-8 tests pass. This establishes the completion PoC, not general
async support or thread safety.

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
