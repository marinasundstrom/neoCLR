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
