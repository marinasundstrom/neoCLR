# HTTP/1.1 foundation and JSON content ergonomics

Author-directed development work, 2026-10-11, on `codex/http-foundation`.
The long-term target is modern web application hosting and WebSockets. This
increment extends the completed bounded POC; it does not qualify a production host.

## Implemented contract

- Both directions buffer at most 1,048,576 body bytes. Known-length client stream
  uploads use the same whole-message bound; source reads remain synchronous and
  one-shot. Server responses remain fixed-length buffered messages.
- Request/response headers admit 32,768 encoded bytes and 100 fields. Client
  application headers reserve three fields for transport; server response headers
  reserve two. The client counts informational and final header bytes together.
  The existing 1,024-byte URL policy and ASCII header grammar remain unchanged.
- The server accepts `Transfer-Encoding: chunked`, case-insensitively, including
  empty bodies and fragmented size/data/delimiter input. Request and response
  parsers share one decoder with a 65,536-byte chunk-framing budget. Conflicting
  Content-Length/Transfer-Encoding, duplicate framing fields, invalid CRLF,
  truncation and numeric overflow fail. Chunk extensions and nonempty trailers
  remain explicitly unsupported. GET/HEAD/DELETE still require empty bodies.
- The client consumes up to 16 informational responses before the final response,
  discarding interim fields. It rejects framing headers on interim responses and
  rejects 101 explicitly: protocol upgrades do not yet transfer socket ownership.
- Request target whitespace, controls and fragments are rejected before dispatch.
  A failed request decoder cannot subsequently produce a successful request.
- Existing cancellation, disposal, deadlines, connection-close behavior and
  malformed-request closure remain. Rejection does not generate an HTTP error page.

These are HTTP budgets, not reserved memory or relaxed host resource limits. The
validation harness configures 16,777,216 interpreter array elements, 32 MiB array
bytes, a 4 MiB native per-byte-array ceiling and a 32 MiB native HTTP heap.
The HTTP build profile exposes `--http-heap-bytes`; its default remains 1 MiB. Growable buffers and encoding
snapshots can allocate more than the message's logical length. Hosts must select
sufficient independent allocation/instruction budgets; default constrained-host
budgets do not promise 1 MiB transfers. No performance or load claim is made. The stress-GC native host collects at every
safepoint: a 1 MiB wire test exceeded the existing network deadline. Therefore
1 MiB is an admission ceiling, not a qualified transfer-size guarantee under this
host. Smaller executable transfers and rejection at the admission ceiling are
validated separately. The runtime thread owns collection-policy follow-up.

## Per-server configuration

[HttpServerOptions](http-server-options.md) now configures smaller request body,
header-byte and field-count limits, plus read/write deadlines. Listen snapshots
validated settings; its original overload preserves defaults. Hard buffer ceilings
and native stress-GC limitations remain unchanged.

## Content-oriented JSON APIs

Import `System.Web.Http.Json.*` and use:

```raven
let result = await request.Content.ReadFromJson<Message>()
let configured = context.RespondJson(message)
```

[Compiled consumers](experiments/http-foundation/) cover the actual API spellings.
`ReadFromJson<T>` and `ReadFromJson(TypeInfo)` return `Task<Result<..., JsonError>>`.
`ReadJsonNode` returns `Task<Result<JsonValue, JsonError>>`. Each has a token overload.
They work on request or response content, keep bytes available for repeated reads,
and preserve serializer errors. Pre-cancellation skips conversion. Conversion is
synchronous over an already buffered body, so it cannot be interrupted mid-parse.
A stream-backed upload returns `Read(IoFailure)` without reading or closing its
source. Explicit JSON reads do not impose status or media-type policy.

`RespondJson(value)` configures status 200; `RespondJson(statusCode, value)` uses
that status. Both return `Result<unit, JsonError>` and set
`application/json; charset=utf-8`. They serialize before changing the response,
so a failure preserves its status/content. Neither sends; call `Complete`, whose
existing status/body validation remains in force. `JsonContent.Create` continues
to support explicit request construction and model/DOM serialization.

The serializer's 1,024-byte document bound, depth/value limits and supported type
mapping remain separate gaps. The larger transport limit does not increase JSON
capacity. The parallel JSON workstream confirmed that its options/converter APIs are not yet
qualified for HTTP integration. Options overloads remain a coordinated follow-up;
content negotiation, lazy JSON streams and asynchronous body readers are also open. These helpers add no compiler or Runtime Contract mode.
Their execution is qualified through native System.Web metadata, not a new legacy
CLI bridge-emitter implementation; the aggregate reference carries their signatures.

## Research and alternatives

Primary sources reviewed 2026-10-11:

- [RFC 9112 sections 6–8](https://www.rfc-editor.org/rfc/rfc9112.html#section-6)
  define HTTP/1.1 framing, chunk decoding and incomplete-message handling. Shared
  framing logic reduces disagreement between the two parsers. We deliberately
  reject CL+TE and duplicates rather than normalize ambiguous messages. Extension
  and trailer support remains incomplete protocol coverage, not a new standard.
- [.NET HttpClient buffering](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.maxresponsecontentbuffersize?view=net-10.0)
  and [Kestrel limits](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/servers/kestrel/options?view=aspnetcore-10.0)
  separate client buffering and configurable server admission. neoCLR's private
  shared fixed limits are a bounded implementation step, not equivalent .NET
  configurability. Keeping 1 KiB blocks ordinary binary/text payloads; simply
  removing limits would sacrifice predictable admission. Live body streams and
  per-endpoint limits are the preferred next evolution, with backpressure and
  ownership tests before widening the host scope.
- [.NET content JSON reads](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.json.httpcontentjsonextensions.readfromjsonasync?view=net-10.0)
  place typed reads on HttpContent and support cancellation; [JsonContent.Create](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.json.jsoncontent.create?view=net-10.0)
  supplies serialization content. The new neoCLR extensions follow that placement
  and its existing omission of the Async suffix. Result errors, strict UTF-8 and
  eager bounded serialization are existing deliberate adaptations. Returning a
  Task fits existing content reads and leaves room for a future body reader, at
  the cost of a completed-task allocation today. RespondJson removes repetitive
  serialization/media-type setup while preserving explicit send lifetime.

## Remaining hosting work

Sequence future work by dependency: configurable admission and JSON budgets;
live body streams with cancellation/backpressure; HTTP connection ownership and
persistent reuse; TLS/provider validation; explicit upgrade ownership followed by
RFC 6455 frame/masking/control/close semantics. HTTP/2/3, redirects, compression,
trailers/extensions, Expect: 100-continue, graceful draining and hosting middleware
remain open. WebSockets cannot be implemented by merely admitting status 101.
No one-off HTTP direction permanently displaces runtime-async priorities.

## Native metadata support

HTTP plus JSON exposed two existing AOT descriptor gaps: closed function signatures
used as generic arguments lacked descriptor mapping, and generated TypeShape dispatch
for 114 retained types exceeded the backend's method-size budget. The binder now
preserves function signature identity/visibility and validates the shape selector
once before emitting true-flag cases with a false default. It does not increase the
backend budget or introduce new reflection admission. Explicit fixture retention
keeps the model constructor/properties available; DOM-only requests use a module
catalog. Missing model retention can raise a native runtime fault before JsonError;
HTTP helpers preserve that existing serializer requirement. The atomic-response
probe uses an oversized DOM string to exercise a recoverable serialization error. These are native binder changes, not Raven compiler/CLI bridge changes.

## Validation

See [reproduction and recorded results](experiments/http-foundation/README.md).

## Byte ownership and lazy materialization direction — 2026-10-14

Author direction: aim for shared byte buffers and fewer temporary strings, making
HTTP work lazy where useful. This is a proposed implementation direction, not a
new public buffer API or a measured speedup. It complements runtime GC/pacing work.

Current `HttpRequestDecoder` already scans bytes, but copies received header bytes
into `ArrayList<byte>`, eagerly decodes each header name/value, and constructs
another byte list to lowercase each name. `HttpByteWindow` is a bounded view, but
the codec owns a snapshot. `HttpRequest.Headers` exposes string-valued `HttpHeader`
objects. Streaming request content already has a context-bound lifetime; header
strings currently have ordinary independent managed lifetimes.

Primary-source review (accessed 2026-10-14):

- [Kestrel HttpParser](https://github.com/dotnet/aspnetcore/blob/main/src/Servers/Kestrel/Core/src/Internal/Http/HttpParser.cs)
  scans byte spans/sequences and passes header slices to its handler. This parser
  boundary does not imply the whole framework is zero-copy.
- [.NET pipelines](https://learn.microsoft.com/en-us/dotnet/standard/io/pipelines)
  make buffer consumption, ownership and backpressure explicit. Pooling reduces
  allocations but readers must respect buffer lifetimes.
- [Go request parsing](https://go.dev/src/net/http/request.go) uses a buffered reader,
  materializes the request line and a MIME header map. Go is useful as an ergonomic
  and performance comparison, not evidence that all successful servers avoid strings.

The [URI byte-parsing refinement](../benchmarks/http-server/uri-byte-parsing.md)
removes the first measured parser hot spot with matched allocation and throughput
evidence. Header ownership and lazy decoding remain future work.

Proposed sequence: first measure byte/string allocations per request by stage, then
remove lowercase-name intermediates using ASCII byte comparison and known-name
identities. Next retain bounded owned header storage with offsets, decode/cache
values only when demanded, and compare that with eagerly owned small strings.
Keep request framing, duplicate framing headers, syntax and configured bounds
validated before dispatch even if the handler never accesses a header. Lazy decoding
must not defer protocol rejection until application access. Do not use grapheme
indexing for wire grammar; body encoding remains a content concern.

Shared means shared ownership of bounded storage within an exchange/connection,
not a mutable global receive buffer. A view surviving `await` must retain its owner;
a buffer cannot return to a pool while any consumer can read it. A retained request
can otherwise pin a whole large buffer, so copy small escaping values or retain a
compact immutable snapshot where that is cheaper. Pool retention itself needs a
cap. Cancellation, shutdown, abandoned operations and fault cleanup must release
ownership; test slow peers alongside fast peers and fragmented input.

Keep streaming bodies pull-driven with bounded read-ahead and preserve bytes already
received after the header terminator. Do not buffer entire bodies to enable views.
Response headers and payloads should be written from bytes without constructing
whole-message strings. Start with private owned byte-array/range helpers; a public
Span/Memory/Pipelines equivalent needs broader contracts and is not a prerequisite.

Validate same-fixture allocations/request, copied bytes, throughput and latency,
peak retained memory and GC cleanup in native JIT/AOT plus an interpreter correctness
control. Preserve metadata/header lifetime semantics or document a deliberate API
change with matching reference documentation. No pooling or lazy-header behavior
is implemented by this design note.

The author clarified that Kestrel recognizes known header bytes and reuses constant
name strings. Its [generated request-header implementation](https://github.com/dotnet/aspnetcore/blob/main/src/Servers/Kestrel/Core/src/Internal/Http/HttpHeaders.Generated.cs)
confirms known-name matching, canonical `HeaderNames` strings and dedicated fields.
It also reuses an eligible previous single ASCII value when the bytes match;
otherwise the value is decoded while appending the header. This is not universal
lazy decoding or global interning of arbitrary network input. neoCLR should start
with fixed known-name identities; bounded value reuse and lazy materialization are
separate choices to measure.

### Known-name implementation — 2026-10-14

[Fixed-name byte matching](../benchmarks/http-server/header-name-bytes.md) now avoids
lowercase buffers and name decoding for seven common request headers. Unknown
names use the existing path; value decoding and all protocol checks stay eager.
This is not permanent interning: the backend may copy literals when text identity
is observable. Shared constant ownership and broader name coverage remain follow-ups.
