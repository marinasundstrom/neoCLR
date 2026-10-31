---
title: API reference
toc: false
---
# API reference

Browse the neoCLR class library: modules, types, functions and constants, with
Raven signatures and member contracts. Modules form namespaces and are the primary
way the library is organized.

[Browse all modules](api/) · [Module guide](namespaces.md) · [Feature guides](/guides/)

Use the API browser to find a module or type, or search for a name. Module pages
list their types, functions and constants; type pages list their members. Open a
member for parameters, return values, errors and usage notes.

Media type structure, typed presets and Content-Type parsing are available in the
[HTTP module](api/System/Web/Http/index.html) (development).

## Find an API

| Area | Starting points |
| --- | --- |
| Text and numbers | [String](xref:System.String) · [Character](/docs/api/System/Character/index.html) · [System.Math](xref:System.Math) · [Text and numeric contracts](text-numbers.md) |
| Collections and queries | [System.Collections](xref:System.Collections) · [System.Linq](xref:System.Linq) · [Arrays](arrays.md) · [Mutable entry helpers](collection-entries.md) · [Interface variance](interface-variance.md) |
| Outcomes and values | [Optional](xref:System.Optional`1) · [Result](xref:System.Result`2) · [Object and value contracts](objects.md) · [Tuples](tuples.md) |
| Error diagnostics | [Error and ContextError](error-context.md) · [Error handling](/features/error-handling/) |
| Functions | [Function shapes and objects](functions.md) · [Callbacks](callbacks.md) |
| Gzip compression | [GZipStream](xref:System.IO.Compression.GZipStream) · [Compression contracts](compression.md) |
| Streams and files | [System.IO](xref:System.IO) · [System.Storage](xref:System.Storage) · [Stream contracts](streams.md) · [File watching](file-system-watcher.md) |
| Console and environment | [Console](xref:System.Console) · [Environment](xref:System.Environment) · [Standard streams](console.md) |
| Tasks and concurrency | [System.Concurrency.Tasks](xref:System.Concurrency.Tasks) · [System.Concurrency](xref:System.Concurrency) |
| Dependency injection | [Registration, resolution and lifetimes](dependency-injection.md) · Neo.DependencyInjection / Neo.DependencyInjection.Abstractions |
| Configuration | [Providers, hierarchy and DI](configuration.md) · Neo.Configuration / Neo.Configuration.Abstractions |
| Logging | [Structured state, factories and providers](logging.md) · Neo.Logging / Neo.Logging.Abstractions |
| Localization | [Catalogue loading and fallback](localization.md) · Neo.Localization / Neo.Localization.Abstractions |
| Dates and globalization | [System.Time](xref:System.Time) · [System.Globalization](xref:System.Globalization) |
| HTTP header views | [Header collection, names and safe lookup](http-headers.md) |
| Experimental web hosting | [Neo.Web builders, context handlers and lifetime](neo-web.md) |
| Networking and HTTP | [System.Networking](xref:System.Networking) · [System.Web.Http](xref:System.Web.Http) · [Sockets](sockets.md) · [HTTP server](http-server.md) · [Body streams](http-body-streams.md) · [Routes](routes.md) |
| Cryptography | [Hashing and secure random bytes](cryptography.md) · System.Security.Cryptography |
| SQL databases | [SQL sessions and mapping](sql.md) · System.Data.Sql · System.Data.PostgreSql |
| JSON | [System.Data.Json](xref:System.Data.Json) · [Serialization and HTTP content](json.md) |
| Native libraries | [Foreign-function attributes](native-interop.md) · [NativeImportAttribute](xref:System.Runtime.InteropServices.NativeImportAttribute) · [MarshalAsAttribute](xref:System.Runtime.InteropServices.MarshalAsAttribute) |
| Runtime diagnostics | [StackTrace](xref:System.Diagnostics.StackTrace) · [Capture and limits](stack-trace.md) · [GC](gc.md) |
| Introspection and reflection | [System.Introspection](xref:System.Introspection) · [Descriptor contracts](introspection.md) · [Runtime reflection](reflection.md) · [Dynamic member requirements](dynamic-access.md) |

Development interpreter APIs: [System.Slice, ReadOnlySlice, Memory and ReadOnlyMemory](memory-views.md).

## Using the reference

This reference follows the **development class library**. Individual pages describe
concrete limits and identify APIs that differ from the published preview. Use the
[matching toolchain](/try/#development) when trying development APIs.

Signatures use Raven notation. The [structural type families](structural-types.md)
guide explains Array, Function, Tuple, Union and Intersection. The
[reference support notes](reference-support.md) cover signatures that need a manual
entry or additional explanation.

For an introduction and working examples, start with the [feature guides](/guides/).
The reference provides the detailed contracts you need while writing code.

## Runtime and tooling APIs

These references are for runtime control, host integration and compiler tooling:

- [Garbage collection](gc.md): object counters, explicit collection and KeepAlive.
- [Runtime failures](faults.md): terminal faults and host diagnostics.
- [Raven testing project](testing.md): development test APIs, data, categories and runner contracts.
- [Raven benchmarking project](benchmarking.md): independent benchmark APIs, argument rows and worker contracts.
- [Runtime hosting](runtime-hosting.md): experimental Object-root load contexts.
- [.NET metadata tooling](experimental-metadata.md): the host library for reading
  and writing native metadata; this is not an application API.
- [Compiler services](xref:System.Runtime.CompilerServices) and
  [async builders](async-builders.md).

<!-- Preserve existing deep links into the former landing-page sections. -->
<a id="features-and-reference"></a>
<a id="coverage-and-availability"></a>
<a id="reading-an-api-page"></a>
<a id="explicit-threads-in-development"></a>
<a id="api-overview"></a>
<a id="browse-namespaces"></a>
<a id="start-with-tasks"></a>
<a id="storage-exploration"></a>
<a id="terminal-failures"></a>
<a id="reading-generated-declarations"></a>
<a id="how-this-differs-from-net"></a>

Development API migration annotations use [ObsoleteAttribute](xref:System.ObsoleteAttribute),
with nullable migration messages and compiler warning/error severity.

[Sequence indexing and slicing](sequence-access.md) documents the development
Index/Range extension and list contracts, copy semantics and safe alternatives.

Development: [TLS stream APIs and HTTP integration](ssl-streams.md).

- [Key/value pair equality and hashing](key-value-pair.md): development generic record contracts and current limits.

The separate development [Neo.Signals extension](signals.html) documents its
subscription-only view and signature-dependent publishing method. See also
[typed function invocation](functions.html#typed-invocation-and-method-signatures-development).
