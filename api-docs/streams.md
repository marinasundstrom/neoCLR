# File byte streams

**Preview 10.** System.IO now has blocking file input and
output APIs. Use matching Preview 10 artifacts. These are a first working slice,
not a finalized provider model or asynchronous I/O contract.

## Browse the API

- [System.IO](xref:System.IO) — namespace and types.
- [InputStream](xref:System.IO.InputStream) — Read and Close contracts.
- [OutputStream](xref:System.IO.OutputStream) — Write, Flush and Close contracts.
- [FileInputStream](xref:System.IO.FileInputStream) — Open, Read and Close.
- [FileOutputStream](xref:System.IO.FileOutputStream) — CreateNew, Write and Close.
- [StreamError](xref:System.IO.StreamError) — expected open and transfer failures.
- [Flush](#flush) — the output and file stream flush contracts.

An input stream reads an existing regular file. An output stream exclusively
creates a new file; it never overwrites an existing entry. The classes expose
only their supported direction. Both own an open file until Close or invocation
teardown. No public constructor accepts a native handle.

## Capability contracts

`InputStream` and `OutputStream` are development platform interfaces. FileInputStream
and FileOutputStream implement them directly. The Storage sample's memory streams
implement the same interfaces; consumers need no disk adapter or native handle.
InputStream exposes Read and Close. OutputStream exposes Write, Flush and Close.
Neither directional interface exposes the opposite operation, seeking or metadata
properties. SeekableStream adds positioning separately; FileInputStream implements it.

Implementers must preserve range validation, partial counts, the 64 KiB request
limit, caller buffer ownership and idempotent Close. Check Closed before validating
ranges. InvalidRange and LimitExceeded fail before transfer. Read changes only the
returned number of elements; Write does not mutate its input. A positive-count
successful write must make progress. No thread safety or concurrent mutation is
promised. File invocation lifetime and native handle limits belong to the file
implementation, not every provider.

Unlike .NET's [Stream](https://learn.microsoft.com/en-us/dotnet/api/system.io.stream?view=net-10.0),
which exposes read/write/seek operations with capability properties, these narrow
interfaces express direction in the type. This helps consumers request only what
they use, at the cost of additional types and explicit capability composition.
This is a provisional library choice, not new VM machinery or a performance claim.

## Buffer and lifetime rules

Read and Write take a managed `byte[]`, an offset and a count, and return
`Result<int, StreamError>`. The count is bounded by the remaining array range and
64 KiB per operation. Invalid ranges fail before file I/O. Reads change only the
returned number of elements, leaving the rest untouched; aliases see those changes.
Writes may accept fewer bytes than requested, so loop over the remaining range.
A positive-count Read returning zero indicates EOF; a zero-count call does not.

Calls complete synchronously and may block the invocation. They do not spawn
threads or schedule Tasks. The caller can reuse the buffer after the call returns.
An asynchronous API will need a separate ownership and cancellation contract.
The host currently uses a temporary byte vector and copies to/from guest arrays;
this is not a zero-copy performance claim.

Close is idempotent. Later operations return Closed. Close does not flush, report
OS close errors or promise durable storage. The invocation releases all remaining
files on return, cancellation or a runtime fault; use Close promptly to release the
64-open-file budget during long operations. GC does not close a stream. A stream
retained across invocations cannot be used again: its old handle is rejected,
even if the new invocation has opened other files.

The host supplies permissions and normal symlink resolution. There is no filesystem
sandbox, seek, append, overwrite or directory enumeration in these stream classes.
An I/O failure may leave partial output; this is not transactional file replacement.

## Flush

`OutputStream.Flush() -> Result<unit, StreamError>`

An output implementation reports `Ok(())` or a typed StreamError, including Closed
after Close. Flush does not close the stream, make writes transactional or promise
durable storage. The memory sample has no pending buffer, so it returns Ok while
open. Callers should check the result before Close. Parameters: none.

`FileOutputStream.Flush() -> Result<unit, StreamError>`

Flush calls the host file's flush operation. It returns `Ok(())`, or a typed error;
a closed stream returns Closed. There is no managed output buffer in this wrapper,
and success does not imply fsync or durability. Call it before Close when the
operation needs to observe a flush error.

RavenDoc renders both Flush methods in their generated type pages, including
`System.Void` inside the generic Result. This guide explains their behavior and limits.

## A runnable example

The [provider sample](storage-experiment.md) uses the same `ByteRoundTrip(directory)`
workflow on disk and memory. It writes `Hello, värld!` through a three-byte buffer,
flushes and closes output, reads incrementally, closes input, and decodes the
collected UTF-8 bytes. Its memory backend transfers at most two bytes at once,
so the same workflow must handle short writes and reads. It also checks exclusive
creation and closed-stream errors. The verifier checks the actual disk bytes.

Text decoding happens after collection; this does not yet implement an incremental
UTF-8 decoder. The application accumulates at most 64 bytes in this small example.

## Comparison and next work

.NET's [Stream.Read](https://learn.microsoft.com/en-us/dotnet/api/system.io.stream.read?view=net-10.0)
is the baseline for buffer slices, partial counts and EOF.
[FileStream](https://learn.microsoft.com/en-us/dotnet/api/system.io.filestream?view=net-10.0)
combines directions, capability flags, seeking, disposal and sync/async operations.
This slice separates file input/output classes and represents expected failures
with Result. That makes the direction visible, but adds types and currently shares
one provisional error family between opening and transfers. No general advantage
or .NET source compatibility is claimed.

Storage alignment has begun with a [validated Path experiment](storage-experiment.md#path-value-object)
using this working stream sample. Provider lookup, identity and error contracts are next. The sample's InputStream/OutputStream interfaces
are still application-owned experiments. General capability interfaces, automatic
disposal, asynchronous I/O and suspension-aware buffer ownership remain open.


## Text readers

[TextReader](xref:System.IO.TextReader) is the consumer interface.
[StreamReader](xref:System.IO.StreamReader) reads strict UTF-8 from an InputStream.
It works with host files and the sample's partial-read memory input. The POC exposes
ReadToEnd(maxUtf8Bytes), ReadLine(maxUtf8Bytes) and Close. The [Console guide](console.md)
describes line reading. Development constructors also accept [Encoding](xref:System.Text.Encoding):
`StreamReader(input, Encodings.Ascii)` and `StreamWriter(output, Encodings.Ascii)`,
optionally followed by `leaveOpen`. Existing constructors select UTF-8. Async work
remains separate. A BOM is preserved as text, rather than
detecting other encodings.

Bounds are 0–65536 UTF-8 bytes. Negative bounds fail before reading. A zero bound
accepts only EOF for ReadToEnd; ReadLine also accepts an empty terminated line. One excess byte may be consumed to detect overflow. Invalid UTF-8 has a
distinct TextReadError; input error distinctions are preserved as named cases.
Errors do not roll back the cursor. Repeated reads at EOF produce an empty string.
In development after Preview 10, ReadToEnd decodes bounded UTF-8 chunks, retaining
at most three incomplete sequence bytes between conversions. It still accumulates
the returned String, so total memory is not constant. ReadLine retains its line
buffer. Host resource budgets also apply.

Malformed completed input is now rejected during chunk processing instead of waiting
for EOF. An underlying read may already have consumed bytes after the error; no
partial text is returned. A size-limit failure takes precedence over decoding the
read that exceeded the limit; stream errors encountered earlier remain stream errors.
This changes how far failing reads can advance and which error is observed first
compared with Preview 10's decode-at-EOF implementation. A final incomplete sequence
returns InvalidUtf8. The development [Decoder](xref:System.Text.Decoder) interface exposes incremental
conversion. The development Encoder and StreamWriter.Finish contracts are described below.

StreamReader(input) owns the input; StreamReader(input, true) leaves it open when
closed. Close is idempotent, and reads after closing return Closed. Construction
performs no read. Consumers receive TextReader; they need not know the input source.

## Seekability

[SeekableStream](xref:System.IO.SeekableStream) adds GetPosition and absolute
Seek(position), returning Int64 positions or StreamError. It is separate from
InputStream. FileInputStream implements both; a generic input need not support seeks.
Negative offsets return InvalidRange without moving, closed inputs return Closed,
and FileInputStream permits seeking past EOF. The sample memory input supports the
same operations within its Int32-backed cursor range. Byte positions are not text
character positions. Close a leave-open reader before independently repositioning
its input and creating another reader; TextReader has no repositioning API.

## Development namespace migration

Streams and text readers now use System.IO. Update System.Streams imports and type
names to System.IO and regenerate applications with matching artifacts. System.Storage
continues to own providers, files, directories and Path. Use matching Preview 10
compiler, reference and runtime artifacts.

## Future asynchronous reads

The [pending-read experiment](pending-read.md) explores cancellation requests versus
terminal completion using Task/Promise, private buffers and queued producer events.
It exercises real await and GC, but does not introduce asynchronous stream methods.


## Console and text output

TextReader/StreamReader now also support bounded ReadLine. TextWriter/StreamWriter
provide UTF-8 output by default, or the explicitly selected encoding, over any OutputStream. See the [Console guide](console.md) for
the standard channels, line endings, byte bounds and manual Flush reference.

## Selected encodings (development)

`System.Text.Encoding` is the reusable conversion policy: `Encode(string)` returns
`Result<Sequence<byte>, EncodingError>` and `CreateDecoder()` creates independent
state. `Encodings.Utf8` and `Encodings.Ascii` supply the first implementations.
`Decoder.Decode(byte[], offset, count, final)` returns valid text. Success accepts
all offered bytes; an incomplete UTF-8 scalar is copied into decoder-owned carry
and may produce empty text. The caller may immediately reuse its array. Invalid
ranges and counts above 65536 fail before mutation and can be retried. Malformed
input or finalization ends the decoder; later calls return Finished. Built-ins
never replace invalid/unrepresentable content. UTF-8 preserves U+FEFF; no codec
sniffing, BOM insertion or automatic BOM removal occurs.

`StreamReader(input, encoding[, leaveOpen])` and
`StreamWriter(output, encoding[, leaveOpen])` select this shared interface. Existing
constructors keep UTF-8 defaults. Each reader owns its decoder; the encoding itself
can be reused. ReadLine recognizes LF and CRLF in decoded text and retains decoded
suffixes supplied by a codec. Built-in codecs do not read ahead across lines.
WriteLine encodes text plus LF together. Writer results count actual encoded bytes;
strict built-in preflight finishes before output callbacks; the writer now drains
a bounded byte buffer before requesting further encoded bytes. Partial writes
are retried. Conversion errors return InvalidEncoding, while malformed UTF-8 reads
retain InvalidUtf8. Stream failures retain their existing cases.

`maxUtf8Bytes` remains the returned text's UTF-8 size, with a separate source-read
ceiling of the same number (plus one overflow byte for ReadToEnd, or up to two line
terminator bytes for ReadLine). These measures coincide for valid built-in input;
custom codecs must respect both. Future expanding/stateful codecs need explicit
source/output quota design. Errors return no partial text and may advance input.
Writer input and encoded output are each limited to 65536 bytes; WriteLine includes
LF in both limits. Runtime allocation/instruction budgets still apply, so the API
ceiling is not an allocation guarantee. Conversion does not add async behavior.

Compared with .NET Encoding/Decoder, the policy/factory roles are familiar but the
conversion boundary returns valid Unicode text instead of a UTF-16 char buffer.
This avoids exposing storage units through character APIs, at the cost of owned
result allocations and no destination-capacity/progress API. Strict ASCII avoids
silent replacement at the cost of handling conversion failures. Whole-value Encode remains available alongside the development incremental
Encoder described below. Flush forwards stream flushing; explicit Finish finalizes
a StreamWriter encoder. HTTP can reuse the
conversion policy later, but charset selection, protocol validation and framing
remain HTTP concerns. This work does not reopen the completed HTTP POC.

Focused consumers: [EncodingMain.rvn](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/text-boundaries/EncodingMain.rvn)
and [verification](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/text-boundaries/verify_encoding.py), plus the
existing reader regression fixtures. No website build or full suite is required.

## Incremental encoding (development)

[Encoding.CreateEncoder](xref:System.Text.Encoding) creates an independent
[Encoder](xref:System.Text.Encoder). Accept a complete valid text chunk with
`Accept(text, final)`, then call `Drain(buffer, offset, count)` until
[EncoderProgress](xref:System.Text.EncoderProgress).State is Ready or Finished.
[EncoderState](xref:System.Text.EncoderState).NeedsOutput means more bytes remain.
The state and BytesWritten describe the same successful drain operation.

Accept retains the immutable text, so callers need no character offset or mutable
source-buffer lifetime rule. Nonfinal pending input rejects another Accept with
EncodingError.Busy. Final accepted input rejects further input with Finished; its
bytes must still drain before the returned progress says Finished. Empty final
input is meaningful. Strict ASCII preflights all text before acceptance; rejected
input can be corrected and retried. Built-in input and drain requests are bounded
at 65536 UTF-8 source bytes and 65536 destination bytes respectively.

Drain writes only BytesWritten bytes in the supplied range, does not retain the
array and supports one-byte destinations. Fragments can split UTF-8 scalars; do not
individually decode them as complete strings. Zero capacity may return NeedsOutput
without progress. Invalid ranges and excessive requests leave output and state
unchanged. Use one session per consumer; concurrent/reentrant calls are unsupported.
Built-ins keep a pending encoded chunk of at most 256 bytes plus the accepted text;
conversion still allocates and normal host resource limits apply.

StreamWriter now drains through an independent encoder with a 256-byte buffer.
It retries partial writes before asking for more encoded output. WriteLine preflights
text plus LF together. Existing constructors keep UTF-8 defaults and leaveOpen
behavior. Write/WriteLine results still count encoded bytes actually written.

Call **StreamWriter.Finish()** to complete conversion and observe final output/errors,
then **Flush()** if stream flushing is required, and **Close()** to release ownership.
Finish returns additional bytes written; repeated successful Finish calls return zero.
It does not flush or close. Write/WriteLine after Finish return InvalidEncoding.
Close does not implicitly Finish or Flush. Finish is on StreamWriter; the general
TextWriter interface has not gained a completion requirement.

A drain/output failure can follow partial output and leaves the writer unusable for
further Write/WriteLine/Finish; these return InvalidEncoding. Earlier source-limit
and built-in ASCII preflight failures remain retryable and write no bytes. The writer
never sends over 65536 bytes per call. For a custom encoder that expands beyond that
output bound, detection can occur after partial output; there is no rollback or
implicit replay. Closed is checked first. Flush remains available until Close.

**Development migration:** custom Encoding implementations must now implement
CreateEncoder and return fresh conversion state. Do not assume an arbitrary
whole-value Encode implementation can safely be called separately for text fragments.
Built-ins and the application custom-codec examples use the same public interface.
EncoderProgress rejects negative byte counts and inactive states; providers must
also respect the capacity offered to Drain. These APIs are not included in Preview 10.

Compared with .NET Encoder.Convert, this design retains valid text rather than
reporting UTF-16 char consumption. It permits arbitrarily small byte destinations,
at the cost of retained input, two-stage usage and fragments that may split a scalar.
The synthetic final-output test is a contract fixture, not an additional supported
encoding. HTTP charset policy and additional codecs remain separate work.
