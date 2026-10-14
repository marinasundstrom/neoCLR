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

### Development: union classification

On the structural-type experimental branch, `TypeInfo.IsUnion` is reserved for
structural unions. Use `NominalTypeInfo.IsUnionDeclaration` to detect nominal Raven
union declarations. Experimental UnionTypeInfo and IntersectionTypeInfo expose direct constituent
snapshots in the interpreter. Member/hierarchy queries and compiler structural
type-token emission remain under development. This is not part of the published
release; see the [descriptor contract](/docs/introspection.html#structural-composition-descriptors-experimental-2026-10-14).
