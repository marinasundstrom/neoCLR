# Public reference coverage and compiler support

The metadata inventory includes all public types in the compiler-reference
assembly, including public nested types. New public types are automatically selected
for generation; missing prose is reported without hiding their pages. Internal
runtime providers and private helpers are outside this public inventory.

Many sample programs use [ArrayList](xref:System.Collections.ArrayList`1),
[HashMap](xref:System.Collections.HashMap`2), [Sequence](xref:System.Collections.Sequence`1),
[Option](xref:System.Option`1), [Result](xref:System.Result`2), [Func](xref:System.Func`2)
and [query operators](xref:System.Linq.Operators). Their member references supplement
the [collections](/features/collections/) and [outcomes](/features/outcomes/) walkthroughs.

## Metadata scaffolds

Public visibility in a compiler reference does not mean that every CLR support type
is an executable neoCLR service. ValueType, Enum, Delegate, MulticastDelegate,
Attribute, attribute metadata and NotImplementedException support compilation or
reference-body scaffolding. Their pages identify that role. In particular,
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
