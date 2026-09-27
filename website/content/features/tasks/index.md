# Tasks and async execution

Task describes whether an operation produced a value. It does not imply a thread. Write async code around the operation you need, with expected errors as values and cancellation as a distinct outcome.

**Preview 11 API.** Use matching runtime, SDK and library artifacts. See [setup](../../try/#development) for package availability.

[See a working example ↓](#await) · [Download the complete example](../../samples/library-async-default-queue.rvn)

<a id="await"></a>

## Starting and awaiting a worker

Direct async Main uses the matching compiler, bridge and runtime. See [supported entry points](../../raven/#entry-points)
for return types and process exit behavior.

```raven
{{TASK_WORKER_SAMPLE}}
```

This prints `Hello on a worker`. `Thread.Run` creates an isolated OS worker and returns a `Task<string>` for its completion. `await` obtains that string. Use `Task<unit>` when completion carries no additional value.

The worker callback is a named function with owned text input and output. Guest objects are not shared. `ThreadPool.Queue` offers the same completion shape through a small reusable pool.

This preview uses System.Concurrency.Thread: construct a Thread and call its instance Start(), or use Thread.Run for immediate submission. The retained Task observes completion, including native thread termination. DNS, sockets and HTTP also return Tasks without creating a thread per operation.

<a id="task-run"></a>

## Submitting work with shared captures (development)

`Task.Run` is the canonical API for new work with captured variables and shared
objects. The runtime chooses execution; the first backend uses bounded native
threads. Overloads return `Task<unit>` or `Task<T>`, and a task-producing callback
is unwrapped into one task. Run uses the default dispatcher. Captures and reference
results preserve identity; arbitrary shared updates are not automatically atomic.

A blocked host call allows other contexts to progress. Guest instruction intervals
still share a managed graph gate, so this does not promise parallel guest CPU
execution. Entry completion drains pending work. Expected failures remain Result
values; a callback Fault fails the invocation and requests sibling shutdown.

Importing `System.Tasks.*` supports `Task.Run` alongside `Task<T>` without an alias.
Inline block callbacks infer their value result; an explicitly typed delegate local
is optional. Direct completion-only await uses the ordinary Task protocol. Mutable locals in
ordinary async methods share storage with their callbacks,
including across suspension. Generic async methods on nongeneric owners now support shared captures and
constructed state-machine types. Development checks cover two suspensions,
value/text results, array/object identity and cancellation. Async methods inside
generic classes are supported for ordinary nongeneric instance methods. Generic
methods on generic owners and broader async-lambda shapes need further work.

[Download the compiled shared-capture example](../../samples/task-run.rvn) ·
[Overload and lifetime reference](../../docs/callbacks.html#task.run-development)

<a id="promise"></a>

## Promise and Task composition

```raven
{{TASK_PROMISE_SAMPLE}}
```

This prints `42`. `Promise<T>` owns completion; its `Task` lets consumers await or compose the result. `Complete(value)` and `Cancel()` return whether they won the first terminal transition. Later attempts leave the outcome unchanged.

`Map` transforms a completed value. `Then` accepts a continuation returning another Task and follows its completion. Both propagate cancellation without invoking skipped callbacks. They treat `Result` as an ordinary payload.

[Download the producer example →](../../samples/library-task-producer.rvn)

<a id="outcomes"></a>

## Completion, cancellation and Fault

`TaskOutcome<T>` is a standard Raven union with `Completed(T)` and `Cancelled`
cases. Use patterns to inspect it. `HasValue` reports an active case, `Value` boxes
that case through `IUnion`, and `TryGetValue` extracts a matching case. An inactive
default is distinct from cancellation. `Task.Outcome` uses `None` while pending.

| Situation | Meaning |
| --- | --- |
| `Pending` | The operation has no terminal outcome yet. |
| `Completed(value)` | Await yields the value. That value can itself be a Result containing Ok or Error. |
| `Cancelled` | Await cancels the enclosing Task and skips the rest of that async body. |
| Fault | Unrecoverable execution failure. There is no Task fault state or catch/rejection operator. |

```raven
{{TASK_CANCELLATION_SAMPLE}}
```

This prints only `Cancelled`. The message after the await never runs. `Outcome` is None while pending and Some of the terminal outcome afterward. Ordinary application code can await; orchestration code can inspect outcomes explicitly.

`MapResult` explicitly maps the Ok payload of a Task containing a Result. It preserves Error and propagates cancellation without invoking the mapper. Ordinary `Map` still receives the whole Result.

```raven
{{TASK_RESULT_SAMPLE}}
```

This prints `42`. [Download the Result mapping example →](../../samples/library-task-result.rvn)

For an operation returning `Task<Result<T, E>>`, await yields the Result and `?` propagates an expected error independently. Cancellation never becomes `Result.Error`. The example uses explicit producer cancellation. Cancellation tokens provide a separate request mechanism, described below.

```raven
{{TASK_PROPAGATION_SAMPLE}}
```

With the matching compiler, `await input?` awaits first, then applies `?` to the result: Error from a Result or None from an Option is propagated to the enclosing function. The explicit `(await input)?` form is also valid.

Here, Error skips the rest of Read and completes its Task with that same Error. The complete example supplies `Error("Unavailable")` and prints `Unavailable`. Propagation also works before an await, returning without waiting for later inputs.

[Download the propagation example →](../../samples/library-task-propagation.rvn)

[Download the cancellation example →](../../samples/library-async-cancellation.rvn) · [Explore Option and Result →](../outcomes/)

## Cooperative cancellation requests

`System.Concurrency.CancellationTokenSource` owns
cancellation authority. Its `Token` can be copied and passed to cooperating work.
`CancellationToken.None` never requests cancellation. `token.Register(callback)`
returns a disposable registration; disposing it removes a callback that has not started.

`source.Cancel()` sets `IsCancellationRequested` and invokes callbacks synchronously
in reverse registration order. Registering after cancellation invokes the callback
inline. Disposing a source releases registrations without requesting cancellation.

A request is not completion. The operation must stop using its resources and finish
cleanup before cancelling its Promise. A Task that already completed keeps its result.
This distinction also applies to future runtime suspension: changing how execution
resumes must not shorten the lifetime of buffers still owned by native I/O.

The first implementation is confined to one invocation; tokens are not shared with
isolated worker threads. Timers and linked sources are not implemented. DNS and socket overloads forward tokens and acknowledge
cancellation through Task. The HTTP client forwards tokens and closes its owned
connection before reporting cancellation. Source/token separation follows .NET, while cross-thread
synchronization and exception aggregation are outside this iteration.

[CancellationToken API →](xref:System.Concurrency.CancellationToken)

<a id="worker-limits"></a>

## Worker payload limits

Each worker has a default 1 MiB quota shared by its returned text and captured console output. Accounting uses UTF-8 bytes plus one byte per output line, including empty lines. Output is checked before copying a line; returned text is checked before publishing success. Exceeding the quota faults when the caller joins the result, without forwarding partial output.

The embedding host can change this quota. With the existing 64-submission limit, default successful payloads total at most 64 MiB. This is not a process-memory limit: inputs, temporary strings, diagnostics and allocation overhead are excluded. A worker printing an empty line and `é`, then returning `é`, uses six logical bytes.

The limit applies to dedicated and pooled workers, including the experimental completion adapter. Its per-worker accounting is predictable, but a future shared budget could distribute capacity more flexibly.

<a id="host-cancellation"></a>

## Host cancellation during completion

The embedding host can request cancellation of an entire invocation. Worker joins check that request around waits, between forwarded output lines and before returning the value. For example, if the host requests cancellation while writing the first of two buffered worker lines, that line remains visible, the second is skipped and the invocation ends with an `execution cancelled` Fault. A fresh invocation can still run the same program.

This is separate from a producer calling `Promise.Cancel()`: host cancellation does not complete an individual Task as Cancelled. Requests are cooperative and cannot interrupt an active host call or undo output. Teardown requests worker cancellation and waits for its threads to stop. Per-operation tokens on DNS, sockets and HTTP are separate from host invocation cancellation.

<a id="dispatch"></a>

## Default TaskQueue dispatch

`Promise<T>()`, async functions and worker APIs select the active queue or the invocation’s `TaskQueue.Default`. The runtime runs default-queue callbacks automatically before the invocation returns, including callbacks posted by other callbacks. Explicit queues remain available for controlled dispatch.

This dispatcher is small and serialized. Worker joins can block it. An unresolved Promise with no queued work does not keep the invocation alive; pending registered I/O uses the private invocation scheduler for completion delivery. Await inside for loops is diagnosed until iterator state and cleanup are suspension-aware. Protected cleanup and async disposal are outside the current subset.

<a id="try"></a>

## Use matching Preview 11 artifacts
With the matching Preview 11 SDK, open a prepared `.rvnproj` in VS Code, replace `Main.rvn` with one of the complete downloads above, and run the neoCLR build/run task. Refresh the compiler, reference core and runtime library together. Use the runtime, SDK and editor tools from a matching build.

neoCLR and the programs it runs do not require .NET. The Raven compiler, build tools and Raven Language Server run on .NET.

[Setup and version compatibility →](../../try/#development)

<a id="release-checkpoint"></a>
<a id="pending-read"></a>

## Completion ownership

Pending native operations retain buffers and completion objects until acknowledged delivery.
Cancelling a Task does not permit early reuse of resources still owned by its provider.
See [networking](../networking/) and [HTTP](../web/) for operation-specific cleanup.

<a id="direction"></a>

## Comparison with .NET and planned work

Both platforms expose awaitable completion. neoCLR uses Promise for producer completion and Result values for expected errors; Task has no faulted outcome. This differs from .NET Task’s exception-based fault state and requires different library and compiler contracts.

Raven emits state machines, with a private invocation scheduler delivering native
completion through TaskQueue. Callbacks must return for other work to progress.
A pending await currently resumes through the producer queue; custom queue and UI
affinity are not settled contracts. Ordinary worker joins can still block the queue.

Task.Run is implemented with the integration limits described above.
Thread may become a backend primitive; its public future remains open. Green
threads are a possible later runtime backend, subject to the same sharing and
progress guarantees.
Runtime-owned suspension and broader cleanup support remain future directions. Their scheduling, isolation and lifetime rules need design;
no public Scheduler API is selected. See the [builder contracts](../../docs/async-builders.html)
for the supported compiler integration.

[Related proposals and open questions →](../../proposals/#async)

<a id="feedback"></a>

## Questions and contributions

Report completion, cancellation or scheduling issues with a reproducible example and the expected operation lifetime.

[Discuss on GitHub ↗](https://github.com/marinasundstrom/neoCLR/issues)

Questions, sample programs and documentation corrections are welcome. See [how to contribute](../../#feedback) for ways to participate.

## API reference

[System.Tasks](xref:System.Tasks) · [System.Concurrency](xref:System.Concurrency)

The generated reference describes Preview 11. Use matching toolchain artifacts.
