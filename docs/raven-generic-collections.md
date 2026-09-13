# Closed collection APIs from Raven

The collection target profile now closes ArrayList, List, Iterable and Iterator
signatures over admitted primitive, String, calendar and empty-error payloads.
This extends the original Int32-only catalog. Metadata retains exact generic
arguments; interface conversions are invariant and preserve object identity.

```raven
let values = ArrayList<string>(2)
values.Add("First")
let copy = values.Copy()
copy[0] = "Changed"
// values[0] is still "First".
```

Both constructors, Count, Capacity, the read/write indexer, Add, Copy, GetIterator,
Iterator.MoveNext/Current and Disposable.Dispose are callable. List and Iterable
views share their concrete object's state. Copy creates a separate list sequence;
it copies elements according to their own semantics rather than promising a deep
copy of any referenced resources.

The [sample](experiments/raven-target/samples/library-generic-collections.rvn)
exercises string-list growth/aliasing, long-list copying and Date iteration. It is
part of the saved-project suite. The existing foreach cleanup limitation and
fault-path cleanup work remain documented in [the iteration contract](raven-target-contracts.md).

The backing array uses ordinary managed array storage. Consequently the current
bridge admits only payloads with runtime defaults: this slice does not claim
arbitrary application classes, nested collection payloads or union carriers that
have no valid default. The later [delegate projection](raven-delegate-api.md) adds Find, FindIndex and
Exists with static callbacks.

This reuses the [ArrayList design](array-list.md) and .NET comparison. The runtime
contract and instruction set are unchanged. The bridge still duplicates a bounded
metadata surface; closed execution examples do not demonstrate general CLR generic
loading. Installed SDK/extension packages require a separate refresh.
