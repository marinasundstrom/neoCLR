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
