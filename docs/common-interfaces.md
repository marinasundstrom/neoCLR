# Common comparison and iteration contracts

Implemented 2026-09-08 in the System library, using ordinary interface metadata and
neoIL. No new opcode, host intrinsic, nominal reference-type category or Object base
is required. Neo uses explicit managed interface views and ordinary member/property
access, including inherited library interface members.

## ComparableTo

`System.ComparableTo<T>.CompareTo(T other) -> Int32` describes natural ordering:
negative means before, zero means equivalent in the ordering, positive means after.
Consumers must use the sign, not assume every implementation returns -1 or 1.
Implementations should provide a consistent, transitive ordering. Ordering equivalence
need not be reference identity. The receiver is a readonly managed reference; the compared T uses value semantics.
T can itself be an explicit managed reference. [EquatableTo<T>](equality.md) now uses
the same readonly receiver contract; both avoid a mandatory whole-receiver copy and
match Neo readonly instance methods. See the equality guide for migration details.
Large argument copies and comparer strategies remain separate API design decisions.

Boolean, Char, all signed/unsigned fixed-width integers, native-sized integers,
Single and Double now implement ComparableTo. Built-in results are -1, 0 or 1.
Integers use comparisons rather than subtraction, preserving extreme-value ordering;
unsigned types use unsigned comparisons. False precedes true. Floating-point ordering
matches .NET CompareTo: NaN precedes numbers, two NaNs compare equal, and signed zeros
compare equal. This is an ordering contract, not a change to floating-point `==`.
Neo records/classes can declare conformance to bundled interfaces, for example
`record Score(Value: int): System.ComparableTo<Score>` with a matching `readonly func CompareTo(other: Score) -> int` method.
String ordering is deferred until explicit ordinal/culture policies are designed;
String equality still compares exact text; its receiver now follows the readonly equality contract.

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
iterator continues over the old one. This iterator policy is independent of ArrayList wrapper assignment, which
now shares the complete list state. There is no concurrent-mutation/thread-safety guarantee.

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
Capturing only its buffer and extent avoids retaining a frame address. ArrayList now
has shared managed state, so a version-checking iterator is technically possible;
this slice preserves the existing traversal contract rather than silently changing
it with assignment semantics. Copying every element would cost O(n) storage and still
share reference elements. Revisit mutation invalidation as a separate API decision.

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

## Directional interface names (development, 2026-09-25)

EquatableTo<T> and ComparableTo<T> replace Equatable<T> and Comparable<T>.
Equals(T) and CompareTo(T), including readonly receiver behavior, are unchanged.
This is a source and metadata identity break: update implementations, constraints,
interface views and record Runtime Contract configuration, then rebuild consumers
with matching compiler references and runtime library. No old-name aliases remain.
The .NET IEquatable<T>/IComparable<T> comparisons above still apply; the naming
change makes the relation to the other operand explicit, at the cost of migration.

System.ConvertibleInto<T> is a new invariant interface with `func Convert() -> T`.
It follows the method spelling in the existing generic-type-relationships proposal.
Its ordinary borrowed receiver may mutate; implementations choose conversion and
failure policy. There are no built-in implementations, implicit conversions,
format-provider rules or return-type-directed overload selection. Call through an
explicitly selected interface when a concrete type exposes multiple conversions.

.NET 10's [IConvertible](https://learn.microsoft.com/en-us/dotnet/api/system.iconvertible?view=net-10.0)
is non-generic, with conversions to standard types and a Type-based ToType method,
usually accepting IFormatProvider (retrieved 2026-09-25). This smaller ordinary
library contract selects one result type statically; it trades that broad conversion
protocol for a typed result and leaves culture/error conventions to implementers.
Keeping no conversion interface or adopting IConvertible were alternatives; the
author explicitly requested this generic capability. No new VM or compiler rule
is needed. Checked IL interface dispatch and bridge signature rejection validate
the contract; broader conversion policies remain open.
