# Reflection descriptor hierarchy

Implemented 2026-09-08 after [constructor chaining](constructor-chaining.md).
System.Reflection.MemberInfo is an abstract record containing Name and DeclaringType.
FieldInfo, MethodInfo and PropertyInfo derive directly from it. ParameterInfo and
System.Type remain independent. There is no mandatory Object root.

## API and addressing

MemberInfo.Name and MemberInfo.DeclaringType use readonly managed receivers. All
instance readers on the three derived member descriptors also use readonly managed
receivers. Public property and method names remain familiar; the common getters
are declared once on MemberInfo.

```swift
func NameOf(readonly member: System.Reflection.MemberInfo&) -> string {
    return member.Name
}

let fields = typeof(Counter).GetFields()
let field = fields[0]
WriteLine(NameOf(&field))
WriteLine(field.DeclaringType.Name)
```

Queries still return concrete value snapshots and value arrays. A FieldInfo value
cannot substitute for a MemberInfo value: that would slice the descriptor. A
FieldInfo& or readonly FieldInfo& can project to a suitable MemberInfo& view.
GetType on that view reports the complete concrete descriptor type.

For heterogeneous collections, store managed references to heap-backed descriptors:

```swift
let storage = new System.Reflection.FieldInfo[1] { fields[0] }
var members = System.Collections.ArrayList<readonly System.Reflection.MemberInfo&>.Allocate(1)
members.Add(&storage[0])
WriteLine(members[0].Name)
```

The list retains the entire heap array owner through the interior reference.
A reference to a frame-local descriptor cannot be embedded in that managed list.
The sample uses heap arrays to select storage explicitly; it does not introduce
general source syntax for promoting or copying arbitrary values onto the heap.

## Neo projection versus reflection enumeration

The compiler now resolves bundled class properties, indexers and instance methods
through the class ancestry and emits the declaring owner's signature plus a managed
base projection. Virtual library accessors/methods use callvirt when appropriate.
Readonly access to an addressable value borrows that location. For a non-addressable
temporary, Neo stores the evaluated value in a hidden local and borrows it; the
expression executes once. This does not grant permission to mutate an immutable
binding or extend a frame reference's lifetime.

Member enumeration has a separate contract: GetFields/GetMethods/GetProperties
remain declared-only, with or without BindingFlags.DeclaredOnly. To enumerate the
Name and DeclaringType declarations, inspect MemberInfo or walk BaseType.
Calling field.Name works because of inherited language lookup; it does not imply
that GetProperties(FieldInfo) enumerates inherited properties. Constructors remain
excluded from GetMethods. Inherited reflection filtering/override suppression and
ReflectedType semantics require a later query-design slice.

## Construction, identity and access

MemberInfo and each derived descriptor have internal managed constructors. Derived
constructors chain to MemberInfo before initializing their own fields. Applications
cannot call these internal constructors or write the private fields.

Trusted runtime reflection factories still materialize complete snapshots directly;
querying metadata does not execute these constructors. The flattened descriptor
field order is preserved: Name and DeclaringType remain the first two fields.
Tests compare an internally constructor-built FieldInfo with its runtime-generated
snapshot and check its metadata layout. The hierarchy changes declaring ownership,
not the identity of the member being described or the inspected program's objects.

No new runtime helper, opcode or metadata field is introduced. ParameterInfo keeps
its existing value-receiver readers. Type descriptors remain ordinary independent
values, and no equality/hash or mandatory Object policy changes accompany this slice.

## .NET comparison and decision

Primary sources consulted 2026-09-08:

- [.NET 10 MemberInfo](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.memberinfo?view=net-10.0)
  supplies the common member abstraction. Its abstract API covers a broader set of
  members and implements ICustomAttributeProvider. neoCLR currently needs two shared
  metadata fields and three concrete descriptor kinds.
- [.NET 10 MethodBase](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.methodbase?view=net-10.0)
  groups method and constructor behavior. neoCLR postpones that intermediate layer
  until constructor introspection supplies a second consumer.
- [.NET 10 Type.GetProperties](https://learn.microsoft.com/en-us/dotnet/api/system.type.getproperties?view=net-10.0)
  distinguishes inherited results from DeclaredOnly filtering. neoCLR's current
  always-declared-only queries remain a deliberate preview limitation.

The [comparison probe](experiments/reflection-hierarchy-dotnet/Program.cs) pins
SDK 10.0.100/net10.0. It prints Age:Model, Read:Model and Count:Model through common
MemberInfo references; confirms MethodInfo → MethodBase → MemberInfo; and prints
2 versus 1 properties with default versus DeclaredOnly queries. No performance or
CoreCLR representation claim is inferred.

Keeping independent records would preserve duplicated storage/readers. A common
interface could expose the metadata but would still duplicate that storage and
construction. The small base reuses both and supplies a concrete test of managed
base views. Its costs are ancestry-aware lookup and a receiver-contract migration.
The full .NET hierarchy would add currently unused abstractions, while deriving
System.Type would complicate DeclaringType/optionality without a present need.
These are scoped choices, not claims that class inheritance always beats interfaces.

Use interfaces for independent capabilities when an actual consumer needs them.
Custom attribute inspection is a candidate, following the role of .NET's
ICustomAttributeProvider while using Neo's naming convention. The
[bridge between inherited class interface implementations and virtual overrides](class-interface-dispatch.md)
is now implemented, as are [explicit interface mappings](explicit-interfaces.md).
[Default interface bodies](default-interface-implementations.md) now implement
precedence, diamond ambiguity, reabstraction and managed receiver rules.

## Run and migration

```sh
cargo run --locked -- run examples/source/reflection-hierarchy.neo --gc-stats
cargo run --locked -- verify examples/source/reflection-hierarchy.neo
cargo run --locked -- run examples/reflection.neoil
cargo test --locked --test reflection_hierarchy --test reflection
(cd docs/experiments/reflection-hierarchy-dotnet && dotnet run)
```

The Neo sample prints three descriptor names/concrete types and returns 3.

Recompile Neo sources against the matching System library. Existing property syntax
remains valid. Raw IL must supply managed receivers to descriptor readers; use
ldloca/ldelema or an existing reference. Calls to Name/DeclaringType must name
MemberInfo and project the receiver with castclass MemberInfo. IL method resolution
still uses exact declaring-owner signatures rather than implicit ancestor lookup.

Existing serialized applications bound to removed duplicate getters or old receiver
contracts require recompilation. Field layout is preserved, but metadata member rows,
BaseType and declared member/property enumeration change. Do not mix older System
artifacts with applications compiled for this hierarchy.
