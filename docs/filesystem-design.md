# File-system capability proposal

Proposal recorded 2026-09-16. The current `System.IO.File` and `File.WriteAllText`
APIs remain implemented as bounded, synchronous host gateways; this document proposes
a future capability-oriented API and does not replace those contracts.

## Problem and model

Applications need file access that can be isolated in tests, redirected to a virtual
or packaged namespace, and constrained by an embedding host. A `Path` is therefore a
pure value, while a `FileSystem` supplies the context in which that path is resolved.
Files, directories and streams are handles or values obtained from that capability.

```text
Path (pure value)
        │ resolved by
        ▼
FileSystem
  ├── HostFileSystem
  ├── MemoryFileSystem
  ├── ZipFileSystem
  └── EmbeddedFileSystem
        │
        ├── File
        ├── Directory
        └── FileSystemEntry
```

The preferred public shape is a deliberately small `System.IO.FileSystem` contract:

```text
FileSystem.Default
FileSystem.Open(path, mode) -> Result<FileStream, IOError>
FileSystem.FileExists(path) -> Bool
FileSystem.DirectoryExists(path) -> Bool
FileSystem.CreateDirectory(path) -> Result<Directory, IOError>
FileSystem.GetFile(path) -> Option<File>
FileSystem.GetDirectory(path) -> Option<Directory>
FileSystem.Enumerate(path) -> Result<Iterable<FileSystemEntry>, IOError>
```

Exact names, receiver shape, stream ownership and enumeration lifetime remain open.
Text convenience methods such as `File.ReadText()` may be built over byte streams and
explicit encoding; the core filesystem capability should not absorb text, buffering or
codec policy. `File`, `Directory` and `FileSystemEntry` should avoid reproducing the
historical `File`/`FileInfo` and `Directory`/`DirectoryInfo` split unless a distinct
identity or caching contract justifies it.

## .NET baseline and alternatives

The .NET baseline is a mixed static and object-oriented `System.IO` surface. `File`
and `Directory` provide convenient static operations, while `FileInfo` and
`DirectoryInfo` represent path-oriented objects and expose related operations. The
baseline is familiar and broad, but static calls select ambient host state and its
object split does not by itself provide dependency injection. `.NET 10` documentation
continues to describe `DirectoryInfo` as a `FileSystemInfo`-derived object and exposes
directory enumeration through that object ([DirectoryInfo](https://learn.microsoft.com/en-us/dotnet/api/system.io.directoryinfo?view=net-10.0),
[GetFiles](https://learn.microsoft.com/en-us/dotnet/api/system.io.directoryinfo.getfiles?view=net-10.0)).
These are library conventions, not CLI requirements or a need for a new runtime
instruction.

Alternatives:

| Alternative | Benefit | Cost / limitation |
| --- | --- | --- |
| Keep static `File`/`Directory` gateways | Maximum .NET familiarity and a small call surface | Harder isolation, ambient host authority, and no natural virtual/remote namespace |
| Add an injectable facade over the static APIs | Easy migration and test substitution | The abstraction inherits static API partitioning and may hide path/handle ownership |
| Use a first-class `FileSystem` capability | Explicit authority and context; host, memory and archive implementations share one contract | More objects and lifetime/error rules; callers must obtain or inject a capability |
| Make every operation a runtime intrinsic | Direct host implementation | Couples public API to one backend, expands runtime surface and makes virtual filesystems harder |

The independent .NET ecosystem project [System.IO.Abstractions](https://github.com/TestableIO/System.IO.Abstractions)
demonstrates that an injectable `IFileSystem` can preserve a familiar API while
providing a mock filesystem. Its nested `File`/`Directory` facade is useful evidence
for testability, but also preserves the .NET partition. NeoCLR's proposal deliberately
uses the filesystem as the context and makes handles explicit; it is not claiming that
the ecosystem abstraction is inadequate for its original migration goal. A backend
comparison with Rust's `std::fs` and archive/virtual-filesystem libraries remains an
evidence gap before freezing the contract.

## Capability, errors and ownership

`FileSystem.Default` is an ergonomic default, not a mutable global replacement point.
Application services that need isolation should receive `FileSystem`; tests can inject
`MemoryFileSystem` or a fixed test double without changing process state. An embedding
host may provide a restricted implementation or omit the host capability entirely.

`Path` operations are pure and should not touch the host. Resolution, permissions,
missing entries, invalid paths, type mismatches and I/O failures belong in typed
`Result` errors. `Option` is appropriate only for an ordinary absence such as a
nonexistent result from `GetFile`; an operation that cannot determine existence should
return an error rather than `false` by policy. Terminal `Fault` remains for runtime
invariant violations, resource exhaustion or failures outside the recoverable I/O
contract.

`FileStream` owns an open byte-I/O operation and therefore needs an explicit close or
dispose contract before long-lived streams are added. The separate [Stream proposal](stream-design.md)
defines the intended byte-capability boundary, but does not yet select a concrete
`FileStream` type. Enumeration needs a defined
snapshot/streaming behavior, cancellation and cleanup policy. A `File` or `Directory`
handle must state whether it is a stable path descriptor, an open native handle, or a
capability-bound view; copying it must not accidentally duplicate or prematurely close
the underlying resource. Symlink behavior, case sensitivity, root restrictions,
atomicity and cross-platform path rules remain backend-specific questions to expose or
constrain explicitly.

## Decision and status

Prefer a small, injectable `FileSystem` capability with `File`/`Directory` handles,
pure `Path`, and streams as the byte-I/O layer. Retain `FileSystem.Default` for simple
programs, but keep authority selection at the composition boundary. This gives the
architecture a useful context for memory, archive and remote filesystems, not merely a
mocking seam. It costs a new capability and resource model and does not make host I/O
portable automatically.

This is a provisional library proposal. It does not authorize removal or renaming of
the current static text-file APIs, add a sandbox, or promise that all backends support
the same filesystem semantics. Before implementation, decide the exact capability
surface, path normalization, error taxonomy, handle lifetime, stream/disposal model,
enumeration behavior, symlinks, permissions, limits, cancellation and async form.

## Validation required before implementation

- Run the same configuration loader against a host filesystem and a memory filesystem;
  verify path resolution, empty files, Unicode text, embedded NUL and typed failures.
- Exercise missing paths, permission denial, file/directory mismatches, invalid paths,
  traversal/root restrictions, partial reads/writes and allocation/resource limits.
- Check that `Path` manipulation is host-independent and that `File`/`Directory`
  copies do not double-close or retain an invalid native resource.
- Test enumeration mutation, ordering guarantees, cancellation and cleanup, plus an
  archive or embedded backend to prove that the abstraction is more than a wrapper.
- Validate handwritten IL and alternate frontend calls against capability availability;
  an unavailable filesystem must be distinguishable from an empty directory or absent
  file. Compare the selected behavior with .NET's static/object APIs and record
  migration adapters before changing existing consumers.
