---
uid: M:System.Web.Http.HttpServer.Serve(Function,System.Concurrency.CancellationToken)
---
Starts server-owned concurrent request dispatch with cancellation of the server lifetime.

The [context callback](/docs/http-server.html#server-managed-callbacks) is the
canonical application API; this overload returns a separately constructed response.

## Parameters

`handler`: An asynchronous callback returning `Task<Result<HttpResponse, HttpError>>`.
The server observes each task and disposes successful responses and owned content,
including results produced after connections close.

`cancellationToken`: Pre-cancellation cancels the returned task without closing the
listener. Later cancellation stops acceptance and closes connections. It does not
forcibly terminate arbitrary application work; capture a suitable application token
inside the handler when that work needs cancellation.

## Returns and lifetime

Caller cancellation cancels the lifetime task only after every started handler
settles. `Close()` instead produces successful lifetime completion after the same
join. Rejected startup and listener failure return an error. Per-request errors are
counted and retained as the last request error while other requests continue.

The 16-exchange bound, exclusive acceptance, ownership and runtime-Fault limitations
are described in the [server lifetime contract](/docs/http-server.html#server-managed-callbacks).
