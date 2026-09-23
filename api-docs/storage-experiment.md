# Storage provider experiment

**Development exploration, not a published System.Storage API.** These
`StorageExperiment` types belong to a tested Raven sample. Their purpose is to
compare provider ownership, file descriptors and path interpretation before
settling the platform API. They are documented here so the experimental contract
is visible; they are not types in the core reference assembly.

Download the tested [project file](/samples/storage-provider/StorageExplorer.rvnproj),
[Storage.rvn](/samples/storage-provider/Storage.rvn),
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

The disk implementation uses the existing synchronous UTF-8 file helpers. The
memory implementation has one slot and uses logical keys rather than filesystem
paths. These are useful comparison cases, not a finished filesystem provider model.

## Build and run the sample

Save the project and both Raven files in one directory. Select matching neoCLR and
Raven SDK artifacts as described in the [setup guide](/try/index.html):

```sh
dotnet msbuild StorageExplorer.rvnproj -p:NeoCLRRoot=/path/to/neoclr -p:RavenSdkRoot=/path/to/raven-sdk
/path/to/neoclr/bin/neoclr run bin/neoclr/Debug/App.neoil --system bin/neoclr/Debug/System.neoil
```

Run in a scratch working directory: the disk case creates or replaces `notes.txt`
there. The checked-in verifier uses a disposable directory and also verifies the
actual bytes, error handling and independent provider state.

## StorageProvider

Implementations interpret their own addresses and perform bounded text operations.
These text methods are temporary adapters; the intended next experiment puts byte
streams below text decoding/encoding.

| Member | Contract |
| --- | --- |
| `FileAt(directory: string, name: string) -> File` | Construct a descriptor for a child address using the provider's own syntax. Does not check existence. Low-level callers supply a valid child name; use Directory.FileAt for validation. |
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
| `ReadText(maxBytes: int) -> Result<string, FileReadError>` | Read through the retained provider. A missing file is reported when read, not when its descriptor is constructed. |
| `WriteText(text: string, maxBytes: int) -> Result<unit, FileWriteError>` | Write through that same provider; never select a provider from ambient state. |

## Supplied providers

`HostStorage()` implements all three StorageProvider methods using native path
combination and the existing whole-file UTF-8 helpers. Reads and writes block;
files are opened and closed within each helper call. Its permissions, symlinks and
path validity follow the host. There is no sandbox or read-only capability here.

`MemorySlotStorage()` implements the same methods with one in-memory entry per
instance. Writing another address replaces that slot. Reads at a missing address
return NotFound. Negative limits return InvalidLimit; UTF-8 byte counts above the
limit return TooLarge. Rejected writes preserve the previous entry. Distinct
instances do not share state. It is a test provider, not a general memory filesystem.

## What remains open

A pure Path type, eager lookup and item identity, provider-specific File
implementations, shared StorageItem contracts, byte streams, cleanup across await,
and async scheduling remain design work. The separate native file-resource
experiment is not yet wired into these descriptors. This sample proves the
provider-bound text workflow; it does not complete the Storage/Stream milestone.

For comparison, .NET's [FileInfo constructor](https://learn.microsoft.com/en-us/dotnet/api/system.io.fileinfo.-ctor?view=net-10.0)
creates a path wrapper. This experiment explores retaining a storage provider as
well, allowing the same consumer to use host files or logical memory storage.
The additional indirection and identity rules need evaluation alongside that benefit.

## Byte-stream integration status

The internal file backend now supports reading into a caller-owned managed byte
array with an offset and count. It reports partial reads, preserves untouched
buffer elements and rejects invalid ranges before consuming file data. Calls are
blocking. This is groundwork for directional streams; the experimental provider
members documented above still use whole-text helpers. No new public member is
implied by the backend change.
