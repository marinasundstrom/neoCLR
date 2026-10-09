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
