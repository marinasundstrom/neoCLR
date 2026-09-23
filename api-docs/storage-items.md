# Storage item interfaces

**Development after Preview 9.** [StorageItem](xref:System.Storage.StorageItem) has
two permitted interface branches: [File](xref:System.Storage.File) and
[Directory](xref:System.Storage.Directory). Providers implement these branches;
the common model does not require native files, handles or filesystem attributes.

Both branches inherit Name and Path, which return represented state without a new
storage query. A Path describes an address in a provider namespace; it does not
establish access or stable identity. Callers depend on the interfaces rather than
constructing File or Directory themselves.
Provider methods return the interfaces, even when they instantiate concrete classes
internally. Current StorageLookup.GetFile returns Result<File, StorageLookupError>;
future directory/item lookup and enumeration must preserve the same boundary.
The sample bootstraps ProviderDirectory explicitly while provider GetDirectory is
pending; its consumer functions accept Directory, not the concrete class.

The Raven compiler and strict CIL importer enforce the closed root. Providers may
implement File/Directory, but an unrelated third StorageItem branch is rejected.
The importer also rejects a raw CIL third branch that bypasses Raven. Raw neoIL does
not enforce this compiler metadata; closure is not a native runtime security boundary.

## Current operations

| Interface member | Contract |
| --- | --- |
| `File.OpenRead()` | Open existing bytes as an InputStream or return StreamError. Close the successful stream. |
| `File.CreateNew()` | POC operation for exclusive creation at an address; no overwrite. Returns OutputStream or StreamError. |
| `Directory.FileAt(name)` | POC operation constructing a child File address without I/O; invalid names return StorageLookupError.InvalidPath. |
| `Directory.GetFile(name)` | Resolve one child name and explicitly look up its file. |
| `Directory.GetFile(relativePath)` | Resolve a relative Path, including nested segments; reject absolute inputs. |

Operations are currently synchronous. Read/write counts can be partial. Creation
is not a transaction; a later write failure can leave an empty or partial file.
See the [stream contract](streams.md).

The tested [sample](storage-experiment.md) owns ProviderFile and ProviderDirectory
implementations which retain their disk/memory provider and logical Path. Its
Directory implementation handles `/` and `.` prefixes without doubling separators;
`.` lookup normally returns WrongKind. String traversal accepts one child name.
The memory fixture uses flat keys and does not model a full directory hierarchy.

StorageProvider's byte methods and the separate StorageLookup capability remain
transitional. The selected model moves path resolution onto StorageProvider, with
GetItem/GetFile/GetDirectory, and adds Directory traversal and GetItems enumeration
of StorageItem values. GetItem/GetDirectory/GetItems are **not implemented yet**.
Creation/address factories will be aligned with resolved storage objects in that
work; FileAt/CreateNew should not be treated as the final creation model. Async
interaction needs scheduling and cancellation contracts before claiming nonblocking I/O.

## Additional information

Information and attributes should be queryable through storage objects without
tying the common model to filesystem-specific flags. Typed information, optional
capabilities and extensible queries remain alternatives. Availability, absent
values, failures and freshness need explicit contracts. No general attribute-query
API is implemented yet.

## Development migration

- File and Directory are interfaces; providers supply concrete implementations.
  The sample's former descriptor logic is now ProviderFile/ProviderDirectory.
- Native static text callers use `System.Storage.FileText.ReadAllText/WriteAllText`
  instead of the former static methods on File. The interface runtime does not
  currently admit static methods. The legacy raw runtime profile retains the old
  File helper aliases for existing artifacts; the development Raven profile uses FileText.
- Existing stream, Path and typed-error behavior is unchanged. Use matching
  regenerated core/runtime artifacts; Preview 9 downloads are unchanged.

## Static native text helpers

[FileText](xref:System.Storage.FileText) accepts native strings independently of any
storage provider. Calls block, open and close their own handle, and return expected
failures as values. UTF-8 byte limits count bytes, not characters. Negative limits
return InvalidLimit; zero permits empty content. Empty paths and NUL return
InvalidPath. Missing paths/parents, permissions and non-regular files have dedicated
cases; remaining failures return ReadFailed or WriteFailed. Reads reject invalid
UTF-8 and preserve a BOM as text.

`FileText.ReadAllText(path: string, maxBytes: int) -> Result<string, FileReadError>`
is covered by the generated member reference.

## WriteAllText

`FileText.WriteAllText(path: string, text: string, maxBytes: int) -> Result<unit, FileWriteError>`

- `path`: native destination path, with host resolution rules.
- `text`: text encoded as UTF-8; no BOM is added.
- `maxBytes`: nonnegative maximum encoded byte count.
- Result: success unit, or InvalidLimit, InvalidPath, NotFound, AccessDenied,
  NotRegularFile, TooLarge or WriteFailed.

Creates a missing file or truncates/replaces an existing regular file's contents.
An invalid limit or oversized text is rejected before modification. A later I/O
failure can leave empty or partial output; writes are not atomic replacement and
do not promise durability. Parent directories are not created.

DocFX cannot render Result<System.Void, FileWriteError>. Only this exact method is
excluded; this entry supplies its signature and contract.

## Design comparison

The .NET/WinRT comparison separates a storage object from its opened stream. Here
File and Directory are interfaces, so providers can supply implementations without
inheriting native filesystem classes or caching rules. This supports common
consumers across storage kinds; the costs are interface dispatch and explicit
contracts for provider differences. Metadata freshness, item identity and async
opening remain open. The initial POC still exposes address factories; interface
migration alone does not finish the proposal's resolved-item model.
