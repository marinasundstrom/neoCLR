# Map prototype for the Raven target

Updated 2026-09-27: the [development comparer slice](#comparer-policies-development)
adds reusable policies and a HashMap constructor; Preview 10 retains callbacks only.

The original prototype below was recorded 2026-09-13. This is not the final collection API
or a complete replacement for .NET Dictionary. The [collection capability work](collection-contracts.md)
is its starting point. The historical Neo profile is unchanged.

## Contract

All three types are invariant in both K and V:

| Type | Members declared here |
| --- | --- |
| `System.Collections.Map<K,V>` | `Count`, `Keys: Sequence<K>`, `Find(K): Option<V>`, `ContainsKey(K): Boolean` |
| `MutableMap<K,V>: Map<K,V>` | `TryAdd(K,V): Boolean`, `Set(K,V): void` |
| `HashMap<K,V>: MutableMap<K,V>` | Implements the above; constructor takes `Func<K,K,Boolean> equal` and `Func<K,Int32> hash` |

`Find` distinguishes absence from a present zero/default value. `TryAdd` returns
false without changing the stored key or value if an equivalent key already exists.
`Set` inserts or replaces the value and retains the originally stored equivalent key.
The original prototype omitted indexers; the development contract below now adds
them. There is still no `Add`. A duplicate is the sole normal rejection
outcome of TryAdd, so a Boolean suffices; a new Result error hierarchy adds no
information for this operation. Faults still terminate execution.

The read interface withholds mutators; another alias may mutate the map. `Keys`
returns an independent shallow snapshot in insertion order. Casting that snapshot
to a mutable sequence and replacing an element cannot alter map storage. Reference
keys are still shared objects: changing a key's equality/hash-relevant state is a
caller error. Each snapshot costs O(n) copying and managed allocation. This is not
a live .NET Dictionary key view or an immutable collection guarantee.

The two callbacks must describe the same stable equivalence relation: equal keys
have equal hashes, and both results must stay stable while the keys are stored.
Collisions are supported; they do not make unequal keys equal. Hash callbacks may
return any Int32, including negative values and Int32.MinValue. The implementation
masks off the sign bit before choosing a bucket; hashes are not persistent identities.
The callbacks run synchronously, may inspect Count/Keys, and must not recursively
call Find, ContainsKey, TryAdd or Set on the same map. Those calls fault to prevent
reentrant mutation invalidating an in-progress lookup. This is not thread safety.

**Current limits:** callers supply both callbacks, including for primitive keys.
There is no default comparer selection, public comparer object, capacity overload,
removal, clearing, pair/value enumeration, serialization or concurrency contract.
The admitted key domain is currently the callbacks' domain: the map adds no uniform
null-key rejection, so a null reference key can work if both callbacks support it.
This differs from Dictionary and must be settled with default comparers/nullability
before presenting a general replacement. Non-null Type keys and integer keys are
demonstrated. Callback faults and invalid accesses are not converted to Result.
No boxing-based default equality or reference-identity default is silently inferred.

## Why this shape, compared with .NET?

Primary sources reviewed 2026-09-13; .NET API baseline is .NET 10. The regular CLI
class/interface/generic/delegate signatures express these APIs. Algorithm and
outcome choices belong to the library; no Raven syntax, compiler rule, new metadata
kind or opcode is required.

- [.NET Dictionary](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.dictionary-2?view=net-10.0)
  supports comparer selection, non-null keys, indexed lookup/update and enumeration.
  Its ordinary missing-key indexer throws. neoCLR uses an Option-returning method
  for expected absence and retains explicit mutation through a separate interface.
  This costs some familiar indexer ergonomics and is a provisional API choice.
- [.NET TryAdd](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.dictionary-2.tryadd?view=net-10.0)
  already reports duplicates without overwriting. The
  [original API proposal](https://github.com/dotnet/runtime/issues/14676) motivated
  avoiding the ContainsKey-then-Add pattern; the proposal is historical and the
  method is now shipped. neoCLR preserves this useful behavior rather than
  inventing a Result for every operation. Our contract does not promise atomic
  multithreaded operations.
- [.NET EqualityComparer.Create](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.equalitycomparer-1.create?view=net-10.0)
  demonstrates callback-based equality/hash configuration. This prototype supplies
  both callbacks directly to the collection. That avoids choosing universal default
  equality before the runtime library provides coherent hashing, but is less
  convenient and makes accidentally mismatched policies easier. A paired comparer
  abstraction and tested primitive defaults are the next Map-specific work.
- [Rust HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
  requires equality/hash consistency and stable keys. Those requirements transfer;
  Rust's borrowing, table implementation and ownership policy are not adopted.
  This prototype's predictable, caller-supplied hashing has no collision-attack
  resistance or performance parity claim.
- [LanguageExt HashMap](https://louthy.github.io/language-ext/LanguageExt.Core/Immutable%20Collections/HashMap/index.html)
  offers `Find` returning `Option<V>`. It is a useful .NET ecosystem comparison for
  absence, but its persistent immutable update model is different from neoCLR's
  ordinary mutable reference object. We do not imply that copying a HashMap binding
  copies its contents.

Alternatives left open include the full Dictionary surface with faults on missing
indexers; a read indexer returning Option with separately named updates; a comparer
interface plus default factory; and pair-based Iterable enumeration. A linear scan
would have avoided hashing but would not test a HashMap. A native host dictionary
would hide the storage and equality choices we need to exercise. The implemented
chained table lets existing managed storage and dispatch do the work.

## Implementation and validation

[Map.neoil](../runtime/raven/Map.neoil) owns Int32 bucket heads and managed ArrayLists
of keys, values, saved hashes and next indexes. Empty bucket heads are zero; occupied
links encode index+1. Bucket counts start at four and double when full, rebuilding
chains from saved hashes without calling user code. Checked arithmetic and existing
array/GC limits retain runtime fault behavior. Collision-heavy lookup can be O(n).
There are no benchmark claims. Multiple ArrayLists increase allocation and dispatch
costs; a compact entry representation can follow after the contract is useful.

An internal callback holder is constructed with its complete field values. This
uses the existing query-library pattern because delegate slots currently have no
valid default construction state. The public map constructor initializes ordinary
reference fields. GC sees the callbacks, keys and values through ordinary managed
fields and arrays, including captured references and cycles.

The metadata-only core declarations and bounded importer gain Map signatures. They
do not execute the dictionary algorithm. `MapBindings` validates invariant generic
metadata and closes argument/result signatures. No Raven repository change is needed.

The [Raven sample](experiments/raven-target/samples/library-maps.rvn) covers order
lookup, duplicate rejection, Option destructuring, growth, snapshots, interface
passing, Type keys, shared ArrayList values and nested generic collection usage.
`tests/raven_collections.rs` adds direct IL checks for collisions/distributed hashes,
GC pressure, reference retention, snapshot isolation, reentrancy faults, missing
mutators and incompatible type arguments. The signature probe rejects forged
returns and variant metadata. Editor checks discover read versus mutable members.

Run the [.NET comparison](experiments/maps/dotnet/Program.cs) with:

```sh
dotnet run --project docs/experiments/maps/dotnet/Comparison.csproj
```

For Raven, regenerate the core declarations and target System library together,
then run the sample through `run_project.py` as described in the
[experiment release procedure](experiments/raven-target/RELEASING.md). The existing
installed .11 SDK/extension/bundle does not contain this API; it was not replaced
by this source slice. Use a fresh source-build probe directory for validation.

LINQ terminal Option/Result changes remain a separate slice, as the author requested.

Source validation on 2026-09-13 passed 13 collection/runtime tests, 61 saved-project
cases, 81 signature checks, ten capability rejection cases and 59 editor checks.
The .NET comparison ran on SDK 11.0.100-rc.1.26425.128 targeting net10.0 and printed
`True`, `False`, `Pending`, `False`, `Shipped`, `2`, `23`, `Stored`, `Stored`, and
`Null key rejected`. The count of two on its previously obtained Keys demonstrates
the live-view difference. Source-built tools were used; this is not installed-bundle
or packaged-release validation.


## Raven implementation — 2026-09-15

The HashMap implementation now lives in `runtime/raven/src/System/Collections/HashMap.rvn`; the map
interface declarations remain in `runtime/raven/Map.neoil`. Generated bootstrap IL
preserves the algorithms and contracts described here. Private implementation helpers
remain private. The class owns equality/hash delegates directly and uses the
Raven-authored ArrayList for entry storage. No default comparer, removal or pair
iteration is added by this migration. See [the authoring gate](raven-system-library.md#hashmap-and-private-instance-helpers--2026-09-15).

## Comparer policies (development)

The author requested comparers on 2026-09-27. This post-Preview-10 slice makes
HashMap key policies reusable and gives ordinary algorithms an explicit ordering.
It is library functionality using existing CLI interfaces, generic classes and
virtual calls; it introduces no opcode or Runtime Contract switch.

| API | Contract |
| --- | --- |
| `System.Collections.EqualityComparer<T>` | Invariant interface: `Equals(T left, T right): bool`, `GetHashCode(T value): int`. |
| `System.Collections.Comparer<T>` | Invariant interface: `Compare(T left, T right): int`; negative/zero/positive means less/equivalent/greater. |
| `FunctionEqualityComparer<T>(equal, hash)` | Retains both callbacks; invokes them synchronously without suppressing faults. |
| `FunctionComparer<T>(compare)` | Retains an ordering callback with the same sign contract. |
| `System.StringComparer.Ordinal` | Stateless policy implementing both string interfaces; exact equality, existing String Object content hash, unsigned UTF-8/scalar ordering. A new policy instance is returned; do not rely on identity. |
| `HashMap<K,V>(EqualityComparer<K> comparer)` | Retains the policy. Existing callback constructor adapts to FunctionEqualityComparer and remains source-compatible. |

Equality must be reflexive, symmetric and transitive; equal values must have equal
hashes. Ordering must be transitive with opposite signs on reversed unequal inputs.
An ordering's zero result need not imply another policy's equality. No runtime
proof of callback laws is possible. Keys and policy behavior must remain stable
while stored. Hash codes may be negative, collide, and change between runtime
versions; never persist them as identities. No universal Default policy is added.

StringComparer accepts non-null strings, including empty text and embedded NUL.
The Ordinal policy neither normalizes Unicode nor ignores case. U+10000 sorts after U+E000, retaining
neoCLR's [native ordinal order](ordinal-text.md). The map continues to delegate its
key domain, including nullable reference keys, to the selected policy. It adds no
uniform null-key rejection; passing null outside a policy's declared domain is
unsupported. Comparers and callbacks themselves must be non-null. Terminal callback
faults stay faults. Existing map reentry protection covers both comparer methods;
this is not a concurrent collection or an async callback contract.

The map and adapters retain ordinary managed references, including callback captures.
An adapter introduces one object for the legacy callback constructor; the explicit
policy can be reused across maps. Algorithms and table growth are unchanged. There
is no performance, collision-attack resistance or singleton allocation claim.
The pinned compiler needs explicit fields for constructor-assigned storage without
initializers in these slices; revisit ordinary private var/val when that form is
supported by the selected compiler.

### Comparisons and alternatives

Primary sources reviewed 2026-09-27 against .NET 10:

- [.NET IEqualityComparer<T>](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.iequalitycomparer-1?view=net-10.0)
  pairs equality and hashing; [IComparer<T>](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.icomparer-1?view=net-10.0)
  separately supplies ordering. neoCLR follows these roles and uses its established
  interface naming without an I prefix. Unlike .NET's abstract EqualityComparer class,
  our EqualityComparer name denotes the interface. Generic variance and automatic
  default selection are deliberately absent in this bounded slice.
- [.NET StringComparer.Ordinal](https://learn.microsoft.com/en-us/dotnet/api/system.stringcomparer.ordinal?view=net-10.0)
  combines string policies but orders UTF-16 code units. neoCLR preserves its native
  UTF-8 ordering and non-null string domain. This simplifies consistency with existing
  String methods at the cost of different supplementary-character order and null behavior.
- [.NET EqualityComparer.Create](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.equalitycomparer-1.create?view=net-10.0)
  supports callback adaptation. Explicit Function comparer classes keep construction ordinary
  in the current target. Requiring a hash callback prevents accidentally selecting a
  different default hash for custom equality; it cannot guarantee the callbacks agree.

Keeping only two raw callbacks would preserve less API surface but perpetuate policy
wiring at each map. A universal Object-based default would choose boxing, null and
value semantics for every T prematurely. A new runtime instruction would add an ABI
without supplying an operation the existing library cannot express. The existing
[Map prototype comparison](#why-this-shape-compared-with-net), including Rust and
LanguageExt, remains applicable to lookup outcomes and stable key requirements.

### Validation

The [comparer sample](experiments/raven-target/samples/library-comparers.rvn) exercises
ordering through an interface, exact/canonical-distinct/case-distinct Unicode text,
content hash agreement, collisions including Int32.MinValue, growth, equivalent-key
replacement, original reference preservation, user-defined policies and the legacy
constructor. The focused verifier also checks reentrant policy faults and rejects
mismatched policy types. See the library tracker for completed validation evidence.


The subsequent development comparison slice adds `StringComparer.OrdinalIgnoreCase`.
It shares Unicode 17 default simple folding across equality, ordering and hashing;
HashMap retains the original spelling when a differently cased key replaces its
value. See the [text contract](ordinal-text.md#explicit-comparison-modes-development)
for .NET differences, normalization/expansion limits and matching-runtime requirements.

## Indexers — development, 2026-10-10

Map now declares a read-only `self[key: K]: V` indexer. MutableMap redeclares it
with a setter, and HashMap implements both accessors. The getter uses the same
comparer and lookup rules as Find but causes a terminal Fault when the key is
absent. Prefer Find when absence is part of normal control flow. This is a fallible
lookup convenience, not a memory-unsafe operation.

The setter has Set's insert-or-replace behavior and preserves an existing key.
It does not fail merely because a key is new; allocation/capacity and comparer or
reentry faults remain possible. No atomic read-modify-write or concurrency guarantee
is added. Existing third-party Map/MutableMap implementations must add the appropriate
indexer accessors; use a matching reference/runtime library bundle.

This follows the indexer shape of .NET IReadOnlyDictionary/IDictionary and
[Dictionary](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.dictionary-2.item?view=net-10.0).
The missing-key channel follows neoCLR's existing terminal Fault model rather than
.NET's catchable KeyNotFoundException. That makes Find especially important for
recoverable lookup. No new error type or exception subsystem is introduced.

Validation: [framework tests](../runtime/raven/tests/collection-construction/MapIndexers.rvn)
and [missing-key probe](experiments/map-indexers/README.md). JsonObject's proposed map
conformance remains a separate follow-up.
