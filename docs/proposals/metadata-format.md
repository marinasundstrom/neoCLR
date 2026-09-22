Yes. I think the right goal is **“ECMA-335 metadata with deliberate extensions,” not “invent a new metadata system.”** We should preserve the existing table/heap/token/signature architecture wherever it still fits, so an existing metadata reader can at least enumerate the image and understand the conventional portions even if it cannot semantically understand every NeoCLR extension.

The proposal should probably be framed around that compatibility objective.

# NeoCLR Metadata Model — CLR-Compatible Extensions Proposal

## 1. Purpose

NeoCLR should base its assembly metadata format closely on the existing CLI metadata model.

The goal is not binary compatibility with the CLR type system. NeoCLR intentionally introduces concepts that the existing CLI cannot fully represent, including:

```text
free functions
native union type expressions
native intersection type expressions
runtime-visible nullability
nominal unions
additional NeoCLR type relationships
```

However, these additions should be made **as conservatively as possible**.

The guiding principle is:

> **Preserve the existing CLI metadata architecture and encodings where they remain sufficient; extend them only where NeoCLR requires additional semantics.**

Ideally, an existing metadata reader should still be able to parse the metadata container, enumerate ordinary tables and understand metadata that uses existing CLI forms.

A reader that does not understand NeoCLR extensions may be unable to interpret particular signatures or declarations, but it should not fail merely because the entire metadata organization has been replaced.

---

# 2. Preserve the CLI metadata architecture

NeoCLR should retain the major structural concepts of CLI metadata:

```text
Metadata root
│
├── metadata streams
│
├── tables
│   ├── Assembly
│   ├── Module
│   ├── TypeDef
│   ├── TypeRef
│   ├── MethodDef
│   ├── Field
│   ├── Property
│   ├── Event
│   ├── GenericParam
│   ├── CustomAttribute
│   └── ...
│
├── string heap
├── blob heap
├── GUID heap
└── user-string heap
```

Likewise, NeoCLR should preserve where practical:

```text
metadata tokens
table indexes
coded indexes
signature blobs
generic instantiation encoding
custom attributes
assembly/module references
```

NeoCLR should extend this model rather than replacing it with a completely unrelated serialization format.

---

# 3. Separate declarations from type expressions

The metadata model should conceptually distinguish two things.

**Declarations** introduce program entities:

```text
Assembly
Module
Namespace
Nominal type
Function
Method
Field
Property
Event
...
```

**Type expressions** describe types used by those declarations:

```text
nominal type reference
generic parameter
generic instantiation
array
reference
nullable type
union
intersection
function type
...
```

A declaration may refer to arbitrary type expressions.

For example:

```raven
func Transform(
    stream: InputStream & OutputStream
) -> String | TransformError
```

contains one function declaration but several type expressions.

This distinction should also correspond naturally to `System.Introspection`:

```text
Metadata                     Introspection

TypeDef                  →   NominalTypeInfo
FunctionDef              →   FunctionInfo
MethodDef                →   MethodInfo

type signature           →   TypeInfo
union expression         →   TypeUnionInfo
intersection expression  →   TypeIntersectionInfo
nullable expression      →   NullableTypeInfo
function type            →   FunctionTypeInfo
```

---

# 4. Nominal types remain `TypeDef`

NeoCLR should retain `TypeDef` as the fundamental representation of a nominal type.

Classes, structs, interfaces, enums and nominal unions all have nominal identity and therefore belong in the nominal type metadata model.

Conceptually:

```text
TypeDef
    Flags
    Name
    Namespace
    Extends
    FieldList
    MethodList
    ...
```

should remain recognizable from CLI metadata.

NeoCLR may extend the available type flags or attach additional metadata describing new nominal type forms, but should not introduce a parallel `ClassDef`, `StructDef`, `InterfaceDef`, `UnionDef`, etc. unless there is a compelling structural reason.

The existing model already expresses an important NeoCLR concept well:

> `TypeDef` means a named type declaration.

This corresponds naturally to:

```raven
NominalTypeInfo
```

in `System.Introspection`.

---

# 5. Nominal unions

Nominal unions such as:

```raven
union Result<T, E>
{
    Ok(T),
    Err(E)
}
```

should remain `TypeDef`-based nominal types.

Likewise:

```raven
union Option<T>
{
    Some(T),
    None
}
```

They should not be represented as ad-hoc:

```raven
T | E
```

or:

```raven
T | ()
```

because their named cases carry semantic identity.

NeoCLR therefore needs metadata describing the cases belonging to a nominal union.

This could be introduced through a small additional metadata table, conceptually:

```text
UnionCase
---------
Owner       TypeDef
Name        String
Payload     Signature / Type
Flags
```

giving:

```text
TypeDef: Result<T,E>

UnionCase:
    Owner   → Result<T,E>
    Name    → Ok
    Payload → T

UnionCase:
    Owner   → Result<T,E>
    Name    → Err
    Payload → E
```

The exact physical encoding requires further design, but the existing `TypeDef` remains the nominal identity.

---

# 6. Methods remain methods

Methods declared inside nominal types should continue to use the existing `MethodDef` concept.

For example:

```raven
class Parser
{
    func Parse(text: String) -> Document
}
```

maps naturally to:

```text
TypeDef Parser
    MethodDef Parse
```

This preserves the existing CLR model where it is already appropriate.

NeoCLR does not need to redesign method metadata merely because it also supports free functions.

---

# 7. Free functions

The major declaration-level extension is support for functions that do not belong to a nominal type.

For example:

```raven
namespace System.Text;

func Encode(...) -> ...
func Decode(...) -> ...
```

should not require a synthetic:

```raven
class <Module>
{
    static func Encode(...)
    static func Decode(...)
}
```

or another artificial containing type.

Functions should have first-class metadata representation.

The least invasive extension would be a new table conceptually similar to `MethodDef`:

```text
FunctionDef
-----------
RVA
ImplFlags
Flags
Name
Signature
ParamList
```

The record can deliberately mirror `MethodDef` where the concepts overlap.

The important difference is that a `FunctionDef` is not owned by a `TypeDef`.

This maps naturally to:

```text
FunctionDef     → FunctionInfo
MethodDef       → MethodInfo
```

while allowing both to reuse the same signature and implementation machinery.

---

# 8. Function and method signatures

NeoCLR should avoid separate signature languages for functions and methods.

Both fundamentally have:

```text
generic arity
parameters
return type
calling convention
```

Therefore the existing method-signature encoding should be generalized or extended only as necessary.

Conceptually:

```text
FunctionSignature
    CallingConvention
    GenericParameterCount
    ParameterCount
    ReturnType
    Parameters[]
```

can be used by both:

```text
MethodDef
FunctionDef
```

A method may additionally have receiver/type-member semantics that a free function does not.

This distinction belongs primarily to the declaration, not the callable signature.

---

# 9. Namespace ownership of functions

Free functions need a namespace identity.

The CLI currently largely stores namespaces as strings associated with types rather than as first-class declaration scopes.

NeoCLR has two broad options.

The least invasive approach is for `FunctionDef` to contain a namespace string/index in addition to its name:

```text
FunctionDef
    Name
    Namespace
    Signature
    ...
```

This follows the existing `TypeDef` model closely.

For example:

```raven
namespace System.Text;

func Encode(...)
```

becomes approximately:

```text
FunctionDef
    Namespace = "System.Text"
    Name      = "Encode"
```

This avoids introducing first-class `NamespaceDef` metadata immediately.

NeoCLR may later decide that namespaces deserve actual metadata identity, but free functions do not require that larger change.

For an initial format, **namespace strings are likely the more conservative extension**.

---

# 10. Type expressions remain signature-based

The CLI already has a compact signature grammar for describing types.

NeoCLR should extend this grammar rather than create a separate table for every non-nominal type.

Existing signature forms already cover concepts such as:

```text
class/value type references
generic parameters
generic instantiations
arrays
pointers
managed references
function pointers
```

NeoCLR can add additional element/type codes for its richer type system.

Conceptually:

```text
ELEMENT_TYPE_...
ELEMENT_TYPE_UNION
ELEMENT_TYPE_INTERSECTION
ELEMENT_TYPE_NULLABLE
...
```

The exact names and binary values are implementation details.

The important principle is:

> **Non-nominal type expressions are encoded in signatures; nominal types continue to be referenced through metadata tokens.**

---

# 11. Native type unions

An ad-hoc union:

```raven
String | Int32
```

should be encoded directly in the signature type grammar.

Conceptually:

```text
UNION
    Count = 2
    Type = String
    Type = Int32
```

For example, a return type:

```raven
func Read() -> String | Int32
```

contains a signature representing:

```text
ReturnType:
    Union
        TypeRef String
        TypeRef Int32
```

There is no:

```text
Union<String, Int32>
```

`TypeDef` or generic instantiation.

The union exists as a type expression in metadata.

---

# 12. Native type intersections

Likewise:

```raven
InputStream & OutputStream
```

should be represented directly:

```text
INTERSECTION
    Count = 2
    Type = InputStream
    Type = OutputStream
```

A parameter:

```raven
func Rewrite(
    stream: InputStream & OutputStream
)
```

therefore contains:

```text
ParameterType:
    Intersection
        TypeRef InputStream
        TypeRef OutputStream
```

Again, no nominal:

```text
Intersection<InputStream, OutputStream>
```

backing type is required.

---

# 13. Nullability as a type expression

NeoCLR should investigate encoding nullability directly in the signature type grammar.

For example:

```raven
String?
```

could be represented conceptually as:

```text
NULLABLE
    TypeRef String
```

rather than:

```text
String
+ compiler-specific NullableAttribute
```

This makes nullability part of NeoCLR's type metadata.

A signature such as:

```raven
func Find(name: String?) -> User?
```

becomes:

```text
Parameter:
    Nullable
        String

Return:
    Nullable
        User
```

This allows every NeoCLR language and metadata consumer to observe the same nullability semantics without reconstructing language-specific attributes.

---

# 14. Nullability and nominal types

Nullability should not alter the nominal identity of the underlying type.

For example:

```text
String
```

remains a nominal type represented by a `TypeDef`/`TypeRef`.

```text
String?
```

is a non-nominal type expression applied to that nominal type:

```text
Nullable
└── String
```

In Introspection:

```text
String
    TypeInfo
    NominalTypeInfo

String?
    TypeInfo
    NullableTypeInfo
        UnderlyingType → String
```

This is analogous to:

```text
String | Int32
```

being a type expression composed from two nominal types.

---

# 15. Nullability and `Option<T>`

NeoCLR should preserve the distinction between:

```raven
String?
```

and:

```raven
Option<String>
```

The former is a nullable type expression.

The latter is a nominal union.

Conceptually:

```text
String?

    nullable String reference/value


Option<String>

    nominal type Option<String>
    cases:
        Some(String)
        None
```

They may occasionally model similar application states, but they have different type-system semantics and metadata identity.

---

# 16. Function types

If NeoCLR supports first-class function types, those should likewise be represented as type expressions rather than nominal declarations.

For example:

```raven
(String, Int32) -> Result<Value, Error>
```

could use an extension of the existing CLI function-pointer/callable signature machinery.

Conceptually:

```text
FUNCTION_TYPE
    Parameters:
        String
        Int32

    Return:
        Result<Value, Error>
```

This is distinct from `FunctionDef`.

The distinction is:

```text
FunctionDef
    declared function

Function type expression
    type of a callable value
```

which corresponds directly to:

```text
FunctionInfo
FunctionTypeInfo
```

in Introspection.

---

# 17. Generic constraints

Existing CLI generic metadata should be retained where possible:

```text
GenericParam
GenericParamConstraint
```

However, NeoCLR constraints can now refer to richer type expressions.

For example:

```raven
where T : InputStream & OutputStream
```

should allow `GenericParamConstraint` to reference an intersection type expression.

Likewise:

```raven
where T : Request | Event
```

should allow a union expression.

This may require extending what `GenericParamConstraint` can reference.

The existing CLI coded index is oriented toward nominal type references. NeoCLR may therefore need either:

```text
an extended coded index capable of referring to a TypeSpec
```

or an equivalent small modification.

This is preferable to creating separate:

```text
UnionConstraint
IntersectionConstraint
```

tables.

The constraint is simply a type expression.

---

# 18. `TypeSpec` becomes more important

The existing CLI `TypeSpec` mechanism is a natural home for NeoCLR's richer non-nominal types.

A `TypeSpec` already describes a type using a signature blob.

NeoCLR can extend that signature grammar to represent:

```text
nullable types
union types
intersection types
function types
other future constructed type forms
```

This gives a particularly conservative extension strategy:

```text
Nominal type
    TypeDef / TypeRef

Constructed/non-nominal type
    TypeSpec
        signature blob
```

That maps extremely well to the Introspection distinction we've just designed.

---

# 19. Mapping to Introspection

The metadata model can map naturally to `System.Introspection`.

```text
Metadata                         Introspection

TypeDef / TypeRef            →  NominalTypeInfo

TypeSpec: Union              →  TypeUnionInfo

TypeSpec: Intersection       →  TypeIntersectionInfo

TypeSpec: Nullable           →  NullableTypeInfo

TypeSpec: Function           →  FunctionTypeInfo

MethodDef                    →  MethodInfo

FunctionDef                  →  FunctionInfo

Field                        →  FieldInfo

Property                     →  PropertyInfo

Event                        →  EventInfo
```

This relationship should be intentional.

The metadata format and Introspection API are different layers, but they should describe the same conceptual model.

---

# 20. Nominal type references

Existing CLI distinctions between:

```text
TypeDef
TypeRef
TypeSpec
```

remain useful.

Conceptually:

```text
TypeDef
    nominal type defined in this module

TypeRef
    nominal type defined elsewhere

TypeSpec
    type expression composed from other types
```

NeoCLR can retain this model almost unchanged.

That gives:

```raven
User
```

as a `TypeDef` or `TypeRef`, while:

```raven
User?
```

is a `TypeSpec`.

Likewise:

```raven
User | Error
```

is a `TypeSpec`.

And:

```raven
InputStream & OutputStream
```

is a `TypeSpec`.

This is probably one of the strongest reasons **not** to invent a completely new metadata architecture.

The CLI already has almost exactly the nominal-versus-type-expression distinction NeoCLR needs.

---

# 21. Existing-reader compatibility

NeoCLR should aim for **structural readability**, not semantic compatibility, with existing CLI metadata readers.

An existing reader should ideally still be able to:

```text
read the metadata root
enumerate known tables
read strings/blobs
resolve ordinary TypeDef/TypeRef records
read conventional MethodDef records
inspect ordinary custom attributes
```

It may not understand:

```text
FunctionDef
new TypeSpec element codes
nominal union case metadata
NeoCLR-specific constraints
```

That is acceptable.

The goal is graceful partial understanding rather than pretending a NeoCLR assembly is an ordinary CLR assembly.

---

# 22. Unknown tables

Adding entirely new tables creates a compatibility concern.

Traditional metadata readers may assume a known table schema when interpreting the tables stream.

NeoCLR should therefore be conservative about adding tables.

Where a concept can naturally be encoded through:

```text
existing tables
TypeSpec signatures
custom attributes
additional flags
```

those mechanisms may provide better compatibility.

However, concepts such as first-class free functions or nominal-union cases may genuinely justify new tables.

The design should not contort the semantic model merely to ensure an old reader can fully understand a format it was never designed for.

The priority should be:

```text
1. preserve the metadata container architecture
2. preserve existing encodings where appropriate
3. prefer extensible existing mechanisms
4. add new tables only where the declaration model genuinely requires them
```

---

# 23. Metadata versioning

NeoCLR assemblies should identify their metadata dialect/version explicitly.

A reader should be able to determine:

```text
this is NeoCLR metadata
based on the CLI metadata architecture
using NeoCLR metadata version N
```

rather than discovering this only after encountering an unknown signature code.

This could be represented through the metadata version string, an assembly-level marker, or another format-level mechanism.

The exact encoding requires implementation investigation.

The important requirement is that NeoCLR extensions are **explicitly versioned**.

---

# 24. Compatibility modes

It may eventually be useful to distinguish metadata that uses only CLI-compatible constructs from metadata requiring NeoCLR extensions.

Conceptually:

```text
CLI-compatible subset

    TypeDef
    MethodDef
    ordinary signatures
    ordinary generics
    ...


NeoCLR extensions

    FunctionDef
    nominal union metadata
    union TypeSpec
    intersection TypeSpec
    nullable TypeSpec
    ...
```

A tooling API could then determine whether an assembly uses NeoCLR-specific metadata features.

This may also help future interop or conversion tools.

---

# 25. Minimal initial changes

A surprisingly small initial extension set may be enough to support the first NeoCLR metadata model.

### Existing structures retained

NeoCLR should retain the existing CLI concepts wherever they already model the desired semantics:

```text
Assembly
Module

TypeDef
TypeRef
TypeSpec

MethodDef
Param
Field
Property
Event

GenericParam
GenericParamConstraint

InterfaceImpl
MethodImpl

MemberRef
MethodSpec

CustomAttribute

string heap
blob heap
GUID heap
user-string heap
```

Existing signature encodings should likewise remain valid wherever NeoCLR does not require richer semantics.

### Initial extensions

The first NeoCLR-specific additions may be limited to:

```text
1. Function declarations outside TypeDef

2. Nominal-union case metadata

3. TypeSpec signature forms for:
       Union
       Intersection
       Nullable
       Function type

4. Generic constraints capable of referring to
   arbitrary TypeSpec expressions

5. Any flags required to identify new nominal
   type forms such as nominal unions
```

This would preserve most of the existing metadata machinery while extending the places where the CLI model is genuinely insufficient.

---

# 26. Free functions should reuse existing metadata machinery

Although a free function requires a new declaration relationship, as much of `MethodDef`'s existing machinery as possible should be reused.

A free function still has:

```text
RVA
implementation flags
declaration flags
name
signature
parameters
generic parameters
custom attributes
method body
```

The principal semantic difference is ownership:

```text
MethodDef
    owned by TypeDef

FunctionDef
    owned by module/namespace
```

NeoCLR should therefore investigate whether a new physical `FunctionDef` table is actually necessary or whether the existing `MethodDef` table can be generalized.

For example, the existing ownership relationship could potentially be extended so that some `MethodDef` rows are not contained in a `TypeDef`.

Conceptually:

```text
CallableDef
    ├── method
    │     owner → TypeDef
    │
    └── function
          owner → namespace/module
```

However, this should only be done if it can be expressed without making existing `MethodDef` interpretation ambiguous or incompatible.

A dedicated `FunctionDef` table may ultimately be cleaner if the CLI encoding fundamentally assumes that every `MethodDef` belongs to a type.

The desired semantic model should take precedence over avoiding one additional table.

---

# 27. Avoid synthetic containing types

NeoCLR should not expose free functions by pretending they are static methods on compiler-generated types.

For example:

```raven
namespace System.Text;

func Encode(...) -> ...
```

should not semantically become:

```text
TypeDef <System.Text.Functions>
    MethodDef Encode
```

or:

```text
TypeDef <Module>
    MethodDef Encode
```

even if such an encoding would make existing CLR tooling easier to reuse.

That representation would leak a compatibility workaround into:

```text
metadata
Introspection
language interoperability
member identity
reflection
API documentation
```

A NeoCLR compiler consuming the assembly should see a function, not reconstruct one from a synthetic class convention.

Compatibility projections may still generate such types when targeting the existing CLR, but they should not define NeoCLR's native metadata model.

---

# 28. Namespaces should initially remain lightweight

Free functions require namespace membership, but this does not necessarily require a `NamespaceDef` table.

The CLI's existing approach of representing namespaces primarily through strings can be extended conservatively.

Conceptually:

```text
FunctionDef
    Name       → "Encode"
    Namespace  → "System.Text"
```

and:

```text
TypeDef
    Name       → "Encoding"
    Namespace  → "System.Text"
```

can then naturally coexist:

```text
System.Text

    function Encode
    function Decode
    type Encoding
```

`System.Introspection.NamespaceInfo` may construct a namespace view over these declarations without requiring the namespace itself to possess a metadata token.

This is similar to how namespace structure can be reconstructed from type metadata today.

A first-class namespace table can be considered later if namespaces acquire metadata that cannot be represented cleanly through this model.

---

# 29. Namespace membership is declaration metadata

The namespace of a function should not be encoded into the function name.

For example:

```text
Name = "System.Text.Encode"
```

should be avoided.

Instead:

```text
Namespace = "System.Text"
Name      = "Encode"
```

preserves the distinction between:

```text
namespace identity
declaration name
```

and matches the existing conceptual model used by `TypeDef`.

This also makes Introspection straightforward:

```text
FunctionInfo
    Name       → Encode
    Namespace  → System.Text
```

rather than requiring parsing of qualified names.

---

# 30. TypeSpec as the compound-type mechanism

`TypeSpec` should become the principal metadata representation for non-nominal NeoCLR type expressions.

This is not a radical departure from CLI metadata.

The CLI already uses `TypeSpec` to represent types that cannot be identified merely by a `TypeDef` or `TypeRef` token.

NeoCLR extends the signature grammar understood by `TypeSpec`.

Conceptually:

```text
TypeDef / TypeRef
    nominal type identity

TypeSpec
    type expression
```

Examples:

```text
String
    TypeRef

List<T>
    TypeSpec / generic instantiation

String?
    TypeSpec / nullable

String | Int32
    TypeSpec / union

InputStream & OutputStream
    TypeSpec / intersection

(String) -> Int32
    TypeSpec / function
```

This makes `TypeSpec` substantially more important in NeoCLR, but does not fundamentally change its role.

---

# 31. Extended type-signature grammar

Conceptually, the NeoCLR type-signature grammar could become:

```text
Type :=
      PrimitiveType
    | TypeDefOrRef
    | GenericParameter
    | GenericInstantiation
    | Array(Type)
    | ManagedReference(Type)
    | Pointer(Type)
    | FunctionType(Signature)

    | Nullable(Type)
    | Union(Type...)
    | Intersection(Type...)
```

The exact binary encoding should follow the existing compressed-signature style where practical.

For example:

```text
UNION
    compressed constituent count
    type
    type
    ...
```

and:

```text
INTERSECTION
    compressed constituent count
    type
    type
    ...
```

This allows compound expressions to nest naturally:

```raven
A & (B | C)
```

as:

```text
INTERSECTION
    count 2

    TypeRef A

    UNION
        count 2
        TypeRef B
        TypeRef C
```

The metadata format therefore does not need a separate table structure for the type algebra.

---

# 32. Nullability should use the same type-expression model

Nullability fits naturally into this grammar:

```text
NULLABLE
    Type
```

For:

```raven
String?
```

the metadata encodes:

```text
NULLABLE
    TypeRef String
```

For:

```raven
(String | Error)?
```

it could encode:

```text
NULLABLE
    UNION
        String
        Error
```

assuming NeoCLR permits that type semantically.

The metadata format should be capable of representing the expression even if individual languages restrict which combinations they expose.

This is preferable to inventing a parallel nullable-annotation system if NeoCLR itself treats nullability as part of the type system.

---

# 33. Nullability should survive language boundaries

Encoding nullability directly into type metadata has an important interoperability consequence.

Suppose Raven emits:

```raven
func Find(name: String) -> User?
```

Another NeoCLR language reading that signature sees:

```text
Parameter:
    String

Return:
    Nullable(User)
```

It does not need to understand:

```text
Raven.NullableAttribute
C# NullableAttribute
compiler-specific nullable contexts
```

to reconstruct the contract.

The distinction:

```text
User
User?
```

belongs to NeoCLR metadata itself.

That makes nullability a platform feature rather than a language convention.

---

# 34. Nullable is not necessarily `Nullable<T>`

The metadata expression:

```text
Nullable(Type)
```

should not automatically imply a nominal generic type:

```text
Nullable<T>
```

Those are separate concepts unless NeoCLR explicitly decides otherwise.

A nullable reference:

```raven
String?
```

does not need to become:

```text
Nullable<String>
```

at the ABI level.

Likewise, nullable value-type representation may eventually be optimized differently.

The metadata describes the semantic type:

```text
Nullable(String)
```

The runtime ABI determines how values of that type are represented.

This follows the same separation used for native union and intersection types.

---

# 35. Generic constraints should accept TypeSpec

The existing CLI constraint model can remain largely intact if a generic parameter constraint can refer to a `TypeSpec`.

Conceptually:

```text
GenericParamConstraint
    Owner       → GenericParam
    Constraint  → TypeDefOrRefOrSpec
```

where the final target can include:

```text
TypeDef
TypeRef
TypeSpec
```

Then:

```raven
where T : InputStream & OutputStream
```

can be encoded as:

```text
GenericParamConstraint
    Owner → T

    Constraint →
        TypeSpec:
            INTERSECTION
                InputStream
                OutputStream
```

Likewise:

```raven
where T : Request | Event
```

uses:

```text
TypeSpec:
    UNION
        Request
        Event
```

The metadata format does not need to understand constraints as a separate expression language.

It already understands the relevant type expression.

---

# 36. Constraint semantics belong to the runtime type system

The metadata merely records:

```text
T constrained by TypeSpec X
```

NeoCLR's type system defines what satisfying `X` means.

For example:

```text
INTERSECTION(A, B)

    satisfied when:
        candidate satisfies A
        AND
        candidate satisfies B
```

while:

```text
UNION(A, B)

    satisfied when:
        candidate satisfies A
        OR
        candidate satisfies B
```

This keeps the metadata representation declarative.

No new:

```text
IntersectionConstraint
UnionConstraint
```

tables or attributes are necessary.

---

# 37. Nominal union metadata should remain declaration-oriented

Nominal unions are different because they introduce declarations rather than merely type expressions.

For:

```raven
union Result<T, E>
{
    Ok(T),
    Err(E)
}
```

the nominal type remains:

```text
TypeDef Result<T,E>
```

while its cases require declaration metadata.

The minimum extension might therefore be:

```text
UnionCase
---------
Flags
Name
Owner
Signature
```

where:

```text
Owner
    → TypeDef Result

Name
    → Ok

Signature
    → T
```

and:

```text
Owner
    → TypeDef Result

Name
    → Err

Signature
    → E
```

A no-payload case such as:

```raven
None
```

can use an empty case signature.

This keeps case identity separate from payload type.

---

# 38. Nominal union cases should have metadata identity

Cases should probably have their own metadata rows/tokens.

This allows Introspection and other metadata systems to refer directly to:

```text
Result.Ok
Result.Err
Option.Some
Option.None
```

without manufacturing identifiers from names.

It also leaves room for case-level:

```text
attributes
accessibility, if supported
documentation references
debug information
future metadata
```

A case is therefore a declaration, not merely a payload entry in a blob.

That fits naturally with the broader NeoCLR Introspection model.

---

# 39. Union case payload signatures

A union case should not necessarily be limited to one unnamed payload type.

The metadata representation should leave room for cases such as:

```raven
union Message
{
    Text(String),
    Position(Int32 x, Int32 y),
    Quit
}
```

Conceptually:

```text
UnionCase Text
    parameters:
        String

UnionCase Position
    parameters:
        Int32 x
        Int32 y

UnionCase Quit
    parameters:
        <none>
```

The case signature could therefore reuse parameter/signature machinery rather than encode only:

```text
PayloadType
```

Even if Raven initially favors simpler case shapes, the metadata should avoid an unnecessarily restrictive ABI.

---

# 40. Existing metadata readers and new TypeSpec codes

Extending the signature grammar creates an unavoidable compatibility boundary.

An existing CLI metadata reader may successfully locate a `TypeSpec` blob but fail when it encounters an unknown element-type code such as:

```text
UNION
INTERSECTION
NULLABLE
```

That is acceptable provided the reader can still parse the overall metadata structure safely.

The goal should not be:

> Every existing reader fully understands NeoCLR metadata.

That is impossible once NeoCLR introduces type-system constructs the CLI does not have.

The more realistic goal is:

> **Existing metadata infrastructure can recognize and traverse the familiar container and table model, while NeoCLR-aware readers understand the extended signature grammar.**

---

# 41. New tables are more invasive than new signatures

There is an important compatibility difference between:

```text
new TypeSpec signature element
```

and:

```text
new metadata table
```

An unknown signature element usually affects interpretation of a particular blob.

An unknown table can affect parsing of the entire tables stream because table layouts and row counts participate in index sizing and stream interpretation.

NeoCLR should therefore be especially conservative about adding new metadata tables.

This strengthens the case for using existing structures wherever possible for:

```text
compound types
nullability
generic constraints
function types
```

Free functions and nominal-union cases remain the main areas where genuinely new declaration metadata may be warranted.

---

# 42. Extension tables may need their own stream

If preserving compatibility with existing table readers is a high priority, NeoCLR should investigate whether new declaration tables belong in a separate NeoCLR metadata stream rather than extending the standard `#~` table set directly.

Conceptually:

```text
Metadata root
│
├── #~
│     standard CLI-compatible tables
│
├── #Strings
├── #Blob
├── #GUID
├── #US
│
└── #Neo
      NeoCLR extension tables
```

For example:

```text
#Neo

FunctionDef
UnionCase
future NeoCLR declarations
```

This could allow an older metadata reader to continue understanding the standard table stream while ignoring an unknown NeoCLR extension stream.

A NeoCLR-aware reader combines both views.

This needs investigation against actual CLI metadata-reader behavior before being adopted, but it may provide a useful compatibility mechanism.

---

# 43. Alternative: companion metadata over existing rows

Another compatibility strategy is to represent as much as possible through existing rows and place NeoCLR-specific semantics in companion metadata.

For example, a nominal union could theoretically remain an ordinary `TypeDef` with additional union-case information stored in a NeoCLR extension stream.

Likewise, if some representation of free functions can reuse an existing method row without introducing a fake type into the semantic model, ownership information could potentially live in extension metadata.

However, this should not create contradictory metadata where standard readers see one semantic structure and NeoCLR sees an entirely different one.

Compatibility metadata should remain a projection, not become the authoritative model.

---

# 44. Existing-reader compatibility levels

It may help to define compatibility explicitly.

### Level 1 — container readability

An existing reader can:

```text
open the PE/metadata image
find the metadata root
read standard heaps
read the standard tables it understands
```

### Level 2 — conventional metadata readability

It can correctly inspect conventional:

```text
TypeDef
TypeRef
MethodDef
Field
Property
Assembly
Module
```

records that do not depend on NeoCLR extensions.

### Level 3 — NeoCLR semantic understanding

Requires a NeoCLR-aware reader capable of interpreting:

```text
extended TypeSpec signatures
free functions
nominal unions
NeoCLR generic constraints
other runtime extensions
```

The initial compatibility goal should probably be **Levels 1 and 2**, not Level 3.

That gives the phrase "existing metadata readers should be able to read it" a concrete meaning.

---

# 45. Unknown extension data should be ignorable

Where possible, NeoCLR extensions should follow an important compatibility rule:

> **A reader that does not understand an extension should be able to ignore it without misinterpreting metadata it does understand.**

This is especially important for:

```text
custom metadata streams
optional extension records
debug information
future metadata additions
```

It is harder to guarantee for unknown type-signature elements because a reader must understand the signature grammar to parse the containing signature.

That limitation should be accepted rather than worked around with lossy encodings.

---

# 46. MethodDef and FunctionDef reconsidered

Before adding `FunctionDef`, NeoCLR should investigate the exact structural assumptions of `MethodDef`.

If the ownership of methods is derived primarily through `TypeDef.MethodList`, it may be possible to introduce module-level functions while retaining ordinary `MethodDef` rows, provided NeoCLR can identify which rows are not owned by a `TypeDef`.

Conceptually:

```text
MethodDef table
    rows 1..N

TypeDef.MethodList ranges
    claim method rows belonging to types

remaining designated rows
    module/namespace functions
```

However, relying on unowned `MethodDef` rows may violate existing CLI invariants and confuse existing readers.

Another possibility is to use the existing `<Module>` `TypeDef` physically while NeoCLR metadata marks particular methods as semantic free functions.

That would improve binary compatibility but introduces a projection layer:

```text
physical CLI view:
    <Module>.Encode

NeoCLR semantic view:
    System.Text.Encode
```

This is worth considering precisely because the project wants existing readers to remain useful.

It should not be rejected merely because `<Module>` is synthetic—the important question is whether the synthetic representation leaks into NeoCLR's semantic metadata model.

---

# 47. Compatibility projection for free functions

A particularly pragmatic design could therefore use two layers.

Physically, a free function may still occupy a `MethodDef` associated with `<Module>` so existing metadata readers can enumerate it.

NeoCLR-specific metadata then supplies its semantic declaration information:

```text
FreeFunction
    MethodDef   → token
    Namespace   → "System.Text"
    Name        → "Encode"
```

NeoCLR Introspection exposes:

```text
FunctionInfo
    Namespace = System.Text
    Name = Encode
```

rather than:

```text
MethodInfo
    DeclaringType = <Module>
```

This would preserve much more of the CLI physical format while still giving NeoCLR first-class free-function semantics.

Whether this is preferable to a real `FunctionDef` table depends on how cleanly the projection can be implemented.

This is one of the places where NeoCLR should be willing to distinguish between:

```text
physical metadata representation
```

and:

```text
NeoCLR semantic representation
```

if doing so provides substantial compatibility benefits without contaminating the higher-level platform model.

The important requirement is that this distinction remains well-defined and invisible to ordinary NeoCLR consumers.

---

# 48. Physical encoding versus semantic metadata

NeoCLR does not necessarily require every semantic concept to receive a completely new physical representation.

A metadata reader can expose a richer semantic model over a conservative CLI-compatible encoding.

For example:

```text
Physical metadata

<Module>
    MethodDef Encode
        + NeoCLR free-function metadata


NeoCLR semantic model

System.Text
    FunctionInfo Encode
```

The physical `<Module>` ownership exists only because the underlying CLI table format requires a `MethodDef` to belong to a type.

It does not imply that NeoCLR considers the function to be a method.

Likewise, `System.Introspection` should expose:

```text
FunctionInfo
```

rather than:

```text
MethodInfo
```

for such a declaration.

This gives NeoCLR freedom to preserve compatibility-oriented encodings without allowing those encodings to define the platform semantics.

---

# 49. Prefer compatible physical encodings where they are lossless

A compatibility encoding is attractive when all of the following are true:

```text
1. Existing metadata infrastructure can parse it.

2. NeoCLR can recover the complete intended semantic model.

3. No meaningful NeoCLR information is lost.

4. The compatibility representation does not become
   observable as the NeoCLR semantic model.

5. The representation does not prevent future evolution.
```

Free functions may satisfy these requirements using `<Module>` plus NeoCLR metadata.

Other features may not.

For example, attempting to encode:

```raven
A | B
```

as:

```text
Union<A,B>
```

would change the actual type identity and reintroduce a nominal backing type that NeoCLR does not otherwise need.

That is not merely a physical encoding detail.

Therefore native union and intersection type expressions should remain actual `TypeSpec` extensions.

The compatibility strategy can consequently vary by feature.

---

# 50. Metadata extensions should preserve semantics, not appearances

The objective should not be to make a NeoCLR assembly appear to be a perfectly ordinary CLR assembly at all costs.

That would eventually force NeoCLR concepts into CLR abstractions that do not represent them correctly.

Instead:

> **Use existing CLI representations where they can faithfully carry NeoCLR semantics, and extend the format where they cannot.**

For example:

```text
Concept                         Strategy

nominal class                   existing TypeDef

nominal struct                  existing TypeDef

interface                       existing TypeDef

enum                            existing TypeDef

method                          existing MethodDef

free function                   MethodDef + extension metadata
                                OR FunctionDef if necessary

generic parameter               existing GenericParam

nominal type reference          existing TypeDef/TypeRef

generic instantiation           existing TypeSpec encoding

ad-hoc union                    extended TypeSpec

intersection                    extended TypeSpec

nullable type                   extended TypeSpec

function type                   extended TypeSpec

nominal union                   TypeDef + union metadata

nominal union case              extension metadata/table
```

This should be the default decision framework.

---

# 51. Extension metadata for free functions

If NeoCLR uses ordinary `MethodDef` rows as the physical carrier for free functions, a NeoCLR extension record could identify their semantic role.

Conceptually:

```text
Function
--------
Method       MethodDef
Namespace    String
Flags        ...
```

The `MethodDef` retains:

```text
RVA
implementation flags
name
signature
parameters
generic parameters
custom attributes
```

while the extension record says:

> This callable is a free function rather than a method of the physical `<Module>` carrier.

For example:

```raven
namespace System.Text;

func Encode(value: String) -> Bytes
```

might physically appear as:

```text
TypeDef <Module>
    MethodDef Encode(String) -> Bytes

NeoCLR Function metadata:
    Method       → Encode MethodDef
    Namespace    → System.Text
```

and semantically as:

```text
FunctionInfo
    Name         → Encode
    Namespace    → System.Text
    Parameters   → String
    ReturnType   → Bytes
```

This avoids duplicating the substantial metadata machinery already associated with `MethodDef`.

---

# 52. Generic free functions

The compatibility representation should also work for generic functions.

For example:

```raven
func Identity<T>(value: T) -> T
{
    return value;
}
```

can continue to use:

```text
MethodDef
GenericParam
Param
signature blob
method body
```

exactly as a generic static method would physically.

NeoCLR metadata merely changes its semantic ownership:

```text
physical:
    <Module>.Identity<T>

semantic:
    FunctionInfo Identity<T>
```

Generic constraints can likewise continue to use:

```text
GenericParamConstraint
```

including NeoCLR `TypeSpec` extensions where richer constraints are required.

This is a strong argument for reusing `MethodDef` rather than duplicating it as `FunctionDef`.

---

# 53. Function references

NeoCLR must also represent references to free functions defined in other modules or assemblies.

If free functions physically reuse method metadata, the existing:

```text
MemberRef
MethodSpec
```

machinery may potentially remain useful.

However, the semantic distinction between:

```text
method reference
function reference
```

must remain observable to NeoCLR.

For example, an external reference to:

```raven
System.Text.Encode
```

should not semantically require a declaring type.

A NeoCLR extension could associate a `MemberRef` with a namespace-level function identity.

Alternatively, this may be one of the points where a dedicated:

```text
FunctionRef
```

extension becomes justified.

This requires investigation against the existing `MemberRefParent` encoding, which assumes a CLI-style member owner.

The design should prefer reuse where it remains coherent but should not force free-function identity through a fake nominal type merely to preserve `MemberRef`.

---

# 54. Function identity

A free function's metadata identity should conceptually include:

```text
module / assembly
namespace
name
signature
generic arity
```

rather than:

```text
declaring type
name
signature
```

as for a method.

For example:

```text
System.Text.Encode(String)
```

and:

```text
Other.Text.Encode(String)
```

are distinct declarations because they belong to different namespaces.

Overloads within a namespace can be distinguished through their signatures in the same way methods are.

This gives NeoCLR a natural namespace-level callable model without introducing synthetic owner types into the semantic identity.

---

# 55. Nominal union compatibility representation

Nominal unions present a somewhat different problem.

A nominal union is already a `TypeDef`, so existing metadata readers can recognize it as a type even if they do not understand its union semantics.

For example:

```raven
union Option<T>
{
    Some(T),
    None
}
```

can physically remain:

```text
TypeDef Option<T>
```

with NeoCLR-specific metadata describing:

```text
this TypeDef is a nominal union

cases:
    Some(T)
    None
```

An existing reader can still report:

```text
Option<T>
```

as a type.

It simply does not understand the richer union declaration.

This is an excellent example of graceful metadata extension.

---

# 56. Identifying nominal union types

NeoCLR needs a way to identify a `TypeDef` as a nominal union.

Several approaches are possible:

```text
new TypeAttributes flag

well-known custom attribute

NeoCLR extension record

dedicated nominal-type-kind metadata
```

A new standard `TypeDef` flag is compact but modifies an existing bit field and depends on unused/reserved space.

A custom attribute is highly compatible but may make a fundamental type-system distinction look too optional or library-defined.

An extension record provides explicit NeoCLR semantics without consuming existing CLI flags.

The exact representation should be selected after examining the available CLI encoding space.

Semantically, however, `System.Introspection` should simply expose:

```text
NominalUnionTypeInfo
```

regardless of the physical encoding used to identify it.

---

# 57. Union cases as extension declarations

Nominal union cases are new declaration forms and therefore likely require genuine NeoCLR metadata.

If compatibility is important, they are good candidates for the NeoCLR extension stream rather than new standard `#~` tables.

Conceptually:

```text
#Neo

UnionCase
---------
Owner       TypeDef
Flags
Name
Signature
```

For:

```raven
union Result<T,E>
{
    Ok(T),
    Err(E)
}
```

the records are:

```text
UnionCase
    Owner      → Result
    Name       → Ok
    Signature  → (T)

UnionCase
    Owner      → Result
    Name       → Err
    Signature  → (E)
```

Existing readers see `Result<T,E>` as an ordinary type.

NeoCLR-aware readers additionally discover that it is a nominal union with two cases.

---

# 58. Extension records should reference standard metadata

Where possible, NeoCLR extension metadata should reference ordinary CLI metadata rather than duplicate it.

For example:

```text
Function extension
    → references MethodDef

NominalUnion extension
    → references TypeDef

UnionCase
    → references TypeDef owner
    → uses existing String/Blob heaps

future extension
    → references existing tokens where appropriate
```

This gives NeoCLR metadata a layered architecture:

```text
standard CLI metadata
        │
        │ referenced by
        ▼
NeoCLR extension metadata
```

rather than two independent metadata universes.

It also means existing metadata infrastructure remains useful for token resolution, names, signatures, attributes and implementation bodies.

---

# 59. NeoCLR extension stream

A dedicated extension stream is therefore worth serious consideration.

Conceptually:

```text
Metadata Root
│
├── #~
│     CLI-compatible tables
│
├── #Strings
├── #Blob
├── #GUID
├── #US
│
└── #Neo
      NeoCLR-specific semantic extensions
```

`#Neo` could contain a versioned table directory using the same general principles as CLI metadata:

```text
table id
row count
row schema
coded/token references
heap references
```

Initial tables might include:

```text
Function
NominalUnion
UnionCase
```

although `NominalUnion` may be unnecessary if union identity can be represented efficiently elsewhere.

The stream should be designed so that readers unaware of `#Neo` can ignore it.

---

# 60. Do not duplicate TypeSpec in the extension stream

Compound type expressions are different.

Types such as:

```raven
A | B
A & B
String?
```

occur directly inside ordinary signatures.

Putting their definitions only into `#Neo` would make ordinary signature interpretation dependent on an external semantic overlay.

Instead, these should extend the existing type-signature grammar directly.

So the boundary becomes:

```text
#~ / normal signature blobs
    type expressions

#Neo
    additional declaration relationships
    and semantics that cannot fit naturally
    into existing tables
```

That seems like a useful architectural rule.

---

# 61. TypeSpec encoding and existing readers

A NeoCLR-aware signature parser understands additional element codes:

```text
NEO_UNION
NEO_INTERSECTION
NEO_NULLABLE
NEO_FUNCTION
```

The exact naming and numeric range remain to be defined.

An old reader encountering one of these may be unable to decode that particular signature.

But this failure remains local to the unsupported signature rather than requiring it to understand an entirely different metadata object model.

NeoCLR tooling should ideally surface this as:

```text
unsupported NeoCLR type signature
```

rather than treating the entire image as malformed.

Whether existing third-party readers behave this gracefully will vary and should be tested experimentally.

---

# 62. Reserving a NeoCLR signature extension prefix

Rather than consuming several unrelated CLI element-type values, NeoCLR could investigate a single extension marker.

Conceptually:

```text
ELEMENT_TYPE_NEO
    subtype
    payload
```

where:

```text
subtype 1 → Union
subtype 2 → Intersection
subtype 3 → Nullable
subtype 4 → Function
...
```

This would give NeoCLR its own extensible namespace inside signature blobs.

For example:

```text
ELEMENT_TYPE_NEO
    UNION
    count = 2
    String
    Int32
```

This may be easier to evolve than claiming a new top-level CLI element code for every future NeoCLR type form.

However, compatibility with existing signature parsers and the available element-type encoding space must be investigated before selecting this approach.

---

# 63. Metadata tokens remain valuable

NeoCLR should preserve the token model.

A token remains a compact identifier for a metadata declaration such as:

```text
TypeDef
MethodDef
Field
Property
Event
```

and potentially NeoCLR extension declarations where token identity is useful.

Non-nominal type expressions generally do not need independent declaration tokens.

They can be referenced through `TypeSpec` tokens when a tokenized reference is required.

Thus:

```text
Result<T,E>
    TypeDef identity

Result.Ok
    NeoCLR UnionCase identity

String | Int32
    TypeSpec identity when required

InputStream & OutputStream
    TypeSpec identity when required
```

This preserves the useful distinction between declarations and type expressions.

---

# 64. Introspection should hide physical compatibility carriers

`System.Introspection` should expose the NeoCLR semantic model rather than the compatibility encoding.

For example, if a free function physically uses:

```text
<Module>.Encode MethodDef
```

plus extension metadata, Introspection exposes:

```text
FunctionInfo Encode
```

It should not normally expose a synthetic:

```text
MethodInfo
DeclaringType = <Module>
```

for the same declaration.

Likewise, if some future compatibility representation requires synthetic metadata, that representation should not leak through the normal semantic Introspection APIs.

A lower-level metadata API may expose the raw rows where necessary.

This gives NeoCLR two useful layers:

```text
raw metadata
    physical encoding

System.Introspection
    semantic metadata model
```

---

# 65. Raw metadata access

Because NeoCLR intentionally retains a CLI-like physical format, tooling may still need access to raw metadata.

A low-level metadata reader can expose:

```text
tables
rows
tokens
heaps
signature blobs
NeoCLR extension records
```

while `System.Introspection` provides higher-level semantic objects.

This separation is valuable for:

```text
compilers
linkers
disassemblers
metadata diagnostics
binary compatibility tools
runtime implementation
```

without forcing application-level introspection consumers to understand compatibility carriers.

---

# 66. Proposed metadata layers

The resulting architecture can be viewed as three layers:

```text
┌─────────────────────────────────────────────┐
│ System.Introspection                        │
│                                             │
│ NominalTypeInfo                             │
│ FunctionInfo                                │
│ MethodInfo                                  │
│ TypeUnionInfo                               │
│ TypeIntersectionInfo                        │
│ NullableTypeInfo                            │
└──────────────────────┬──────────────────────┘
                       │ semantic projection
                       ▼
┌─────────────────────────────────────────────┐
│ NeoCLR metadata semantics                   │
│                                             │
│ free functions                              │
│ nominal unions / cases                      │
│ native compound types                       │
│ nullable types                              │
│ richer constraints                          │
└──────────────────────┬──────────────────────┘
                       │ encoded using
                       ▼
┌─────────────────────────────────────────────┐
│ CLI-derived physical metadata               │
│                                             │
│ #~ tables                                   │
│ TypeDef / TypeRef / TypeSpec                │
│ MethodDef                                   │
│ GenericParam                                │
│ signature blobs                             │
│ heaps                                       │
│ + NeoCLR extension stream                   │
└─────────────────────────────────────────────┘
```

This allows NeoCLR to evolve semantically without unnecessarily discarding the mature CLI metadata architecture.

---

# 67. Proposed direction

NeoCLR should use the existing CLI metadata format as its structural foundation and make **targeted extensions rather than designing an unrelated metadata format**.

The initial direction should be:

1. **Keep `TypeDef` for all nominal types.** Classes, structs, interfaces, enums and nominal unions remain named type declarations.

2. **Keep `TypeRef` for references to nominal types.**

3. **Use `TypeSpec` for non-nominal and constructed type expressions.** Extend its signature grammar for native unions, intersections, nullability and function types.

4. **Keep `MethodDef` for methods.**

5. **Investigate reusing `MethodDef` as the physical carrier for free functions**, potentially through `<Module>`, while NeoCLR extension metadata supplies namespace-level function semantics.

6. **Do not expose the physical `<Module>` carrier through normal Introspection.** Free functions are `FunctionInfo`, not synthetic `MethodInfo`.

7. **Keep existing parameter, generic parameter, custom attribute and method-body machinery** for free functions wherever possible.

8. **Extend generic constraints so they can refer to arbitrary `TypeSpec` expressions.** This allows unions and intersections to participate directly without dedicated constraint encodings.

9. **Represent nominal unions through `TypeDef` plus NeoCLR union metadata.** Their cases remain declarations with nominal case identity.

10. **Give union cases stable metadata identity**, preferably through NeoCLR extension metadata referencing their owning `TypeDef`.

11. **Keep nullability in the type signature itself** if NeoCLR treats it as a genuine type-system distinction.

12. **Consider a dedicated NeoCLR metadata extension stream** for new declaration relationships rather than modifying the standard table stream unnecessarily.

13. **Preserve standard heaps, tokens, signatures and table references wherever possible.**

14. **Allow existing readers to ignore NeoCLR extension streams where possible.**

15. **Accept that existing readers cannot semantically decode NeoCLR-specific type signatures.** Compatibility means structural readability, not pretending NeoCLR's type system is identical to the CLI.

The guiding rule is:

> **Preserve the CLI encoding when it can faithfully carry NeoCLR semantics. Extend it when the semantic model genuinely exceeds what the CLI can express.**

That gives NeoCLR something recognizably CLR-like at the binary level without forcing its type system or Introspection model to inherit the CLR's historical assumptions.