---
title: API documentation
toc: false
---
# API documentation

**Development after Preview 9.** These pages describe the current development
contracts. They may precede a downloadable release. In particular, the development
`System.Concurrency` APIs replace the `System.Threading` names in Preview 9 bundles.

[String](xref:System.String) documents exact text comparison, graphemes and explicit
UTF-8 operations, with [Object content behavior](objects.md#string-through-object-development).

[Char](xref:System.Char) now has a generated type/member reference for grapheme
construction, equality, ordering and display; see [Object contracts](objects.md)
for boxed behavior.

Start with a feature guide for behavior and working examples, or open a namespace
for generated type and member documentation. Both are parts of this site.

<a id="explicit-threads-in-development"></a>
<a id="api-overview"></a>
<a id="browse-namespaces"></a>
<a id="start-with-tasks"></a>
<a id="storage-exploration"></a>
<a id="terminal-failures"></a>

## Features and reference

| Area | Read the guide | Browse types and members |
| --- | --- | --- |
| Tasks and completion | [Tasks and async](/features/tasks/) · [Callbacks](callbacks.md) | [System.Tasks](xref:System.Tasks) |
| Isolated workers | [Thread and worker behavior](/features/tasks/) | [System.Concurrency](xref:System.Concurrency) |
| Storage and files | [Files and Storage](/features/files/) · [Providers](storage-provider.md) | [System.Storage](xref:System.Storage) |
| Byte and text streams | [Stream contracts](streams.md) · [Pending reads](pending-read.md) | [System.IO](xref:System.IO) |
| Console | [Console guide](/features/console/) · [Standard stream ownership](console.md) | [Console](xref:System.Console) |
| Metadata discovery | [Introspection walkthrough](/features/introspection/) · [Descriptor identity](introspection.md) | [System.Introspection](xref:System.Introspection) |
| Identity, values and records | [Object and value contracts](objects.md) | [Object](xref:System.Object) · [Value](xref:System.Value) · [HashCode](xref:System.HashCode) · [Equatable&lt;T&gt;](xref:System.Equatable`1) |
| Compiler support | [Transitional async builders](async-builders.md) | [System.Runtime.CompilerServices](xref:System.Runtime.CompilerServices) |
| Runtime failures | [Terminal faults and host diagnostics](faults.md) | Manual host reference in that guide |

[Browse all generated namespaces](api/) · [Namespace overview](namespaces.md) ·
[All feature guides](/guides/)

<a id="reading-generated-declarations"></a>
<a id="how-this-differs-from-net"></a>

## Reading an API page

RavenDoc generates declarations from the compiler reference assembly and renders
the authored documentation sidecar. Type pages list members; member pages include
summaries, parameters, results and available remarks. Signatures use Raven notation.
A reference signature does not imply support for arbitrary .NET assemblies.

Generic unit-valued results and callbacks are included: `Func<System.Void>` and
`Result<System.Void, E>` no longer require omissions from generated reference.
The callback and stream guides explain the behavior behind those signatures.
Reference-only carrier/scaffold details may differ from idiomatic source; use the
tested examples in feature guides when writing an application.

## Coverage and availability

Reference coverage is still being expanded. Arrays, collections, queries,
Option/Result helpers, text/encoding, time/calendar and other older areas have
feature guides but do not yet have complete generated member reference.
The [namespace overview](namespaces.md) makes those gaps visible; a guide is not a
claim of full member coverage.

Names and contracts are experimental. Pages label published behavior, development
changes and proposals separately. Use [the matching toolchain](/try/#development)
for development examples.
