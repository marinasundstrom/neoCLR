# HTTP connection reuse and pipelining investigation — 2026-10-25

The author requested this investigation after the bitmap-seeding GC screen.
This is a protocol/ownership baseline, not implemented persistent HTTP support.

## Executed baseline

`probe.py` runs retained single-owner sync and concurrent HTTP hosts. For each it
sends a keep-alive GET, then opens another connection and sends two GET requests
in one write. Both hosts return exactly one complete `Hello, World!` response with
Connection: close and EOF in each case; all four served requests validate and both
hosts shut down successfully. The second pipelined request is not served. The
initial invocation selected a missing sync artifact and is retained separately;
the completed report records actual executable hashes and all received bytes.

```sh
python3 benchmarks/http-server/pipelining/probe.py \
  --server target/concurrent-http-owners-plain/sync-1 \
  --server target/gc-index-only-owners/live-index-only-1 \
  --output target/pipelining-observation.json
```

The probe asserts today's close behavior. It must be replaced with positive reuse
and ordering qualification when support is implemented; passing it does not mean
pipelining is implemented. The retained sync executable is a protocol control, not
a fresh comparative performance build.

## .NET and protocol reference

Primary sources retrieved 2026-10-25:

- [.NET 10 Kestrel HttpProtocol](https://raw.githubusercontent.com/dotnet/aspnetcore/v10.0.0/src/Servers/Kestrel/Core/src/Internal/Http/HttpProtocol.cs): HTTP/1 state belongs to a connection and is reset between requests; keep-alive begins enabled and parsing/error handling may disable it. This is an implementation reference, not a requirement to copy its classes or pipelines.
- [RFC 9112 §9.3 and §9.3.2](https://www.rfc-editor.org/rfc/rfc9112.html#section-9.3): persistence requires unambiguous message framing and consuming the request body or closing. Pipelined responses must preserve request order; parallel execution is optional. Sequential processing is therefore a viable first implementation.
- [TechEmpower comparison boundary](../../gc-suite/techempower-comparison.md): keep these new workloads separate from the current close-per-request GC series.

## Current ownership seams

In `runtime/raven/src/System/Web/Http/HttpServer.rvn`:

- `HttpRequestDecoder.PushCore` records `Consumed`, so the parser already exposes a
  message boundary. It need not discard or reinterpret the next request's bytes.
- `HttpServerExchange.OnReceived` transfers read-ahead to `HttpBodyInput` for streamed
  bodies, but otherwise completes the first context and releases receive storage.
- `HttpContext.Close` disposes both messages and closes the transport. `Finish`
  always calls it, including after successful response transmission.
- The response encoder unconditionally emits Connection: close.

`HttpBodyInput` has private position/extent/remaining state, and its public `Close`
closes the connection even when `ownsConnection` is false. The flag currently only
controls automatic closure when body consumption completes. Persistence needs a
separate completed-body handoff; changing this flag alone would be insufficient.

## Proposed first implementation

A private session owns the connection, unread input window and terminal state.
Each exchange borrows that transport. After response completion and handler/body
completion, retire request-local content and callback roots, then create the next
exchange from the unread window. Keep at most one active handler/response per
connection initially; ordered completion avoids a response reorder queue and its
retained object graphs. Other connections remain independently scheduled.

Do not merely remove the close header. Reuse must be conditional on valid framing,
complete body consumption and absence of close/error/cancellation. An unconsumed
body may safely force connection closure initially; bounded draining can be a
separate measured policy. Explicit server/context abort and transport disposal
must remain idempotent. Preserve HTTP/1.0 limitations until explicitly qualified.

Suspend an idle session without retaining the previous request/response graph.
The session/read buffer must remain rooted while receives are pending under both
collectors. Do not move live managed sessions between isolated owner heaps.
Connection and request admission limits need distinct accounting; a quiet client
must not monopolize all Serve lanes indefinitely.

## Qualification before throughput

1. Two sequential requests on one connection and two coalesced pipelined requests;
   split every header boundary and preserve a partial next request.
2. Ordered distinguishable responses, including delayed handlers and errors.
3. Content-Length/chunked body followed immediately by the next request; unread
   body, malformed framing, explicit close and incomplete EOF cases.
4. Request cancellation, idle timeout, server shutdown, late handler completion and
   repeated disposal, with no callbacks accessing retired exchange state.
5. Force GC between messages and during suspended reads; verify old graph cleanup
   and live session survival in JIT/AOT, sync/concurrent modes.
6. Measure close, sequential keep-alive and pipelined traffic separately with matched
   Kestrel peers, checked response framing/order and client saturation reporting.

No API signature, compiler bridge or runtime protocol behavior changes in this
investigation. The next implementation unit is private session/input ownership,
with executable framing tests before changing public close/completion behavior.
