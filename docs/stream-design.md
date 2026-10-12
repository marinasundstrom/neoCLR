# Stream API proposal

## Async-first naming direction — 2026-10-12

**Author correction after the first implementation:** asynchrony is the default;
Async suffixes are not necessary. Where both forms need distinguishing, mark the
synchronous alternative with Sync. This supersedes the naming choice in the first
contract below, while preserving its implemented behavior and historical evidence.
.NET remains the ergonomics target, with this explicit naming divergence; Windows
Runtime and other platforms remain comparison inputs rather than naming mandates.
See the [API naming policy](raven-conventions.md#async-first-api-naming-2026-10-12).

The next stream migration should be coherent across interfaces, implementations,
callers, compiler reference declarations and both API snapshots:

| Current implemented member | Intended member after migration |
| --- | --- |
| AsyncInputStream.ReadAsync | Read |
| AsyncOutputStream.WriteAsync | Write |
| AsyncOutputStream.FlushAsync | Flush |
| Synchronous InputStream.Read | ReadSync |
| Synchronous OutputStream.Write / Flush | WriteSync / FlushSync |

MemoryStream implements both forms, so its synchronous and Task-returning methods
must move together. Although a required cancellation token currently differentiates
some overloads, omitting a token should not select synchronous I/O. Retain short
reads, complete async writes, partial synchronous writes, cancellation precedence,
buffer ownership and closure behavior; name changes must not silently alter these
contracts. Existing socket Receive/Send and HTTP task-returning operations already
use unsuffixed names and provide a useful consistency reference.

The author discussed operation names, not a compulsory rename of AsyncInputStream /
AsyncOutputStream types or compiler protocol members. Review capability type names
separately; do not infer a new class hierarchy. Close/disposal and awaited graceful
TLS shutdown are different operations and need semantic design before naming them.
The table is a migration plan, not an API already available: the first implementation
and current reference still expose ReadAsync/WriteAsync/FlushAsync.

## First async contract — 2026-10-12

Author clarification: compare .NET, Windows Runtime and other platforms, while
keeping .NET as the ergonomics target. The initial implemented development surface
is `System.IO.AsyncInputStream` and `AsyncOutputStream`, with MemoryStream as the
first implementation. Socket/TLS adapters and pending-I/O qualification follow.

| Reference | Relevant behavior | neoCLR choice |
| --- | --- | --- |
| [.NET Stream](https://learn.microsoft.com/en-us/dotnet/api/system.io.stream?view=net-10.0) | ReadAsync returns a count; WriteAsync represents the complete write; CancellationToken parameter | Keep familiar Async names, short reads and complete-write success. Use existing Task<Result<...>> for recoverable errors. |
| [Windows Runtime IInputStream](https://learn.microsoft.com/en-us/uwp/api/windows.storage.streams.iinputstream?view=winrt-26100) and [InputStreamOptions](https://learn.microsoft.com/en-us/uwp/api/windows.storage.streams.inputstreamoptions?view=winrt-26100) | Separate input capability; Partial allows completion when bytes become available | Separate async capabilities fit existing neoCLR streams. Short reads are ordinary behavior; no read-options enum yet. |
| [Windows Runtime IOutputStream.WriteAsync](https://learn.microsoft.com/en-us/uwp/api/windows.storage.streams.ioutputstream.writeasync?view=winrt-26100) | Returns an operation with byte-count progress using IBuffer | Keep ordinary managed byte arrays and Task; a cross-language buffer/operation ABI is not required for this slice. |
| [Go io.Reader/Writer](https://pkg.go.dev/io) | Small directional interfaces and explicit transferred-byte counts/errors | Retain directional composition. Prefer .NET complete writes at the async stream boundary; Socket.Send remains the partial primitive. |

Sources reviewed 2026-10-12. This supersedes the older proposal below where all
writes returned partial counts. `AsyncOutputStream.WriteAsync` returns
`Task<Result<unit, StreamError>>`; success accepts the full range, while an error
or cancellation can follow partial progress and provides no rollback. Adapters
must loop over partial transport writes and reject zero progress. The synchronous
OutputStream contract remains unchanged. This asymmetry costs adapter code but
makes ordinary awaited writes familiar to .NET callers.

Memory operations settle inline without a worker. Pre-requested cancellation wins
before validation/mutation, following existing neoCLR task adapters. Other errors
retain synchronous MemoryStream semantics. The stream shares one cursor across
sync/async views and is not thread-safe. Cancellation is task state, never a new
StreamError case; close is not graceful async shutdown. The initial interface
requires an explicit token (`CancellationToken.None` when unused); convenience
operations can follow without new primitive implementation requirements.

Validation exposed an existing Raven native limitation: an assembly-level async
helper returning Task<Result<unit, TestFailure>> compiles with an empty discovery
registry but reports RAV2704 with generated references to the tests. Moving the same
body into an internal static helper compiles on c71baae94 and 371d1d707. The memory
suite uses that helper while retaining assembly-level Test methods. Separating
awaited results from inline pattern assertions also avoids NEOMETA001 (value block
cannot exit its enclosing expression). The state-machine native backend also rejects a test retaining many result locals
with a 77-lane layout (its existing limit is 64), and the smaller 43-lane
variant exceeds its 264-byte reference-object bound. The bounded fixture awaits
the write/read exchange and observes other settled memory outcomes directly. These are recorded compiler/backend follow-ups, not restrictions of
AsyncInputStream/AsyncOutputStream or intended Raven semantics.

[Validation evidence](../runtime/raven/tests/async-stream-validation.json): seven
memory-stream tests pass in interpreter, JIT and AOT on macOS ARM64, including the
five existing synchronous regression tests. The suite uses the existing HTTP host
profile for Task/cancellation/timer lifecycle services; no network request is needed.
Both reference snapshots are refreshed and checked. The new managed APIs have not
yet received Windows execution qualification or pending socket/TLS coverage.

The author also asks whether `System.Tasks` should become
`System.Concurrency.Tasks`. The assistant recommends that hierarchy as a separate
migration, keeping Scheduler and cancellation at the broader concurrency layer.
This is a recommendation, not a completed rename or confirmed author decision.
Compiler well-known names, bootstrap ownership, native metadata, reference APIs,
samples and tests must move together when undertaken; this slice keeps System.Tasks.

## Common async transport direction — 2026-10-12

**Status: broader design follow-through; the first async/memory slice is above.** The author requests async
stream operations and a secure-stream equivalent, then emphasizes common APIs and
learning from other platforms rather than reproducing .NET's hierarchy. The original
implemented contracts were synchronous `System.IO.InputStream` and `OutputStream`;
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

### Security and certificate foundations — 2026-10-12

**Author direction:** further development requires foundational async stream,
cryptography (`System.Security.Cryptography`) and certificate APIs. The following
is the proposed implementation sequence, not a set of shipped signatures:

1. Establish common async input/output contracts and a socket adapter, including
   buffer lifetime, cancellation, partial progress and disposal as described below.
2. Add certificate loading and inspection under the .NET-aligned
   `System.Security.Cryptography.X509Certificates` module. Start with explicit
   PEM/DER inputs and a separately owned server identity containing its certificate
   chain and private key. Specify malformed-input errors, key/certificate matching,
   disposal and retention while authentication is pending. Loading is not trust
   validation; public certificate data must not implicitly expose private key bytes.
3. Expose explicit client/server authentication through the reusable secure-stream
   layer in `System.Networking.Security`. Use certificate identities and explicit
   trust configuration rather than environment variables as the managed contract.
   Validate hostname, validity and chain policy; select and document the default
   trust source and revocation limitations before claiming client TLS support.
4. Grow `System.Security.Cryptography` with concrete consumers: secure random bytes,
   SHA-256 and HMAC-SHA-256 are initial candidates. Public primitive APIs need not
   precede certificate/TLS integration when the existing provider already performs
   those operations internally. Keep provider selection behind runtime contracts;
   do not implement cryptographic algorithms from scratch for this milestone.

The .NET baseline separates [cryptographic primitives](https://learn.microsoft.com/en-us/dotnet/standard/security/cryptography-model),
[certificate loading](https://learn.microsoft.com/en-us/dotnet/api/system.security.cryptography.x509certificates.x509certificateloader?view=net-10.0)
and [PEM certificate/private-key loading](https://learn.microsoft.com/en-us/dotnet/api/system.security.cryptography.x509certificates.x509certificate2.createfrompemfile?view=net-10.0).
These sources were reviewed on 2026-10-12. Preserve familiar responsibilities and
names where they fit; the proposed separate identity/key ownership needs comparison
with X509Certificate2 before settling signatures. It makes lifetime and private-key
access explicit but adds types and adapters. A shared provider offers consistency;
OS-backed stores can offer integration but have differing trust and key semantics.
The current Rustls host proof establishes neither a public provider abstraction nor
cross-platform certificate-store behavior.

Validate parsing, mismatched keys, untrusted/expired/wrong-host certificates,
intermediate chains and disposal during pending authentication. Run one managed
consumer on macOS and Windows with focused interpreter/JIT/AOT coverage as support
is added. Cryptographic primitives require published known-answer vectors and
provider error tests; random-byte tests cannot establish cryptographic quality.
Keep OS certificate stores, certificate issuance, broad algorithm coverage and
PKCS#12 import as later candidates rather than prerequisites for the JSON showcase.
No change here adds APIs or warrants an API-reference snapshot update yet.

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
