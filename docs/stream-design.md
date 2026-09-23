# Stream API proposal

**Planning update, 2026-09-23:** this note preserves the earlier sync/async design.
The [supplied Streams revision](proposals/streams-api.md) instead proposes
System.Streams InputStream/OutputStream with Task-based I/O. The
[HTTP POC roadmap](http-poc-roadmap.md) compares these as prototype candidates and
records the cancellation, buffer and cleanup decisions needed next. Neither surface
is implemented by this note; the earlier Cancelled error below is not an adopted
replacement for Task cancellation.

Proposal recorded 2026-09-16. Streams are a future byte-I/O layer for the proposed
[FileSystem capability](filesystem-design.md). The current whole-file File helpers and
console byte API remain synchronous bounded implementations and are not Stream APIs.

## Capabilities, not an omnipotent base class

The preferred starting point is a small composable capability family under
`System.IO`:

```text
ReadableStream       Read(buffer) -> Result<Size, IOError>
WritableStream       Write(buffer) -> Result<Size, IOError>
                      Flush() -> Result<Void, IOError>
SeekableStream       Position, Seek(offset, origin) -> Result<UInt64, IOError>
SizedStream          Length -> UInt64
```

Whether an empty `Stream` base containing only `Close` is useful remains open. Close
and disposal must follow the existing explicit lifecycle contracts; an interface alone
must not imply automatic cleanup. There should be no `CanRead`, `CanWrite` or `CanSeek`
flags when the capability type already communicates the operation.

Synchronous and asynchronous capabilities remain distinct:

```text
AsyncReadableStream  Read(Memory<Byte>) -> Task<Result<Size, IOError>>
AsyncWritableStream  Write(ReadOnlyMemory<Byte>) -> Task<Result<Size, IOError>>
                      Flush() -> Task<Result<Void, IOError>>
```

An implementation may support both. Wrapping blocking I/O in a Task must not be
advertised as genuinely asynchronous. `Task`/`Result` composition follows the selected
async direction: `await` removes the Task layer and `?` handles the Result layer.

## Read/write contract

Read and Write are partial operations. Read returns the number of bytes currently read,
up to the buffer length; Write returns the number accepted. A successful Read returning
zero means EOF. This retains the useful OS convention rather than wrapping every byte
read in `Option`; higher-level `TextReader.ReadLine()` may return `Option<String>` because
EOF then means absence of another line. `ReadExactly` and `WriteAll` are ordinary library
helpers that loop and return `Result<Void, IOError>`.

Expected stream failures are values, with a provisional family including Closed,
Interrupted, TimedOut, Cancelled, InvalidPosition and DeviceFailure. Filesystem errors
remain the responsibility of opening/resolving through `FileSystem`; a file open may
return `FileError`, while later stream reads return `IOError`. The final taxonomy and
cancellation semantics remain open.

## Composition and higher-level readers

Stream decorators request only what they need:

```text
ReadableStream -> BufferedReader -> GzipReader -> TextReader(Encoding.Utf8)
WritableStream -> BufferedWriter -> GzipWriter -> TextWriter(Encoding.Utf8)
```

Text interpretation belongs above bytes and uses the separate Encoding model. Binary
reading/writing, byte order, varints and serialization are a later design exercise,
not members of the first stream contract. Buffering should be explicit rather than
secretly inserted into every stream.

Memory sources and sinks should distinguish borrowed/read-only input from owned,
growable output (`MemoryReader`, `MemoryWriter`, `MemoryBuffer`) instead of reproducing
one constructor-heavy `MemoryStream` type. `CopyTo` can be an extension over
`ReadableStream` and `WritableStream`, returning the number of bytes transferred; an
async form uses the corresponding async capabilities.

## .NET baseline and alternatives

The .NET `Stream` class combines read, write, seek, length, timeout and synchronous/
asynchronous members, with capability flags and unsupported-operation behavior. This is
familiar and interoperable, but a consumer can discover unsupported operations only at
runtime and the API reflects historical sync/async accretion. The proposal preserves
familiar operation names while moving capability requirements into types.

| Alternative | Benefit | Cost / limitation |
| --- | --- | --- |
| Adopt .NET `Stream` | Maximum familiarity and easy conceptual porting | Large base contract, capability flags, unsupported-operation errors and sync/async duplication |
| Keep one `Stream` with capability queries | Smaller migration from .NET and dynamic composition | Invalid operations remain runtime discoveries; interfaces do not enforce requirements |
| Use composable sync/async capability interfaces | Static clarity, precise decorators and honest async boundaries | More interface types, generic bounds and adapter work |
| Use callbacks only | Works on constrained event-driven hosts | Poor fit for ordinary sequential composition and text-reader ergonomics |

This is a deliberate adaptation, not a claim that .NET's design is universally wrong;
its broad base remains useful for compatibility boundaries. A future adapter can expose
the capability family to a .NET host without making the host model the NeoCLR contract.

## Decision and validation

Prefer capability-oriented streams, explicit buffering, separate text readers/writers,
and FileSystem methods that return the narrow stream capability they provide. Keep the
base `Stream` question, exact buffers/views, ownership and async scheduler provisional.
The design costs more types and requires precise generic/interface and lifetime support.

Before implementation, validate partial reads/writes, zero-byte EOF, `ReadExactly`,
`WriteAll`, close/flush failure, repeated disposal, buffer aliasing, borrowed-memory
lifetime, seek bounds, sparse/unknown length, decorator cleanup, cancellation, and
multiple pending async operations. Run the same copy/text pipeline against host-file,
memory, archive and a non-seekable/network-like backend. Handwritten IL must be unable
to call capabilities it does not implement. Compare observable behavior with .NET
Stream on a pinned toolchain; measure buffering/allocation costs instead of assuming
the smaller hierarchy is faster.
