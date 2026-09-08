# Reflection introspection

neoCLR exposes metadata queries on `System.Type`. `MethodInfo`, `FieldInfo` and
`PropertyInfo` share an abstract `MemberInfo` base; `ParameterInfo` remains independent.
See [descriptor hierarchy](reflection-hierarchy.md) for readonly receiver contracts,
base views and migration. Queries inspect declarations without executing methods
or reading an object's field values.

## Run the examples

From the repository root:

```sh
cargo run -- run examples/source/reflection.neo
cargo run -- verify examples/source/reflection.neo
cargo run -- assemble examples/source/reflection.neo /tmp/reflection.neo.json
cargo run -- run /tmp/reflection.neo.json
cargo run -- run examples/reflection.neoil
```

The Neo example prints the Counter record's Age field and Add method, their types,
System.Type's properties, and the `Counter&` argument of `Option<Counter&>`. Existing
Neo arrays, loops, property access and library calls support this API; no new syntax
or instruction is needed.

```swift
let fields = typeof(Counter).GetFields()
for i in 0..<fields.Length {
    WriteLine(fields[i].Name)
    WriteLine(fields[i].FieldType.Name)
}
```

## Type queries

| API | Result / contract |
| --- | --- |
| `BaseType` | `Option<Type>`, immediate substituted record base or None |
| `GetFields()` / `GetFields(BindingFlags)` | `FieldInfo[]`, declared fields |
| `GetMethods()` / `GetMethods(BindingFlags)` | `MethodInfo[]`, declared instance and static methods, including property accessors |
| `GetProperties()` / `GetProperties(BindingFlags)` | `PropertyInfo[]`, declared properties |
| `GetInterfaces()` | `Type[]`, direct and transitive interfaces, deduplicated, excluding self |
| `GetGenericArguments()` | `Type[]`, closed signature arguments, empty for nongeneric signatures |
| `GetElementType()` | `Option<Type>`, Some for array, managed-reference or pointer target, otherwise None |
| `IsArray`, `IsByRef`, `IsPointer`, `IsInterface` | Boolean signature/definition facts |
| `Name` | Existing qualified definition name, e.g. `System.Option`; wrappers include target signature |
| `FullName` | Qualified signature, e.g. `System.Option<System.Int32&>` |
| `Namespace` | Namespace of the outermost declaring definition; empty for unqualified definitions and array/reference/pointer signatures |

Existing `Equals(Type)`, `GenericArgumentCount`, and `GetGenericArgument(Int32)` remain
available. See [type inspection](type-inspection.md) for handle identity and `ldtoken`.
Name/FullName deliberately retain neoCLR signature spelling rather than .NET backtick
arity and assembly-qualified formatting. `Foo&.IsByRef` describes an addressing mode;
there is no inherent value/reference classification on Foo. Wrapper signatures do not
enumerate their target's members: use GetElementType first.

Use `typeof(T)` for a declared signature. `TypeOf<T>.Of(T)` also describes declared T.
For a live initialized managed reference, Neo's `reference.GetType()` intrinsic describes
its target, including the concrete type behind an interface view. IL uses `ref.type`
to produce a RuntimeTypeHandle, followed by GetTypeFromHandle. An interior reference
describes its field/element type. This creates only owned metadata, without copying
or retaining the guest target, boxing, or requiring Object. Existing declared GetType
methods retain normal method dispatch; the intrinsic is a fallback for managed references.
Raw pointers and ordinary values are not implicitly addressed or followed. Uninitialized
or expired targets fault; use typeof to inspect a signature without a live value.

## Member descriptors

All properties below have public getters and private backing fields. Query arrays and
descriptors are returned by value. Parameter positions are zero-based and exclude the
receiver. Parameter names are empty when absent from metadata.

| Descriptor | Properties / methods |
| --- | --- |
| `FieldInfo` | Name, DeclaringType, FieldType, IsPublic, IsPrivate, IsAssembly, IsStatic, DefinitionIndex |
| `MethodInfo` | Name, DeclaringType, ReturnType, IsStatic, IsPublic, IsPrivate, IsAssembly, IsReceiverByRef, DefinitionIndex; `GetParameters(): ParameterInfo[]` |
| `PropertyInfo` | Name, DeclaringType, PropertyType, IsStatic, CanRead, CanWrite, DefinitionIndex; `GetIndexParameters(): ParameterInfo[]`; `GetGetMethod(): Option<MethodInfo>`, `GetSetMethod(): Option<MethodInfo>`, each also accepting Boolean nonPublic |
| `ParameterInfo` | Name, Position, ParameterType, IsOut, IsOutWhenTrue |

IsAssembly means neoCLR's internal visibility. Fields currently have instance storage,
so FieldInfo.IsStatic is false. IsReceiverByRef reports a method's explicit managed
reference receiver; ParameterType preserves T& signatures. IsOut marks an unconditional
output contract; IsOutWhenTrue marks the separate conditional contract. These facts do
not require callers to explicitly dereference managed references in Neo.

CanRead/CanWrite report accessor presence regardless of visibility. A property is
public if either accessor is public. GetGetMethod/GetSetMethod without arguments
return only public accessors; passing true allows nonpublic accessor metadata. Missing
or filtered accessors return None, since this API does not invent a null descriptor.
Property index-parameter names are empty because property metadata stores only types;
accessor method parameters retain their own declared names. No accessor executes while
constructing a descriptor, including accessors with side effects or faulting bodies.

Generic owner arguments substitute throughout member and accessor signatures. A
method on ArrayList<Foo&> reports Foo& wherever its definition uses T. Snapshots refer
to types by identity and do not recursively enumerate those types' members, so recursive
signatures such as Node containing Node& remain finite.

Enumeration follows loaded metadata declaration order, including distinct overloads.
It is deterministic within that loaded build, not a stable ordering across builds.
DefinitionIndex is a neoCLR extension, **not a CLR MetadataToken**: method indices are
module function rows; field/property indices are rows within the declaring definition.
Use descriptor category, DeclaringType identity and DefinitionIndex together. Closed
owners retain their substituted type identity. Names alone do not identify members.

Constructors are excluded from GetMethods; a future GetConstructors can expose them
separately. Free functions belong to modules and never appear in Type.GetMethods.
FunctionInfo is reserved for a future Module.GetFunctions API; it is not a Type member
or a required base class for MethodInfo.

## Filtering

The familiar .NET [member query](https://learn.microsoft.com/en-us/dotnet/api/system.type.getmethods)
and [BindingFlags](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.bindingflags)
names guide the API. This preview supports only the following flag bits:

| Factory on `System.Reflection.BindingFlags` | Bit |
| --- | --- |
| `Default()` | 0 |
| `DeclaredOnly()` | 2 |
| `Instance()` | 4 |
| `Static()` | 8 |
| `Public()` | 16 |
| `NonPublic()` | 32 |

BindingFlags is currently a typed record, not an enum. Combine factories with the
instance method `Or(BindingFlags)`, or use `FromValue(Int32)` and `get_Value()`.
For example, Neo can query private and internal instance fields with:

```swift
let flags = System.Reflection.BindingFlags.Instance().Or(System.Reflection.BindingFlags.NonPublic())
let fields = typeof(Counter).GetFields(flags)
```

Parameterless queries select Public | Instance | Static (28). An explicit filter
must select both visibility and storage mode; Default(), or Public() alone, returns
an empty array. NonPublic includes private and internal members. A public property
with a private accessor does not appear in a NonPublic-only query. DeclaredOnly is
accepted; method/field/property results are currently declared-only. Unsupported
bits fault, including IgnoreCase and FlattenHierarchy. No inherited, name-filtered,
or singular GetMethod/GetField/GetProperty lookup is provided yet.

## Storage, limits and access boundaries

Descriptors own metadata snapshots and contain no references to inspected guest
objects. Copying a descriptor or result array copies its value; modifying a result
array cannot mutate runtime metadata or an earlier copy. Applications may explicitly
store these values in managed heap arrays, where ordinary GC rules apply. There is
no descriptor disposal obligation or hidden target-object root.

Value arrays retain their fixed shape on ordinary replacement. Neo emits local.reset
when executing declarations again, so successive loop iterations may retrieve parameter
arrays of different lengths directly. The runtime rejects reset while managed aliases
to the old local remain live. This renews declaration storage without changing array
assignment or promising automatic block cleanup.

The runtime enforces array element and payload budgets during result construction and
subsequent execution, including nested parameter/accessor arrays. This uses the existing
logical array budgets, not an exact accounting of host allocator bytes. Type identities
retain module/revision/definition information within the load set. Returned handles
remain subject to the existing restriction on importing host handles into execution.

Inspection can reveal nonpublic and transitive member signatures from loaded metadata.
It does not grant direct module references, private invocation, field access, mutation,
or construction permission. Queries expose declarations, not executable capabilities.

Backend reachability reports TypeInspection and, for array-producing queries,
ManagedArrays. Property/accessor options and GetElementType also require ValueStorage
through the current Option carrier representation. The host binding validates helper
signatures; all public descriptor accessors and Type forwarding methods are ordinary IL.

Invoke, GetValue/SetValue, reflective construction, attribute discovery, class inheritance,
module enumeration and metadata mutation remain
future work. [Acceptance tests](../tests/reflection.rs) cover filtering, substitution,
accessors, output contracts, module boundaries, serialization, resource limits, copied
arrays and GC pressure.

The [readonly input-parameter slice](readonly-parameters.md) now enforces restricted
managed access at runtime, with Neo declarations and ParameterInfo.IsReadOnly.
Readonly instance receivers are also implemented, with MethodInfo.IsReadOnly.
[Readonly storage and return signatures](readonly-storage.md) now preserve declared
permissions. The verifier remains conservative about aliases and lifetime provenance.

MethodInfo.IsReadOnly reports an enforced readonly receiver (separate from
ParameterInfo.IsReadOnly). Reassemble external System artifacts for the expanded
MethodInfo descriptor; static and value-receiver methods report false.

See [readonly storage and return signatures](readonly-storage.md) for implemented
readonly T& type positions, checked boundaries and migration. Binding immutability
remains a language feature. [Explicit nullability](nullability.md) is a planned
signature characteristic and special state, not implemented syntax or zeroing.

[Interface inheritance](interface-inheritance.md) now supplies transitive
GetInterfaces results. Member enumeration remains declared-only; query each base
interface separately for its declarations.

See [inherited value layout](inherited-layout.md) for the preliminary record-base
contract. Field descriptors retain their declaring-type-relative DefinitionIndex.

Type.IsAbstract and MethodInfo.IsVirtual/IsOverride/IsAbstract now expose the
[class-dispatch flags](class-dispatch.md). The [descriptor hierarchy](reflection-hierarchy.md)
is implemented; interface capabilities and default bodies remain in the
[follow-up plan](reflection-hierarchy-plan.md).
