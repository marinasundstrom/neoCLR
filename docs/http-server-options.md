# HTTP server admission and deadlines

Development increment on `codex/http-foundation`, following the integrated JSON
baseline `2c8204d5` and the [HTTP framing foundation](http-foundation.md).

`HttpServer.Listen(address, port, backlog, options)` validates and copies an
`HttpServerOptions` before binding. Unsupported values return
`HttpError.InvalidRequest`; transport failures retain their existing errors.
Mutating the options later cannot change a listening server or an active exchange.
The three-argument overload preserves the existing defaults.

| Property | Default | Admitted configuration |
| --- | --- | --- |
| MaxRequestBodySize | 1,048,576 bytes | 0–1,048,576 decoded bytes |
| MaxRequestHeadersTotalSize | 32,768 bytes | 1–32,768 request-line/header bytes, including CRLF |
| MaxRequestHeaderCount | 100 | 1–100 fields, including Host |
| RequestReadTimeout | 15 seconds | Positive duration through 60 seconds |
| ResponseWriteTimeout | 15 seconds | Positive duration through 60 seconds |

The read clock starts after accepting a connection and covers headers and body.
Waiting in Accept and application handler execution are outside this clock. The
write clock starts after response encoding; partial transfers do not renew it.
Fractional milliseconds round upward. Cancellation retains precedence over an
expired networking deadline. There is no infinite-timeout mode in this slice.

The shared body ceiling, chunk-framing budget, outgoing-response budgets and
16-exchange cap remain unchanged. Zero body bytes allows empty fixed/chunked
requests and rejects nonempty ones. Limits neither reserve memory nor guarantee
that a permitted size completes within its deadline. Native safepoint-stress GC
still prevents qualifying the 1 MiB wire boundary; no GC policy changes are made.

## .NET comparison and deliberate limits

[KestrelServerLimits](https://github.com/dotnet/aspnetcore/blob/main/src/Servers/Kestrel/Core/src/KestrelServerLimits.cs)
provides separate request body, header-byte and header-count limits. The names and
units here follow that structure. Kestrel's body limit can be nullable/unlimited
and its header timeout is separate from body data-rate policy. neoCLR currently
buffers the entire request and uses one read deadline, so `RequestReadTimeout`
explicitly names that narrower implementation instead of promising Kestrel's
header/data-rate semantics. `ResponseWriteTimeout` covers its buffered send phase.

Validated snapshots at Listen give predictable per-server behavior without mutable
shared configuration affecting in-flight requests. The cost is that changing
configuration requires a new listener. Options setters themselves do not fault;
validation uses the server's established Result contract at the resource boundary.

These settings can reduce admission within existing hard ceilings. Larger or
unlimited bodies, dynamic per-request policy and streaming remain gaps, not new
platform rules. The 60-second deadline ceiling comes from the current bounded
NetworkDeadline provider; removing it requires provider validation. The comparison
with [.NET SocketsHttpHandler.MaxResponseHeadersLength](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.socketshttphandler.maxresponseheaderslength?view=net-10.0)
also shows that client response configuration belongs to separate client/handler
policy. This increment does not add ambiguous settings shared across both roles.

## Validation

The [compiled consumer and independent TCP peer](experiments/http-server-options/)
cover invalid configuration before binding, snapshot isolation, fixed/chunked body
boundaries, cumulative chunks, zero admission, header bytes/counts and read expiry.
The configured 60-second write budget is exercised by normal response completion;
forcing backpressure/write-timeout expiry remains unqualified in this small probe.
