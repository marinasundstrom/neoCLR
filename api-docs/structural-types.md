# Structural type families

Structural types describe shapes without declaring a
name or declaring module. These family names organize the reference; they do not introduce
nominal types. A shape can have members and applicable extension members.

| Family | Current contract | Member reference |
| --- | --- | --- |
| Array | Element-shaped reference type; values can be passed as Object | [Array](arrays.md) |
| Function | Signature-shaped type with Object as its base | [Function](functions.md) |
| Tuple | Current generic Tuple declarations remain nominal value types | [Tuple](tuples.md) |
| Union | Current declared unions remain nominal; structural unions are future work | [Union](unions.md) |
| Intersection | Proposed structural family; no runtime member contract yet | [Intersection](intersections.md) |

Use [TypeInfo](xref:System.Introspection.TypeInfo) for common shape and member
queries. IsNominalType determines whether a descriptor can provide
[NominalTypeInfo](xref:System.Introspection.NominalTypeInfo) declaration metadata.
DisplayName is diagnostic text, not a declaration name or an identity key.
FunctionTypeInfo exposes Parameters, ReturnType and InvokeMethod directly.

Object compatibility is family-specific. Function and Array values support it;
structural identity does not make every family a reference type or an inheritable
base. Factories for constructing arbitrary array, function, tuple, union and
intersection descriptors remain future work.

These authored family pages are rendered and indexed by RavenDoc alongside the
metadata reference. Closed shapes do not need a separate page for every signature.
Their members are documented by shape, with links to generated declaration pages
where the implementation uses named library contracts.
