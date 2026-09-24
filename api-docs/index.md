---
title: API documentation
toc: false
---
# API documentation

**Development after Preview 9.** These pages describe the current development
contracts. They may precede a downloadable release. In particular, the development
`System.Concurrency` APIs replace the `System.Threading` names in Preview 9 bundles.

[String](xref:System.String) documents construction from [Sequence&lt;Char&gt;](xref:System.Collections.Sequence`1),
read-only grapheme indexing, exact text comparison and explicit
UTF-8 operations, with [Object content behavior](objects.md#string-through-object-development).

[Char](xref:System.Char) now has a generated type/member reference for grapheme
construction, equality, ordering and display; see [Object contracts](objects.md)
for boxed behavior.

[Enum](xref:System.Enum) provides names and values through either a TypeInfo or an
enum type parameter. Generic values remain typed; discovery preserves the exact
boxed enum identity. See [Introspection](/features/introspection/) for current limits.

Start with a feature guide for behavior and working examples, or open a namespace
for generated type and member documentation. Both are parts of this site.

<a id="explicit-threads-in-development"></a>
<a id="api-overview"></a>
<a id="browse-namespaces"></a>
<a id="start-with-tasks"></a>
<a id="storage-exploration"></a>
<a id="terminal-failures"></a>

[Socket clients](sockets.md) documents the development TCP connection and listener
APIs, its Task/Result behavior and current limits. [Dns](xref:System.Networking.Dns)
resolves hostnames to IPv4 addresses; the [networking guide](/features/networking/)
walks through the compiled hostname/echo POC.

## Features and reference

| Area | Read the guide | Browse types and members |
| --- | --- | --- |
| Collections and arrays | [Collections and queries](/features/collections/) · [Arrays](/features/arrays/) | [ArrayList](xref:System.Collections.ArrayList`1) · [HashMap](xref:System.Collections.HashMap`2) · [System.Collections](xref:System.Collections) · [Array](xref:System.Array`1) |
| Outcomes and callbacks | [Outcomes](/features/outcomes/) | [Option](xref:System.Option`1) · [Result](xref:System.Result`2) · [Func](xref:System.Func`2) |
| Queries | [Collections and queries](/features/collections/) | [System.Linq](xref:System.Linq) |
| Time and clocks | [Dates and clocks](/features/time/) | [Date](xref:System.Date) · [Time](xref:System.Time) · [Instant](xref:System.Instant) · [Clock](xref:System.Clock) |
| Text and encoding | [Strings](/features/strings/) | [String](xref:System.String) · [Char](xref:System.Char) · [System.Text](xref:System.Text) |
| Mathematics and environment | [Expected outcomes](/features/outcomes/) | [System.Math](xref:System.Math) · [Environment](xref:System.Environment) |
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

Every public type in the compiler-reference assembly is included in the reference
inventory, including public nested cases and compiler support types. Types do not
need documentation comments to receive pages. The types used in samples—collections,
arrays, outcomes, callbacks, queries, text and time—have type and member descriptions.

Compiler-reference scaffolds are labelled as metadata support, rather than advertised
as executable .NET services. [Renderer limitations](reference-support.md) have linked
manual entries; internal implementation types remain outside the public reference.

Names and contracts are experimental. Pages label published behavior, development
changes and proposals separately. Use [the matching toolchain](/try/#development)
for development examples.

The development [HTTP client, server and handler APIs](xref:System.Web.Http) support a bounded
GET/200 POC over sockets. Read the [Web guide](/features/web/) for pipeline examples,
ownership, framing limits and missing transfer deadlines.
