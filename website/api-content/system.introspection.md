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

Generic type parameters expose `GenericParameterTypeInfo.Variance`:
`Invariant`, `Covariant` (`out`) or `Contravariant` (`in`). Obtain parameters from
an open definition's generic arguments. Closed constructions expose actual argument
types. Method `ParameterInfo.IsOut` continues to describe an output argument.
Native discovery supports retained assembly/module type catalogs,
including open definitions and their variance annotations. AOT/JIT require explicit
`typeCatalogs` retention; open member/relationship queries remain unsupported.
See the [reflection reference](/docs/reflection.html).

Development MethodInfo generic-state flags distinguish method definitions from
owner openness. Interpreted GetMethods supports descriptive generic signatures;
native type-method discovery requires explicit `methodTypes` retention.
