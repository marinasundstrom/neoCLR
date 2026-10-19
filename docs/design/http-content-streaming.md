# Streaming HTTP content direction

Author discussion, 2026-10-19. **Proposal, not an implemented API.**
The author first requested HttpContent(bytes), with HttpStringContent and
HttpJsonContent derived types, then asked how the base should allow streaming and
whether HttpContent should be based on Stream. The author then clarified that
both HttpClient and HttpServer have producer and consumer roles, Stream is the
interface, and the string/JSON types should support creation and async reading.
The assistant revised its proposed
concrete byte base to an abstract, stream-capable body contract. The exact public
signatures and transport migration remain open.

## Proposed contract

HttpContent represents a body and its content headers. It need not store a Stream,
but must be able to write incrementally to an OutputStream. A conceptual CopyTo
operation accepts a destination and CancellationToken, returns an awaitable typed
result, and handles partial writes and backpressure. The exact error union must
preserve serialization, cancellation and transport causes. A known byte length is
optional; unknown length must not trigger implicit whole-body buffering.

- HttpMemoryContent retains ReadOnlyMemory<byte>, including its owner and selected
  window, across asynchronous writes. This replaces the proposed concrete byte base.
- HttpStreamContent reads an InputStream incrementally into bounded owned storage.
  Ownership/leaveOpen and one-shot consumption are explicit.
- HttpStringContent retains immutable UTF-8 text and writes its storage without a
  whole-text encoding array. Borrowed slices cannot cross awaits; persistent string
  byte views or bounded staging must be supported before claiming zero-copy writes.
- HttpJsonContent serializes to the destination, eventually through an incremental
  UTF-8 writer. Current JSON stream serialization still stages a bounded complete
  document; merely wrapping it is not incremental serialization.

Buffering, decoding text and deserializing JSON are explicit consuming operations.
Receiving headers must not force body reads. Received bodies expose the transport's
framing-aware InputStream; producer-only content must not silently buffer merely to
pretend that it already has a readable stream. An optional buffered-memory fast path
may avoid an adapter, but cannot be required of every content implementation.

Use a wide optional length, not -1 as a public sentinel. HTTP/1.1 transport selects
Content-Length for known lengths and chunked framing for unknown lengths where
permitted; HTTP/2 would use its own data frames. Content implementations do not
write HTTP framing. Cancellation, disposal and replayability need explicit contracts;
nonseekable streams must not be automatically replayed on redirects/retries.

## Comparison and costs

[.NET HttpContent](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpcontent?view=net-10.0)
is an abstract body/header abstraction whose extension points include serializing
to a stream and trying to compute its length.
[StreamContent](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.streamcontent?view=net-10.0)
is one derived implementation. This proposal follows that separation. neoCLR's
Memory-based buffers, UTF-8 String storage, async naming and typed errors fit its
existing platform contracts; they are not drop-in .NET signatures.

Incremental output reduces full-body storage and permits backpressure, but failures
can happen after headers/body bytes have been sent. Current eager JSON validation
can fail before any output. A progressive JSON contract must document this change;
explicit buffered content remains useful when preflight validation, known length or
replay is required. Lazy JSON also retains its input object until sending completes
and needs a documented mutation policy. Allocation-free completion is a separate
runtime awaitable concern, not guaranteed by an HttpContent hierarchy.

## Current gaps and bounded migration

Current Raven HttpContent supports buffered Sequence<byte> and one-shot known-length
FromStream uploads; incoming streams can have unknown length. It has no general
producer extension point. HTTP upload code branches on IsBuffered, pulls bytes via
ReadUpload, and assumes a known upload length. A new base alone cannot make unknown-
length uploads work. Shared InputStream/OutputStream signatures still need coordinated
Memory migration. The current JSON byte-input improvement is independent and should
be retained while these contracts are developed.

Implementation should first qualify memory and stream content through the same
transport path, with delayed/partial writes, cancellation, exact framing, resource
ownership and retry rejection. Follow with UTF-8 string and JSON producers; measure
bounded peak storage and allocations on large bodies before claiming throughput gains.

## Producer and consumer roles (author clarification)

| Role | HttpClient | HttpServer |
| --- | --- | --- |
| Producer | Writes request content | Writes response content |
| Consumer | Reads response content | Reads request content |

Stream is the common interface in both directions (InputStream and OutputStream
in the current library). Content provides metadata, lifetime and typed conversion
around that interface. Transport owns HTTP framing in either role.

Proposed usage, not yet callable signatures:

```text
HttpStringContent.Create(text)
await HttpStringContent.Read(content, cancellationToken)
HttpJsonContent.Create(value)
await HttpJsonContent.Read<T>(content, cancellationToken)
```

Factories produce outgoing content; readers accept any HttpContent and consume its
readable stream. A received JSON body need not have the HttpJsonContent runtime
type, and a received text body need not be HttpStringContent. Static readers avoid
wrapping an already-owned body merely to change its interpretation. A generic
HttpContent must therefore expose the incoming stream without forcing text, JSON
or byte-array materialization. Whether convenience instance methods are also
useful remains open; no duplicate helper layers are required by this proposal.

Async text reading necessarily creates the requested owned String. JSON reading
creates the requested object and should eventually parse incrementally from bytes.
Both require bounded reads and typed decoding/deserialization errors. Ordinary
read methods consume a one-shot body; explicit buffering can enable repeated reads.
The current eager JSON factory and full-text stream reader are transitional
implementations, not the final streaming semantics proposed here.

## Implemented first transfer slice (2026-10-19)

The existing concrete HttpContent now exposes CopyTo(OutputStream) and its
CancellationToken overload, returning Task<Result<unit, HttpError>>. Buffered
arrays write directly; general sequences and stream sources use bounded staging.
Each destination write settles before another source read. Cancellation and
stream errors propagate, source content remains one-shot, and neither stream is
flushed or closed by the operation. See [the API contract](../../api-docs/http-body-streams.md).

This is a foundation on the existing base, not completion of the proposed abstract
hierarchy. Existing socket upload/server response paths still use their current
pull adapters; CopyTo does not yet replace them or add unknown-length output.
The InputStream/OutputStream Memory migration and specialized producer/consumer
types remain separate bounded steps. Cancellation uses the current Task machinery;
this change does not introduce a new awaitable or claim allocation-free completion.
