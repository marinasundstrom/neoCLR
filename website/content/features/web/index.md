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
The sample maps typed HTTP failures to display text at its application boundary; `?` propagates that result; the response exposes headers and byte content.
ReadText returns a Task/Result and strictly decodes UTF-8. The prototype buffers the
complete small body before returning a response, so decoding currently finishes
immediately. Wire lengths count bytes, not characters.

## Base addresses and URI values

`HttpClient.BaseUri` is an `Option<string>`, initially None. With no base configured,
`Get` requires an absolute URL. With `Some("http://localhost:8080/api/")`,
`Get("items")` constructs `/api/items`. A base ending in `/api` instead constructs
`/items`: the last path segment is replaced, following URI resolution rules.

`Get(string)` and `Get(Uri)` have the same behavior. Root-relative paths, parent paths,
query-only references and empty references use the existing Uri resolver. A configured
base rejects absolute URLs and `//host/path` references, so helpers cannot replace its
authority. `Send` takes an already constructed request and does not apply the base.

Invalid URI syntax becomes `HttpError.InvalidUri` with its original UriError. Unsupported
HTTP forms and invalid base/address combinations are reported before handler dispatch.
Base text is checked when constructing the request; assigning it does not perform I/O.
Changes affect subsequent requests. Unlike .NET's Uri-valued BaseAddress, this property
accepts optional text and can change after a request has been sent.

[HttpClient API →](xref:System.Web.Http.HttpClient) · [Uri API →](xref:System.Uri)

## Attach behavior with handlers

HttpClient delegates `Send(HttpRequest, CancellationToken)` to an `HttpHandler`. Its contract returns
`Task<Result<HttpResponse, HttpError>>`. A forwarding handler receives an inner handler:
it can act before sending and after the result arrives. Several handlers can form
a pipeline, with socket transport at the end. Forwarding handlers pass the token to
their inner handler. A fake handler can return a response
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
reports the selected port. `ServeOne(Respond)` accepts one GET, HEAD, POST, PUT, PATCH or DELETE, awaits the callback,
sends its response and closes that connection. The caller closes the listener.
`Close()` stops listening and closes owned exchanges, including pending accepts and callback waits.

[Download the server and interoperability verifier](/samples/http-server.zip).
It pairs separate neoCLR processes, tests an independent .NET client, and sends
fragmented and invalid raw requests. Small-heap runs collect during pending work
and finish with no live managed objects. This is a bounded exchange, not an
application hosting framework.

Received headers have lowercase names and trimmed surrounding whitespace.
The development server requires exactly one Host. GET/HEAD/DELETE bodies remain unsupported;
POST, PUT and PATCH read up to 1,024 bytes according to Content-Length before calling the handler.
Without Content-Length, a request has an empty body. It validates application response headers, computes the byte length
and adds `Connection: close`. Malformed requests or callback errors close the
connection without an HTTP error response. Final statuses 200–599 are supported in development. Statuses 204, 205 and 304 require empty content; the server omits Content-Length for 204/304.

## Own an HTTP exchange

In development, `HttpServer.Accept()` returns a context with the buffered request and
outgoing response. The context keeps their lifetime together:

```raven
{{HTTP_CONTEXT_SAMPLE}}
```

`context.Response.Respond(statusCode, content)` sets status and content without sending.
The status-only overload clears the content. `context.Respond(...)` forwards those
same operations, while `RespondText` supplies UTF-8 text/plain content. `Complete(token)`
validates, snapshots and sends the response, returning a Task with a Result. Completion
ends the scope on success, failure or acknowledged cancellation. `Close()`/`Dispose()`
ends the scope without sending; both are safe to repeat. Sending twice returns an error.

The server permits at most 16 outstanding accept/context scopes, with one native accept
pending at a time. Applications can hold two contexts and complete the second before the
first. `ServeOne` uses this same ownership path. Cancellation can stop waiting for a
callback but cannot preempt its code; shutdown closes exchanges rather than draining
handlers. Request reads and response sends have separate provisional 15-second budgets.
These contracts still use buffered content and one connection per exchange.

See [HttpContext](/docs/api/System/Web/Http/HttpContext/) in the API reference for the
complete signatures and limits. A future interface split could give inbound and outbound
requests/responses different capabilities. Today they remain concrete message classes;
configuring a received client response only changes the local object.

## Exchange JSON reports

```raven
{{HTTP_JSON_SAMPLE}}
```

The [application sample](/samples/http-json/http-json/Client.rvn) constructs a report
with the public System.Data.Json DOM and POSTs it to a neoCLR server. The server
uses HttpContext to parse the body and return a JSON acknowledgement with status
201. Bad JSON or report shapes return 400; unknown routes return 404.

Application-owned converters project HttpError and JsonError into AppError while
retaining the original causes through helpers and async methods. The completion
callback converts errors to display text at its reporting boundary. Ordinary steps use propagation. The
server handles invalid input explicitly where it chooses the HTTP response.

[Download the client/server sample](/samples/http-json.zip). It uses the runtime
library's JSON parser and includes checks against independent Python HTTP peers.
Serialization is synchronous; HTTP bodies are currently buffered. The provisional
DOM limits are 128 UTF-8 bytes, four container levels and 32 values. The development
sample has an opt-in variant using the provisional [object serializer overloads](/docs/json/).
It maps both report and acknowledgement models through checked runtime reflection:
constructors, getters and setters execute normally. Flat public `string`, `int` and
`bool` properties are supported, using exact property names. Writable properties
must be present; extra JSON fields are ignored. Nested models, null mapping and
naming policies are not supported. The sample's property names match its lowercase
wire names explicitly. Both mapped peers pass against
independent servers/clients and together in an isolated run. An earlier overlapping
test run reached the transport deadline, so latency under load remains unverified.
The downloadable demo defaults to direct DOM mapping; its verifier selects this
experiment with `--mapped`.

## Current limits

Plain HTTP/1.1 GET, HEAD, DELETE and buffered POST/PUT/PATCH support final statuses 200–599 in development. Responses use a single Content-Length, bounded chunked coding,
or connection-close framing. HEAD and bodyless 204/304 complete after headers. Informational responses are not
yet supported. The experiment bounds URLs to 1,024 bytes, headers to 2,048 bytes and 16
fields, and decoded bodies to 1,024 bytes. Chunk framing has a separate 2,048-byte budget.
It rejects ambiguous framing, chunk extensions/trailers, other transfer codings
and content encodings. It does not implement TLS, redirects, pooling,
streaming content or informational response handling. Text always means strict UTF-8.

The socket handler now has a provisional 15-second exchange budget, starting before
DNS lookup and ending with the buffered response. DNS, connection and transfers keep
their shorter five-second bounds. Short progress does not renew the shared budget;
expiry closes the connection and returns an error. This requires scheduler progress
and does not preempt guest code. The limit is not configurable yet.

Request construction, custom pipeline work outside the socket handler, server accept
and server application-handler waiting are not covered by this exchange budget.
The verifier's watchdog is only a test guard.
The transport uses a 256-byte reusable buffer and handles short transfers. This
is a correctness POC, not a performance benchmark. Generated async states still perform suspension.

## Direction

The client/server greeting, JSON report and socket-backed client exchange budget are
in place. Server cancellation ownership, a public JSON contract and broader
request/response behavior remain open gates. The fixed transport budget is a POC
policy, not a complete HttpClient timeout configuration model. The System.Web.Http boundary
separates HTTP policy from [networking](/features/networking/); a future transport
could use sockets or a host facility. Shared handler concurrency and ownership need further cases as these provisional APIs evolve. No complete HTTP stack or runtime suspension is claimed.

Browse the [HTTP API reference](xref:System.Web.Http) for constructors, members,
parameters and ownership details. The parser and operation adapter remain internal.

HTTP operations now return the standard `HttpError` union. Match NameResolution
and Transport to inspect the original DNS or socket cause. InvalidRequest, Protocol,
Unsupported and LimitExceeded distinguish validation and message failures; TimedOut
represents the shared exchange deadline, and Handler can carry an application error.
InvalidUri is available for URI integration. HTTP statuses remain response values as
status support expands. ReadText still has its separate provisional string error for
UTF-8 decoding. Default HttpError is inactive and formats as Empty.

## URI references in development

`System.Uri` now parses escaped ASCII references and resolves relative references
against an absolute base. `Parse` returns `Result<Uri, UriError>`; `Resolve` accepts
either a string or another Uri. A trailing slash matters: resolving `child` against
`http://example.test/api/` gives `/api/child`, while a base ending in `/api` gives
`/child`. Query-only references preserve the base path, and literal dot segments are
removed during resolution.

This first iteration preserves exact text for equality and hashing. It does not
perform network access, implicit escaping, IDNA conversion or IPv6-literal parsing.
Input is limited to 4096 bytes. A parsed Uri does not imply transport support.
UriError is a development union with named cases and generated pattern support.
Its former per-case `Is*`/`Get*` helpers have been removed; match the cases directly.

The [API reference](/docs/api/System/Uri/) describes both overloads and the limits.

Typed HttpError results, BaseUri resolution and token-aware Send/Get/GetString are
integrated in development. Both string and Uri overloads are available; tokenless
client overloads use CancellationToken.None. GetString decodes the buffered body as
strict UTF-8 and preserves HTTP errors. Invalid bytes produce HttpError.Protocol;
GetString accepts 200–299 and reports other statuses as HttpError.UnsuccessfulStatus
with the status code. Send/Get return these responses as ordinary response values;
IsSuccessStatusCode lets callers apply their own policy. Charset handling and streaming
remain future work.

Cancellation tokens are invocation-local. Pre-cancellation skips parsing and handler
dispatch. During an exchange the socket handler waits for native acknowledgement and
closes its connection before its Task becomes cancelled. A completed native result
can survive a later request; cancellation between phases prevents the next operation.
Separate requests retain separate connections and cancellation state. DNS host work
may continue after guest cancellation, with its bounded capacity still charged.

Custom HttpHandler implementations now require Send(request, cancellationToken).
They must forward or honor the token and finish owned cleanup before reporting
cancellation. The client does not force completion of an uncooperative handler or
dispose injected handlers. Existing implementations must be updated and rebuilt.

URI/URL encoding utilities are also planned separately. Their design will distinguish
path segments, query values and form data; the current Uri parser expects text that
has already been escaped.

## Inspect response properties

Development responses now expose `HttpStatusCode` names such as `NotFound`, while
retaining unnamed numeric status values. `IsSuccessStatusCode` checks the 200–299
range. Cast the enum to `int` when you need its numeric value.

Property patterns are one option alongside ordinary property access. For a
`Result<HttpResponse, HttpError>` named `result`, this target-tested form matches a
status and captures the headers:

```raven
if let Ok(HttpResponse { StatusCode: HttpStatusCode.NotFound, Headers: headers }) = result {
    Console.WriteLine(headers.Count)
}
```

An `is` pattern places `let` on the capture instead:

```raven
if result is Ok(HttpResponse { StatusCode: HttpStatusCode.NotFound, Headers: let headers }) {
    Console.WriteLine(headers.Count)
}
```

The outer `let` in a binding or `if let` already supplies capture binding. Both
forms use readable properties; neither requires `Deconstruct` or value equality.
Positional deconstruction remains a separate design question. Neither form is a
preferred style for every caller. The [HTTP API reference](/docs/api/System/Web/Http/)
describes the response, status enum and errors.

## Post buffered text

Development POST support uses the same handler pipeline, BaseUri rules and cancellation
token as GET. This sample propagates HTTP failures on its normal path. It separately
chooses to reject non-success statuses, then adapts ReadText's provisional string error
to HttpError at the application boundary.

```raven
{{HTTP_POST_SAMPLE}}
```

`HttpContent.FromText` encodes UTF-8 and sets the outbound Content-Type to
`text/plain; charset=utf-8`. `HttpContent(bytes)` preserves binary bytes without a
Content-Type; its second constructor accepts one explicitly. `HttpRequest.Content`
exposes the body to the server handler, and incoming headers retain Content-Type.
The socket provider computes Content-Length from bytes and snapshots the request
before DNS. Content sources must remain stable while consumed.

[Download the POST echo example and focused checks](/samples/http-post.zip).
The checks cover neoCLR peers, an independent raw peer and a .NET client, including
empty and binary bodies. Header append/remove operations, OPTIONS, streaming,
streaming, compression and Expect/continue are still outside this checkpoint.

## Looking up headers

In development, requests and responses provide `GetHeaderValues(name)`. Names use
ASCII case-insensitive comparison. The returned sequence contains every stored matching
field value in order; it is empty if the name is absent or invalid. Empty values remain
present, and repeated fields are not joined or split at commas. For example,
`response.GetHeaderValues("Set-Cookie")` preserves separate cookie fields.

Stream-backed `HttpContent` is planned. Content is currently buffered; stream ownership,
cancellation and unknown-length framing still need contracts and implementation.

## Adding request headers

In development, `request.WithHeader(name, value)` returns a new request with that
application header set, replacing existing matches case-insensitively. It returns a
Result, so request preparation can use `?` before calling `HttpClient.Send`:

```raven
let acceptsText = request.WithHeader("Accept", "text/plain")?
```

The original request keeps its headers. Content is shared, so keep it stable during
use. The current helper validates ASCII names and printable ASCII values; it does not
parse field-specific syntax. Transport headers such as Host and Content-Length are
reserved, and Content-Type comes from HttpContent. The provider allows 13 stored
application fields and at most 2,048 encoded header bytes. Stream-backed content
remains planned.

## Other request methods

Development APIs include `Put`, `Patch` and `Delete`, with string/Uri addresses and
optional cancellation tokens. They use the same BaseUri and handler pipeline as Get
and Post. PUT/PATCH take HttpContent; DELETE has no body in this checkpoint. Non-success
statuses remain responses, so callers choose their status policy:

```raven
{{HTTP_VERB_SAMPLE}}
```

The server accepts these methods and exposes their buffered content to its handler.
PATCH content is opaque: applications choose its media type and behavior. The library
does not apply patches. HEAD returns headers with empty content; the server computes representation length from the buffered response without sending its body. OPTIONS is not implemented.

## JSON DOM

The development `System.Data.Json` API can parse, inspect, construct and write a small
JSON document through strings or streams. `JsonValue` is a closed hierarchy:
`JsonObject` and `JsonArray` expose their own container methods; strings, numbers,
booleans and null have distinct node types. A missing member differs from a present
JSON null. Numbers retain their JSON spelling until an explicit conversion is requested.

```raven
{{JSON_DOM_SAMPLE}}
```

The tested example adds an acknowledgement to an object and propagates failures.
`JsonSerializer` uses StreamReader/StreamWriter and leaves supplied streams open;
the caller controls flushing and closing. The sample also runs through
[MemoryStream](/docs/api/System/IO/MemoryStream/) using write, rewind and read.

This POC buffers complete documents and currently limits them to 128 UTF-8 bytes,
four nested containers and 32 value occurrences. Containers admit 31 children.
Duplicate names are rejected. A write failure can leave a partial output; cycles
fail the depth limit. Nodes use reference identity, with shared mutable children.

The shape is closest to .NET's mutable JsonNode model, but neoCLR calls the root
`JsonValue` and returns Result errors, including nested stream causes. See the
[JSON API reference](/docs/api/System/Data/Json/) for the current contracts.
The [object-mapping guide](/docs/json/) covers the provisional Object/TypeInfo
overloads for strings and borrowed streams. Broader mapping, generic overloads and
HTTP JSON conveniences for both client and server remain later work.
