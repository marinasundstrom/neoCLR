# Typed equality

System.Equatable<T> declares an ordinary generic interface:

```text
.interface System.Equatable<T>
    .method instance Equals(T other) -> Boolean
    .end
.end
```

Its shape follows [.NET IEquatable<T>](https://learn.microsoft.com/en-us/dotnet/api/system.iequatable-1),
with neoCLR's interface naming convention. T specifies the type being compared;
it need not be the implementing type. Type arguments are invariant under the
current interface model. Implementations supply an exact public instance method.
There is no implicit implementation based on a method name alone.

The receiver uses existing value-receiver semantics and other is a value parameter.
Interface callers explicitly borrow the receiver slot as Equatable<T>&. Dispatch
copies the concrete receiver into the implementation; it does not box it or transfer
ownership. This contract does not introduce readonly references or an exclusive
borrow rule. Implementations should compare values without observable mutation.

## Implementations

- System.Int32 compares integer values.
- System.String compares exact text, with no case folding or Unicode normalization.
- System.Type compares descriptor identity using its existing Equals method.

The [sample](../examples/equatable.neoil) defines Point equality by comparing both
coordinates, passes an Equatable<Point>& to a free function, and also compares
primitive Int32 and String values through the same interface pattern.

```text
.local Int32 number
ldc.i4 42
stloc number
ldloca number
interface.borrow System.Equatable<Int32>
ldc.i4 42
callvirt instance System.Equatable<Int32>::Equals(Int32)
```

Illustrative Raven-like source:

```text
func SamePoint(left: Equatable<Point>&, right: Point) -> Boolean {
    return left.Equals(right)
}

let point = Point(2, 3)
let equal = SamePoint((&point) as Equatable<Point>&, Point(2, 3))
```

The source syntax is explanatory; there is no high-level compiler yet. In neoIL,
the method signature selects Equals(Point), and the explicit interface view selects
the concrete implementation.

## Equality contract and scope

For same-type equality, implementations should be reflexive, symmetric and transitive,
and return consistent results while the compared state is unchanged. The VM verifies
method signatures and dispatch compatibility, not these mathematical properties.
For comparisons between different types, document the intended relation explicitly.

Implementing Equatable does not change ceq, create equality operators, or cause
collections to discover a comparer automatically. It also does not add Object.Equals,
hashing, ordering or a generic constraint mechanism. Future hash-based collections
will need a compatible hash contract: values considered equal must have equal hashes.
Those facilities can be designed separately without broadening this one-method interface.

```sh
cargo run --locked -- verify examples/equatable.neoil
cargo run --locked -- run examples/equatable.neoil
cargo test --locked --test equatable
```

The sample prints true, false, true, true, then the CLI reports `=> Void`.
The automated walkthrough also assembles and executes its serialized artifact.
