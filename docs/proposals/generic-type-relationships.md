# NeoCLR Generics and Type Relationships — Proposal

## 1. Motivation

NeoCLR should treat generics as a fundamental mechanism for expressing relationships between types rather than merely as parameterized containers.

The CLR already demonstrated the value of reified generics, but its generic model retains several historical restrictions and patterns that make some abstractions more complicated than they need to be.

NeoCLR should explore a more regular model built around three principles:

> **Types that have meaningful semantics should be usable as generic arguments without arbitrary special cases.**

> **Generic contracts should be able to express relationships between an operation and its result.**

> **The implementing type itself should be directly expressible through `self` rather than requiring recursive generic patterns.**

The initial areas of exploration are:

```text
void as a generic type argument
generic operation results
self types
numeric and mathematical contracts
collection transformation contracts
function and task composition
```

These features are related, but they do not need to be implemented as one indivisible feature.

---

# 2. `void` is a real type in generic contexts

NeoCLR should permit:

```raven
SomeType<void>
```

wherever substituting `void` has meaningful semantics.

`void` represents the absence of a produced value. It should not become unusable merely because that absence appears through a generic parameter.

For example:

```raven
Operation<void>
Operation<int>
Operation<String>
```

are all legitimate instantiations of the same generic abstraction.

This allows APIs to express “no result” directly instead of introducing artificial placeholder types.

---

# 3. Generic operations can produce `void`

Consider:

```raven
interface Operation<TResult>
{
    func Execute() -> TResult
}
```

NeoCLR should permit:

```raven
Operation<void>
```

with the corresponding member:

```raven
func Execute() -> void
```

There should be no need for:

```text
Unit
NoResult
Empty
None
```

solely to satisfy the generic type system.

This is especially useful when an abstraction describes an operation rather than a stored value.

---

# 4. Generic operation results

This becomes more useful when implementations of the same conceptual operation produce different kinds of results.

For example:

```raven
interface Addable<T, TResult>
{
    func Add(value: T) -> TResult
}
```

An in-place implementation may use:

```raven
Addable<T, void>
```

while another implementation could use:

```raven
Addable<T, SomeResult>
```

The contract describes the operation.

`TResult` describes what performing that operation produces.

This gives us several useful patterns:

```text
TResult = void
    the operation produces no value

TResult = self
    the operation produces another value of the implementing type

TResult = SomeType
    the operation produces an independent value

TResult = Result<T, E>
    the operation produces an explicit success/error result
```

These should compose naturally rather than requiring separate abstraction families.

---

# 5. Collection transformations

Collections provide one concrete motivation.

A generalized list contract could eventually express structural operations through a result parameter:

```raven
interface List<T, TChangeResult>
{
    func Add(value: T) -> TChangeResult
    func Insert(index: int, value: T) -> TChangeResult
    func RemoveAt(index: int) -> TChangeResult
    func Clear() -> TChangeResult
}
```

The conventional mutable list contract can then be:

```raven
interface List<T> : List<T, void>
{
}
```

An `ArrayList<T>` performs changes in place:

```raven
class ArrayList<T> : List<T>
{
    func Add(value: T) -> void
    {
        // mutate this instance
    }
}
```

A persistent collection can instead return another collection:

```raven
class ImmutableList<T> :
    List<T, ImmutableList<T>>
{
    func Add(value: T) -> ImmutableList<T>
    {
        // produce a new value
    }
}
```

The operation remains `Add`, while the result type communicates an important part of its semantics.

This should not imply that every collection operation must share one generic result type. If doing so produces types such as:

```raven
List<
    T,
    TAddResult,
    TRemoveResult,
    TInsertResult,
    TClearResult
>
```

the abstraction has probably been parameterized at the wrong level.

---

# 6. `self` as a type

Raven already uses the `self` keyword.

NeoCLR should investigate allowing the same concept to appear in type position to represent:

> **The concrete type implementing this contract.**

For example:

```raven
interface Addable<T>
{
    func Add(value: T) -> self
}
```

An implementation:

```raven
class Vector<T> : Addable<T>
```

would therefore effectively have:

```raven
func Add(value: T) -> Vector<T>
```

This is substantially clearer than encoding the relationship through an explicit recursive type parameter.

---

# 7. Avoid CRTP-style generic contracts

Without a `self` type, APIs frequently need patterns resembling:

```raven
interface Addable<T, TSelf>
    where TSelf : Addable<T, TSelf>
{
    func Add(value: T) -> TSelf
}
```

This makes an implementation repeat itself:

```raven
class Vector<T> :
    Addable<T, Vector<T>>
```

The generic parameter does not represent an independently variable type.

It represents information the runtime and type system already know:

```text
the implementing type
```

NeoCLR should therefore investigate representing that relationship directly:

```raven
interface Addable<T>
{
    func Add(value: T) -> self
}
```

This is both more expressive and harder to misuse.

---

# 8. `self` should work as a generic argument

If `self` is a genuine type expression, it should not be restricted to direct return positions.

For example:

```raven
interface PersistentList<T> :
    List<T, self>
{
}
```

should be meaningful.

Likewise:

```raven
Result<self, Error>
Task<self>
Option<self>
Array<self>
```

should be expressible wherever the resulting semantics make sense.

This is important because otherwise `self` becomes syntax attached specifically to method return types rather than a genuine part of the type system.

---

# 9. Mathematical and numeric contracts

Mathematics is one of the strongest motivations for `self`.

Numeric contracts frequently describe operations that are closed over the implementing type.

Conceptually:

```raven
interface Additive
{
    static func Add(
        left: self,
        right: self
    ) -> self
}
```

Similarly:

```raven
interface Multiplicative
{
    static func Multiply(
        left: self,
        right: self
    ) -> self
}
```

A numeric type can then implement these contracts naturally:

```raven
struct Decimal : Additive, Multiplicative
{
    static func Add(
        left: Decimal,
        right: Decimal
    ) -> Decimal

    static func Multiply(
        left: Decimal,
        right: Decimal
    ) -> Decimal
}
```

There is no need to write:

```raven
Additive<Decimal>
Multiplicative<Decimal>
```

when the only intended generic argument is “this implementing type.”

---

# 10. Generic mathematics becomes simpler

This could make generic mathematical algorithms considerably easier to describe.

For example:

```raven
func Sum<T>(values: Iterable<T>) -> T
    where T : Additive
{
    ...
}
```

The constraint already tells us that:

```raven
T + T -> T
```

because that relationship is part of `Additive`.

This is preferable to requiring every numeric capability to repeat a `TSelf` parameter.

More sophisticated mathematical contracts can build upon the same principle:

```raven
interface Additive
interface Subtractive
interface Multiplicative
interface Divisible
interface Negatable
interface Comparable
```

Higher-level concepts may then compose those capabilities:

```raven
interface Number :
    Additive,
    Subtractive,
    Multiplicative,
    ...
{
}
```

The exact numeric hierarchy is a separate proposal. The important point here is that `self` gives that proposal a much cleaner foundation.

---

# 11. Operations do not always have to be closed over `self`

NeoCLR should not assume that every mathematical operation returns the same type.

For example, there may be meaningful contracts such as:

```raven
interface Convertible<TResult>
{
    func Convert() -> TResult
}
```

or mixed-type arithmetic:

```raven
interface Multipliable<TOther, TResult>
{
    func Multiply(other: TOther) -> TResult
}
```

The type system should therefore support both:

```text
self relationships
```

and:

```text
explicit generic relationships
```

`self` eliminates unnecessary generic parameters. It does not replace generics.

---

# 12. Functions become more regular

Allowing `void` as a generic argument also raises the possibility of simplifying function abstractions.

Instead of requiring fundamentally separate concepts equivalent to:

```text
Action<T>
Func<T, TResult>
```

the type system could potentially describe both through:

```raven
Function<T, TResult>
```

where:

```raven
Function<T, void>
```

means a function that consumes `T` without producing a value, while:

```raven
Function<T, int>
```

produces an integer.

Whether NeoCLR actually exposes such a `Function` type is an API-design question.

The important point is that the type system should not force two abstraction families merely because one operation returns `void`.

---

# 13. Tasks and asynchronous operations

The same regularity applies to asynchronous operations.

If:

```raven
Task<T>
```

represents eventual completion with a value, then:

```raven
Task<void>
```

can naturally represent eventual completion without a produced value.

Likewise:

```raven
Task<Result<void, IOError>>
```

means:

> asynchronously perform an operation that produces no success value but may produce a recoverable `IOError`.

This fits NeoCLR's broader task and explicit-error model without requiring special non-generic task variants.

---

# 14. `void` is not necessarily a runtime value

Allowing:

```raven
Container<void>
```

does not necessarily mean that NeoCLR must permit ordinary values such as:

```raven
let x: void
```

or allocate storage for `void`.

There is an important distinction between:

```text
void is a valid type argument
```

and:

```text
void has ordinary inhabitable value semantics
```

NeoCLR can allow the former without committing to the latter.

The runtime may specialize or erase storage associated with `void` where appropriate.

The precise IL and runtime representation should be specified separately.

---

# 15. `self` is not an ordinary generic parameter

Likewise, `self` should not merely be compiler sugar for an invisible `TSelf` parameter unless that turns out to provide the correct runtime semantics.

It represents a relationship:

```text
self = the concrete implementing type
```

That relationship potentially affects:

- interface implementation;
- interface dispatch;
- inheritance;
- static interface members;
- generic constraints;
- variance;
- value types;
- interface/existential values;
- metadata representation.

These semantics need explicit design.

---

# 16. Open question: interface values and `self`

Consider:

```raven
interface Cloneable
{
    func Clone() -> self
}
```

and:

```raven
func Clone(value: Cloneable)
{
    let copy = value.Clone()
}
```

What is the static type of `copy`?

The implementation knows its concrete `self`, but the caller may only know `Cloneable`.

NeoCLR therefore needs rules for how self-typed members behave when invoked through an interface value.

Possible models include preserving an existential relationship, projecting the result to the interface type, or restricting particular uses of `self`.

This should be resolved before `self` becomes part of the runtime contract model.

---

# 17. Design principle

These features point toward a broader NeoCLR principle:

> **Do not introduce parallel abstractions merely to compensate for limitations in generic expressiveness.**

If:

```raven
Task<void>
```

naturally represents a task without a result, NeoCLR should not require a separate non-generic task abstraction solely for that reason.

If:

```raven
Function<T, void>
```

naturally represents a procedure, the type system should not force a separate function family.

If:

```raven
func Add(other: self) -> self
```

naturally describes a closed mathematical operation, the API should not need a recursive `TSelf` parameter simply to tell the type system what it already knows.

And if:

```raven
List<T, void>
```

naturally represents operations whose changes occur in place, `void` should not be prohibited merely because it appears as a generic argument.

---

# 18. Anti-goals

This proposal does **not** mean that every difference between APIs should be encoded through generic parameters.

NeoCLR should avoid types such as:

```raven
Collection<
    T,
    TAddResult,
    TRemoveResult,
    TInsertResult,
    TClearResult
>
```

unless those parameters represent genuinely useful independent dimensions.

Nor should `self` replace legitimate generic relationships.

The goal is not maximum genericity.

The goal is:

> **Use the type system to express meaningful relationships without artificial ceremony.**

---

# 19. Proposed direction

NeoCLR should investigate and, where runtime semantics permit, support:

1. `void` as a valid generic type argument.
2. Generic methods and interfaces whose substituted result type may be `void`.
3. Generic types such as `Task<void>` and `Result<void, E>` without parallel special-case abstractions.
4. Raven's existing `self` keyword in type position.
5. `self` as both a direct type and a generic type argument.
6. Self-typed instance and static interface members.
7. Numeric contracts based on `self` rather than CRTP-style `TSelf` parameters.
8. Generic operation-result contracts where different implementations naturally produce different results.

Collections, Tasks, Functions, Results, and Numerics can then build upon these capabilities without defining the underlying type-system rules themselves.

The result should be a more regular generic model:

```text
void
    expresses absence of a produced value

self
    expresses the implementing type

T
    expresses an independently variable type
```

Those three concepts cover fundamentally different relationships and should all be first-class participants in NeoCLR's generic type system.