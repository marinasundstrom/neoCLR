# File byte streams

**Development after Preview 9.** System.Streams now has blocking file input and
output APIs. Use matching development artifacts. These are a first working slice,
not a finalized provider model or asynchronous I/O contract.

## Browse the API

- [System.Streams](xref:System.Streams) — namespace and types.
- [FileInputStream](xref:System.Streams.FileInputStream) — Open, Read and Close.
- [FileOutputStream](xref:System.Streams.FileOutputStream) — CreateNew, Write and Close.
- [StreamError](xref:System.Streams.StreamError) — expected open and transfer failures.
- [Flush](#flush) — complete manual entry for the one renderer exclusion.

An input stream reads an existing regular file. An output stream exclusively
creates a new file; it never overwrites an existing entry. The classes expose
only their supported direction. Both own an open file until Close or invocation
teardown. No public constructor accepts a native handle.

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

`FileOutputStream.Flush() -> Result<unit, StreamError>`

Flush calls the host file's flush operation. It returns `Ok(())`, or a typed error;
a closed stream returns Closed. There is no managed output buffer in this wrapper,
and success does not imply fsync or durability. Call it before Close when the
operation needs to observe a flush error.

DocFX 2.80.1 cannot render this metadata signature because neoCLR's unit is
`System.Void` inside a generic Result. Flush is therefore omitted from the generated
class page and documented here, using its actual Raven signature. It remains part
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
