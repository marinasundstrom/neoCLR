# Known-length HTTP stream uploads

Development after Preview 9. This is the closing feature increment for the bounded
[HTTP POC](../../http-capabilities.md), not general streaming HTTP or HTTP/2/3 support.

```sh
python3 docs/experiments/http-stream-upload/verify.py \
  --toolchain-root /absolute/path/to/matching/bundle \
  --runner /absolute/path/to/measure_async
```

The checked Main.rvn constructs HttpContent.FromStream(input, length, leaveOpen)
and sends it through HttpClient and a forwarding HttpHandler. Its generated binary
source retains no whole body; 71-byte short reads are sent in bounded chunks to an
independent Python peer. The 1,025-byte body exceeds the old buffered upload bound.
This does not enlarge neoCLR server admission, which remains 1,024 buffered bytes.

## Contract

- Factory length is 0–65,536 bytes, measured from the source's current position.
  Negative/oversized lengths fail without reading, closing or taking ownership.
  Exact-length consumption leaves extra source bytes unread; early EOF and impossible
  read counts become HttpError.Content(StreamError.IoFailure). Other read errors
  retain their StreamError cause. Zero-length uploads do not read the source.
- leaveOpen=false transfers close responsibility to the returned content. Socket
  admission occurs after request validation. An admitted exchange consumes it once
  and disposes it after success, error or acknowledged cancellation. Unsent content
  remains the caller's disposal responsibility. Pre-cancellation and invalid headers
  leave it unconsumed. Dispose is idempotent; borrowed sources remain open.
- The sender writes headers first, then reads at most 256 bytes and fully sends that
  chunk before reading more. Existing socket deadlines remain unchanged. No whole
  body is assembled; the current chunk is copied into bounded managed storage.
- Cancellation is checked around source reads and forwarded to native operations.
  A synchronous InputStream.Read cannot be interrupted mid-call. Sources must return
  promptly and remain exclusively available; guest faults are not Result failures.
- IsBuffered and TryGetBytes distinguish content representation. Bytes retains its
  buffered contract and faults for stream content instead of inventing empty bytes.
  ReadText returns an error for a stream. JSON readers reject stream content and
  server response encoding returns Unsupported. Responses remain buffered.
- Custom handlers may inspect and forward stream content. General custom body
  consumption, async sources, unknown lengths, retries/replay and streaming responses
  are separate future contracts. Length is provisional for this known-length slice.

## .NET and HTTP comparison

The [.NET 10 StreamContent implementation](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Net.Http/src/System/Net/Http/StreamContent.cs)
wraps a stream, computes length when seekable, disposes its source, and permits
replay by rewinding seekable sources. neoCLR's factory instead takes an explicit
range and ownership flag and rejects all reuse. This admits non-seekable sources
without a new seek contract, but leaves sizing to callers and postpones replay and
asynchronous source reads. Retaining only buffered content is simpler but requires
whole-body storage. A polymorphic async content reader/writer is the longer-term
alternative, with larger handler and lifecycle implications.

[HTTP/1.1 body length rules](https://www.rfc-editor.org/rfc/rfc9112.html#section-6.3)
guide fixed-length framing. No Transfer-Encoding is emitted here. The wire still
closes each connection. Those choices are provider limits, not the future common
HTTP model; see the [modern HTTP direction](../../http-capabilities.md#modern-http-direction).

## Validation and integration

All 11 serial independent-peer cases pass: owned and borrowed uploads, empty body,
premature EOF, source error, invalid read count, cancellation during Read,
pre-cancellation, disposed content, invalid headers and refused connection.
They check binary bytes, declared length, short reads, one-shot/concurrent reuse,
response association, idempotent disposal, no reads on admission failures and
connection closure. All runs end with zero live managed objects under collection
pressure. [Retained results](results.jsonl) record the final observations.

The compiler bridge projects the new source union case and admits the new public
members while rejecting internal upload helpers. Runtime Contract configuration,
Raven compiler sources and native socket APIs are unchanged. Rebuild with matching
reference/library/bridge artifacts; the new union case is a development API change.
The conditional expression-bodied Length getter initially emitted a default zero;
the committed getter uses explicit returns and the fixture checks its value before
sending. This remains a separate compiler investigation, not an optimized workaround.

Library fragments for HttpClient (including server/content), HttpError and JsonValue
were regenerated; other manifests refresh shared source fingerprints only. Their
unchanged bodies were not all recompiled. The [validation record](validation.json) retains artifact hashes and regression
outcomes. All 623 signature checks, library/API snapshots, 18 website tests and the
combined 1,038-page site build pass. The mapped HTTP/JSON pair, independent client/
server cases and buffered POST client checks pass with zero final live objects.
An additional consumer checks maximum declared length, JSON rejection and Disposable
interface dispatch; the importer needed explicit admission of that conversion. No full runtime suite or
publication is part of this feature slice.
