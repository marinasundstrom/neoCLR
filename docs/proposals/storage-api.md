# NeoCLR Storage API

## Selected object model — 2026-09-23

The author selects this proposal's storage model as the implementation goal, with
an explicit correction: **StorageItem, File and Directory are interfaces**.
StorageItem has the two permitted branches File and Directory. Concrete providers
implement those branches; they do not add unrelated direct StorageItem branches.
The existing development File/Directory classes are implementation material for
provider-specific objects, not the final public abstractions.

StorageProvider resolves paths into these objects. GetItem returns one StorageItem;
Directory.GetItems enumerates StorageItem values, allowing files and directories in
the same result. The sequence/async-sequence and task signatures remain to be settled
with the operations; the common element model is selected.

The interface hierarchy is now integrated in the development library, with
provider-owned sample implementations and compiler/importer closure checks. It is
not in Preview 9. StorageProvider now resolves GetItem/GetFile/GetDirectory; the
separate StorageLookup and byte-first provider contract are removed. FileSystem host integration, relative directory traversal and bounded synchronous
GetItems(maxItems) snapshots are now implemented. Async/incremental enumeration
remains a future alternative. The [roadmap](../platform-roadmap.md) governs the bounded
implementation sequence. Optional topology, rich metadata and unneeded mutations
in this proposal are not prerequisites for the initial POC.

## Overview

NeoCLR models **storage as an explicit platform capability**.

```raven
namespace System.Storage
```

The Storage API does not assume that an application has unrestricted access to a host filesystem. Storage may instead be exposed through a native filesystem, an application-specific area, package resources, memory, an archive, remote storage, or another provider.

The fundamental model is:

```text
StorageProvider
      │
      │ resolves Path
      ▼
 StorageItem
   ├── File
   └── Directory
```

Additional concepts such as storage units, storage media, permissions, links, volumes, and native filesystem metadata are valid parts of the storage domain, but they are exposed only where they are meaningful.

The Storage API therefore models **accessible storage first**, rather than treating the host filesystem as the universal abstraction.

---

# Core model

The initial core consists of:

```text
System.Storage

StorageProvider
StorageItem

File
Directory
Path

FileSystem
```

The relationship between these concepts is:

```text
                   StorageProvider
                          │
                          │ Path
                          ▼
                     StorageItem
                    ╱           ╲
                   ▼             ▼
                File         Directory
                                │
                                │ relative Path
                                ▼
                           StorageItem
```

A `StorageProvider` establishes a storage namespace.

A `Path` identifies a location within a storage namespace.

A `StorageItem` represents an accessible object resolved from storage.

A `Directory` can itself be used as a narrower capability from which additional items can be resolved.

---

# StorageProvider

`StorageProvider` represents access to a storage namespace.

Its fundamental responsibility is resolving paths.

Conceptually:

```raven
interface StorageProvider
{
    func GetItem(path: Path)
        -> Result<StorageItem, StorageError>;

    func GetFile(path: Path)
        -> Result<File, StorageError>;

    func GetDirectory(path: Path)
        -> Result<Directory, StorageError>;
}
```

Typed operations allow callers to request the expected item directly:

```raven
let file =
    storage.GetFile(Path("/config/settings.json"))?;
```

Generic lookup remains useful when the item type is not known:

```raven
let item =
    storage.GetItem(path)?;

match item
{
    File file =>
        ...,

    Directory directory =>
        ...
}
```

The provider determines which paths are meaningful within its namespace.

A provider may represent:

```text
native filesystem
application storage
package resources
memory storage
archive contents
remote storage
sandboxed storage
virtual storage
```

Not every provider needs to expose the same functionality.

---

# FileSystem

`FileSystem` represents conventional filesystem access.

It is a storage provider:

```raven
interface FileSystem : StorageProvider
{
    ...
}
```

For example:

```raven
let fs = FileSystem.Default;

let file =
    fs.GetFile(Path("/home/user/config.json"))?;
```

A `FileSystem` may understand additional native concepts such as:

```text
filesystem roots
volumes
mounts
storage units
storage media
links
native permissions
filesystem attributes
native paths
```

These capabilities belong to the filesystem model rather than being requirements placed on all storage providers.

Therefore:

> `FileSystem` is a storage provider, not the definition of storage.

---

# Path

`Path` is a value object representing a syntactically valid storage path.

A `Path` can represent either an **absolute path** or a **relative path**.

```raven
let absolute =
    Path("/home/user/config.json");

let relative =
    Path("config/settings.json");
```

The distinction is a property of the value:

```raven
absolute.IsAbsolute
relative.IsRelative
```

NeoCLR does not require separate `AbsolutePath` and `RelativePath` types.

The `Path` type describes what the value is. Individual APIs determine which kinds of paths they accept.

For example:

```text
Path
 ├── absolute
 └── relative

StorageProvider
 └── provider-defined path requirements

Directory
 └── relative paths
```

This follows a general principle:

> Value types describe values. APIs define constraints on which values they accept.

---

# Path parsing

`Path` is responsible for parsing and validating path syntax.

For example:

```raven
let path =
    Path.Parse("config/settings.json")?;
```

Parsing establishes that the input represents a structurally valid path.

It does not establish that the corresponding storage item exists.

Resolution is separate:

```raven
let path =
    Path.Parse("config/settings.json")?;

let file =
    storage.GetFile(path)?;
```

Conceptually:

```text
String
   │
   │ Parse
   ▼
 Path
   │
   │ Resolve
   ▼
StorageItem
```

`Path` may provide operations such as:

```text
IsAbsolute
IsRelative

Name
Parent
Extension
Segments

Combine
Normalize
```

The exact surface should remain focused on path manipulation rather than storage access.

---

# Path syntax and provider semantics

A valid `Path` does not necessarily mean that every provider can resolve it.

For example, a path may be syntactically valid while being inappropriate for a particular provider.

The distinction is:

```text
Path
    validates path structure

StorageProvider
    validates whether the path can be resolved
    within that storage namespace
```

This allows providers to impose additional restrictions without requiring separate path types.

For example, an application-storage provider may reject paths that are valid in the general path model but outside its supported namespace.

---

# StorageItem

Objects exposed through storage are represented by `StorageItem`.

`StorageItem` forms a sealed hierarchy:

```raven
sealed interface StorageItem
{
    Name: String;
    Path: Path;
}

interface File : StorageItem
{
    ...
}

interface Directory : StorageItem
{
    ...
}
```

The initial hierarchy is:

```text
StorageItem
   ├── File
   └── Directory
```

The hierarchy is closed at the **kind** boundary, not at the provider implementation
boundary. File and Directory remain implementable by providers. For example,
filesystem-backed and memory-backed File implementations can expose different
internal state while consumers depend only on File. Do not seal the File branch
to a fixed set of built-in concrete classes.

The author further clarifies that providers may construct and use concrete classes
internally, but the contract exposes interfaces. GetFile returns File, GetDirectory
returns Directory, GetItem returns StorageItem, and GetItems enumerates StorageItem
values (inside the chosen Result/Task/sequence layers). Consumers do not need casts
to provider-specific classes. Implementation classes are not a required public API.

Provider-specific implementations own resolution and opening behavior. Public
constructors on the File/Directory interfaces are not part of the target model.
Ordinary consumers obtain objects from provider resolution, directory traversal or
explicit creation operations. Construction of a provider implementation is separate
from resolving an accessible item; an address alone is not proof of access.

Validate the compiler metadata and importer contract for both requirements together:
reject an unrelated direct StorageItem branch, and allow a separately compiled
provider to implement File or Directory. Do not advertise this closure as enforced
by the runtime until that layer has been checked too.

Because the hierarchy is closed, callers can naturally pattern-match over storage objects:

```raven
match item
{
    File file =>
        ...,

    Directory directory =>
        ...
}
```

This provides one common model without requiring separate metadata abstractions such as `FileInfo`, `DirectoryInfo`, and `FileSystemInfo` merely to identify storage objects.

---

# File

`File` represents access to a particular stored file.

For example:

```raven
let file =
    storage.GetFile(Path("/data/index.bin"))?;

let stream =
    file.OpenRead()?;
```

A file may provide convenience operations:

```raven
let text = file.ReadText()?;
let bytes = file.ReadBytes()?;
```

A `File` does not imply a native filesystem file.

It may represent:

```text
native filesystem file
package resource
archive entry
in-memory file
remotely backed file
virtual file
```

Operations available through the file should reflect the semantics supported by the underlying storage implementation.

---

# Directory

`Directory` represents a navigable collection of storage items.

A directory can resolve direct children by name:

```raven
let file =
    directory.GetFile("settings.json")?;

let child =
    directory.GetDirectory("cache")?;
```

It can also resolve descendants through a relative `Path`:

```raven
let file =
    directory.GetFile(
        Path("config/settings.json"))?;

let child =
    directory.GetDirectory(
        Path("resources/templates"))?;
```

Conceptually:

```raven
interface Directory : StorageItem
{
    func GetItem(name: String)
        -> Result<StorageItem, StorageError>;

    func GetFile(name: String)
        -> Result<File, StorageError>;

    func GetDirectory(name: String)
        -> Result<Directory, StorageError>;

    func GetItem(path: Path)
        -> Result<StorageItem, StorageError>;

    func GetFile(path: Path)
        -> Result<File, StorageError>;

    func GetDirectory(path: Path)
        -> Result<Directory, StorageError>;
}
```

---

# Mixed-item enumeration

A directory can enumerate files and subdirectories through the common interface:

```text
Directory.GetItems
    → collection/sequence of StorageItem
        → File implementation
        → Directory implementation
```

This differs from GetItem, which resolves one named or relative item. The common
element is StorageItem, not a provider-specific descriptor or an untyped object.
An empty directory produces an empty successful enumeration; failure is a separate
outcome. Ordering, eager versus incremental enumeration, failure timing and
cancellation need definition alongside the first implementation. The
[asynchronous extension](storage-api-extensions.md#enumeration) discusses task and
async-sequence alternatives; the author has not selected a container signature here.

---

# Names versus paths

The `String` overloads represent **direct child names**, not string-based paths.

For example:

```raven
directory.GetFile("settings.json")
```

requests the child named `settings.json`.

A string containing path traversal syntax is not interpreted as a path:

```raven
directory.GetFile("config/settings.json")
```

is invalid as a child name.

Traversal requires an explicit `Path`:

```raven
directory.GetFile(
    Path("config/settings.json"));
```

This gives the API a useful rule:

> Strings represent names. `Path` values represent paths.

Storage APIs therefore do not ambiguously accept arbitrary strings as either names or paths.

If a stronger representation for individual names becomes useful later, a dedicated value such as `StorageName` could be considered. It is not required for the initial model.

---

# Directory path semantics

`Directory` accepts **relative paths**.

For example:

```raven
directory.GetFile(
    Path("config/settings.json"));
```

is valid.

An absolute path is not valid for directory-relative traversal:

```raven
directory.GetFile(
    Path("/config/settings.json"));
```

The `Path` itself remains perfectly valid. It simply does not satisfy the contract of this particular operation.

This distinction is important:

```text
Path("/config/settings.json")
    ↓
valid Path

Directory.GetFile(...)
    ↓
requires relative Path
```

The constraint belongs to `Directory.GetFile`, not to the `Path` type.

---

# Directory as a capability

A `Directory` can act as a narrowed storage capability.

Consider:

```text
FileSystem
    │
    ▼
Application Directory
    │
    ├── config
    ├── cache
    └── data
```

A component that only needs access to application configuration can receive the relevant directory:

```raven
class ConfigLoader
{
    let directory: Directory;

    init(directory: Directory)
    {
        self.directory = directory;
    }

    func Load()
        -> Result<Config, ConfigError>
    {
        let file =
            directory.GetFile("config.json")?;

        let text =
            file.ReadText()?;

        return Config.Parse(text);
    }
}
```

The component does not need access to the original `FileSystem` or `StorageProvider`.

Because directory traversal accepts relative paths, possession of the directory does not inherently provide a way to restart resolution from the provider's absolute root.

This allows authority to be narrowed:

```text
StorageProvider
      │
      ▼
  Directory
      │
   ┌──┴──────────┐
   ▼             ▼
Directory       File
```

Code should generally receive the narrowest storage capability appropriate to its work.

---

# Absolute and relative resolution

`Path` itself supports both absolute and relative forms:

```text
/config/settings.json
config/settings.json
```

The meaning of those forms depends on the API receiving the path.

A `StorageProvider` establishes the namespace in which paths are interpreted and may define whether an operation accepts:

```text
absolute paths
relative paths
both
```

A `Directory`, by contrast, performs traversal relative to itself and therefore accepts relative paths.

This keeps path representation independent from storage authority.

---

# Storage units

Some storage systems contain independently rooted storage namespaces.

NeoCLR recognizes this as the concept of a `StorageUnit`.

Conceptually:

```raven
interface StorageUnit
{
    Name: String;
    Root: Directory;
}
```

A storage unit therefore provides a root:

```text
StorageUnit
     │
     ▼
    Root
     │
     ▼
 Directory
```

Depending on the provider, a storage unit might correspond to concepts such as:

```text
filesystem volume
logical drive
mounted filesystem
partition-backed filesystem
provider-defined storage namespace
```

A native filesystem might expose:

```text
FileSystem
   │
   ├── StorageUnit
   │      └── Root
   │
   └── StorageUnit
          └── Root
```

However, `StorageUnit` is **not required by the core storage abstraction**.

A package, archive, memory store, application sandbox, or remote provider should not invent storage units merely to satisfy an interface.

Storage units are exposed only when they represent something meaningful.

---

# Storage media

NeoCLR also recognizes the concept of an underlying `StorageMedium`.

A storage medium represents the backing medium rather than the addressable storage namespace.

Conceptually:

```raven
interface StorageMedium
{
    ...
}
```

A medium may contain or back one or more storage units:

```text
StorageMedium
      │
      ├── StorageUnit
      ├── StorageUnit
      └── StorageUnit
```

Possible examples include:

```text
physical disk
removable drive
optical medium
virtual disk
provider-specific storage device
```

This establishes an important distinction:

> A `StorageUnit` represents an independently rooted storage namespace.

> A `StorageMedium` represents something that backs storage.

The two concepts must not be conflated merely because operating systems sometimes use terms such as "drive" for both.

Storage media are optional concepts.

Providers without meaningful media semantics do not expose artificial media objects.

---

# Optional storage topology

The conceptual storage topology may therefore be:

```text
StorageMedium
      │
      │ backs
      ▼
 StorageUnit
      │
      │ provides
      ▼
     Root
      │
      ▼
  Directory
      │
      ├── Directory
      └── File
```

But this is **not** the required shape of every storage provider.

A filesystem may expose the full topology:

```text
StorageMedium
      │
 StorageUnit
      │
 Directory
      │
     File
```

An application store may expose only:

```text
Directory
   │
   ├── Directory
   └── File
```

A package may similarly expose:

```text
Package
   │
Resources
   │
   ├── Directory
   └── File
```

All are legitimate storage models.

The existence of a concept in the domain does not imply that every provider must expose it.

---

# Optional provider capabilities

The core `StorageProvider` should not accumulate members merely because some providers can expose additional topology.

For example, the core may remain:

```raven
interface StorageProvider
{
    func GetItem(path: Path)
        -> Result<StorageItem, StorageError>;

    func GetFile(path: Path)
        -> Result<File, StorageError>;

    func GetDirectory(path: Path)
        -> Result<Directory, StorageError>;
}
```

If there is a genuine need to abstract providers capable of enumerating storage units, that capability could later be represented separately:

```raven
interface StorageUnitProvider
{
    StorageUnits: Sequence<StorageUnit>;
}
```

Likewise for storage media:

```raven
interface StorageMediumProvider
{
    StorageMedia: Sequence<StorageMedium>;
}
```

These interfaces should not be introduced merely to complete a taxonomy.

They should exist when concrete APIs require those capabilities.

---

# Permissions

Permissions are a valid storage concept, but several different notions of permission must not be conflated.

At minimum there is a distinction between:

```text
effective capability
    What may this caller do through this object?

underlying storage permissions
    What permissions does the storage system itself represent?
```

For example, an underlying filesystem object may be writable according to native filesystem permissions while the application has deliberately received read-only access to it.

Possible effective operations include:

```text
read
write
create
delete
enumerate
rename
move
```

Provider-native permissions may include concepts such as:

```text
ACLs
Unix modes
ownership
security descriptors
provider-specific access policies
```

The exact permissions API should therefore be designed separately.

NeoCLR should not prematurely introduce one `Permissions` object that attempts to represent all of these concepts.

---

# Provider-specific semantics

Storage providers do not necessarily support identical operations.

A native filesystem may support:

```text
files
directories
creation
deletion
renaming
moving
links
timestamps
permissions
random access
locking
atomic operations
storage units
storage media
```

Package storage may be read-only.

Application storage may expose only a restricted directory tree.

Archive storage may expose directories conceptually even if directories are not independently stored.

Memory storage may have no physical medium or native path.

Remote storage may not support atomic moves or filesystem-style locking.

The Storage API should therefore avoid defining one enormous interface containing the union of all possible storage functionality.

> Shared abstractions should represent genuinely shared semantics.

Provider-specific functionality can remain provider-specific.

---

# Streams

Storage and streams are separate concerns.

Storage identifies stored objects and provides access to them.

Streams perform byte I/O.

For example:

```raven
let file =
    directory.GetFile(
        Path("data/index.bin"))?;

let stream =
    file.OpenRead()?;
```

The stream abstraction does not belong to `System.Storage` merely because files commonly produce streams.

Likewise:

```text
text encoding
JSON
XML
serialization
compression
cryptography
networking
```

can compose with storage without becoming part of the storage model.

---

# Error model

Storage operations use explicit errors.

For example:

```raven
func GetFile(path: Path)
    -> Result<File, StorageError>;
```

Possible storage errors may eventually include concepts such as:

```text
NotFound
AlreadyExists
AccessDenied
InvalidPath
ExpectedFile
ExpectedDirectory
ReadOnly
UnsupportedOperation
StorageUnavailable
```

The exact error hierarchy should be designed alongside the operations that produce those errors.

Provider-specific failures may require provider-specific error information without forcing every provider into native filesystem error semantics.

---

# Initial namespace

An initial namespace might contain:

```text
System.Storage

StorageProvider

StorageItem
File
Directory

Path

FileSystem

StorageError
FileMode
FileAccess
FileAttributes
```

Recognized concepts that may be introduced as the design develops include:

```text
StorageUnit
StorageMedium
Permissions
```

Their existence in the conceptual model does not require them to be part of the first implementation.

---

# Conceptual architecture

The portable core is:

```text
                       StorageProvider
                              │
                              │ Path
                              ▼
                        StorageItem
                       ╱           ╲
                      ▼             ▼
                   File         Directory
                                   │
                                   │ relative Path
                                   ▼
                              StorageItem
```

A richer filesystem environment may additionally expose:

```text
                     StorageMedium
                           │
                           │ may back
                           ▼
                      StorageUnit
                           │
                           │ provides
                           ▼
                          Root
                           │
                           ▼
                       Directory
                      ╱         ╲
                     ▼           ▼
                  Directory     File
```

These concepts describe additional storage topology. They do not form mandatory layers through which storage access must pass.

A simpler provider may expose only:

```text
                     StorageProvider
                           │
                           ▼
                       Directory
                      ╱         ╲
                     ▼           ▼
                  Directory     File
```

And code that has already been given a specific capability may see only:

```text
                       Directory
                      ╱         ╲
                     ▼           ▼
                  Directory     File
```

or simply:

```text
                          File
```

The API therefore does not require callers to understand the topology from which a storage object originated.

---

# Capability model

Storage objects naturally form capabilities.

Possession of a `StorageProvider` grants whatever storage access that provider represents.

Possession of a `Directory` grants access to that directory and the items reachable through its permitted relative paths.

Possession of a `File` grants access to that particular file according to the operations available on it.

This allows capabilities to be narrowed as they are passed through an application:

```text
                    StorageProvider
                           │
                           ▼
                       Directory
                           │
                     ┌─────┴─────┐
                     ▼           ▼
                 Directory      File
                     │
                     ▼
                    File
```

For example, an application may have access to its complete data directory:

```raven
let data = application.Data;
```

but a configuration subsystem can receive only:

```raven
let config =
    data.GetDirectory("config")?;

let loader =
    ConfigLoader(config);
```

A parser may receive an even narrower capability:

```raven
let file =
    config.GetFile("settings.json")?;

ParseConfiguration(file);
```

The parser does not need to know about the directory, provider, storage unit, medium, or native filesystem that ultimately backs the file.

This follows the principle:

> Give a component the storage capability it needs rather than the storage system from which that capability originated.

---

# Paths and capabilities

`Path` is an address value, not an authority.

For example:

```raven
let path =
    Path("/system/config.json");
```

possessing this value does not grant access to the referenced item.

A path becomes meaningful for storage access only when supplied to an appropriate capability:

```raven
let file =
    storage.GetFile(path)?;
```

Similarly:

```raven
let relative =
    Path("config/settings.json");

let file =
    directory.GetFile(relative)?;
```

The authority comes from `storage` or `directory`, not from the path.

Conceptually:

```text
Path
    describes where

StorageProvider / Directory
    determines what can be accessed
```

This separation prevents path strings from becoming implicit storage capabilities.

---

# Directory traversal

Directory traversal is always relative to the directory on which the operation is performed.

For example:

```raven
let root =
    storage.GetDirectory(Path("/application"))?;

let file =
    root.GetFile(
        Path("config/settings.json"))?;
```

Conceptually:

```text
/application
    │
    └── config
          │
          └── settings.json
```

The relative path is evaluated from `root`.

An absolute path remains a valid `Path` value:

```raven
let path =
    Path("/config/settings.json");
```

but cannot be supplied to an operation whose contract requires a relative path:

```raven
root.GetFile(path); // invalid for this operation
```

This does not make the `Path` invalid. It means the value does not satisfy the requirements of `Directory.GetFile`.

---

# Names

A direct child can be addressed by name without constructing a `Path`.

For example:

```raven
let file =
    directory.GetFile("settings.json")?;

let cache =
    directory.GetDirectory("cache")?;
```

A name represents exactly one child component.

It cannot contain path traversal.

Therefore:

```raven
directory.GetFile("settings.json");
```

is valid, while:

```raven
directory.GetFile("config/settings.json");
```

is not interpreted as a path.

The caller must explicitly construct a `Path`:

```raven
directory.GetFile(
    Path("config/settings.json"));
```

This preserves a clear distinction:

```text
String
    direct child name

Path
    structured path
```

The API therefore remains convenient for common operations without reverting to stringly typed path handling.

---

# Storage topology is descriptive

`StorageUnit` and `StorageMedium` describe storage topology when such topology exists.

They do not define the access model.

For example, a filesystem might expose:

```text
Physical SSD
    │
    ├── System volume
    │      └── Root
    │
    └── Data volume
           └── Root
```

which could be represented as:

```text
StorageMedium
    │
    ├── StorageUnit
    │      └── Directory
    │
    └── StorageUnit
           └── Directory
```

But application code operating on:

```raven
Directory
```

does not need to know any of this.

Similarly, a remotely backed provider might expose storage units without any meaningful storage medium, while another provider may expose neither.

Topology is therefore discoverable where useful rather than imposed on ordinary storage access.

---

# Querying additional information

**Author direction, 2026-09-23:** storage objects may expose ways to query additional
information or attributes, informed by System.IO and Windows Runtime. The common
Storage model must remain independent of any one backing system. This selects an
extension direction, not a query signature or a mandatory metadata inventory.

Queries should be reachable through the storage objects/capabilities a component
already has. Retrieving attributes must not require recovering a native filesystem
path or broadening access back to an unrestricted provider. An archive entry or
remote object may have useful information without having native filesystem flags.

Distinguish locally represented state (such as Name and Path) from information that
requires provider interaction. A metadata query may involve I/O, fail or be denied;
its availability and freshness must be part of the contract. Unsupported or absent
information must not silently become a fabricated zero, empty string or false value.
Attribute information is not, by itself, permission to perform an operation.

Typed common information, optional metadata interfaces and extensible property
queries are alternatives to evaluate against concrete consumers. Typed results
improve discoverability and checking but require shared semantics; extensible keys
accommodate provider-specific data but need naming, value-type and availability
rules. Neither a universal attribute enum nor a string/object property bag is
selected. Additional metadata should not add new StorageItem branches or expose
provider internals as part of the core object model.

**Comparison, checked 2026-09-23:** .NET FileSystemInfo exposes filesystem metadata;
[Refresh](https://learn.microsoft.com/en-us/dotnet/api/system.io.filesysteminfo.refresh?view=net-10.0)
updates its snapshot, illustrating why freshness needs a contract. Windows Runtime
separates [top-level, basic and extended properties](https://learn.microsoft.com/en-us/windows/apps/develop/files/file-properties).
Its [RetrievePropertiesAsync](https://learn.microsoft.com/en-us/uwp/api/windows.storage.fileproperties.storageitemcontentproperties.retrievepropertiesasync?view=winrt-28000)
accepts property names and returns a map of values, which can be null. Those are
useful reference approaches, not requirements to adopt filesystem-specific flags,
Windows property keys or the same caching scheme.

Implement the first useful query after the core interface/provider migration, with
explicit absence, failure and freshness behavior. Metadata enumeration, mutation
and a general query framework are not implied by this direction.

---

# Provider extensibility

Providers may extend the common storage model with capabilities specific to their domain.

For example, `FileSystem` may provide APIs for:

```text
links
mounts
volumes
native attributes
filesystem permissions
storage units
storage media
```

An archive provider might expose:

```text
compression information
archive entry metadata
```

A remote provider might expose:

```text
version identifiers
ETags
synchronization state
remote metadata
```

These concepts should not automatically migrate into `StorageItem` merely because one provider supports them.

The common model remains intentionally small:

```text
StorageProvider
StorageItem
File
Directory
Path
```

Additional APIs build around that core.

---

# Design principles

The NeoCLR Storage API follows these principles:

1. **Storage is broader than filesystems.**

   `FileSystem` is one kind of storage provider.

2. **Storage access is explicit.**

   Code accesses storage through provider, directory, or file capabilities rather than assuming ambient host filesystem access.

3. **Paths are values.**

   Storage paths are represented by `Path`, not arbitrary strings.

4. **Paths may be absolute or relative.**

   `Path` represents both forms. Individual APIs determine which forms they accept.

5. **Strings represent names, not paths.**

   String overloads on `Directory` address direct children. Traversal uses `Path`.

6. **Paths do not grant authority.**

   A `Path` identifies a location. A storage capability determines whether that location can be accessed.

7. **Directories support relative traversal.**

   A `Directory` can resolve descendants through relative `Path` values without granting access to an unrelated provider root.

8. **Storage objects can narrow authority.**

   Components can receive a directory or individual file instead of the provider from which it originated.

9. **Files and directories form the common object model.**

   `StorageItem` is a sealed interface hierarchy over the `File` and `Directory`
   interfaces; concrete implementations are supplied by providers.

10. **Topology is optional.**

    Storage units and media are recognized concepts but are exposed only where they are meaningful.

11. **Shared interfaces contain genuinely shared semantics.**

    Providers are not required to emulate filesystem operations that do not naturally apply to them.

12. **Provider-specific functionality remains provider-specific.**

    Native filesystems, archives, remote storage, and other providers can expose richer APIs without expanding the portable core.

13. **Storage and I/O composition remain separate.**

    Storage provides access to stored objects. Streams, encodings, serialization, compression, and similar facilities compose with those objects.

---

# Core principle

The traditional model is often:

```text
Application
    │
    ▼
FileSystem
    │
    ▼
Path
    │
    ▼
File / Directory
```

NeoCLR instead models:

```text
                  Storage capability
                         │
             ┌───────────┴───────────┐
             ▼                       ▼
        FileSystem             Other provider
             │                       │
             └───────────┬───────────┘
                         │
                         │ Path
                         ▼
                    StorageItem
                   ╱           ╲
                  ▼             ▼
               File         Directory
                                │
                                │ relative Path
                                ▼
                           StorageItem
```

Optional topology can exist behind or alongside this model:

```text
StorageMedium
      │
      ▼
 StorageUnit
      │
      ▼
     Root
      │
      ▼
  Directory
```

but ordinary application code does not need to know that topology exists.

The central abstraction is therefore not the physical filesystem or storage device.

It is **access to stored objects through explicit storage capabilities**.

`Path` describes where an object is located within a storage namespace.

`StorageProvider` provides access to such a namespace.

`Directory` provides narrower, relative access to part of that namespace.

`File` represents access to an individual stored file.

`StorageUnit` and `StorageMedium` describe additional topology where that topology is meaningful.

**NeoCLR models the storage that code can access without requiring that code to understand the machinery behind it.**


## Future task-returning operations — 2026-09-23

The author raised whether future Storage methods should return Task. The assistant
recommends evaluating Task<Result<T, StorageLookupError>> for provider GetItem,
GetFile and GetDirectory and directory GetItems, and Task<Result<T, StreamError>>
for opening/creating content streams. These operations may involve remote or other
latency-bearing providers. This is future direction, not a POC signature change.
[WinRT GetItemsAsync](https://learn.microsoft.com/en-us/uwp/api/windows.storage.storagefolder.getitemsasync?view=winrt-26100)
(primary documentation reviewed 2026-09-23) illustrates this separation; the current .NET directory
retrieval comparison above remains synchronous. A completed Task around blocking
work does not make it nonblocking.

Path.Parse and represented Name/Path properties should remain synchronous. Establish
execution/suspension policy, cancellation outcomes, buffer ownership and races with
Close before claiming asynchronous I/O. Do not assume a thread pool exists on every
platform or implement general storage concurrency by indiscriminately spawning
threads. The synchronous POC should explicitly document blocking behavior. Whether
to replace signatures or retain a separately named sync/async pair remains open.


### Pending-operation contract evidence

The [queued pending-read experiment](../experiments/pending-read/README.md) tests
Task/Promise cancellation through actual awaits and GC without changing Storage.
Its provisional rule is to keep the Task pending after a cancellation/close request
until a terminal producer event establishes modeled resource release. Completion
may win after the request; expected producer failure remains a Result error. This
separates cancellation of an operation from merely abandoning a wait. Native release
and concurrent backend races are not established by queued guest events. Private
owned buffers simplify this fixture, but owned-result versus caller-buffer APIs and
multiple-operation close behavior remain open. No public type or signature is selected.


The [host-backed experiment](../experiments/host-pending-read/README.md) now connects
that lifetime question to existing worker notifications. It chooses deferred discard
for its consumer: cancellation requested before invocation-side delivery waits for
host acknowledgement, then cancels without copying the result. Completion already
delivered is retained. This provides real Task/GC evidence but no producer interruption,
OS cancellation or public Storage signature decision.
