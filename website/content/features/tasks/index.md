# Tasks and async

A `Task<T>` represents an operation and its eventual result. It does not imply a
thread. Use `await` to continue when the operation completes, including in `Main`.
Import `System.Concurrency.Tasks.*` for Task and Promise.

## Start and await work

```raven
{{TASK_RUN_SAMPLE}}
```

This prints `completed`. `Task.Run` submits work and returns its completion task.
A value-producing callback returns `Task<T>`; a task-producing callback is unwrapped.
Captured objects retain their identity, so shared updates need coordination.

[Download the example](/samples/task-run-await.rvn) ·
[Supported entry points](/raven/#entry-points)

## Return a completed result

`Task.CompletedTask` provides an already successful `Task<()>`.
`Task.FromResult(42)` creates a completed `Task<int>`. Neither schedules work.
Use a `Promise<T>` when adapting an operation that will complete later: the producer
completes or cancels the promise, while consumers receive its Task.

## Expected errors and cancellation

Use `Task<Result<T, E>>` for an operation that can return an expected error.
Await obtains the Result; `?` propagates its error:

```raven
{{TASK_PROPAGATION_SAMPLE}}
```

`await input?` awaits first and then propagates; `(await input)?` is also valid.
Cancellation is a distinct task outcome, never `Result.Error`. A runtime Fault is
terminal and cannot be recovered through Task. This differs from .NET's faulted Task.

[Complete propagation example](/samples/library-task-propagation.rvn) ·
[Option and Result](/features/outcomes/)

## Request cancellation

`System.Concurrency.CancellationTokenSource` owns cancellation authority. Pass its
Token to cooperating I/O; `CancellationToken.None` makes no cancellation request.
Requesting cancellation does not mean the operation has stopped. It must finish
resource cleanup before acknowledging cancellation. An already completed Task keeps
its result. Dispose token registrations and sources when their lifetimes end.

For HTTP, configure [HttpClient.Timeout](/docs/api/System/Web/Http/HttpClient/index.html)
before sending. Timers and cancellation require scheduler progress; they cannot
preempt arbitrary synchronous work.

## Execution and resource lifetime

The runtime chooses how submitted work executes. The current Task.Run backend can
progress other contexts during a blocked host call, but it does not promise parallel
execution of arbitrary guest CPU code. Entry completion drains pending work.

The native compiler supports await in functions, class methods and ordinary for
loops, with locals and cleanup retained across suspension. The SDK selects
runtime-await and supplies private providers for async functions and generators.
Native suspension supports macOS ARM64 and Windows x64 with matching runtime/JIT
helpers. Compiler state machines remain an explicit project option.
This does not promise parallel guest execution or unrestricted portable green threads.

## Read further

- [Task and Promise APIs](xref:System.Concurrency.Tasks) — results, completion and callbacks.
- [Cancellation and timers](xref:System.Concurrency) — cancellation ownership and delayed work.
- [Native compilation](/features/native-compilation/) — execution modes and supported targets.
- [Async compiler contracts](/docs/async-builders.html) — integration details.
- [Runtime suspension design and validation](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/runtime-async/README.md) — contributor material.

<a id="await"></a>

<a id="native-compiler-integration-development"></a>

<a id="task-run"></a>

<a id="submitting-work-with-shared-captures-development"></a>

<a id="promise"></a>

<a id="outcomes"></a>

<a id="worker-limits"></a>

<a id="host-cancellation"></a>

<a id="dispatch"></a>

<a id="try"></a>

<a id="release-checkpoint"></a>

<a id="pending-read"></a>

<a id="direction"></a>

<a id="feedback"></a>
