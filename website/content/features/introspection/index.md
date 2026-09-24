# Runtime type and member metadata

From an object’s type to an assembly’s members, one descriptive model gives you a way to explore what a neoCLR program contains.

**Preview 9 implementation · September 19, 2026.** This guide describes the Preview 9 API. The examples run on neoCLR with its matching Raven toolchain; the design can still change.

[Follow the walkthrough ↓](#walkthrough) · [Download the complete sample](../../samples/library-introspection-tour.rvn)

<a id="walkthrough"></a>

## TypeInfo acquisition

Suppose a diagnostic tool wants to describe the types and members available to a program. It needs metadata, without running the methods it discovers. Introspection is that descriptive layer; invoking code is a separate, future capability.

The example declares an empty `Widget` class. Its instance is held through `Object`, but `GetType()` still describes the concrete allocation. `typeof(Widget)` describes the declared type. Both return the same public contract: `System.Introspection.TypeInfo`.

```raven
{{TOUR_ACQUISITION}}
```

Use `Equals` to compare type identity. Names are useful for display, but do not uniquely identify types across modules. There is no public `System.Type` class or intermediate `.Info` property in this Raven profile. Calling `GetType()` on null faults.

<a id="object-contracts"></a>

## Type identity through Object

**Development after Preview 9:** TypeInfo now compares represented types through both typed equality and Object.Equals. Repeated queries may allocate different descriptors; ReferenceEquals still compares those allocations. Generic arguments and array element types participate in type identity.

Object.GetHashCode is consistent with that equality, and ToString displays the represented FullName. Different types can have colliding hashes; neither names nor hashes are persistent identity keys. Equatable&lt;TypeInfo&gt; takes a non-null TypeInfo. Object.Equals(Object?) is the explicitly null-aware boundary.

Development assembly descriptors now compare full catalog identities; module descriptors compare that identity plus their module name. Their Object hashes use the same keys, and display returns the assembly FullName or module Name. This is scoped to one loaded program, without CLR loader-context semantics. Field, method and property descriptors now compare their kind, closed declaring type and definition index, with matching hashes and Name display. Parameter descriptors now retain owner kind, closed declaring type, definition index and position for equality and hashing. Tokens may be zero; a property index parameter remains distinct from its accessor parameter. Owner resolution through a public Member property is still future work. See the [introspection guide](../../docs/introspection.html) and [TypeInfo API reference](../../docs/api/System.Introspection.TypeInfo.html) for current contracts.

<a id="context"></a>

## Assembly discovery through RuntimeContext

`RuntimeContext.Current` describes the current loaded program. `ExecutingAssembly` identifies the assembly of the source caller through the runtime facade. A call made from a dependency reports that dependency, rather than always reporting the entry assembly.

```raven
{{TOUR_DISCOVERY}}
```

In the saved `Demo` project, the executing assembly references `System.Runtime`. That is the foundation’s logical identity; compiler bootstrap names are not additional platform dependencies. References are direct edges, not a recursive dependency listing.

`AssemblyInfo` describes an assembly; `ModuleInfo` describes a module within it. Both expose `GetTypes()`. The current implementation lists retained, loaded definitions, including nonpublic types—not every type from an original source assembly that the importer may have discarded.

<a id="tokens"></a>

## Module-scoped metadata tokens

```raven
{{TOUR_TOKENS}}
```

The token agrees here because both descriptions refer to `Widget` in the same module. A token alone is not a global identifier. Type, member and parameter interfaces expose `Module` alongside `MetadataToken`; the assembly and module interfaces expose their own tokens too.

Source definition tokens are preserved where the importer retains them. Merged runtime definitions receive module-scoped tokens. They are stable within an artifact, not a persistence key across rebuilds. Constructed generic types share their definition’s token. Arrays, pointer/by-reference wrappers and generic-parameter placeholders currently return zero; an absent parameter row also has token zero.

<a id="collections"></a>

## Metadata sequences

```raven
{{TOUR_SEQUENCES}}
```

Every public collection-returning Introspection method now uses `Sequence<T>`. That includes types, members, parameters, generic arguments, interfaces and enum names. The sample uses `Count`, an indexer and a `for` loop; Iterable-based query extensions also work.

The interface has no collection mutation members. The current implementation returns independent snapshots, but the interface alone does not promise immutable concrete storage. Sequence is invariant: a `Sequence<MethodInfo>` is not implicitly a `Sequence<MemberInfo>`; individual methods can still be passed as `MemberInfo`.

**Updating an older sample?** Replace array result annotations with `Sequence<Element>` and `Length` with `Count`. Rebuild against a matching reference and runtime library. Array assignment and indexer writes through the Sequence contract are rejected.

<a id="members"></a>

## Member kinds and pattern matching

The public Info contracts are sealed interfaces. The current `MemberInfo` hierarchy has four public cases, so Raven can check that this match covers the whole hierarchy.

```raven
{{TOUR_MATCH}}
```

Callers work with those public cases, without matching private runtime implementation classes. `TypeInfo` is a member case, so a nested type can be described as a member. `DeclaringType` returns `Option<TypeInfo>`: nested types and ordinary members have an owner; top-level types do not. `ParameterInfo` remains separate.

```raven
{{TOUR_MEMBERS}}
```

`BindingFlags` is an ordinary enum. Here its combined flags include nonpublic instance fields of `Date`. Metadata visibility does not grant access to read those fields or invoke private methods. Describing a property does not execute its accessor. Current member queries enumerate declarations on the requested type; they do not walk base types. BaseType can be inspected explicitly.

<a id="try"></a>

## Complete example

Use the matching Preview 9 compiler, runtime and reference library. Follow the [project-based setup guide](../../try/), then save this sample as Main.rvn and run the neoCLR task.

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

Sequence states the collection capability without requiring an array in the public contract. It permits future storage changes, at the cost of a deliberate API break for existing array-oriented callers. These choices aim for a coherent small API; they do not promise .NET binary compatibility or identical behavior in every edge case.

[Read the design record and primary .NET comparisons →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/introspection-design.md)

<a id="limits"></a>

## Implemented scope and limitations

- There is one loaded-program context. Dynamic assembly loading and resolution belong to future RuntimeContext work.
- Queries cover retained metadata. Application property projection and generic method-definition reflection remain limited.
- Open generic definitions can report identity, shape, arguments, tokens and module. Their member, base-type and interface queries require a closed type and fault otherwise.
- Dynamic invocation, emit and offline metadata contexts remain future work. TypeInfo is part of the sealed MemberInfo hierarchy.

The Introspection story closes at this boundary for now. The next slice is described in the [Strings and UTF-8 guide](../strings/).

<a id="objects"></a>

## Object and value semantics

Class assignment shares a reference; value assignment copies fields, including any references those fields contain. GetType preserves the concrete type through an Object view. System.Value is a separate temporary erased-storage facility used by carriers such as Option and Result; it is not the .NET ValueType base class.

The [Object and Value guide](../../docs/objects.html) explains current support and missing methods. Development Object.ReferenceEquals compares class, array and box identity; ordinary class/array Equals and GetHashCode use identity by default. Class overrides can provide equality and matching hashes. Object is abstract: construct a concrete application class, not Object itself. Development ToString supports a type-name fallback and class overrides. Named structs with explicit ToString overrides support boxed formatting. Boxed Int32 and Int64 produce culture-independent decimal text; Boolean produces True or False, matching .NET spelling. String through Object returns unchanged text; format strings, culture providers and other primitive boxed formatting remain unsupported. String Object equality/hash now use exact contents, while identity calls still explicitly fault until conversions preserve identity; boxed Int32 and Int64 virtual equality compare the complete stored integer only with the same concrete type. Int32 hashes to its value; Int64 hashes by XORing its two 32-bit halves, as in .NET. Hash collisions do not make values equal. These hashes are not persistent identifiers, while named structs dispatch their explicit Object overrides. Boxed Single and Double also support exact-type Object equality and hashing: NaNs of the same type compare equal, and positive/negative zero compare equal with matching hashes. Floating `==` retains IEEE behavior (NaN is unequal to itself); boxed floating display remains unsupported. Boxed Char compares and hashes the full grapheme text without normalization, and displays that text unchanged. ReferenceEquals still distinguishes separate boxes. Equals accepts a nullable comparison argument, and ReferenceEquals accepts nullable arguments on both sides. These reference annotations let Raven check calls against the existing runtime null behavior; the final metadata representation remains open. For APIs and domain models that express absence, neoCLR favors Option&lt;T&gt; for both value and reference types. Nullable structs and nullable-value boxing are deferred. The development Raven target rejects nullable value declarations such as `int?` with RAV0407: “Value types can't be declared as nullable.” Reference annotations such as `Object?` remain supported.

### Records (development)

Raven record classes now generate equality, hashing and display for integer, non-null string and same-compilation record-class components; class assignment still shares a reference. Strings compare by contents; nested records use typed equality and matching hashes. Nullable record references preserve null through equality, hashing, display and deconstruction. Development record structs now use the same component contract, generating typed/Object/interface equality, hashes, display and deconstruction while preserving value copying. The checked Coordinate sample compares values through Object and Equatable&lt;Coordinate&gt;; separate boxes retain distinct identities. A separate Point/Rectangle sample demonstrates nested record structs: equality and display use component methods, and construction/deconstruction copy the point values. Record classes can also contain record structs. Default struct initialization leaves reference fields null, including fields declared non-nullable. Generated record methods handle those defaults: null components compare safely, contribute zero to hashing and display as empty fields; deconstruction preserves null. The Defaults sample demonstrates this behavior. Generated Object.Equals also preserves the inherited nullable comparison parameter; the record sample checks null and boxed comparisons through Object? arguments. Typed record-class Equals accepts a nullable reference to the same record type, including literal null, and returns false for absence. Generated class == and != also accept nullable references: two absent references compare equal, one absent reference compares unequal, and present records compare by components. Record-struct typed Equals and operator operands still take values. Equatable&lt;T&gt; keeps its existing interface signature. Nullable string/value components, externally compiled record components and generic/inherited records remain unsupported. Ordinary structs need explicit Object overrides; automatic .NET ValueType field equality is not implemented. Boxed Boolean values also support Object equality and hashing: true and false compare only with Boolean values, and hash to 1 and 0 respectively. Boxing preserves a copy; separate boxes retain separate identities. System.HashCode provides a mutable accumulator for integer and non-null string components, with independent value copies.

```raven
{{OBJECT_DISPLAY_SAMPLE}}
```

[Download the checked Object display sample](../../samples/object-display.zip). Calling Describe with a Plain prints Plain; passing Named prints Named instance. The full sample also distinguishes virtual dispatch from an explicit base call.

The [record sample](../../samples/records.zip) checks record classes and structs, ordinary struct copies, box identity, component equality, display, deconstruction and hashing. The [Object equality sample](../../samples/object-equality.zip) shows a mutable Cell retaining identity and its hash, and two distinct Key objects providing equal values and matching hashes.

<a id="direction"></a>

## Planned work and open questions

The aim is one descriptive model that can later support reflection and emit. Dynamic assembly loading belongs with RuntimeContext; offline metadata could use a different resolution context. These are directions to explore, not implemented APIs or release commitments. Identity, resolution and lifetime rules need further work.

[See the proposals and their tradeoffs →](../../proposals/#introspection)

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

The generated reference describes development after Preview 9. Use the availability
notes above to distinguish it from the published toolchain.
