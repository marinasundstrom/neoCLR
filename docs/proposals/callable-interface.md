# neoCLR `Callable` Interface — Extension Proposal

## Status

**Extension proposal**

**Open proposal — not implemented on main (2026-09-30).** This document
explores a possible contract, not completed work or a release commitment. Main's
callback annotations use Raven function type syntax while retaining the existing
nominal Func/delegate runtime. Structural Function types, nominal specializations
and the Callable interface described here are not part of this backport. Example
syntax below is illustrative proposal notation; current Raven callback types use
`(T) -> R`.


This proposal introduces `Callable`, a common interface for neoCLR values that represent exactly one callable signature.

`Callable` complements structural Function types and nominal delegates by providing a shared abstraction similar to the role `System.Delegate` serves in .NET, without requiring callable types to share a common base class.

---

## 1. Summary

neoCLR Function types represent callable signatures structurally:

```raven
(int) => string
```

A value of such a type represents one callable binding with one authoritative signature.

However, APIs sometimes need to accept a callable without statically specifying what that signature is.

For example:

```raven
func Register(handler: Callable)
```

`Register` may accept:

```raven
() => ()

(int) => string

(HttpRequest) => HttpResponse
```

without requiring a common Function signature.

To support this, neoCLR introduces:

```raven
interface Callable {
    FunctionTypeInfo FunctionType { get; }
}
```

The fundamental invariant is:

> **A `Callable` represents exactly one callable signature.**

`FunctionType` preserves that signature after the concrete Function type has been erased behind the `Callable` interface.

---

## 2. Motivation

.NET delegates share the common base type:

```csharp
System.Delegate
```

This permits APIs such as:

```csharp
void Register(Delegate callback)
```

to accept arbitrary delegate types without knowing their exact signatures statically.

neoCLR structural Function types remove the requirement for every callable signature to have a nominal delegate declaration.

For example:

```raven
(int) => int
```

and:

```raven
(string, bool) => Result
```

are independent structural Function types.

Without another abstraction, there is no common type through which an API can express:

> Accept any callable value.

`Callable` provides that abstraction.

Unlike `System.Delegate`, it is an interface rather than a common base class.

---

## 3. Definition

The conceptual interface is:

```raven
interface Callable {
    FunctionTypeInfo FunctionType { get; }
}
```

A type implementing `Callable` guarantees that its value represents exactly one callable contract.

For example:

```raven
val callback: Callable = GetUser
```

may erase the concrete Function type statically, but:

```raven
callback.FunctionType
```

still identifies:

```raven
(UserId) => User
```

at runtime.

The interface therefore provides **type erasure without signature erasure**.

---

## 4. Single-signature invariant

A `Callable` must represent exactly one Function type.

Conceptually:

```text
Callable
    │
    └── FunctionType
            │
            ▼
      FunctionTypeInfo
      (T1, T2) => R
```

There must be one authoritative answer to:

```raven
callable.FunctionType
```

This distinguishes `Callable` from the broader language concept of something that can be invoked.

---

## 5. Structural Function types implement `Callable`

Every structural Function type intrinsically implements `Callable`.

For example:

```raven
(int) => string
```

has the conceptual relationship:

```text
Callable
    ▲
    │
(int) => string
```

A value:

```raven
val formatter: (int) => string = Format
```

can therefore be assigned to:

```raven
val callable: Callable = formatter
```

and:

```raven
callable.FunctionType
```

returns the `FunctionTypeInfo` corresponding to:

```raven
(int) => string
```

The Function type itself remains structural.

Implementing `Callable` does not give it additional nominal Function identity.

---

## 6. Function types remain authoritative

`Callable` does not introduce another representation of callable signatures.

Its `FunctionType` property exposes the existing structural Function type.

For:

```raven
func Format(value: int) -> string
```

the model is:

```text
MethodInfo
    │
    └── FunctionType
            │
            ▼
       (int) => string
            │
            │ instantiated as
            ▼
      callable value
            │
            └── Callable.FunctionType
                       │
                       └── same FunctionTypeInfo
```

`MethodInfo.FunctionType` and `Callable.FunctionType` therefore refer to the same type-system concept.

The interface does not derive a signature independently from `Invoke`.

---

## 7. Invocation contract

A structural Function type has a synthesized typed `Invoke` operation corresponding to its Function type.

For:

```raven
(int) => string
```

the callable contract includes:

```raven
func Invoke(value: int) -> string
```

Conceptually:

```text
Callable
    │
    └── FunctionType
           │
           ▼
      (int) => string
           │
           └── Invoke(int) -> string
```

The `FunctionType` is authoritative for the callable contract.

`Invoke` is its executable manifestation.

---

## 8. `Callable` does not itself define `Invoke`

The `Callable` interface cannot declare one universally typed `Invoke` method because different implementations have different signatures.

For example:

```raven
() => ()

(int) => string

(Request, Context) => Response
```

cannot share a statically typed `Invoke` declaration without erasing their parameter and return types.

Therefore:

```raven
interface Callable {
    FunctionTypeInfo FunctionType { get; }
}
```

does not itself declare:

```raven
func Invoke(...)
```

Instead, the consuming compiler or runtime knows that a `Callable` represents one Function type and that the corresponding callable implementation provides an `Invoke` operation matching that signature.

---

## 9. Dynamic invocation

One important purpose of `Callable` is to support framework and dynamic invocation scenarios.

Consider:

```raven
func Execute(callback: Callable) {
    // exact Function type is not statically known here
}
```

The implementation can inspect:

```raven
callback.FunctionType
```

to determine the callable contract.

For example:

```text
Callable
    │
    └── FunctionType
           │
           ├── Parameters
           └── ReturnType
```

The runtime can then validate dynamically supplied arguments against the Function type and dispatch to the corresponding `Invoke` operation.

This avoids requiring the consumer to rediscover callable structure from arbitrary methods.

The value explicitly advertises its one authoritative callable signature.

---

## 10. Framework signature matching

`Callable` also supports APIs that intentionally accept callbacks with different signatures.

For example:

```raven
func MapGet(path: string, handler: Callable)
```

could accept:

```raven
func GetUsers() -> List<User>

func GetUser(id: UserId) -> User

func GetUser(id: UserId, context: HttpContext) -> User
```

Each supplied value is a `Callable`.

Their Function types differ:

```text
() => List<User>

(UserId) => User

(UserId, HttpContext) => User
```

but all can be passed through the common interface:

```raven
MapGet("/users", GetUsers)

MapGet("/users/{id}", GetUser)
```

The framework or consuming compiler can inspect `FunctionType` and match the callback against supported handler patterns.

This provides a platform-level equivalent to APIs that currently accept or infer different .NET delegate shapes.

---

## 11. Callable versus language-level callability

`Callable` should not define every object that a language permits invocation syntax against.

For example, Raven may allow an object with an appropriate `Invoke` method to be called:

```raven
class Handler {
    func Invoke(value: int) -> string {
        // ...
    }
}

val handler = Handler()

handler(42)
```

This can remain a Raven language feature based on static resolution or duck typing.

It does not necessarily imply:

```text
Handler implements Callable
```

The distinction is intentional.

### Language-callable

A language may permit invocation because it can resolve an appropriate operation.

### `Callable`

A platform abstraction stating that the value itself represents exactly one authoritative callable signature.

These concepts overlap but are not identical.

---

## 12. Multiple `Invoke` methods

Consider:

```raven
class Router {
    func Invoke(request: HttpRequest) -> HttpResponse

    func Invoke(message: Message) -> Result
}
```

Raven may permit either invocation:

```raven
router(request)

router(message)
```

through normal overload resolution.

However, `Router` does not naturally represent one `Callable`.

It has two potential callable signatures:

```raven
(HttpRequest) => HttpResponse

(Message) => Result
```

Therefore there is no single correct value for:

```raven
router.FunctionType
```

The object should not automatically implement `Callable`.

A particular callable view can instead be selected:

```raven
val handler: (HttpRequest) => HttpResponse = router
```

`handler` now represents exactly one callable contract and therefore implements `Callable`.

Conceptually:

```text
Router
    │
    ├── Invoke(HttpRequest) -> HttpResponse
    │
    └── Invoke(Message) -> Result
    │
    │ select callable view
    ▼
(HttpRequest) => HttpResponse
    │
    └── Callable
```

---

## 13. Nominal delegates

Nominal delegates also implement `Callable`.

For example:

```raven
delegate Handler(request: Request) -> Response
```

has exactly one structural Function type:

```raven
(Request) => Response
```

Conceptually:

```text
Callable
    ▲
    │
Handler
    │
    └── FunctionType
            │
            ▼
      (Request) => Response
```

Therefore:

```raven
val handler = Handler(HandleRequest)

val callable: Callable = handler
```

is valid.

The nominal type remains `Handler`.

The structural callable contract remains:

```raven
(Request) => Response
```

and is available through:

```raven
callable.FunctionType
```

---

## 14. Relationship to nominal delegates

A nominal delegate and its structural Function type can both satisfy `Callable` independently.

For example:

```raven
delegate Handler(request: Request) -> Response
```

produces the conceptual relationships:

```text
                 Callable
                   ▲   ▲
                   │   │
                   │   │
(Request) => Response  Handler
         structural     nominal
```

Both represent exactly one callable signature.

Their type identities remain different.

`Callable` provides only their common callable abstraction.

---

## 15. Relationship to `Delegate`

`Callable` fills the general role served by `System.Delegate`, but the models differ.

Conceptually:

```text
.NET

              Delegate
                 ▲
        ┌────────┼────────┐
        │        │        │
   Action<T>  Func<T,R>  CustomDelegate


neoCLR

              Callable
              interface
                 ▲
        ┌────────┼────────┐
        │        │        │
    (T) => () (T) => R   Handler
    structural structural nominal
```

The CLR places all delegates into a common nominal class hierarchy.

neoCLR instead provides a common interface across structural and nominal callable types.

This better reflects the fact that callable signatures are first-class structural types.

---

## 16. Why an interface

`Callable` should be an interface rather than a common base class.

Structural Function types are a distinct category of type and should not need to pretend to be ordinary classes merely to participate in a common callable abstraction.

Likewise, nominal callable types should not consume their class inheritance relationship merely to become callable.

An interface allows:

```text
Structural Function type ──┐
                           │
Nominal delegate ──────────┼──> Callable
                           │
Other eligible type ───────┘
```

without imposing a shared object hierarchy.

This keeps `Callable` focused on capability rather than representation.

---

## 17. Introspection

`Callable` provides a natural bridge from type-erased callable values into neoCLR introspection.

For example:

```raven
func Inspect(callback: Callable) {
    val type = callback.FunctionType

    // type.Parameters
    // type.ReturnType
}
```

Conceptually:

```text
Callable
    │
    └── FunctionType
            │
            ▼
      FunctionTypeInfo
            │
            ├── Parameters
            ├── ReturnType
            └── InvokeMethod
```

This is particularly useful when the caller knows only that a value is callable but still needs to reason about its exact signature.

---

## 18. Referenced executable

Function/delegate values may additionally expose the executable declaration to which they are bound:

```raven
callback.Function
```

This remains separate from `Callable.FunctionType`.

The two answer different questions:

```text
FunctionType
    "What callable signature does this value represent?"

Function
    "What executable declaration is this value bound to?"
```

For example:

```raven
func Foo(value: int) -> string

val foo = Foo
```

conceptually exposes:

```text
foo
    │
    ├── FunctionType
    │      └── (int) => string
    │
    └── Function
           └── MethodInfo(Foo)
```

The first belongs to the common `Callable` abstraction.

The second belongs to callable bindings that reference inspectable executable declarations.

---

## 19. No universal invocation through `Callable`

Because `Callable` intentionally erases the static signature, ordinary typed invocation cannot occur directly through the interface.

For example:

```raven
val callable: Callable = GetUser
```

does not provide enough compile-time information for:

```raven
callable(42)
```

to be statically type-safe.

The consumer must instead:

- preserve a concrete Function type,
- cast or match to an expected Function type,
- or perform dynamic invocation using `FunctionType`.

This is analogous to accepting an arbitrary `Delegate`: the common abstraction is useful for storage, inspection, matching, and dynamic dispatch, while strongly typed invocation requires the concrete callable type.

---

## 20. Design principles

### One `Callable`, one signature

Every `Callable` has exactly one authoritative `FunctionType`.

### Preserve signatures after type erasure

Passing:

```raven
(T) => R
```

as:

```raven
Callable
```

must erase the static signature without losing its runtime representation.

### Function types remain authoritative

`Callable` exposes the existing `FunctionTypeInfo`; it does not introduce a parallel signature model.

### Do not equate callability with `Callable`

Languages remain free to support duck-typed or overloaded invocation beyond the platform interface.

### Do not impose a base class

Structural and nominal callable types share an interface, not an object hierarchy.

### Keep typed invocation typed

`Callable` is intended for storage, matching, introspection and dynamic invocation. Concrete Function types remain the normal mechanism for statically typed invocation.

---

## 21. Result

`Callable` provides the common abstraction needed once neoCLR evolves delegates from exclusively nominal types into structural Function types.

The model becomes:

```text
Executable declaration
        │
        │ has
        ▼
   FunctionType
        │
        │ instantiated/bound as
        ▼
   Callable value
        │
        ├── FunctionType
        ├── Invoke(...)
        └── optionally Function
```

Across different callable kinds:

```text
                       Callable
                      (interface)
                          ▲
              ┌───────────┼───────────┐
              │           │           │
         (T) => R     (A, B) => C   Handler
         structural    structural    nominal
```

The central invariant is:

> **`Callable` represents a value with exactly one authoritative callable signature, exposed as a `FunctionTypeInfo`.**

This gives neoCLR the general-purpose abstraction that `Delegate` provides in .NET while preserving the structural nature of Function types and avoiding a mandatory callable class hierarchy.

It also creates a clean separation between platform-level callable values and broader language-level invocation rules: neoCLR provides `Callable` and Function types, while languages such as Raven remain free to support richer callable-object and duck-typing semantics on top.
