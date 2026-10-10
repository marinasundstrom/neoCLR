# Runtime type and member metadata

From an object’s type to an assembly’s members, one descriptive model gives you a way to explore what a neoCLR program contains.

[Follow the walkthrough ↓](#walkthrough) · [Download the complete sample](../../samples/library-introspection-tour.rvn)

<a id="walkthrough"></a>

## TypeInfo acquisition

Suppose a diagnostic tool wants to describe the types and members available to a program. It needs metadata, without running the methods it discovers. Introspection is that descriptive layer; execution belongs to the separate [Reflection feature](../reflection/).

The example declares an empty `Widget` class. Its instance is held through `Object`, but `GetType()` still describes the concrete allocation. `typeof(Widget)` describes the declared type. Both return the same public contract: `System.Introspection.TypeInfo`.

```raven
{{TOUR_ACQUISITION}}
```

Use `Equals` to compare type identity. Names are useful for display, but do not uniquely identify types across modules. There is no public `System.Type` class or intermediate `.Info` property in this Raven profile. Calling `GetType()` on null faults.

<a id="object-contracts"></a>

## Type identity through Object

TypeInfo compares represented types through both typed equality and Object.Equals. Repeated queries may allocate different descriptors; ReferenceEquals still compares those allocations. Generic arguments and array element types participate in type identity.

Object.GetHashCode is consistent with that equality, and ToString displays the represented DisplayName. Different types can have colliding hashes; neither names nor hashes are persistent identity keys. EquatableTo&lt;TypeInfo&gt; takes a non-null TypeInfo. Object.Equals(Object?) is the explicitly null-aware boundary.

Assembly descriptors compare full catalog identities; module descriptors compare that identity plus their module name. Their Object hashes use the same keys, and display returns the assembly FullName or module Name. This is scoped to one loaded program, without CLR loader-context semantics. Field, method and property descriptors compare their kind, closed declaring type and definition index, with matching hashes and Name display. Parameter descriptors retain owner kind, closed declaring type, definition index and position for equality and hashing. Tokens may be zero; a property index parameter remains distinct from its accessor parameter. Owner resolution through a public Member property is still future work. See the [introspection guide](../../docs/introspection.html) and [TypeInfo API reference](../../docs/api/System.Introspection.TypeInfo.html) for current contracts.

<a id="type-classification-development"></a>

## Type classification

TypeInfo exposes IsAbstract, IsOpen, IsClosedHierarchy, IsUnion, IsEnum and
IsValueType. IsOpen means open to unrestricted inheritance or implementation.
IsClosedHierarchy describes a declared closed family, while IsUnion specifically
identifies a nominal union such as Option or Result. A non-inheritable leaf is not
itself a closed family. These flags describe metadata; the fuller hierarchy of
specialized TypeInfo interfaces remains planned.

<a id="context"></a>

## Assembly discovery through RuntimeContext

`RuntimeContext.Current` describes the current loaded program. `ExecutingAssembly` identifies the assembly of the source caller through the runtime facade. A call made from a dependency reports that dependency, rather than always reporting the entry assembly.

```raven
{{TOUR_DISCOVERY}}
```

In the saved `Demo` project, the executing assembly references `System.Runtime`. That is the foundation’s logical identity; compiler bootstrap names are not additional platform dependencies. References are direct edges, not a recursive dependency listing.

`AssemblyInfo` describes an assembly; development `ModuleInfo` describes a logical namespace of members within it. `GetModules()` returns flat qualified names, including declared empty modules; dotted names do not add metadata hierarchy. Both expose `GetTypes()`, with modules selecting exact logical ownership, including nested types. The current implementation lists retained, loaded definitions, including nonpublic types—not every type from an original source assembly that the importer may have discarded.

<a id="tokens"></a>

## Module-scoped metadata tokens

```raven
{{TOUR_TOKENS}}
```

The token agrees here because both descriptions refer to `Widget` in the same module. A token alone is not a global identifier. Type, member and parameter interfaces expose logical `Module` alongside physical definition tokens. Development ModuleInfo no longer exposes MetadataToken: a logical module is not a physical metadata row. Those tokens must not be resolved using a logical module name.

Source definition tokens are preserved where the importer retains them. Merged runtime definitions receive module-scoped tokens. They are stable within an artifact, not a persistence key across rebuilds. Constructed generic types share their definition’s token. Arrays, pointer/by-reference wrappers and generic-parameter placeholders currently return zero; an absent parameter row also has token zero.

<a id="collections"></a>

## Metadata sequences

```raven
{{TOUR_SEQUENCES}}
```

Every public collection-returning Introspection method uses `Sequence<T>`. That includes types, members, parameters, generic arguments, interfaces and enum names. The sample uses `Count`, an indexer and a `for` loop; Iterable-based query extensions also work.

The interface has no collection mutation members. The current implementation returns independent snapshots, but the interface alone does not promise immutable concrete storage. Sequence is invariant: a `Sequence<MethodInfo>` is not implicitly a `Sequence<MemberInfo>`; individual methods can still be passed as `MemberInfo`.

Sequence does not expose element replacement.

<a id="members"></a>

## Member kinds and pattern matching

The public Info contracts are sealed interfaces. A match can distinguish fields,
methods, constructors, properties and nominal types:

```raven
{{TOUR_MATCH}}
```

Callers work with those public cases, without matching private runtime implementation classes. `NominalTypeInfo` describes nominal types as members, including nested types. `DeclaringType` returns `Option<TypeInfo>`: nested types and ordinary members have an owner; top-level types do not. `ParameterInfo` remains separate.

```raven
{{TOUR_MEMBERS}}
```

`BindingFlags` is an ordinary enum. Here its combined flags include nonpublic instance fields of `Date`. Metadata visibility does not grant access to read those fields or invoke private methods. Describing a property does not execute its accessor. Current member queries enumerate declarations on the requested type; they do not walk base types. BaseType can be inspected explicitly.

<a id="try"></a>

## Complete example

Use matching development compiler, runtime and reference artifacts for this tour.
Follow the [project-based setup guide](../../try/), then save this sample as Main.rvn
and run the neoCLR task.

1. [Download the complete Raven sample](../../samples/library-introspection-tour.rvn), including its imports, Widget class, helper and Main function.
2. Copy it into `Main.rvn` in the prepared `Demo` project and save.
3. Choose **Terminal → Run Task → neoCLR: Run saved project**. The task compiles, imports, verifies and runs the saved program on neoCLR.

Raven’s ordinary Run/Debug commands target .NET; use the neoCLR task for this walkthrough. For the project named Demo and the current library snapshot, the expected output is:

```text
{{TOUR_OUTPUT}}
```

The method count and parameter name reflect this preview’s metadata inventory, not a promise that later releases keep the same members or ordering. The snippets and [expected output](../../samples/library-introspection-tour.expected.txt) come from the same files used by the saved-project check.

<a id="design"></a>

## Comparison with .NET

The .NET comparison informs this API’s ergonomics: assembly/module descriptions, member queries, BindingFlags and module-scoped metadata tokens are familiar concepts. neoCLR places ambient discovery on RuntimeContext and returns one TypeInfo model directly from both forms of type acquisition.

.NET’s `Assembly.GetReferencedAssemblies()` returns assembly-name identities. This preview instead returns resolved AssemblyInfo descriptions, making traversal convenient but requiring references to exist in the loaded catalog. An unavailable reference faults explicitly; it is neither hidden nor loaded from disk.

Sequence states the collection capability without requiring an array in the public contract. It permits future storage changes, at the cost of requiring callers to use the narrower collection contract. These choices aim for a coherent small API; they do not promise .NET binary compatibility or identical behavior in every edge case.

[Read the design record and primary .NET comparisons →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/introspection-design.md)

<a id="limits"></a>

## Implemented scope and limitations

- There is one loaded-program context. Dynamic assembly loading and resolution belong to future RuntimeContext work.
- Queries cover retained metadata. Development includes application instance properties and accessor tokens; static application properties and generic method-definition reflection remain limited.
- Open generic definitions can report identity, shape, arguments, tokens and module. Their member, base-type and interface queries require a closed type and fault otherwise.
- Emit and offline metadata contexts remain future work; bounded invocation is covered by [Reflection](../reflection/). NominalTypeInfo is part of the development sealed MemberInfo hierarchy.

Constructor discovery adds `GetConstructors()` and the `ConstructorInfo`
member case. It returns declared public instance constructors by default; explicit
BindingFlags can inspect nonpublic declarations. Constructors remain separate from
GetMethods. Exhaustive matches over MemberInfo include ConstructorInfo, as in the
example above.

[Reflection](../reflection/) separately supplies Result-based construction, method
invocation and member access. Metadata discovery alone grants no execution permission.

<a id="objects"></a>

## Object and value semantics

Class assignment shares a reference; value assignment copies fields, including any references those fields contain. GetType preserves the concrete type through an Object view. System.Value is a separate temporary erased-storage facility used by carriers such as Option and Result; it is not the .NET ValueType base class.

The [Object and Value guide](../../docs/objects.html) explains current support and missing methods. Object.ReferenceEquals compares class, array and box identity; ordinary class/array Equals and GetHashCode use identity by default. Class overrides can provide equality and matching hashes. Object is abstract: construct a concrete application class, not Object itself. ToString supports a type-name fallback and class overrides. Named structs with explicit ToString overrides support boxed formatting. Boxed Int32 and Int64 produce culture-independent decimal text; Boolean produces True or False, matching .NET spelling. String through Object returns unchanged text; format strings, culture providers and other primitive boxed formatting remain unsupported. String Object equality/hash use exact contents, while identity calls compare the retained immutable text owner; boxed Int32 and Int64 virtual equality compare the complete stored integer only with the same concrete type. Int32 hashes to its value; Int64 hashes by XORing its two 32-bit halves, as in .NET. Hash collisions do not make values equal. These hashes are not persistent identifiers, while named structs dispatch their explicit Object overrides. Boxed Single and Double also support exact-type Object equality and hashing: NaNs of the same type compare equal, and positive/negative zero compare equal with matching hashes. Floating `==` retains IEEE behavior (NaN is unequal to itself); boxed floating display remains unsupported. Boxed Char compares and hashes the full grapheme text without normalization, and displays that text unchanged. ReferenceEquals still distinguishes separate boxes. Equals accepts a nullable comparison argument, and ReferenceEquals accepts nullable arguments on both sides. These reference annotations let Raven check calls against the existing runtime null behavior; the final metadata representation remains open. For APIs and domain models that express absence, neoCLR favors Option&lt;T&gt; for both value and reference types. Nullable structs and nullable-value boxing are deferred. The development Raven target rejects nullable value declarations such as `int?` with RAV0407: “Value types can't be declared as nullable.” Reference annotations such as `Object?` remain supported.

<a id="records-development"></a>

### Records

Raven record classes generate equality, hashing and display for integer, non-null string and same-compilation record-class components; class assignment still shares a reference. Strings compare by contents; nested records use typed equality and matching hashes. Nullable record references preserve null through equality, hashing, display and deconstruction. Record structs use the same component contract, generating typed/Object/interface equality, hashes, display and deconstruction while preserving value copying. The checked Coordinate sample compares values through Object and EquatableTo&lt;Coordinate&gt;; separate boxes retain distinct identities. A separate Point/Rectangle sample demonstrates nested record structs: equality and display use component methods, and construction/deconstruction copy the point values. Record classes can also contain record structs. Default struct initialization leaves reference fields null, including fields declared non-nullable. Generated record methods handle those defaults: null components compare safely, contribute zero to hashing and display as empty fields; deconstruction preserves null. The Defaults sample demonstrates this behavior. Generated Object.Equals also preserves the inherited nullable comparison parameter; the record sample checks null and boxed comparisons through Object? arguments. Typed record-class Equals accepts a nullable reference to the same record type, including literal null, and returns false for absence. Generated class == and != also accept nullable references: two absent references compare equal, one absent reference compares unequal, and present records compare by components. Record-struct typed Equals and operator operands still take values. EquatableTo&lt;T&gt; keeps its existing interface signature. Nullable string/value components, externally compiled record components and generic/inherited records remain unsupported. Ordinary structs need explicit Object overrides; automatic .NET ValueType field equality is not implemented. Boxed Boolean values also support Object equality and hashing: true and false compare only with Boolean values, and hash to 1 and 0 respectively. Boxing preserves a copy; separate boxes retain separate identities. System.HashCode provides a mutable accumulator for integer and non-null string components, with independent value copies.

```raven
{{OBJECT_DISPLAY_SAMPLE}}
```

[Download the checked Object display sample](../../samples/object-display.zip). Calling Describe with a Plain prints Plain; passing Named prints Named instance. The full sample also distinguishes virtual dispatch from an explicit base call.

The [record sample](../../samples/records.zip) checks record classes and structs, ordinary struct copies, box identity, component equality, display, deconstruction and hashing. The [Object equality sample](../../samples/object-equality.zip) shows a mutable Cell retaining identity and its hash, and two distinct Key objects providing equal values and matching hashes.

<a id="direction"></a>

## Planned work and open questions

The descriptive model supports the bounded development reflection extensions above. Broader invocation and emit remain future work. Dynamic assembly loading belongs with RuntimeContext; offline metadata could use a different resolution context. These are directions to explore, not implemented APIs or release commitments. Identity, resolution and lifetime rules need further work.

[See the proposals and their tradeoffs →](../../proposals/#introspection)

`NominalTypeInfo` provides names, module paths and declaration metadata, alongside
`TypeInfo.IsNominalType`.
Structural Function types describe callable shapes, with Function objects
as their instances. Common
TypeInfo exposes DisplayName, IsNominalType and IsFunctionType; declaration metadata requires
NominalTypeInfo. Structural types can still have members and extension members;
member discovery stays on common TypeInfo. Raven callbacks use structural
Function shapes, and extensions can target a function shape. FunctionTypeInfo.InvokeMethod and
GetMethods expose the same synthesized public instance Invoke signature. Member
and parameter Module/MetadataToken, and MethodInfo.DefinitionIndex, are optional;
synthesized descriptors return None. Typed invocation is supported; dynamic
reflection invocation of this descriptor remains future work. See the [Function API reference](/docs/functions.html). Named function
types may follow later.

Development FunctionTypeInfo also exposes Parameters and ReturnType directly.
InvokeMethod remains the member-reflection view; no generalized function-info
interface is introduced.

<a id="feedback"></a>

## Questions and contributions

- Does RuntimeContext make the ownership of discovery clear?
- Do Sequence results provide enough capability for your metadata tooling?
- Would your application need to inspect unresolved reference identities?
- Which concrete use case is blocked by the current discovery limits?

Share the scenario, the sample you tried and the behavior you expected. Questions and criticism are welcome.

[Discuss on GitHub ↗](https://github.com/marinasundstrom/neoCLR/issues)

Questions, sample programs and documentation corrections are welcome. See [how to contribute](../../#feedback) for ways to participate.

## API reference

[System.Introspection](xref:System.Introspection)


## Enum names and values

APIs on `System.Enum` support both a known enum and discovery through
`TypeInfo`:

```raven
let names = Enum.GetNames<EntryKind>()
let values = Enum.GetValues<EntryKind>()
let boxed: Object = EntryKind.File
let discovered = Enum.GetValues(boxed.GetType())
```

The generic values stay typed as `EntryKind`; discovery returns boxed enum values.
Both return fresh `Sequence` snapshots ordered by unsigned underlying value, with
aliases retained. Supported enum examples include `EntryKind`, `TaskState` and
`BindingFlags`. Generic calls reject non-enum arguments; a non-enum `TypeInfo` faults.
This sample uses imports from `System` and `System.Storage`.

Boxed enums format named values, flags combinations and unnamed numeric values.
An unnamed zero prints `0`. This follows the basic .NET behavior, with stable
metadata order for aliases and Sequence results instead of public array contracts.
The current generic implementation allocates an intermediate boxed snapshot.
See [Enum API reference](/docs/api/System/Enum/) for both overload families.

EquatableTo&lt;T&gt; and ComparableTo&lt;T&gt; describe equality and comparison.
ConvertibleInto&lt;T&gt; supplies an explicit Convert() contract with an implementation-defined policy.

<a id="development-case-inspecting-route-declarations-at-startup"></a>

## Case: inspecting route declarations at startup

A server can describe routes with attributes on union cases, then inspect those
declarations once during startup. This example reads the catalog routes:

```raven
{{ATTRIBUTE_ROUTE_DECLARATION}}
```

`RoutePatternAttribute` is an application attribute with a String constructor
argument. Introspection reads its data without constructing the attribute:

```raven
{{ATTRIBUTE_ROUTE_READING}}
```

`MemberInfo.GetCustomAttributesData()` also works for fields, properties,
constructors and methods. `ParameterInfo` exposes the same method. Multiple
attributes are returned individually in metadata order, including repeated
attributes allowed by their declarations. Each description identifies its type,
constructor and typed arguments.

The [tested source archive](/samples/http-json.zip) includes `attribute-introspection`
and its verifier. This development slice supports retained application attributes
with String, Int32 and Boolean constructor constants. It does not instantiate
attributes, merge inherited attributes or expose all framework annotations. See
[attribute data and limits](/docs/introspection.html#attribute-data-development-after-preview-10).

Use these descriptions to prepare and cache a mapping before request handling.
The [attributed HTTP case](/cases/http-server/#case-attributed-item-routes) combines
these descriptions with retained constructors, compiled patterns and capture
conversions. Source generation remains a future alternative.


The [retained constructor case](../reflection/#development-case-preparing-union-constructors-for-routes)
uses these attribute descriptions to select case/carrier constructors once, then
constructs ordinary union values through checked Reflection extensions.

See [Function types and objects](../functions/) for signature identity, value bindings
and the transitional bound-target Function property.

The separate [C# metadata API](/docs/experimental-metadata.html) reads and authors native
assemblies on the host. It is distinct from the guest introspection APIs described here.
Development host tooling exposes logical modules directly through `ModuleInfo`,
including empty modules and direct member ownership. Dotted names are a convention;
module metadata is flat. Assemblies package one or several modules. Physical token
scopes remain separate reader details. See the [host module API](/docs/experimental-metadata.html#context-owned-declaration-views-development-2026-10-10).
Guest traversal through the executing assembly is now qualified in the interpreter. AOT supports explicitly retained type-to-module name inspection; assembly-wide native traversal remains open. See the [module API and limits](/docs/introspection.html#logical-modules-development-2026-10-10).


## Development: general custom attributes

Host metadata tooling now preserves and inspects annotations on types, callables,
fields, properties and parameters across native and CLI snapshots. Inspection runs
no attribute constructors. Host enum constructor arguments and primitive named
field/property data also survive native/CLI projection. Raven native import now
applies usage policies across assemblies. Source annotation emission now passes focused
compiler checks. A development TestAttribute consumer now discovers module tests
through host introspection and produces typed Raven registration adapters. These are not yet released
runtime capabilities. See the [API contract](../../docs/experimental-metadata.html#member-custom-attributes-development-2026-10-10)
and [implementation plan](https://github.com/marinasundstrom/neoCLR/blob/main/docs/custom-attributes.md).

Explicit development AOT retention now runs attribute-data inspection through the
public Raven API on macOS ARM64 and Windows x64 without executing attribute constructors. Fixed
arguments, Int32 enum values and member/parameter annotations are covered by a
[metadata-authored consumer](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/native-attributes/README.md).
Named field/property argument snapshots also pass on macOS in both modes with the
updated source library. The [Windows action](https://github.com/marinasundstrom/neoCLR/actions/runs/38045805683) also succeeds at 96ad94e2; independent artifact-hash verification remains open.
See the [named argument API](/docs/introspection.html#named-attribute-arguments-development-2026-10-10).
The [testing helper reference](/docs/testing.html) covers descriptions, signature
validation, manual registration and the host/AOT boundary. Guest in-process test
discovery remains open.

Development source libraries also expose AttributeTargets and AttributeUsageAttribute
with .NET flag values and policy defaults (AllowMultiple=false, Inherited=true).
Imported target validation is covered by the
[usage consumer](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/attribute-usage/README.md).
Inherited guest queries and guest in-process discovery remain future work. See the
[AttributeUsageAttribute reference](/docs/api/System.AttributeUsageAttribute.html).
