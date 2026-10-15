# Structural type families

Structural types compose data without requiring a named nominal declaration.
They are real types, with identity, signatures and runtime introspection. Use them
when a nominal name or domain-specific declaration would add no useful meaning:
unions represent alternatives, tuples combine values, intersections describe
simultaneous contracts, and function types describe callable shapes.

Efficient data computation is a design requirement. Value semantics do not require
heap allocation, and a reference held inside a value remains a reference. Current
implementation limits are separate from those semantic contracts.

| Family | Native contract | Member reference |
| --- | --- | --- |
| Array | Structural element-shaped reference type, also representable through nominal `Array<T>` | [Array](arrays.md) |
| Function | Signature-shaped callable type with Object compatibility | [Function](functions.md) |
| Tuple | Ordered structural value; native element names do not affect identity | [Tuple](tuples.md) |
| Union | Structural value admitting any listed alternative, including values and references | [Union](unions.md) |
| Intersection | Structural view requiring every listed nominal contract; interface conformance stays explicit | [Intersection](intersections.md) |

See the [runtime operation guide](https://github.com/marinasundstrom/neoCLR/blob/codex/structural-union-intersection/docs/structural-runtime-operations.md) for union tests/extraction, tuple operations, intersections and callable binding/invocation.

## Availability

Native unions, intersections and tuples are implemented on the structural feature
branches in neoCLR and the Raven neoCLR compiler, using matching native metadata
and libraries. They execute in the interpreter. Bounded tuple storage also has
macOS ARM64 AOT execution checks, as do bounded intersection reference views and reference-alternative unions.
Broader AOT/JIT support and efficient native
layout remain integration requirements. Legacy CLI
bridge carriers remain available for existing consumers; those are nominal types.
This feature-branch work is not a published release.

Scoped parameters inside nominal constituents are supported, including
`Response<T> | HttpError | JsonError`. Bare parameter alternatives such as `T | Error`
and all combinations of structural types are not yet supported. See each family
for its current boundary.

## Introspection and identity

Use [TypeInfo](xref:System.Introspection.TypeInfo) for common type queries.
`IsNominalType` distinguishes declaration-backed descriptors. Structural signatures
need no separate TypeDef, declaration name or module ownership to be types.
`DisplayName` is diagnostic text, not an identity key.

`UnionTypeInfo.Types` and `IntersectionTypeInfo.Types` expose constituents.
`TupleTypeInfo.ElementTypes` and `ElementNames` describe ordered elements.
`FunctionTypeInfo` exposes the callable signature. Array shape and the nominal
`Array<T>` declaration are distinct descriptors serving the same array value
contract; the generic spelling does not introduce another wrapper allocation.

Object compatibility is family-specific. Structural identity alone does not imply
reference semantics, Object inheritance or extensibility. Public factories for
arbitrary guest structural descriptors remain future work.

These authored family pages are rendered and indexed alongside generated nominal
API declarations. A separate page is not required for every closed signature.
