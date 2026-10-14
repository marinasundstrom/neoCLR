# Tuple

The structural Tuple family is a design direction. Today's runtime represents
tuples with nominal generic value declarations, `System.Tuple<T1,...,TN>` for
one through seven components. They remain nominal in introspection; this reference
does not claim they have already become structural types.

## Current representation

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

Future structural tuple identity and construction through introspection need their
own contracts. Function's Object inheritance does not decide tuple inheritance.

[All structural families](structural-types.md)

## Structural migration: host metadata foundation

The 2026-10-14 experimental runtime model adds
`neoclr::metadata::Type::Tuple(Vec<Type>)`. Its ordered element signatures have no
nominal definition name. JSON and native binary metadata decoding preserve that
shape, and type/method/Self substitution traverses its elements. Signature equality
preserves order, duplicates and nesting; it is not a claim of resolved runtime
assignability. Missing substitution arguments and excessive substitution depth
return a host Fault. Scope normalization rejects zero or more than 256 elements.

This Rust host metadata node is not a guest TupleTypeInfo API. The interpreter now
admits closed scalar, nominal and nested tuple elements with `experimental-tuples`.
The union/intersection feature alone does not enable tuple execution. Existing
`Module::instantiated_fields` returns synthesized public `ItemN` views; these have
no nominal FieldDef. Native aggregate construction, indexed element reads/writes,
managed element addresses and default initialization preserve value copying and
reference-element identity. Invalid element types, positions and stores fault.
JIT/AOT and native layout remain unsupported. Open, byref, pointer, array, function
and composition direct elements are not admitted in this bounded slice. Existing nominal
Tuple APIs remain available during migration; their removal is planned. No new Raven
source syntax, CLI tuple projection or guest reflection member is shipped here.

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
construction rights. This is host inspection; a guest `TupleTypeInfo` remains pending.
Unlike .NET ValueTuple reflection, the tuple itself has no nominal generic definition.
