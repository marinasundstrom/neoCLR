# Storage provider experiment

**Development exploration, not a published System.Storage API.** These
`StorageExperiment` providers, descriptors and memory stream classes belong to a tested Raven sample.
They consume the development System.Storage.Path value and System.Streams.InputStream/OutputStream
interfaces from the platform library. Their purpose is to
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
| `GetFile(path: Path) -> Result<File, StorageLookupError>` | Query a current file and return a provider-bound descriptor; does not retain a stream or guarantee later availability. Root/directory returns WrongKind and missing entries return NotFound. |
| `FileAt(path: Path, name: string) -> File` | Construct a descriptor retaining a parsed path and display name; does not check existence. Directory.FileAt constructs and validates child paths for ordinary callers. |
| `OpenRead(path: Path) -> Result<InputStream, StreamError>` | Open an existing byte file with a new read cursor at zero, or report an expected error. |
| `CreateNew(path: Path) -> Result<OutputStream, StreamError>` | Exclusively create a byte file. Existing entries are not overwritten. |
| `ReadText(path: Path, maxBytes: int) -> Result<string, FileReadError>` | Read UTF-8 text within a byte limit. Reports expected read errors as values. |
| `WriteText(path: Path, text: string, maxBytes: int) -> Result<unit, FileWriteError>` | Create or replace text within a byte limit. The supplied providers reject an oversized value before modifying storage. |

Path values describe logical syntax; addresses are meaningful only with a provider.
HostStorage maps them under its configured native root; MemoryStorage uses
rooted logical string keys. Providers choose resolution rules, not Path parsing.
In this experiment, relative provider operations start at the provider root.
Directory.FileAt instead appends a direct child to that directory's Path.

## Directory

A provider-bound directory address. Construction does not create or verify a directory.

| Member | Contract |
| --- | --- |
| `Directory(provider: StorageProvider, path: Path)` | Retain the provider and its directory address. |
| `Path: Path` | Return the validated logical path without querying storage. |
| `GetFile(relativePath: Path) -> Result<File, StorageLookupError>` | Resolve a relative logical path against this directory and query through its retained provider. Nested segments are accepted. Absolute values return InvalidPath without querying; `.` queries this directory address as a file and normally returns WrongKind. |
| `GetFile(name: string) -> Result<File, StorageLookupError>` | Validate a direct child name, then query the provider. Invalid names return InvalidPath; provider lookup errors are preserved. |
| `FileAt(name: string) -> Result<File, FileReadError>` | Validate a direct child name, parse the combined logical path, then construct the File descriptor through the retained provider. Empty names, `.`, `..` and names containing separators, colon or NUL are rejected with InvalidPath. |

FileAt constructs an address without I/O and can describe a file to create later.
GetFile queries its current kind/existence. Validation is not a security
boundary or a restriction on the provider's other operations.

### Relative paths and direct child names

The Path overload permits nested relative paths such as `nested/note.txt`. The
string overload remains a direct-child-name operation: that same slash-containing
string returns InvalidPath. This is a provisional distinction from the Storage
proposal, not a general policy that string-taking APIs must accept only names.
An absolute Path is valid syntax but invalid for this operation; use a provider
lookup to resolve an absolute logical address.

Resolution preserves the directory's provider and logical prefix. Root `/`, current
relative `.` and relative directory addresses are handled explicitly. It does not
canonicalize, check parent directories, follow links itself or establish a sandbox.
The Path parser already rejects parent traversal segments. Directory construction
still does not establish that its address exists or is a directory; a lookup delegates
the final address to its provider. MemoryStorage still has flat keys rather than a
directory tree, so parent existence and non-root directory kinds are not yet shared
contracts. Returned File.Path is the combined spelling; Name is the final component.

Compared with [.NET Path.Combine](https://learn.microsoft.com/en-us/dotnet/api/system.io.path.combine?view=net-10.0),
which discards earlier components when a later component is rooted, this operation
rejects absolute input to preserve its selected directory context. This extra
restriction is useful for explicit resolution but is not a containment guarantee.
It also makes the two overloads less interchangeable; convenience parsing and shared
error design remain open. Comparison reviewed 2026-09-23.

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

`HostStorage(root: string)` configures a native disk root and implements all six
StorageProvider methods using native path combination, [Metadata.GetKind](xref:System.Storage.Metadata), the existing whole-file helpers, and FileInputStream/FileOutputStream.
Calls block. Whole-text helpers close per operation; byte streams remain open until
the caller closes them or the invocation ends. Its permissions, symlinks and native path validity follow the host. Logical `/`
means the configured provider root, not the OS filesystem root. A relative root
configuration remains relative to the process working directory; no canonicalization
or sandbox claim is made. There is no sandbox or read-only capability here.

`MemoryStorage()` replaces the earlier experimental MemorySlotStorage. It implements
all six StorageProvider methods using one byte payload per logical address; text
reads decode that payload and text writes encode UTF-8 without a BOM. Files written
through either API are visible through the other. Instances do not share state.
Relative and absolute spellings with the same names resolve to the same entry.
Logical `/` and `.` denote the root and cannot be opened or written as files.
GetFile checks this same address table without opening a stream. Missing entries
return NotFound and the root returns WrongKind.
Nested names are flat keys; this is not a directory tree or a general filesystem.

The store permits eight addresses, each with at most 64 KiB of current contents.
CreateNew rejects an existing address with AlreadyExists and a new address at
capacity with LimitExceeded. Text writes reject negative limits with InvalidLimit,
root paths with NotRegularFile, and contents beyond the caller limit or 64 KiB with
TooLarge. At address capacity, a new text address returns WriteFailed because the
temporary FileWriteError family lacks a capacity case. Existing addresses remain
writable. Rejected writes preserve their previous contents and publish no new entry.

ReadText checks negative limits, root kind, existence and byte count before strict
UTF-8 decoding. It reports InvalidLimit, NotRegularFile, NotFound, TooLarge or
InvalidUtf8 respectively. OpenRead reports WrongKind for the root and NotFound for
missing addresses. Stream methods retain the buffer/error rules below.

Each successful text write publishes a fresh byte payload. Already-open memory
inputs and outputs retain the previous payload; later reads/opens use the replacement.
Within one payload, readers have separate cursors and observe appended bytes.
This is a provisional memory-provider replacement policy, not a snapshot guarantee
or a common disk/memory contract. Old streams can keep old payloads alive; the limits
are not a bound on total retained VM memory. No concurrency guarantee is provided.
Streams still transfer at most two bytes per call to exercise partial-transfer loops.

## Directional capability interfaces

These are now platform interfaces in System.Streams: [InputStream](xref:System.Streams.InputStream) and [OutputStream](xref:System.Streams.OutputStream). File and Directory remain application-owned experiments.

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

## Supplied stream implementations

The core file stream classes now implement these interfaces directly. The sample's
former DiskInput and DiskOutput forwarding adapters have been removed. Providers
return the opened stream through its directional interface; Close through either
reference closes the same stream.

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
The same sample checks text-to-stream and stream-to-text reads on disk and in memory.
The verifier also checks exact disk bytes, invalid UTF-8, address capacity, replacement
with retained streams and text workflow failures.

## What remains open

Storage consumes the platform System.Storage.Path value object described below. The author's
intended boundary is Storage-specific: other APIs can continue accepting strings.
Callers may optionally parse a Path, then pass Text to a string-taking API. The native
metadata and file-stream APIs accept strings. String overloads on provider APIs
remain an open ergonomic choice; this slice does not add them.

GetFile now queries kind/existence and returns a provider-bound address. It does not
promise stable filesystem identity or later availability, nor does success grant
read/write permission. Later opens perform their own checks and can fail. Metadata
lookup adds an I/O operation; callers who only need bytes may still open directly.
Host lookup follows symlinks and uses the process's normal permissions. Name uses
the existing host filename helper on the experiment's shared slash-only logical
syntax; general provider-specific naming remains open.

Common item identity, provider-specific File implementations and StorageItem
contracts remain exploratory. Replacement while streams remain open, directory
structure and provider identity still need a common contract. General stream
capabilities, automatic disposal, cleanup across await and async scheduling also
remain design work. The sample APIs currently block and are not an async-first
provider contract.

For comparison, .NET's [FileInfo constructor](https://learn.microsoft.com/en-us/dotnet/api/system.io.fileinfo.-ctor?view=net-10.0)
creates a path wrapper. This experiment also retains a storage provider, allowing
the same consumer to use host files or logical memory storage. That indirection
and its identity rules need evaluation alongside the benefit.


## Path value object

**Integrated development API: [System.Storage.Path](xref:System.Storage.Path).**
`Path.Parse(text: string) -> Result<Path, InvalidPathError>` is the construction API.
The former StorageExperiment.Path implementation has moved into the platform library.
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
| `System.Storage.InvalidPathError()` | Empty error value identifying invalid logical syntax; no filesystem error is implied. |

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
Directory owns child resolution and calls Parse to validate its result. The existing
static Combine(string, string) and GetFileName(string) compatibility methods retain
their native string behavior; they do not return or validate Path values.
The downloaded Path.rvn now contains only `RequirePath(text)`, a sample fixture helper
for known paths: it calls
Parse and terminates with UserFault on invalid syntax. It is not a second parser.

.NET's [System.IO.Path](https://learn.microsoft.com/en-us/dotnet/api/system.io.path?view=net-10.0)
is a static string API with platform-dependent rules.
[GetFullPath](https://learn.microsoft.com/en-us/dotnet/api/system.io.path.getfullpath?view=net-10.0)
resolves native full-path spellings. This experiment separates validated logical
syntax from provider resolution. The benefit is reuse of a checked value; costs
include the new type, narrower grammar and explicit parsing at call sites.


### Future path formats

The author intends Path.Parse to support Unix and Windows formats and normalize
accepted paths through the value object. This is future work: the current parser
still accepts only the documented logical slash grammar and preserves its spelling.
Format selection, Windows drive/UNC roots, separator and dot-segment handling, and
normalization/equality rules need an explicit contract. No overload spelling or
automatic host-format detection has been selected. Normalization will remain distinct
from filesystem existence, permissions, symlink resolution and item identity.


Path-taking overloads of Combine and related helpers are also a future direction.
For now these helpers keep their string contracts; no new overload signature is
selected. The immediate goal is a minimal integrated Storage read/write/lookup POC.
