# NeoCLR Streams API v1

## 1. Design goals

The API should preserve the useful parts of .NET's stream model:

* familiar `Read`, `Write`, `Flush`, `Seek` concepts;
* streams compose naturally;
* efficient buffer-oriented I/O;
* wrappers for buffering, compression, encryption, text, etc.;
* both memory-backed and OS-backed implementations.

But NeoCLR should remove historical baggage:

* no `Read` + `ReadAsync` duplication;
* no `Write` + `WriteAsync` duplication;
* no `CanRead`, `CanWrite`, `CanSeek`;
* no unsupported-operation exceptions;
* no expected I/O failures represented as exceptions;
* no requirement that every stream expose every capability;
* no artificial asynchronous work just because an API returns `Task`.

The basic rule becomes:

> **Potentially waiting I/O returns `Task<Result<T, E>>`.**

An immediately available operation simply returns an already-completed task.

---

# 2. Namespace

Initially:

```text
System.IO
```

This is broad enough for:

```text
ReadableStream
WritableStream
BufferedReader
BufferedWriter
MemoryStream
TextReader
TextWriter
IOError
SeekOrigin
```

The filesystem is a related but separate domain:

```text
System.IO.FileSystem
```

or potentially another namespace once we settle the overall filesystem proposal.

Streams model **byte transport**.

Filesystems model **files, paths, directories, metadata and storage namespaces**.

---

# 3. There does not need to be a universal `Stream`

I would revise the earlier proposal here.

We don't necessarily gain anything from:

```raven
interface Stream
{
}
```

The meaningful concepts are its capabilities.

The fundamental ones are:

```raven
interface ReadableStream
{
    func Read(buffer: Memory<Byte>)
        -> Task<Result<Size, IOError>>;
}

interface WritableStream
{
    func Write(buffer: ReadOnlyMemory<Byte>)
        -> Task<Result<Size, IOError>>;

    func Flush()
        -> Task<Result<Void, IOError>>;
}
```

Something that supports both simply implements both:

```raven
class SomeStream :
    ReadableStream,
    WritableStream
{
    ...
}
```

This is preferable to:

```raven
stream.CanRead
stream.CanWrite
```

The type itself expresses the capability.

---

# 4. Asynchrony is the default

There is deliberately no:

```raven
ReadAsync(...)
WriteAsync(...)
FlushAsync(...)
```

Instead:

```raven
let count = await stream.Read(buffer)?;
await stream.Write(buffer)?;
await stream.Flush()?;
```

The method name describes the operation.

Its return type describes its completion semantics.

```raven
func Read(...)
    -> Task<Result<Size, IOError>>
```

And importantly, `Task` does **not** mean that a thread must be scheduled or that suspension must occur.

For example, a memory-backed implementation can complete immediately:

```raven
func Read(buffer: Memory<Byte>)
    -> Task<Result<Size, IOError>>
{
    let count = CopyAvailableBytes(buffer);
    return Ok(count);
}
```

A socket implementation may suspend:

```raven
async func Read(buffer: Memory<Byte>)
    -> Task<Result<Size, IOError>>
{
    let count = await runtime.ReadSocket(handle, buffer)?;
    return count;
}
```

This gives us an important distinction:

> `Task<T>` is part of the contract. `async` is an implementation mechanism.

An interface therefore generally declares:

```raven
func Read(...) -> Task<...>;
```

rather than:

```raven
async func Read(...) -> Task<...>;
```

unless Raven ultimately gives `async` some additional contract-level meaning.

---

# 5. `Memory<T>` rather than `Span<T>` at suspension boundaries

The core asynchronous API takes:

```raven
Memory<Byte>
ReadOnlyMemory<Byte>
```

rather than:

```raven
Span<Byte>
ReadOnlySpan<Byte>
```

because a stream operation may outlive the current stack frame while suspended.

Therefore:

```raven
ReadableStream.Read(Memory<Byte>)
WritableStream.Write(ReadOnlyMemory<Byte>)
```

while purely synchronous parsing and memory manipulation can continue using spans.

This creates a useful boundary:

```text
Span        → immediate computation
Memory      → potentially suspended operation
```

without making streams themselves responsible for memory ownership.

---

# 6. Read semantics

The fundamental read operation is:

```raven
func Read(buffer: Memory<Byte>)
    -> Task<Result<Size, IOError>>;
```

It means:

> Read up to `buffer.Length` bytes into the supplied memory.

A successful result of zero means EOF.

```raven
let count = await stream.Read(buffer)?;

if count == 0
{
    // end of stream
}
```

I would **not** use:

```raven
Result<Option<Size>, IOError>
```

here.

Although NeoCLR embraces `Option`, EOF is part of the fundamental streaming protocol and zero is an efficient, well-understood representation of "no more bytes."

`Option` remains appropriate at semantic layers above raw byte I/O.

For example:

```raven
func ReadLine()
    -> Task<Result<Option<String>, IOError>>;
```

Here `None` genuinely means:

> There is no next line.

---

# 7. Partial I/O is fundamental

`Read` does not promise to fill the buffer.

Likewise:

```raven
func Write(buffer: ReadOnlyMemory<Byte>)
    -> Task<Result<Size, IOError>>;
```

may consume fewer bytes than supplied.

This allows the lowest-level API to map cleanly onto OS, device and network behavior.

Higher-level helpers provide the normal ergonomic operations.

```raven
func ReadExactly(
    this ReadableStream stream,
    buffer: Memory<Byte>
) -> Task<Result<Void, IOError>>;
```

and:

```raven
func WriteAll(
    this WritableStream stream,
    buffer: ReadOnlyMemory<Byte>
) -> Task<Result<Void, IOError>>;
```

Usage:

```raven
await stream.ReadExactly(header)?;
await stream.WriteAll(payload)?;
```

Most application code will probably use these rather than implementing partial-I/O loops itself.

---

# 8. Seeking is a separate capability

Not every stream is seekable.

```raven
interface Seekable
{
    func Seek(
        offset: Int64,
        origin: SeekOrigin
    ) -> Task<Result<UInt64, IOError>>;
}
```

With:

```raven
enum SeekOrigin
{
    Start,
    Current,
    End
}
```

This means there is no:

```raven
CanSeek
```

A function requiring seeking asks for it explicitly:

```raven
func ReadIndex(stream: ReadableStream & Seekable)
{
    ...
}
```

depending on the intersection/capability syntax we eventually settle on.

---

# 9. Size is also a capability

Knowing a stream's size isn't the same thing as seeking it.

```raven
interface Sized
{
    func GetLength()
        -> Task<Result<UInt64, IOError>>;
}
```

I'm deliberately leaning toward a method rather than:

```raven
let Length: UInt64;
```

because retrieving length from an external resource may involve I/O and may fail.

An in-memory implementation simply completes immediately.

This follows a broader NeoCLR principle:

> Properties should generally represent cheap, immediately available state rather than hidden potentially-suspending operations.

---

# 10. Position needs similar treatment

I would avoid automatically putting:

```raven
Position
```

onto `Seekable`.

Some streams can conceptually seek without position necessarily being cheap local state.

We could eventually define:

```raven
interface Positioned
{
    func GetPosition()
        -> Task<Result<UInt64, IOError>>;
}
```

But I wouldn't include it in the absolute minimum API until we find concrete use cases.

`Seek(0, Current)` shouldn't become the accidental way of querying it either.

---

# 11. Copying is a standard operation

A very common operation should be built in:

```raven
func CopyTo(
    this ReadableStream source,
    destination: WritableStream
) -> Task<Result<UInt64, IOError>>;
```

Usage:

```raven
let copied = await source.CopyTo(destination)?;
```

Potential options:

```raven
record CopyOptions
{
    BufferSize: Size;
}
```

giving:

```raven
await source.CopyTo(destination, options)?;
```

The implementation can later take advantage of runtime/platform optimizations such as zero-copy transfers when the concrete stream implementations support them.

The abstraction shouldn't prevent that optimization.

---

# 12. Buffering is explicit

Streams aren't implicitly buffered.

Instead:

```raven
let source = BufferedReader(stream);
```

and:

```raven
let destination = BufferedWriter(stream);
```

These expose the same capabilities:

```raven
class BufferedReader : ReadableStream
{
    init(source: ReadableStream);
}

class BufferedWriter : WritableStream
{
    init(destination: WritableStream);
}
```

I prefer separate reader/writer types over a single mode-dependent:

```text
BufferedStream
```

although a bidirectional buffered stream could exist if real use cases justify it.

Again, capabilities make direction explicit.

---

# 13. Transform streams compose

Compression, encryption, checksumming and similar transformations operate on the capability they actually need.

For example:

```raven
let compressed =
    GzipReader(source);

let buffered =
    BufferedReader(compressed);
```

Rather than one class:

```text
GZipStream
```

with construction modes controlling whether it reads or writes, we'd prefer:

```text
GzipReader
GzipWriter
```

Conceptually:

```raven
class GzipReader : ReadableStream
{
    init(source: ReadableStream);
}

class GzipWriter : WritableStream
{
    init(destination: WritableStream);
}
```

This removes a whole category of runtime mode checks.

---

# 14. Memory I/O

We shouldn't force every operation involving bytes through the Stream abstraction.

For immediate memory manipulation, use:

```text
Span<Byte>
ReadOnlySpan<Byte>
Memory<Byte>
ReadOnlyMemory<Byte>
Buffer
```

A parser can therefore be completely synchronous:

```raven
let reader = BinaryReader(data.Span);

let version = reader.Read<UInt16>()?;
let flags = reader.Read<UInt32>()?;
```

No Task is involved because nothing can suspend.

For stream-backed data:

```raven
let buffer = Memory<Byte>(size);

await stream.ReadExactly(buffer)?;

let reader = BinaryReader(buffer.Span);
```

This gives NeoCLR a useful conceptual division:

```text
Memory operations
      ↓
Span / Memory / Buffer

External sequential I/O
      ↓
ReadableStream / WritableStream

Structured interpretation
      ↓
TextReader / BinaryReader / codecs
```

---

# 15. Memory-backed streams still make sense

Sometimes an API requires a `ReadableStream`, while the data already exists in memory.

So we should still provide adapters.

Something approximately like:

```raven
let stream = MemoryReader(data);
```

and:

```raven
let buffer = MemoryBuffer();

await buffer.Write(data)?;
```

Potential types:

```text
MemoryReader
MemoryWriter
MemoryBuffer
```

`MemoryReader` can implement:

```text
ReadableStream
Seekable
Sized
```

and every returned Task can complete synchronously.

The Task abstraction makes that cheap conceptually: asynchronous *compatibility* doesn't imply asynchronous *execution*.

---

# 16. Text is layered above bytes

A stream has no encoding.

It transports bytes.

This fits directly with our NeoCLR text model where `String` is Unicode text and UTF-8 is the platform expectation.

Text decoding happens explicitly above the stream:

```raven
let reader = TextReader(
    stream,
    Encoding.Utf8
);
```

Then:

```raven
let line = await reader.ReadLine()?;

match line
{
    Some(line) => Process(line),
    None       => break
}
```

Likewise:

```raven
let writer = TextWriter(
    stream,
    Encoding.Utf8
);

await writer.WriteLine("Hello 🌍")?;
```

UTF-8 can be the default:

```raven
let reader = TextReader(stream);
```

meaning:

```raven
TextReader(stream, Encoding.Utf8)
```

without making the underlying byte stream encoding-aware.

---

# 17. Binary I/O is also layered

Similarly, binary interpretation isn't a responsibility of streams.

We can later design:

```text
BinaryReader
BinaryWriter
ByteOrder
VarInt
...
```

around spans and/or streams.

For example:

```raven
let reader = BinaryReader(
    data,
    ByteOrder.LittleEndian
);

let version = reader.Read<UInt16>()?;
```

But binary serialization deserves its own proposal rather than being stuffed into the Stream API.

---

# 18. Errors are values

Expected I/O failures use `Result`.

```raven
await stream.Read(buffer)?
```

rather than exception-based control flow.

An initial error family could look approximately like:

```raven
enum IOError
{
    Closed,
    Interrupted,
    TimedOut,
    Cancelled,

    PermissionDenied,

    InvalidPosition,

    Device(IOErrorCode),
    Other(IOErrorCode)
}
```

But I'd keep this provisional.

In particular, filesystem-specific failures belong to the filesystem domain:

```raven
FileSystem.OpenRead(path)
    -> Task<Result<ReadableStream, FileError>>
```

Opening a nonexistent path is a filesystem failure.

Failure while consuming the already-open byte stream is an I/O failure.

That distinction prevents `IOError` from becoming another enormous `IOException` bucket.

---

# 19. Cancellation should integrate with Task

This is an area where NeoCLR shouldn't automatically copy .NET.

Rather than:

```raven
stream.Read(buffer, cancellationToken)
```

everywhere, the Task model should ideally support structured cancellation.

Then:

```raven
await stream.Read(buffer)
```

executes within the current task context.

Cancelling that task propagates cancellation to the suspended I/O operation where supported.

That could eliminate the enormous amount of:

```text
CancellationToken cancellationToken = default
```

API surface that modern .NET needs.

Explicit cancellation scopes could still exist:

```raven
using scope = CancellationScope(timeout: 30.seconds);

await stream.Read(buffer);
```

The exact syntax belongs to the Task/concurrency proposal, but **Streams should be designed assuming structured cancellation is available**.

---

# 20. Timeouts should probably follow the same model

Likewise, I'm skeptical of:

```text
ReadTimeout
WriteTimeout
```

being properties of every stream.

Timeouts are really execution policy.

Something conceptually like:

```raven
await Task.Timeout(
    stream.Read(buffer),
    30.seconds
)?;
```

or a cancellation scope is more composable than modifying stream state.

That also works consistently across:

```text
filesystem
network
HTTP
database
process I/O
```

instead of each subsystem inventing its own timeout model.

---

# 21. Resource lifetime

Streams represent resources, so they need deterministic lifetime management.

Exactly how:

```raven
using stream = ...
```

works should follow NeoCLR's general resource/disposal model rather than inventing stream-specific lifetime management.

I would therefore **not** put:

```raven
Close()
```

on every stream merely because .NET does.

The resource protocol should handle this.

Something like:

```raven
using stream = await fileSystem.OpenRead(path)?;

await Process(stream);
```

should release the underlying resource when the scope ends.

This also needs to account for async cleanup where necessary.

---

# 22. Filesystem integration

The filesystem API can expose capabilities rather than concrete `FileStream` classes.

```raven
interface FileSystem
{
    func OpenRead(path: Path)
        -> Task<Result<ReadableStream, FileError>>;

    func OpenWrite(
        path: Path,
        options: WriteOptions = default
    ) -> Task<Result<WritableStream, FileError>>;
}
```

Then:

```raven
let source =
    await fileSystem.OpenRead(sourcePath)?;

let destination =
    await fileSystem.OpenWrite(destinationPath)?;

await source.CopyTo(destination)?;
```

A memory filesystem can implement exactly the same interface with immediately completed Tasks.

A remote filesystem can genuinely suspend.

The consumer doesn't care.

This is one of the strongest reasons for making asynchronous I/O the platform default.

---

# 23. Network integration

The same abstraction naturally fits the Network API we're designing.

A TCP connection might expose:

```raven
interface Connection :
    ReadableStream,
    WritableStream
{
    ...
}
```

Then higher-level protocols can consume stream capabilities rather than know anything about sockets:

```text
Socket
  ↓
TCP Connection
  ↓
BufferedReader/Writer
  ↓
TLS
  ↓
HTTP
```

This gives us the same composability that streams traditionally provide without tying the abstraction to files.

---

# 24. Proposed core surface

So the first cut could actually be remarkably small:

```raven
namespace System.IO;

interface ReadableStream
{
    func Read(buffer: Memory<Byte>)
        -> Task<Result<Size, IOError>>;
}

interface WritableStream
{
    func Write(buffer: ReadOnlyMemory<Byte>)
        -> Task<Result<Size, IOError>>;

    func Flush()
        -> Task<Result<Void, IOError>>;
}

interface Seekable
{
    func Seek(offset: Int64, origin: SeekOrigin)
        -> Task<Result<UInt64, IOError>>;
}

interface Sized
{
    func GetLength()
        -> Task<Result<UInt64, IOError>>;
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
    this ReadableStream stream,
    buffer: Memory<Byte>
) -> Task<Result<Void, IOError>>;

func WriteAll(
    this WritableStream stream,
    buffer: ReadOnlyMemory<Byte>
) -> Task<Result<Void, IOError>>;

func CopyTo(
    this ReadableStream source,
    destination: WritableStream
) -> Task<Result<UInt64, IOError>>;
```

And implementations/adapters initially:

```text
MemoryReader
MemoryBuffer

BufferedReader
BufferedWriter

TextReader
TextWriter
```

with compression, crypto, binary I/O, etc. layered independently.

---

# 25. The overall model

The architecture then becomes:

```text
                 ┌───────────────────────┐
                 │   Files / Sockets /   │
                 │ Devices / Processes   │
                 └
```
