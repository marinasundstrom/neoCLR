# Stream API proposal

## Common async transport direction — 2026-10-12

**Status: design follow-through, not implemented APIs.** The author requests async
stream operations and a secure-stream equivalent, then emphasizes common APIs and
learning from other platforms rather than reproducing .NET's hierarchy. The current
implemented contracts are synchronous `System.IO.InputStream` and `OutputStream`;
`System.Networking.Sockets.Socket` already provides Task-based Receive/Send with
cancellation and nonrenewing deadlines. The [HTTPS host proof](experiments/https-server/README.md)
shares its TLS engine and socket operation policy across native host builds, but
still selects TLS for every accepted socket through a private host flag.

The next bounded API slice should connect these existing layers, not add Windows
or macOS stream types. Prefer common async input/output capabilities in System.IO,
a socket-backed adapter in System.Networking, and a TLS decorator in
System.Networking.Security. `SslStream` is a candidate name. Keep protocol framing
in HTTP/SQL, TLS authentication/records below that, and readiness/scheduling/root
ownership in the runtime. Use the existing managed Task and CancellationToken
contracts; no new awaiter, scheduler or public native handle is needed for this step.

### Comparison and unresolved choices

Primary contracts reviewed 2026-10-12 (shipped APIs, not neoCLR commitments):

- [.NET 10 SslStream](https://learn.microsoft.com/en-us/dotnet/api/system.net.security.sslstream?view=net-10.0)
  provides TLS over another stream with explicit authentication and ownership
  configuration. Reuse that composition principle; inheritance from a single
  all-capabilities Stream base is not necessary for neoCLR's existing interfaces.
- [.NET 10 Stream.WriteAsync](https://learn.microsoft.com/en-us/dotnet/api/system.io.stream.writeasync?view=net-10.0)
  completes the requested write rather than returning a partial byte count.
  neoCLR's existing OutputStream.Write and Socket.Send return counts. Settle this
  difference explicitly: a complete-write helper can loop over a partial primitive,
  but cancellation after partial progress is not transactional rollback.
- [Go 1.27.2 net.Conn](https://pkg.go.dev/net@go1.27.2#Conn) composes byte I/O,
  close and deadlines; [crypto/tls.Conn](https://pkg.go.dev/crypto/tls@go1.27.2#Conn)
  wraps a connection. This supports a shared transport contract without exposing
  OS-specific socket implementations. Its blocking-looking calls and goroutine
  scheduling are not evidence that neoCLR should wrap blocking I/O in completed Tasks.
- [Tokio AsyncRead](https://docs.rs/tokio/latest/tokio/io/trait.AsyncRead.html)
  separates asynchronous read capability and polling. The capability split fits
  neoCLR; exposing Rust polling or pinning to managed callers would duplicate
  responsibilities already owned by neoCLR's runtime.

Adding async members to the existing synchronous interfaces would force every
implementation to provide them. Separate async capabilities allow genuine async
network implementations and synchronously completed memory operations without
pretending that blocking file/console operations have become nonblocking. Costs
include additional interfaces, adapters and overloads. This is the preferred first
experiment, not a final public signature. No independent .NET transport alternative
has been evaluated for this slice; include that review before settling the broader
secure-stream API. No performance advantage is claimed.

### Showcase acceptance target

The author identifies two neoCLR applications exchanging JSON over HTTP as the
ultimate showcase. A proposed first end-to-end scenario is a client posting a typed
record and reading it back, verifying nested JSON and error responses against the
server. Use the same application code and common transport contracts on supported
hosts; add explicit HTTPS and verified peer identity once client TLS is implemented.
This is a proposed scenario, not a new sample claimed to run today.

PostgreSQL in Docker is an optional persistence stage, reusing the developing SQL
provider after its relevant authentication/transport gaps are addressed. Running
both neoCLR applications in containers is another optional stage. Linux container
packaging needs its own interpreter/JIT/AOT and network qualification; macOS and
Windows native binaries cannot be treated as Linux container images. Decide that
execution target explicitly, then verify startup/readiness, configuration, shutdown
and the same JSON round trip. Do not couple reusable stream APIs to Docker or SQL.

### Requirements for the first executable slice

- Compile one common consumer against memory and socket adapters on macOS and
  Windows. Keep byte-array range validation, short reads, zero-count reads, EOF,
  partial writes, flush and closed-state behavior explicit. Decide ReadAsync /
  WriteAsync naming and full-write versus partial-write semantics before publishing.
- Preserve Task cancellation, distinguish timeout from cancellation and transport
  failure, and never resume a poisoned TLS session after an interrupted transfer.
  StreamError currently lacks network timeout/TLS distinctions: define the mapping
  rather than silently flattening all failures to IoFailure. Do not make System.IO
  depend on System.Networking.NetworkDeadline; evaluate a shared deadline contract
  separately while retaining the existing HTTP deadline behavior.
- Document buffer ownership until an operation settles and validate suspended GC
  roots. Start with the runtime's supported arrays/copies; new span/memory types
  or zero-copy claims are not prerequisites for common APIs.
- Specify one reader/one writer concurrency, rejection of overlapping same-direction
  operations, and close during pending I/O. Qualify duplex TLS explicitly; sequential
  HTTP request/response coverage alone does not establish duplex behavior.
- Separate abort/disposal from awaited graceful TLS shutdown under backpressure.
  Define inner-stream ownership and authentication state before exposing a decorator.
  The current best-effort close_notify is not a completed async shutdown contract.
- Replace the host-wide TLS switch with explicit per-connection authentication only
  after that common stream/ownership boundary is executable. Keep the current HTTPS
  parity matrix as a regression gate and add JIT/interpreter/AOT coverage relevant
  to each new managed contract. Update API reference snapshots when APIs are added.


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
