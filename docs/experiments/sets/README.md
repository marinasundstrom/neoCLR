# Set contracts and storage experiment — 2026-10-10

The author adds sets to the post-JSON collection work. This is a general-purpose
collection requirement, independent of scheduling. Candidate production names are
Set<T>, MutableSet<T> and HashSet<T>; the application-local prototype deliberately
uses SetView, MutableSetView and SetStorage. No new runtime-library API is shipped.

## Contract and comparison

Propose invariant Set<T> : Collection<T> with Contains(T), and MutableSet<T> adding
Add(T) -> bool, Remove(T) -> bool and Clear(). Count and iteration are inherited.
Add reports whether membership changed; a duplicate does not replace the original
element. Remove reports absence without faulting. A read contract does not imply
immutability: mutations through another alias remain visible. Iteration has no
promised ordering; do not inherit indexed Sequence solely to reuse traversal.

HashSet should accept the existing EqualityComparer<T> contract. Equivalent values
must have equal hashes; equality/hash-relevant state must stay stable while stored.
Start with explicit comparers, matching HashMap, rather than inventing default
identity/equality behavior. Ordinary string comparison remains UTF-8 ordinal policy,
with case-insensitive behavior selected explicitly. Nullable elements need separate
validation; non-nullable contracts must not silently admit null.

Sources reviewed 2026-10-10:

- [.NET 10 HashSet](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.hashset-1?view=net-10.0)
  supplies comparer-based uniqueness, membership, removal and set operations. Its
  ISet/IReadOnlySet separation is the ergonomic baseline. The proposed neoCLR core
  is smaller; union/intersection/difference/subset operations remain a follow-up,
  including comparer ownership and self-alias behavior. No parity claim is made.
- The open [.NET interface inheritance proposal](https://github.com/dotnet/runtime/issues/31001)
  describes confusion about mutable/read-only interface relationships. The existing
  [neoCLR capability review](../../collection-contracts.md) already evaluates it.
  It is not evidence of a shipped .NET inheritance change. MutableSet inheriting
  Set fits our current capability approach but adds interface evolution costs.
- [Rust HashSet](https://doc.rust-lang.org/std/collections/struct.HashSet.html)
  demonstrates boolean insertion/removal and explicit equality/hash obligations.
  Rust's ownership restrictions and hashing policy are not automatically neoCLR's.
- C5 at `224386389d83bce81a55f9da304a95d18be15aac` provides a library-only
  [HashSet](https://github.com/sestoft/C5/blob/224386389d83bce81a55f9da304a95d18be15aac/C5/Hashing/HashSet.cs)
  through a broader [ICollection](https://github.com/sestoft/C5/blob/224386389d83bce81a55f9da304a95d18be15aac/C5/Interfaces/ICollection.cs)
  with boolean Add and richer membership operations. This supports avoiding new
  runtime mechanisms, but does not justify importing its entire collection model.

Keeping concrete-only types reduces abstraction cost; the split makes required
capabilities explicit and matches Map/MutableMap. A linear list-backed set is simpler
but requires a scan for membership. Reusing HashMap shares collision/comparer logic
at the cost of unnecessary values/storage; a shared hash-table core or dedicated
key-only implementation is the production comparison to make next. No measured
speed improvement is claimed.

## Executable prototype

The Raven probe uses HashMap<T,bool> for membership and snapshots its Keys for
iteration. Clear replaces the map. Remove temporarily rebuilds all surviving keys
because HashMap has no removal API. That allocates and rehashes the survivors,
with potentially quadratic work under heavy collisions. This is a contract probe,
not an acceptable final deletion algorithm. Snapshot iteration also allocates and
retains references; GC reclamation is not established by checking membership.

Mutating/lookup operations guard against comparer callback reentry, following the
current HashMap terminal-fault policy. Hostile callback/fault coverage is still
required before production use. Constructor-assigned storage uses explicit fields
because the pinned compiler rejects the private-val form; revisit with the compiler
bridge, as the current HashMap implementation also requires this workaround.

The consumer tests inherited interface dispatch, 24 distinct keys with identical
hashes, duplicate results, growth, absent/present removal, membership preservation,
iteration count/sum, snapshot behavior, Clear/reuse and ordinal Unicode text.
The .NET comparison covers the shared membership behavior without asserting the
prototype's snapshot iteration policy. These cases are not a benchmark.

## Validation and next slice

Run the .NET baseline:

```sh
dotnet run --project docs/experiments/sets/dotnet/Comparison.csproj -c Release
```

Build and run the Raven consumer using a fresh output directory:

```sh
SDKROOT=$(xcrun --sdk macosx --show-sdk-path) python3 scripts/build-native-project.py \
  --profile console --project docs/experiments/sets/raven/Native.rvnproj \
  --bundle target/json-collection-hash-development/bundle \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/set-storage-probe-qualified
python3 docs/experiments/sets/validate.py \
  --build target/set-storage-probe-qualified \
  --bundle target/json-collection-hash-development/bundle \
  --interpreter target/release/neoclr
```

Next resolve shared hash removal/free-slot/reference-release behavior, then publish
library contracts with API reference coverage and tests. Validate callback reentry,
GC, nullable/reference/value shapes and Windows/macOS execution before declaring
production readiness. Sets do not automatically enter JSON's admitted collection
families: duplicate-input handling, output order and construction/comparer policy
need a separate mapper decision. Sorted, immutable and frozen sets remain separate
candidates, not requirements inferred from the request for sets.

Validation outcome: the [recorded report](storage-validation.json) passes native
macOS ARM64 and matching release interpreter execution with identical output and
empty diagnostics, using the development bundle with compiler `71cafd353`. The
.NET comparison passes on runtime 10.0.0 with SDK 11.0.100-rc.1.26425.128. Windows,
hostile callbacks and GC reclamation were not tested in this slice.
