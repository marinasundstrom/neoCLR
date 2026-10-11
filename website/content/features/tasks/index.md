# Tasks and async execution

Task describes whether an operation produced a value. It does not imply a thread. Write async code around the operation you need, with expected errors as values and cancellation as a distinct outcome.

[See a working example ↓](#await) · [Download the complete example](../../samples/task-run-await.rvn)

<a id="await"></a>

<a id="native-compiler-integration-development"></a>

## Native async support

Nongeneric functions and class methods can await completed or pending Tasks. Locals and
receiver identity survive suspension. Native `Task<unit>` and `Task<int>` entry points
complete registered work before reading the result; cancellation and unresolved tasks
fail explicitly. Generic async methods and `Task<Result<...>>` entry points are not supported.

## Already completed work

`Task.CompletedTask` returns an already successful `Task<()>`. For a value, use
`Task.FromResult(42)`, which infers `Task<int>`. Neither schedules work; explicit
completion callbacks still use the task dispatcher. Use `Promise<T>` when completion
will happen later. These helpers are development APIs and require rebuilt libraries.

```raven
await Task.CompletedTask
let answer = await Task.FromResult(42)
```

## Starting and awaiting work

Use `Task.Run` to submit work and `await` to wait for its completion. Direct async
Main uses the matching compiler and runtime; see [supported entry points](../../raven/#entry-points)
for return types and process exit behavior.

```raven
{{TASK_RUN_SAMPLE}}
```

This prints `completed`. The callback returns no additional value, so `Task.Run`
returns `Task<()>`; `await` waits for that work to finish before Main returns.
A value-producing callback returns `Task<T>` instead. The runtime chooses how to
execute submitted work; application code does not need to create a Thread.

<a id="task-run"></a>

<a id="submitting-work-with-shared-captures-development"></a>

## Submitting work with shared captures

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
Inline block callbacks infer their value result; an explicitly typed function local
is optional. Direct completion-only await uses the ordinary Task protocol. Mutable locals in
ordinary async methods share storage with their callbacks,
including across suspension. Generic async methods on nongeneric owners support shared captures and
constructed state-machine types. Async methods inside
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
that case through `UnionValue` in development references (formerly `IUnion`), and `TryGetValue` extracts a matching case. An inactive
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

This prints only `Cancelled`. The message after the await never runs. `Outcome` is None while pending and Some of the terminal outcome afterward. Use `await` by default in application code, including async `Main`. This example
uses `OnCompleted` deliberately to observe and print cancellation: awaiting that
cancelled result would cancel Main before it could print. Orchestration code can
inspect outcomes explicitly.

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

## Explicit isolated workers

Use `Thread` when you specifically need the isolated worker contract. For ordinary
shared-capture work, start with `Task.Run` above.

```raven
{{TASK_WORKER_SAMPLE}}
```

This prints `Hello on a worker`. `Thread.Run` creates an isolated OS worker and returns a `Task<string>` for its completion. `await` obtains that string. Use `Task<unit>` when completion carries no additional value.

The worker callback is a named function with owned text input and output. Guest objects are not shared. `ThreadPool.Queue` offers the same completion shape through a small reusable pool.

This preview uses System.Concurrency.Thread: construct a Thread and call its instance Start(), or use Thread.Run for immediate submission. The retained Task observes completion, including native thread termination. DNS, sockets and HTTP also return Tasks without creating a thread per operation.

[Download the isolated-worker example](../../samples/library-async-default-queue.rvn)

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

## Run the examples

Open the bundled Tasks project in VS Code and use the neoCLR run task.
[Installation instructions →](../../try/)

<a id="release-checkpoint"></a>
<a id="pending-read"></a>

## Completion ownership

Pending native operations retain buffers and completion objects until acknowledged delivery.
Cancelling a Task does not permit early reuse of resources still owned by its provider.
See [networking](../networking/) and [HTTP](../web/) for operation-specific cleanup.

<a id="direction"></a>

## Comparison with .NET and planned work

Both platforms expose awaitable completion. neoCLR uses Promise for producer completion and Result values for expected errors; Task has no faulted outcome. This differs from .NET Task’s exception-based fault state and requires different library and compiler contracts.

By default Raven emits state machines, with a private invocation scheduler delivering native
completion through TaskQueue. Native JIT/AOT hosts now share a private completion
step that distinguishes ready work, pending outcomes and idle sources; callback roots
remain retained through dispatch. Outer native HTTP hosts now explicitly acquire and
complete owned callback deliveries independently of operation-result consumption;
JIT and HTTP AOT entry pumps acknowledge their owned deliveries at the next poll or
entry exit, including guest faults.
This is development infrastructure, not a public
Scheduler API or runtime-managed async suspension. Callbacks must return for other
work to progress.
Task queues are intended to become implementation details; the currently public
TaskQueue remains a transitional API pending coordinated consumer migration.
The interpreter scheduler can retain ready completions without a queue binding;
its current dispatch adapter still uses the invocation's default queue.
A private native submission hook now exercises deferred runtime-owned dispatch with
a retained managed HTTP callback, independently of Task construction. An internal managed Scheduler prototype now exercises the submission service in
interpreter, AOT and JIT with existing state machines. It is not yet the public
Scheduler API or a supported foreign-language ABI. A separate internal awaitable
experiment also passes pending/completed awaits, multiple awaiters, cancellation and
producer faults without pumping the producer queue. Public Task routing still uses
the existing adapter. The experiment now captures an invocation identity and rejects
foreign or expired destinations. Its runtime-owned registration can be activated
or cancelled once; that cancellation does not cancel the Task. Task represents the
operation and state, while Promise is its producer-side completion source.
Explicit observers in that internal experiment can use scoped disposal to withdraw
dormant callbacks without cancelling the Task. Automatic abandoned-await cleanup,
cross-invocation routing and production migration remain open.
A pending await currently resumes through the producer queue; custom queue and UI
affinity are not settled contracts. Ordinary worker joins can still block the queue.

Task.Run is implemented with the integration limits described above.
Thread may become a backend primitive; its public future remains open. Green
threads are a possible later runtime backend, subject to the same sharing and
progress guarantees.
The intended foundation is runtime-owned asynchrony, with language implementations
projecting their async constructs onto a shared lifecycle. Native languages and other
runtimes should eventually integrate through explicit ownership and completion
adapters; no foreign-runtime async ABI is implemented or selected.
A separate, opt-in [Raven runtime-async POC](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/runtime-async/README.md#raven-runtime-async-language-poc-2026-10-11)
now lowers ordinary async methods and Main to retained runtime bodies without
generated state machines. Its source-included provider returns ordinary Task values.
The development interpreter POC also runs the HTTP client/server showcase sources,
including a combined loopback consumer and synchronous `use` across awaits. GC and
cancellation cleanup checks pass. Both native HTTP projects now support explicit
runtime-await selection. Both projects and timeout/cancellation/fault cases now pass
macOS ARM64 JIT/AOT qualification with explicit parked GC roots and terminal teardown.
The development compiler also supports `await` inside ordinary `for` loops, with
iteration state and cleanup preserved across suspension, cancellation and early
exits. Both async lowerings pass interpreter and macOS ARM64 JIT/AOT
[loop-lifetime qualification](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/runtime-async/README.md#await-inside-for-both-lowering-paths--2026-10-11).
The native stack carrier remains temporary POC scaffolding; Windows, production
integration and default readiness remain open.
[Async generators are being investigated](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/runtime-async/async-generators.md);
Raven's .NET support does not yet provide a neoCLR async-iteration protocol or
retained generator execution.
Explicit await mappings also qualify an independent runtime-backed manual operation
and a distinct awaiter. An imported generic interface also supports pending and immediate
implementations through one mapping. The final runtime-facing contract and native
interop bindings remain in design.
Executable samples cover nested awaits, reference/receiver retention, unit results,
cancellation, terminal faults and GC reclamation. This is development support, with
restricted method/await shapes and no default switch. The independent operation
experiments remain interpreter-qualified; compiled evidence covers the managed Task provider.
Production notification/lifetime integration and broader cleanup remain open;
a managed Scheduler API backed by runtime implementation is the selected direction,
with Task/Promise remaining managed. Its members are not yet selected or implemented.
See the [builder contracts](../../docs/async-builders.html)
for the supported compiler integration.

[Related proposals and open questions →](../../proposals/#async)

[Development measurement](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/scheduler/README.md) now includes completed and pending sequential-await workloads
for the current state-machine path in interpreter, JIT and AOT. Startup and external
batch timings are separate. An opt-in Raven runtime-async interpreter comparison now
measures lower execution time but more allocated objects in bounded nested-await
workloads ([results and limitations](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/runtime-async/README.md#matched-language-benchmark--2026-10-11)).
A [matched three-mode microbenchmark](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/runtime-async/README.md#matched-interpreter-jit-and-aot-measurement--2026-10-11)
now also measures native suspension. The current native POC is slower than its
state-machine baseline. Bounded invocation-local stack reuse now reduces mapping
churn and records reserved bytes, with zero reservation after teardown
([measurement](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/runtime-async/README.md#bounded-native-stack-reuse--2026-10-11)).
Production carrier optimization and resident-memory measurements remain open. A matched pending-await comparison also measures
direct versus captured invocation registrations. These exploratory runs do not
establish a general performance improvement.

<a id="feedback"></a>

## Questions and contributions

Report completion, cancellation or scheduling issues with a reproducible example and the expected operation lifetime.

[Discuss on GitHub ↗](https://github.com/marinasundstrom/neoCLR/issues)

Questions, sample programs and documentation corrections are welcome. See [how to contribute](../../#feedback) for ways to participate.

## API reference

[System.Tasks](xref:System.Tasks) · [System.Concurrency](xref:System.Concurrency)

## Timers in development

`System.Concurrency.Timer` accepts either `() -> ()` or `() -> Task<()>`. Start it
with a `Duration` delay, optionally followed by a positive repeating interval.
Repeats wait for callback completion before starting the next interval. Restart,
Stop and Dispose do not overlap or cancel in-flight callback work. A cancelled
callback stops its schedule. Zero delay is deferred; positive fractional milliseconds
round upward. This is invocation-local scheduling, not thread-pool execution.

See the [Timer API](/docs/api/System/Concurrency/Timer/index.html) and the
[tested async scenarios](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/timers/Main.rvn).

### Asynchronous iteration (development)

`System.Collections.AsyncIterable<T>` acquires an `AsyncIterator<T>` with
`GetAsyncIterator()`. Advance with `await iterator.MoveNextAsync()`, read `Current`
after a true result, and await `iterator.DisposeAsync()` when finished. Operations
must not overlap. The managed interfaces are implemented. The explicit retained-runtime async POC
supports `await for` consumption, including awaited cleanup on early exit and
cancellation, with interpreter/JIT/AOT qualification on macOS ARM64. The native
state-machine path and generator `yield` synthesis remain incomplete.
