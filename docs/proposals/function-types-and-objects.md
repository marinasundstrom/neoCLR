# neoCLR Function Types and Function Objects

## Status

**Proposal**

## Summary

neoCLR should introduce **function types** and **function objects** as first-class runtime concepts.

Functions and methods will have function types that describe their callable signatures. References to functions will be represented by invocable function objects rather than by `Delegate` instances.

This separates three concepts that are combined by the .NET delegate model:

1. The type of a callable function.
2. The runtime representation of a particular function.
3. A lightweight, type-safe pointer or reference to callable code.

neoCLR will therefore not adopt the .NET `Delegate` and `MulticastDelegate` object hierarchy.

Delegates, if retained as a concept, will instead represent type-safe pointer-like references suitable for low-level scenarios and interoperability. They will not be the general-purpose container for managed function references.

Multicast invocation will not be intrinsic to functions or delegates. It can instead be provided through explicit collection or event abstractions.

This model provides a stronger foundation for first-class functions, introspection, closures, method binding, dynamic dispatch, and interoperability with dynamic languages and other runtimes.

---

# Motivation

The .NET delegate model combines several responsibilities.

A delegate simultaneously represents:

- a callable signature;
- a reference to executable code;
- an optional target object;
- a runtime object;
- a mechanism for method binding;
- and, through `MulticastDelegate`, an invocation list.

Function signatures are consequently represented indirectly through generated delegate types such as:

```csharp
Func<int, int, string>
```

rather than as types in their own right.

This model made sense for the CLR's original object-oriented type system, but neoCLR does not need to preserve this restriction.

Functions should instead be directly represented by the type system and runtime.

For example:

```raven
(int, int) -> int
(string) -> Result<int, ParseError>
() -> void
```

should be genuine types.

A function declaration:

```raven
func add(x: int, y: int) -> int {
    x + y
}
```

therefore has the function type:

```raven
(int, int) -> int
```

without requiring a generated delegate class to describe that signature.

---

# Goals

The function model should:

- make function signatures first-class types;
- make functions first-class runtime values;
- support direct invocation of function values;
- represent methods and standalone functions using a common underlying model;
- support bound and unbound methods;
- support closures;
- expose function types through introspection;
- avoid requiring generated delegate classes;
- distinguish managed function objects from low-level callable pointers;
- support efficient static invocation without requiring runtime allocation;
- provide a natural interoperability model for dynamic languages;
- allow future dynamic dispatch mechanisms to participate in the same callable model.

The model should not require every function invocation to perform dynamic dispatch or allocate a function object.

---

# Function Types

A **function type** describes the callable contract of a function.

For example:

```raven
(int, int) -> int
```

describes a function accepting two `int` values and returning an `int`.

Function types participate directly in the type system.

For example:

```raven
let operation: (int, int) -> int = add
```

Functions can therefore be accepted and returned without introducing nominal delegate declarations:

```raven
func transform(
    value: int,
    transformation: (int) -> string
) -> string
```

Function types may participate in generic types:

```raven
List<(int) -> string>
```

and other compound types:

```raven
Option<(Request) -> Response>
```

The exact variance rules for function parameters and return values are a separate type-system concern.

---

# Function Objects

A **function object** represents a particular callable function at runtime.

Conceptually, it may expose information such as:

```text
Function
    Name
    Type
    Module
    EntryPoint
    Binding
    Metadata
```

A function object remains directly invocable:

```raven
let operation = add

let result = operation(10, 20)
```

Its function type is:

```raven
(int, int) -> int
```

The distinction is important:

> A function type describes what can be invoked. A function object identifies what will be invoked.

Function objects therefore provide runtime identity and behavior, while function types provide static callable contracts.

---

# Functions and Methods

Standalone functions and methods should share the same fundamental runtime representation.

Consider:

```raven
class Calculator {
    func add(x: int, y: int) -> int {
        x + y
    }
}
```

The underlying unbound method can conceptually be represented as:

```text
(Calculator, int, int) -> int
```

The receiver is part of what is required to invoke the function.

Binding a receiver:

```raven
let operation = calculator.add
```

produces a callable value with the effective function type:

```text
(int, int) -> int
```

The resulting function object contains or otherwise represents the receiver binding.

This general mechanism can support:

- instance methods;
- static methods;
- module functions;
- local functions;
- extension functions;
- closures;
- runtime-generated functions.

The runtime therefore does not need a fundamentally different callable representation merely because a function originated as a method.

---

# Bound and Unbound Functions

neoCLR should distinguish between the underlying function and any environment required for its invocation.

An unbound method may require an explicit receiver:

```text
Calculator.add
    (Calculator, int, int) -> int
```

Binding it:

```raven
calculator.add
```

produces:

```text
(int, int) -> int
```

The same concept generalizes naturally to closures.

For example:

```raven
let offset = 10

let addOffset = (x: int) => x + offset
```

`addOffset` has the function type:

```text
(int) -> int
```

while its function object additionally carries the environment necessary to provide `offset`.

The callable type remains independent of the implementation of that environment.

---

# Function Objects Are Semantic, Not Allocation Requirements

The existence of function objects must not imply that every function declaration or function call requires a heap allocation.

For example:

```raven
let result = add(1, 2)
```

may compile directly to a call instruction.

Likewise:

```raven
process(items, transform)
```

may use a direct function address when `transform` is statically known.

The specification should therefore distinguish the semantic model from its physical representation:

> Function objects define the runtime semantics of first-class callable values. Implementations are not required to materialize function objects when function identity, bindings, and invocation targets can be represented more efficiently.

Possible implementations may include:

- direct calls;
- function pointers;
- stack values;
- compact function references;
- closure objects;
- dynamically dispatched callable objects.

This leaves the JIT, AOT compiler, and runtime free to optimize common cases.

---

# Delegates

neoCLR should not adopt the .NET `Delegate` abstraction as its general representation of callable values.

If delegates remain part of neoCLR terminology, they should instead represent **type-safe pointer-like callable references**.

Conceptually:

```text
Function
    ↓
Function reference
    ↓
Delegate / callable pointer
```

A delegate could be useful where a stable, constrained representation of a callable target is required, particularly for:

- native interoperability;
- ABI boundaries;
- callbacks;
- runtime interoperability;
- low-level APIs.

A delegate would not be responsible for:

- representing all first-class functions;
- providing the function type system;
- carrying arbitrary invocation lists;
- serving as the primary reflection representation of methods.

This also means that a function object and a delegate are not necessarily interchangeable.

A function object may contain runtime state that cannot be represented by a simple delegate.

Conversion or projection into a delegate can therefore be explicit and subject to runtime or ABI constraints.

---

# Delegate and MulticastDelegate

neoCLR should not provide equivalents of the .NET:

```text
System.Delegate
System.MulticastDelegate
```

hierarchy.

In particular, multicast behavior should not be an intrinsic property of callable references.

The .NET behavior:

```csharp
handler += first;
handler += second;
```

implicitly constructs an invocation list inside the delegate.

This combines two separate concepts:

1. a callable value;
2. a collection of callable values.

neoCLR should model these independently.

---

# Multicast Invocation

Where multicast behavior is required, it should be represented explicitly using a collection-oriented abstraction.

Conceptually:

```raven
let handlers = FunctionList<(Event) -> void>()

handlers.Add(first)
handlers.Add(second)

handlers.Invoke(event)
```

The exact abstraction could eventually be named:

```text
FunctionList<T>
InvocationList<T>
HandlerList<T>
Signal<T>
```

depending on its intended semantics.

An event system may build upon this abstraction rather than requiring every function reference to support multicast behavior.

Making multicast explicit also allows different collection types to define different semantics around:

- ordering;
- removal;
- duplicate handlers;
- concurrency;
- async invocation;
- cancellation;
- errors;
- results.

These concerns should not be baked into the fundamental representation of a function.

---

# Introspection

Function types should be visible through `System.Introspection`.

A function should expose information describing both its identity and callable type.

Conceptually:

```text
FunctionInfo
    Name
    FunctionType
    Parameters
    ReturnType
    Module
    Attributes
```

A corresponding function type could expose:

```text
FunctionTypeInfo
    ParameterTypes
    ReturnType
    CallingConvention
```

Methods may continue to expose `MethodInfo`, but their callable representation can be related directly to a function.

Conceptually:

```text
MethodInfo
    DeclaringType
    Function
    ...
```

This prevents `MethodInfo` from having to serve as the runtime's closest approximation of a function.

It also provides meaningful introspection for functions that are not members of classes.

---

# Invocation

Function values should be directly invocable.

For example:

```raven
let operation: (int, int) -> int = add

operation(1, 2)
```

The invocation syntax should not expose whether the runtime ultimately performs:

- a direct call;
- an indirect call;
- virtual dispatch;
- interface dispatch;
- closure invocation;
- dynamic dispatch;
- foreign-runtime dispatch.

The callable contract remains the function type.

This creates a common invocation model while allowing multiple runtime implementations underneath it.

---

# Dynamic Dispatch

First-class function objects provide an important foundation for future dynamic dispatch.

A dynamic runtime can expose something callable without manufacturing a CLR-style delegate subclass.

For example, another language runtime may provide a callable whose effective signature is:

```text
(dynamic, dynamic) -> dynamic
```

or a callable whose signature is discovered at runtime.

The invocation system can determine how the function should be dispatched while still presenting it as a callable runtime value.

This allows neoCLR to support both:

```text
statically typed function
        │
        ├── direct invocation
        ├── indirect invocation
        └── optimized invocation

dynamic callable
        │
        └── runtime dispatch
```

through a shared conceptual model.

---

# Language and Runtime Interoperability

Function objects provide a natural boundary between neoCLR and other language runtimes.

A foreign runtime can expose callable objects through a projection rather than translating every callable into a nominal neoCLR delegate type.

Likewise, neoCLR functions can be projected into foreign runtimes according to their capabilities.

For example, a dynamic language could treat a neoCLR function object as its ordinary callable value.

A native runtime could instead request projection into an ABI-compatible delegate or function pointer.

This distinction is important:

```text
Managed/Dynamic Interop
        Function Object
              │
              └── rich callable semantics

Native/ABI Interop
        Function Object
              │
              ▼
        Delegate / Function Pointer
```

The runtime can therefore preserve richer semantics where possible without forcing those semantics into low-level ABI representations.

---

# Closures

Closures naturally become function objects carrying an associated environment.

For example:

```raven
func createAdder(offset: int) -> (int) -> int {
    return x => x + offset
}
```

The returned value has type:

```text
(int) -> int
```

Its runtime representation may conceptually contain:

```text
Function
    Code: closure implementation
    Environment:
        offset
```

The environment representation remains an implementation detail.

A compiler may eliminate the closure object entirely where escape analysis or other optimization demonstrates that it is unnecessary.

---

# Function Identity

Function identity should not automatically imply object identity.

Two function values may reference:

- the same underlying function;
- the same function with different receivers;
- the same function with different closure environments;
- dynamically generated functions;
- projected foreign callables.

Consequently, equality semantics for function objects should not simply inherit ordinary reference equality assumptions.

Whether function values support equality at all should be explicitly defined rather than inherited accidentally from an object hierarchy.

---

# Proposed Conceptual Model

The resulting model is:

```text
                 Function Type
              (int, int) -> int
                       │
                       │ describes
                       ▼
                    Function
              runtime callable value
                       │
          ┌────────────┼────────────┐
          │            │            │
       Direct       Bound        Closure
      function      method        function
          │            │            │
          └────────────┼────────────┘
                       │
                    Invoke
                       │
          ┌────────────┴────────────┐
          │                         │
     Static/managed            Dynamic/foreign
       dispatch                   dispatch
```

Where required, a function may additionally be projected into:

```text
Function
   │
   ▼
Delegate / FunctionRef
   │
   ▼
ABI-compatible callable reference
```

Multicast behavior exists separately:

```text
Collection<Function>
        │
        ▼
Invocation/Event abstraction
```

---

# Consequences

## Advantages

This model makes functions genuine members of the neoCLR type system rather than objects modeled indirectly through delegate subclasses.

It provides:

- structural function types;
- first-class function values;
- natural closure representation;
- unified treatment of functions and methods;
- explicit method binding;
- richer introspection;
- cleaner native interoperability;
- cleaner dynamic-language interoperability;
- separation between callable values and multicast collections;
- greater freedom for JIT and AOT optimization.

Most importantly, it establishes a callable model that does not assume every language targeting neoCLR has the same object-oriented method semantics.

## Compatibility Cost

The design intentionally diverges from the CLR.

APIs depending directly on:

```text
Delegate
MulticastDelegate
Action<T>
Func<T>
```

cannot simply be projected one-to-one into neoCLR's native type system.

Interop layers may therefore need to translate between CLR delegates and neoCLR function values.

For .NET interoperability, a CLR delegate can naturally be projected as a neoCLR function object, while projection in the opposite direction may require creation of an appropriate CLR delegate.

This cost is acceptable because reproducing the CLR delegate hierarchy would permanently constrain the neoCLR callable model around historical CLR semantics.

---

# Design Principle

The fundamental distinction is:

> **Function types describe callable contracts. Function objects represent callable functions. Delegates, where required, represent constrained type-safe references to callable code. Collections represent multicast behavior.**

These concepts should remain independent.

This allows neoCLR to treat functions as genuine runtime entities while retaining efficient low-level representations and leaving room for future dynamic dispatch and cross-runtime interoperability.

---

# Open Design Questions

Several details should be specified separately.

### Function type variance

Determine whether parameter types are contravariant and return types covariant, and how such conversions interact with Raven's general conversion model.

### Function equality

Determine whether function objects support equality and, if so, whether identity includes the bound receiver or closure environment.

### Function object hierarchy

Determine whether `Function` is represented as a normal runtime class, a special runtime type, an interface/capability, or primarily as an introspection abstraction.

The semantic concept should not require every function value to have an allocated `Function` object.

### Delegate terminology

Determine whether the low-level callable-reference concept should retain the familiar name `Delegate` or use a name that better distinguishes it from CLR delegates, such as:

```text
FunctionRef<T>
CallableRef<T>
```

### Calling conventions

Determine whether calling convention information belongs directly to function types or to projections such as delegates and native function pointers.

Ordinary managed function types should ideally remain independent of platform-specific ABI details.

### Partial application

Bound methods already imply a form of argument binding. A future design should determine whether generalized partial application uses the same runtime mechanism.

### Dynamic signatures

Determine how dynamically typed callable objects describe their signatures when complete parameter and return types are unavailable until invocation.

This should extend the function model rather than introduce an unrelated dynamic-call mechanism.

### Events and multicast

Design the collection/event abstraction independently from the function system, including ordering, async behavior, error propagation, cancellation, mutation during invocation, and concurrency.

---

# Recommendation

neoCLR should adopt function types and function objects as fundamental runtime concepts and should not reproduce the CLR `Delegate`/`MulticastDelegate` hierarchy.

Functions should be first-class callable runtime values with explicit function types visible through introspection. Methods, closures, and dynamically provided callables should participate in the same model.

A lightweight delegate or function-reference mechanism may still exist for cases requiring constrained pointer-like semantics, particularly native and runtime interoperability.

Multicast invocation should be modeled explicitly through collections or higher-level event abstractions rather than being an intrinsic feature of every callable reference.

This gives neoCLR a function model designed around the requirements of a modern multi-language runtime rather than inheriting the CLR's historical delegate abstraction.