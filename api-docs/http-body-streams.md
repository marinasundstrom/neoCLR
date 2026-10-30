---
title: HTTP body streams
---
# HTTP body streams

Development after Preview 14. HTTP can deliver headers before the body is complete,
then read body bytes on demand. Client uploads and server responses can also use a
known-length `HttpStreamContent.Create` source.

## Receive a client response

Pass `HttpCompletionOption.ResponseHeadersRead` to `HttpClient.Send(request,
completionOption, cancellationToken = default)`. The default
`ResponseContentRead` and existing Get/Send overloads still buffer the whole body.

Use `response.Content.TryGetStream()` to obtain `Some(InputStream)` for incoming
stream content. For HttpMemoryContent, its Memory property provides the retained readonly window.
Use CopyTo for a generic body rather than assuming buffered storage. Neither
TryGetStream nor Memory implicitly reads or materializes data.
Dispose the response when finished, including when abandoning an unread body.

`HttpClient.Timeout` and the Send token end at the selected completion point.
Pass a token to each body Read to control later consumption. The existing transport
ceiling of five seconds per pending read remains. Client disposal after a successful
headers handoff does not own the transferred response's connection.

## Receive a server request

`HttpServer.Serve`, `ServeOne`, `Accept` and `AcceptHeaders` all dispatch after
valid request headers. Read nonempty `Request.Content` asynchronously during the
exchange lifetime. The canonical Serve context callback configures the response;
the server sends and cleans up after callback success. Only manual Accept callers
call Complete or dispose the context themselves. Empty bodies need no stream.
`await content.ReadFromJson<T>()` and `await content.ReadJsonNode()` accept both
streaming and buffered bodies; stream reads consume content once.

The acceptance token ends at handoff; body reads take their own token. The server's
original request read deadline and body-size limit continue to apply. Complete or
Close ends body access without draining it. Explicitly closing the body stream
aborts the connection. Do not close it before sending a response you still need.

## Read and frame bodies

One consumer may read at a time. A read borrows the requested destination range
until its task settles; short reads are valid. Zero-count reads consume nothing.
A positive-count read returns zero only at the framing boundary. Content-Length,
chunked bodies, and close-delimited client responses share the same reader.
Request bodies cannot use EOF-delimited framing.

The receiver retains one bounded transport block, including any body bytes that
arrived with the headers. Development after Preview 16: server acceptance starts
with 256 bytes and grows to 4 KiB when a partial request requires another read;
the body stream keeps that block. Client reception retains its 256-byte block. Further reads happen only on consumer demand. Chunk
metadata is bounded separately; chunk extensions and trailers remain unsupported.
The decoded limit is enforced cumulatively, not per chunk. There is no implicit
whole-body string decoding or buffer allocation.

| Outcome | InputStream result |
| --- | --- |
| Invalid destination range | InvalidRange |
| Concurrent read | WrongAccess |
| Malformed or truncated framing | InvalidData |
| Decoded body or chunk-framing limit | LimitExceeded |
| Transport error or request deadline | IoFailure |
| Read after close | Closed |
| Acknowledged token cancellation | Cancelled task |

Pre-cancelled reads do not consume or close the stream. Cancellation during a
pending read ends the body/connection. A pending read releases its borrowed destination only after transport acknowledgement,
including when Dispose/Close interrupts it. Close itself does not await completion.

`HttpContent.Length` is the declared size, not the unread remainder. It is **-1** for
incoming chunked or close-delimited bodies. HttpStringContent.Read accepts buffered
or streaming content; the current JSON content helpers still require buffered content.
Unknown-length content cannot yet be sent as an upload or server response.

These are HTTP/1.1 development APIs over the shared cleartext/TLS connection layer.
Fresh streaming qualification covers local macOS interpreter, JIT and AOT;
Windows and streaming-over-TLS qualification remain open.

## Copy a body to an output stream (development)

`HttpContent.CopyTo(destination: OutputStream, cancellationToken: CancellationToken = default)` returns
`Task<Result<unit, HttpError>>`. It accepts buffered content and incoming or
known-length source streams. The loopback consumer in
[the copy experiment](https://github.com/marinasundstrom/neoCLR/tree/main/docs/experiments/http-content-copy)
copies headers-first responses into a MemoryStream.

Each read is followed by an awaited complete-range destination write before the
next read. Memory-backed bodies and streams use at most 1024 bytes of staging. The current
OutputStream array/range contract prevents forwarding a persistent window directly;
local synchronous helpers borrow it while filling each block. Eager text content
can still forward its encoded array directly. CopyTo does not flush or close either stream, perform
text conversion, add HTTP framing or allocate a whole-body snapshot. The caller
must keep borrowed storage unchanged until completion. Existing OutputStream
array/range signatures remain in use pending coordinated Memory migration.

Known-length streams stop at the declared length without reading beyond it; early
EOF or impossible read counts yield `HttpError.Content(StreamError.IoFailure)`.
Unknown incoming lengths stop at the framing stream's EOF. Underlying read/write
errors retain their StreamError inside HttpError.Content. A failure may follow
partial output. Cancellation propagates as a cancelled task; a pre-cancelled call
performs no I/O and does not consume the source, even for an empty body.

A started streaming copy consumes the one-shot content, including after failure or
cancellation. Further copies/uploads return InvalidRequest. Buffered bodies remain
replayable while undisposed. Disposed content returns Content(Closed); disposal
during a pending write does not revoke the destination's borrowed bytes and stops
the next read after that write settles. Destination ownership never transfers.
Do not mix direct TryGetStream reads with copying/uploading, or run them concurrently;
the base cannot account for reads performed through an independently held stream.

This establishes the transfer operation, not yet the proposed abstract hierarchy,
custom producers, unknown-length outgoing framing or incremental JSON serialization.

## Concrete content types (breaking development redesign)

HttpContent is abstract. Its public byte constructors, FromText, FromStream, Bytes,
TryGetBytes and ReadText are removed; no compatibility aliases remain.

| Type/member | Contract |
| --- | --- |
| `HttpMemoryContent(memory: ReadOnlyMemory<byte>)` | Retains selected bounds and owner without copying; arrays convert implicitly. |
| `HttpMemoryContent(memory: ReadOnlyMemory<byte>, contentType: string)` | Same storage with an outbound media type. |
| `HttpMemoryContent.Memory: ReadOnlyMemory<byte>` | Persistent readonly window; writes through another alias remain visible. |
| `HttpStreamContent.Create(stream: InputStream, length: int, leaveOpen: bool) -> Result<HttpStreamContent, HttpError>` | Validates known length before reading or taking ownership. Negative length returns InvalidRequest; excessive length returns LimitExceeded. |
| `HttpStringContent(text: string)` | Encodes owned UTF-8 bytes eagerly and selects text/plain; charset=utf-8. |
| `HttpStringContent(text: string, contentType: string)` | Eager UTF-8 with an explicit media type. |
| `HttpStringContent.Read(content: HttpContent) -> Task<Result<string, HttpError>>` | Reads any content as owned strict UTF-8 text, without closing it or checking Content-Type. |

Read returns Protocol for malformed UTF-8, Content preserving stream I/O failures,
InvalidRequest for an already consumed streaming body and Content(Closed) for
disposed content. A stream is consumed once, including decoding failure. Current
stream text materialization uses StreamReader's incremental UTF-8 decoder with a
256-byte read buffer and a 65536-byte text quota; quota failure is Content(LimitExceeded).
Known lengths stop without reading the next byte, and early EOF is Content(IoFailure). Larger already-buffered content can use the
existing segmented decoder. This asymmetry is an implementation gap, not a permanent
text-model rule. Reader cancellation overloads remain pending. The decoder still
creates text fragments and concatenates them; this is not allocation-free reading. Creating text content does not retain the original String after encoding.

Memory content stores its window directly without a Sequence wrapper or byte copy.
Wire code and JSON reads borrow that selected window directly. Both HttpResponse
constructors take HttpContent and retain that exact producer without reading or
copying its body. Memory aliases remain visible, and streams remain unconsumed.
Content metadata replaces all Content-Type headers without mutating the caller's
header list; with no content media type, the supplied headers are retained.
Keep borrowed memory stable until sending completes. Request decoding creates
memory content from its buffered bytes. Sequence body constructors are removed;
choose an explicit memory, string or stream producer.

The three ReadOnlyMemory-bearing signatures above are rendered from native metadata.
The legacy CLI reference exposes the HttpMemoryContent type but omits its two
constructors and Memory property because it cannot safely project opaque views.
This manual entry supplies their exact signatures; no replacement array overload
is introduced. HttpJsonContent and a general custom-producer extension point remain
pending, as do optional wide lengths and unknown-length outgoing framing.

## Message ownership and version (development, 2026-10-19)

`HttpRequest` and `HttpResponse` implement `Disposable`. Disposal releases their
content without reading or draining it and is safe to repeat. Client callers own
requests and returned responses; sending does not dispose a caller-owned request.
`HttpContext` owns incoming requests and outgoing responses. Serve manages the
context; explicit Accept callers must close/dispose it. A stream content producer
with `leaveOpen: true` retains external stream ownership. Header-derived request
copies share content, so disposing an alias affects the others. A disposed request
is rejected before dispatch to a custom handler.

Both messages expose `val Version: HttpVersion`. The value has `Major`, `Minor`,
`Http11`, `Equals` and explicit `ToString`; reading it allocates no version object
or string. Current parsers and writers accept/emit HTTP/1.1 only, so the property
is read-only and equals `HttpVersion.Http11`. It is available with incoming
headers, before any body read. Representing another version does not enable its
transport or negotiation.

The ownership and version concepts follow [.NET HttpRequestMessage](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httprequestmessage?view=net-10.0)
and [HttpResponseMessage](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpresponsemessage?view=net-10.0).
neoCLR deliberately uses a compact protocol value instead of a `System.Version`
reference and does not expose a version setter while only HTTP/1.1 is supported.
The cost is a distinct value API and no version-selection policy yet; it avoids
per-message version allocations without claiming support for other protocols.

`HttpResponse` exposes `StatusCode`, `IsSuccessStatusCode`, `Version` and
`ReasonPhrase`. Incoming HTTP/1.1 reason text is preserved exactly within the
current ASCII-header contract, including tabs, trailing spaces and an empty
phrase. Informational phrases do not replace final-response metadata. Custom
phrases are decoded and cached only on first access, without reading the body;
until then they retain the bounded header buffer. Empty phrases and `OK` use
constant text. Metadata remains readable after disposal. Locally constructed
responses use `OK` for 200 and an empty phrase otherwise, matching the current
writer; `Respond` resets the phrase with the new status. The phrase is descriptive
only: `IsSuccessStatusCode` checks the numeric 200–299 range.
