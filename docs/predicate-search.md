# Eager ArrayList predicate searches

Implemented 2026-09-08 as ordinary platform IL on System.Collections.ArrayList<T>.
These are concrete collection methods, analogous to .NET List<T> members. LINQ,
deferred queries and query pipelines remain future work.

| Method | Result |
| --- | --- |
| Find(Func<T,Boolean> match) | Option<T>: Some(first matching value), otherwise None |
| FindIndex(Func<T,Boolean> match) | Zero-based first matching index, otherwise -1 |
| Exists(Func<T,Boolean> match) | Whether an element matches |

All three use readonly managed receivers. They test elements in ascending index
order and stop at the first match. Empty lists do not invoke the predicate. Predicates
must return Boolean; incompatible delegates are rejected by normal signature checks.
Neo wraps a matching method group or lambda in the existing Func family. No separate
Predicate delegate, equality discovery, comparer default or generic constraint is
introduced. These methods are on ArrayList, not additional List interface obligations.

A value element is passed to the predicate by value. If T is Foo&, that value is a
managed reference: the predicate can access the same Foo and Find returns that same
reference in Some. No explicit dereference is needed in Neo. Receiver readonly does
not make reference elements readonly; use a readonly element type when required.
Find retains the exact value passed to the successful predicate, even if the predicate
replaces that position in the list. Reference-valued results still observe mutations
to their referent. None never initializes or reads a default T.

The implementation uses the existing ArrayList iterator and therefore captures the
initial backing buffer and Count. Changes to the retained buffer are visible to later
reads; growth does not extend the search or switch its buffer. There is no version
check. See [iteration contracts](common-interfaces.md) for the descriptor/copy model.
Each search creates one managed iterator, disposes it on both successful and exhausted
normal returns, and releases the frame's references on return. Callback faults retain
the ordinary guest stack trace and terminate the run; no exception unwinding or
guaranteed resource cleanup on faults is added. This iterator owns only managed memory.

Search takes O(n) predicate calls in the worst case. Find constructs an Option carrier;
Exists and FindIndex do not. Value copies, delegate/capture allocations and the iterator
allocation are real costs; no performance comparison is claimed.

## .NET comparison and decision

Primary sources consulted 2026-09-08:

- [.NET List<T>.Find](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.list-1.find?view=net-10.0)
  supplies the eager, first-match predicate pattern but returns default(T) on absence.
- [List<T> API](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.list-1?view=net-10.0)
  includes FindIndex and Exists and describes automatic equality-comparer selection
  for other methods such as Contains.
- [Typed equality](equality.md) records the Equatable/CLR receiver comparison and
  the separate need for a consistent future equality/hash strategy.

Keeping default(T) would make zero indistinguishable from absence and cannot supply a
valid default for nonnullable references. Returning Option<T> represents both cases
without changing element nullability; its cost is carrier construction and a different
return API from .NET. FindIndex retains the familiar -1 sentinel because it is an
ordinary Int32 result, not an invalid element/reference. The caller explicitly chooses
the predicate, so arbitrary record/reference types work without inventing a default
Object hierarchy or comparer policy.

The existing Func<T,Boolean> replaces .NET Predicate<T>, following neoCLR's one-family
callable decision. A future comparer strategy can support Contains, ordering and hash
containers after its contracts are established; it is not needed to add these three
concrete operations. LINQ remains a separate future concern.

The [pinned .NET probe](experiments/common-interfaces-dotnet/Program.cs) demonstrates
first-match short-circuiting and the ambiguous zero/default result. Neo regression
coverage adds Option<UserType> and Option<UserType&>, no-default empty searches,
readonly custom equality, callback faults, GC and mutation during callbacks.

## Run the example

```sh
cargo run --locked -- run examples/source/predicate-search.neo
cargo run --locked -- check examples/source/predicate-search.neo
cargo run --locked -- debug examples/source/predicate-search.neo
cargo test --locked --test predicate_search
```

The [sample](../examples/source/predicate-search.neo) searches Person values using
readonly Equals, prints index 1 and Grace, then returns 42. The debugger enters the
ordinary IL search method and delegate target, with the lambda's source positions.
