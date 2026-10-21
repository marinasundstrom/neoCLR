# Introspection

Describe a program's types, members and assemblies without executing the members
you discover. `System.Introspection.TypeInfo` is the shared description contract.
Use [reflection](/features/reflection/) when you need to construct objects or invoke them.

## Get a type description

`typeof(Widget)` describes a declared type. Calling `GetType()` through an Object
still describes the concrete allocation:

```raven
{{TOUR_ACQUISITION}}
```

Compare descriptions with `Equals`, not reference identity or a display name.
Names and metadata tokens are not global or persistent identifiers. Calling
`GetType()` on null faults.

## Find assemblies and modules

`RuntimeContext.Current` describes the loaded program. Its `ExecutingAssembly`
identifies the source caller's assembly, including when called from a dependency.

```raven
{{TOUR_DISCOVERY}}
```

An assembly contributes logical modules, each corresponding to a namespace.
Dotted names are conventions, not a metadata hierarchy. More than one assembly can
contribute the same module name. Discovery lists retained, loaded definitions.

## Inspect members

Type descriptions expose methods, fields, constructors and properties. These
sequences describe members; reading them does not invoke an accessor or method.

```raven
{{TOUR_MEMBERS}}
```

Queries enumerate declarations on the requested type rather than walking its base
types. Inspect `BaseType` explicitly when needed. Nonpublic metadata visibility
does not grant permission to invoke or read those members.

## Work with the results

Metadata collections use `Sequence<T>`. You can iterate them, count them and use
query operators. The interface has no mutation operations; it does not promise
that every implementing object has immutable storage.

Descriptors distinguish nominal types, member kinds, generic arguments and array
elements. Attribute data can describe annotations without constructing attribute
objects. Native builds require explicit retention for metadata accessed dynamically.

[Download the complete introspection tour](/samples/library-introspection-tour.rvn).
The tour covers type identity, modules, metadata tokens and member classification.

## Read further

- [Introspection contracts](/docs/introspection.html) — identity, visibility, attributes and limits.
- [TypeInfo](xref:System.Introspection.TypeInfo) and [module APIs](xref:System.Introspection.ModuleInfo).
- [Reflection](/features/reflection/) — construct, invoke and access members.
- [Native compilation](/features/native-compilation/) — retain metadata for dynamic use.

<a id="walkthrough"></a>

<a id="object-contracts"></a>

<a id="type-classification-development"></a>

<a id="context"></a>

<a id="tokens"></a>

<a id="collections"></a>

<a id="members"></a>

<a id="try"></a>

<a id="design"></a>

<a id="limits"></a>

<a id="objects"></a>

<a id="records-development"></a>

<a id="direction"></a>

<a id="feedback"></a>

<a id="development-case-inspecting-route-declarations-at-startup"></a>

<a id="development-union-classification"></a>

### Union classification

`TypeInfo.IsUnion` describes structural unions. Use
`NominalTypeInfo.IsUnionDeclaration` for nominal Raven union declarations.
UnionTypeInfo and IntersectionTypeInfo expose direct constituent snapshots.
Interface constituents remain nominal and require explicit implementation.
Member/hierarchy queries, open-generic compositions and backend coverage remain
limited; further structural work is parked. See the
[descriptor contract](/docs/introspection.html#structural-composition-descriptors-experimental-2026-10-14).

## Generic parameter information

Generic type parameters expose `GenericParameterTypeInfo.Variance`:
`Invariant`, `Covariant` (`out`) or `Contravariant` (`in`). Obtain parameters from
an open definition's generic arguments. Closed constructions expose actual argument
types. Method `ParameterInfo.IsOut` continues to describe an output argument.
Native discovery supports retained assembly/module type catalogs,
including open definitions and their variance annotations. AOT/JIT require explicit
`typeCatalogs` retention; open member/relationship queries remain unsupported.
See the [reflection reference](/docs/reflection.html).

<a id="development-field-and-initializer-facts"></a>

## Field and initializer facts

FieldInfo distinguishes readonly storage and Int32 literals through `IsInitOnly`,
`IsLiteral` and `ConstantValue`. TypeInfo exposes `IsBeforeFieldInit` and an optional
`TypeInitializer` constructor descriptor. Inspection never runs the initializer.
See the [introspection reference](/docs/introspection.html) and
[reflection execution limits](/docs/reflection.html#fields-and-type-initializers-development).

## Generic state

The development `TypeInfo` API distinguishes `IsGenericType`,
`IsGenericTypeDefinition`, `IsConstructedGenericType` and `IsGenericParameter`.
`ContainsGenericParameters` checks the whole identity recursively. A closed
`ArrayList<int>` is constructed and generic, but contains no unbound parameters.
An inheritance-open class need not be generic: `IsOpen` keeps its inheritance meaning.
Arrays and structural function types are not nominal generic types.

DI uses this shared contract to reject unbound registrations. The development
`MethodInfo` API also exposes `IsGenericMethod`, `IsGenericMethodDefinition` and
`ContainsGenericParameters`. An ordinary method on an open generic owner can
contain unbound parameters without itself being generic. Interpreted method
discovery preserves these signature identities. Native discovery requires explicit
method retention for the inspected declarations. It does not grant reflective
invocation or runtime generic construction.
