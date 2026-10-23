# Mutable collection helpers

<a id="mutable-collection-helpers-development"></a>

Import `System.Collections.*` for extensions on `MutableMap<K,V>` and
`MutableSet<T>`. HashMap and HashSet implement those contracts. Existing indexers,
Find, Set and Add keep their behavior.

```raven
let entry = map.Entry("test")
let value = entry.AndModify(value => value + 1).OrInsert(2)
let stored = set.GetOrInsert(candidate)
```

The complete [validated consumer](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/map-entry/raven/Main.rvn)
exercises these operations. See the [design contract](https://github.com/marinasundstrom/neoCLR/blob/main/docs/collection-entry-design.md)
for comparison with .NET and Rust.

| Member | Contract |
| --- | --- |
| `MutableMapExtensions.Entry<K,V>(self: MutableMap<K,V>, key: K) -> MapEntry<K,V>` | Allocates a reusable live locator; no lookup or insertion. |
| `MapEntry<K,V>(map: MutableMap<K,V>, key: K)` | Retains the map and key; same contract as Entry. |
| `Key: K` | Supplied lookup key, not necessarily the canonical stored key. |
| `Value: V` | Current value; missing keys use the map's normal indexer Fault. |
| `Find() -> Optional<V>` | Current lookup; Some(null), where permitted, differs from None. |
| `Set(value: V) -> MapEntry<K,V>` | Inserts/replaces and returns the same locator. |
| `OrInsert(value: V) -> V` | Returns the existing value, or inserts/returns the eager fallback. |
| `OrInsertWith(factory: () -> V) -> V` | Calls factory once only if absent; callback insertion wins. |
| `AndModify(transform: (V) -> V) -> MapEntry<K,V>` | Skips absent keys; otherwise transforms once and writes the returned value, even after nested replacement/removal. |
| `MutableSet<T>.Find(value: T) -> Optional<T>` | Returns the actual stored equivalent value using the set's policy, or None. Implemented by HashSet. |
| `MutableSetExtensions.GetOrInsert<T>(self: MutableSet<T>, value: T) -> T` | Returns the stored equivalent element, preserving identity, or inserts/returns the candidate. |

Callbacks run synchronously and may reenter the collection. Failures propagate before
the helper's write; callback side effects are not rolled back. neoCLR terminal Faults
do not promise cleanup. Required callback parameters are non-null. A retained entry
survives growth and replacement and resolves current state each time. It holds a
strong owner/key reference; it is not a borrowed storage reference or a snapshot.
Returned V/T values use ordinary copy/reference semantics. Keys must remain stable
under the collection's comparer.

Neither operations nor chains add synchronization or atomicity. Externally synchronize
shared use. There is one managed allocation per Entry and potentially multiple
lookups. Custom MutableSet implementations must implement the new Find member;
rebuild matching references and dependent native libraries. No list helper is added.
