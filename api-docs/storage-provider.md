# Storage providers

**Development after Preview 9.** [System.Storage.StorageProvider](xref:System.Storage.StorageProvider)
is the platform's minimal byte-access contract. Implementations provide two operations:

| Member | Contract |
| --- | --- |
| `OpenRead(path: Path) -> Result<InputStream, StreamError>` | Open an existing file with a fresh cursor at zero. |
| `CreateNew(path: Path) -> Result<OutputStream, StreamError>` | Exclusively create an empty file; never overwrite an existing entry. |

Both calls are synchronous and may block. A successful call transfers a stream to
the caller, who must close it. Read and write counts can be partial; zero on a
positive read means EOF. See the [stream guide](streams.md) for buffer validation,
close behavior and errors. Stream errors preserve expected failures as values.
Creation is not a transaction: a failed later write can leave an empty or partially
written file. Flush does not promise durable storage.

[Path](xref:System.Storage.Path) validates logical syntax. A provider interprets that
address; it may reject a syntactically valid path or deny access. Neither parsing nor
opening implies a security sandbox. The contract does not standardize replacement
identity, concurrent mutation, parent creation or maximum storage capacity.

## Disk and memory example

The [tested Storage sample](storage-experiment.md) supplies HostStorage and
MemoryStorage implementations. Its File.OpenRead and File.CreateNew dispatch through
the platform interface; the same byte workflow runs against both. The host provider
maps logical paths beneath a native root and uses the platform file streams. The
memory provider deliberately uses two-byte transfers to exercise partial I/O.
Their concrete implementations and text conveniences remain sample-owned.
[StorageItem/File/Directory interfaces](storage-items.md) and the StorageLookup capability
are integrated into the platform; provider-specific descriptor state belongs to
the sample implementation classes.

A byte-only provider is not required to implement lookup, ReadText or WriteText.
StorageLookup extends it with GetFile and GetDirectory; the sample extends that capability with
temporary text conveniences. UTF-8
conversion belongs above byte access; existing static file text helpers remain
available with their current string paths and error types.

## Design comparison

.NET uses static File operations and FileStream over native paths, while Windows
Runtime exposes StorageFile and StorageFolder descriptors with separate opening
operations. This small provider interface lets one consumer use disk and memory
without requiring every backend to encode text or reproduce a metadata hierarchy.
The cost is another dispatch boundary and provider-specific identity and lifetime
rules. Async I/O, richer metadata and portable replacement semantics remain open;
this is an incremental POC contract, not the complete Storage proposal.

## Directory lookup

`StorageLookup.GetDirectory(path: Path) -> Result<Directory, StorageLookupError>`
queries the current entry without creating anything or opening content. The result
is the public Directory interface; its concrete implementation belongs to the
provider. Missing entries return NotFound and files return WrongKind. Access and
I/O failures are preserved. Success does not guarantee later child access.

The disk sample checks the mapped native entry. The bounded memory sample exposes
only `/` (also addressed by `.`); slash-containing file keys do not imply a tree.
The product obtains its root through this method and uses the returned Directory
for the same disk/memory workflow. Calls remain synchronous. Adding this required
method is a development migration for StorageLookup implementers; byte-only
StorageProvider implementations are unaffected. The separate lookup interface is
transitional pending consolidation into the proposal's provider contract.
