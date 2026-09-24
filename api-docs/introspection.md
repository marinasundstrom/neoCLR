# Introspection and Object contracts

Development after Preview 9. [TypeInfo](xref:System.Introspection.TypeInfo) describes
a type in the current loaded program. [MemberInfo](xref:System.Introspection.MemberInfo)
provides its name, declaring type, module and token; the type interface adds shape,
generic arguments and member queries. The [type reference](xref:System.Introspection.TypeInfo)
lists each supported operation. These APIs describe metadata; they do not invoke
methods, read fields or construct arbitrary objects.

## Type identity

Repeated queries can allocate different descriptors of the same type. RuntimeTypeInfo
now applies represented-type identity through both `Equals(TypeInfo)` and
`Object.Equals(Object?)`. Generic arguments, element shape and loaded definition
identity participate; a name or metadata token alone is insufficient.

Typed equality implements `Equatable<TypeInfo>.Equals(TypeInfo)`, with no nullable
operand. Object equality is the separate null-aware reference boundary: null and
unrelated objects compare false. `Object.ReferenceEquals` still compares descriptor
allocations. This follows the distinction between type identity and wrapper identity
in .NET's Type APIs, without requiring CLR wrapper caching.

`GetHashCode`, called through Object, hashes the represented FullName with System.HashCode.
Equal represented types have equal hashes. Distinct definitions with matching display
names may collide; hashing never substitutes for equality. Hashes are not persistent
identifiers and may change between runtime versions. `ToString` returns the represented
FullName, rather than the runtime wrapper class name.

```raven
let first = typeof(int)
let second = typeof(int)
let view: Object = first
// Both compare the represented type, despite separate descriptor allocations.
first.Equals(second)
view.Equals(second)
```

The executable integration fixture also checks constructed generic types, arrays,
wrong types, null at the Object boundary, HashMap callbacks and retention through GC.
There is no general default comparer or cross-program descriptor identity contract.

## Assembly and module identity

[AssemblyInfo](xref:System.Introspection.AssemblyInfo) compares the full assembly
identity in the current loaded catalog. [ModuleInfo](xref:System.Introspection.ModuleInfo)
compares that identity together with its module name. Catalog validation ensures
these keys are unique within one loaded program. Short assembly names, module names
alone and metadata tokens are not sufficient keys. Repeated queries can allocate
separate, equal wrappers; ReferenceEquals still compares allocations.

These contracts apply through Object.Equals(Object?). Null and other descriptor kinds
compare false. GetHashCode hashes the same identity components with System.HashCode;
collisions are allowed and hashes must not be persisted. Assembly display returns
FullName; module display returns Name. This does not introduce nullable typed equality
or a new Equatable interface on these descriptors.

.NET Assembly/Module describe loaded runtime entities, with loader contexts and
runtime-specific identity. neoCLR currently has one descriptive loaded catalog and
no dynamic assembly loading. Its full-name-based assembly equality is scoped to that
model, not a promise of CLR loader equivalence. Additional load contexts would require
an additional identity component. String hashing currently creates encoding temporaries.

The executable fixture checks repeated wrappers, Object dispatch and HashMap callbacks
under collection pressure. A separate catalog regression uses assemblies with matching
short names and module names but different full identities, plus multiple modules in
one assembly.

## Other descriptors

FieldInfo, MethodInfo, PropertyInfo and ParameterInfo still use Object allocation
identity. Further equality work must include defining scope and owner context;
parameter snapshots do not yet retain their declaring member. Tokens can be absent
or module-scoped. Their generated reference coverage remains an explicit gap in the
maintenance notes. TypeInfo, MemberInfo, AssemblyInfo and ModuleInfo now have generated
type/member reference coverage.
