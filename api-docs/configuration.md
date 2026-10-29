---
title: Configuration
---
# Configuration (development)

Import `Neo.Configuration.*`. Reference `Neo.Configuration.Abstractions` for
consumer contracts; `Neo.Configuration` supplies defaults and DI integration.
Neither assembly depends on the default DI container.

| API | Members and behavior |
| --- | --- |
| Configuration | Mutable `[key: string]: string?`, GetSection(key), GetChildren(). Missing and explicit null both read null. Set writes every provider; an empty root faults on Set. |
| ConfigurationSection | Configuration plus Key, Path and mutable Value. Nonexistent sections are live objects; GetChildren returns immediate descendants. |
| ConfigurationRoot | Configuration plus Disposable, Providers and Reload(). Default roots load each provider on construction; dispose each disposable registration once per root lifetime. |
| ConfigurationBuilder | Sources snapshot, Add(source) returning the builder, Build() returning an independently owned root. |
| ConfigurationSource | Build(builder) creates a provider owned by the returned root. |
| ConfigurationProvider | `Find(key): Optional<string?>` distinguishes absence from present null; Set(key,value), Load(), `GetChildKeys(parentPath: string?): Sequence<string>`. Null parent means root; empty string denotes an empty section path. |
| ConfigurationEntry | Immutable Key and nullable Value, constructed with `(key, value)`. |
| DefaultConfigurationBuilder | Parameterless constructor. Stores sources; Build constructs and loads fresh providers in registration order. Sources returns a snapshot. |
| DefaultConfigurationRoot | Constructor `(providers: Sequence<ConfigurationProvider>)` copies and owns registrations. Later providers win, including Some(null). Providers returns a snapshot of references. Reload calls each Load. Dispose is idempotent; existing sections share its lifetime. |
| DefaultConfigurationSection | Constructor `(root: DefaultConfigurationRoot, path: string)`; Key is the final segment, Path is the full path, Value/indexer delegate to the root. Child sections are live. |
| MemoryConfigurationSource | Constructor `(initialData: Sequence<ConfigurationEntry>)` snapshots entries. Build creates a new MemoryConfigurationProvider. |
| MemoryConfigurationProvider | Constructor with initialData; copies key/value data. Find/Set use ordinal-ignore-case keys; repeated keys replace earlier values. Load does nothing, preserving in-memory mutations. GetChildKeys returns sorted distinct immediate keys. |
| MemoryConfigurationExtensions | `ConfigurationBuilder.AddInMemoryCollection(initialData)` snapshots input and appends a source; returns the builder. |
| ConfigurationExtensions | `Configuration.GetConnectionString(name): string?` reads `ConnectionStrings:name`. |
| ConfigurationSectionExtensions | Exists() means non-null Value or at least one child. Empty strings exist; a null-only leaf does not. |
| ConfigurationServices | `ServiceCollection.AddConfiguration(configuration): Result<unit, ServiceError>` preserves an existing Configuration registration. The supplied instance remains caller-owned. |

Paths use `:`; numeric Int32 child segments sort before text. Values and children
are independent. Child sequences are snapshots, while sections reflect current
values after writes or reloads. Key comparison follows runtime OrdinalIgnoreCase;
Unicode normalization is not applied. No concurrent access is supported.

The [integrated sample](https://github.com/marinasundstrom/neoCLR/blob/main/samples/extensions/README.md)
registers a caller-owned configuration root, reads its localization directory at
startup and injects Configuration into GreetingService to select the greeting name.
Dispose the container before its borrowed configuration root.

Change tokens, automatic reload, environment/command-line configuration providers,
binding and options are not implemented. Load/Set/source faults propagate;
no rollback is promised. See [design and .NET comparison](https://github.com/marinasundstrom/neoCLR/blob/main/docs/configuration.md).


## Try it

Build configuration from appsettings.json, then read a colon-delimited key. The file is read during Build; `use` disposes the root.

Run from `samples/extensions`, containing this appsettings.json:

```json
{
  // Configuration accepts comments and trailing commas, like Microsoft's provider.
  "Greeting": { "Name": "Ada", },
  "Localization": { "Directory": "resources", },
}
```

```raven
module Examples.Extensions.Introductions.Configuration

import System.*
import Neo.Configuration.*

func Main() -> int {
    let builder: ConfigurationBuilder = DefaultConfigurationBuilder()
    _ = builder.AddJsonFile("appsettings.json")
    use configuration = builder.Build()
    Console.WriteLine(configuration["Greeting:Name"] ?? "world")
    return 0
}
```

[Complete sample project](https://github.com/marinasundstrom/neoCLR/tree/main/samples/extensions/src/ConfigurationIntro) · [Build and run instructions](https://github.com/marinasundstrom/neoCLR/blob/main/samples/extensions/README.md#introductory-examples)

## JSON file configuration

Reference the optional `Neo.Configuration.Json` assembly. `AddJsonFile` is in
Neo.Configuration; provider/source/error types are in Neo.Configuration.Json.
The abstractions and base implementation do not reference System.Data.

| API | Members and behavior |
| --- | --- |
| JsonConfigurationExtensions | `builder.AddJsonFile(path)` adds a required file; `(path, optional)` ignores only missing files when optional. Returns the builder; no I/O until Build. |
| JsonConfigurationSource | Constructor `(path, optional)`, read-only Path/Optional and Build(builder) producing a fresh provider. |
| JsonConfigurationProvider | Constructors `(path)` and `(path, optional)`; Path/Optional, Find, Set, GetChildKeys and Load. Relative paths use the working directory. Set changes memory only. Load faults on failure; Reload through the root rereads the file. |
| JsonConfigurationProvider.TryLoad | Returns `Result<unit, JsonConfigurationError>`. Failed reads/parsing preserve previous values. A missing optional file replaces values with an empty provider. |
| JsonConfigurationProvider.LoadText | Parses supplied text with the same rules and atomic replacement; future Load rereads Path. Returns the same Result. |
| JsonConfigurationError | File(FileReadError), Json(JsonError), InvalidRoot and DuplicateKey(string). Duplicate flattened keys compare ignoring case. |

Objects use colon keys and arrays numeric segments. Strings decode JSON escapes;
numbers retain their text, booleans become True/False, null stays null, empty
objects yield null and empty arrays yield an empty string. The root must be an
object. The provider accepts line/block comments, trailing commas and an initial
UTF-8 BOM, using the public [JSON document options](json.html#document-parsing-options).
Input is limited to **1 MiB and 64 container levels**; there is no fixed value or
per-container entry limit. File watching remains unsupported. Size/depth/syntax
failures preserve prior values; they never truncate input. TryLoad reports
File(TooLarge) for oversized files; LoadText reports Json(LimitExceeded).
The sample's [appsettings.json](https://github.com/marinasundstrom/neoCLR/blob/main/samples/extensions/appsettings.json)
sets Greeting:Name and Localization:Directory.
