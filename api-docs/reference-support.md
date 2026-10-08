# Public reference coverage and compiler support

The metadata inventory includes all public types in the compiler-reference
assembly, including public nested types. New public types are automatically selected
for generation; missing prose is reported without hiding their pages. Internal
runtime providers and private helpers are outside this public inventory.

Many sample programs use [ArrayList](xref:System.Collections.ArrayList`1),
[HashMap](xref:System.Collections.HashMap`2), [Sequence](xref:System.Collections.Sequence`1),
[Option](xref:System.Option`1), [Result](xref:System.Result`2), [Function shapes](functions.md)
and [query operators](xref:System.Linq.Operators). Their member references supplement
the [collections](/features/collections/), [outcomes](/features/outcomes/) and
[Function types](/features/functions/) walkthroughs.

[Enum](xref:System.Enum) additionally exposes executable development helpers for
names and values, with TypeInfo and constrained generic overloads. Its class remains
non-constructible; its presence does not imply every .NET Enum method is supported.

## Metadata scaffolds

Public visibility in a compiler reference does not mean that every CLR support type
is an executable neoCLR service. ValueType, Delegate, MulticastDelegate,
attribute metadata and NotImplementedException support compilation or
reference-body scaffolding. Delegate and MulticastDelegate are excluded transport
scaffolds; the Function family page documents the runtime contract. Other scaffold
pages identify their role. In particular,
MulticastDelegate does not promise multicast callbacks, and the placeholder exception
does not add exception construction or throwing to applications.

The non-generic Array, Option, Result and TaskOutcome metadata scaffolds are
intentionally excluded from generated pages and navigation to avoid duplicate
application types. Use `Array<T>`, `Option<T>`, `Result<T, E>` and
`TaskOutcome<T>`. CLR case-carrier types nested in the non-generic containers are
excluded as well. Exact exclusions and reasons
are recorded in `exclusions.json`; the complete metadata inventory is retained.

## Manual entries for renderer limitations

No public type is silently excluded. The following entries preserve declarations
that the pinned metadata model or publisher does not emit as ordinary type pages:

| Public type | Reference and reason |
| --- | --- |
| System.NamespaceMembers | [Container reference](/docs/reference-system-functions.html). Its functions are promoted to the System namespace. |
| System.Math.NamespaceMembers | [Container reference](/docs/reference-math-functions.html). Its functions are promoted to the System.Math namespace. |
| System.Runtime.CompilerServices.IsReadOnlyAttribute | [Marker and constructor](/docs/reference-readonly-attribute.html). The metadata model consumes the marker without exposing its public definition to RavenDoc. |

These are rendering limitations, not application-type exclusions. All other public
types, except the explicitly excluded scaffolds above, have generated reference
pages. The existing String and ThreadPool scaffold
constructor exclusions remain explicit; they are not supported application constructors.

## CLI union carrier details

The CLI reference assembly used by this website represents unions through generated
carriers. Its UnionValue contract exposes the boxed active case; default carriers are
inactive. These are details of the documentation/CLI representation, not an additional
interface requirement for native NeoCLR unions. Ordinary source should use named cases,
patterns and propagation, including explicit None construction for absence.

## Source attribute base (development)

`System.Attribute` now has a source-built abstract declaration with a protected
constructor, following the basic .NET nominal attribute inheritance shape.
`System.Runtime.CompilerServices.UnionAttribute` derives from it; source-owned union
markers retain that base relationship in native metadata. The reference assembly
exposes the same abstract/protected shape. This adds no .NET attribute discovery,
usage-policy enforcement or runtime instantiation service. Bootstrap compiler markers
remain explicitly bootstrap-owned while source Attribute/UnionAttribute belong to
the rebuilt library.
