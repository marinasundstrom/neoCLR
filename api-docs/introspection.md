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

## Other descriptors

AssemblyInfo, ModuleInfo, FieldInfo, MethodInfo, PropertyInfo and ParameterInfo still
use Object allocation identity in this slice. Their existing metadata-query behavior
is unchanged. Further equality work must include the defining assembly/module and
owner context. In particular, metadata tokens can be absent or scoped to one module;
name equality and token equality alone are not valid general implementations.

TypeInfo and MemberInfo now have generated type/member reference coverage. The remaining
introspection interfaces are an explicit documentation gap, tracked in the maintenance
notes while their identity contracts are investigated.
