# Storage providers

**Development after Preview 9.** [StorageProvider](xref:System.Storage.StorageProvider)
resolves logical paths into provider-implemented item interfaces.

| Member | Contract |
| --- | --- |
| `GetItem(path: Path) -> Result<StorageItem, StorageLookupError>` | Look up an existing file or directory without knowing its kind. |
| `GetFile(path: Path) -> Result<File, StorageLookupError>` | Look up a file; a directory returns WrongKind. |
| `GetDirectory(path: Path) -> Result<Directory, StorageLookupError>` | Look up a directory; a file returns WrongKind. |

All three calls are synchronous and may block. Missing entries return NotFound;
access, invalid-path and I/O errors are preserved. They do not create entries or
retain open content streams. Success observes current state; later access can fail.
Unsupported backing entry kinds can return WrongKind. A Path validates syntax,
not existence, authority or stable identity.

Providers may construct concrete implementations internally, but these results
expose [StorageItem, File and Directory](storage-items.md). Consumers need no
provider-specific casts. File implementations supply byte opening; the provider
interface requires neither byte methods nor text encoding helpers.

## Disk and memory sample

The platform [FileSystem](xref:System.Storage.FileSystem) implements host storage.
The [tested sample](storage-experiment.md) wraps it in HostStorage for temporary text
conveniences and supplies MemoryStorage as a bounded comparison.
The product obtains its root Directory through GetDirectory, writes and reads a
file through directional streams, and checks expected failures. Generic lookup
returns both branches through StorageItem; contract tests check their interface types.
The host maps paths below its configured root and queries native kind. This mapping
is not a symlink-safe security sandbox. Memory uses flat keys and exposes only its
root (`/`, also addressed by `.`); slash-containing keys do not imply directories.

Sample ByteStorage routes stream operations inside the provider implementation;
it is not a platform interface. The sample's extended StorageProvider additionally
includes temporary whole-text helpers. Applications needing lookup use only
System.Storage.StorageProvider. The host provider and directory traversal are integrated. GetItems returns a
bounded snapshot of direct child StorageItem values. The final creation model and
async I/O remain follow-up work.

## Development migration

StorageLookup has been removed. Its GetFile/GetDirectory members now belong to
StorageProvider, alongside GetItem. The former StorageProvider.OpenRead/CreateNew
methods are removed from that contract; move byte routing into provider-owned File
implementations or an implementation-specific helper. Use matching regenerated
reference/runtime artifacts. Preview 9 downloads are unchanged.

## Design comparison

.NET System.IO primarily exposes native-path File/Directory operations and concrete
FileInfo/DirectoryInfo objects. WinRT Storage objects separate retrieval from opening.
The selected neoCLR contract follows that separation while returning interfaces and
typed errors over a provider's logical namespace. It allows the same consumer to
use disk and memory, at the cost of dispatch and provider-specific identity rules.
Async operation, metadata queries and portable replacement semantics remain open.

## Bounded enumeration

`Directory.GetItems(maxItems)` returns `Result<Sequence<StorageItem>, StorageLookupError>`.
No partial successful result is returned. FileSystem accepts 0–1024 items and at
most 64 KiB of native names, subject to host array limits. Negative bounds return
InvalidRange; count/resource overflow returns LimitExceeded. Zero succeeds only
for an empty directory. Native names are sorted ordinally before resolving items.

There is no retained enumeration handle. Enumeration and child lookup are separate
observations; concurrent removal or an unsupported child name/kind can fail the
whole operation. Symlinks follow host metadata behavior. The memory fixture lists
only direct root children and does not promise host ordering. Metadata queries,
recursive traversal and stable snapshot identity are not implied.


## Future asynchronous operations

Lookup, enumeration and stream opening may later return Task-wrapped Results to
support providers with asynchronous I/O. Parsing and represented properties can
remain synchronous. Current methods block; task signatures, scheduling, cancellation
and ownership rules are not implemented by this POC.
