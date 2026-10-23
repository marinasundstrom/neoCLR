# Mutable collection entry helpers

Development contract, 2026-10-16. The author selected `MutableMap<K,V>.Entry(key)`
and `MutableSet<T>.GetOrInsert(value)`. Concrete HashMap/HashSet names stay unchanged;
GetItem/list helpers are deferred. These are library extensions, not compiler syntax.

## Live map locator

`Entry` creates a MapEntry class holding a strong map reference and the supplied
key. Creation does not probe the map or reserve a slot. Each operation looks up
current state using the map's own comparer. Handles may be retained and reused
across growth, replacement and, for implementations supporting it, removal and
reinsertion. They identify a key, not one particular insertion. Keys must continue
to satisfy the underlying map's stable hash/equality contract. Key exposes the
supplied lookup key, not necessarily the stored canonical representative.

Find returns Optional (present null is distinct from absence); Value uses the existing
indexer missing-key behavior. Set inserts/replaces and returns the same handle.
OrInsert eagerly receives a candidate, inserts only if absent and returns the
existing/inserted V. Returned values follow normal value-copy/reference-sharing
semantics; no storage reference escapes. AndModify and Set return the same handle.

## Callback and concurrency contract

- OrInsertWith checks presence first. If absent, it calls the synchronous factory
  exactly once, then attempts insertion. A value inserted by the callback wins over
  the candidate; if the callback inserted then removed it, the candidate is inserted.
- AndModify reads once. If absent, it skips the callback. If present, it invokes
  the synchronous `(V) -> V` transform exactly once and writes the returned value.
  This write wins over nested callback writes and recreates a removed key.
- No helper-owned lock, busy state or cached bucket survives a callback. Callbacks
  may use the same map/handle. Each chained operation independently reads current
  state; a chain is not a transaction or a fixed occupied/vacant snapshot.
- A callback failure propagates before the helper's subsequent write. Callback side
  effects, including mutation of a reference-valued payload, are not rolled back.
  Ordinary .NET exceptions unwind; neoCLR terminal Faults retain their existing
  non-unwinding behavior. A storage/comparer failure follows the collection contract.
- No atomicity or concurrent safety is added, even when a .NET backing map is
  concurrent. Synchronize the entire operation/chain externally when sharing it.
  At-most-once is per call, not once per key across competing callers.

The extension uses only public contracts. It allocates one small managed handle per
Entry call and may repeat lookups; we make no single-probe or speedup claim. A class
avoids invalid default struct handles and allows safe retention, at an allocation
cost. No GC/runtime change is needed: ordinary fields retain the owner and key.

## Mutable set canonical lookup

GetOrInsert returns the stored equivalent element, preserving reference identity,
or inserts and returns the candidate. It never replaces an equivalent element.
MutableSet.Find returns Optional of the actual stored element using its equality
policy; HashSet implements it through its existing hash lookup and reentry guard.
This additional interface member requires custom MutableSet implementations to add
Find and dependent native artifacts to be rebuilt. Set's read-only contract stays
unchanged in this bounded slice, as directed by the author.

Raven.Core extends IDictionary for Entry and HashSet/SortedSet for GetOrInsert.
.NET ISet has neither canonical lookup nor an exposed comparer, so an arbitrary
ISet extension cannot implement these semantics faithfully. No fallback enumeration
with default equality, casts to a guessed implementation, or new .NET interface is
introduced. Both targets retain their existing key/null/comparer restrictions.

## Comparison and alternatives

Primary sources checked 2026-10-16:

- [.NET CollectionExtensions.TryAdd](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.collectionextensions.tryadd?view=net-10.0)
  and its [implementation](https://source.dot.net/System.Private.CoreLib/src/runtime/src/libraries/System.Private.CoreLib/src/System/Collections/Generic/CollectionExtensions.cs.html)
  provide the ordinary IDictionary insertion baseline. The fluent helper composes
  existing operations; it does not strengthen their concurrency guarantees.
- [.NET CollectionsMarshal.GetValueRefOrAddDefault](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.collectionsmarshal.getvaluereforadddefault?view=net-10.0)
  supplies direct storage access but restricts additions/removals while that ref is
  used. Our reusable managed handle trades that direct access for lifetime safety.
- [Rust Entry](https://doc.rust-lang.org/std/collections/hash_map/enum.Entry.html)
  is a shipped occupied/vacant view with mutable-reference results and exclusive
  borrowing. We adopt conditional fluent operations, not its borrow/lifetime model.
- [.NET HashSet.TryGetValue](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.hashset-1.trygetvalue?view=net-10.0)
  explicitly supports retrieving a stored representative/reference. MutableSet.Find
  adapts this lookup to neoCLR's existing Optional convention.
- [Language-ext Map](https://louthy.github.io/language-ext/LanguageExt.Core/Immutable%20Collections/Map/index.html)
  offers FindOrAdd returning an updated immutable map and value. It supports a library
  solution but its persistent ownership and null policy do not fit these mutable maps.

Keeping only Find/TryAdd/Set avoids allocation and surface growth, but makes chained
conditional updates more verbose. Borrowed or version-checked cached slots would
require new lifetime/mutation contracts from every map; snapshot handles would become
stale. A live locator is the selected bounded addition. No specific .NET API-review
resolution or independent user-experience study is claimed; the ergonomic motivation
is the author's request, and performance superiority is not asserted.

## Validation

See [focused consumer and reproduction](validation/map-entry/README.md). Validate
behavior rather than implementation opcodes: absent/present values, comparer policy,
retained handles, nested callbacks, side effects/failure, stored set identity and
removal/reinsertion. Public API reference and website feature descriptions accompany
the implementation. Native and .NET callback encodings remain target-specific.
