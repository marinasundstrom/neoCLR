---
uid: N:System.Introspection
---
## Descriptions of the loaded program

AssemblyInfo and ModuleInfo describe the loaded catalog. TypeInfo, MemberInfo and
specialized descriptors expose type shapes, declarations, parameters and retained
metadata identities. BindingFlags selects supported query categories.

See [introspection](/docs/introspection.html) for descriptor identity and metadata
availability. These descriptors do not dynamically load assemblies; invocation and
member-access operations are provided separately by System.Runtime.Reflection.

Development generic type parameters expose `GenericParameterTypeInfo.Variance`:
`Invariant`, `Covariant` (`out`) or `Contravariant` (`in`). Obtain parameters from
an open definition's generic arguments. Closed constructions expose actual argument
types. Method `ParameterInfo.IsOut` continues to describe an output argument.
Open-definition discovery currently uses interpreter reflection; native reflection
continues to require closed types. See the [reflection reference](/docs/reflection.html).
