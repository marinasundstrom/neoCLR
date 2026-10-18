---
title: HTTP server lifetime and concurrency
---
# HTTP server lifetime and concurrency

Development behavior after Preview 14. `HttpServer.Accept` and `ServeOne` can be
started concurrently. At most 16 outstanding queued accepts and live exchange
contexts belong to one server; further calls return `HttpError.LimitExceeded`.

Calls waiting for a connection are admitted in FIFO order. Internally there is one
native socket accept at a time, but the next accept starts as soon as a socket is
accepted, before TLS or request parsing finishes on the earlier connection.
Cancelling a queued call removes it and releases its admission slot immediately.
This does not promise callback completion order or concurrent access from threads.

`Accept` still completes after buffering the entire valid request. Its cancellation
token covers waiting and reading; the caller owns the returned `HttpContext` and
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
Accept/ServeOne and default client Send remain buffered; unknown-length output
is still unsupported. This does not introduce
connection reuse. See the [stream contract](streams.md) for buffer ownership.
