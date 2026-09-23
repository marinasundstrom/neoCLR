# NeoCLR Storage API — Asynchronous Operations and File I/O Extension

## Status

Extension proposal to the core `System.Storage` object model.

The author selected the interface-based object model on 2026-09-23: StorageItem
has File and Directory as its two permitted interface branches, with provider-specific
implementations. GetItems enumerates StorageItem values. That interface hierarchy is integrated in development; provider-resolution and
enumeration operations still need alignment. Preview 9 is unchanged.
Async signatures and enumeration strategy below remain proposals. In particular,
the current Task implementation has no faulted state; the conceptual fault examples
below do not describe a shipped Task outcome.

This proposal does not redefine:

```text
StorageProvider
StorageItem
File
Directory
Path
StorageUnit
StorageMedium
```

Instead, it considers how operations over those objects interact with NeoCLR's task model and how files may subsequently expose byte I/O.

The two primary questions are:

1. Which storage operations should be asynchronous?
2. How should a resolved `File` be opened for reading or writing?

The second question is intentionally secondary. The storage object model should not depend on having a complete stream or file-opening API.

---

# Motivation

The core Storage API deliberately allows providers other than native filesystems.

A `StorageProvider` might represent:

```text
native filesystem
application storage
package resources
memory storage
archive contents
remote storage
virtual storage
```

This creates an important constraint.

An operation such as:

```raven
storage.GetFile(path)
```

may look like a simple object lookup, but resolving the path may require actual I/O.

For a memory provider this may be effectively immediate.

For a native filesystem it may require interaction with the operating system.

For a remote provider it may require network communication.

The common API should therefore avoid assuming that storage resolution is always local, immediate, or inexpensive.

---

# Asynchronous provider operations

Operations that may require interaction with the underlying storage provider should generally be task-based.

For example:

```raven
interface StorageProvider
{
    func GetItem(path: Path)
        -> Task<Result<StorageItem, StorageError>>;

    func GetFile(path: Path)
        -> Task<Result<File, StorageError>>;

    func GetDirectory(path: Path)
        -> Task<Result<Directory, StorageError>>;
}
```

Calling code remains straightforward:

```raven
let file =
    await storage.GetFile(
        Path("/config/settings.json"))?;
```

A provider that can complete the operation immediately may return an already-completed task.

The API therefore does not require every operation to involve asynchronous work. It merely permits implementations to perform asynchronous work without blocking the caller.

---

# Directory resolution

The same principle applies to traversal through a `Directory`.

Conceptually:

```raven
interface Directory : StorageItem
{
    func GetItem(name: String)
        -> Task<Result<StorageItem, StorageError>>;

    func GetFile(name: String)
        -> Task<Result<File, StorageError>>;

    func GetDirectory(name: String)
        -> Task<Result<Directory, StorageError>>;

    func GetItem(path: Path)
        -> Task<Result<StorageItem, StorageError>>;

    func GetFile(path: Path)
        -> Task<Result<File, StorageError>>;

    func GetDirectory(path: Path)
        -> Task<Result<Directory, StorageError>>;
}
```

The existing path rules remain unchanged.

Strings represent direct child names:

```raven
let file =
    await directory.GetFile("settings.json")?;
```

`Path` represents structured traversal:

```raven
let file =
    await directory.GetFile(
        Path("config/settings.json"))?;
```

Paths supplied to `Directory` traversal are relative paths.

Asynchrony changes how the operation completes, not how paths are interpreted.

---

# Asynchrony belongs to operations, not objects

Storage objects themselves are not inherently asynchronous.

For example:

```raven
file.Name
file.Path

directory.Name
directory.Path

path.IsAbsolute
path.IsRelative
```

should remain ordinary synchronous access when the information is already represented by the object.

The distinction is:

> Operations requiring interaction with storage may return tasks. Operations over already available state remain synchronous.

This prevents APIs from becoming task-based merely because the object ultimately originated from an asynchronous provider.

---

# Task and Result semantics

NeoCLR distinguishes asynchronous execution from expected operation failure.

A storage operation therefore naturally has the form:

```raven
Task<Result<T, StorageError>>
```

The two layers represent different concerns.

`Task` describes execution:

```text
pending
completed
cancelled
faulted
```

`Result` describes the expected result of the storage operation:

```text
Ok(T)
Error(StorageError)
```

For example:

```raven
let file =
    await storage.GetFile(path)?;
```

may result in:

```text
successful task
    └── Ok(File)

successful task
    └── Error(NotFound)

cancelled task

faulted task
```

A missing file is therefore not a task fault.

An access denial is not a task fault.

A read-only provider rejecting a mutation is not a task fault.

These are expected storage errors represented through `Result`.

Cancellation belongs to the task execution model.

Faults remain reserved for hard failures that cannot be represented as normal recoverable storage outcomes.

---

# Cancellation

Task-based storage operations naturally participate in NeoCLR's cancellation model.

For example:

```raven
let file =
    await storage.GetFile(path)?;
```

can be cancelled through the surrounding task context according to the general Task API.

The Storage API should not need to invent a separate storage-specific cancellation mechanism.

This keeps cancellation orthogonal to storage errors:

```text
NotFound
AccessDenied
UnsupportedOperation
StorageUnavailable
```

are storage results, while:

```text
Cancelled
```

is a task outcome.

---

# Mutation operations

The same reasoning applies to operations that modify storage.

Future APIs may include operations such as:

```raven
directory.CreateFile(...)
directory.CreateDirectory(...)

item.Delete()

file.Copy(...)
file.Move(...)

directory.Move(...)
```

Where those operations require provider interaction, they should generally be task-based:

```raven
func CreateDirectory(name: String)
    -> Task<Result<Directory, StorageError>>;

func Delete()
    -> Task<Result<void, StorageError>>;
```

The exact mutation API is outside the scope of this extension.

The important rule is that provider interaction should not silently require blocking merely because a particular implementation happens to be local.

---

# Enumeration

Directory enumeration also potentially involves asynchronous provider interaction.

A simple design could initially return:

```raven
func GetItems()
    -> Task<Result<Sequence<StorageItem>, StorageError>>;
```

However, this requires the provider to materialize the complete result.

Some storage systems may naturally support incremental or asynchronous enumeration.

A future async-sequence model could instead allow something conceptually similar to:

```raven
func GetItems()
    -> AsyncSequence<Result<StorageItem, StorageError>>;
```

The exact design should follow NeoCLR's eventual asynchronous iteration model.

This proposal therefore does not commit the Storage API to either eager or streaming enumeration.

---

# File I/O

A resolved `File` identifies and grants access to a particular stored file.

Actual byte I/O is a separate concern.

Conceptually:

```text
Storage
    resolves the file

Streams
    perform byte I/O
```

A future API may allow:

```raven
let file =
    await directory.GetFile("data.bin")?;

let stream =
    await file.OpenRead()?;
```

and:

```raven
let stream =
    await file.OpenWrite()?;
```

Opening a file may itself require provider interaction and should therefore be capable of asynchronous completion.

---

# OpenRead and OpenWrite

Convenience operations may include:

```raven
interface File : StorageItem
{
    func OpenRead()
        -> Task<Result<Stream, StorageError>>;

    func OpenWrite()
        -> Task<Result<Stream, StorageError>>;
}
```

These provide simple common cases without requiring callers to construct detailed opening options.

For example:

```raven
let file =
    await directory.GetFile("data.bin")?;

let stream =
    await file.OpenRead()?;
```

The resulting stream belongs to the general stream abstraction rather than to the Storage API itself.

---

# General file opening

More advanced scenarios may eventually require a general opening operation.

Conceptually:

```raven
file.Open(...)
```

with options describing concepts such as:

```text
access
creation mode
truncation
append
sharing
random/sequential access
buffering
locking
```

Possible supporting types might include:

```text
FileMode
FileAccess
FileShare
FileOpenOptions
```

However, these concepts should not be finalized as part of the core storage object model.

They depend heavily on the eventual stream model and on determining which semantics are genuinely portable across storage providers.

---

# Provider differences

Not every provider can support every file-opening mode.

For example:

```text
native filesystem
    read/write
    random access
    creation
    truncation
    append

package storage
    read-only

archive storage
    potentially read-only
    potentially sequential

remote storage
    provider-specific write semantics
    potentially no random access

memory storage
    potentially complete read/write support
```

The common API should therefore avoid pretending that every `File` behaves exactly like an operating-system file handle.

Unsupported operations should be represented explicitly through the storage error model.

---

# Convenience reads and writes

Higher-level convenience operations may eventually exist:

```raven
let bytes =
    await file.ReadBytes()?;

await file.WriteBytes(data)?;
```

Text operations may similarly be provided:

```raven
let text =
    await file.ReadText()?;

await file.WriteText(text)?;
```

Whether these belong directly on `File`, in extensions, or in higher-level I/O APIs should be decided separately.

They are conveniences over the fundamental storage and stream models rather than requirements for representing a file.

---

# Separation of concerns

The resulting architecture can be viewed as three layers:

```text
┌─────────────────────────────────────┐
│         Storage object model        │
│                                     │
│ StorageProvider                     │
│ Path                                │
│ StorageItem                         │
│ File                                │
│ Directory                           │
└──────────────────┬──────────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐
│         Storage operations          │
│                                     │
│ resolution                          │
│ enumeration                         │
│ creation                            │
│ deletion                            │
│ copy / move                         │
│ metadata operations                 │
│                                     │
│ typically Task<Result<T, Error>>    │
└──────────────────┬──────────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐
│              File I/O               │
│                                     │
│ OpenRead                            │
│ OpenWrite                           │
│ Open                                │
│ Stream                              │
│ read / write                        │
└─────────────────────────────────────┘
```

The layers compose, but they should not be designed as one monolithic abstraction.

---

# Design principle for asynchronous storage

The general rule proposed here is:

> An operation that may require interaction with a storage provider should be capable of asynchronous completion.

This includes operations such as:

```text
path resolution
directory lookup
enumeration
creation
deletion
copying
moving
opening files
reading provider-backed metadata
```

It does not include ordinary access to state already represented locally:

```text
Path.IsAbsolute
Path.Name
File.Name
File.Path
Directory.Name
Directory.Path
```

This allows providers to range from entirely in-memory implementations to remote storage systems without changing the common programming model.

---

# Open questions

This extension intentionally leaves several areas unresolved.

### Async-only versus synchronous counterparts

NeoCLR must decide whether storage operations should be async-first/async-only:

```raven
func GetFile(path: Path)
    -> Task<Result<File, StorageError>>;
```

or whether explicitly synchronous variants should also exist.

The default should avoid forcing providers that naturally require asynchronous I/O to block.

### Enumeration

Directory enumeration depends on the eventual async-sequence model.

The API should avoid prematurely committing to materialized collections if incremental asynchronous enumeration is expected to become a platform primitive.

### File opening

The relationship between:

```text
OpenRead
OpenWrite
Open
```

and the general stream model requires further design.

### Random access

It remains to be decided whether random-access file operations belong directly on `File`, on a specialized stream, or on another capability.

### File creation

The relationship between:

```text
Directory.CreateFile(...)
```

and:

```text
File.Open(...)
```

must be clarified so that creation and opening semantics are not duplicated unnecessarily.

### Provider capabilities

Some operations may eventually be represented through capability interfaces rather than returning `UnsupportedOperation` for functionality that a provider fundamentally cannot perform.

That decision should be driven by concrete API use cases.

---

# Proposed direction

The Storage API should proceed in stages.

The first stage defines the storage object model:

```text
StorageProvider
Path
StorageItem
File
Directory
```

The second stage defines provider operations using the NeoCLR task model:

```raven
Task<Result<T, StorageError>>
```

The third stage integrates `File` with the stream and byte-I/O model.

This sequencing allows the fundamental meaning of storage objects to remain stable while asynchronous execution and I/O semantics evolve independently.

The central rule is:

> **Storage objects describe accessible storage. Tasks describe interaction with that storage. Streams describe byte I/O.**

These concepts compose, but none needs to subsume the others.