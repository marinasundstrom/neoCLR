# Storage provider experiment

**Development exploration, not a published System.Storage API.** These
`StorageExperiment` types belong to a tested Raven sample. Their purpose is to
compare provider ownership, file descriptors and path interpretation before
settling the platform API. They are documented here so the experimental contract
is visible; they are not types in the core reference assembly.

Download the tested [project file](/samples/storage-provider/StorageExplorer.rvnproj),
[Storage.rvn](/samples/storage-provider/Storage.rvn),
[Path.rvn](/samples/storage-provider/Path.rvn),
[Streams.rvn](/samples/storage-provider/Streams.rvn),
[ByteRoundTrip.rvn](/samples/storage-provider/ByteRoundTrip.rvn),
[Main.rvn](/samples/storage-provider/Main.rvn) and
[expected output](/samples/storage-provider/expected.txt). The same
`RoundTrip(directory)` function writes and reads `Hello, värld!` using disk and
memory. An oversized replacement is rejected without changing the saved contents.

```raven
match Path.Parse("/") {
    Ok(let path) => {
        let host: StorageProvider = HostStorage("sandbox")
        let disk = Directory(host, path)
        ByteRoundTrip(disk)
    }
    Error(_) => System.Fault("Invalid logical path")
}
```

The text workflow uses existing synchronous UTF-8 helpers. The additional
`ByteRoundTrip(directory)` workflow uses directional [file streams](streams.md) on
disk and two-byte partial transfers in memory. The memory provider uses logical
keys rather than filesystem paths. These are useful comparison cases, not a finished filesystem provider model.

## Build and run the sample

Save the project and all five Raven files in one directory. Select matching neoCLR and
Raven SDK artifacts as described in the [setup guide](/try/index.html):

```sh
dotnet msbuild StorageExplorer.rvnproj -p:NeoCLRRoot=/path/to/neoclr -p:RavenSdkRoot=/path/to/raven-sdk
/path/to/neoclr/bin/neoclr run bin/neoclr/Debug/App.neoil --system bin/neoclr/Debug/System.neoil
```

Create a `sandbox` subdirectory inside a scratch working directory before running.
The disk provider is explicitly rooted there: it creates or replaces
`sandbox/notes.txt` and exclusively creates `sandbox/stream.txt` (a repeat run in the same directory
will reject that existing file). The checked-in verifier uses a disposable directory and also verifies the
actual bytes, error handling and independent provider state.

## StorageProvider

Implementations resolve validated logical Path values in their own namespace. Text methods remain temporary
adapters; the byte workflow opens directional streams and layers UTF-8 conversion
above them.

| Member | Contract |
| --- | --- |
| `FileAt(path: Path, name: string) -> File` | Construct a descriptor retaining a parsed path and display name; does not check existence. Directory.FileAt constructs and validates child paths for ordinary callers. |
| `OpenRead(path: Path) -> Result<InputStream, StreamError>` | Open an existing byte file with a new read cursor at zero, or report an expected error. |
| `CreateNew(path: Path) -> Result<OutputStream, StreamError>` | Exclusively create a byte file. Existing entries are not overwritten. |
| `ReadText(path: Path, maxBytes: int) -> Result<string, FileReadError>` | Read UTF-8 text within a byte limit. Reports expected read errors as values. |
| `WriteText(path: Path, text: string, maxBytes: int) -> Result<unit, FileWriteError>` | Create or replace text within a byte limit. The supplied providers reject an oversized value before modifying storage. |

Path values describe logical syntax; addresses are meaningful only with a provider.
HostStorage maps them under its configured native root; MemorySlotStorage uses
rooted logical string keys. Providers choose resolution rules, not Path parsing.
In this experiment, relative provider operations start at the provider root.
Directory.FileAt instead appends a direct child to that directory's Path.

## Directory

A provider-bound directory address. Construction does not create or verify a directory.

| Member | Contract |
| --- | --- |
| `Directory(provider: StorageProvider, path: Path)` | Retain the provider and its directory address. |
| `Path: Path` | Return the validated logical path without querying storage. |
| `FileAt(name: string) -> Result<File, FileReadError>` | Validate a direct child name, parse the combined logical path, then construct the File descriptor through the retained provider. Empty names, `.`, `..` and names containing separators, colon or NUL are rejected with InvalidPath. |

FileAt resolves an address; it is not the proposal's existence-checking GetFile.
It can describe a file that will be created later. Validation is not a security
boundary or a restriction on the provider's other operations.

## File

A descriptor retaining its provider, address and display name. It owns no open file
handle and has no disposal requirement in this experiment.

| Member | Contract |
| --- | --- |
| `File(provider: StorageProvider, path: Path, name: string)` | Retain the supplied values; normally called by a provider. Does not verify the address or that the name matches it. |
| `Path: Path` | Return the validated logical path. |
| `Name: string` | Return the supplied child name. Does not parse Path with host rules. |
| `OpenRead() -> Result<InputStream, StreamError>` | Ask the retained provider for an input stream. The caller must close it. |
| `CreateNew() -> Result<OutputStream, StreamError>` | Ask the retained provider for a new file output stream. The caller must close it. |
| `ReadText(maxBytes: int) -> Result<string, FileReadError>` | Read through the retained provider. A missing file is reported when read, not when its descriptor is constructed. |
| `WriteText(text: string, maxBytes: int) -> Result<unit, FileWriteError>` | Write through that same provider; never select a provider from ambient state. |

## Supplied providers

`HostStorage(root: string)` configures a native disk root and implements all five
StorageProvider methods using native path combination, the existing whole-file helpers, and FileInputStream/FileOutputStream.
Calls block. Whole-text helpers close per operation; byte streams remain open until
the caller closes them or the invocation ends. Its permissions, symlinks and native path validity follow the host. Logical `/`
means the configured provider root, not the OS filesystem root. A relative root
configuration remains relative to the process working directory; no canonicalization
or sandbox claim is made. There is no sandbox or read-only capability here.

`MemorySlotStorage()` has a text slot and a separate byte slot per instance during
this migration experiment. They are separate stores, even for the same address;
do not mix text helpers and byte streams as though they were a coherent filesystem. Writing another text address replaces the text slot. Reads at a missing address
return NotFound. Negative limits return InvalidLimit; UTF-8 byte counts above the
limit return TooLarge. Rejected writes preserve the previous entry. Distinct
instances do not share state. The byte slot uses a shared mutable list, with a separate cursor on each input.
CreateNew rejects the same existing address with AlreadyExists and a different
second byte-file address with LimitExceeded. Memory streams cap file size at
64 KiB and each successful transfer at two bytes. An open reader sees subsequently
appended bytes; there is no snapshot or concurrency guarantee. It is a test
provider, not a general memory filesystem.

## Directional capability interfaces

These are application-owned interfaces, separate from the core file classes.

| Member | Contract |
| --- | --- |
| `InputStream.Read(buffer: byte[], offset: int, count: int) -> Result<int, StreamError>` | Attempt one synchronous partial read. A positive-count read returning zero is EOF; zero-count does not establish EOF. Only the returned number of elements change. |
| `InputStream.Close()` | Release input; repeated calls are harmless and subsequent reads report Closed. |
| `OutputStream.Write(buffer: byte[], offset: int, count: int) -> Result<int, StreamError>` | Attempt one synchronous partial write; callers loop over the remaining range. |
| `OutputStream.Flush() -> Result<unit, StreamError>` | Report flush success or expected failure; no durability promise. |
| `OutputStream.Close()` | Release output; repeated calls are harmless. Later writes and flushes report Closed. |

Both directions reject negative/out-of-bounds ranges before transfer and permit at
most 64 KiB per request. Close is checked before the range. Buffer ownership remains
with the caller; no operation suspends. The [stream guide](streams.md) details the
host lifetime and error behavior.

## Supplied stream adapters

`DiskInput(stream: FileInputStream)` implements InputStream by forwarding Read and
Close. `DiskOutput(stream: FileOutputStream)` implements OutputStream by forwarding
Write, Flush and Close. The adapter retains the supplied stream and closing either
reference closes the same resource. Providers create these adapters after a successful
open; callers ordinarily obtain the capability through File.

`MemoryInput(bytes: List<byte>)` implements InputStream over the supplied shared list,
starting at position zero. Read copies at most two bytes and advances its own cursor;
Close marks only that reader closed. `MemoryOutput(bytes: List<byte>)` implements
OutputStream by appending at most two bytes per write. A request that would exceed
64 KiB total storage fails before appending. Flush succeeds while open; Close marks
that writer closed. Constructors retain the supplied list, not a copy. These small
adapters exercise the application's progress loops rather than emulate FileStream.

## What the sample checks

The byte workflow uses a three-byte reusable buffer, writes and flushes, closes
before inspecting the result, then reads and closes before decoding. UTF-8 decoding
uses the collected bytes, not individual chunks. This is not an incremental decoder.
It checks exclusive creation, repeated close and Closed outcomes in both backends.
The separate verifier also checks the exact disk bytes and text workflow failures.

## What remains open

Storage now consumes the first validated Path value object described below. The
exact grammar and whether path-taking public APIs should also accept string overloads
remain exploratory. A proposed string overload would parse and forward to the Path
version, preserving the same validation and typed errors; no overload family is
added by this slice.
Eager lookup, item identity, provider-specific File implementations and common
StorageItem contracts remain exploratory. The separate text and byte slots are a
visible limitation of the test provider, not a final storage model. General stream
capabilities, automatic disposal, cleanup across await and async scheduling also
remain design work.

For comparison, .NET's [FileInfo constructor](https://learn.microsoft.com/en-us/dotnet/api/system.io.fileinfo.-ctor?view=net-10.0)
creates a path wrapper. This experiment also retains a storage provider, allowing
the same consumer to use host files or logical memory storage. That indirection
and its identity rules need evaluation alongside the benefit.


## Path value object

**Application-owned exploration, not a replacement for System.Storage.Path.**
`Path.Parse(text: string) -> Result<Path, InvalidPathError>` is the construction API.
The constructor is private and the object exposes no mutation. Ok guarantees the
logical syntax below, not existence, permissions or support by every provider.

| Member | Contract |
| --- | --- |
| `Path.Parse(text: string) -> Result<Path, InvalidPathError>` | Validate without performing I/O. Preserve accepted spelling; return Error for invalid syntax. |
| `Text: string` | The validated spelling, unchanged. |
| `IsAbsolute: bool` | True when the spelling starts with `/`; rooted within a provider namespace. |
| `IsRelative: bool` | The inverse of IsAbsolute; resolution requires provider/directory context. |
| `Equals(other: Path) -> bool` | Ordinal lexical equality. Different objects with identical spelling compare equal; it does not resolve paths or compare filesystem identity. Use this method, not reference equality. |
| `ToString() -> string` | Return the validated spelling. |
| `InvalidPathError()` | Empty error value identifying invalid logical syntax; no filesystem error is implied. |

This first grammar accepts `/` as root, `.` as the current relative location, and
slash-separated nonempty names with an optional leading slash. It rejects empty
input, NUL, backslashes, colon, doubled/trailing separators (except `/`), and dot or
parent segments (except standalone `.`). Unicode and spaces are preserved. It does
not trim, case-fold, normalize Unicode, collapse segments, query storage or enforce
every host's filename restrictions. Windows drive/UNC/device spellings are outside
this experimental grammar; native root configuration remains provider-specific.

The conservative grammar makes one shared sample reproducible across providers but
excludes useful native paths and traversal forms. It is provisional, not a claim
that all storage should permanently use these restrictions. Symlinks and native
host behavior can still resolve outside the configured root; this is not a security
boundary. Absolute and relative spellings may resolve to the same item while
remaining unequal Path values.

Path is currently an immutable reference class with value comparison, avoiding a
struct's invalid default value. That costs an allocation and does not yet supply
value operators or a hashing/Equatable contract. Additional operations are deferred;
Directory owns direct-child construction and calls Parse to validate its result.
`RequirePath(text)` in the sample is a small helper for known fixture paths: it calls
Parse and terminates with UserFault on invalid syntax. It is not a second parser.

.NET's [System.IO.Path](https://learn.microsoft.com/en-us/dotnet/api/system.io.path?view=net-10.0)
is a static string API with platform-dependent rules.
[GetFullPath](https://learn.microsoft.com/en-us/dotnet/api/system.io.path.getfullpath?view=net-10.0)
resolves native full-path spellings. This experiment separates validated logical
syntax from provider resolution. The benefit is reuse of a checked value; costs
include the new type, narrower grammar and explicit parsing at call sites.
