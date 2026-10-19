---
title: HTTP body streams
---
# HTTP body streams

Development after Preview 14. HTTP can deliver headers before the body is complete,
then read body bytes on demand. Client uploads and server responses can also use a
known-length `HttpContent.FromStream` source.

## Receive a client response

Pass `HttpCompletionOption.ResponseHeadersRead` to `HttpClient.Send(request,
completionOption)` or its overload with a cancellation token. The default
`ResponseContentRead` and existing Get/Send overloads still buffer the whole body.

Use `response.Content.TryGetStream()` to obtain `Some(InputStream)` for incoming
stream content. Custom handlers may return buffered content; `TryGetBytes()`
provides that alternative. Neither method implicitly reads or materializes data.
Dispose the response when finished, including when abandoning an unread body.

`HttpClient.Timeout` and the Send token end at the selected completion point.
Pass a token to each body Read to control later consumption. The existing transport
ceiling of five seconds per pending read remains. Client disposal after a successful
headers handoff does not own the transferred response's connection.

## Receive a server request

`HttpServer.AcceptHeaders()` and `AcceptHeaders(cancellationToken)` return a context
after valid request headers. Read `context.Request.Content.TryGetStream()` during
the context lifetime, then respond and Complete, or dispose the context.
`Accept` and `ServeOne` retain their existing buffered request behavior.

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
incoming chunked or close-delimited bodies. `Bytes` remains buffered-only;
`ReadText` and the current JSON content helpers still require buffered content.
Unknown-length content cannot yet be sent as an upload or server response.

These are HTTP/1.1 development APIs over the shared cleartext/TLS connection layer.
Fresh streaming qualification covers local macOS interpreter, JIT and AOT;
Windows and streaming-over-TLS qualification remain open.

## Copy a body to an output stream (development)

`HttpContent.CopyTo(destination: OutputStream)` and
`CopyTo(destination: OutputStream, cancellationToken: CancellationToken)` return
`Task<Result<unit, HttpError>>`. They accept buffered content and incoming or
known-length source streams. The loopback consumer in
[the copy experiment](https://github.com/marinasundstrom/neoCLR/tree/main/docs/experiments/http-content-copy)
copies headers-first responses into a MemoryStream.

Each read is followed by an awaited complete-range destination write before the
next read. Byte arrays are written directly; other sequences and streams use at
most 1024 bytes of staging. CopyTo does not flush or close either stream, perform
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
