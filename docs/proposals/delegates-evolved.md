# neoCLR — Delegates Evolved: Structural Function Types and Nominal Delegates

## Status

**Proposal**

This proposal defines neoCLR's evolved delegate model.

neoCLR retains the fundamental CLR model of a callable object that references executable code and optionally carries a receiver or captured environment. The major change is that callable signatures become structural types in their own right.

A signature such as:

```raven
(int) => string
```

is therefore itself an instantiable callable type.

Traditional nominal delegates remain possible as nominal types built on top of structural Function types.

The result is an evolution rather than a replacement of delegates:

> **neoCLR generalizes delegates from exclusively nominal callable types into structural callable types, while retaining nominal delegates when explicit type identity is useful.**

---

## 1. Motivation

The CLR already provides a useful runtime model for callable references through delegates.

A delegate combines:

- a callable signature,
- a target method,
- an optional receiver,
- a captured environment where necessary,
- and an `Invoke` operation.

The limitation is primarily in the type system.

Consider:

```csharp
delegate int Parser(string value);
delegate int Converter(string value);
```

These are different CLR types even though both describe the callable signature:

```raven
(string) => int
```

The signature itself does not have independent type identity.

neoCLR changes this by making the callable signature a structural type.

---

## 2. Structural Function types

A **Function type** is a structural callable type.

For example:

```raven
(int) => string
```

describes an operation accepting an `int` and returning a `string`.

Likewise:

```raven
(Request, CancellationToken) => Response
```

describes a callable operation accepting `Request` and `CancellationToken` and returning `Response`.

Function type identity is determined structurally.

At minimum, identity includes:

- ordered parameter types,
- return type,
- reference-passing contracts,
- and other calling characteristics defined as part of the neoCLR callable contract.

It does not include declaration-specific information such as:

- method or function name,
- parameter names,
- declaring type,
- module,
- metadata token,
- visibility,
- or unrelated attributes.

Therefore:

```raven
func Increment(value: int) -> int
func Double(number: int) -> int
```

both have the Function type:

```raven
(int) => int
```

---

## 3. Function types are instantiable callable types

A Function type is more than a descriptor used for compatibility checking.

It is an instantiable callable type.

For example:

```raven
val funcObj = ((int) => ())(x => {})
funcObj.Invoke(42)
```

constructs a callable object whose structural type is:

```raven
(int) => ()
```

Conceptually, the structural type provides synthesized callable machinery equivalent to:

```raven
// Conceptual representation

class (int) => () {
    func Invoke(x: int) -> ()
}
```

The actual runtime representation does not need to be expressed as a conventional nominal class.

The important property is that `(int) => ()` is a real runtime type capable of representing callable values.

---

## 4. Compiler-supported construction

Most Function values do not need to be constructed explicitly.

For example:

```raven
val operation: (int) => int = x => x * 2
```

can be understood as compiler-supported construction equivalent to:

```raven
val operation = ((int) => int)(x => x * 2)
```

Likewise:

```raven
func Double(value: int) -> int {
    return value * 2
}

val operation = Double
```

causes the compiler to infer `Double`'s Function type and construct the corresponding callable reference.

Conceptually:

```raven
val operation = ((int) => int)(Double)
```

The ordinary language syntax hides this construction because the required type is normally evident from inference or target typing.

---

## 5. Methods and functions have Function types

Function types are not created only when callable objects are instantiated.

Every callable declaration has a corresponding structural Function type.

For example:

```raven
func Parse(value: string) -> int {
    // ...
}
```

has:

```raven
(string) => int
```

as its Function type.

Its introspection descriptor can therefore expose:

```raven
method.FunctionType
```

Conceptually:

```text
MethodInfo
    │
    ├── Name
    ├── Parameters
    ├── ReturnType
    ├── DeclaringType
    ├── metadata...
    │
    └── FunctionType
             │
             ▼
       FunctionTypeInfo
       (string) => int
```

`MethodInfo` describes the declaration.

`FunctionTypeInfo` describes the structural callable projection of that declaration.

---

## 6. Inferring callable types from declarations

Consider:

```raven
func Foo(x: int) {}

val foo = Foo
```

The compiler obtains the Function type represented by `Foo`'s callable signature:

```text
Foo
 │
 ▼
MethodInfo
 │
 └── FunctionType
        │
        ▼
   (int) => ()
```

and creates a callable object of that structural type.

Conceptually:

```raven
val foo = ((int) => ())(Foo)
```

Inference copies all characteristics of the callable signature that participate in Function type identity.

It does not merely approximate the declaration as something callable with superficially similar parameters.

---

## 7. Target-typed callable construction

An explicit Function type instead establishes the required callable structure:

```raven
val foo: (int) => () = Foo
```

Here:

```raven
(int) => ()
```

is authoritative as the type of `foo`.

The compiler verifies that `Foo` can be bound to that callable contract.

Conceptually:

```text
Foo.FunctionType
       │
       │ compatible?
       ▼
 (int) => ()
       │
       ▼
      foo
```

This differs from:

```raven
val foo = Foo
```

where the Function type is inferred from `Foo`.

The distinction leaves room for future compatibility rules where callable signatures may be compatible without being identical.

---

## 8. Callable object versus executable declaration

A Function value is not the executable declaration itself.

It is a callable reference to that declaration.

Conceptually:

```text
Executable declaration
        │
        │ bind
        ▼
Callable object
        │
        ├── referenced executable
        ├── receiver, if required
        ├── captured environment, if required
        └── Invoke(...)
```

This is fundamentally similar to the CLR delegate model.

neoCLR does not need to turn executable methods and functions into ordinary objects merely to make functions usable as values.

Instead, callable references can behave like first-class function objects from the programmer's perspective.

---

## 9. Referenced executable

A callable object exposes its referenced executable through:

```raven
foo.Function
```

For the current development model this returns a `MethodInfo`.

For example:

```raven
func Foo(x: int) {}

val foo = Foo

foo.Function
// MethodInfo describing Foo
```

The distinction is intentional.

The Function object's type answers:

> Through what structural signature can this object be invoked?

The `Function` property answers:

> Which executable declaration does this object reference?

Thus:

```raven
foo.Function.Parameters
foo.Function.ReturnType
```

describe the original declaration.

They need not be flattened onto the callable object itself.

---

## 10. Function type introspection

Structural Function types are represented through `FunctionTypeInfo`.

For:

```raven
(string) => int
```

the corresponding descriptor exposes the callable structure:

```text
FunctionTypeInfo
    ├── Parameters
    ├── ReturnType
    └── InvokeMethod
```

These members describe the structural callable type.

They are distinct from declaration metadata obtained through:

```raven
foo.Function
```

For example, parameter names belong to the referenced declaration rather than structural Function identity.

Two declarations:

```raven
func Foo(value: string) -> int
func Bar(text: string) -> int
```

can therefore share the same:

```raven
FunctionTypeInfo
```

while their respective `MethodInfo` descriptors retain `value` and `text`.

---

## 11. Delegates evolved

This model can be understood directly as an evolution of CLR delegates.

The CLR model is approximately:

```text
Nominal Delegate Type
        │
        ▼
Delegate object
        ├── target
        ├── receiver/environment
        └── Invoke(...)
```

neoCLR introduces:

```text
Structural Function Type
        │
        ▼
Callable object
        ├── target
        ├── receiver/environment
        └── Invoke(...)
```

The runtime concept remains familiar.

The major change is that the signature itself has type identity.

Instead of requiring:

```csharp
delegate int Transformer(int value);
```

neoCLR can directly express:

```raven
(int) => int
```

as the callable type.

---

## 12. Nominal delegates

Structural Function types do not eliminate the usefulness of nominal callable types.

neoCLR can retain explicit delegate declarations:

```raven
delegate Foo<T>(x: T) -> ()
```

Such a declaration introduces a nominal callable type associated with the structural signature:

```raven
(T) => ()
```

It can be understood conceptually as:

```raven
class Foo<T> : (T) => () {
    // Inherited or synthesized from the signature:
    //
    // func Invoke(x: T) -> ()
}
```

This representation is conceptual. Structural Function types do not need to become ordinary nominal classes internally.

The important relationship is:

```text
Foo<T>
   │
   │ nominal callable type based on
   ▼
(T) => ()
```

---

## 13. Instantiating nominal delegates

A nominal delegate is instantiated in the familiar object-like manner:

```raven
delegate Foo<T>(x: T) -> ()

val funcObj = Foo<int>(x => {})
funcObj.Invoke(42)
```

The structural equivalent is:

```raven
val funcObj = ((int) => ())(x => {})
funcObj.Invoke(42)
```

The compiler can provide more concise forms in contexts where the target type is already known.

For example:

```raven
val funcObj: Foo<int> = x => {}
```

can lower to the nominal construction.

Likewise:

```raven
val funcObj: (int) => () = x => {}
```

can lower to structural construction.

The two forms therefore share the same fundamental callable model.

---

## 14. Structural versus nominal identity

Consider:

```raven
delegate Foo<T>(x: T) -> ()
delegate Bar<T>(x: T) -> ()
```

For `T = int`, both have the callable signature:

```raven
(int) => ()
```

but `Foo<int>` and `Bar<int>` remain different nominal types.

Conceptually:

```text
              (int) => ()
                 ▲  ▲
                 │  │
          Foo<int>  Bar<int>
```

Therefore:

```text
Foo<int> → (int) => ()     valid
Bar<int> → (int) => ()     valid

Foo<int> → Bar<int>        not implied
Bar<int> → Foo<int>        not implied
```

Sharing structural callable identity does not erase nominal identity.

---

## 15. Structural conversion may forget nominal identity

A nominal delegate can be consumed through its structural Function type.

For example:

```raven
delegate Parser(value: string) -> int

val parser = Parser(Parse)

val operation: (string) => int = parser
```

This is valid because `Parser` explicitly has `(string) => int` as its callable structure.

The conversion forgets the additional nominal identity.

The reverse should not happen implicitly merely because the signatures match:

```raven
val operation: (string) => int = Parse

val parser: Parser = operation
// Not implied merely by structural compatibility
```

The general rule is:

> **Structural conversion may forget nominal identity, but structural compatibility does not invent nominal identity.**

---

## 16. Nominal delegates are not aliases

A nominal delegate declaration:

```raven
delegate Parser(value: string) -> int
```

does not mean:

```text
Parser ≡ (string) => int
```

Instead:

```text
Parser
   │
   │ callable as
   ▼
(string) => int
```

`Parser` has its own nominal identity.

A true type alias, if supported:

```raven
type Parser = (string) => int
```

would instead make `Parser` another name for the structural Function type itself.

The two concepts should remain distinct.

---

## 17. Function types alongside other structural types

Function types fit naturally into neoCLR's broader structural type system:

```raven
A | B

A & B

(A, B) => R
```

Each describes type structure without requiring a nominal declaration.

Nominal types can then be introduced where semantic identity is useful.

This means the neoCLR type system can distinguish between:

```raven
(Request) => Response
```

which means:

> Any callable with this structure.

and:

```raven
delegate RequestHandler(request: Request) -> Response
```

which means:

> A callable specifically carrying the nominal identity `RequestHandler`.

---

## 18. Multicast behavior

Structural Function values represent a single callable binding.

neoCLR does not require callable values to inherit the CLR `MulticastDelegate` model.

Multiple callbacks can instead be represented explicitly through collections or higher-level abstractions.

For example, conceptually:

```raven
List<(Event) => ()>
```

can represent multiple event handlers without making multicast invocation an intrinsic property of every callable object.

A separate multicast abstraction can be introduced later if required.

---

## 19. Future FunctionInfo

The current development API exposes:

```raven
callable.Function -> MethodInfo
```

This is sufficient while referenced executable declarations are represented as methods.

neoCLR's support for free or module-level functions may eventually justify a broader abstraction:

```text
FunctionInfo
    ├── MethodInfo
    └── ModuleFunctionInfo
```

or an equivalent model.

Both forms could expose:

```raven
FunctionType
```

while callable objects expose their referenced declaration through:

```raven
callable.Function
```

The exact hierarchy is outside the scope of this proposal.

---

## 20. Relationship to the CLR model

The design intentionally avoids changing more of the CLR model than necessary.

The existing delegate model already solves important runtime problems:

- storing executable references,
- binding instance receivers,
- representing closures,
- typed invocation,
- and passing behavior as values.

neoCLR retains these ideas.

The type-system limitation is addressed independently.

The evolution is therefore:

```text
CLR

delegate Foo<T>(T value)
        │
        ▼
nominal callable type
        │
        ▼
callable object


neoCLR

(T) => ()
        │
        ▼
structural callable type
        │
        ▼
callable object
```

with nominal identity optionally restored through:

```text
delegate Foo<T>(x: T) -> ()
        │
        ▼
Foo<T>
        │
        │ callable structure
        ▼
(T) => ()
```

This creates a superset of the traditional delegate model rather than replacing it.

---

## 21. Design principles

### Preserve delegate semantics

neoCLR should retain the useful target, receiver, capture, and invocation semantics of CLR delegates.

### Make callable signatures first-class types

A signature such as:

```raven
(T) => R
```

should exist independently within the type system.

### Make structural callable types instantiable

A Function type should be capable of constructing a callable object:

```raven
((int) => int)(x => x + 1)
```

even though compiler inference normally hides this syntax.

### Keep executable identity separate

A callable object references a method or function. It is not itself the executable declaration.

The declaration remains inspectable through:

```raven
callable.Function
```

### Preserve nominal delegates where useful

Nominal delegate declarations remain available for APIs that require semantic callable identity.

### Do not conflate structural and nominal identity

Two nominal delegates can share the same structural Function type while remaining distinct nominal types.

---

## 22. Result

neoCLR does not need to replace delegates with a fundamentally different function-object abstraction.

Instead, it can **evolve delegates**.

The core model becomes:

```text
Executable declaration
        │
        │ has
        ▼
Structural Function type
        │
        │ instantiated/bound as
        ▼
Callable object
        │
        ├── Function
        └── Invoke(...)
```

and, optionally:

```text
Structural Function type
        │
        │ given explicit identity by
        ▼
Nominal delegate type
        │
        ▼
Nominal callable object
```

Thus:

```raven
(int) => int
```

is the structural form of a delegate type, while:

```raven
delegate Transformer(value: int) -> int
```

is its nominal counterpart.

Both use the same fundamental callable model.

The difference is type identity.

> **neoCLR evolves delegates by making callable signatures structural, first-class and instantiable types, while retaining nominal delegates as an explicit specialization when semantic type identity is required.**

This provides function-like values and first-class callable signatures without abandoning the proven execution model of the CLR.