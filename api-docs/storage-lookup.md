# Storage metadata lookup

**Development after Preview 9.** This synchronous host adapter queries a native path
without opening its contents or retaining a stream handle. Use matching development
artifacts. It is separate from the application-owned Storage provider experiment.

- [Metadata](xref:System.Storage.Metadata): `GetKind(path: string) -> Result<EntryKind, StorageLookupError>`.
- [EntryKind](xref:System.Storage.EntryKind): File and Directory cases.
- [StorageLookupError](xref:System.Storage.StorageLookupError): InvalidPath, NotFound,
  AccessDenied, WrongKind and IoFailure cases.

```raven
match System.Storage.Metadata.GetKind(".") {
    Ok(let kind) => {
        if !kind.IsDirectory {
            System.Fault("Directory metadata lost its kind")
        }
    }
    Error(let error) => System.Fault(error.ToString())
}
```

This fragment is compiled in the sample's LookupContracts check. Native paths use
host rules; relative strings resolve against the process working directory. The
Path value object is useful within Storage, but is not mandatory for this API or
for file-stream Open/CreateNew. A caller can optionally parse and pass Path.Text;
logical provider roots are meaningful only through the provider's resolver.

GetKind follows symbolic links. Empty/NUL-containing paths return InvalidPath,
missing or dangling targets return NotFound, denied metadata access returns
AccessDenied, and other host failures return IoFailure. Filesystem objects other
than regular files and directories return WrongKind. Host service policy can reject
execution separately; typed I/O errors do not override host authority. The call may
block and does not imply permission to read or write contents.

Success is a point-in-time observation. A file may disappear, change kind or be
replaced before the next open. Existing handles can retain different contents from
those reached by the same address later. Neither EntryKind nor lexical Path equality
provides stable item identity. There is no metadata cache, content read, open-file
budget consumption or sandbox guarantee in this wrapper.

The [provider experiment](storage-experiment.md) adds GetFile(Path) and direct-child
Directory.GetFile(name). HostStorage queries GetKind and requires File; MemoryStorage
checks its shared address table. Both return descriptors retaining the provider.
FileAt still constructs destinations without I/O. This lets callers create a missing
file without a lookup, or query one explicitly when that observation is useful.

Compared with [.NET FileInfo.Exists](https://learn.microsoft.com/en-us/dotnet/api/system.io.fileinfo.exists?view=net-10.0),
this API distinguishes supported kinds and typed errors rather than a cached boolean.
That adds result handling and still cannot eliminate lookup/open races. The internal
host service uses [Rust metadata](https://doc.rust-lang.org/std/fs/fn.metadata.html),
which follows links and reports I/O errors. Sources reviewed 2026-09-23. The sample's
provider API, directory model and async completion remain exploratory.
