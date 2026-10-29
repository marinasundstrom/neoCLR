# Neo.Configuration foundation (development)

The author requested Microsoft.Extensions.Configuration as the baseline, separate
abstractions and integration into the existing DI sample (2026-10-29).

`Neo.Configuration.Abstractions` references only System.Runtime. Its contracts
are Configuration, ConfigurationSection, ConfigurationRoot, ConfigurationBuilder,
ConfigurationSource and ConfigurationProvider, plus immutable ConfigurationEntry
inputs and provider-neutral helpers. `Neo.Configuration` adds default root/section/
builder implementations, an in-memory source/provider and DI registration. It
references its abstractions and Neo.DependencyInjection.Abstractions, not the DI
container. The optional `Neo.Configuration.Json` project adds file configuration through
public System.Data.Json APIs without a friend-assembly grant. All three projects
share the component's `src` folder; its Raven tests live
under `test/Neo.Configuration.Tests`.

## Baseline and decisions

The implementation follows .NET 10's
[configuration root](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/Microsoft.Extensions.Configuration/src/ConfigurationRoot.cs),
[sections](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/Microsoft.Extensions.Configuration/src/ConfigurationSection.cs)
and [provider model](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/Microsoft.Extensions.Configuration/src/ConfigurationProvider.cs).
Later providers override earlier ones; explicitly present null masks lower values.
Colon paths represent hierarchy, including nodes with both values and children.
Sections are live views and never null. Writes reach all providers and do not imply
persistence. In-memory Reload preserves changes; custom providers choose how Load
refreshes data. Immediate children are distinct ignoring case and sort numeric
Int32 segments before text. Empty path segments remain representable.

Raven interfaces omit the I prefix, so concrete implementations use Default names.
Provider Find returns Optional<string?> instead of bool/out: None is absent and
Some(null) is present. GetChildKeys takes only the parent path; the root owns merge,
deduplication and ordering instead of asking providers to merge earlier keys. This
reduces provider obligations but is not a signature-compatible .NET port.
Builder Sources is an inspection snapshot rather than .NET's mutable IList; Add
is the supported mutation path in this slice. ConfigurationRoot explicitly extends Disposable so callers can express lifetime
through the abstraction. AddConfiguration preserves an existing Configuration
registration and borrows the supplied instance; the caller disposes its root after
the container. The root owns each provider registration, including duplicates.

Case-insensitive operations use the runtime's OrdinalIgnoreCase contract, including
its documented Unicode simple-fold behavior; they do not normalize Unicode.
Colon scanning uses actual UTF-8 boundaries even when case-equivalent parent names
have different byte lengths. The internal substring helper temporarily uses the
runtime's obsolete SliceUtf8 API; replace it when the public unbounded decoding/view
replacement is available. This is implementation debt, not a new string contract.
The initial memory provider uses linear lookup and copied entry sequences. No
performance improvement or concurrency guarantee is claimed.

## Scope and gaps

Access is synchronous and owner-thread serialized. Source Build and provider Load/
Set failures propagate using the runtime's terminal fault behavior; transactional
reload, rollback and recoverable provider-load errors are not implemented. Writes
with no providers and reads/writes/reloads through a disposed default root fault.
No IChangeToken equivalent or GetReloadToken is supplied yet. Automatic reload,
environment/command-line providers, file watching, binder, options, mutable configuration
manager, debug views and builder Properties are gaps, not claimed replacements.
The sample reads configuration from appsettings.json and uses JSON localization
resources. No artifact-folder change or automatic logging configuration
binding is included.

See the [API reference](../api-docs/configuration.md),
[integrated sample](../samples/extensions/README.md) and
[focused qualification](validation/neo-configuration-20261029/README.md).

## appsettings.json provider

The author additionally requested appsettings.json loading. The optional
Neo.Configuration.Json assembly provides JsonConfigurationProvider/Source and
AddJsonFile. Required files are the default; optional ignores only NotFound. Paths
are explicit and relative paths resolve against the current working directory.
No automatic directory search, environment suffix selection or watching occurs.

Objects flatten through colon paths and arrays through zero-based numeric segments.
The root must be an object. Scalars become strings; number spelling is retained,
booleans use True/False, null stays null, empty objects become null and empty arrays
become empty strings, following the [.NET 10 JSON configuration parser](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/Microsoft.Extensions.Configuration.Json/src/JsonConfigurationFileParser.cs).
Case-insensitive flattened collisions are rejected. This also rejects colliding empty
containers rather than silently overwriting them. JsonConfigurationError preserves
file and JSON errors and distinguishes InvalidRoot and DuplicateKey. TryLoad and
LoadText return Results and replace data only on success; Load faults on failure.
An optional missing-file reload clears prior data. Root reload is not transactional
across providers. Writes modify only provider memory; reloading restores file values.

This initial provider reuses the public, bounded strict-JSON DOM: at most 1024 UTF-8
bytes, 32 values including the root, four nested container levels and 31 entries
per container. Larger appsettings files return quota errors. Comments, trailing
commas and BOM handling are gaps relative to .NET's file provider. These are parser
limitations, not intended permanent configuration contracts. Raising these limits
or admitting relaxed syntax should be a public JSON parser capability with its own
validation, not an internal friend grant or silent truncation.
