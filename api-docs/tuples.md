# Tuple

The experimental native profile binds and emits Raven tuples as structural
signatures, with native names, value copies and `TupleTypeInfo` introspection.
Native project profiles enable this capability with matching compiler/runtime
artifacts; compiler-host APIs retain explicit capability switches.

The legacy CLI bridge still uses nominal generic `System.Tuple<T1,...,TN>` value
declarations for one through seven components. These remain nominal in introspection
and remain in the runtime library pending migration of existing consumers.

## Legacy nominal representation

`System.Tuple<T1,...,TN>` is neoCLR's value-tuple
family, corresponding to .NET `System.ValueTuple`. One through seven components
are available. These are structs with mutable public `Item1` through `ItemN` fields
and a constructor taking each component in order.

The [pair reference](xref:System.Tuple`2) documents the common two-component form.
All seven arities are included in the generated reference. Assignment copies fields;
reference components continue to share their referenced objects. Labels used in
Raven tuple annotations are source metadata, not separately stored fields.

Raven uses `()` for the existing `System.Void` unit contract. The one-component
type requires explicit `Tuple<T>(value)` construction. The initial slice does not
provide larger flat tuples, a `Rest` carrier, tuple-specific equality/comparison/
formatting interfaces or `ITuple`. It does not promise full .NET ValueTuple API parity.

`TupleElementNamesAttribute` exists only in compiler-reference metadata to retain
labels; it is excluded from executable API pages with this explicit explanation.
Use a matching development compiler, reference, importer and System library.

## Current members

Each arity has a constructor taking its component values in order and public
`Item1` through `ItemN` fields of the corresponding component types. Assignment
copies the value's fields. Member details are generated from the matching reference:

- [Tuple&lt;T1&gt;](xref:System.Tuple`1)
- [Tuple&lt;T1, T2&gt;](xref:System.Tuple`2)
- [Tuple&lt;T1, T2, T3&gt;](xref:System.Tuple`3)
- [Tuple&lt;T1, T2, T3, T4&gt;](xref:System.Tuple`4)
- [Tuple&lt;T1, T2, T3, T4, T5&gt;](xref:System.Tuple`5)
- [Tuple&lt;T1, T2, T3, T4, T5, T6&gt;](xref:System.Tuple`6)
- [Tuple&lt;T1, T2, T3, T4, T5, T6, T7&gt;](xref:System.Tuple`7)

Structural identity is ordered and positional; names are retained per signature
without changing compatibility. General guest construction through introspection
remains incomplete. Function's Object inheritance does not decide tuple inheritance.

[All structural families](structural-types.md)

## Structural migration: host metadata foundation

The 2026-10-14 experimental runtime model adds
`neoclr::metadata::Type::Tuple(TupleSignature)`. Its ordered element signatures have no
nominal definition name. JSON and native binary metadata decoding preserve that
shape, and type/method/Self substitution traverses its elements. Signature equality
preserves order, duplicates and nesting; it is not a claim of resolved runtime
assignability. Missing substitution arguments and excessive substitution depth
return a host Fault. Scope normalization rejects zero or more than 256 elements.

The Rust metadata node is separate from the guest descriptor API below. The interpreter now
admits closed scalar, nominal and nested tuple elements with `experimental-tuples`.
The union/intersection feature alone does not enable tuple execution. Existing
`Module::instantiated_fields` returns synthesized public `ItemN` views; these have
no nominal FieldDef. Native aggregate construction, indexed element reads/writes,
managed element addresses and default initialization preserve value copying and
reference-element identity. Invalid element types, positions and stores fault.
A bounded native backend now lowers tuple storage inline (see below). Open, byref, pointer, array, function
and composition direct elements are not admitted in this bounded slice. Existing nominal
Tuple APIs remain available during migration; their removal is planned. Raven native source binding and guest element descriptors are described below;
CLI tuple projection does not represent this structural identity.

## Resolved host tuple descriptors (development)

With `experimental-tuples`, `LoadedProgram::resolve_type_identity` returns
`TypeIdentity::Tuple(Vec<TypeIdentity>)` for admitted closed tuple signatures.
Element keys retain resolved nominal identities where applicable. Order, repeated
elements and nesting contribute to identity; a one-element tuple differs from its
element. The tuple has no TypeDef or nominal metadata token.

`LoadedProgram::describe_type` returns a `TypeDescriptor` with ordered
`tuple_elements: Vec<TypeDescriptor>` and a recursive `tuple<...>` name. Each element
has its own descriptor. `generic_arguments` and `constituents` remain empty for a
tuple; `tuple_elements` is empty for other shapes. These APIs use the same feature,
closedness, arity and element-admission checks as executable tuples, returning
`Fault` for unsupported signatures. Descriptor access does not grant layout or
construction rights. This host inspection underlies the guest `TupleTypeInfo` provider.
Unlike .NET ValueTuple reflection, the tuple itself has no nominal generic definition.


## TupleTypeInfo (development)

[`System.Introspection.TupleTypeInfo`](xref:System.Introspection.TupleTypeInfo)
extends `TypeInfo`. Its read-only `ElementTypes: Sequence<TypeInfo>` property
returns a snapshot in positional order, including repeated types and recursive
`TupleTypeInfo` descriptors for nested tuples. It is not a nominal generic type;
elements are separate from `GetGenericArguments`. `IsValueType` is true and
`IsUnion` is false. Element labels are not part of runtime identity.

`ElementNames: Sequence<string>` returns a fresh snapshot of effective names in
matching positional order. Declared native names are preserved; an unnamed position
uses `ItemN` (one-based). Equal tuple types can expose different names because each
signature retains its descriptive metadata. This query does not distinguish an
explicit `ItemN` label from the same synthesized default; host signature APIs retain
that distinction. Identity-only handles cannot recover declared names.

Requires the `experimental-tuples` runtime and the matching native source library.
There is no public constructor. The runtime selects this provider when a type handle
contains an admitted structural tuple signature. The same arity, closedness and
element-admission limits apply as for tuple execution. Unsupported signatures fault.
Nominal `System.Tuple<T...>` descriptors do not implement this interface. Synthesized
field/constructor introspection, nominal module ownership, native layout and unrestricted
Raven tuple emission remain incomplete. Do not infer those APIs from ElementTypes.

The executable [Raven consumer](https://github.com/marinasundstrom/neoCLR/blob/codex/structural-union-intersection/docs/experiments/structural-tuples/TypeInfo.rvn)
checks ordered duplicates, nested names, value classification, equal renamed types and
repeated snapshot queries. A documented test fixture substitutes two native tokens;
this validates guest introspection, not compiler tuple lowering.


## Native tuple element names (2026-10-14 development)

Rust `metadata::Type::Tuple` now contains `metadata::TupleSignature`. Its
`new(elements: Vec<Type>, names: Vec<Option<String>>) -> Result<Self, Fault>`
constructor stores native names directly. `names() -> &[Option<String>]` returns
the declared names; empty means entirely unnamed. `element_name(index: usize) ->
Option<String>` returns the declared name or positional `ItemN`, and returns None
for an out-of-range index. `From<Vec<Type>>` creates an unnamed signature; callers
constructing the old variant migrate to `Type::Tuple(elements.into())`. Deref and
borrowed iteration expose the ordered element types.

A nonempty name vector must have one entry per element. Names must be nonempty,
NUL-free, at most 1024 UTF-8 bytes and unique after unnamed positions receive their
`ItemN` fallback. These bounded validation rules are provisional development limits.
Invalid names/counts return Fault; malformed named serialized shapes fail decoding.
The existing one-to-256-element and nesting checks still apply during normalization.

Named signatures encode as `{"Tuple":{"elements":["Int32","Int32"],"names":["x","y"]}}`.
Unnamed signatures retain `{"Tuple":["Int32","Int32"]}`. Matching native readers are
required for the new object payload; the updated experimental C# reader supports
it while older readers reject it. This is
direct signature metadata, without a nominal carrier or tuple-name attribute.

Equality/hashing of TupleSignature and Type ignore names recursively; TypeIdentity
also remains the ordered element identities. Names are preserved through native
serialization, scope binding and type/method/Self substitution.
`TypeDescriptor::tuple_element_names: Vec<Option<String>>` returns names in the same
order as `tuple_elements`, including None for unnamed elements; non-tuples return
an empty vector. Descriptors obtained from differently named signatures retain their
respective names while their `identity` values compare equal. Identity alone cannot
recover names: consumers must retain the descriptive signature/descriptor rather
than intern it solely by structural identity. Synthesized runtime field layouts use
declared names with the existing `ItemN` fallback.

Unlike the CLR's ValueTuple/name-attribute representation, native names need no
attribute decoding. The cost is an expanded signature payload and explicit separation
of descriptive metadata from equality. Guest ElementNames and Raven tuple import
now preserve these names; named assembler syntax remains a follow-up.
C# authoring/reading/import and Raven native emission now preserve names; see
[the C# contract](experimental-metadata.md#native-tuple-names-in-c-authoringimport-2026-10-14-development). Existing positional guest element
queries and execution remain supported. Focused tests prove exact name roundtrips,
equal identity/hash under renaming, name-preserving substitution, host descriptors,
invalid-name rejection and interpreter parameter compatibility (result 42).

## Raven native binding contract (development)

Raven hosts now select `MetadataImportOptions.WithStructuralTuples()` alongside
`NeoClrEmitOptions.AllowExperimentalTuples`. The binding option defaults to false
and is neoCLR-only; the emitter requires both options. `ITupleTypeSymbol` exposes
ordered fields and names with `UnderlyingTupleType == null`, no nominal declaration
or constructors, and empty TypeArguments. Positional ItemN aliases are available.
Names do not affect structural equality/hash; substitution preserves names. CLI
emission rejects this profile. Ordinary .NET tuple APIs are unchanged.

Local-rooted element mutation supports nested writeback and preserves copies;
immutable let roots and nonlocal roots are rejected. The current native emitter
still admits only closed scalar/nested elements. Legacy runtime tuple declarations
remain for CLI bridge consumers but are not used by native structural binding.
These are compiler-host contracts, documented in Raven's runtime-contract and
neoCLR integration guides; they do not add a guest class-library API.

## Native execution development

The structural branch admits ordered tuple storage in the shared native backend.
macOS ARM64 AOT tests cover nested integer/floating-point tuples, copying, calls,
returns, element mutation through managed addresses and default initialization.
Elements use the existing private eight-byte scalar slots, capped at 256 flattened
slots. Tuple construction itself allocates no heap carrier. Element names remain
metadata; positional access and compatibility are unchanged.

Nested tracing recipes retain reference slots and omit numeric slots. This bounded
storage slice does not complete native tuple reflection, boxing, array storage,
Windows qualification or full Raven application integration. Direct JIT tests also execute two- and sixteen-integer tuple copy/call workloads;
managed-host and collector qualification remain open. These are
integration gaps, not permanent restrictions of structural identity.

Initial AOT microbenchmarks compare two- and sixteen-integer tuples with equivalent
nominal records: each pair emits byte-identical native objects with no allocator
imports. This is evidence for those copy/call workloads, not a general performance
guarantee. Reference-bearing values, boxing and union workloads remain unmeasured.

Development native specialization also admits tuples as closed generic arguments
and inline fields of nominal records. Recursive cycles and the flattened-lane budget
remain checked. See the [native structural qualification](../docs/experiments/native-structural-types/README.md).
