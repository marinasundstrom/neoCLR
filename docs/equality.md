# Typed equality

System.Equatable<T> declares an ordinary generic interface:

```text
.interface System.Equatable<T>
    .method instance readonly byref Equals(T other) -> Boolean
    .end
.end
```

Its shape follows [.NET IEquatable<T>.Equals](https://learn.microsoft.com/en-us/dotnet/api/system.iequatable-1.equals?view=net-10.0),
with neoCLR's naming convention. T specifies the compared type; it need not be the
implementing type. Arguments are invariant. Implementations declare conformance and
provide matching receiver, parameter and return contracts.

The receiver is now a readonly managed reference, consistently with Comparable<T>.
Other is passed by value; T may itself be an explicit managed reference. Dispatch
uses the actual receiver storage without a mandatory whole-receiver copy or boxing.
Readonly prevents writes through this receiver and its owned projections; it does
not freeze other aliases or references held in fields. Equality implementations
should avoid observable mutation and remain consistent while compared state is stable.

## Neo and IL implementations

```swift
record Point(X: int, Y: int): System.Equatable<Point> {
    readonly func Equals(other: Point) -> bool {
        return this.X.Equals(other.X) && this.Y.Equals(other.Y)
    }
}

func SamePoint(left: readonly System.Equatable<Point>&, right: Point) -> bool {
    return left.Equals(right)
}
```

Int32 compares integer values, String compares exact text without case folding or
Unicode normalization, and Type compares canonical descriptor identity. Their direct
Equals methods use the same readonly managed receiver as interface dispatch.

```text
.local Int32 number
ldc.i4 42
stloc number
ldloca number
interface.borrow System.Equatable<Int32>
ldc.i4 42
callvirt instance System.Equatable<Int32>::Equals(Int32)
```

The [IL sample](../examples/equatable.neoil) implements Point equality and prints
true, false, true, true. The [Neo predicate-search sample](../examples/source/predicate-search.neo)
uses custom equality in an ordinary ArrayList operation.

## CLR comparison and migration

Revised 2026-09-08. .NET's IEquatable<T> describes an equality method, not a universal
readonly receiver requirement. C# [readonly struct members](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/struct#readonly-instance-members)
are a language mechanism for declaring nonmutating receiver access. NeoCLR instead
uses its existing runtime-checked readonly managed receiver contract uniformly for
this interface, whether the object resides in a frame or on the heap.

Keeping the former value receiver would preserve old IL but require copying the
receiver and would not match ordinary readonly Neo instance methods. Keeping parallel
old/new interface implementations would add two contracts to maintain. During preview,
we choose one readonly receiver contract. This removes the mandatory receiver copy;
it does not eliminate argument copies or imply a measured performance improvement.
A separate by-reference argument/comparer strategy remains a later API decision.

This is a **breaking preview library change**. Update Equatable implementations to
`instance readonly byref`; Neo uses `readonly func Equals`. Direct IL callers of
Int32.Equals, String.Equals and Type.Equals must pass a managed receiver address
instead of an owned value (for example ldloca rather than ldloc). Rebuild old modules;
old value-receiver implementations fail conformance checks. Neo automatically borrows
addressable receivers and materializes readonly temporaries where required.

The host snapshot invocation API does not import managed references. A host wishing
to call these members can invoke a guest wrapper accepting owned arguments, borrow
its receiver argument with ldarga, then call Equals. Host tests demonstrate that
boundary. Service planning now reports slot-reference services for direct equality.
No opcode or artifact format changes are introduced.

The pinned [.NET comparison probe](experiments/common-interfaces-dotnet/Program.cs)
checks typed equality and ordinary List predicate searches on SDK 10.0.100/net10.0.
Neo tests verify direct/interface equality, readonly receiver enforcement even when
IL verification is skipped, and rejection of the previous receiver contract.

## Scope

For same-type equality, implementations should be reflexive, symmetric and transitive.
The runtime checks signatures, access and dispatch, not these mathematical laws.
Cross-type implementations must document their relation explicitly.

Equatable does not change ceq, introduce equality operators or select a default
collection comparer. [Predicate searches](predicate-search.md) use a caller-supplied
function, which may call Equals. There is no mandatory Object.Equals fallback, hash
contract or generic constraint added here. Hash-based containers must wait for a
compatible equality/hash design: equal keys need equal hashes. LINQ is future work.

```sh
cargo run --locked -- verify examples/equatable.neoil
cargo run --locked -- run examples/equatable.neoil
cargo run --locked -- run examples/source/predicate-search.neo
cargo test --locked --test equatable --test predicate_search
```
