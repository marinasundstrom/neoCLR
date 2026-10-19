---
title: HTTP server lifetime and concurrency
---
# HTTP server lifetime and concurrency

Use `HttpServer.Serve(handler)` for HTTP web applications. The server manages
concurrent requests, observes handler tasks and owns response/connection cleanup.
This API is development behavior after Preview 16; use a matching toolchain.

## Server-managed callbacks

Development after Preview 16 adds:

- `Serve(handler) -> Task<Result<System.Void, HttpError>>`
- `Serve(handler, cancellationToken) -> Task<Result<System.Void, HttpError>>`
- `CompletedRequests: long`, `FailedRequests: long`, `LastRequestError: Option<HttpError>`

Serve accepts either `(HttpRequest) -> Task<Result<HttpResponse, HttpError>>`
(the same callback as ServeOne) or `(HttpContext) -> Task<Result<unit, HttpError>>`.
Both have overloads with a CancellationToken. A context handler configures
`context.Response` or calls `context.Respond(...)`, then returns `Ok(())`.
Serve sends that configured response after successful callback completion. An Error
or cancelled task closes the exchange without sending it. Do not call Complete or
retain the context after returning. The server disposes configured content on
success, failure and joined shutdown, including content assigned by late handlers.
All server entry points dispatch after headers. Nonempty Request.Content is a
one-shot stream; body bytes are read asynchronously on consumer demand. Empty
bodies remain buffered empty content. Completing or closing the exchange abandons
an unread body without draining it (connections currently close after each response).

The context variant reuses the response already owned by HttpContext. It observes
the original callback task directly, without a per-request mapping task or Promise.
One dispatcher adapter is shared by all Serve lanes. ServeOne's response callback
uses an adapter per invocation; this is an implementation allocation to measure,
not an allocation-free claim. Existing Task and exchange allocations remain.

This follows ASP.NET Core's
[RequestDelegate context/task shape](https://learn.microsoft.com/en-us/dotnet/api/microsoft.aspnetcore.http.requestdelegate?view=aspnetcore-10.0).
neoCLR deliberately adds typed Result errors and owns deferred transmission after
successful configuration. That keeps transport ownership explicit, but does not
provide ASP.NET's writable response-body pipeline or general middleware contract.

Calling Serve starts the
server-managed dispatcher. Its returned task represents server lifetime, not one
request. Awaiting that lifetime task in async Main is normal: the dispatcher still
advances concurrent requests. If application code needs to coordinate shutdown or
do other work, retain the task and await it after requesting shutdown. Observe its
Result rather than discarding the task. The server owns and observes every request
task; applications do not need a request-dispatch loop. Suspended callbacks do not prevent other requests
from advancing. Synchronous blocking code still blocks the cooperative dispatcher.

Serve owns at most 16 queued/active exchanges. At the bound it waits for a slot;
additional connections wait in the configured TCP backlog. It requires an open idle
server. Duplicate Serve, mixed Accept/AcceptHeaders/ServeOne, and starting Serve with
an outstanding manual exchange return InvalidRequest. Pre-cancellation cancels the
returned task without starting Serve or closing the listener.

Request parse, handler, response validation and transfer errors close that exchange
and serving continues. Cancelled handler tasks count as a request failure. The
cumulative counters cover Serve only; shutdown-driven cancellations are excluded.
LastRequestError retains only the latest failure, not an unbounded error history.
There is no automatic error response or retry. A listener failure stops the server
and returns Error from its lifetime task, taking precedence over concurrent caller
cancellation. Runtime Faults remain terminal.

Close stops accepting and closes connections immediately. Serve's lifetime task
then waits for all started handlers to settle and disposes late successful responses,
including owned streaming content. Close completes that task successfully; cancellation
of the supplied token closes the server and ultimately cancels the lifetime task.
Handlers must settle cooperatively; arbitrary callback work cannot be forcibly
cancelled, and a never-completing handler prevents lifetime completion. Capture an
application cancellation token when the handler's work needs cancellation. This is
joined shutdown after connection closure, not graceful completion of in-flight HTTP
responses. ServeOne retains its existing single-exchange cancellation behavior.

The [tested Serve fixture](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/http-server/serve/Server.rvn)
starts Serve, waits independently for shutdown, closes the listener, then awaits its
lifetime task. HTTP/1.1 connection-close behavior and existing read/write limits remain.

## Lower-level acceptance and single exchanges

Use `Accept` when you need direct context ownership (`AcceptHeaders` has the same
headers-first behavior), and `ServeOne` when you need one callback exchange. They remain supported
for adapters, tests and applications with their own dispatch policy.

Awaiting `Accept()` waits for a context; it does not wait for your subsequent handler.
Awaiting the entire handling/completion step before accepting again serializes the
loop, as does repeatedly awaiting `ServeOne`. To process concurrently, the caller
must supervise a bounded set of processing tasks, observe failures, dispose contexts
and join work on shutdown. Simply dropping a task does not move work onto another
thread or provide that supervision. Prefer `Serve` for this ownership policy.

Development behavior after Preview 14. `HttpServer.Accept` and `ServeOne` can be
started concurrently. At most 16 outstanding queued accepts and live exchange
contexts belong to one server; further calls return `HttpError.LimitExceeded`.

Calls waiting for a connection are admitted in FIFO order. Internally there is one
native socket accept at a time, but the next accept starts as soon as a socket is
accepted, before TLS or request parsing finishes on the earlier connection.
Cancelling a queued call removes it and releases its admission slot immediately.
This does not promise callback completion order or concurrent access from threads.

`Accept` completes after valid headers. Its cancellation token covers waiting and
header reads; subsequent body reads use their own token; the caller owns the returned `HttpContext` and
must complete or dispose it. `ServeOne` owns that lifetime through its callback and
response send. Closing the server cancels queued work and closes owned contexts.

The request deadline begins after connection acceptance. Each pending socket
transfer also has the existing five-second ceiling, so a longer configured request
timeout does not remove that shorter bound. Waiting for a connection has no deadline.

## Stream-backed responses

`HttpContent.FromStream(input, length, leaveOpen)` can supply a server response
through `context.Respond(status, content)` followed by `await context.Complete()`.
The same content contract supplies client uploads. Length is required and bounded
by 1 MiB. HTTP framing headers are validated before the producer is read.

Development after Preview 16: the server sends the headers first, then requests
at most 4 KiB at a time.
It waits for each write before reading again. Short reads are valid; zero before
the declared length, or a count larger than requested, returns
`HttpError.Content(StreamError.IoFailure)`. Producer errors retain their
`StreamError`. Once headers have been sent, a failure closes the connection;
it cannot replace the already-started response with an error status.

A successfully admitted stream is one-shot. Completion/failure/cancellation closes
owned sources once; `leaveOpen` preserves borrowed sources. HEAD advertises the
length without reading the source. Zero-length content is also never read.
Response disposal still follows context ownership. Validation failures before
consumption do not take ownership of the source.

The response-write deadline also cancels a pending producer read. Development
after Preview 16 accounts for the network clock's millisecond rounding when
scheduling that cancellation, preserving TimedOut rather than a cancelled task
when only the deadline expires. Producers must
honor the supplied cancellation token and settle their task; cancellation remains
cooperative. Caller cancellation returns a cancelled task; deadline expiry returns
`HttpError.TimedOut` after acknowledgement. Close cancels the pending producer and
closes owned content. No unbounded producer queue or complete body copy is created.

Incoming bodies can now be read through [headers-first APIs](http-body-streams.md).
`Serve`, `Accept`, `ServeOne` and default client `Send` buffer incoming bodies; unknown-length output
is still unsupported. This does not introduce
connection reuse. See the [stream contract](streams.md) for buffer ownership.

## Deferred body consumption (development after Preview 16)

This follows [ASP.NET Core request-body streams](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/middleware/request-response?view=aspnetcore-10.0):
headers are available independently of asynchronous body consumption. neoCLR uses
HttpContent/InputStream rather than a PipeReader. Header reads can receive a bounded
prefix of the body; that read-ahead is retained without copying the whole payload.
Later framing, EOF, size and read errors surface during consumption. A handler that
does not read the body does not validate its unread framing. Declared-length and
header validation still happen before dispatch. Keep-alive draining/reuse is not
implemented. Lower buffering is an allocation benefit; throughput needs measurement.
