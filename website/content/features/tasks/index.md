# Tasks and async execution

Task describes whether an operation produced a value. It does not imply a thread. Write async code around the operation you need, with expected errors as values and cancellation as a distinct outcome.

**Preview 9 implementation.** These APIs are included in the matching Preview 9 downloads. The API and implementation are provisional. Current execution uses compiler-generated state machines; runtime suspension remains a research direction.

[See a working example ↓](#await) · [Download the complete example](../../samples/library-async-default-queue.rvn)

<a id="await"></a>

## Starting and awaiting a worker

```raven
{{TASK_WORKER_SAMPLE}}
```

This prints `Hello on a worker`. `Thread.Start` explicitly creates an isolated OS worker and returns a `Task<string>` for its completion. `await` obtains that string. Use `Task<unit>` when completion carries no additional value.

The worker callback is a named function with owned text input and output. Guest objects are not shared. `ThreadPool.Queue` offers the same completion shape through a small reusable pool.

A Task is a completion handle, not a thread wrapper. Other operations can produce Tasks without creating threads; future I/O and stream operations will build on awaitable completion too.

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

For an operation returning `Task<Result<T, E>>`, await yields the Result and `?` propagates an expected error independently. Cancellation never becomes `Result.Error`. The example uses explicit producer cancellation. Development cancellation tokens provide a separate request mechanism, described below.

```raven
{{TASK_PROPAGATION_SAMPLE}}
```

With the development compiler, `await input?` awaits first, then applies `?` to the result: Error from a Result or None from an Option is propagated to the enclosing function. Parentheses are no longer required. The explicit `(await input)?` form remains valid, including with older toolchains.

Here, Error skips the rest of Read and completes its Task with that same Error. The complete example supplies `Error("Unavailable")` and prints `Unavailable`. Propagation also works before an await, returning without waiting for later inputs.

[Download the propagation example →](../../samples/library-task-propagation.rvn)

[Download the cancellation example →](../../samples/library-async-cancellation.rvn) · [Explore Option and Result →](../outcomes/)

## Cooperative cancellation requests

**Development after Preview 9.** `System.Concurrency.CancellationTokenSource` owns
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
isolated worker threads. Timers, linked sources and HTTP/socket token overloads are
not implemented yet. Source/token separation follows .NET, while cross-thread
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

This is separate from a producer calling `Promise.Cancel()`: host cancellation does not complete an individual Task as Cancelled. Requests are cooperative and cannot interrupt an active host call or undo output. Teardown requests worker cancellation and waits for its threads to stop. Guest operation tokens, native I/O acknowledgement and bounded shutdown remain open.

<a id="dispatch"></a>

## Default TaskQueue dispatch

`Promise<T>()`, async functions and worker APIs select the active queue or the invocation’s `TaskQueue.Default`. The runtime runs default-queue callbacks automatically before the invocation returns, including callbacks posted by other callbacks. Explicit queues remain available for controlled dispatch.

This dispatcher is small and serialized. Worker joins can block it. An unresolved Promise with no queued work does not keep the invocation alive; OS I/O completion is not implemented yet. Await inside for loops is diagnosed until iterator state and cleanup are suspension-aware. Protected cleanup and async disposal are outside the current subset.

<a id="try"></a>

## Use the matching Preview 9 toolchain
With the Preview 9 SDK, open a prepared `.rvnproj` in VS Code, replace `Main.rvn` with one of the complete downloads above, and run the neoCLR build/run task. Refresh the compiler, reference core and runtime library together. Use the runtime, SDK and VSIX from the same release.

neoCLR and the programs it runs do not require .NET. The Raven compiler, build tools and Raven Language Server run on .NET.

[Setup and version compatibility →](../../try/#development)

<a id="release-checkpoint"></a>

## Preview 9: async and Tasks

Preview 9 includes the current Task/Promise model, named async functions, await, composition, Result propagation, producer cancellation and automatic default-queue dispatch. Isolated workers are included with their current limits: ordinary worker joins can block the queue.

The release was checked with fresh extracted runtime, SDK and editor packages, eight Async Workbench samples, six Task contract probes and the Linux/macOS/Windows source matrix. Prebuilt package validation covers macOS arm64. HTTP, guest cancellation tokens and the experimental nonblocking worker adapter are not supported features of this preview.

<a id="pending-read"></a>

## Pending-read contract experiment

A [development sample](../../docs/pending-read.html) separates a cancellation request from terminal producer acknowledgement. Real Task/Promise, await and managed GC exercise both ordered outcomes and reject late events. The first case models producer callbacks; a follow-up uses real host-worker notification and defers cancellation until producer acknowledgement. Neither is an async Storage implementation. Experimental raw runtime services now support individual worker cancellation with acknowledged outcomes; an isolated Raven adapter now maps those outcomes to Promise cancellation and tests both dedicated and pooled jobs. A cross-queue fixture also records a current limitation: a pending await resumes through the producer’s queue, while observation of the async method’s result uses its caller’s queue. Public cancellation APIs and the future affinity contract remain open.

<a id="direction"></a>

## Comparison with .NET and planned work

Both platforms expose awaitable completion. neoCLR uses Promise for producer completion and Result values for expected errors; Task has no faulted outcome. This differs from .NET Task’s exception-based fault state and requires different library and compiler contracts.

Raven currently emits state machines. Runtime-owned suspension is the intended
longer-term direction. Task describes eventual completion; scheduling determines
when and where runnable work executes. TaskQueue currently stores and dispatches
callbacks, but it is transitional scaffolding rather than the full scheduling model.

Development now includes a private invocation scheduler for completion-source
selection, worker wakeups and retained roots. It uses the same rotating source order
while the queue is busy and while waiting for work. TaskQueue remains the adapter
that executes current generated callbacks. Cancellation and the test socket source
still require bounded polling; portable host integration is unfinished.

The design separates operation completion from how an execution resumes, allowing
runtime-owned suspended executions later. Those suspended frames are not implemented,
and no public Scheduler API has been selected. Full runtime suspension is not required
before the first socket application.

Application-facing Task/Promise outcomes, await behavior and I/O results are separate
from compiler and runtime integration machinery. Generated state machines remain the
implementation we use to build useful APIs now. Async builders and the current public
TaskQueue surface are transitional contracts; private scheduling records can change
without becoming APIs applications must use. Runtime suspension itself is deferred.

The private scheduler now retains each ready callback and its default-queue destination
until an active VM frame takes ownership. A failed handoff retains those roots. This
clarifies ownership without changing current continuation affinity.

Continuation affinity also needs refinement: a pending await currently resumes through
the awaited Task’s producer queue. That behavior is not a permanent affinity guarantee.
The proposed initial resumption target is the owning invocation; custom queues and
future UI or worker contexts need explicit behavior and migration checks.

Development: the importer now supports opt-in value-type state machines for non-generic methods using Task awaiters. Startup runs in place; pending awaits reuse one retained state. Ready, pending and cancelled cases are checked under garbage collection pressure, including unit and Result payloads. Ready completion saves one managed state object in the focused comparison; pending cases have allocation parity. Heap states remain the default while broader cases and costs are evaluated.

Generated state machines help us build the platform now. Runtime-owned async suspension is a future direction, and these compiler-facing builder APIs may later be deprecated or removed. See the [development builder contracts](../../docs/async-builders.html) for the supported scope.

Cancellation tokens, host I/O completion, stream operations, scheduling policies and broader cleanup support remain development work. This page demonstrates the current direction, not a final design.

A development [Delayed Copy experiment](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/delayed-copy/README.md) now connects worker completion to the interpreter and default TaskQueue. A Raven consumer retains its byte array through collection and resumes to copy the result. This uses an isolated worker-library adapter; the normal Thread APIs above still use queued joins. The experiment now checks ready completions between default-queue callbacks, so a callback that reposts itself no longer prevents delivery. Callbacks must still return; preemption, explicit queue affinity and per-operation cancellation remain open.

A separate development test adapter now connects real loopback TCP reads to the
interpreter’s managed-array roots and default TaskQueue callbacks. It checks
collection during a pending read, callback delivery while the queue remains busy,
and completion alongside a pending worker. The host-connection injection service exists only in test builds. The private receive
backend now retains reusable connections and accounts for pending and completed read
results separately. It supports cancellation before reading without closing the
connection.

The first public development [TCP client API](/docs/sockets.html) now connects to
numeric IPv4 addresses and sends/receives bytes through `Task<Result<...>>`. Pending
buffers and completion objects are traced until delivery. It uses nonblocking
sockets and the private scheduler, with the current generated async state machines.
Sends snapshot their source ranges and permit short writes. Listen and asynchronous Accept now support a two-process echo POC; this is not yet a complete networking API.

The [networking POC](../networking/) now resolves hostnames through Dns and
connects using the returned IPv4 addresses. Host lookup runs off the VM thread,
with bounded concurrency and scheduler-delivered completion.

**Development after Preview 9:** explicit thread APIs move to `System.Concurrency`; Task and Promise stay in `System.Tasks`. A retained `Thread(callback, input)` exposes a pending `Task` before instance `Start()`. Starting twice faults. `Thread.Run(callback, input)` is the immediate-start shortcut. Both retain the current isolated string callback restriction; successful completion includes native thread termination. These changes require matching development artifacts and are not in the Preview 9 downloads.

`Task.Run` is planned to spawn work concurrently, with overloads for completion-only and value-producing callbacks. Calling Run submits the work; awaiting its Task observes completion. Scheduling, captured state and async callback behavior need to be defined with the suspension model. The string-only worker API is not the intended general Task.Run contract.

System.Concurrency will cover concurrency, including threading. Thread remains an explicit API for threads and may be unavailable on some platforms. Task will be a general abstraction and API for work, with a way to submit work concurrently, such as Task.Run. Its execution mechanism depends on the platform rather than requiring a thread. On WebAssembly it might use Web Workers behind the Task API. This is post-release direction; exact scheduling, isolation and progress contracts still need design and implementation.

[Related proposals and open questions →](../../proposals/#async)

<a id="feedback"></a>

## Questions and contributions

Report completion, cancellation or scheduling issues with a reproducible example and the expected operation lifetime.

[Discuss on GitHub ↗](https://github.com/marinasundstrom/neoCLR/issues)

Questions, sample programs and documentation corrections are welcome. See [how to contribute](../../#feedback) for ways to participate.

## API reference

[System.Tasks](xref:System.Tasks) · [System.Concurrency](xref:System.Concurrency)

The generated reference describes development after Preview 9. Use the availability
notes above to distinguish it from the published toolchain.
