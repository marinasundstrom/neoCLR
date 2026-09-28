# neoCLR Structural Function Types and Callable References

## Status

**Proposal**

This proposal defines Function types as structural callable types in neoCLR and describes their relationship to methods, functions, callable references, introspection, and the existing CLR delegate model.

The intent is not to introduce executable functions as ordinary runtime objects. Instead, neoCLR evolves the delegate model by giving callable signatures structural identity within the type system.

---

## 1. Summary

neoCLR introduces **Function types** as structural types describing callable signatures.

For example:

```raven
(int) => string
```

describes any callable operation accepting an `int` and returning a `string`.

A reference to a method or function can be represented as a value of such a type:

```raven
func Format(value: int) -> string {
    // ...
}

val formatter = Format
```

The inferred type of `formatter` is:

```raven
(int) => string
```

The resulting value remains conceptually similar to a CLR delegate: it identifies an executable target and retains a receiver or captured environment where required.

The important difference is that its type is not a nominal delegate declaration. Its type is the callable signature itself.

Function types therefore evolve delegates from:

```text
nominal callable types
```

into:

```text
structural callable types
```

This allows callable references to be passed, stored, returned, and invoked much like first-class function objects without requiring neoCLR to model executable functions themselves as ordinary objects.

---

## 2. Motivation

The CLR already has a strong mechanism for representing callable references through delegates.

A delegate combines:

- an invocation signature,
- a target method,
- an optional receiver,
- and, for closures, a captured environment.

However, delegate identity is nominal.

Two declarations such as:

```csharp
delegate int Parser(string value);
delegate int Converter(string value);
```

represent distinct CLR types even though their callable signatures are identical.

The callable shape:

```text
(string) => int
```

does not itself have type identity.

neoCLR changes this.

A callable signature becomes directly representable within the type system:

```raven
(string) => int
```

This type can then be used wherever a callable value is required.

The goal is therefore not to replace the useful runtime semantics of delegates. It is to make the signature that delegates describe a first-class structural type.

---

## 3. Function types

A **Function type** describes a callable signature.

For example:

```raven
(int, int) => int
```

describes an operation accepting two `int` values and returning an `int`.

Function type identity is structural.

Two callable signatures represent the same Function type when all components participating in callable identity are equivalent.

At minimum, this includes:

- ordered parameter types,
- return type,
- reference-passing contracts,
- and other calling characteristics defined as part of the neoCLR callable contract.

Parameter names do not participate in Function type identity.

Neither do declaration-specific properties such as:

- function or method name,
- declaring type,
- visibility,
- metadata token,
- module,
- or declaration attributes unrelated to invocation.

For example:

```raven
func AddOne(value: int) -> int
func Double(number: int) -> int
```

both have the Function type:

```raven
(int) => int
```

despite having different names and differently named parameters.

---

## 4. Callable values

A value whose type is a Function type is a **callable value** or **Function value**.

For example:

```raven
func Double(value: int) -> int {
    return value * 2
}

val operation = Double
```

`operation` has the inferred type:

```raven
(int) => int
```

The runtime value remains delegate-like.

It contains sufficient information to invoke the referenced executable and may retain:

- the target method or function,
- an instance receiver,
- or a captured closure environment.

Conceptually:

```text
Function type
(int) => int
      │
      ▼
Callable value
      │
      ├── referenced executable
      ├── receiver, if required
      └── captured environment, if required
```

The callable value should not be understood as the executable function itself.

It is a callable reference or binding to an executable target.

This distinction allows neoCLR to provide function-like values without requiring executable code declarations to become ordinary runtime objects.

---

## 5. Delegates evolved

Function values can be understood as an evolution of CLR delegates.

The CLR model is approximately:

```text
Nominal Delegate Type
        │
        ▼
Delegate value
        ├── target method
        ├── receiver/environment
        └── Invoke(...)
```

neoCLR generalizes this to:

```text
Structural Function Type
        │
        ▼
Callable value
        ├── target method/function
        ├── receiver/environment
        └── Invoke(...)
```

The primary change is therefore **type identity**, rather than the fundamental mechanics of callable binding.

neoCLR does not require a nominal declaration equivalent to:

```csharp
delegate int Transformer(int value);
```

The signature itself:

```raven
(int) => int
```

is the callable type.

This removes the need for nominal delegate declarations when nominal identity provides no useful semantic distinction.

---

## 6. Methods and functions have Function types

Function types are not created only when callable values are constructed.

A method or function declaration itself has a corresponding Function type.

For example:

```raven
func Parse(value: string) -> int {
    // ...
}
```

has the callable signature:

```raven
(string) => int
```

Its introspection descriptor can therefore expose:

```raven
method.FunctionType
```

returning a `FunctionTypeInfo` describing:

```raven
(string) => int
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

`MethodInfo` describes the complete method declaration.

`FunctionTypeInfo` describes the structural callable projection of that declaration.

Multiple unrelated methods may therefore expose the same Function type.

```raven
func Foo(x: int) -> string
func Bar(value: int) -> string
```

Conceptually:

```raven
Foo.FunctionType == Bar.FunctionType
```

while:

```raven
Foo != Bar
```

The declarations remain distinct even though their callable structures are identical.

---

## 7. Inferred Function types

When a callable reference is created without an explicit target type, its Function type is inferred from the referenced declaration.

For example:

```raven
func Foo(x: int) {}

val foo = Foo
```

is conceptually equivalent to:

```raven
val foo: (int) => () = Foo
```

The first form derives the structural Function type from `Foo`.

Conceptually:

```text
Foo
 │
 ▼
MethodInfo
 │
 │ FunctionType
 ▼
(int) => ()
 │
 ▼
foo
```

Inference should preserve every part of the method signature that participates in Function type identity.

It should not create an approximate or reduced callable signature.

---

## 8. Explicit Function types

An explicit Function type reverses the relationship.

Consider:

```raven
val foo: (int) => () = Foo
```

Here, `(int) => ()` is already established as the type of `foo`.

The compiler must determine whether the callable signature of `Foo` is compatible with that Function type.

Conceptually:

```text
Foo.FunctionType
       │
       │ compatibility
       ▼
(int) => ()
       │
       ▼
      foo
```

This distinction is important.

With:

```raven
val foo = Foo
```

the Function type is **derived from `Foo`**.

With:

```raven
val foo: (int) => () = Foo
```

the Function type is **imposed by the target context**, and `Foo` must satisfy the required callable contract.

These operations commonly produce the same result but are not conceptually identical.

This distinction leaves room for future callable compatibility rules where two signatures may be compatible without being identical.

---

## 9. Function type versus referenced declaration

A callable value has two related but distinct pieces of information.

First, it has its structural Function type:

```text
(int) => string
```

Second, it references a particular executable declaration:

```text
Parser.Parse
```

These should remain distinguishable.

For example:

```raven
val parser: (string) => int = Parser.Parse
```

The callable type answers:

> Through what signature can this value be invoked?

The referenced declaration answers:

> Which executable is this callable value bound to?

These are usually closely related but are not the same concept.

---

## 10. The `Function` property

Function values expose their referenced executable through a synthesized, read-only `Function` property.

For the current development API:

```raven
val parser = Parser.Parse

parser.Function
```

returns the `MethodInfo` describing `Parser.Parse`.

This makes the delegate-like nature of callable values explicit.

The callable value does not pretend to be the method or function itself. Instead, it provides a reference to the declaration it represents.

For example:

```raven
parser.Function.Parameters
parser.Function.ReturnType
```

describe the actual referenced method declaration.

This information should not automatically be flattened onto the callable value itself.

In particular, APIs such as:

```raven
parser.Parameters
parser.ReturnType
```

would create ambiguity between:

- the structural signature of the callable value, and
- the declaration signature of the referenced executable.

The explicit `Function` boundary preserves that distinction.

---

## 11. Function type introspection

`FunctionTypeInfo` describes a structural Function type.

For:

```raven
(string) => int
```

the corresponding `FunctionTypeInfo` exposes the structural signature.

Conceptually:

```text
FunctionTypeInfo
    ├── Parameters
    ├── ReturnType
    └── InvokeMethod
```

These descriptors represent the callable type rather than a particular declaration.

Therefore:

```text
FunctionTypeInfo.Parameters
```

need not contain declaration-specific parameter information such as source parameter names.

By contrast:

```text
callable.Function.Parameters
```

describes the parameters of the referenced declaration and may expose names and other declaration metadata.

This gives neoCLR two deliberate introspection paths:

```text
Callable value
    │
    ├── Type
    │     └── FunctionTypeInfo
    │           ├── structural Parameters
    │           └── structural ReturnType
    │
    └── Function
          └── MethodInfo
                ├── declared Parameters
                ├── declared ReturnType
                ├── Name
                ├── DeclaringType
                └── metadata...
```

---

## 12. Function types as a general type-system concept

Function types should not be treated merely as implementation details for callable values.

They are structural types in their own right.

This places them alongside other neoCLR structural type constructs such as unions and intersections:

```raven
A | B

A & B

(A, B) => R
```

Each describes type structure without requiring a nominal declaration solely to establish identity.

Nominal types can still appear as components:

```raven
(HttpRequest) => HttpResponse
```

Here `HttpRequest` and `HttpResponse` may be nominal types while the Function type itself is structural.

Function types can therefore participate naturally in:

- parameters,
- return types,
- fields,
- generic arguments,
- collections,
- type inference,
- overload resolution,
- introspection,
- and eventually dynamic-language interoperability.

---

## 13. Functions as values without functions as objects

Function types provide most of the practical benefits commonly associated with first-class function objects.

A developer can:

```raven
val handler = HandleRequest

Register(handler)

return handler
```

Callable references can be passed through an application as ordinary typed values.

From the language user's perspective, this behaves much like passing function objects.

neoCLR does not, however, need to assert that the executable declaration itself is an object.

Instead:

```text
Executable declaration
        │
        │ bind/reference
        ▼
Callable value
        │
        │ has
        ▼
Structural Function type
```

This distinction fits naturally with a CLR-like runtime, where executable declarations, metadata descriptors, objects, and callable bindings already have different runtime roles.

---

## 14. Future `FunctionInfo`

The current development model exposes:

```raven
callable.Function -> MethodInfo
```

This is sufficient while callable targets are represented as methods.

However, neoCLR also supports or intends to support functions that are not members of nominal types.

A future common abstraction may therefore be introduced:

```text
FunctionInfo
    ├── MethodInfo
    └── ModuleFunctionInfo
```

or an equivalent model.

Callable values could then expose:

```raven
callable.Function -> FunctionInfo
```

Both method and free-function descriptors could expose:

```raven
FunctionType
```

giving the general relationship:

```text
Executable descriptor
    │
    ├── MethodInfo
    └── FunctionInfo
          │
          │ FunctionType
          ▼
    FunctionTypeInfo
          │
          │ type of binding
          ▼
     Callable value
```

The exact `FunctionInfo` hierarchy is outside the scope of this proposal.

---

## 15. Multicast behavior

Structural Function values represent a single callable binding.

neoCLR does not require the CLR `MulticastDelegate` model.

Combining multiple callable values is better represented explicitly using an appropriate collection or higher-level abstraction.

This keeps Function types focused on a single structural callable contract rather than coupling callable identity to multicast behavior.

---

## 16. Design principles

This proposal follows several principles.

### Preserve what works in delegates

Target binding, receivers, closure environments, typed invocation, and callable references are useful concepts and do not need to be discarded merely because delegate type identity is nominal.

### Make signatures first-class types

The signature:

```raven
(T1, T2) => R
```

should itself be representable within the neoCLR type system.

A nominal delegate declaration should not be required merely to give that signature type identity.

### Separate callable shape from executable identity

`FunctionTypeInfo` answers:

> What callable structure does this type have?

`MethodInfo` or a future `FunctionInfo` answers:

> What executable declaration is this?

A callable value connects the two.

### Do not pretend bindings are executable declarations

A callable reference may behave like a first-class function value without being identical to the underlying method or function.

The referenced declaration remains separately inspectable through `Function`.

### Structural identity should remain structural

Names, declaring types, modules, metadata tokens, and other declaration properties do not become part of Function type identity merely because the Function type originated from a method.

---

## 17. Example

Consider:

```raven
func Increment(value: int) -> int {
    return value + 1
}

func Double(number: int) -> int {
    return number * 2
}

func Apply(value: int, operation: (int) => int) -> int {
    return operation(value)
}

val increment = Increment
val double = Double

Apply(10, increment)
Apply(10, double)
```

Both `increment` and `double` have:

```raven
(int) => int
```

as their Function type.

Their structural type information is therefore equivalent:

```text
increment type
    └── (int) => int

double type
    └── (int) => int
```

Their referenced declarations remain different:

```raven
increment.Function
// MethodInfo for Increment

double.Function
// MethodInfo for Double
```

The declarations may expose different names and parameter names:

```raven
increment.Function.Parameters[0].Name
// "value"

double.Function.Parameters[0].Name
// "number"
```

while their structural Function type remains the same.

---

## 18. Result

The neoCLR Function model can therefore be summarized as:

> **Function types are structural delegate types. Function values are delegate-like callable references whose identity is determined by their callable signature rather than a nominal delegate declaration. Methods and functions expose their corresponding Function type, allowing callable signatures to exist independently as first-class members of the neoCLR type system.**

This provides the practical experience of passing functions as values while remaining compatible with the fundamental execution model of a CLR-like runtime.

Rather than replacing delegates with an unrelated abstraction, neoCLR evolves them.