# Byte streams

Development stream I/O is asynchronous by default. Use matching compiler/runtime
artifacts: this is a breaking change from the synchronous contracts and the interim
AsyncInputStream/AsyncOutputStream interfaces, which have been removed.

<a id="browse-the-api"></a>
<a id="capability-contracts"></a>
<a id="buffer-and-lifetime-rules"></a>
<a id="flush"></a>
<a id="comparison-and-next-work"></a>
<a id="future-asynchronous-reads"></a>
<a id="async-streams-development"></a>

## Contracts

- [InputStream](xref:System.IO.InputStream): `Read(buffer, offset, count)` returns
  `Task<Result<int, StreamError>>`. Short reads succeed; a positive-count read
  returning zero means EOF. A zero-count read consumes nothing.
- [OutputStream](xref:System.IO.OutputStream): `Write(buffer, offset, count)` returns
  `Task<Result<unit, StreamError>>`. Success covers the entire requested range.
  `Flush()` returns `Task<Result<unit, StreamError>>`.
- Read, Write and Flush also accept a final
  [CancellationToken](xref:System.Concurrency.CancellationToken). Omitted tokens
  mean CancellationToken.None. Cancellation produces a cancelled Task, not a
  StreamError or a successful short write.
- Both interfaces expose immediate, idempotent `Close()`. Close disposes resources;
  it does not implicitly flush, finalize an encoder or perform graceful TLS shutdown.
- [SeekableStream](xref:System.IO.SeekableStream) supplies immediate position/seek
  operations separately. No synchronous I/O interface is currently provided.

A Task represents an operation that may finish later. It may already be complete
when returned; it does not promise another thread or nonblocking execution. Use
`await` even when the current implementation completes immediately.

Keep the source/destination range alive and unmodified until settlement. Reads
change only the returned bytes; writes do not modify their source. An error or
cancellation may follow partial transfer and does not roll it back. Serialize
operations on a stream unless its implementation explicitly documents concurrency.
Await pending operations before closing or reusing their buffers.

.NET offers paired synchronous/asynchronous Stream methods. neoCLR deliberately
uses ordinary names and separate directional contracts, reducing requirements on
providers at the cost of compatibility with old synchronous implementations. Full
write success and short-read/EOF semantics follow .NET's asynchronous I/O contracts.

<a id="seekability"></a>
<a id="streams-and-storage"></a>
<a id="a-runnable-example"></a>

## Implementations and limits

[MemoryStream](xref:System.IO.MemoryStream) implements both directions and seeking.
Read/Write/Flush settle inline without scheduled work. Storage is limited to 64 KiB;
gap writes zero-fill, overwrites preserve remaining content, and empty writes do not
extend the length. Rejected ranges/capacity changes preserve position and storage.
GetLength, GetPosition and Seek return immediate Results. Close is shared across
interface views. Pre-cancelled operations settle as cancelled before validation or
mutation, including on closed streams.

[FileInputStream](xref:System.IO.FileInputStream) opens an existing regular file and
supports seeking. [FileOutputStream](xref:System.IO.FileOutputStream) exclusively
creates a new file; it never overwrites an existing entry. Their current providers
perform **blocking host calls inline**, then return completed Tasks. The same is true
of console byte streams. These adapters do not yet interrupt an in-progress host
call; they check cancellation before starting. Nonblocking file/console integration
is separate work. Output adapters loop over host partial writes and reject zero
progress. Buffers and individual requests are bounded to 64 KiB.

File Open/CreateNew and Close remain immediate resource operations. Stream errors
include Closed, InvalidRange, LimitExceeded, AccessDenied and IoFailure. Invalid
ranges fail before transfer; pre-cancellation precedes Closed/range checks. The host
supplies permissions and normal symlink resolution. These APIs provide no filesystem
sandbox or transactional replacement. Flush asks the OS to flush buffered output;
it does not guarantee durable disk storage. Close cannot report OS close errors.

A file remains owned until Close or invocation teardown. GC does not close it. The
current invocation budget is 64 open files; old handles cannot be reused in a later
invocation. Native host buffers may copy bytes; this is not a zero-copy API.

<a id="text-readers"></a>
<a id="console-and-text-output"></a>
<a id="selected-encodings-development"></a>
<a id="incremental-encoding-development"></a>

## Text and JSON adapters

[TextReader](xref:System.IO.TextReader), [StreamReader](xref:System.IO.StreamReader),
[TextWriter](xref:System.IO.TextWriter) and [StreamWriter](xref:System.IO.StreamWriter)
return Task results from I/O. TextReader/StreamReader now also provide PeekChar, ReadChar
and counted Read with shared grapheme-aware lookahead; see the [text read contract](console.md#grapheme-peek-and-counted-reads-development).
Await ReadLine, ReadToEnd, Write, WriteLine, Flush and
StreamWriter.Finish. Text writes still report encoded byte counts. Finish finalizes
the encoder; Flush only flushes the underlying output. Close follows leaveOpen and
never implicitly finalizes conversion. Encoding operations on memory remain immediate.

One-shot [Encoding.Decode](xref:System.Text.Encoding) takes `ReadOnlySlice<byte>`
and returns `Result<string, EncodingError>`. It borrows only for the synchronous
call and returns owned text; later source mutation cannot change that text.
UTF-8 decoding reads the selected window directly in native execution, without a
managed input snapshot. The interpreter builds owned UTF-8 storage from its value
slots; this is not a claim of zero host allocation. Empty input is valid; BOM and
NUL are preserved. Built-ins retain their 65,536-byte quota, strict malformed or
truncated UTF-8 errors, ASCII rejection and independent subsequent calls.

This development signature replaces `Decode(Sequence<byte>)`. Array callers
convert implicitly; custom encodings must implement the borrowed signature. A
noncontiguous sequence must materialize storage explicitly when decoding it as one
window. No legacy sequence overload or AsSlice adapter is added. Native metadata
renders the complete member; the legacy CLI scaffold cannot represent its borrowed
parameter and deliberately omits it.

Incremental codecs take views directly: `Decoder.Decode(ReadOnlySlice<byte>, final)`
and `Encoder.Drain(Slice<byte>)`. Arrays convert implicitly; pass `buffer[start..end]`
for a subwindow. These replace the former array/offset/count signatures without
compatibility overloads. Decoders copy incomplete UTF-8 bytes into owned state;
encoders retain accepted text but never retain a destination view. Built-ins accept
at most 65536 bytes per call. Drain modifies only the reported prefix of its view,
leaving the suffix and surrounding array untouched. Zero-capacity progress and
retryable quota errors are unchanged. Invalid window construction faults before
the codec call, rather than returning a codec error; EncodingError.InvalidRange has been removed.
StreamReader/StreamWriter use these borrowed views only for immediate codec calls;
their underlying asynchronous stream transfers still use the existing array API.

JsonSerializer overloads accepting InputStream/OutputStream return Task results;
string/JSON-node/object conversion overloads remain immediate. The serializer borrows
streams and preserves existing document quotas and errors. HTTP stream uploads await
InputStream reads, including pending reads, before sending the next body chunk.

See [the design and current qualification](https://github.com/marinasundstrom/neoCLR/blob/main/docs/stream-design.md) and the tested
memory-stream, encoding-streams, encoding-boundaries and json-streams suites under
the owning components’ `test` projects. Native library support does not imply that the historical
synchronous CLI execution bridge supports these new signatures.

<a id="gzip-composition-development"></a>

## Gzip composition

[GZipStream](compression.md) wraps these capabilities. Compression writes incrementally
and requires awaited Finish before Close. The initial decoder reads to EOF with
64 KiB compressed/decoded limits and validates gzip before exposing bytes. InvalidData
is preserved through StreamReader as TextReadError.InvalidData.

[Development SslStream](ssl-streams.md) supplies authenticated encrypted input/output over these stream capabilities.


## Optional synchronous byte streams (development)

`SyncInputStream` and `SyncOutputStream` are independent capabilities. A provider
may implement either, both, or only asynchronous InputStream/OutputStream. Calls
may block the current thread and do not create Tasks through these interfaces.

| Capability member | Contract |
| --- | --- |
| `SyncInputStream.Read(buffer: Slice<byte>) -> Result<int, StreamError>` | Reads up to the borrowed destination length. Short reads succeed; zero means EOF for nonempty buffers. Empty reads consume nothing. Unwritten bytes are preserved. |
| `SyncOutputStream.Write(buffer: ReadOnlySlice<byte>) -> Result<unit, StreamError>` | Success means the entire range was accepted. Failure can follow partial output; the provider must document stronger guarantees. |
| `SyncOutputStream.Flush() -> Result<unit, StreamError>` | Flushes implementation buffers; does not promise disk durability or peer receipt. |
| `Close()` on either capability | Idempotent closure, without an implicit flush. |

Borrowed views last only for the call and must not be retained. These contracts
introduce neither cancellation nor an obligation to offer synchronous I/O on network
or HTTP providers. Use interface conformance to discover support.

MemoryStream explicitly implements Read, Write and Flush through these interfaces.
Its existing concrete methods remain asynchronous. Both paths share storage, cursor,
65536-byte quota and closure; serialize access. Closed operations report Closed,
and writes exceeding the quota report LimitExceeded without changing storage or
position. Synchronous calls use the shared direct core, never wait on a Task and
never materialize a temporary array for the view. Storage growth and Result handling
may still allocate; this is not a zero-allocation stream guarantee.

The native API reference includes both borrowed members. The legacy CLI scaffold
omits precisely `SyncInputStream.Read(Slice<byte>)` and
`SyncOutputStream.Write(ReadOnlySlice<byte>)`, whose opaque view representations it
cannot safely project. This manual entry covers that interim reference gap.
Console now exposes these synchronous byte capabilities. Separate SyncStreamReader
and SyncStreamWriter wrappers adapt them to direct text results; see the
[factory and capability reference](console.md#separate-text-capabilities-and-factories-development).
