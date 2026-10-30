---
title: Web and HTTP
---
# Web and HTTP

For application composition, see [experimental Neo.Web hosting](/features/neo-web/)
and its [reference app](/cases/neo-web/). The HTTP APIs below remain independently usable.

Raven client and server applications exchange UTF-8 content through neoCLR TCP sockets,
with DNS on the client. HttpClient, HttpServer, request/response types, content and
handlers live in System.Web.Http. Use async functions and `await` for application
control flow; handlers and completion adapters can still use callbacks.

Use a matching development toolchain. [Native compilation](../native-compilation/)
explains supported hosts and execution modes.

## HttpServer: serve concurrent requests

Use `HttpServer.Serve(handler)` as the main API for HTTP web applications. After
listening, pass a context callback that configures the response:

```raven
{{HTTP_SERVE_CONTEXT_SAMPLE}}
```

The callback runs after request headers, before the body is read in full. Read
`context.Request.Content` asynchronously only when needed; unread bodies are
abandoned when the exchange closes.

Its signature is `(HttpContext) -> Task<Result<unit, HttpError>>`. Serve sends the
configured response after the handler succeeds, and owns completion and cleanup.
The handler must not call Complete or retain the context after returning.
The response-returning variant,
`(HttpRequest) -> Task<Result<HttpResponse, HttpError>>`, remains available.
The server observes the original task in both variants; the context callback adds
no per-request Promise mapping wrapper. Development after Preview 16
admits up to 16 exchanges; suspended async handlers allow other requests to advance.

Awaiting the task returned by `Serve` in async Main waits for **server lifetime**;
it does not serialize requests. Alternatively, retain that task while coordinating
shutdown, call `Close`, then await it and handle its Result. Shutdown closes
connections and joins started handlers; handlers must settle cooperatively.

Concurrency does not imply a thread per request. Blocking synchronous handler code
blocks the cooperative dispatcher, and omitting `await` does not offload work to a
thread. `Accept`/`AcceptHeaders` and `ServeOne` remain lower-level options: callers
must manage bounded concurrency and cleanup if they use them to build a server.
Awaiting context acquisition is fine; awaiting all processing before accepting the
next context makes a loop sequential.

[Download the context callback sample](/samples/http-context-serve.zip).
See the [server lifetime reference](/docs/http-server.html) and
[server application walkthrough](/cases/http-server/) for ownership and examples.

## HttpClient: call an HTTP service

Use HttpClient on its own to call a service. This example sets a base address and
an Accept header, then fetches a UTF-8 greeting with cancellation support:

```raven
{{HTTP_CLIENT_OVERVIEW_SAMPLE}}
```

Calling `GreetingClient().Read("http://localhost:8080/", token)` requests
`http://localhost:8080/greeting`. It returns a Task containing either the response
text or a structured HttpError. `GetString` rejects non-success statuses and
invalid UTF-8. Acknowledged cancellation cancels the task instead of returning an
HTTP error. Use `CancellationToken.None` when no cancellation is needed.

An async entry point awaits the operation and handles its Result:

```raven
{{HTTP_CLIENT_MAIN_SAMPLE}}
```

### What the client offers

| Need | API and behavior |
| --- | --- |
| Send a request | Get, Head, Post, Put, Patch and Delete; Send accepts a prepared HttpRequest |
| Inspect a response | Get returns HttpResponse with status, headers, content and its associated request; non-2xx responses remain available to inspect |
| Read text | GetString checks the status and decodes a buffered UTF-8 body |
| Exchange JSON | `GetFromJson<T>` reads a successful JSON response; `PostAsJson` serializes a model and returns the response for explicit status/body handling |
| Set addresses and headers | String or Uri addresses, optional BaseUri, DefaultRequestHeaders and per-request WithHeader |
| Cancel work | Token overloads cover lookup, connection and transfer |
| Customize sending | HttpHandler supports forwarding behavior and replacement transports for tests |
| Upload content | Buffered bytes/text or a known-length InputStream source with explicit ownership |
| Stream incoming content | Development: Send with ResponseHeadersRead; read Content.TryGetStream and dispose the response |

Import `System.Web.Http.Json.*` for JSON helpers. These are bounded experimental
HTTP/1.1 APIs. Development HTTPS uses explicit trust/credentials and managed SslStream; connection pooling and HTTP/2/3 remain unsupported. See the [TLS reference](/docs/ssl-streams.html) and limits below.

[Download the general client example and verifier](/samples/http-client.zip).
`Overview.rvn` contains the full program. With a matching runtime bundle and SDK,
run the verifier from the extracted directory:

```sh
python3 verify.py --overview --case "fragmented UTF-8" \
  --toolchain-root /absolute/path/to/runtime-with-raven-sdk \
  --runner /absolute/path/to/runtime-with-raven-sdk/tools/http-runner
```

It starts an independent local service, verifies the Accept header and runs the
client against a fragmented response. The program prints `HTTP 200` and `Café 🌍`.

## Case: Building a Http server app

A report service accepts a station report and returns `{"accepted":true}`. Its
server reads the model and constructs an acknowledgement:

```raven
{{HTTP_NESTED_READ_SAMPLE}}
```

The client fetches a report from the service, submits it, and reads the reply:

```raven
{{HTTP_MAPPED_CLIENT_SAMPLE}}
```

The server's POST handler uses the first operation to respond with 201; malformed
reports receive 400. `ReadReply` checks the response status and reads its JSON.
The full case includes the nested station model.

[Case: Building a Http server app →](/cases/http-server/)
The walkthrough includes the shared models, routing, client connection setup,
complete projects, run instructions and expected responses.

<a id="streaming-bodies--development"></a>

## Streaming bodies

`Send(request, HttpCompletionOption.ResponseHeadersRead)` returns after final
response headers. `HttpServer.AcceptHeaders()` does the same for incoming requests.
Use `Content.TryGetStream()` to consume their bytes on demand. The shared reader
retains a bounded transport block, including read-ahead from header parsing.
Development after Preview 16 reduces server response copying and uses larger
bounded transfers; buffered responses still preserve their snapshot semantics.

Both client uploads and server responses accept known-length `HttpStreamContent.Create`
sources. Default client Get/Send buffer incoming bodies; server Serve/Accept/ServeOne dispatch after headers, so the JSON showcase
continues to use its existing content helpers. Incoming streaming works with fixed
and chunked framing, plus close-delimited client responses; unknown-length outgoing
bodies are still unsupported.

[Body ownership, cancellation and limits](/docs/http-body-streams.html). This is
post-Preview-14 development with local macOS interpreter/JIT/AOT qualification;
fresh Windows and TLS streaming checks remain open.

## Customize requests

Use `HttpRequest`, headers and content when a convenience method is not enough.
`Send` returns the response even for non-success status codes; `GetString` and
`GetFromJson<T>` check success before decoding. Method values allow custom names.

Derive from `DelegatingHandler` to observe or modify an exchange, then await
`base.Send(request, cancellationToken)`. The handler pipeline can also replace
transport for tests. It owns its inner handlers and freezes after sending begins.

[HTTP API and handler contracts](/docs/api/System/Web/Http/index.html) ·
[HttpClient](xref:System.Web.Http.HttpClient)

<a id="own-an-http-exchange"></a>

## Own the exchange

Dispose clients when finished. Default calls buffer responses; headers-first calls
transfer a body stream that the response owns. Dispose streaming responses as well. On the server,
`Serve` owns response and connection cleanup. When using `Accept` directly,
complete or dispose each request context so its connection is closed. Use async `InputStream`/`OutputStream`
operations for transfer and explicit encoding when converting bytes to text.

`HttpClient.Timeout` sets one cooperative budget across the handler pipeline,
lookup, connection and response completion. It defaults to 100 seconds. Expiry
requests cancellation and waits for cleanup; handlers that ignore cancellation
cannot be forcibly interrupted. Transport phases have their own bounds.

## HTTPS and current limits

Development HTTPS uses explicit certificate credentials and trust through managed
`SslStream`. Supported HTTPS/JSON workloads include certificate rejection,
deadlines and cancellation on macOS ARM64 and Windows x64. Backend coverage
depends on the matching build kit. See the
[TLS guide](/docs/ssl-streams.html) for setup and
[release availability](/try/#development) for matching tools.
SQL transport has separate limitations; HTTP TLS support does not imply SQL TLS.

HTTP/1.1 responses support fixed-length, chunked and connection-close framing.
The server buffers a request and closes its connection after the exchange. Connection
pooling, redirects, HTTP/2/3 and WebSocket upgrades are not implemented.

Development incoming requests now validate Host and Target directly, retaining
string properties without constructing an intermediate URI. The existing input
validation and size limits remain in force; client URL/Uri overloads are unchanged.
A fresh local Serve comparison records 7.1% fewer allocations and 6.4% lower CPU
per completion in its final 60-second pair, but worse tail latency and missed
arrivals prevent a sustained throughput claim. See the
[complete experiment and Go controls](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/http-server/serve/incoming-url-20261018/README.md).

A subsequent development change replaces blind async idle sleeps with bounded
socket-readiness waits. After removing a busy background process, all eight
60-second control/candidate/Go runs complete every arrival at 300 requests/sec.
Readiness reduces median latency by 25–29% and p99 by 12–19% versus URI removal
alone; CPU changes are mixed. Unchanged controls also pass, so the admission
recovery cannot be attributed solely to the fix. Go still uses substantially less
CPU per request; maximum capacity remains unmeasured. See the
[readiness investigation and full evidence](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/http-server/serve/admission-followup-20261018/README.md).

Bodies, headers, URLs and JSON documents have separate size limits. Supported
admission bounds are not a throughput guarantee; use the [HTTP API reference](/docs/api/System/Web/Http/index.html)
for exact limits and error behavior. Choose a matching development toolchain for
the examples rather than assuming every published package has these APIs.

<a id="json-dom"></a>

JSON can be represented as typed models or explicit JsonValue nodes. Development
serialization also supports public nongeneric record structs and nested class/record
properties, with matching interpreter, JIT and AOT behavior. Record deserialization
remains unsupported. See the
[JSON guide](/docs/json.html) for nested values, collection mapping and limits.

## Read further

- [Build the HTTP server case](/cases/http-server/) — models, routes and client calls.
- [SQL databases](/features/sql/) — query and map application data.
- [TCP networking](/features/networking/) — lower-level socket operations.
- [HTTP benchmarks](/benchmarks/http/) — workload matrix and measurement status.


## Memory-view integration in development

HTTP request parsing now borrows readonly byte slices, and pending body reads retain
managed memory windows. Fixed-length and chunked server bodies and streaming client
responses are checked in interpreter, JIT and AOT modes, including collection,
cancellation and destination boundaries. Public stream signatures still use array
ranges until the coordinated stream API migration. Socket transfers already use
Memory/ReadOnlyMemory; no AsSlice adapter is required.


The current memory-view cohort has a local two-collector HTTP comparison: the
experimental concurrent collector improves 8 KiB overload throughput and tail
latency at 100 requests/sec, while tiny-response overload throughput is lower.
The [matched workload results](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/http-server/serve/memory-gc-20261019/README.md)
retain misses, CPU/memory diagnostics and all trials. These plain-HTTP AOT results
do not establish HTTPS or production maximum capacity.

The development [Promise storage follow-up](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/http-server/serve/promise-result-20261019/README.md)
removes one allocation per Promise while retaining array-backed result storage.
Matched local tests reduce CPU per completion, with variable bulk tail latency;
this does not establish higher server capacity. A runtime completion bridge to
avoid Promise wrappers in HTTP operations is under investigation.

In development, JSON HTTP content and output streams serialize directly into bounded
UTF-8 byte storage. `SerializeToUtf8Bytes` exposes an owned byte result; text-returning
APIs still return text. InputStream deserialization remains available and currently
materializes text before parsing. The parser now borrows that text’s UTF-8 storage,
avoiding repeated encoding and temporary quoted-token strings. Buffered HTTP JSON
now parses byte windows directly, validating UTF-8 without creating a document string.
Non-array sequences still require one contiguous snapshot. See [JSON contracts](/docs/json.html).

Development `HttpContent.CopyTo` transfers buffered or streaming bodies into an
`OutputStream`, awaiting destination writes before reading further. It uses direct
array writes or bounded staging, propagates cancellation and stream errors, and
leaves resource disposal explicit. See the [body-stream contract](/docs/http-body-streams.html).
The development content API now has an abstract HttpContent base, memory-backed
HttpMemoryContent, one-shot HttpStreamContent and UTF-8 HttpStringContent. Arrays
convert to readonly memory; selected windows retain their owner without copying.
Use HttpStringContent.Read for generic text consumption; streamed reads decode
incrementally without first collecting the entire body into a byte array. Base storage getters and
construction helpers have been removed rather than retained as compatibility layers.
The [API guide](/docs/http-body-streams.html) records migration and current streaming limits.

### Message version and ownership (development)

Requests and responses expose `Version.Major` and `Version.Minor` without creating
version strings. The current transport supports HTTP/1.1 only. Both messages are
disposable: client callers own their messages, while the server context owns its
request and response. Disposal does not read an unread body; borrowed streams
honor `leaveOpen`. See [content ownership](/docs/http-body-streams.html).

Client responses expose `StatusCode`, `IsSuccessStatusCode`, `Version` and
`ReasonPhrase`. The reason text is descriptive; success uses the numeric status.
Custom HTTP/1.1 phrases are decoded only when read, without consuming body data.

Development `HttpVersion` now implements `EquatableTo<HttpVersion>` and matching
Object equality/hash overrides. Typed and boxed values compare both protocol
components, and ToString consistently formats major.minor. This also applies to
versions obtained from requests and responses; supported transport protocols remain
unchanged. Hashes are execution-local and do not indicate protocol support.

## Media types (development)

HTTP content accepts a structured `MediaType`, with predefined values such as
`MediaTypes.Application.Json` and `MediaTypes.Text.Plain`. Custom types and quoted
parameters can be parsed with `MediaType.Parse`; `WithParameter` returns a modified
value without mutating the original. Presets, formatted strings and hashes are
cached, and parameter views do not copy their contents on access.

`HttpRequest.GetContentType()` and `HttpResponse.GetContentType()` distinguish absent
headers from malformed or duplicate Content-Type fields. Typed content retains its
media type; string content still encodes UTF-8 and rejects a conflicting charset.
Media types describe content; they do not select serializers or add multipart or
Accept negotiation support. Outbound legacy string metadata now receives full media
type syntax validation before sending. The parser allows up to 32 KiB and 100
parameters; existing transport ASCII-header limits still apply.

The executable fixtures in
[media type validation](https://github.com/marinasundstrom/neoCLR/tree/main/docs/experiments/media-types)
exercise typed content against independent HTTP peers. Browse the
[HTTP API reference](/docs/api/System/Web/Http/index.html) for type/member contracts.
