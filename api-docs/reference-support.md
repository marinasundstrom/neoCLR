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
Development Parse/TryParse generic overloads now support Int32 enum names, signed
numbers and comma-separated names. Parse returns Result with NumberParseError;
TryParse resets its out parameter on failure. Enum type arguments are runtime-checked
until native method constraints can represent struct/Enum bounds.

<a id="metadata-scaffolds"></a>

## Native metadata and documentation inputs

Production libraries are built from Raven source and imported through native
metadata. The compiler and language server use these native definitions; host CLR
support types do not become application APIs.

The website renders the committed native API snapshot. A separate legacy CLI
reference remains for documentation tooling and fixture compatibility. Delegate,
MulticastDelegate, placeholder exceptions and CLI case carriers in that projection
do not describe additional executable neoCLR capabilities. Exact exclusions and
manual rendering exceptions remain recorded in the reference inventory.

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

<a id="cli-union-carrier-details"></a>

## Union representations

Native structural unions use signature metadata. Declared Raven unions remain
nominal. Ordinary source uses named cases, patterns and propagation, including
explicit None construction for absence. Generated carriers in the isolated CLI
reference are tooling details, not a native interface requirement.

<a id="source-attribute-base-development"></a>

## Source attribute base

`System.Attribute` has a source-built abstract declaration with a protected
constructor, following the basic .NET nominal attribute inheritance shape.
`System.Runtime.CompilerServices.UnionAttribute` derives from it; source-owned union
markers retain that base relationship in native metadata. The reference assembly
exposes the same abstract/protected shape. This adds no .NET attribute discovery,
usage-policy enforcement or runtime instantiation service. Bootstrap compiler markers
remain explicitly bootstrap-owned while source Attribute/UnionAttribute belong to
the rebuilt library.
