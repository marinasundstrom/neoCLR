# File byte streams

**Development after Preview 9.** System.Streams now has blocking file input and
output APIs. Use matching development artifacts. These are a first working slice,
not a finalized provider model or asynchronous I/O contract.

## Browse the API

- [System.Streams](xref:System.Streams) — namespace and types.
- [InputStream](xref:System.Streams.InputStream) — Read and Close contracts.
- [OutputStream](xref:System.Streams.OutputStream) — Write, Flush and Close contracts.
- [FileInputStream](xref:System.Streams.FileInputStream) — Open, Read and Close.
- [FileOutputStream](xref:System.Streams.FileOutputStream) — CreateNew, Write and Close.
- [StreamError](xref:System.Streams.StreamError) — expected open and transfer failures.
- [Flush](#flush) — complete manual entries for both renderer exclusions.

An input stream reads an existing regular file. An output stream exclusively
creates a new file; it never overwrites an existing entry. The classes expose
only their supported direction. Both own an open file until Close or invocation
teardown. No public constructor accepts a native handle.

## Capability contracts

`InputStream` and `OutputStream` are development platform interfaces. FileInputStream
and FileOutputStream implement them directly. The Storage sample's memory streams
implement the same interfaces; consumers need no disk adapter or native handle.
InputStream exposes Read and Close. OutputStream exposes Write, Flush and Close.
Neither direction exposes the opposite operation, seeking or metadata properties.

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
they use, at the cost of additional types and no ready-made duplex/seek abstraction.
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

DocFX 2.80.1 cannot render this metadata signature because neoCLR's unit is
`System.Void` inside a generic Result. Both Flush methods are therefore omitted from their generated
type pages and documented here, using its actual Raven signature. It remains part
of the public API.

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


## Text reader direction

The planned TextReader interface will describe text reading; StreamReader will
implement it over InputStream for the Storage POC so
applications can read UTF-8 text without assembling byte buffers themselves. It is
not implemented yet. The first iteration should define bounded reads, partial-byte
handling, decoding errors and ownership of the wrapped stream. Additional encodings,
line reading and async behavior are separate extensions.
