# File and Directory descriptors

**Development after Preview 9.** [File](xref:System.Storage.File) and
[Directory](xref:System.Storage.Directory) are platform-owned addresses. Construction
and Path/Name access do not query storage, create entries or open streams. A descriptor
can describe a destination that does not exist yet; it has no close/dispose operation.

File retains a [StorageProvider](xref:System.Storage.StorageProvider) and Path.
OpenRead and CreateNew delegate to that provider and return the directional streams.
Close each successful stream; a descriptor owns no open handle. Name derives from
the accepted logical path's last component, preserving its spelling.

Directory retains a [StorageLookup](xref:System.Storage.StorageLookup) and Path.
StorageLookup extends the byte-provider contract with GetFile(Path): inspect a current
file and return its descriptor, without opening content or retaining a stream.
Byte-only providers remain valid StorageProvider implementations.

| Operation | Behavior |
| --- | --- |
| `Directory.FileAt(name)` | Construct one child File address; invalid names return StorageLookupError.InvalidPath. No lookup. |
| `Directory.GetFile(name)` | Resolve one child name and ask the provider to look it up. |
| `Directory.GetFile(relativePath)` | Resolve a relative Path, including nested segments; reject absolute inputs. |
| `StorageLookup.GetFile(path)` | Query within the provider namespace; return a File or typed lookup error. |

The string Directory overloads accept a single name. The Path overload supports
nested relative addresses; `.` queries the directory address as a file, normally
WrongKind. Resolution handles `/` and `.` without doubling separators. This provisional
overload distinction is not a system-wide restriction on string-based APIs.

A successful lookup is an observation, not a promise that opening later succeeds.
Descriptors preserve provider context and spelling, not filesystem identity.
Directory construction does not verify its parent or kind. The current memory fixture
uses flat keys; richer directory hierarchy, enumeration, timestamps and size metadata
remain deferred. Provider root mapping is not a security sandbox.

The [tested sample](storage-experiment.md) uses these platform descriptors for the
same disk/memory byte workflow. Its concrete providers and whole-text conveniences
remain application code. The earlier sample File.ReadText/WriteText methods are
replaced by explicit provider text helpers; they are not platform descriptor members.
Directory.FileAt now uses StorageLookupError rather than the sample's FileReadError.

## Static native text helpers

The instance API preserves the existing static methods on System.Storage.File.
They accept native strings independently of any descriptor or provider. Calls block,
open and close their own handle, and return expected failures as values. UTF-8 byte
limits count bytes, not characters. Negative limits return InvalidLimit; zero permits
empty content. Empty paths and NUL return InvalidPath. Missing paths/parents,
permissions and non-regular files have dedicated cases; remaining failures return
ReadFailed or WriteFailed. Reads reject invalid UTF-8 and preserve a BOM as text.

`File.ReadAllText(path: string, maxBytes: int) -> Result<string, FileReadError>`
is covered by the generated member reference.

## WriteAllText

`File.WriteAllText(path: string, text: string, maxBytes: int) -> Result<unit, FileWriteError>`

- `path`: native destination path, with host resolution rules.
- `text`: text encoded as UTF-8; no BOM is added.
- `maxBytes`: nonnegative maximum encoded byte count.
- Result: success unit, or InvalidLimit, InvalidPath, NotFound, AccessDenied,
  NotRegularFile, TooLarge or WriteFailed.

Creates a missing file or truncates/replaces an existing regular file's contents.
An invalid limit or oversized text is rejected before modifying storage. A later
I/O failure can leave empty or partial output; writes are not atomic replacement
and do not promise durability. Parent directories are not created.

DocFX cannot render this method's Result<System.Void, FileWriteError> metadata.
Only this exact method is excluded; this entry supplies its signature and contract.

## Design comparison

The existing .NET/WinRT comparison informs this slice: like .NET FileInfo and WinRT
StorageFile, a file object is separate from an opened stream. Unlike .NET FileInfo,
these descriptors expose no cached filesystem metadata or Refresh operation.
Provider-bound addresses let the same consumer operate on disk and memory; the
cost is explicit provider construction and more interface dispatch. Keeping lookup
as a separate capability avoids requiring it for every byte provider. Stable identity,
asynchronous opening and metadata freshness remain open design questions.
