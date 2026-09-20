### NeoCLR Generic Operation Results — Extension Proposal

**Problem.** Generic interfaces sometimes describe the same conceptual operation across implementations, while the appropriate return type depends on how that implementation performs the operation.

For example, two collections may both support `Add`, but with different semantics:

```raven
ArrayList<T>.Add(T) -> void
ImmutableList<T>.Add(T) -> ImmutableList<T>
```

Likewise, numeric and other self-referential contracts often need operations whose result is related to the implementing type.

**Runtime principle.** NeoCLR should allow `void` to participate as a normal generic type argument where semantically meaningful:

```raven
Operation<T, void>
Operation<T, SomeResult>
```

This removes the need for artificial marker/result types merely because a generic parameter cannot otherwise represent “no value.”

**Generic operation contracts.** This enables interfaces to parameterize the result of an operation:

```raven
interface Addable<T, TResult>
{
    func Add(value: T) -> TResult
}
```

A mutable implementation could then say:

```raven
class ArrayList<T> : Addable<T, void>
{
    func Add(value: T) -> void
    {
        // modifies this instance
    }
}
```

while a persistent implementation could say:

```raven
class ImmutableList<T> :
    Addable<T, ImmutableList<T>>
{
    func Add(value: T) -> ImmutableList<T>
    {
        // returns a new value
    }
}
```

The contract describes the **operation**, while the generic result describes what performing that operation produces.

That gives us three particularly useful result patterns:

```text
TResult = void
    operation has no resulting value

TResult = Self
    operation produces another value of the implementing type

TResult = SomeType
    operation produces some independent result
```

I would make **`Self` a related but separate part of the extension**. We should investigate whether NeoCLR can make this:

```raven
interface Addable<T>
{
    func Add(value: T) -> Self
}
```

a first-class construct rather than requiring the .NET-style self-referential pattern:

```raven
interface Addable<T, TSelf>
    where TSelf : Addable<T, TSelf>
```

That could simplify not just collections but numeric contracts and many other APIs.

The important thing is that this proposal shouldn't prescribe that *all* modification interfaces must be parameterized this way. Something like `MutableList<T>` may still be the clearer public capability. This extension simply gives platform/API designers a more expressive tool when **the operation is common but its result semantics vary by implementation**.

And I'd explicitly state one anti-goal: we don't want types turning into monstrosities like:

```raven
Collection<T, TAddResult, TRemoveResult, TInsertResult, TClearResult>
```

If an abstraction requires that, we've probably parameterized at the wrong level. Smaller operation contracts, associated types, `Self`, or separate capabilities would be preferable.

So yes: **keep this out of Collections v1 itself.** The Collections proposal can mention that NeoCLR's generic-contract extensions may later allow persistent and mutable operations to share certain contracts, but it shouldn't depend on that work. This deserves its own design exploration because it has platform-wide implications well beyond collections.