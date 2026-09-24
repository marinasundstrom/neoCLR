# NeoCLR Introspection Model — Capability-Oriented Metadata Proposal

## Author clarification — 2026-09-24

The immediate implementation requirement is to ensure TypeInfo has IsAbstract,
IsOpen, IsClosedHierarchy, IsUnion, IsEnum and IsValueType. IsUnion specifically means
**nominal union**; earlier illustrative IsUnion examples below must be read with
that clarification. IsOpen uses positive inheritance terminology; IsClosedHierarchy
separately describes a declared closed family. See the
[current contract](../introspection-design.md#type-classification-flags--development-2026-09-24).

The author also directs work toward the more extensive closed hierarchy of TypeInfo
interfaces described here. This is an incremental direction, not a request to
implement the entire proposal with the flags. Specialized contracts should retain
the nominal/non-nominal distinction and overlapping qualities described below.

## 1. Purpose

`System.Introspection` provides NeoCLR's model for describing program metadata and type-system information.

It should not reproduce the design of .NET's `System.Type`.

In .NET, `System.Type` serves several overlapping purposes: it identifies a type, describes metadata, classifies the type, provides member discovery, and acts as an entry point into runtime reflection. NeoCLR deliberately separates these concerns.

In particular:

```text
System.Introspection
    describes programs, declarations, types and metadata

System.Runtime.Reflection
    provides runtime binding and execution capabilities
```

Within Introspection itself, NeoCLR should also avoid concentrating every possible type characteristic into one large `TypeInfo` interface.

NeoCLR has types with fundamentally different qualities:

```text
nominal types
    User
    String
    InputStream
    MyEnum
    Result<T, E>

non-nominal type expressions
    String | Int32
    InputStream & OutputStream
    T&
    function types
    ...
```

A language may have syntax for representing all of these, but that does not mean every type has a name, declaration, members, namespace, or other nominal properties.

The model should therefore use **interfaces representing meaningful qualities**.

The central principle is:

> **Introspection interfaces describe the qualities an introspection object actually possesses. They are contracts, not merely nodes in a classification hierarchy.**

---

# 2. Core concepts

The model begins with a small number of foundational contracts.

Conceptually:

```raven
interface TypeInfo
{
    ...
}

interface MemberInfo
{
    Name: String
    ...
}

interface NominalTypeInfo :
    TypeInfo,
    MemberInfo
{
    ...
}

interface FunctionInfo :
    MemberInfo
{
    ...
}

interface MethodInfo :
    FunctionInfo
{
    ...
}
```

These interfaces express overlapping qualities.

For example:

```text
NominalTypeInfo

    is TypeInfo
    is MemberInfo
```

means:

> A nominal type is both a type and a named program member.

Likewise:

```text
MethodInfo

    is FunctionInfo
    is MemberInfo
```

means:

> A method is a function declaration that participates as a member of a type.

This should not be interpreted primarily as an object-oriented inheritance taxonomy.

The interfaces describe contracts that happen to overlap.

---

# 3. `TypeInfo`

`TypeInfo` is the common contract for anything that denotes a NeoCLR type.

```raven
interface TypeInfo
{
    ...
}
```

It should contain only information that is genuinely meaningful for arbitrary types, together with selected classifications that are useful across the type system.

For example, NeoCLR may expose:

```raven
interface TypeInfo
{
    IsNominal: bool
    IsUnion: bool
    IsIntersection: bool

    ...
}
```

The exact set of classifications remains separate API design work.

A classification such as:

```raven
type.IsNominal
```

is meaningful for every type because every type can answer the question.

That does not imply that nominal-specific information belongs on `TypeInfo`.

For example, this should not be necessary:

```raven
TypeInfo.Name: Option<String>
```

A non-nominal type does not have an absent name.

It simply does not possess the quality represented by `MemberInfo`.

---

# 4. `MemberInfo`

`MemberInfo` represents a named declaration with member identity.

Conceptually:

```raven
interface MemberInfo
{
    Name: String

    ...
}
```

The complete contract may eventually include qualities such as declaration metadata, accessibility, attributes, or ownership where they are genuinely common.

The important point is that `Name` is guaranteed.

The sealed member family can contain forms such as:

```text
MemberInfo

    NominalTypeInfo
    FunctionInfo
    PropertyInfo
    FieldInfo
    EventInfo
    ...
```

This gives metadata consumers a finite set of declaration/member forms.

It also establishes an important distinction:

```text
String | Int32
```

is a `TypeInfo`, but not a `MemberInfo`.

There is no member declaration called `String | Int32`.

---

# 5. Nominal types

A nominal type has declared identity in program metadata.

Examples include:

```text
class
struct
interface
enum
nominal union
```

The corresponding contract is:

```raven
interface NominalTypeInfo :
    TypeInfo,
    MemberInfo
{
    ...
}
```

A nominal type therefore possesses both:

```text
type identity
member/declaration identity
```

Properties such as these naturally belong on nominal types or their associated member contracts:

```text
Name
Namespace
Module
accessibility
declared attributes
declared members
generic parameters
```

They should not be pushed onto arbitrary `TypeInfo` objects.

---

# 6. Nominal type qualities

More specific interfaces describe additional nominal qualities:

```raven
interface ClassTypeInfo : NominalTypeInfo
{
    ...
}

interface StructTypeInfo : NominalTypeInfo
{
    ...
}

interface InterfaceTypeInfo : NominalTypeInfo
{
    ...
}

interface EnumTypeInfo : NominalTypeInfo
{
    ...
}

interface NominalUnionTypeInfo : NominalTypeInfo
{
    ...
}
```

These interfaces need not imply that Introspection is fundamentally a rigid inheritance hierarchy.

They express facts.

For example:

```text
MyEnum

TypeInfo             ✓
MemberInfo           ✓
NominalTypeInfo      ✓
EnumTypeInfo         ✓
```

Likewise:

```text
Result<T,E>

TypeInfo                  ✓
MemberInfo                ✓
NominalTypeInfo           ✓
NominalUnionTypeInfo      ✓
GenericTypeInfo           ✓
```

A type may possess several independently meaningful qualities.

---

# 7. Non-nominal types

NeoCLR also contains types that have no nominal declaration.

For example:

```raven
String | Int32
```

and:

```raven
InputStream & OutputStream
```

are genuine NeoCLR types.

They may appear in:

```text
method signatures
function signatures
locals
fields
generic arguments
generic constraints
metadata
```

but they are not named declarations.

Therefore they implement:

```text
TypeInfo
```

without implementing:

```text
MemberInfo
NominalTypeInfo
```

This distinction should be reflected directly in the Introspection contracts rather than represented through optional nominal properties.

---

# 8. Type unions

A native ad-hoc union can expose:

```raven
interface TypeUnionInfo : TypeInfo
{
    Types: List<TypeInfo>
}
```

For:

```raven
String | Int32
```

the introspection qualities are:

```text
TypeInfo             ✓
TypeUnionInfo        ✓

MemberInfo           ✗
NominalTypeInfo      ✗
```

The type has constituents:

```text
String
Int32
```

but it has no `Name`.

Raven's representation:

```raven
String | Int32
```

is source-language syntax for the type.

It is not the runtime name of the type.

Another NeoCLR language may display the same `TypeInfo` differently.

---

# 9. Type intersections

Likewise:

```raven
interface TypeIntersectionInfo : TypeInfo
{
    Types: List<TypeInfo>
}
```

For:

```raven
InputStream & OutputStream
```

the introspection qualities are:

```text
TypeInfo                    ✓
TypeIntersectionInfo        ✓

MemberInfo                  ✗
NominalTypeInfo             ✗
```

Again, the intersection is a real NeoCLR type-system expression but not a declared member.

This is particularly important now that unions and intersections may be represented natively in NeoCLR metadata rather than through nominal backing types.

---

# 10. Nominal unions and type unions

NeoCLR supports two distinct union concepts.

A nominal union:

```raven
Result<T, E>
```

has declared identity and cases:

```text
Result<T,E>
    Ok(T)
    Err(E)
```

It may expose:

```text
TypeInfo
MemberInfo
NominalTypeInfo
NominalUnionTypeInfo
GenericTypeInfo
```

An ad-hoc type union:

```raven
T | E
```

instead exposes:

```text
TypeInfo
TypeUnionInfo
```

It has constituent types rather than declared nominal cases.

These are not interchangeable concepts.

For example:

```raven
Result<T, E>
```

is not simply another representation of:

```raven
T | E
```

The former carries nominal case identity; the latter expresses a type relationship.

Whether some shared union-oriented Introspection contract eventually exists should be decided from actual consumer requirements rather than assumed merely because both concepts are unions.

---

# 11. Interfaces represent qualities

The Introspection API should not primarily ask:

> Which single kind of object is this?

Instead, it should ask:

> Which contracts truthfully describe this metadata object?

For example, a generic class may expose:

```text
TypeInfo
MemberInfo
NominalTypeInfo
ClassTypeInfo
GenericTypeInfo
```

A generic nominal union may expose:

```text
TypeInfo
MemberInfo
NominalTypeInfo
NominalUnionTypeInfo
GenericTypeInfo
```

An ad-hoc union may expose:

```text
TypeInfo
TypeUnionInfo
```

An intersection may expose:

```text
TypeInfo
TypeIntersectionInfo
```

No object needs to be forced into exactly one classification merely to satisfy an inheritance hierarchy.

---

# 12. Classification properties

The capability-oriented model does not prevent `TypeInfo` from exposing useful classifications.

For example:

```raven
if type.IsNominal
{
    ...
}
```

may be considerably more convenient than always attempting to narrow to `NominalTypeInfo`.

Likewise, classifications such as:

```raven
type.IsUnion
type.IsIntersection
```

may be useful.

The distinction is:

```text
classification property
    answers a meaningful question about any type

specialized interface
    exposes information available only when that quality exists
```

Therefore:

```raven
TypeInfo.IsNominal: bool
```

can coexist naturally with:

```raven
NominalTypeInfo : TypeInfo, MemberInfo
```

The former discovers the quality.

The latter provides its contract.

---

# 13. Avoid recreating `System.Type` through booleans

Classification properties should nevertheless be added conservatively.

The model should not simply replace a broad `System.Type` API with:

```text
IsNominal
IsClass
IsStruct
IsInterface
IsEnum
IsUnion
IsIntersection
IsArray
IsReference
IsFunction
IsGeneric
IsGenericInstantiation
...
```

for every specialized interface that happens to exist.

Common and useful classifications belong on `TypeInfo`.

Less common distinctions can be discovered through the specialized contracts themselves.

The API should balance:

```text
convenient discovery
```

against:

```text
precise contracts
```

rather than mechanically mirroring every interface with an `IsX` property.

---

# 14. Shared traits across boundaries

Some meaningful qualities may cross the nominal/non-nominal boundary.

For example, future API design may discover that nominal unions and type unions share a useful introspection contract.

Other type forms may share generic, callable, sequence-like, or other metadata characteristics.

The interface model allows such overlaps.

However, cross-cutting interfaces should be introduced conservatively.

The existence of similar properties is not sufficient reason to create a shared contract.

For example:

```text
ArrayTypeInfo.ElementType
ReferenceTypeInfo.ReferencedType
```

do not automatically justify:

```raven
interface WrappedTypeInfo
{
    InnerType: TypeInfo
}
```

unless consumers have a meaningful reason to treat arrays and references uniformly.

---

# 15. Rule for shared interfaces

Before introducing a cross-cutting Introspection contract, the design should ask:

> **Can a consumer meaningfully depend on this contract without immediately needing to know which underlying metadata form supplied it?**

If yes, a shared interface may represent a real semantic quality.

If consumers immediately need:

```raven
if array ...
else if reference ...
else if ...
```

then the shared abstraction is probably not useful.

This deliberately permits some duplicated property shapes.

Duplication is preferable to an abstraction that has no coherent semantic meaning.

The principle is:

> **Shared interfaces describe shared semantics, not merely shared representation.**

---

# 16. Functions are declarations

Functions introduce another important distinction between **declarations** and **types**.

Consider:

```raven
func Parse(text: String) -> Result<Value, ParseError>
```

There is a function declaration named `Parse`.

That declaration should be represented by:

```raven
interface FunctionInfo : MemberInfo
{
    Parameters: List<ParameterInfo>
    ReturnType: TypeInfo

    ...
}
```

`FunctionInfo` therefore represents a **function declaration**, not the type of a callable value.

Because it is a declared member, it has:

```text
Name
parameters
return type
generic parameters, where applicable
attributes
other declaration metadata
```

The exact function contract remains separate API work.

---

# 17. Methods are functions in a type

A method is a function whose declaration belongs to a nominal type.

Therefore:

```raven
interface MethodInfo : FunctionInfo
{
    DeclaringType: NominalTypeInfo

    ...
}
```

Conceptually:

```text
FunctionInfo
    function declaration

MethodInfo
    function declaration belonging to a type
```

For:

```raven
class Parser
{
    func Parse(text: String) -> Result<Value, ParseError>
}
```

`Parse` exposes:

```text
MemberInfo       ✓
FunctionInfo     ✓
MethodInfo       ✓
```

The method contract can add information that only makes sense for functions declared on types.

This might eventually include distinctions involving:

```text
instance/static behavior
virtual dispatch
overrides
receiver information
```

where appropriate.

---

# 18. Namespace or module functions

A free function such as:

```raven
namespace Parsing;

func Parse(text: String) -> Result<Value, ParseError>
```

is still a function declaration.

It therefore exposes:

```text
MemberInfo       ✓
FunctionInfo     ✓
MethodInfo       ✗
```

This suggests that `MemberInfo` should not mean specifically:

> member of a type.

Instead, it represents a named metadata declaration/member within an appropriate declaration scope.

A method is the specialized case where that containing scope is a nominal type.

The exact model for namespace/module ownership should be defined alongside `NamespaceInfo` and `ModuleInfo`.

---

# 19. Function types are types, not functions

The type of a function is a separate concept.

For example, a language may express a callable type conceptually as:

```raven
(String) -> Result<Value, ParseError>
```

That type does not become a `FunctionInfo` merely because a function declaration happens to have it.

It should instead be represented through a type-oriented contract such as:

```raven
interface FunctionTypeInfo : TypeInfo
{
    ParameterTypes: List<TypeInfo>
    ReturnType: TypeInfo

    ...
}
```

The distinction is:

```text
FunctionInfo
    describes a function declaration

FunctionTypeInfo
    describes a callable type
```

This mirrors the broader Introspection model.

A declaration and the type associated with that declaration are not the same metadata concept.

---

# 20. Function declaration versus function type

Consider:

```raven
func Parse(text: String) -> Result<Value, ParseError>
```

Introspection may expose:

```text
FunctionInfo
    Name = Parse
    Parameters:
        text: String
    ReturnType:
        Result<Value, ParseError>
```

The callable type associated with the function may independently be:

```text
FunctionTypeInfo
    ParameterTypes:
        String
    ReturnType:
        Result<Value, ParseError>
```

The former has declaration identity.

The latter does not necessarily have a name at all.

This is analogous to:

```text
nominal type                 type expression

Result<T,E>                  T | E

function declaration         function type

Parse                        (String) -> Result<Value,E>
```

The Introspection model should preserve these distinctions rather than collapsing them because the objects are related.

---

# 21. Anonymous functions

An anonymous function or lambda may have a function type without corresponding to a named `FunctionInfo` declaration in metadata.

For example:

```raven
let parser = (text: String) => Parse(text);
```

may have a type described through:

```text
FunctionTypeInfo
```

without there necessarily being a:

```text
FunctionInfo
MemberInfo
```

representing the source-level lambda.

Whether compiler-generated implementation methods are exposed separately is a metadata/compiler question.

The semantic callable type should not depend on that implementation detail.

---

# 22. Functions and methods demonstrate contract composition

The relationship between functions and methods illustrates the broader Introspection philosophy particularly well.

```text
                           MemberInfo
                          /          \
                         /            \
          NominalTypeInfo          FunctionInfo
                                      │
                                      │
                                  MethodInfo
```

But this diagram should not be interpreted as the primary architecture.

The more useful view is:

```text
User
    TypeInfo             ✓
    MemberInfo           ✓
    NominalTypeInfo      ✓
    ClassTypeInfo        ✓

Parse (free function)
    MemberInfo           ✓
    FunctionInfo         ✓

Parser.Parse
    MemberInfo           ✓
    FunctionInfo         ✓
    MethodInfo           ✓

(String) -> Result<...>
    TypeInfo             ✓
    FunctionTypeInfo     ✓
```

Each object exposes the contracts that truthfully describe it.

---

# 23. Generic qualities

Generics provide another example of qualities that may cross other classifications.

For example:

```raven
class List<T>
{
}
```

may expose:

```text
TypeInfo
MemberInfo
NominalTypeInfo
ClassTypeInfo
GenericTypeInfo
```

Likewise:

```raven
union Result<T, E>
{
    Ok(T),
    Err(E)
}
```

may expose:

```text
TypeInfo
MemberInfo
NominalTypeInfo
NominalUnionTypeInfo
GenericTypeInfo
```

A generic function may expose:

```text
MemberInfo
FunctionInfo
GenericFunctionInfo
```

if a separate generic-function contract proves useful.

Again, a shared generic interface should be introduced only if there is a meaningful common contract across these declaration forms.

The existence of type parameters alone does not require every generic concept to be forced through one large abstraction.

---

# 24. Nominal types as members

The relationship:

```raven
interface NominalTypeInfo :
    TypeInfo,
    MemberInfo
```

has a useful consequence for the sealed `MemberInfo` family.

continue`MemberInfo` and that member is a type, it is necessarily a `NominalTypeInfo`.

There is no separate member representation for arbitrary type expressions.

For example:

```raven
class Container
{
    class Item
    {
    }
}
```

`Item` is represented directly as:

```text
TypeInfo             ✓
MemberInfo           ✓
NominalTypeInfo      ✓
ClassTypeInfo        ✓
```

There is no need for a separate wrapper such as:

```text
NestedTypeMemberInfo
    Type: NominalTypeInfo
```

The nominal type itself is the member.

This gives a useful invariant:

> **Types that participate as declared members are nominal types. Non-nominal type expressions do not independently participate in the member model.**

Thus none of these can independently appear as a `MemberInfo`:

```raven
String | Int32
InputStream & OutputStream
(String) -> Result<Value, Error>
```

They may occur *within* member signatures, but they are not themselves member declarations.

---

# 25. Nested nominal types

A nominal type may be declared within another nominal type:

```raven
class Container
{
    class Item
    {
    }
}
```

In this case, `Item` remains the same kind of introspection object as any other nominal class type.

Its context provides the additional fact that its declaring scope is another nominal type.

Conceptually:

```text
Item

TypeInfo             ✓
MemberInfo           ✓
NominalTypeInfo      ✓
ClassTypeInfo        ✓

DeclaringType:
    Container
```

A top-level nominal type may instead belong to a namespace or module.

The model should therefore avoid treating "nested type" as a fundamentally separate kind of type.

Nesting is a declaration relationship.

The type remains nominal regardless of where it is declared.

---

# 26. Member signatures use `TypeInfo`

Although only nominal types are themselves members, members may freely refer to arbitrary NeoCLR types.

For example:

```raven
func Read(
    value: InputStream & OutputStream
) -> String | Error
```

could be represented conceptually as:

```text
FunctionInfo
    Name:
        Read

    Parameters:
        value
            Type:
                TypeIntersectionInfo
                    InputStream
                    OutputStream

    ReturnType:
        TypeUnionInfo
            String
            Error
```

This is an important reason for member signatures to depend on:

```raven
TypeInfo
```

rather than:

```raven
NominalTypeInfo
```

A parameter or return type need not have nominal identity.

The same applies to:

```text
fields
properties
events
generic constraints
function types
base/interface relationships where permitted
```

according to the rules of the NeoCLR type system.

---

# 27. Parameters are not necessarily members

Parameters illustrate another boundary worth preserving.

Given:

```raven
func Parse(text: String) -> Result<Value, Error>
```

`text` has a name and metadata, but that does not necessarily mean it should implement `MemberInfo`.

A parameter is part of a function signature rather than an independently addressable member declaration.

It may therefore have its own contract:

```raven
interface ParameterInfo
{
    Name: String
    Type: TypeInfo

    ...
}
```

Likewise, generic parameters may have their own introspection contracts.

The existence of a `Name` property alone should not be enough to classify something as `MemberInfo`.

`MemberInfo` should represent a specific metadata relationship, not merely "anything with a name."

This is another example of why shared interfaces should be based on semantics rather than property shape.

---

# 28. Member ownership

Different kinds of members may belong to different declaration scopes.

For example:

```text
nominal type
    namespace / module
    nominal type when nested

free function
    namespace / module

method
    nominal type

property
    nominal type

field
    nominal type
```

The common `MemberInfo` contract should not necessarily force every member to expose the same owner type if that concept does not apply universally.

Instead, ownership can be expressed through appropriately scoped contracts or specialized interfaces.

For example:

```raven
interface MethodInfo : FunctionInfo
{
    DeclaringType: NominalTypeInfo
}
```

while a free function might expose its namespace/module context through `FunctionInfo` or another declaration-context contract.

The exact ownership API should be designed around the NeoCLR metadata model rather than copied from `.NET MemberInfo.DeclaringType`.

---

# 29. Type formatting is separate from Introspection identity

Every type needs to be representable to humans and tooling, including types without names.

For example:

```raven
String | Int32
InputStream & OutputStream
(String) -> Result<Value, Error>
```

all have useful textual representations.

But those strings are not necessarily properties of the underlying type metadata.

A Raven formatter might produce:

```text
String | Int32
```

while another language might produce:

```text
Union[String, Int32]
```

for the same NeoCLR `TypeUnionInfo`.

Therefore:

```text
nominal name
    metadata identity

type formatting
    language/tool representation
```

should remain distinct.

`MemberInfo.Name` provides the former where it exists.

A formatting facility can provide the latter for arbitrary `TypeInfo`.

---

# 30. Introspection consumers

The capability model allows consumers to request the information they actually require.

A metadata browser interested in declarations might operate primarily on:

```raven
MemberInfo
```

A compiler performing type checking works primarily with:

```raven
TypeInfo
```

A documentation generator interested in named types might require:

```raven
NominalTypeInfo
```

A union analyzer might consume:

```raven
TypeUnionInfo
```

A callable-signature analyzer might consume:

```raven
FunctionTypeInfo
```

while a tool inspecting declared functions consumes:

```raven
FunctionInfo
```

This avoids requiring every consumer to understand one enormous metadata object containing every possible concept.

---

# 31. Introspection and runtime Reflection

The distinction between Introspection and Reflection remains fundamental.

For example:

```raven
interface MethodInfo : FunctionInfo
{
    ...
}
```

describes the method.

It does not necessarily provide:

```raven
Invoke(...)
```

Likewise:

```raven
NominalTypeInfo
```

does not necessarily provide:

```raven
CreateInstance(...)
```

and:

```raven
PropertyInfo
```

does not necessarily provide runtime get/set operations.

Those capabilities belong to:

```text
System.Runtime.Reflection
```

which can provide behavior over Introspection objects where a corresponding runtime entity is available.

Conceptually:

```text
System.Introspection

    TypeInfo
    MemberInfo
    FunctionInfo
    MethodInfo
    PropertyInfo
    ...


System.Runtime.Reflection

    runtime binding
    invocation
    construction
    runtime member access
```

This allows Introspection to work against metadata that has never been loaded for execution.

---

# 32. Metadata context

The capability-oriented model should work equally well when inspecting external metadata.

For example:

```text
MetadataContext
    ↓
AssemblyInfo
    ↓
NominalTypeInfo
    ↓
MethodInfo
    ↓
parameter TypeInfo
```

The parameter type may itself be:

```text
TypeUnionInfo
TypeIntersectionInfo
FunctionTypeInfo
```

without requiring the assembly or any constituent type to be loaded into a runtime execution context.

This is one of the reasons `TypeInfo` must remain a metadata model rather than becoming a runtime type handle.

---

# 33. Runtime context

A separate runtime context may expose information about loaded types and assemblies.

Conceptually:

```text
RuntimeContext
    loaded assemblies
    loaded nominal types
    runtime binding capabilities
```

The relationship between a runtime-loaded entity and its corresponding `TypeInfo` can be provided by the Reflection layer.

This keeps:

```text
What does this type mean?
```

separate from:

```text
Is this type loaded?
Can I instantiate it?
Can I invoke this method?
```

The distinction applies equally to nominal and non-nominal types.

---

# 34. Initial contract map

The model can initially be thought of as a set of overlapping contracts:

```text
Type information
────────────────

TypeInfo
├── NominalTypeInfo ─────────────┐
├── TypeUnionInfo                │
├── TypeIntersectionInfo         │
├── FunctionTypeInfo             │
├── ArrayTypeInfo                │
├── ReferenceTypeInfo            │
├── GenericParameterInfo         │
└── ...                          │
                                 │
Member information               │
──────────────────               │
                                 │
MemberInfo ◄─────────────────────┘
├── FunctionInfo
│     └── MethodInfo
├── PropertyInfo
├── FieldInfo
├── EventInfo
└── ...
```

This diagram is illustrative rather than a requirement for a strict inheritance tree.

The important relationships are the contracts.

For example:

```text
NominalTypeInfo
    TypeInfo       ✓
    MemberInfo     ✓

MethodInfo
    MemberInfo     ✓
    FunctionInfo   ✓

TypeUnionInfo
    TypeInfo       ✓
    MemberInfo     ✗

FunctionTypeInfo
    TypeInfo       ✓
    MemberInfo     ✗
```

---

# 35. Example: nominal class

Given:

```raven
class Parser
{
    func Parse(text: String) -> Result<Value, ParseError>
}
```

the class may expose:

```text
Parser

TypeInfo
MemberInfo
NominalTypeInfo
ClassTypeInfo
```

Its method may expose:

```text
Parser.Parse

MemberInfo
FunctionInfo
MethodInfo
```

The parameter:

```text
text
```

is represented by `ParameterInfo`.

Its type:

```text
String
```

is a `TypeInfo`, specifically a nominal type.

The return type:

```text
Result<Value, ParseError>
```

is likewise a `TypeInfo`, with nominal-union qualities.

Each metadata concept is represented according to what it actually is.

---

# 36. Example: compound signature

Given:

```raven
func Transform(
    stream: InputStream & OutputStream,
    converter: (Bytes) -> String
) -> String | TransformError
```

the function declaration is:

```text
FunctionInfo
MemberInfo
```

Its first parameter type is:

```text
TypeInfo
TypeIntersectionInfo
```

Its second parameter type is:

```text
TypeInfo
FunctionTypeInfo
```

Its return type is:

```text
TypeInfo
TypeUnionInfo
```

None of those type expressions need names merely because they occur in the signature of a named function.

That distinction is encoded directly by the interfaces rather than by nullable/optional properties.

---

# 37. Example: nominal union

Given:

```raven
union Result<T, E>
{
    Ok(T),
    Err(E)
}
```

the declaration may expose:

```text
TypeInfo
MemberInfo
NominalTypeInfo
NominalUnionTypeInfo
GenericTypeInfo
```

Its cases are part of the nominal-union metadata model.

This remains distinct from:

```raven
T | E
```

which exposes:

```text
TypeInfo
TypeUnionInfo
```

The runtime may share union machinery between them, but Introspection preserves their semantic distinction.

---

# 38. Conservative evolution

The initial Introspection model should prefer a small number of strong interfaces over a large vocabulary of speculative traits.

New interfaces should be introduced when concrete consumers demonstrate that a shared semantic contract exists.

This gives NeoCLR room to evolve.

For example, if experience later demonstrates that several type forms meaningfully expose the same generic-substitution capability, a shared interface can be added.

Because the model uses interfaces rather than a rigid class hierarchy, adding such a contract does not require reorganizing the entire taxonomy.

The desired evolution is:

```text
start with strong concepts
        ↓
observe real common behavior
        ↓
introduce shared contract
```

rather than:

```text
notice similar properties
        ↓
invent abstraction immediately
```

---

# 39. Design principles

The `System.Introspection` model should follow these principles:

1. **`TypeInfo` represents arbitrary NeoCLR type information.** It does not imply nominal identity.

2. **`MemberInfo` represents named metadata declarations with member identity.**

3. **`NominalTypeInfo` implements both `TypeInfo` and `MemberInfo`.** Nominal types are both types and declared members.

4. **Non-nominal type expressions do not acquire nominal properties.** A union or intersection may have language syntax without having a runtime name.

5. **Specialized interfaces describe qualities, not merely positions in a hierarchy.** An introspection object may implement every contract that truthfully describes it.

6. **Classification properties may exist on common interfaces when they are genuinely useful.** They should not be used to recreate a giant `System.Type` API through booleans.

7. **Shared trait interfaces should be introduced conservatively.** Similar property shapes alone do not justify a shared contract.

8. **`FunctionInfo` represents a function declaration. `FunctionTypeInfo` represents a callable type.** Declaration and type are separate concepts.

9. **`MethodInfo` represents a function declared as a member of a nominal type.**

10. **Member signatures refer to `TypeInfo`, not `NominalTypeInfo`.** Parameters, returns, fields, and other signatures may use non-nominal type expressions.

11. **Introspection describes metadata rather than runtime execution capabilities.** Invocation, construction, and runtime binding belong to `System.Runtime.Reflection`.

12. **The model should capture meaningful qualities without forcing every metadata object into one rigid taxonomy.**

---

# 40. Proposed direction

NeoCLR should build `System.Introspection` around a **capability-oriented interface model**.

At its foundation:

```raven
interface TypeInfo
{
    // Universal type information and selected classifications.
}

interface MemberInfo
{
    Name: String

    // Universal member/declaration information.
}

interface NominalTypeInfo :
    TypeInfo,
    MemberInfo
{
    // Nominal type information.
}

interface FunctionInfo :
    MemberInfo
{
    Parameters: List<ParameterInfo>
    ReturnType: TypeInfo
}

interface MethodInfo :
    FunctionInfo
{
    DeclaringType: NominalTypeInfo
}

interface TypeUnionInfo : TypeInfo
{
    Types: List<TypeInfo>
}

interface TypeIntersectionInfo : TypeInfo
{
    Types: List<TypeInfo>
}

interface FunctionTypeInfo : TypeInfo
{
    ParameterTypes: List<TypeInfo>
    ReturnType: TypeInfo
}
```

These signatures are illustrative rather than final API definitions.

The important architecture is the relationship between the contracts:

```text
                    TypeInfo
                   /   |    \
                  /    |     \
                 /     |      \
                ▼      ▼       ▼
       NominalType  TypeUnion  FunctionType
            │
            │
            ▼
        MemberInfo
            ▲
            │
      FunctionInfo
            │
            ▼
        MethodInfo
```

Again, this should be read as **overlapping qualities**, not as a requirement to construct a conventional inheritance tree.

The resulting model avoids both extremes:

```text
one enormous Type object
```

and:

```text
hundreds of tiny trait interfaces
```

Instead, NeoCLR can expose a relatively small set of contracts corresponding to concepts that genuinely exist in its metadata and type system.

The guiding question for every interface should be:

> **What does this contract guarantee that a consumer can meaningfully depend upon?**

That should be the foundation for the continued design of `System.Introspection`.

