# Storage provider experiment

**Development exploration, not a published System.Storage API.** These
`StorageExperiment` types belong to a tested Raven sample. Their purpose is to
compare provider ownership, file descriptors and path interpretation before
settling the platform API. They are documented here so the experimental contract
is visible; they are not types in the core reference assembly.

Download the tested [project file](/samples/storage-provider/StorageExplorer.rvnproj),
[Storage.rvn](/samples/storage-provider/Storage.rvn),
[Streams.rvn](/samples/storage-provider/Streams.rvn),
[ByteRoundTrip.rvn](/samples/storage-provider/ByteRoundTrip.rvn),
[Main.rvn](/samples/storage-provider/Main.rvn) and
[expected output](/samples/storage-provider/expected.txt). The same
`RoundTrip(directory)` function writes and reads `Hello, värld!` using disk and
memory. An oversized replacement is rejected without changing the saved contents.

```raven
let host: StorageProvider = HostStorage()
let memory: StorageProvider = MemorySlotStorage()
let disk = Directory(host, ".")
let temporary = Directory(memory, ".")
```

The text workflow uses existing synchronous UTF-8 helpers. The additional
`ByteRoundTrip(directory)` workflow uses directional [file streams](streams.md) on
disk and two-byte partial transfers in memory. The memory provider uses logical
keys rather than filesystem paths. These are useful comparison cases, not a finished filesystem provider model.

## Build and run the sample

Save the project and all four Raven files in one directory. Select matching neoCLR and
Raven SDK artifacts as described in the [setup guide](/try/index.html):

```sh
dotnet msbuild StorageExplorer.rvnproj -p:NeoCLRRoot=/path/to/neoclr -p:RavenSdkRoot=/path/to/raven-sdk
/path/to/neoclr/bin/neoclr run bin/neoclr/Debug/App.neoil --system bin/neoclr/Debug/System.neoil
```

Run in a scratch working directory: the disk case creates or replaces `notes.txt`
there and exclusively creates `stream.txt` (a repeat run in the same directory
will reject that existing file). The checked-in verifier uses a disposable directory and also verifies the
actual bytes, error handling and independent provider state.

## StorageProvider

Implementations interpret their own addresses. Text methods remain temporary
adapters; the byte workflow opens directional streams and layers UTF-8 conversion
above them.

| Member | Contract |
| --- | --- |
| `FileAt(directory: string, name: string) -> File` | Construct a descriptor for a child address using the provider's own syntax. Does not check existence. Low-level callers supply a valid child name; use Directory.FileAt for validation. |
| `OpenRead(path: string) -> Result<InputStream, StreamError>` | Open an existing byte file with a new read cursor at zero, or report an expected error. |
| `CreateNew(path: string) -> Result<OutputStream, StreamError>` | Exclusively create a byte file. Existing entries are not overwritten. |
| `ReadText(path: string, maxBytes: int) -> Result<string, FileReadError>` | Read UTF-8 text within a byte limit. Reports expected read errors as values. |
| `WriteText(path: string, text: string, maxBytes: int) -> Result<unit, FileWriteError>` | Create or replace text within a byte limit. The supplied providers reject an oversized value before modifying storage. |

Addresses are meaningful only with their provider. HostStorage uses native path
combination; MemorySlotStorage uses `directory::name`. The general Directory and
File objects do not parse those addresses with filesystem rules.

## Directory

A provider-bound directory address. Construction does not create or verify a directory.

| Member | Contract |
| --- | --- |
| `Directory(provider: StorageProvider, path: string)` | Retain the provider and its directory address. |
| `Path: string` | Return that address without filesystem interpretation. |
| `FileAt(name: string) -> Result<File, FileReadError>` | Validate a direct child name, then ask the retained provider to construct the File descriptor. Empty names, `.`, `..`, `/`, `\` and NUL are rejected with InvalidPath. |

FileAt resolves an address; it is not the proposal's existence-checking GetFile.
It can describe a file that will be created later. Validation is not a security
boundary or a restriction on the provider's other operations.

## File

A descriptor retaining its provider, address and display name. It owns no open file
handle and has no disposal requirement in this experiment.

| Member | Contract |
| --- | --- |
| `File(provider: StorageProvider, path: string, name: string)` | Retain the supplied values; normally called by a provider. Does not verify the address or that the name matches it. |
| `Path: string` | Return the provider-owned address. |
| `Name: string` | Return the supplied child name. Does not parse Path with host rules. |
| `OpenRead() -> Result<InputStream, StreamError>` | Ask the retained provider for an input stream. The caller must close it. |
| `CreateNew() -> Result<OutputStream, StreamError>` | Ask the retained provider for a new file output stream. The caller must close it. |
| `ReadText(maxBytes: int) -> Result<string, FileReadError>` | Read through the retained provider. A missing file is reported when read, not when its descriptor is constructed. |
| `WriteText(text: string, maxBytes: int) -> Result<unit, FileWriteError>` | Write through that same provider; never select a provider from ambient state. |

## Supplied providers

`HostStorage()` implements all five StorageProvider methods using native path
combination, the existing whole-file helpers, and FileInputStream/FileOutputStream.
Calls block. Whole-text helpers close per operation; byte streams remain open until
the caller closes them or the invocation ends. Its permissions, symlinks and
path validity follow the host. There is no sandbox or read-only capability here.

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

The author directs Storage alignment after working stream reads and writes,
including a Path value object. Strings remain temporary provider-owned addresses.
Eager lookup, item identity, provider-specific File implementations and common
StorageItem contracts remain exploratory. The separate text and byte slots are a
visible limitation of the test provider, not a final storage model. General stream
capabilities, automatic disposal, cleanup across await and async scheduling also
remain design work.

For comparison, .NET's [FileInfo constructor](https://learn.microsoft.com/en-us/dotnet/api/system.io.fileinfo.-ctor?view=net-10.0)
creates a path wrapper. This experiment also retains a storage provider, allowing
the same consumer to use host files or logical memory storage. That indirection
and its identity rules need evaluation alongside the benefit.
