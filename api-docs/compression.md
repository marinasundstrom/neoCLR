# System.IO.Compression (development)

`GZipStream` implements InputStream and OutputStream in System.Runtime. It adapts
async byte streams to the RFC 1952 gzip format. See the
[design and HTTP follow-through](https://github.com/marinasundstrom/neoCLR/blob/main/docs/gzip-compression.md) for implementation choices.

| Member | Contract |
| --- | --- |
| `CreateCompressor(output: OutputStream)` | Owns the output stream. |
| `CreateCompressor(output: OutputStream, leaveOpen: bool)` | Creates a compressor; preserves output on Close when requested. |
| `CreateDecompressor(input: InputStream)` | Owns the input stream. |
| `CreateDecompressor(input: InputStream, leaveOpen: bool)` | Creates a bounded decoder; preserves input on Close when requested. |
| `Read(buffer: byte[], offset: int, count: int[, cancellationToken: CancellationToken])` | Task of Result<int, StreamError>; checks complete gzip before returning bytes, then zero at EOF. Zero count does not read input. |
| `Write(buffer: byte[], offset: int, count: int[, cancellationToken: CancellationToken])` | Task of Result<unit, StreamError>; incrementally compresses and writes the complete range. Zero count emits nothing. |
| `Finish([cancellationToken: CancellationToken])` | Task of Result<unit, StreamError>; emits final block and checksum/size trailer, including empty input. Idempotent after success. |
| `Flush([cancellationToken: CancellationToken])` | Task of Result<unit, StreamError>; flushes the destination without finishing gzip. |
| `Close()` | Idempotent disposal; never writes a trailer or flushes. |

Await Finish before closing compressed output, and Flush if required by the consumer.
Writes after Finish and wrong-direction operations return WrongAccess. Close defaults
to closing the underlying stream; leaveOpen is explicit. No seeking or level selector
is provided. Do not overlap operations or mutate borrowed buffers until completion.

Pre-cancellation cancels the Task before mutation or validation. Invalid ranges return
InvalidRange. After Close, operations return Closed. Once input/output begins, errors
or cancellation poison the wrapper; the first stream error is preserved, subsequent
operations return IoFailure. Input consumed or output delivered is not rolled back.

Decompression supports all DEFLATE block types, optional headers and concatenated
members. It rejects malformed headers/blocks, truncated input, checksum/size mismatch
and trailing junk with `StreamError.InvalidData`. This additive union case may require
updating exhaustive matches. It consumes the source to EOF before returning data and
limits input to 65536 bytes and total decoded data to 65536 bytes; overflow returns
LimitExceeded without modifying the caller's destination. Use an EOF-delimited body,
not a live HTTP connection. Incremental decoding and automatic HTTP integration are
gaps, not implemented behavior. Compression has no whole-stream size bound and uses
bounded blocks; it can expand incompressible data.

The [compiled suite](https://github.com/marinasundstrom/neoCLR/blob/main/runtime/raven/tests/gzip/Tests.rvn) demonstrates creation,
Write/Finish/Read ordering, ownership and error handling. The factories avoid ambiguous
.NET-style constructors when a stream supports both input and output capabilities.

StreamReader preserves binary format failures as `TextReadError.InvalidData`; this
additive text-error case also requires updating exhaustive matches.
