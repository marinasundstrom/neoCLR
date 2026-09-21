# NeoCLR Streams API

## 1. Purpose

Streams represent **sequential byte flow**.

They are not inherently files, network connections, memory buffers, or devices. Those systems may expose streams as a common way of reading or writing bytes.

NeoCLR therefore treats streams as a small independent platform abstraction:

```text
System.Streams
```

The API follows familiar .NET stream concepts while adapting them to NeoCLR's type system, Task model, and capability-oriented API design.

The major differences from .NET are:

```text
.NET Stream                     NeoCLR

Stream                          no universal Stream required
CanRead                         InputStream type
CanWrite                        OutputStream type
CanSeek                         Seekable capability
Read / ReadAsync                Read
Write / WriteAsync              Write
exceptions                      Result<T, E>
CancellationToken parameters    Task cancellation
System.IO ownership             System.Streams
```

The central principle is:

> **Direction and capabilities are expressed by types rather than runtime state.**

---

# 2. Namespace

The core abstractions live in:

```raven
namespace System.Streams;
```

This is intentionally independent of:

```text
System.Storage
System.Networking
System.Data
System.Security
System.Text
```

because streams cross all of those domains.

For example:

```text
Storage.File
    ↓
InputStream

Networking.Connection
    ↓
InputStream + OutputStream

Data.Compression.GzipReader
    ↓
InputStream → InputStream

Security.Cryptography
    ↓
InputStream → InputStream
```

No one subsystem owns streams.

---

# 3. There is no universal `Stream`

NeoCLR does not begin with:

```raven
interface Stream
{
}
```

The fundamental concepts are instead the two directions of byte flow:

```raven
interface InputStream
{
    func Read(buffer: Memory<Byte>)
        -> Task<Result<Size, StreamError>>;
}

interface OutputStream
{
    func Write(buffer: ReadOnlyMemory<Byte>)
        -> Task<Result<Size, StreamError>>;

    func Flush()
        -> Task<Result<(), StreamError>>;
}
```

An object supporting both directions simply implements both:

```raven
class Connection :
    InputStream,
    OutputStream
{
    ...
}
```

There is consequently no need for:

```raven
stream.CanRead
stream.CanWrite
```

An API requiring input asks for an `InputStream`.

An API requiring output asks for an `OutputStream`.

This also prevents APIs from unnecessarily requesting capabilities they do not use.

---

# 4. Input and output are semantic roles

The names `InputStream` and `OutputStream` describe the direction from the perspective of the consumer.

An `InputStream` is something from which bytes can be obtained:

```raven
func Parse(input: InputStream)
    -> Task<Result<Document, ParseError>>;
```

An `OutputStream` is something to which bytes can be sent:

```raven
func Encode(
    document: Document,
    output: OutputStream
) -> Task<Result<(), EncodeError>>;
```

This makes direction visible directly in API signatures.

It also allows implementations to expose only the appropriate side of a resource.

---

# 5. Task-based I/O is the default

NeoCLR does not duplicate operations into synchronous and asynchronous variants.

There is no:

```text
Read
ReadAsync

Write
WriteAsync

Flush
FlushAsync
```

Instead:

```raven
let count = await input.Read(buffer)?;
await output.WriteAll(data)?;
await output.Flush()?;
```

Potentially waiting operations return:

```raven
Task<Result<T, E>>
```

`Task` describes completion semantics. It does **not** imply that another thread must execute the operation.

A memory-backed input stream may simply return an already-completed task:

```raven
func Read(buffer: Memory<Byte>)
    -> Task<Result<Size, StreamError>>
{
    let count = CopyAvailableBytes(buffer);
    return Ok(count);
}
```

An OS-backed implementation may genuinely suspend.

Today Raven may implement that suspension using a compiler-generated state machine.

The long-term NeoCLR model may instead suspend execution directly in the runtime.

Neither mechanism changes the public stream contract.

> **Task is part of the API model; suspension strategy is an implementation detail.**

---

# 6. Buffers across suspension boundaries

The fundamental operations use:

```raven
Memory<Byte>
ReadOnlyMemory<Byte>
```

rather than stack-bound spans:

```raven
Span<Byte>
ReadOnlySpan<Byte>
```

because an operation may remain suspended after its caller's current stack frame has yielded.

Therefore:

```raven
InputStream.Read(Memory<Byte>)

OutputStream.Write(ReadOnlyMemory<Byte>)
```

while immediate memory processing can continue using spans.

This establishes a useful distinction:

```text
Span<T>
    ↓
immediate computation

Memory<T>
    ↓
potentially suspended operation
```

Streams themselves do not thereby take ownership of the supplied memory.

---

# 7. Reading

The primitive input operation is:

```raven
func Read(buffer: Memory<Byte>)
    -> Task<Result<Size, StreamError>>;
```

It means:

> Read up to `buffer.Length` bytes into the supplied memory.

The operation may return fewer bytes than requested.

A successful result of zero indicates the end of the stream:

```raven
let count = await input.Read(buffer)?;

if count == 0 {
    // End of input.
}
```

This remains preferable to:

```raven
Result<Option<Size>, StreamError>
```

at the raw byte-stream level.

EOF is part of the byte-stream protocol.

Semantic readers layered above streams may use `Option` where absence has domain meaning:

```raven
func ReadLine()
    -> Task<Result<Option<String>, TextError>>;
```

---

# 8. Writing

The primitive output operation is:

```raven
func Write(buffer: ReadOnlyMemory<Byte>)
    -> Task<Result<Size, StreamError>>;
```

A successful write may consume fewer bytes than supplied.

Partial I/O remains part of the low-level contract because it maps naturally onto operating-system, device, and network behavior.

Most application code should use higher-level operations:

```raven
func ReadExactly(
    this InputStream input,
    buffer: Memory<Byte>
) -> Task<Result<(), StreamError>>;

func WriteAll(
    this OutputStream output,
    buffer: ReadOnlyMemory<Byte>
) -> Task<Result<(), StreamError>>;
```

giving:

```raven
await input.ReadExactly(header)?;
await output.WriteAll(payload)?;
```

The primitive contract remains efficient while normal code remains ergonomic.

---

# 9. Seeking is orthogonal

Direction does not imply seeking.

Seeking is therefore another capability:

```raven
interface Seekable
{
    func Seek(
        offset: Int64,
        origin: SeekOrigin
    ) -> Task<Result<UInt64, StreamError>>;
}
```

with:

```raven
enum SeekOrigin
{
    Start,
    Current,
    End
}
```

A file-backed input stream might implement:

```text
InputStream
Seekable
Sized
```

while a TCP input stream implements only:

```text
InputStream
```

An API requiring both reading and seeking expresses both requirements:

```raven
func ReadIndex(input: InputStream & Seekable)
{
    ...
}
```

The exact intersection syntax remains a Raven language question.

---

# 10. Size is independent

Knowing the total size of a resource is not equivalent to seeking through it.

It can therefore be represented independently:

```raven
interface Sized
{
    func GetLength()
        -> Task<Result<UInt64, StreamError>>;
}
```

This is deliberately an operation rather than necessarily a property.

For an external resource, determining its length may require I/O or may fail.

For an in-memory implementation, the returned Task can complete immediately.

This follows the broader NeoCLR API principle that properties should generally represent immediately available state rather than hide potentially suspending work.

---

# 11. Copying between directions

The relationship between input and output naturally gives us:

```raven
func CopyTo(
    this InputStream source,
    destination: OutputStream
) -> Task<Result<UInt64, StreamError>>;
```

Usage:

```raven
let copied = await source.CopyTo(destination)?;
```

The implementation may initially use an ordinary buffer loop.

NeoCLR or platform-specific implementations may later optimize compatible endpoints using zero-copy or other native transfer mechanisms without changing the API.

---

# 12. Directional wrappers

The directional model should continue into higher-level stream types.

Rather than a mode-dependent:

```text
BufferedStream
```

prefer:

```text
BufferedInputStream
BufferedOutputStream
```

or, if we want reader/writer terminology at that layer:

```text
BufferedReader
BufferedWriter
```

For example:

```raven
class BufferedInputStream : InputStream
{
    init(source: InputStream);
}

class BufferedOutputStream : OutputStream
{
    init(destination: OutputStream);
}
```

There is no mode flag and no unsupported direction.

The type tells us what it does.

---

# 13. Transform streams

Transformations follow the same rule.

Compression should not require a single mode-dependent equivalent of .NET's `GZipStream`.

Instead:

```raven
class GzipReader : InputStream
{
    init(source: InputStream);
}

class GzipWriter : OutputStream
{
    init(destination: OutputStream);
}
```

This allows natural composition:

```raven
let compressed = GzipReader(source);
let buffered = BufferedReader(compressed);
```

Likewise, encryption, checksumming, encoding and other transformations can expose precisely the direction they implement.

---

# 14. Streams and memory are different abstractions

Not every sequence of bytes should become a stream.

Immediate memory processing uses the memory abstractions:

```text
Span<Byte>
ReadOnlySpan<Byte>
Memory<Byte>
ReadOnlyMemory<Byte>
Buffer
```

For example:

```raven
let reader = BinaryReader(data.Span);

let version = reader.Read<UInt16>()?;
let flags = reader.Read<UInt32>()?;
```

No Task is necessary because nothing waits.

Streams enter the model when sequential I/O is required:

```raven
let buffer = Memory<Byte>(size);

await input.ReadExactly(buffer)?;

let reader = BinaryReader(buffer.Span);
```

This produces three distinct layers:

```text
Memory manipulation
        ↓
Span / Memory / Buffer

Sequential byte flow
        ↓
InputStream / OutputStream

Structured interpretation
        ↓
Text / binary readers and writers / codecs
```

---

# 15. Memory-backed streams

Adapters remain useful when an API expects a stream but the source or destination is memory.

For example:

```raven
let input = MemoryReader(data);
```

and:

```raven
let output = MemoryBuffer();
await output.WriteAll(data)?;
```

A memory reader may implement:

```text
InputStream
Seekable
Sized
```

Every Task may complete immediately.

Again:

> **Asynchronous compatibility does not imply asynchronous execution.**

---

# 16. Storage integration

Streams are not part of `System.Storage`, but Storage can expose them.

Conceptually:

```raven
namespace System.Storage;

interface File
{
    func OpenRead()
        -> Task<Result<InputStream, FileError>>;

    func OpenWrite(...)
        -> Task<Result<OutputStream, FileError>>;
}
```

This gives Storage responsibility for:

```text
File
Directory
Path
StorageProvider
StorageUnit
permissions
metadata
opening resources
```

while `System.Streams` takes over once sequential byte transfer begins.

That boundary is useful:

```text
System.Storage
      │
      │ OpenRead()
      ▼
System.Streams.InputStream
```

A storage error opening a nonexistent file is not necessarily the same thing as an error encountered while consuming an already-open stream.

---

# 17. Networking integration

Networking uses exactly the same capabilities.

A TCP connection could expose both directions:

```raven
interface Connection :
    InputStream,
    OutputStream
{
    ...
}
```

Higher-level protocols can then operate on stream capabilities without depending on sockets:

```text
Socket / Connection
        ↓
InputStream + OutputStream
        ↓
buffering / TLS
        ↓
HTTP
```

This is one reason streams deserve their own namespace rather than living under Storage or Networking.

---

# 18. Text is layered above streams

Streams transport bytes.

They do not inherently have a text encoding.

Text decoding belongs to the text layer:

```raven
let reader = TextReader(
    input,
    Encoding.Utf8
);
```

and:

```raven
let writer = TextWriter(
    output,
    Encoding.Utf8
);
```

UTF-8 can naturally be the NeoCLR default:

```raven
let reader = TextReader(input);
```

without making `InputStream` itself text-aware.

Whether `TextReader`/`TextWriter` ultimately belong in `System.Text`, `System.Streams`, or a more specialized namespace can be decided with the Text API.

---

# 19. Errors

Expected stream-operation failures are values:

```raven
await input.Read(buffer)?
```

rather than catchable exceptions.

A provisional error family could be:

```raven
enum StreamError
{
    Closed,
    Interrupted,
    TimedOut,

    PermissionDenied,

    InvalidPosition,

    Device(StreamErrorCode),
    Other(StreamErrorCode)
}
```

The important point is that **cancellation is not a `StreamError`**.

With the NeoCLR Task model:

```text
Task<Result<Size, StreamError>>
```

can conceptually complete as:

```text
Completed(Ok(size))
Completed(Err(streamError))
Cancelled
```

Cancellation belongs to Task execution rather than being reported as an I/O failure.

Likewise, domain-specific errors should remain with their originating domain. Failure to locate or open a file belongs to Storage; failure while reading from an already-open byte source belongs to the stream operation.

---

# 20. Cancellation and timeouts

Streams should not repeat:

```text
CancellationToken cancellationToken = default
```

throughout their API.

Cancellation belongs to the Task execution model.

Thus:

```raven
await input.Read(buffer)
```

participates in the surrounding Task cancellation context, and a suspended underlying operation should be cancelled where the runtime/provider supports it.

Timeouts should similarly be treated primarily as execution policy rather than mutable properties of every stream.

This allows the same Task mechanisms to apply consistently to:

```text
Streams
Storage
Networking
HTTP
database operations
processes
```

The precise mechanism belongs to the Task proposal rather than the Streams API.

---

# 21. Resource lifetime

Some stream implementations represent resources requiring deterministic cleanup.

Streams should use NeoCLR's general resource-lifetime mechanism rather than inventing a stream-specific `Close()` protocol.

Conceptually:

```raven
using input = await file.OpenRead()?;
await Process(input);
```

should release the underlying resource at the end of its lifetime.

The general resource model must eventually account for resources requiring asynchronous cleanup.

This remains deliberately outside the core Streams proposal.

---

# 22. Initial core surface

The first version can consequently remain very small:

```raven
namespace System.Streams;

interface InputStream
{
    func Read(buffer: Memory<Byte>)
        -> Task<Result<Size, StreamError>>;
}

interface OutputStream
{
    func Write(buffer: ReadOnlyMemory<Byte>)
        -> Task<Result<Size, StreamError>>;

    func Flush()
        -> Task<Result<(), StreamError>>;
}

interface Seekable
{
    func Seek(
        offset: Int64,
        origin: SeekOrigin
    ) -> Task<Result<UInt64, StreamError>>;
}

interface Sized
{
    func GetLength()
        -> Task<Result<UInt64, StreamError>>;
}

enum SeekOrigin
{
    Start,
    Current,
    End
}
```

Standard operations:

```raven
func ReadExactly(
    this InputStream input,
    buffer: Memory<Byte>
) -> Task<Result<(), StreamError>>;

func WriteAll(
    this OutputStream output,
    buffer: ReadOnlyMemory<Byte>
) -> Task<Result<(), StreamError>>;

func CopyTo(
    this InputStream source,
    destination: OutputStream
) -> Task<Result<UInt64, StreamError>>;
```

Everything else can grow around these primitives as real application requirements appear.

---

# 23. Platform architecture

This gives us a rather clean place for Streams in the larger NeoCLR API:

```text
System
├── Collections
├── Concurrency
├── Data
├── Networking ─────────────┐
├── Security               │
├── Storage ────────────┐   │
├── Streams             │   │
├── Tasks               │   │
├── Text                │   │
└── Time                │   │
                        ▼   ▼
                    InputStream
                    OutputStream
                         │
             ┌───────────┼────────────┐
             ▼           ▼            ▼
          buffering   transforms   text/binary
```

The dependency relationship is deliberately not represented through namespace nesting.

`System.Storage` can use `System.Streams`.

`System.Networking` can use `System.Streams`.

`System.Data.Compression` can use `System.Streams`.

`System.Security.Cryptography` can use `System.Streams`.

And all of them can use `System.Tasks`.

---

# 24. Design principle

The resulting model retains what is useful and recognizable about .NET streams:

```text
Read
Write
Flush
Seek
byte-oriented sequential I/O
buffering
composition
memory streams
transform streams
```

but does not inherit the assumption that all of those capabilities must hang from one `Stream` base class.

The NeoCLR principle becomes:

> **A stream represents one direction of sequential byte flow. Additional capabilities are composed explicitly.**

That gives us APIs which still *feel*


---

## MemoryStream demonstrates capability composition

`InputStream` and `OutputStream` describe capabilities. They do not imply that an object must represent only one direction of data flow.

A memory-backed stream naturally supports several capabilities at once:

```raven
class MemoryStream :
    InputStream,
    OutputStream,
    Seekable,
    Sized
{
    ...
}
```

When used as a `MemoryStream`, the complete interface of the concrete type is available:

```raven
let stream = MemoryStream();

await stream.WriteAll(data)?;
await stream.Seek(0, SeekOrigin.Start)?;

let buffer = Memory<Byte>(data.Length);
await stream.ReadExactly(buffer)?;
```

There is no need to choose between a separate input or output object merely because the stream interfaces are directional.

Instead, capabilities are narrowed at API boundaries.

An operation that only consumes bytes should request an `InputStream`:

```raven
func Decode(input: InputStream)
    -> Task<Result<Document, DecodeError>>
{
    ...
}
```

An operation that only produces bytes should request an `OutputStream`:

```raven
func Encode(
    document: Document,
    output: OutputStream
) -> Task<Result<(), EncodeError>>
{
    ...
}
```

The same `MemoryStream` can be passed to both:

```raven
let stream = MemoryStream();

await Encode(document, stream)?;
await stream.Seek(0, SeekOrigin.Start)?;

let decoded = await Decode(stream)?;
```

When passed to `Encode`, the relevant contract is `OutputStream`. When passed to `Decode`, it is `InputStream`. The underlying `MemoryStream` itself still supports both.

An operation that genuinely requires several capabilities can express that requirement explicitly:

```raven
func RewriteHeader(
    stream: InputStream & OutputStream & Seekable
) -> Task<Result<(), RewriteError>>
{
    ...
}
```

This avoids introducing interfaces for every possible combination:

```text
InputStream
OutputStream
InputOutputStream
SeekableInputStream
SeekableOutputStream
SeekableInputOutputStream
...
```

Instead, individual capabilities compose independently.

This is an important distinction between **concrete stream types** and **stream contracts**:

> **Concrete types expose the capabilities they support. API contracts request only the capabilities they require.**

`MemoryStream` therefore remains a useful and familiar concrete type even though NeoCLR has no universal `Stream` base interface.