# Attribute introspection — development contract

Author direction, 2026-09-27: expose attributes on members and parameters through
introspection, similar to .NET. Use reflection during startup to prepare and cache
route-to-union mappings; consider source generation later. The generated prototype
is contract evidence, not the selected immediate workflow.

## Development contract

MemberInfo (including TypeInfo, FieldInfo, PropertyInfo, MethodInfo and
ConstructorInfo) and ParameterInfo expose `GetCustomAttributesData() -> Sequence<CustomAttributeData>`. Results describe
directly declared attributes without instantiating them or executing constructors.
CustomAttributeData identifies the attribute type and constructor and exposes typed
constructor arguments. Multiple declarations, grouped declarations and repeated allowed attributes retain
metadata order and individual arguments. No implicit inheritance or AttributeUsage
merging occurs.
The initial imported argument subset is String (including null), Int32 and Boolean;
named arguments and other constants require a further slice and must not be silently
reported as complete supported data. Retained application attributes are checked
at import; existing compiler-only annotations remain governed by bridge contracts.

This is development behavior after Preview 10, requiring matching runtime/library
and compiler bridge artifacts. It is not included in the published preview.
The route mapper will resolve case metadata, pattern compilation, capture positions,
parsers and case construction at preparation, retain the result, and avoid repeating
metadata discovery on requests. Checked dynamic case construction still needs a
separate implementation above current class-only reflection construction.

## Comparison and tradeoffs (primary sources reviewed 2026-09-27)

[.NET CustomAttributeData](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.customattributedata?view=net-10.0)
provides metadata inspection without constructing attribute instances and includes
constructor and named arguments. This is the ergonomic baseline. Executing an
attribute constructor is a separate capability and unnecessary for route templates.
The bounded initial constants are a neoCLR limitation, not a .NET improvement.

[Java AnnotatedElement](https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/lang/reflect/AnnotatedElement.html)
distinguishes directly present, inherited and repeatable annotations. We initially
choose directly declared data; automatic inheritance would need explicit policy.
Build-time generation removes runtime discovery but adds build integration and
requires regeneration. Startup reflection keeps ordinary application builds and
has metadata, validation and allocation costs once at preparation. No performance
advantage is claimed without measurements. Prepared request execution must still
respect argument types, access checks and normal union construction semantics.

## Retention and limits

The bridge retains attributes defined in the imported application modules, including
union case types. Source member/parameter tokens remain scoped to their assembly and
module; target tokens must belong to the containing definition. Values are copied
into ordinary GC-traced snapshots. GetConstructorArguments returns a fresh sequence;
changing an array obtained by casting that sequence cannot modify stored metadata.
There is no global attribute instance cache, inheritance walk or constructor call.

The existing UnionAttribute marker remains readable. Compiler-only nullable and
Raven union-case/companion metadata, and external framework annotations without an
admitted runtime definition, remain outside this bounded reflection surface. This is
not full CLI attribute retention. Attribute classes lower the compiler-only Attribute
base to the existing Object foundation; general Attribute instance APIs are not added.
Attribute constructor bodies still require ordinary bridge admission even though
inspection does not execute them. Named arguments, enum/type/array constants and
other numeric widths on retained user attributes fail import explicitly.

The runtime metadata accepts null String constants. The reduced Raven source case
with a nullable String attribute constructor currently fails during compiler emission
in CustomAttributeBuilder; this is an observed compiler limitation, not a runtime
claim of source support. It is recorded for independent compiler investigation.

The executable [consumer](experiments/attribute-introspection/README.md) covers all
available MemberInfo categories and parameters, exact argument types, snapshots and
non-execution. Native tests cover artifact round trips, null constants, malformed
arguments, invalid target tokens and isolation of member/type annotations. The
route-to-union mapper still needs cached construction binding for value cases;
current general reflection construction supports reference classes only.
