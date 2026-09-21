I think the proposal should now deliberately leave the earlier nominal `Union<T...>` / `Intersection<T...>` experiment behind. The core proposal becomes: **NeoCLR itself understands ad-hoc union and intersection type expressions**, while nominal unions remain a separate named-type facility that can share underlying union semantics.

# NeoCLR Runtime Union and Intersection Types — Proposal

## 1. Summary

NeoCLR should support **union and intersection type expressions directly in the runtime type system and metadata model**.

Conceptually:

```text
A | B       union type
A & B       intersection type
```

Their fundamental relationships are:

```text
A | B       A OR B
A & B       A AND B
```

These are type-system constructs rather than nominal library types.

NeoCLR therefore does **not** require generic backing types such as:

```text
Union<A, B>
Intersection<A, B>
```

for ad-hoc unions and intersections.

A high-level language such as Raven may project the runtime types using native syntax:

```raven
String | Int32
InputStream & OutputStream
```

Other NeoCLR languages may expose different syntax while consuming the same runtime type information.

NeoCLR should additionally allow these compound types to participate directly in generic constraints.

Nominal unions such as:

```raven
Option<T>
Result<T, E>
```

remain named types with named cases and their own nominal identity. They are not replaced by ad-hoc unions.

However, nominal unions and ad-hoc union types should share the underlying runtime understanding of union semantics where appropriate.

---

# 2. Goals

The design should provide a common NeoCLR model for:

```text
ad-hoc union types
ad-hoc intersection types
generic constraints involving either
runtime type checking
compiler assignability
metadata and Introspection
high-level language projection
nominal union interoperability
```

The design should avoid requiring compilers to know about artificial standard-library ABI types merely to express compound type relationships.

In particular, Raven should not have to lower:

```raven
A | B
```

to:

```raven
Union<A, B>
```

or:

```raven
A & B
```

to:

```raven
Intersection<A, B>
```

when targeting NeoCLR.

The type relationship itself belongs to the runtime.

---

# 3. Three distinct concepts

NeoCLR should distinguish three related but different concepts.

### Ad-hoc union type

```raven
A | B
```

A value satisfies one of the constituent types.

This type has no independent nominal identity.

### Ad-hoc intersection type

```raven
A & B
```

A value satisfies all constituent types simultaneously.

This type has no independent nominal identity.

### Nominal union

```raven
Result<T, E>
Option<T>
```

A named type declaring named union cases.

For example:

```text
Result<T,E>
    Ok(T)
    Err(E)

Option<T>
    Some(T)
    None
```

Nominal unions have their own identity, API, cases and ABI.

The distinction is therefore:

```text
Ad-hoc union
    A | B
    anonymous combination of type alternatives

Intersection
    A & B
    anonymous combination of simultaneous type requirements

Nominal union
    Result<T,E>
    named union with declared cases
```

---

# 4. Runtime type expressions

NeoCLR metadata should be capable of describing compound type expressions directly.

Conceptually, the metadata type model gains:

```text
Type expression
├── named type
├── generic instantiation
├── array
├── managed reference
├── ...
├── union
│   └── constituent types
└── intersection
    └── constituent types
```

For example:

```raven
String | Int32
```

is represented conceptually as:

```text
Union
├── System.String
└── System.Int32
```

and:

```raven
InputStream & OutputStream
```

as:

```text
Intersection
├── InputStream
└── OutputStream
```

These are metadata type expressions, not references to nominal generic definitions.

---

# 5. System.Introspection

NeoCLR uses `System.Introspection.TypeInfo` as its metadata type model rather than a `.NET`-style `System.Type`.

`TypeInfo` should therefore be capable of describing union and intersection type expressions directly.

The exact API remains to be designed, but conceptually:

```text
TypeInfo
    kind: Union

    constituent types:
        String
        Int32
```

or:

```text
TypeInfo
    kind: Intersection

    constituent types:
        InputStream
        OutputStream
```

A compiler, metadata reader, IDE or other tool can therefore determine that a type expression is a union or intersection without recognizing a particular library type.

This is an important difference from a nominal encoding.

There is no need to inspect:

```text
Union<A,B>
```

and infer from its name or attributes that it represents a union.

The metadata already says so.

---

# 6. Union semantics

A union type expresses an **alternative relationship**.

Given:

```text
A | B
```

a candidate type `T` satisfies the union when:

```text
Satisfies(T, A | B)

    =

Satisfies(T, A)
OR
Satisfies(T, B)
```

For example:

```raven
String | Int32
```

accepts:

```text
String     ✓
Int32      ✓
Boolean    ✗
```

The same rule extends naturally to additional alternatives:

```raven
A | B | C
```

meaning:

```text
A OR B OR C
```

---

# 7. Intersection semantics

An intersection expresses a **simultaneous relationship**.

Given:

```text
A & B
```

a candidate type `T` satisfies the intersection when:

```text
Satisfies(T, A & B)

    =

Satisfies(T, A)
AND
Satisfies(T, B)
```

For example:

```raven
InputStream & OutputStream
```

can be satisfied by:

```raven
class MemoryStream :
    InputStream,
    OutputStream,
    Seekable
{
    ...
}
```

because:

```text
MemoryStream satisfies InputStream      ✓
MemoryStream satisfies OutputStream     ✓
```

and therefore:

```text
MemoryStream satisfies
InputStream & OutputStream               ✓
```

The individual relationships remain nominal.

A type does not satisfy `InputStream` merely because it happens to contain methods with compatible signatures.

The intersection composes existing type relationships; it does not introduce general structural typing.

---

# 8. Assignability

Union and intersection types become part of NeoCLR's normal assignability rules.

For unions:

```text
A <: A | B
B <: A | B
```

where `<:` denotes assignability/type satisfaction.

For intersections:

```text
A & B <: A
A & B <: B
```

and any type satisfying both constituent relationships is assignable to the intersection:

```text
T <: A
T <: B

therefore:

T <: A & B
```

These rules belong to NeoCLR itself.

A language compiler should implement the same relationships statically that the runtime and verifier enforce.

---

# 9. Generic constraints

Compound types should be valid directly in generic constraint position.

For example:

```raven
func Process<T>(value: T)
    where T : InputStream & OutputStream
{
    ...
}
```

does not require special intersection constraint metadata separate from the type expression.

The constraint is simply:

```text
Intersection
├── InputStream
└── OutputStream
```

and normal constraint satisfaction evaluates it recursively:

```text
Satisfies(T, InputStream)
AND
Satisfies(T, OutputStream)
```

Likewise:

```raven
func Convert<T>(value: T)
    where T : String | Bytes
{
    ...
}
```

uses the union expression itself:

```text
Satisfies(T, String)
OR
Satisfies(T, Bytes)
```

The type model and constraint model therefore share the same compound relationships.

---

# 10. Constraint expressions

Native union and intersection support allows more expressive constraints without introducing a second constraint language.

For example:

```raven
where T : A & (B | C)
```

means:

```text
A
AND
(
    B
    OR
    C
)
```

Conceptually:

```text
Intersection
├── A
└── Union
    ├── B
    └── C
```

The runtime constraint checker can recursively evaluate the same type expression represented in metadata.

This is more general than the traditional CLR model where multiple constraints are implicitly conjunctive.

NeoCLR should therefore define generic constraint satisfaction in terms of **type expressions**, rather than assuming that a generic parameter merely carries a flat list of nominal requirements.

---

# 11. Constraints remain type relationships

Native compound constraints should not turn constraint checking into executable logic.

There is no equivalent of:

```raven
func Check(type: TypeInfo) -> bool
```

attached to a constraint.

Instead, the runtime evaluates the metadata type expression according to defined type-system rules.

For example:

```text
Named(A)
    → normal NeoCLR type satisfaction

Union(A,B)
    → satisfy A OR B

Intersection(A,B)
    → satisfy A AND B
```

Constraint evaluation remains:

```text
deterministic
metadata-driven
verifiable
language-independent
```

---

# 12. High-level language projection

Raven can map its existing type syntax directly to NeoCLR metadata.

For example:

```raven
func Parse(value: String | Bytes)
```

emits a parameter whose type is the NeoCLR union expression:

```text
String | Bytes
```

There is no intermediate:

```raven
Union<String, Bytes>
```

Likewise:

```raven
func Rewrite(
    stream: InputStream & OutputStream
)
```

emits the intersection type directly.

The relationship becomes:

```text
Raven                         NeoCLR metadata

A | B            ───────►     Union(A, B)

A & B            ───────►     Intersection(A, B)
```

Raven syntax is a language projection of the NeoCLR type model rather than a projection over standard-library carrier types.

---

# 13. Compiler member resolution for intersections

A high-level language can expose the combined member surface of an intersection.

Given:

```raven
let stream: InputStream & OutputStream = ...
```

Raven knows that the static type provides both contracts:

```text
InputStream
    Read(...)

OutputStream
    Write(...)
    Flush(...)
```

so completion can expose:

```text
Read(...)
Write(...)
Flush(...)
```

For:

```raven
stream.Read(buffer)
```

member resolution determines that the selected member belongs to `InputStream`.

For:

```raven
stream.Write(buffer)
```

it resolves against `OutputStream`.

The compiler then emits ordinary interface dispatch against the underlying value.

The intersection type itself provides the static guarantee that those interface projections are valid.

No nominal `Intersection<A,B>` ABI needs to be understood by the compiler.

---

# 14. Intersection member conflicts

Constituent types can expose conflicting members.

For example:

```raven
interface A
{
    Value: String;
}

interface B
{
    Value: Int32;
}
```

For:

```raven
let value: A & B;
```

the expression:

```raven
value.Value
```

is ambiguous.

The runtime type model only establishes that both contracts are present.

How conflicting members are selected is a language-level member-resolution question.

Raven may eventually provide explicit qualification syntax.

This does not affect the fundamental NeoCLR intersection semantics.

---

# 15. Runtime representation is separate from type representation

Making unions and intersections native type expressions does not require a sophisticated physical representation immediately.

NeoCLR should distinguish:

```text
logical type representation
```

from:

```text
runtime value representation
```

The metadata can understand:

```text
A | B
A & B
```

intrinsically while the runtime initially uses a simple CLR-like representation internally.

This representation is an implementation detail rather than a public nominal backing type.

There is therefore no requirement for:

```text
Union<A,B>
Intersection<A,B>
```

to exist merely to carry values.

---

# 16. Runtime representation of ad-hoc unions

An ad-hoc union needs to retain one active value.

An initial implementation may conceptually use:

```text
Union value
┌──────────────────────┐
│ active value         │
└──────────────────────┘
```

where the active value may use an object/reference representation when appropriate.

The runtime already knows the declared alternatives from the union type expression:

```text
String | Int32
```

so the value representation does not need to encode the union's type definition through a generic nominal wrapper.

Whether an explicit discriminator is required depends on the underlying value representation and should be treated as an ABI/runtime-layout question.

---

# 17. Runtime representation of intersections

An intersection contains one value satisfying all constituent contracts.

For:

```raven
InputStream & OutputStream
```

the runtime does not need separate stored values for each constituent.

Conceptually:

```text
Intersection value
┌──────────────────────┐
│ underlying value     │────► MemoryStream
└──────────────────────┘

static type guarantees:
    InputStream
    OutputStream
```

The runtime/compiler can project the value to the appropriate constituent interface when dispatching a member.

Again, no nominal `Intersection<T...>` carrier is required.

---

# 18. Runtime type testing

Because unions and intersections are native type expressions, ordinary NeoCLR type-test operations can understand them directly.

For example:

```raven
let value: String | Int32 = "hello";

if value is String
{
    ...
}
```

does not need a special wrapper-projection attribute.

NeoCLR knows that `value` has union semantics and can test its active value against `String`.

Likewise, testing against a compound target can use normal type satisfaction:

```raven
value is A | B
```

means:

```text
value satisfies A
OR
value satisfies B
```

and:

```raven
value is A & B
```

means:

```text
value satisfies A
AND
value satisfies B
```

This may use ordinary type-test IL with compound type metadata rather than requiring dedicated instructions such as:

```text
union.is
intersection.is
switch.type
```

Native type-system support does not imply dedicated IL for every compound operation.

---

# 19. Pattern matching

High-level languages can build pattern matching over the runtime type model.

For an ad-hoc union:

```raven
match value
{
    String text => ...
    Int32 number => ...
}
```

Raven can emit ordinary type tests and branches.

The runtime understands the union's active value without the compiler knowing about a nominal union-wrapper ABI.

For nominal unions:

```raven
match result
{
    Ok(value) => ...
    Err(error) => ...
}
```

the matching operation concerns **nominal cases**, rather than merely constituent runtime types.

These two forms share union semantics but are not identical.

---

# 20. Nominal unions remain first-class named types

Native ad-hoc union support does not replace nominal unions.

NeoCLR continues to support declarations such as:

```raven
union Result<T, E>
{
    Ok(T),
    Err(E)
}
```

and:

```raven
union Option<T>
{
    Some(T),
    None
}
```

These have properties that an ad-hoc union does not:

```text
nominal identity
declared name
named cases
case-specific payloads
members and methods
generic declaration
stable public ABI
```

Therefore:

```raven
Result<T, E>
```

is not merely another spelling of:

```raven
T | E
```

and:

```raven
Option<T>
```

is not merely:

```raven
T | ()
```

The nominal case identity carries semantic information that the corresponding ad-hoc type alternatives do not.

---

# 21. Shared union model

Although nominal and ad-hoc unions remain distinct kinds of type, NeoCLR should consolidate their common semantics.

Conceptually:

```text
                         union semantics
                               │
                 ┌─────────────┴─────────────┐
                 │                           │
          ad-hoc union                 nominal union
                 │                           │
               A | B                    Result<T,E>
                                        Option<T>
```

Both describe a finite set of alternatives.

The difference is how those alternatives are identified.

For an ad-hoc union:

```text
String | Int32

alternatives:
    type String
    type Int32
```

For a nominal union:

```text
Result<T,E>

cases:
    Ok(T)
    Err(E)
```

The first alternatives are identified by **type**.

The second alternatives are identified by **nominal case identity**.

NeoCLR should share runtime machinery where this distinction does not matter while preserving it where it does.

---

# 22. Nominal union metadata

A nominal union should explicitly describe its cases in metadata.

Conceptually:

```text
NominalUnionInfo
    Name: Result<T,E>

    Cases:
        Ok
            Payload: T

        Err
            Payload: E
```

The exact `System.Introspection` API is outside this proposal, but `TypeInfo` should make it possible to determine that `Result<T,E>` is a nominal union and inspect its declared cases.

This is different from an ad-hoc union:

```text
UnionTypeInfo
    Constituents:
        T
        E
```

The distinction remains visible through Introspection.

---

# 23. Mapping nominal unions into runtime union semantics

A nominal union may internally use the same runtime machinery that supports ad-hoc unions.

For example:

```text
Result<T,E>
    cases:
        Ok(T)
        Err(E)
```

can participate in the runtime's general notion of:

```text
one active alternative from a finite set
```

However, the runtime must retain the nominal case discriminator.

This is necessary even when two cases have the same payload type.

For example:

```raven
union State
{
    Previous(String),
    Current(String)
}
```

cannot be represented semantically as:

```raven
String | String
```

because the case identity distinguishes `Previous` from `Current`.

Therefore:

> **Ad-hoc unions discriminate by constituent type; nominal unions discriminate by declared case.**

They may share storage and matching machinery, but they are not interchangeable type forms.

---

# 24. Conversion between nominal and ad-hoc unions

NeoCLR should not automatically equate a nominal union with an ad-hoc union merely because their payload types happen to correspond.

For example:

```raven
Result<T,E>
```

should not automatically be considered identical to:

```raven
T | E
```

The nominal union carries additional semantic information:

```text
Result<T,E>
    Ok(T)
    Err(E)
```

whereas:

```text
T | E
```

only expresses:

```text
T OR E
```

There may nevertheless be useful **explicit projections or conversions** between nominal and ad-hoc unions.

For example, a language could permit an explicit operation that discards nominal case identity and exposes the payload as an ad-hoc union:

```text
Result<T,E>
      │
      │ explicit projection
      ▼
    T | E
```

Whether Raven should provide such a conversion is a language-design question.

NeoCLR should preserve enough metadata for languages to implement it without treating the two type forms as identical.

---

# 25. Nominal unions in generic constraints

Nominal unions and ad-hoc union constraints should also remain distinct.

Given:

```raven
where T : A | B
```

the constraint means:

```text
T satisfies A
OR
T satisfies B
```

But:

```raven
where T : Result<A, B>
```

normally means that `T` satisfies the nominal `Result<A,B>` type relationship.

The runtime should not automatically reinterpret every nominal union appearing in constraint position as the union of its case payload types.

Otherwise:

```raven
Result<A, B>
```

would lose its nominal meaning merely because it appeared as a constraint.

The general rule should therefore be:

> **Ad-hoc union expressions provide disjunctive type constraints. Nominal unions remain nominal constraints unless explicitly projected into another type relationship.**

This preserves the distinction between:

```text
A | B
```

and:

```text
Result<A,B>
```

throughout the type system.

---

# 26. Nominal union assignability

A nominal union follows ordinary nominal assignability rules.

For example:

```raven
Result<String, Error>
```

is assignable according to the declared relationships of `Result`, not merely according to its case payload types.

An `Ok<String>` value belongs to `Result<String,Error>` because `Ok` is a declared case of that nominal union.

By contrast, an ordinary `String` does not automatically become:

```raven
Result<String,Error>
```

simply because one of the union's cases carries a `String`.

This differs deliberately from an ad-hoc union:

```raven
String | Error
```

where:

```text
String <: String | Error
Error  <: String | Error
```

follows directly from the compound type relationship.

---

# 27. Shared runtime representation does not imply shared identity

NeoCLR may discover that nominal and ad-hoc unions can use very similar physical representations.

For example, both may conceptually require:

```text
active alternative
payload
```

or some optimized equivalent.

That does not imply that they have the same type semantics.

Conceptually:

```text
                    runtime storage machinery
                             │
                 ┌───────────┴───────────┐
                 │                       │
              A | B                 Result<A,B>
                 │                       │
          discriminate by          discriminate by
          constituent type         nominal case
```

The runtime may therefore share implementation machinery while metadata preserves the distinction.

This is similar to other CLR-like implementation choices where multiple type constructs can share layout or dispatch mechanisms without becoming the same type.

---

# 28. Intersection values and identity

Intersections are simpler in one respect: they do not represent several alternatives.

Given:

```raven
InputStream & OutputStream
```

there is still exactly one underlying value.

For example:

```raven
let memory = MemoryStream();
let stream: InputStream & OutputStream = memory;
```

both expressions refer to the same logical object.

The intersection merely retains stronger static knowledge:

```text
underlying value
      │
      ▼
 MemoryStream
   │       │
   ▼       ▼
Input    Output
Stream   Stream
```

No new nominal identity is introduced by forming the intersection.

This is an important difference from a nominal union, whose case identity is part of the value.

---

# 29. Intersection runtime representation

Because an intersection represents one existing value, NeoCLR may initially represent it using the ordinary managed representation of that value.

The runtime type information carries the additional guarantees:

```text
static type:
    InputStream & OutputStream

value:
    managed reference ─────► MemoryStream
```

This may mean that intersections require **no additional wrapper representation at all** for reference values.

Member dispatch simply selects the appropriate constituent interface.

For value types, NeoCLR may require boxing or another managed representation when an intersection value escapes its concrete static type.

The exact ABI should be specified separately.

The important point is that native intersection types no longer force an artificial:

```text
Intersection<A,B>
    Value: Object
```

carrier.

---

# 30. Union runtime representation

Ad-hoc unions require somewhat more runtime information because the active alternative must remain observable.

However, because alternatives are type-based, the runtime may be able to derive the active alternative from the represented value in many cases.

For example:

```raven
String | Int32
```

containing a `String` can determine its active alternative from the value's runtime type.

Cases involving:

```text
value types
nullability
overlapping assignability
subtyping
nested unions
```

may require additional representation rules.

NeoCLR should therefore specify union value representation independently from the metadata type model.

The initial representation can favor simplicity over optimal packing.

---

# 31. Overlapping union alternatives

Native unions raise an important question when one alternative satisfies another.

For example:

```raven
Animal | Dog
```

where:

```text
Dog <: Animal
```

A `Dog` value satisfies both alternatives.

For an ad-hoc union whose alternatives are identified by type, NeoCLR needs a deterministic interpretation.

Possible rules include:

```text
preserve the statically selected alternative
choose the most specific matching alternative
normalize redundant alternatives
```

The third option would potentially reduce:

```raven
Animal | Dog
```

to:

```raven
Animal
```

because every `Dog` already satisfies `Animal`.

That may be mathematically attractive but could erase information a source language intended to preserve.

This requires explicit design.

It is another reason metadata representation and runtime value representation should be designed together rather than assuming that an `Object` payload solves every union case.

---

# 32. Canonicalization

Once union and intersection expressions are native NeoCLR types, the runtime needs canonicalization rules.

At minimum, NeoCLR should investigate whether:

```text
A | B == B | A
A & B == B & A
```

and whether duplicates collapse:

```text
A | A → A
A & A → A
```

Nested expressions may also flatten:

```text
(A | B) | C → A | B | C
(A & B) & C → A & B & C
```

These rules affect:

```text
metadata identity
generic instantiation
signature equality
constraint checking
Introspection
compiler interoperability
JIT caches
```

Canonicalization should therefore be defined by NeoCLR rather than independently by each language.

More aggressive simplifications based on subtyping should be considered separately.

---

# 33. Interaction between unions and intersections

Because both are native type expressions, they can theoretically compose:

```raven
A & (B | C)
(A & B) | C
A | (B & C)
```

The type system can interpret these recursively.

For example:

```text
Satisfies(T, A & (B | C))

    =

Satisfies(T, A)
AND
(
    Satisfies(T, B)
    OR
    Satisfies(T, C)
)
```

This gives NeoCLR a small but expressive type algebra.

However, the initial implementation does not need to support every possible composition in every type position.

NeoCLR may initially constrain the grammar while retaining a metadata model capable of future expansion.

---

# 34. Generic constraint model

Native unions and intersections suggest that NeoCLR should define generic constraints in terms of **type satisfaction**, rather than a fixed CLR-style collection of base/interface constraints.

Conceptually:

```text
GenericParameterInfo
    Constraints:
        TypeInfo expressions
```

or perhaps a single compound constraint expression.

Then:

```raven
where T : A & B
```

and:

```raven
where T : A | B
```

do not require special constraint metadata forms.

They are ordinary NeoCLR type expressions interpreted in constraint position.

This also means the existing familiar form:

```raven
where T : A, B
```

could simply be a language projection of:

```raven
where T : A & B
```

if Raven chooses to retain comma-style constraint syntax.

The runtime model itself only needs the AND relationship.

---

# 35. Constraint evaluation

Constraint checking becomes a recursive operation over `TypeInfo`.

Conceptually:

```text
Satisfies(candidate, constraint):

    Named type:
        apply normal nominal satisfaction rules

    Union:
        return ANY constituent is satisfied

    Intersection:
        return ALL constituents are satisfied

    Other type expressions:
        apply their corresponding NeoCLR rules
```

This gives the runtime one coherent mechanism for:

```text
generic instantiation validation
generic method validation
compiler static checking
metadata verification
```

The compiler and runtime must implement equivalent semantics.

---

# 36. Runtime type checks and constraints share type relationships

A major advantage of native compound types is that constraint checking and ordinary runtime type checking no longer need unrelated integration mechanisms.

Both operate on the same type relationships.

For example:

```raven
value is A | B
```

and:

```raven
where T : A | B
```

both ultimately rely on:

```text
satisfies A OR B
```

Likewise:

```raven
value is A & B
```

and:

```raven
where T : A & B
```

both rely on:

```text
satisfies A AND B
```

The context differs—one evaluates a value, the other a type—but the underlying type algebra is shared.

This is substantially simpler than teaching separate runtime mechanisms how to interpret nominal `Union<T...>` and `Intersection<T...>` wrappers.

---

# 37. High-level language responsibilities

NeoCLR defines the type relationships.

A high-level language remains responsible for projecting those relationships into its own programming model.

For Raven, this includes:

```text
syntax
    A | B
    A & B

member completion for intersections

member conflict resolution

pattern matching syntax

exhaustiveness analysis

implicit/explicit conversions

diagnostics

type inference
```

NeoCLR does not need to prescribe Raven's syntax or IDE behavior.

It provides enough metadata and runtime semantics for Raven to implement those features without inventing a private ABI.

---

# 38. Example: Raven ad-hoc union

Raven source:

```raven
func Format(value: String | Int32) -> String
{
    return match value
    {
        String text => text,
        Int32 number => number.ToString()
    };
}
```

NeoCLR metadata describes the parameter directly as:

```text
Union
├── String
└── Int32
```

Raven can perform exhaustiveness analysis from those constituents.

At runtime, NeoCLR understands the same union type when performing type checks.

No:

```text
Union<String,Int32>
```

backing type is required.

---

# 39. Example: Raven intersection

Raven source:

```raven
func Rewrite(
    stream: InputStream & OutputStream
) -> Task<Result<(), StreamError>>
{
    let count = await stream.Read(buffer)?;
    await stream.Write(buffer[..count])?;

    return Ok(());
}
```

NeoCLR metadata describes:

```text
Intersection
├── InputStream
└── OutputStream
```

Raven's compiler combines the two member surfaces.

`Read` resolves through `InputStream`.

`Write` resolves through `OutputStream`.

The runtime knows that any value passed to the parameter must satisfy both contracts.

No:

```text
Intersection<InputStream,OutputStream>
```

backing type is required.

---

# 40. Example: generic constraints

Raven can express:

```raven
func Copy<T>(value: T)
    where T : Cloneable & Serializable
{
    ...
}
```

NeoCLR receives:

```text
constraint:
    Intersection
    ├── Cloneable
    └── Serializable
```

and verifies both relationships.

Likewise:

```raven
func Handle<T>(value: T)
    where T : Request | Event
{
    ...
}
```

uses:

```text
constraint:
    Union
    ├── Request
    └── Event
```

and accepts a type satisfying either constituent.

Nothing about generic constraints requires nominal wrapper types or special marker interfaces.

---

# 41. Example: nominal `Result`

Consider:

```raven
func Load() -> Result<User, LoadError>
```

The return type remains:

```text
Named type:
    Result<User, LoadError>

kind:
    nominal union

cases:
    Ok(User)
    Err(LoadError)
```

It does **not** become:

```text
User | LoadError
```

Raven can therefore distinguish:

```raven
Ok(user)
```

from:

```raven
Err(error)
```

using nominal case identity.

Yet the runtime can use common union machinery for storage, discrimination, matching, and optimization where applicable.

---

# 42. Example: nominal `Option`

Likewise:

```raven
Option<User>
```

remains:

```text
Named type:
    Option<User>

kind:
    nominal union

cases:
    Some(User)
    None
```

It is not rewritten as:

```raven
User | ()
```

even if those types appear superficially similar.

The nominal union communicates semantic intent and case identity that an ad-hoc union does not possess.

---

# 43. ABI implications

Ad-hoc union and intersection types no longer require public nominal ABI types.

Their ABI is defined directly by NeoCLR.

This is an important simplification:

```text
Previous model:

Raven type
    ↓
nominal backing type
    ↓
backing-type ABI
    ↓
runtime conventions


Native model:

Raven type
    ↓
NeoCLR type expression
    ↓
NeoCLR ABI
```

The compiler does not need to know the field layout or special members of `Union<T...>` or `Intersection<T...>` because those types do not exist.

The runtime is free to evolve their physical implementation subject to the NeoCLR ABI specification.

Nominal unions still require a defined nominal-union ABI because they are real named types exchanged between languages and assemblies.

---

# 44. IL implications

Native compound types do not necessarily require a large number of new IL instructions.

Existing general instructions can accept compound type metadata operands where appropriate.

For example, an ordinary type-test instruction can test against:

```text
Named(String)
Union(String, Int32)
Intersection(InputStream, OutputStream)
```

using NeoCLR's normal type-satisfaction rules.

Likewise, generic constraint verification operates against the metadata expression.

Specialized instructions should only be introduced when they represent a genuinely distinct runtime operation or provide a compelling implementation advantage.

The type system should not force instruction proliferation.

---

# 45. JIT implications

Native compound metadata gives the JIT explicit semantic information.

For example:

```text
InputStream & OutputStream
```

guarantees both interface relationships.

The JIT may therefore be able to optimize interface projections or eliminate repeated type checks.

For:

```text
String | Int32
```

the JIT knows the complete declared set of alternatives and may optimize matching accordingly.

Nominal unions similarly expose their complete case set.

The initial implementation need not exploit all of this information.

Native representation ensures that the information is available for future optimization without reverse-engineering library conventions.

---

# 46. Interoperability

Every NeoCLR language can observe the same compound type metadata.

A language does not need Raven's syntax to consume it.

For example:

```text
NeoCLR metadata:
    Union(String, Int32)

Raven:
    String | Int32

Language X:
    union(String, Int32)

Language Y:
    perhaps exposes the TypeInfo form directly
```

Likewise for intersections.

Languages that cannot naturally express a compound type need an interoperability strategy, but they no longer need to understand a Raven-specific nominal wrapper ABI.

Nominal unions remain straightforward named types for languages that support NeoCLR nominal union declarations.

---

# 47. .NET backend implications

Raven targeting .NET remains a separate concern.

The existing .NET backend can continue lowering ad-hoc unions to nominal:

```text
Union<T1,T2,...>
```

because the CLR cannot directly encode the richer NeoCLR type expression.

Likewise, Raven may need a nominal or compiler-generated representation for intersections when targeting .NET.

Therefore:

```text
Raven source
      │
      ├── NeoCLR
      │      A | B → native union type
      │      A & B → native intersection type
      │
      └── .NET
             A | B → nominal lowering
             A & B → backend-specific lowering
```

This is acceptable.

Raven semantics do not need to be restricted to the intersection of every target runtime's native type system.

---

# 48. Migration from current ad-hoc union backing types

Raven currently uses nominal generic backing types for ad-hoc unions.

When targeting NeoCLR, those backing types can eventually disappear.

The migration is conceptually:

```text
Current:

A | B
    ↓
Union<A,B>


NeoCLR:

A | B
    ↓
native union type expression
```

The nominal `Union<T...>` types may remain available where required by the .NET backend or compatibility tooling, but they are no longer part of the NeoCLR representation of ad-hoc unions.

This also removes the need to keep adding generic arities merely to represent larger ad-hoc unions on NeoCLR.

---

# 49. Design boundaries

Native union/intersection support should not imply:

- general structural typing;
- automatic equivalence between nominal and ad-hoc unions;
- elimination of nominal union declarations;
- arbitrary executable constraint predicates;
- a requirement that every language expose `|` and `&`;
- dedicated IL instructions for every union/intersection operation;
- a sophisticated optimized value representation in the first implementation.

The initial goal is narrower:

> **Make union and intersection relationships first-class parts of NeoCLR's metadata and type-satisfaction model.**

---

# 50. Open questions

Several areas require follow-up design:

```text
Metadata encoding
-----------------
How are union/intersection expressions encoded in neoIL metadata?

Type identity
-------------
Are constituent orderings canonicalized?
Are duplicates removed?
Are nested expressions flattened?

Subtyping
---------
Should A | B