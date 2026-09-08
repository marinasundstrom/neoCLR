# Reflection as an inheritance and interface consumer

Planned follow-up to [class dispatch and abstract classes](class-dispatch.md),
recorded 2026-09-08. Current reflection descriptors remain separate records.

Constructor chaining is the next runtime slice and must land before migrating the
reflection descriptors. Specify base-before-derived initialization, argument evaluation,
exactly-once base construction and restrictions on publishing a partially initialized
receiver. Compare CLR constructor calls and C# base/this initializers with neoCLR
managed receiver views; preserve complete-owner identity and initialization checks.

After constructor chaining, the library exercise should use a small shared MemberInfo base for the common
name/declaring-type metadata of MethodInfo, FieldInfo and PropertyInfo. Evaluate
MethodBase for method/constructor metadata only when constructor introspection needs
it. ParameterInfo and Type do not need to be forced into that hierarchy simply for
uniformity. Preserve optional Object ancestry and ordinary value/reference modes.
Use interfaces for independent capabilities that need polymorphic behavior without
shared storage; choose those interfaces from actual consumers rather than creating
an interface for every descriptor class.

Before migration, complete inherited property lookup in Neo/library projection and
specify declared-only versus inherited reflection queries. Review descriptor receiver
contracts: inherited behavior needs readonly managed views rather than copying a base
value. Keep published member names and descriptor identity semantics where they fit,
and document any construction/layout changes. Exercise both frame descriptors and
managed collections of references to heterogeneous descriptors.

Also evaluate inherited interface implementations across class bases, including how
virtual class overrides satisfy an inherited interface contract. That currently
restricted bridge is relevant if a common reflection base exposes a capability
interface. Test owner identity, readonly access, dispatch and GC through both views.

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
