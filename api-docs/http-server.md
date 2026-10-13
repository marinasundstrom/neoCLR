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

Streaming bodies are under development, not yet part of this contract. Headers and
bodies remain bounded, and this change does not introduce connection reuse.
