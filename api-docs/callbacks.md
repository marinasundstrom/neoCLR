# Task callback methods

These three public methods use `Func<System.Void>`, a callback returning neoCLR's
unit value. RavenDoc renders these methods in the generated reference; this guide
explains their queue and continuation contracts.

This is the same queue used by [Task&lt;T&gt;](xref:System.Tasks.Task`1) and
[Promise&lt;T&gt;](xref:System.Tasks.Promise`1). See the
[Tasks guide](/features/tasks/index.html) for tested producer and async examples.

## TaskQueue.Post

`func Post(callback: Func<System.Void>)`

Append a callback to the queue without executing it. A queue pump processes work
in FIFO batches. Work posted by a running callback joins the next batch. This does
not start a thread or make blocking work nonblocking.

## TaskQueue.Run

`func Run(callback: Func<System.Void>)`

Run the supplied callback, then drain this queue until it is empty. Recursive
pumping is a runtime fault. A fault stops execution; this API is not an exception
recovery boundary. While `Run` or `Drain` is active, new promises and async methods use the nearest
active queue. `Promise<T>(queue)` instead uses the queue supplied explicitly.

## Task&lt;T&gt;.OnCompleted

`func OnCompleted(callback: Func<System.Void>)`

Register a callback for either terminal state. When the promise completes or is
cancelled, it posts the callback to its associated queue. Registration after a
terminal transition also posts the callback; it never calls it inline.

The callback receives no argument. Inspect `Outcome` to distinguish completion and
cancellation. `GetResult()` faults for a cancelled or pending task. Application code
normally uses `await` or outcome patterns; this method also serves the compiler's
await protocol.

## Task.Run (development)

`Task.Run(callback)` submits work with shared lexical captures. Its overloads accept
`Func<unit>`, `Func<T>` and `Func<Task<T>>`, returning `Task<unit>` or `Task<T>`.
The task-returning overload follows the inner value or cancellation. It does not
return `Task<Task<T>>` unless you explicitly select the value overload with a task
as its result type.

The first backend starts native threads, with one invocation owning their heap,
services and instruction/frame quotas. A blocked host call releases managed graph
access so other contexts can progress. Guest instruction intervals still serialize
through that graph gate; there is no promise of parallel guest CPU execution or
native-thread affinity. Arbitrary shared updates need application coordination.
Promise completion and queue bookkeeping have runtime-supported atomic regions.

Run uses the default dispatcher, even from a custom queue. Pending awaits retain the
existing producer-queue behavior. An inner task using a custom queue still requires
that queue to be pumped. Ordinary task GetResult remains a nonblocking read.

Entry completion drains submitted native work and continuations. Faults fail the
invocation and request sibling shutdown; there is no task exception/error carrier.
Host cancellation and shutdown are cooperative and cannot interrupt arbitrary native
calls. No per-call cancellation-token overload is added in this slice.

Total submissions currently share the `Limits.frames` bound; completed jobs retain
internal result registrations until invocation exit. Captured output uses the existing
per-job `worker_result_bytes` cap. These host quota choices are provisional. This
runtime-owned, invocation-scoped backend differs from .NET's process-wide thread pool.
See the [task feature page](/features/tasks/#task-run) for the compiled example.

Current Raven integration limitations: use `alias Task = System.Tasks.Task` (or the
fully qualified owner) for Run lookup. Give multi-statement value callbacks a
`Func<T>` local type when overload inference reports a void conversion. Direct await
of `Task<unit>` currently fails importer stack validation; the consumer uses
`completion.Map(_ => 0)` before awaiting. These are tracked limitations, not intended
API semantics. Typed awaits and async unwrapping use the ordinary Task protocol.

Mutable scalar locals captured inside async callers currently do not preserve writes
back to the caller: the reduced case returns 0 rather than 42. Explicit shared objects
retain identity and mutations. This is a semantic compiler/integration release blocker,
not a change to the intended shared lexical capture contract.
