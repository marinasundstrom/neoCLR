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
