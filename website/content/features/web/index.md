---
title: Web and HTTP
---
# Web and HTTP

**Development experiment after Preview 9.** Raven client and server applications can exchange a small UTF-8
response through neoCLR TCP sockets, with DNS on the client. HttpClient, HttpServer, request/response types,
content and handlers are development APIs in System.Web.Http. They require a matching
development toolchain and are not part of the published Preview 9 SDK.

## A request and its content

```raven
{{HTTP_CLIENT_SAMPLE}}
```

This function comes from the [tested source](/samples/http-client/Main.rvn).
`await ...?` propagates a failed request; the response exposes headers and byte content.
ReadText returns a Task/Result and strictly decodes UTF-8. The prototype buffers the
complete small body before returning a response, so decoding currently finishes
immediately. Wire lengths count bytes, not characters.

## Attach behavior with handlers

HttpClient delegates `Send(HttpRequest)` to an `HttpHandler`. Its contract returns
`Task<Result<HttpResponse, string>>`. A forwarding handler receives an inner handler:
it can act before sending and after the result arrives. Several handlers can form
a pipeline, with socket transport at the end. A fake handler can return a response
without a network connection.

The sample checks nested before/after ordering, preserves errors and replaces the
transport in tests. This follows the role of .NET's HttpMessageHandler and
DelegatingHandler, adapted to Task/Result. Logging, authentication and retry policy
are possible uses of this boundary; they are not implemented features. In particular,
retries need decisions about replaying requests and safe methods.

## Run the controlled example

[Download the client, handler checks and verifier](/samples/http-client.zip).
The archive includes run instructions and requires matching development
artifacts. A local Python peer fragments headers and a `Café 🌍` body; the verifier
also runs a .NET comparison client. The peer stays open until the client finishes,
checking that completion follows Content-Length rather than waiting for EOF.
Malformed/truncated responses and collection during pending work are exercised too.

## Receive a request and return a response

```raven
{{HTTP_SERVER_SAMPLE}}
```

This [server callback](/samples/http-server/Server.rvn) builds a byte response.
`HttpServer.Listen("127.0.0.1", 0, 4)` binds a loopback listener; `GetLocalPort()`
reports the selected port. `ServeOne(Respond)` accepts one GET, awaits the callback,
sends its response and closes that connection. The caller closes the listener.
`Close()` stops listening; an already accepted exchange remains active.

[Download the server and interoperability verifier](/samples/http-server.zip).
It pairs separate neoCLR processes, tests an independent .NET client, and sends
fragmented and invalid raw requests. Small-heap runs collect during pending work
and finish with no live managed objects. This is a bounded exchange, not an
application hosting framework.

Received headers have lowercase names and trimmed surrounding whitespace.
The server requires exactly one Host and no request body; optional Content-Length
must be `0`. It validates application response headers, computes the byte length
and adds `Connection: close`. Malformed requests or callback errors close the
connection without an HTTP error response. Only 200 responses are supported.

## Read a JSON report

```raven
{{HTTP_JSON_SAMPLE}}
```

The next [application sample](/samples/http-json/http-json/Client.rvn) fetches a sensor
report from a neoCLR server. `Summarize` checks fields and types, reads the first
measurement and constructs a local JSON acknowledgement. Each `?` propagates an
error from its own layer: HTTP, UTF-8 decoding or JSON access. The acknowledgement
is printed locally; the sample does not implement POST.

[Download the JSON client/server sample](/samples/http-json.zip). It includes the
application-local JSON codec and a verifier using independent Python HTTP peers.
The codec handles the six JSON value kinds with explicit field access and construction;
its 128-byte, four-container-depth and 32-value bounds are demonstration policies.
It remains exploratory source, not a public runtime-library JSON API.

## Current limits

Only plain HTTP/1.1 GET and a 200 response with exactly one Content-Length are
supported. The experiment bounds URLs to 1,024 bytes, headers to 2,048 bytes and 16
fields, and bodies to 1,024 bytes. It rejects duplicate lengths, transfer encodings
and content encodings. It does not implement chunking, TLS, redirects, pooling,
streaming content or general HTTP status handling. Text always means strict UTF-8.

DNS and connection attempts have separate bounds. Each nonempty socket transfer
now has a five-second deadline; HTTP closes its connection when a transfer fails.
A silent peer therefore ends with an error, but a trickling peer can keep completing
short transfers. There is no whole-request deadline, accept deadline or bound on an
application handler task yet. The verifier's watchdog is only a test guard.
The transport uses a 256-byte reusable buffer and handles short transfers. This
is a correctness POC, not a performance benchmark. Generated async states still perform suspension.

## Direction

The client/server greeting and JSON report are in place. Request lifetime/deadlines,
a public JSON contract and broader request/response behavior remain open gates. The System.Web.Http boundary
separates HTTP policy from [networking](/features/networking/); a future transport
could use sockets or a host facility. Handler ownership, concurrency, cancellation
and a structured error model need concrete cases as these provisional APIs evolve. No complete HTTP stack or runtime suspension is claimed.

Browse the [HTTP API reference](xref:System.Web.Http) for constructors, members,
parameters and ownership details. The parser and operation adapter remain internal.

One error-model candidate groups request, transport and response failures, nesting
resolver/socket causes where useful. HTTP statuses should remain response values as
status support expands; decoding errors belong to ReadText. This is a proposal:
the current POC returns explicitly provisional string errors.
