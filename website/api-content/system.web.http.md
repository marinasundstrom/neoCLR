---
uid: N:System.Web.Http
---
## HTTP requests, responses and handlers

HttpClient and HttpServer provide the bounded HTTP proof of concept. Requests,
responses, headers and content describe exchanged messages; HttpContext and handler
contracts connect server behavior and the replaceable client pipeline. RoutePattern
and RouteMatch support the implemented routing surface.

See [Web](/features/web/) for the current cleartext scope, limits and tested samples.
This preview is not a general production web stack. System.Web.Http.Json supplies
JSON content and client helpers.

HttpClient.Timeout is an `Optional<Duration>`, defaulting to Some(100 seconds).
None disables the client countdown. Configure it before sending. Expiry requests
cancellation and waits for handler cleanup before returning HttpError.TimedOut; caller
or disposal cancellation remains task cancellation. Independent transport phase bounds
still apply. See [HTTP contracts](/features/web/) for limits and handler responsibilities.

HttpMethod provides canonical method properties, custom token construction and typed
Parse. HttpClientRequest.Method returns HttpMethod; Create accepts it with URL/Uri and optional
content. Exact spelling is available through Method.Method; value equality ignores
ASCII case. CONNECT is representable but the socket transport does not implement tunnelling.

HTTP/1.1 admits 1 MiB buffered bodies and known-length uploads, 32 KiB
headers and 100 fields, subject to independent host allocation budgets. Incoming
requests support bounded chunked coding; client responses support up to 16 interim
replies before the final response. Chunk extensions/trailers and status-101 upgrades
remain unsupported. JSON has a separate 1 KiB bound. See the Web guide for exact
ownership, limits and deferred hosting capabilities.

The 1 MiB admission ceiling is not a qualified transfer guarantee: the native
stress-GC host currently exceeds its network deadline at that size. See the
HTTP foundation evidence in `docs/http-foundation.md`
for host budgets and validation limitations.

`HttpServerOptions` configures smaller request body/header limits and read/write
budgets. Pass it to the four-argument `HttpServer.Listen`; validation occurs before
binding and later option mutations do not affect the server. Existing defaults
remain 1 MiB bodies, 32 KiB/100 headers and 15-second phase deadlines. Positive
timeouts through 60 seconds are supported; no infinite deadline is admitted.

Development `HttpServer.Serve(handler)` owns bounded concurrent callback dispatch.
Its task represents the server lifetime; applications do not await each request in
an accept loop. See [server-managed callbacks](/docs/http-server.html#server-managed-callbacks)
for outcome diagnostics, ownership and joined shutdown.

## Media types (development)

`MediaType.Parse` provides immutable type/subtype, suffix and parameter structure.
Use typed presets such as `MediaTypes.Application.Json` and `MediaTypes.Text.Plain`
with the typed HTTP content constructors. Presets and formatted strings are cached;
custom inputs are not globally interned. `WithParameter` returns a changed value,
and equality preserves case-sensitive boundary/unknown parameter values.

`GetContentType()` on client and server request/response types return
`Result<Optional<MediaType>, HttpError>`: missing and malformed headers remain distinct.
`HttpContent.GetContentType()` reads producer metadata. None of these reads consumes
a body. Content-Type is a concrete media type, not an Accept range or quality weight.
See the [media type contract](/features/web/#media-types-development) for limits.

Development `HttpServerResponse.SetHeader(name: string, value: string) -> Result<unit, HttpError>`
replaces matching fields without replacing the response or its content. Names are
HTTP tokens; prohibited controls return InvalidRequest without mutation. Response
framing/content validation still happens on send. Neo.Web uses this to preserve
response identity while configuring a 405 Allow header.

### Header domain views (development)

Requests and responses expose `HttpHeaderCollection`. Use
`headers[HttpHeaderNames.ContentType]` for all raw values, `Contains(name)` for
presence, and `GetSingle(name)` or `GetContentType()` for checked access.
Missing names return an empty value sequence; repeated values remain separate.
The existing canonical `HttpHeaderNames` catalog is now public.

Parsed fields retain compact name/value storage. Named access avoids constructing
`HttpHeader` entries; positional access and iteration project entries on demand.
The view is read-only, and existing message mutation/ownership behavior remains.
See the [header collection contract](/docs/http-headers.html) for exact errors,
snapshot behavior and allocation tradeoffs.

See [HTTP message roles](/docs/http-messages.html) for the independent client/server
contracts, mutation rules and migration from the provisional shared message types.
