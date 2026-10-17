---
uid: N:System.Collections
---
## Collections and iteration

Collection capabilities separate iteration, indexed reads, mutation and map lookup.
Iterable and Iterator support traversal; Sequence and MutableSequence describe
indexed access. ArrayList and HashMap provide concrete list and map storage, with
explicit equality and comparison policies where supported.

Use System.Linq for query operators over these collections. The
[collections guide](/features/collections/) explains the current generic and
execution-mode limits.

Map indexing uses `map[key]`. `Map<K,V>` exposes the getter;
`MutableMap<K,V>` also exposes an insert-or-replace setter. `HashMap<K,V>` uses the
same comparer as `Find` and `Set`, retaining the originally stored equivalent key
when replacing a value. A getter for an absent key causes a terminal Fault: prefer
`Find` for lookup when the key may be absent. Indexing adds no concurrent or atomic
update guarantee.

Fluent helpers extend MutableMap with `Entry(key)` and MutableSet with
`GetOrInsert(value)`. MapEntry is a reusable live locator, with Find, Value, Set,
OrInsert, OrInsertWith and AndModify. MutableSet.Find returns the stored equivalent
value. See [the full contracts](/docs/collection-entries.html), including callback
reentry, failures, allocation and the lack of atomicity.
