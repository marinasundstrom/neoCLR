---
title: API documentation
toc: false
---
# API documentation

[Experimental runtime hosting](runtime-hosting.md) covers explicit Object-root load contexts.

[Experimental .NET metadata tooling](experimental-metadata.md) documents the
host library used by the development Raven native importer and emitter; it is not
a guest API or published release.

[Cloning with Self](cloning.md) documents the development nongeneric Clonable contract.

Browse [structural type families](structural-types.md): Array, Function, Tuple,
Union and Intersection, with current member contracts and explicit proposal status.


[Function shapes and objects](functions.md) document the development structural
callable family and its Invoke/extension-member contracts.

[Value tuples](tuples.md) are a development addition after Preview 11, using
`System.Tuple<T1,...,TN>` as the value-type family.

[Native numeric Self](text-numbers.md) is a development addition after Preview 11:
Number is nongeneric and uses the implementing type for its operands and results.

**Preview 11 API.** Comparer policies, explicit String comparison modes and the
HashMap policy constructor are included. Rebuild applications
with matching compiler, references and runtime library artifacts.

[String](xref:System.String) documents construction from [Sequence&lt;Char&gt;](xref:System.Collections.Sequence`1),
read-only grapheme indexing, exact text comparison and explicit
UTF-8 operations, with [Object content behavior](objects.md#string-through-object-development).

[Comparer policies](/features/collections/#comparer-policies-development) cover
[EqualityComparer](xref:System.Collections.EqualityComparer`1),
[Comparer](xref:System.Collections.Comparer`1) and
[StringComparer](xref:System.StringComparer), including OrdinalIgnoreCase.
[StringComparison](xref:System.StringComparison) selects the explicit String.Compare
mode. These APIs use documented Unicode simple-fold semantics,
which can differ from .NET.

[Calendar](xref:System.Time.Calendar), [Culture](xref:System.Globalization.Culture),
[Language](xref:System.Globalization.Language) and [DateTimeFormat](xref:System.Globalization.DateTimeFormat)
provide Gregorian/Hebrew rendering. [CultureProvider](xref:System.Globalization.CultureProvider),
[FixedCultureProvider](xref:System.Globalization.FixedCultureProvider) and
[SystemCultureProvider](xref:System.Globalization.SystemCultureProvider) separate explicit selection
from host discovery. See [globalization](/features/globalization/) for the tested example.

[DateTime](xref:System.Time.DateTime) is a development nominal union of
[LocalDateTime](xref:System.Time.LocalDateTime) and [ZonedDateTime](xref:System.Time.ZonedDateTime).
[TimeOffset](xref:System.Time.TimeOffset) is fixed; [TimeZone](xref:System.Time.TimeZone) supplies
named IANA rules. [LocalTimeMapping](xref:System.Time.LocalTimeMapping) preserves unique,
ambiguous and skipped local times; [TimeZoneError](xref:System.Time.TimeZoneError) reports
lookup/range/discovery failures. See [DateTime and zones](/features/time/) and the [exact union declaration](date-time-union.md).

[Casing and decimal reporting](text-numbers.md) covers development
String.ToUpperInvariant/ToLowerInvariant, Number arithmetic, concrete primitive
parsing, and Int64 formatting and bounds (development).

[Char](xref:System.Char) now has a generated type/member reference for grapheme
construction, equality, ordering and display; see [Object contracts](objects.md)
for boxed behavior.

[Enum](xref:System.Enum) provides names and values through either a TypeInfo or an
enum type parameter. Generic values remain typed; discovery preserves the exact
boxed enum identity. See [Introspection](/features/introspection/) for current limits.

[JSON serialization](json.md) describes the DOM and provisional flat-object overloads,
including property, error and borrowed-stream rules.

Start with a feature guide for behavior and working examples, or open a module
for generated type and member documentation. Both are parts of this site.

<a id="explicit-threads-in-development"></a>
<a id="api-overview"></a>
<a id="browse-namespaces"></a>
<a id="start-with-tasks"></a>
<a id="storage-exploration"></a>
<a id="terminal-failures"></a>

[Socket clients](sockets.md) documents the development TCP connection and listener
APIs, its Task/Result behavior and current limits. [Dns](xref:System.Networking.Dns)
resolves hostnames to immutable [IPAddress](xref:System.Networking.IPAddress) values
from the closed IPv4Address/IPv6Address hierarchy (IPv4 answers only today); the [networking guide](/features/networking/)
walks through the compiled hostname/echo POC.

## Features and reference

| Area | Read the guide | Browse types and members |
| --- | --- | --- |
| Collections and arrays | [Collections and queries](/features/collections/) · [Arrays](/features/arrays/) | [ArrayList](xref:System.Collections.ArrayList`1) · [HashMap](xref:System.Collections.HashMap`2) · [System.Collections](xref:System.Collections) · [Array](xref:System.Array`1) |
| Outcomes and callbacks | [Outcomes](/features/outcomes/) · [Function types](/features/functions/) | [Option](xref:System.Option`1) · [Result](xref:System.Result`2) · [Function shapes](functions.md) |
| Queries | [Collections and queries](/features/collections/) | [System.Linq](xref:System.Linq) |
| Time and clocks | [Dates and clocks](/features/time/) | [Date](xref:System.Time.Date) · [TimeOfDay](xref:System.Time.TimeOfDay) · [Instant](xref:System.Time.Instant) · [Clock](xref:System.Time.Clock) |
| Text and encoding | [Strings](/features/strings/) | [String](xref:System.String) · [Char](xref:System.Char) · [System.Text](xref:System.Text) · [Encoding](xref:System.Text.Encoding) · [Decoder](xref:System.Text.Decoder) · [Encoder](xref:System.Text.Encoder) |
| Mathematics and environment | [Expected outcomes](/features/outcomes/) | [System.Math](xref:System.Math) · [Environment](xref:System.Environment) |
| Tasks and completion | [Tasks and async](/features/tasks/) · [Task.Run and callbacks](callbacks.md) | [System.Tasks](xref:System.Tasks) |
| Isolated workers | [Thread and worker behavior](/features/tasks/) | [System.Concurrency](xref:System.Concurrency) |
| HTTP messages and exchanges | [Web guide](/features/web/) | [HttpClient](xref:System.Web.Http.HttpClient) · [HttpContext](xref:System.Web.Http.HttpContext) · [HttpServer](xref:System.Web.Http.HttpServer) |
| Storage and files | [Files and Storage](/features/files/) · [Providers](storage-provider.md) | [System.Storage](xref:System.Storage) |
| Route parsing (development) | [Matching and parameter guide](routes.md) | [RoutePattern](xref:System.Web.Http.RoutePattern) · [RouteMatch](xref:System.Web.Http.RouteMatch) |
| HTTP JSON conversion (development) | [Mapping and content guide](json.md) | [JsonContent](xref:System.Web.Http.Json.JsonContent), [client helpers](xref:System.Web.Http.Json.HttpClientJsonExtensions), [errors](xref:System.Web.Http.Json.HttpJsonError) |
| JSON DOM (development) | Bounded string/stream parsing and writing; closed, kind-specific node hierarchy | [System.Data.Json](xref:System.Data.Json) |
| Byte and text streams | [Stream contracts](streams.md) · [Pending reads](pending-read.md) | [System.IO](xref:System.IO) |
| Console | [Console guide](/features/console/) · [Standard stream ownership](console.md) | [Console](xref:System.Console) |
| Metadata discovery | [Introspection walkthrough](/features/introspection/) · [Descriptor identity](introspection.md) | [System.Introspection](xref:System.Introspection) |
| Identity, values and records | [Object and value contracts](objects.md) | [Object](xref:System.Object) · [Value](xref:System.Value) · [HashCode](xref:System.HashCode) · [EquatableTo&lt;T&gt;](xref:System.EquatableTo`1) |
| Compiler support | [Transitional async builders](async-builders.md) | [System.Runtime.CompilerServices](xref:System.Runtime.CompilerServices) |
| Runtime failures | [Terminal faults and host diagnostics](faults.md) | Manual host reference in that guide |

[Browse all generated modules](api/) · [Module overview](namespaces.md) ·
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

Development cancellation APIs are in `System.Concurrency`:
[CancellationTokenSource](xref:System.Concurrency.CancellationTokenSource),
[CancellationToken](xref:System.Concurrency.CancellationToken) and
[CancellationRegistration](xref:System.Concurrency.CancellationRegistration).
They separate a cooperative request from operation completion and are currently
confined to one invocation. See [the task feature](../features/tasks/#cooperative-cancellation-requests).
HTTP/socket token overloads, timers and linked sources remain following work.

`HttpClient.BaseUri` now accepts `Option<string>`. Its string and Uri Get overloads
share validation/resolution and handler dispatch. With a base, pass relative references
without authority; without a base, pass an absolute HTTP URI. See
[HttpClient](xref:System.Web.Http.HttpClient) and [HttpRequest](xref:System.Web.Http.HttpRequest).

The development [MemoryStream](xref:System.IO.MemoryStream) provides bounded managed
byte storage for InputStream, OutputStream and SeekableStream, including text/JSON
round trips without file access. It has a shared cursor and a provisional 64 KiB bound.

Development [runtime reflection](reflection.md) adds checked construction and property access to runtime-backed introspection descriptors.

Development directional contracts: [EquatableTo&lt;T&gt;](xref:System.EquatableTo`1),
[ComparableTo&lt;T&gt;](xref:System.ComparableTo`1) and
[ConvertibleInto&lt;T&gt;](xref:System.ConvertibleInto`1). The former Equatable/Comparable
identities are removed; rebuild consumers.

Development [Reflection](reflection.md) adds constructor discovery, argument-based typed
activation, method invocation and field access. See the dedicated [feature page](/features/reflection/).

[Garbage collection](gc.md) covers development `System.Runtime.GC` object counters,
explicit collection and KeepAlive.
