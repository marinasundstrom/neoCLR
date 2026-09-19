Yes. I think that fits the direction we've been taking with NeoCLR very well. Rather than treating `File` and `Directory` as globally available static gateways into the machine, I'd make the **file system itself an explicit capability**.

Conceptually:

```raven
namespace System.IO

interface FileSystem
{
    func Open(path: Path, mode: FileMode) -> Result<FileStream, IOError>;
    func FileExists(path: Path) -> bool;
    func DirectoryExists(path: Path) -> bool;

    func CreateDirectory(path: Path) -> Result<Directory, IOError>;

    func GetFile(path: Path) -> Option<File>;
    func GetDirectory(path: Path) -> Option<Directory>;

    func Enumerate(path: Path) -> Result<Iterable<FileSystemEntry>, IOError>;
}
```

Then the runtime provides the actual implementation:

```raven
let fs = FileSystem.Default;

let config = fs.GetFile(Path("config.json"))?;

let text = config.ReadText()?;
```

But application code can receive it as a dependency:

```raven
class ConfigLoader
{
    let fileSystem: FileSystem;

    init(fileSystem: FileSystem)
    {
        self.fileSystem = fileSystem;
    }

    func Load(path: Path) -> Result<Config, ConfigError>
    {
        let file = fileSystem.GetFile(path)?;
        let text = file.ReadText()?;
        return Config.Parse(text);
    }
}
```

For a test, you can supply an in-memory implementation without pretending that arbitrary static `File` methods are somehow mockable:

```raven
let fs = MemoryFileSystem();

fs.WriteText(
    Path("/config.json"),
    """{ "enabled": true }"""
);

let loader = ConfigLoader(fs);
let config = loader.Load(Path("/config.json"))?;
```

There is another important benefit here beyond testing: **`FileSystem` gives filesystem objects a context**.

A `Path` doesn't have to mean "a thing on the host OS." It's just a path. Its interpretation belongs to a particular filesystem:

```text
Path
  │
  ▼
FileSystem
  ├── HostFileSystem
  ├── MemoryFileSystem
  ├── ZipFileSystem
  ├── EmbeddedFileSystem
  └── potentially RemoteFileSystem
```

That makes the abstraction useful architecturally rather than merely being an `IFileSystem` added for dependency injection.

I'd actually distinguish three concepts quite deliberately:

**`Path`** is a pure value. It can be manipulated without touching I/O.

```raven
let path = Path("/users/marina/data.json");

path.Name        // "data.json"
path.Extension   // ".json"
path.Parent
path / "child"
```

**`FileSystem`** represents the namespace and operations through which paths are resolved.

**`File`, `Directory`, and perhaps `FileSystemEntry`** are objects/handles obtained from a filesystem.

That allows something rather pleasant:

```raven
let dir = fs.Directory(Path("/data"))?;

for entry in dir.Entries()?
{
    match entry
    {
        File file =>
            Console.WriteLine(file.Name);

        Directory directory =>
            Console.WriteLine($"{directory.Name}/");
    }
}
```

And I'd avoid copying .NET's split where you have `File`, `FileInfo`, `Directory`, `DirectoryInfo`, `Path`, and static methods scattered among them. That split carries a lot of historical baggage.

NeoCLR could instead have a much cleaner model:

```text
System.IO

FileSystem
├── FileSystem.Default
├── Open(...)
├── File(...)
├── Directory(...)
├── Exists(...)
├── Move(...)
├── Copy(...)
└── Delete(...)

FileSystemEntry
├── File
└── Directory

Path
FileStream
IOError
FileMode
FileAccess
FileAttributes
```

One thing I'd be careful about is **not making `FileSystem` enormous**. It should represent filesystem semantics, while streams remain the byte-I/O abstraction. `File.ReadText()` can be ergonomic convenience built over `Open()`, rather than forcing text, encoding, buffering, etc. into the core filesystem interface.

This also continues a pattern that's emerging in NeoCLR: instead of static ambient runtime state, we expose a **runtime service/capability with a sensible default**. Production code can conveniently say `FileSystem.Default`, while code that cares about isolation can receive a `FileSystem`. That's a considerably nicer foundation than designing mocking as an afterthought.
