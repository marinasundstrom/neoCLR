# Reflection as an inheritance and interface consumer

The [MemberInfo hierarchy](reflection-hierarchy.md) is implemented as of 2026-09-08,
following constructor chaining. It shares Name/DeclaringType storage and readonly
readers across FieldInfo, MethodInfo and PropertyInfo, and exercises frame views,
heap reference collections and internal base construction.

Neo now resolves inherited bundled class properties/methods. Reflection queries
remain explicitly declared-only. Future inherited query support must specify
filtering, override suppression and DeclaringType versus ReflectedType before
changing default results. MethodBase remains deferred until constructor introspection
needs shared behavior. ParameterInfo and Type remain independent.

[Inherited class interface implementations](class-interface-dispatch.md) now bridge
class bases and virtual overrides, preserving owner identity, readonly access and GC.

## Explicit interface implementations

[Explicit interface implementations](explicit-interfaces.md) are now implemented in
runtime metadata, IL and Neo. Tests cover distinct mappings, inherited/redeclared
conformance, class virtual independence, readonly/output contracts, generic substitution,
reflection visibility and lifetime preservation. Property/indexer declaration syntax,
GetInterfaceMap and import compatibility remain separate work. The .NET probe records
a provisional reimplementation edge difference; see the implementation comparison.

## Default interface implementations

Explore this as a separate contract and implementation slice. The existing
[C# interface specification](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/interfaces)
provides a shipped language baseline for interface member bodies and inherited
implementation selection. It does not decide neoCLR's reference-receiver policy.

Compare abstract contracts alone, shared class implementations, and default interface
bodies. Defaults can share behavior without forcing class ancestry, but introduce
selection rules and implementation dependencies. Specify class-versus-interface
precedence, most-specific inherited implementations, diamond ambiguity, explicit
replacement and reabstraction before enabling bodies. A default body must retain the
original concrete receiver through its interface view, respect readonly/output
contracts and avoid invented interface storage. Decide how it calls other members
and appears in reflection, stack traces and closed dispatch analysis.

Use a real reflection capability for the first demonstration if it needs shared
behavior; otherwise choose another concrete library consumer. Default implementations
are not implemented by the class-dispatch slice, and existing bodyless-interface
validation remains in force.

The .NET comparison starts with
[MemberInfo](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.memberinfo?view=net-10.0)
and [MethodBase](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.methodbase?view=net-10.0).
Research the exact member/API migration before implementing it. Familiarity is in
contracts and behavior; neither .NET's full hierarchy nor its Object requirement is
an automatic neoCLR requirement.
