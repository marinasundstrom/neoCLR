> **Development status, 2026-09-23:** the Storage POC implements byte streams,
> TextReader/StreamReader and optional SeekableStream in **System.IO**. The original
> proposal below remains exploratory, including its older System.Streams namespace.
> See the [current API guide](../../api-docs/streams.md) for the implemented surface.

Here is the revised proposal, keeping the directional model but replacing the overly generic `Seekable` with the stream-specific `SeekableStream`. I’ve also made the `&` composition model explicit and kept `RandomAccessStream` as a possible nominal alternative rather than another mandatory abstraction. This revises the earlier proposal’s `Seekable` sections while retaining its core principle that API contracts request only the capabilities they need. :chatgpt-content-reference{index="0"} :chatgpt-content-reference{index="1"}

# NeoCLR Streams API

## 1. Purpose

Streams represent **sequential byte flow**.

They are not inherently files, network connections, memory buffers, or devices. Those systems may expose streams as a common way of reading or writing bytes.

NeoCLR therefore treats streams as a small independent platform abstraction:

```text
System.Streams
```

The API follows familiar .NET stream concepts while adapting them to NeoCLR's type system, Task model, explicit error model, and capability-oriented API design.

The major differences from .NET are:

```text
.NET Stream                     NeoCLR

Stream                          directional stream contracts
CanRead                         InputStream type
CanWrite                        OutputStream type
CanSeek                         SeekableStream type
Read / ReadAsync                Read
Write / WriteAsync              Write
exceptions                      Result<T, E>
CancellationToken parameters    Task cancellation
System.IO ownership             System.Streams
```

The central principle is:

> **Direction and stream capabilities are expressed by types rather than runtime state.**

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

# 3. Stream contracts

NeoCLR does not begin with a universal .NET-style `Stream` containing every possible stream operation.

The fundamental concepts are instead contracts describing what can be done with a stream.

The two basic directional contracts are:

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

This prevents APIs from unnecessarily requesting capabilities they do not use.

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

# 9. Seekable streams

Some streams have a current position that can be changed.

This capability is represented by:

```raven
interface SeekableStream
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

The name is deliberately `SeekableStream`, rather than simply `Seekable`.

`Seekable` would describe a very general concept that could apply to cursors, media timelines, iterators, storage devices, and other unrelated abstractions.

`SeekableStream` states what is actually being described:

> **A stream whose current byte position can be moved.**

A seekable input stream therefore implements:

```text
InputStream
SeekableStream
```

while a seekable bidirectional stream may implement:

```text
InputStream
OutputStream
SeekableStream
```

A TCP connection, by contrast, might implement only:

```text
InputStream
OutputStream
```

Seeking remains a stream capability, but it is not assumed merely because something can transfer bytes.

---

# 10. Capability composition

NeoCLR should avoid defining nominal interfaces for every meaningful combination of stream capabilities.

Instead, Raven's intersection/composition syntax can express requirements directly.

An API requiring readable sequential input asks for:

```raven
InputStream
```

An API requiring readable and seekable input asks for:

```raven
InputStream & SeekableStream
```

For example:

```raven
func ReadIndex(
    stream: InputStream & SeekableStream
) -> Task<Result<Index, IndexError>>
{
    ...
}
```

An operation that needs bidirectional access and seeking can request:

```raven
func RewriteHeader(
    stream:
        InputStream &
        OutputStream &
        SeekableStream
) -> Task<Result<(), RewriteError>>
{
    ...
}
```

This avoids constructing a hierarchy such as:

```text
InputStream
OutputStream
InputOutputStream

SeekableInputStream
SeekableOutputStream
SeekableInputOutputStream
```

The individual concepts receive names.

Their combinations generally do not.

The design rule is:

> **Name semantic stream capabilities. Compose combinations structurally.**

A nominal combination should be introduced only when the combination itself carries semantics beyond the capabilities from which it is composed.

---

# 11. RandomAccessStream as a nominal alternative

There is another reasonable way to model seekable bidirectional streams.

A platform could introduce:

```raven
interface RandomAccessStream :
    InputStream,
    OutputStream
{
    func Seek(
        offset: Int64,
        origin: SeekOrigin
    ) -> Task<Result<UInt64, StreamError>>;
}
```

This resembles the approach taken by Windows Runtime, where random-access streams group input, output, and positioning into a nominal abstraction.

Such an API gives a convenient name to a common concept:

```raven
func RewriteHeader(stream: RandomAccessStream)
```

rather than:

```raven
func RewriteHeader(
    stream:
        InputStream &
        OutputStream &
        SeekableStream
)
```

There is nothing inherently wrong with that model.

However, it makes a stronger statement:

```text
RandomAccessStream
        =
InputStream
+ OutputStream
+ positioning
```

That may be unnecessarily restrictive.

A resource could reasonably support:

```text
InputStream
SeekableStream
```

without supporting output.

Likewise:

```text
OutputStream
SeekableStream
```

could be meaningful independently.

The compositional NeoCLR model therefore remains more general:

```text
InputStream
OutputStream
SeekableStream
```

with:

```raven
InputStream & SeekableStream
```

or:

```raven
InputStream & OutputStream & SeekableStream
```

used where necessary.

`RandomAccessStream` remains a possible nominal convenience if experience shows that the combination occurs frequently enough to deserve its own semantic name.

It should not be required by the fundamental stream model.

---

# 12. Size is independent

Knowing the total size of a stream is not necessarily equivalent to being able to seek through it.

Length can therefore remain independently representable.

A stream-specific contract could be:

```raven
interface SizedStream
{
    func GetLength()
        -> Task<Result<UInt64, StreamError>>;
}
```

However, this abstraction should be treated more cautiously than `SeekableStream`.

Length often belongs conceptually to the underlying resource:

```text
File
Memory buffer
Storage object
HTTP representation
```

rather than to sequential byte flow itself.

For example, a file can have a known size before a stream is opened.

Conversely, an input stream may have no meaningful known total length.

The initial Streams API therefore does not necessarily need `SizedStream`.

Where size belongs to the underlying resource, that resource should expose it.

If practical stream APIs repeatedly need length independently of their backing resources, `SizedStream` can be introduced later.

---

# 13. Copying between directions

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

# 14. Directional wrappers

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

or, if reader/writer terminology is preferred at that layer:

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

# 15. Transform streams

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

Likewise, encryption, checksumming, encoding, and other transformations can expose precisely the direction they implement.

---

# 16. Streams and memory are different abstractions

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

# 17. MemoryStream demonstrates composition

A concrete type is not limited to a single stream contract.

A memory-backed stream naturally supports several capabilities:

```raven
class MemoryStream :
    InputStream,
    OutputStream,
    SeekableStream
{
    ...
}
```

When used as a `MemoryStream`, its complete interface remains available:

```raven
let stream = MemoryStream();

await stream.WriteAll(data)?;
await stream.Seek(0, SeekOrigin.Start)?;

let buffer = Memory<Byte>(data.Length);
await stream.ReadExactly(buffer)?;
```

There is no need to create separate input and output objects merely because the contracts themselves are directional.

Capabilities are narrowed at API boundaries.

For example:

```raven
func Decode(input: InputStream)
    -> Task<Result<Document, DecodeError>>
{
    ...
}
```

and:

```raven
func Encode(
    document: Document,
    output: OutputStream
) -> Task<Result<(), EncodeError>>
{
    ...
}
```

The same `MemoryStream` can satisfy both:

```raven
let stream = MemoryStream();

await Encode(document, stream)?;
await stream.Seek(0, SeekOrigin.Start)?;

let decoded = await Decode(stream)?;
```

When passed to `Encode`, the relevant contract is `OutputStream`.

When passed to `Decode`, it is `InputStream`.

When code needs to reposition it, the relevant additional contract is `SeekableStream`.

> **Concrete stream types expose the capabilities they support. API contracts request only the capabilities they require.**

---

# 18. Storage integration

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

Storage remains responsible for:

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

while `System.Streams` takes over once byte transfer begins.

```text
System.Storage
      │
      │ OpenRead()
      ▼
System.Streams.InputStream
```

A storage error opening a nonexistent file is not necessarily the same thing as an error encountered while consuming an already-open stream.

If Storage needs to expose a stream that can be repositioned, its contract may return an intersection:

```raven
func OpenRead(...)
    -> Task<
        Result<
            InputStream & SeekableStream,
            FileError
        >
    >;
```

Whether Raven permits intersection types directly as returned values is ultimately a language/type-system question, but the stream model itself does not require a new nominal interface merely to express the combination.

---

# 19. Networking integration

Networking uses the same directional contracts.

A TCP connection could expose both directions:

```raven
interface Connection :
    InputStream,
    OutputStream
{
    ...
}
```

Higher-level protocols can then operate on stream contracts without depending on sockets:

```text
Socket / Connection
        ↓
InputStream + OutputStream
        ↓
buffering / TLS
        ↓
HTTP
```

A network connection normally does not implement `SeekableStream`, because its byte position cannot arbitrarily be repositioned.

This illustrates why seeking is not part of the basic input/output contracts.

---

# 20. Text is layered above streams

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

Whether `TextReader` and `TextWriter` ultimately belong in `System.Text`, `System.Streams`, or a more specialized namespace can be decided with the Text API.

---

# 21. Errors

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

The precise error taxonomy can evolve as the underlying platform abstractions become clearer.

The important distinction is that **cancellation is not a `StreamError`**.

With the NeoCLR Task model:

```raven
Task<Result<Size, StreamError>>
```

can conceptually complete as:

```text
Completed(Ok(size))
Completed(Err(streamError))
Cancelled
```

Cancellation belongs to Task execution rather than being reported as an I/O failure.

Likewise, domain-specific errors should remain with their originating domain.

Failure to locate or open a file belongs to Storage.

Failure while reading from an already-open byte source belongs to the stream operation.

This keeps the abstraction boundary clear:

```text
Storage operation
    ↓
Result<Stream, FileError>

Stream operation
    ↓
Result<T, StreamError>
```

---

# 22. Cancellation and timeouts

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

participates in the surrounding Task cancellation context.

If the operation is suspended in an operating-system or provider operation, cancellation should propagate to that operation where supported.

This applies equally to:

```text
Read
Write
Flush
Seek
```

There is no stream-specific cancellation mechanism.

Timeouts should similarly be treated primarily as execution policy rather than mutable properties of every stream.

The same Task mechanisms can consequently apply consistently to:

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

# 23. Resource lifetime

Some stream implementations represent resources requiring deterministic cleanup.

Examples include:

```text
files
sockets
pipes
device handles
compressed streams
cryptographic transforms
```

Streams should use NeoCLR's general resource-lifetime mechanism rather than inventing a stream-specific `Close()` protocol.

Conceptually:

```raven
using input = await file.OpenRead()?;

await Process(input);
```

should release the underlying resource at the end of its lifetime.

The fact that an object implements:

```text
InputStream
```

does not itself imply ownership of a native resource.

For example:

```text
MemoryStream
```

may require no meaningful cleanup, while:

```text
FileStream
```

may own an operating-system handle.

Resource lifetime is therefore orthogonal to the stream contracts.

The general NeoCLR resource model must eventually account for resources requiring asynchronous cleanup.

That remains outside the core Streams proposal.

---

# 24. Stream implementations

The contracts describe capabilities rather than prescribing concrete stream types.

A platform may provide types such as:

```text
FileStream
MemoryStream
PipeStream
NetworkStream
```

along with higher-level wrappers and transformations.

Their contracts depend on what the implementation actually supports.

For example:

```text
MemoryStream

    InputStream
    OutputStream
    SeekableStream
```

A file opened read-only might expose:

```text
InputStream
SeekableStream
```

while a writable file could expose:

```text
OutputStream
SeekableStream
```

or:

```text
InputStream
OutputStream
SeekableStream
```

depending on how it was opened.

A TCP connection naturally exposes:

```text
InputStream
OutputStream
```

but not:

```text
SeekableStream
```

A decompression reader may expose only:

```text
InputStream
```

even if its underlying source is seekable.

Capabilities therefore describe the stream object being used, not necessarily every capability possessed by the resource somewhere beneath it.

---

# 25. Capability preservation through wrappers

A wrapper should not automatically expose every capability of its underlying stream.

Consider:

```raven
let file = ...; // InputStream & SeekableStream
let gzip = GzipReader(file);
```

The resulting `GzipReader` is naturally:

```text
InputStream
```

but not necessarily:

```text
SeekableStream
```

Seeking the compressed source does not imply that arbitrary positions in the decompressed byte sequence can be sought correctly.

Likewise, buffering may or may not preserve seeking depending on the implementation.

This gives an important rule:

> **Stream capabilities describe the observable semantics of the current stream layer, not capabilities inherited mechanically from the underlying resource.**

A wrapper explicitly implements `SeekableStream` only when it can provide meaningful seek semantics itself.

This is another reason to keep stream capabilities represented by contracts rather than by mutable flags.

---

# 26. Position

A seekable stream inherently has the concept of a current byte position.

There are several possible ways to expose it.

One possibility is a property:

```raven
interface SeekableStream
{
    Position: UInt64;

    func Seek(
        offset: Int64,
        origin: SeekOrigin
    ) -> Task<Result<UInt64, StreamError>>;
}
```

However, NeoCLR generally avoids properties that may hide I/O, suspension, or failure.

If obtaining the current position is guaranteed to be immediately available for every `SeekableStream`, a property is appropriate.

Otherwise, position should remain an operation:

```raven
func GetPosition()
    -> Task<Result<UInt64, StreamError>>;
```

There is also a simpler possibility: `Seek` already returns the resulting absolute position.

For example:

```raven
let position =
    await stream.Seek(128, SeekOrigin.Current)?;
```

The initial API therefore does not necessarily need a separate position member.

It should be added only if concrete use cases demonstrate that querying position independently is sufficiently common.

---

# 27. Seeking and random access

`SeekableStream` describes a stream whose current position can be moved.

That should not automatically be generalized into a larger random-access abstraction.

The basic operation remains:

```raven
await stream.Seek(position, SeekOrigin.Start)?;
await stream.Read(buffer)?;
```

Some systems can provide stronger operations, such as obtaining independent stream cursors at arbitrary positions:

```text
GetInputAt(position)
GetOutputAt(position)
```

Those semantics are useful, but they are stronger than ordinary seeking.

They imply something closer to access to an underlying random-access resource than merely movement of the current stream position.

NeoCLR therefore does not initially require such operations as part of `SeekableStream`.

A future API may introduce them where appropriate.

For example, a storage resource could potentially provide:

```raven
func OpenReadAt(position: UInt64)
    -> Task<Result<InputStream, FileError>>;
```

without changing the fundamental stream model.

This preserves an important distinction:

```text
SeekableStream
    ↓
move this stream's cursor

Random-access resource
    ↓
access arbitrary positions,
possibly through independent cursors
```

The latter does not need to be modeled until an actual platform API requires it.

---

# 28. Nominal versus structural composition

The stream design deliberately leaves room for both nominal and structural modeling.

The primitive contracts are nominal:

```text
InputStream
OutputStream
SeekableStream
```

But combinations can be expressed structurally:

```raven
InputStream & SeekableStream
```

and:

```raven
InputStream &
OutputStream &
SeekableStream
```

This is particularly attractive because the combinations themselves generally introduce no new semantics.

For example:

```raven
func ReadTable(
    source: InputStream & SeekableStream
)
```

says exactly what the operation requires.

There is little gained by introducing:

```raven
interface SeekableInputStream :
    InputStream,
    SeekableStream
{
}
```

merely to give the intersection a name.

The same applies to:

```text
InputOutputStream
SeekableOutputStream
SeekableInputOutputStream
```

Such interfaces would largely encode combinations rather than concepts.

The default NeoCLR rule should therefore be:

> **Nominal types describe concepts. Intersection types describe combinations of capabilities.**

---

# 29. When a nominal combination may still make sense

Structural composition does not mean that NeoCLR must reject every combined interface.

A combined interface is justified when its name represents a useful semantic abstraction of its own.

`RandomAccessStream` is one possible example.

It could conceptually be defined as:

```raven
interface RandomAccessStream :
    InputStream,
    OutputStream,
    SeekableStream
{
}
```

An API could then request:

```raven
func UpdateContainer(stream: RandomAccessStream)
```

rather than:

```raven
func UpdateContainer(
    stream:
        InputStream &
        OutputStream &
        SeekableStream
)
```

Whether this is worthwhile depends on what `RandomAccessStream` means.

If it is merely shorthand for three interfaces, the intersection is preferable.

If it establishes additional guarantees or operations associated with random-access semantics, then it may deserve a nominal type.

The distinction is:

```text
mere combination
    ↓
A & B & C

semantic abstraction
    ↓
NamedInterface
```

NeoCLR should therefore not introduce `RandomAccessStream` merely because other platforms have such an abstraction.

It remains a legitimate design alternative if concrete requirements justify it.

---

# 30. Initial core surface

The first version can remain deliberately small:

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

interface SeekableStream
{
    func Seek(
        offset: Int64,
        origin: SeekOrigin
    ) -> Task<Result<UInt64, StreamError>>;
}

enum SeekOrigin
{
    Start,
    Current,
    End
}
```

Standard operations can be provided on top:

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

In particular, the initial API does not need to settle immediately on abstractions for:

```text
stream length
independent stream cursors
random-access resources
specialized file streams
buffer ownership
zero-copy transfer
```

Those can be introduced without disturbing the fundamental directional contracts.

---

# 31. Platform architecture

This gives Streams a small and well-defined place in the larger NeoCLR API:

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
               ┌─────────┼─────────┐
               ▼         ▼         ▼
           buffering  transforms  text/binary
```

`SeekableStream` is another contract within `System.Streams`, used only where positioning semantics exist.

The dependency relationship is deliberately not represented through namespace nesting.

`System.Storage` can use `System.Streams`.

`System.Networking` can use `System.Streams`.

`System.Data.Compression` can use `System.Streams`.

`System.Security.Cryptography` can use `System.Streams`.

And all of them can use `System.Tasks`.

---

# 32. Design principles

The resulting model retains what is useful and recognizable about conventional stream APIs:

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

while avoiding the assumption that every operation must hang from one universal `Stream` base class.

The fundamental directional contracts remain:

```text
InputStream
OutputStream
```

Seeking adds:

```text
SeekableStream
```

when applicable.

Concrete types can implement any meaningful combination:

```raven
class MemoryStream :
    InputStream,
    OutputStream,
    SeekableStream
{
}
```

while APIs request only what they actually require:

```raven
InputStream

InputStream & SeekableStream

InputStream & OutputStream

InputStream & OutputStream & SeekableStream
```

This leads to four related principles:

> **A stream represents sequential byte flow.**

> **Direction is expressed through `InputStream` and `OutputStream`.**

> **Additional stream semantics receive stream-specific contracts such as `SeekableStream`.**

> **Combinations of capabilities are normally expressed structurally rather than by inventing interfaces for every combination.**

The distinction between concrete types and contracts is therefore central:

> **Concrete stream types expose the capabilities they support. API contracts request only the capabilities they require.**

This gives NeoCLR a stream model that remains recognizable to .NET developers while making capability requirements explicit in the type system and leaving room for future abstractions where actual platform requirements justify them.

## Minimal text reader for the Storage POC — 2026-09-23

**Exploration, not an implemented API.** The author suggested a StreamReader class
and then specified that there will also be a TextReader interface. StreamReader
will implement that consumer-facing contract over InputStream,
so the Storage POC can demonstrate text file access without making every consumer
assemble byte chunks and decode them. The assistant proposes a small UTF-8 reader
above InputStream, usable with disk and memory alike. Consumers should request
TextReader when they need text, without requiring a byte-stream-backed implementation.
This permits future string-backed readers without making them part of this POC.
Neither the interface nor StreamReader should resolve paths
or depend on a Storage provider.

### Comparison and provisional scope

Primary references checked 2026-09-23: [.NET 10 StreamReader constructors](https://learn.microsoft.com/en-us/dotnet/api/system.io.streamreader.-ctor?view=net-10.0)
accept streams, offer encoding/BOM and leaveOpen options, and normally dispose the
wrapped stream. [ReadToEnd](https://learn.microsoft.com/en-us/dotnet/api/system.io.streamreader.readtoend?view=net-10.0)
returns the remaining text as one string. Those library policies are separate from
CLR stream or scheduler mechanisms.

Start by evaluating bounded whole-text reading over the existing InputStream and
strict Utf8 decoder. This can replace the sample's ReadMessage + Utf8.Decode glue
without introducing native entry points. A helper function would be the smallest alternative, but the author selected a
TextReader interface as part of the model. That separates the text contract from
byte decoding. StreamReader provides lifetime state and a place for later buffered
line reading, at the cost of another object, interface dispatch and ownership rules. Broad .NET encoding,
path-constructor and async overload parity is not needed for this POC.

Compared with the [.NET TextReader abstract base class](https://learn.microsoft.com/en-us/dotnet/api/system.io.textreader?view=net-10.0),
neoCLR's selected TextReader is an interface: implementations provide behavior
without inheriting a common class. Shared implementation, default operations and
the exact method set are not selected yet. Do not copy the full .NET member set.

Before implementing, settle the minimal TextReader members, constructor/namespace and these observable contracts:

- Maximum input-byte bound, including how to distinguish exact-bound EOF from
  overflow and whether an overflow probe consumes a byte.
- Ownership of the wrapped stream, close behavior and use after close; do not
  silently change ownership on a failed read.
- Typed distinction between underlying I/O failure, malformed UTF-8 and size limits.
- BOM policy, empty text and multi-byte characters split across partial reads.
- Remaining-stream semantics on repeated reads and after an error.

Tests should run the same reader against disk and a short-read memory stream,
including split multi-byte text, EOF, exact/over bounds, invalid UTF-8, propagated
I/O failure and close ownership. Bounded whole-text reading retains bytes and a
result string, so it is not constant-memory streaming. ReadLine/incremental decoding,
other encodings and async operation remain follow-ups unless the POC requires them.


## POC seekability question — 2026-09-23

**Author question:** should a provisional ReadOnlyAccessStream interface allow
seeking? **Assistant recommendation, not an author-selected new API:** retain
InputStream for reading and add the already-proposed SeekableStream capability
when a concrete seek-and-reread sample needs it. ReadOnlyAccessStream does not
communicate the extra positioning guarantee. A nominal SeekableInputStream may
combine the two if the target compiler cannot express the required capability
composition; that support must be validated before selecting consumer signatures.

[.NET 10 Stream.CanSeek](https://learn.microsoft.com/en-us/dotnet/api/system.io.stream.canseek?view=net-10.0)
and [Stream.Seek](https://learn.microsoft.com/en-us/dotnet/api/system.io.stream.seek?view=net-10.0)
(primary references checked 2026-09-23) expose seek support as a runtime property;
unsupported streams can reject positioning operations. The neoCLR alternative
expresses the requirement as an interface, at the cost of additional types and
capability discovery/composition. No new seek interface is implemented in this slice.

Keep this optional for the sequential Storage/text-reading POC. A follow-up should
prove byte-position semantics on disk and memory, seek-and-reread, EOF, invalid
positions, closed streams and unchanged cursor on rejected positions. Select
absolute versus origin-relative positioning and numeric bounds explicitly; length
and resizing are not implied. A buffered StreamReader must not assume that seeking
its underlying stream also resets unread bytes or decoding state. TextReader does
not require byte seekability; reader-aware repositioning remains a separate design.


## Implemented POC contract — 2026-09-23

The development implementation groups streams and readers under System.IO, leaving
providers/items/paths under System.Storage. StreamReader implements TextReader with
ReadToEnd(maxUtf8Bytes) and Close. It performs strict UTF-8 decoding, preserves a BOM,
and bounds whole-text accumulation to at most 64 KiB. The exact-bound EOF probe may
consume one excess byte before LimitExceeded; failures do not rewind. It owns its
input by default, with a leaveOpen constructor overload. Repeated EOF produces empty
text and repeated Close is harmless. TextReadError preserves named stream errors and
adds InvalidUtf8. This is bounded accumulation, not incremental text decoding.

SeekableStream provides GetPosition and absolute Seek in signed 64-bit byte positions.
FileInputStream implements it alongside InputStream; sequential inputs need not.
Negative positions fail without moving the cursor; positions beyond EOF are valid.
The reader has no retained decoder/buffer state between successful ReadToEnd calls;
the POC closes its leave-open reader, seeks and creates a new reader. Length, relative
origins, output seeking, ReadLine and async readers remain outside this slice.

Compared with the .NET contracts cited above, this supplies fewer operations and
encoding choices, explicit bounded failure and separate optional capabilities. The
cost is more interface/error types and explicit capability discovery. The current
Raven target requires an object view to test an unrelated interface capability; no
new language-level intersection types are assumed. See the [standalone sample](../experiments/storage-poc/README.md)
and [disk/memory contract checks](../experiments/storage-provider/ReaderContracts.rvn).
