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

The [default interface contract](default-interface-implementations.md) is now recorded
with a pinned .NET probe, negative compilation checks and a runtime groundwork audit.
It chooses class precedence, most-specific default selection, explicit diamond failures,
reabstraction and an interface-view receiver retaining the original owner. Default
bodies are not yet enabled. Implement selection, validation and receiver entry together,
then wire Neo, reflection, closed analysis and debugger coverage as specified there.

Use a real library capability once shared behavior is needed; do not introduce a
reflection interface merely to demonstrate a default. The first implementation demo
should exercise a default calling a required member on frame and heap owners.

The .NET comparison starts with
[MemberInfo](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.memberinfo?view=net-10.0)
and [MethodBase](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.methodbase?view=net-10.0).
Research the exact member/API migration before implementing it. Familiarity is in
contracts and behavior; neither .NET's full hierarchy nor its Object requirement is
an automatic neoCLR requirement.
