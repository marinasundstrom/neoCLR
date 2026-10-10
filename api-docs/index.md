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

## Find an API

| Area | Starting points |
| --- | --- |
| Text and numbers | [String](xref:System.String) · [Char](xref:System.Char) · [System.Math](xref:System.Math) · [Text and numeric contracts](text-numbers.md) |
| Collections and queries | [System.Collections](xref:System.Collections) · [System.Linq](xref:System.Linq) · [Arrays](arrays.md) |
| Outcomes and values | [Option](xref:System.Option`1) · [Result](xref:System.Result`2) · [Object and value contracts](objects.md) · [Tuples](tuples.md) |
| Functions | [Function shapes and objects](functions.md) · [Callbacks](callbacks.md) |
| Streams and files | [System.IO](xref:System.IO) · [System.Storage](xref:System.Storage) · [Stream contracts](streams.md) |
| Console and environment | [Console](xref:System.Console) · [Environment](xref:System.Environment) · [Standard streams](console.md) |
| Tasks and concurrency | [System.Tasks](xref:System.Tasks) · [System.Concurrency](xref:System.Concurrency) |
| Dates and globalization | [System.Time](xref:System.Time) · [System.Globalization](xref:System.Globalization) |
| Networking and HTTP | [System.Networking](xref:System.Networking) · [System.Web.Http](xref:System.Web.Http) · [Sockets](sockets.md) · [Routes](routes.md) |
| JSON | [System.Data.Json](xref:System.Data.Json) · [Serialization and HTTP content](json.md) |
| Introspection and reflection | [System.Introspection](xref:System.Introspection) · [Descriptor contracts](introspection.md) · [Runtime reflection](reflection.md) |

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
- [Raven testing helpers](testing.md): development TestAttribute, discovery and runner contracts.
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
