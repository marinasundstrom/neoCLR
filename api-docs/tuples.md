# Value tuples

**Development after Preview 11.** `System.Tuple<T1,...,TN>` is neoCLR's value-tuple
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
