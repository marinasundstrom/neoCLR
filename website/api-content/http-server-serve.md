---
uid: M:System.Web.Http.HttpServer.Serve(Function)
---
Starts concurrent, server-owned request dispatch. This method returns immediately;
its task represents the lifetime of the server, not an individual request.

## Parameters

`handler`: An asynchronous callback returning `Task<Result<HttpResponse, HttpError>>`.
The server observes every started task and owns each successful response, including
late or invalid responses and their owned content. It does not create an OS thread
per handler; blocking synchronous code blocks the cooperative dispatcher.

## Returns and lifetime

The lifetime task succeeds after `Close()` and settlement of every started handler.
An already active/closed server rejects startup with `InvalidRequest`. Listener
failure terminates serving with a transport error. Request errors close only their
exchange, update `FailedRequests`/`LastRequestError`, and serving continues.

The server admits at most 16 exchanges and owns acceptance exclusively. Closing
connections does not force arbitrary handler tasks to finish; a handler that never
settles prevents lifetime completion. See the [server lifetime contract](/docs/http-server.html#server-managed-callbacks).
