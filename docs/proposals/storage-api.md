# NeoCLR Storage API

## Overview

NeoCLR models **storage as an explicit platform capability**.

```raven
namespace System.Storage
```

`System.Storage` does not assume that an application has access to a host filesystem. A runtime can expose storage through different providers, and those providers may expose either complete storage systems or deliberately constrained areas.

The important abstraction is therefore not "the filesystem." It is **access to storage**.

## Storage providers

A storage provider supplies access to some storage domain.

Conceptually:

```text
Storage
   │
   ├── FileSystem
   │     └── host/native filesystem
   │
   ├── ApplicationStorage
   │     └── application-specific area
   │
   ├── PackageStorage
   │     └── application/package resources
   │
   ├── MemoryStorage
   │
   └── other providers
```

`FileSystem` is therefore a **storage provider**, not the root abstraction of the Storage API.

A provider may expose an entire filesystem:

```raven
let storage = FileSystem.Default;
```

but another provider might expose only a particular area:

```raven
let storage = application.Data;
```

or:

```raven
let storage = package.Resources;
```

Code consuming that storage should not have to assume that it corresponds to `/`, `C:\`, or any other host filesystem root.

## Storage areas

A useful fundamental concept is an accessible **area of storage**.

The exact name still needs to be decided. Possible vocabulary includes:

```text
StorageArea
StorageLocation
StorageScope
Container
Directory
```

The important semantic distinction is that an area represents **what the caller has access to**, rather than necessarily representing a directory in a global filesystem.

For example:

```raven
func LoadConfiguration(storage: StorageArea)
    -> Result<Config, ConfigError>
{
    let file = storage.GetFile(Path("config.json"))?;
    let text = file.ReadText()?;

    return Config.Parse(text);
}
```

The caller doesn't need to know whether `storage` represents:

```text
/home/user/myapp
C:\Users\User\AppData\...
a WASM application sandbox
an archive
package resources
an in-memory store
a remotely backed area
```

The path is resolved **relative to the supplied storage capability**.

## FileSystem as a provider

`FileSystem` represents conventional filesystem access:

```raven
interface FileSystem : StorageProvider
{
    ...
}
```

Its implementation can understand native filesystem concepts such as roots, volumes, absolute paths, links, filesystem attributes, and OS-specific behavior.

For example:

```raven
let fs = FileSystem.Default;

let home = fs.GetDirectory(Path("/home/user"))?;
```

But ordinary application components do not necessarily need a `FileSystem`.

Instead of:

```raven
class ConfigLoader
{
    init(fileSystem: FileSystem);
}
```

a component could receive only the storage area it actually needs:

```raven
class ConfigLoader
{
    let storage: StorageArea;

    init(storage: StorageArea)
    {
        self.storage = storage;
    }

    func Load() -> Result<Config, ConfigError>
    {
        let file = storage.GetFile(Path("config.json"))?;
        let text = file.ReadText()?;

        return Config.Parse(text);
    }
}
```

That is both a cleaner dependency and a stronger capability boundary.

## Capabilities are naturally scoped

This gives NeoCLR a useful capability model.

Giving code:

```raven
FileSystem
```

might mean:

> You can access the filesystem exposed by this provider.

Giving code:

```raven
StorageArea
```

might instead mean:

> You can access this storage area.

Giving code:

```raven
File
```

means:

> You can access this particular file.

Capabilities therefore become progressively narrower:

```text
FileSystem
     │
     ▼
StorageArea
     │
     ├── StorageArea
     │
     └── File
```

Code does not need global filesystem access merely because it needs to read one configuration file.

## Files and paths

The natural names remain:

```text
System.Storage

File
Directory
Path
FileSystem
FileSystemEntry
...
```

There is no requirement for names such as `StorageFile`.

A `File` is simply a file exposed through some storage provider.

Similarly, a `Path` is a value used to identify or navigate storage according to the semantics of a particular provider or storage area. It does not inherently mean a native host path.

Relative paths become particularly important:

```raven
let file = storage.GetFile(Path("config/settings.json"))?;
```

Here the path is interpreted within the supplied storage capability. It does not grant access outside that capability merely because the underlying implementation happens to use a native filesystem.

## Provider-specific semantics

Not every storage provider needs to expose exactly the same functionality.

A filesystem might support:

```text
directories
files
renaming
moving
links
permissions
timestamps
random access
```

Package storage might be read-only.

Application storage might prohibit navigating above its root.

Memory storage might have no meaningful native path.

Remote storage might not support atomic moves.

The Storage API should therefore avoid defining one enormous universal interface containing the union of every possible storage operation.

Shared abstractions should represent **genuinely shared semantics**, while provider-specific capabilities can remain provider-specific.

## Streams remain cross-cutting

Storage identifies and provides access to stored objects.

Streams handle byte I/O.

For example:

```raven
let file = storage.GetFile(Path("data.bin"))?;
let stream = file.OpenRead()?;
```

Storage does not own the entire stream abstraction merely because files commonly produce streams.

Likewise, text encoding, JSON, XML, serialization, compression, and networking can compose with storage without becoming part of `System.Storage`.

## Initial namespace

The namespace can remain relatively flat while the API develops:

```text
System.Storage

File
Directory
Path

StorageArea
StorageProvider

FileSystem

StorageError
FileMode
FileAccess
FileAttributes
```

The names `StorageArea` and `StorageProvider` are provisional. Their exact shape should emerge from the provider model rather than being introduced simply to complete a taxonomy.

## Core principle

The central model is no longer:

```text
Application
    ↓
FileSystem
    ↓
File / Directory
```

It is:

```text
                Storage capability
                       │
            ┌──────────┴──────────┐
            │                     │
       FileSystem            Other providers
            │                     │
            └──────────┬──────────┘
                       ▼
              Accessible storage
                       │
                 ┌─────┴─────┐
                 ▼           ▼
               File        Area
```

`FileSystem` is therefore **a provider**, not the definition of storage.

This allows NeoCLR to expose storage according to the capabilities of the environment: anything from an entire native filesystem to one tightly constrained application area or individual resource.