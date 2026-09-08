# Common comparison and iteration contracts

Implemented 2026-09-08 in the System library, using ordinary interface metadata and
neoIL. No new opcode, host intrinsic, nominal reference-type category or Object base
is required. Neo uses explicit managed interface views and ordinary member/property
access, including inherited library interface members.

## Comparable

`System.Comparable<T>.CompareTo(T other) -> Int32` describes natural ordering:
negative means before, zero means equivalent in the ordering, positive means after.
Consumers must use the sign, not assume every implementation returns -1 or 1.
Implementations should provide a consistent, transitive ordering. Ordering equivalence
need not be reference identity. The receiver is a readonly managed reference; the compared T uses value semantics.
T can itself be an explicit managed reference. Unlike the older value-receiver
Equatable<T> contract, Comparable avoids a required receiver copy and matches Neo
readonly instance methods. Equatable remains unchanged pending a separate migration.
Large argument copies and comparer strategies remain separate API design decisions.

Boolean, Char, all signed/unsigned fixed-width integers, native-sized integers,
Single and Double now implement Comparable. Built-in results are -1, 0 or 1.
Integers use comparisons rather than subtraction, preserving extreme-value ordering;
unsigned types use unsigned comparisons. False precedes true. Floating-point ordering
matches .NET CompareTo: NaN precedes numbers, two NaNs compare equal, and signed zeros
compare equal. This is an ordering contract, not a change to floating-point `==`.
Neo records/classes can declare conformance to bundled interfaces, for example
`record Score(Value: int): System.Comparable<Score>` with a matching `readonly func CompareTo(other: Score) -> int` method.
String ordering is deferred until explicit ordinal/culture policies are designed;
Equatable and existing String equality remain unchanged.

## Iterable and Iterator

| Contract | Members and receiver behavior |
| --- | --- |
| System.Collections.Iterable<T> | readonly GetIterator() -> Iterator<T>& |
| System.Collections.Iterator<T> | mutable MoveNext() -> Boolean; readonly Current -> T; inherits Disposable.Dispose() |
| System.Collections.List<T> | now inherits Iterable<T>, retaining Count, Item and Add |

GetIterator creates an independent traversal. The cursor starts before the first
item. A successful MoveNext makes Current available. Exhaustion is sticky: repeated
MoveNext returns false. Current faults before the first item, after exhaustion, or
after Dispose. Dispose is explicit and idempotent; the bundled iterator treats it as
exhaustion. The protocol requires no null/default T and no duplicate object-valued
Current member. There is no Reset requirement. A readonly Iterator view can read
Current but cannot advance or dispose the cursor.

Current returns T: an ordinary value is copied; if T is Foo&, the copied value is a
managed reference to the same Foo. Neo accesses both through the same syntax.
Readonly access to the iterator does not turn references returned as T into readonly
references; use an explicitly readonly element type for that contract.

ArrayList.GetIterator works through a concrete reference, List<T>& or Iterable<T>&.
Its internal managed iterator retains the backing array and initial Count, never a
reference to the list descriptor. It can therefore outlive a stack-local descriptor.
The GC traces both the buffer and any reference elements. Dispose stops traversal;
the buffer remains retained until the iterator becomes unreachable. It does not
manually free the array or introduce nullable fields. Each iterator costs a heap
allocation, with no element-array copy.

This is an **extent and buffer capture**, not an immutable snapshot or a fail-fast
.NET List enumerator. Later additions are outside the captured extent. Writes to the
retained buffer are visible. Growth installs a new buffer on the list; an existing
iterator continues over the old one. This follows ArrayList's existing descriptor-copy
behavior. There is no concurrent-mutation/thread-safety guarantee.

## Neo example and commands

```swift
func Sum(values: readonly System.Collections.Iterable<int>&) -> int {
    let iterator = values.GetIterator()
    var total = 0
    while iterator.MoveNext() { total = total + iterator.Current }
    iterator.Dispose()
    return total
}
```

Run [the full sample](../examples/source/common-interfaces.neo), which prints 0 for
the comparison and 42 for the sum, then returns 42:

```sh
cargo run --locked -- run examples/source/common-interfaces.neo
cargo run --locked -- check examples/source/common-interfaces.neo
cargo run --locked -- debug examples/source/common-interfaces.neo
cargo test --locked --test common_interfaces
```

Iteration is expressed with while for now. Neo's `for` still accepts integer ranges;
there is no implicit foreach disposal, yield transformation or resource unwinding
added in this slice. Early exits must explicitly dispose resource-owning iterators.
The bundled array iterator owns only managed memory.

## .NET baseline, alternatives and tradeoffs

Primary sources consulted 2026-09-08:

- [IComparable<T>.CompareTo](https://learn.microsoft.com/en-us/dotnet/api/system.icomparable-1.compareto?view=net-10.0)
  supplies the sign-based ordering contract.
- [Double.CompareTo](https://learn.microsoft.com/en-us/dotnet/api/system.double.compareto?view=net-10.0)
  defines NaN and numeric ordering independently of operator equality.
- [IEnumerable<T>](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.ienumerable-1?view=net-10.0)
  separates obtaining a traversal from advancing it.
- [IEnumerator<T>](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.ienumerator-1?view=net-10.0)
  supplies typed Current and inherits IDisposable and nongeneric IEnumerator.
- [List<T>.GetEnumerator](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.list-1.getenumerator?view=net-10.0)
  documents invalidation after list modification.

These are library contracts; C# foreach adds language lowering and cleanup behavior.
Neo adopts the familiar MoveNext/Current protocol but uses the requested Iterable /
Iterator naming and GetIterator. Omitting legacy nongeneric duplication and Reset
reduces implementation requirements; it requires API translation from .NET and does
not provide Reset compatibility. A Next() -> Option<T> protocol would eliminate
invalid Current states, but would construct today's union carrier on each
step and diverge from the familiar cursor API. An Option adapter can be evaluated
later without replacing these interfaces. No performance claim is made.

The ArrayList iterator deliberately differs from .NET's modification-version checks.
Capturing only its buffer and extent supports the existing stack/value descriptor
without retaining a frame address or changing descriptor-copy semantics. A shared
version/owner object would add state and affect copied descriptors; copying every
element would cost O(n) storage and still share explicitly referenced elements. The
current bounded choice makes retention and mutation effects explicit. Revisit it
when collection ownership or concurrent traversal requirements change.

The [SDK 10.0.100/net10.0 comparison probe](experiments/common-interfaces-dotnet/Program.cs)
checks extreme/NaN ordering, independent cursors, exhaustion and .NET list mutation
invalidation. Run `dotnet run -c Release` in that directory. Neo regression tests
cover those applicable contracts plus retained buffers, escaped descriptors, GC,
reference elements and readonly enforcement.

Compatibility: existing List<T> implementers must add GetIterator. Library interface
methods are now projected through their declaring parent interface in Neo. The slice
also fixes generic static calls such as ArrayList<int>.Allocate being mistakenly
parsed as delegate-construction type names. Artifact format remains unchanged.
Future work includes managed-array adapters, foreach with guaranteed disposal,
resource-owning iterators, comparer strategies, sorting, string policies and variance.
