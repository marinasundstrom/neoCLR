---
uid: N:System.Concurrency
---
## Workers and cancellation

Thread and ThreadPool run work through isolated runtime workers. CancellationToken,
CancellationTokenSource and CancellationRegistration coordinate cooperative
cancellation. Worker lifetime and cancellation are explicit; callbacks do not imply
arbitrary shared-object access between workers.

Use [Tasks](/features/tasks/) for completion and composition. The
[API overview](/docs/index.html#explicit-threads-in-development) describes the
current worker model; these APIs do not yet promise a green-thread scheduler.

`Timer` accepts synchronous callbacks or callbacks returning `Task<()>`.
Use `Start(Duration)` for one run or `Start(delay, interval)` for repeating work.
Repeating intervals begin after callback completion, including awaited work. Restart
never overlaps an in-flight callback; Stop and Dispose suppress future runs without
cancelling callback work. Cancelled callback tasks stop their schedule. Timers require
invocation scheduler progress; they do not create threads.

`CancellationTokenSource.CancelAfter(delay)` schedules a cancellation request and
replaces an earlier countdown. `CancelAfter(Optional<Duration>.None())` disables it.
Cancel and Dispose release the countdown. An already requested source cannot be reset;
operations must still acknowledge cancellation and finish their cleanup.
