# Collection contracts: candidates for review

Recorded 2026-09-13. The [first generic-array slice](generic-managed-arrays.md) now implements the managed shape and its explicit Iterable<T> declaration in the Raven profile. The author wants to review collection/enumerable APIs before
accumulating readonly, immutable and frozen contracts. The following proposals are
inputs, not selected library declarations. The capability prototype below supersedes the earlier instruction to keep List<T>
unchanged during review. Mutable arrays remain invariant.

## Proposals supplied by the author

The first illustration separated Iterable<T>, Collection<T> for count,
Sequence<T> for ordered/indexed access, Map<K,V> and Set<T>, with separate
MutableSequence<T>, MutableMap<K,V> and MutableSet<T>.

The author then supplied this alternative, attributed to ChatGPT:

```text
Iterable<out T>
    Collection<out T>
        List<out T>
        Set<out T>

Map<K, out V>

MutableCollection<T> : Collection<T>
MutableList<T> : List<T>, MutableCollection<T>
MutableSet<T> : Set<T>, MutableCollection<T>
MutableMap<K,V> : Map<K,V>
```

Its concrete examples initially included ArrayList<T>, LinkedList<T>, ImmutableList<T>
and FrozenList<T> implementing List<T>. The subsequent concrete inventory was:

```text
Array<T>
ArrayList<T>
HashSet<T>
HashMap<K,V>

ImmutableList<T>
ImmutableSet<T>
ImmutableMap<K,V>

FrozenList<T>
FrozenSet<T>
FrozenMap<K,V>
```

These were supplied as candidate names and roles, not a request to implement every
type immediately. In a subsequent explicit direction, the author selected
**System.Array<T>** as the intended generic array type. The remaining collection
hierarchy stays open.

### Generic array direction

The author clarified that languages continue to treat arrays the same way; the
purpose is a known generic shape in the runtime type system. System.Array<T> supplies
an identifiable element parameter, member/interface contract and invariant policy.
The work is to connect ordinary array signatures and intrinsic operations to that
shape, rather than require a new source syntax or wrapper.

System.Array<T> should represent the existing typed managed array, not require a
separate collection-wrapper allocation. Retaining ordinary `T[]` language syntax and
CLI array instructions/signatures is the compatibility target. The exact mapping of
array signatures to the generic library type, type identity, reflection and member
lookup needs a dedicated design/implementation slice; this prototype does not make
that mapping exist.

Audit the current nongeneric System.Array members, generic method parameters and
Raven's array-symbol projection. Specify how array element identity, constructors,
null/default states, generic constraints and debugger display map to System.Array<T>.
Do not accidentally admit both an unrelated generic-instance object and an array
signature as different meanings of the same API. Standard CLI array element signatures
already carry T; a generic public type need not by itself require new opcodes.
Declare System.Array<T> invariant: the same T is both produced by reads and consumed
by replacement. Generic declarations provide a regular place to state this policy;
they do not make mutable covariance safe. A separate producer-only interface may
support covariance once its full contract and runtime conversions are validated.
Map that declared policy to intrinsic array signatures and operations so a declaration
and the runtime cannot disagree. In CLR terms, ordinary generic classes are invariant;
variant interface/delegate rules do not automatically apply to arrays.

Maintain mutable-array invariance and fixed size. Replacement of existing elements
must not imply Add/Remove. No generic Array API migration is implemented in this slice.

### Managed arrays versus native buffers

The implementation audit found a naming collision: the existing
`runtime/System/Array.neoil` declares System.Array<T> as a copyable native descriptor
with public Data/Length fields, Allocate, View and Free. Ordinary managed arrays
instead use the runtime's ArrayRef signature and GC-owned array payload. The native
descriptor is not the generic managed-array shape selected above.

After this finding, the author directed us to normalize with .NET. Apply that
direction to ownership and storage roles:

| Role | Direction |
| --- | --- |
| Ordinary T[] / intended System.Array<T> | Fixed-length managed reference object. Assignment shares identity; the GC owns storage. No explicit Free or raw-pointer ownership on the array API |
| Native allocation | Explicit unsafe interop allocation and release, following System.Runtime.InteropServices.NativeMemory's separation from arrays |
| Borrowed contiguous view | A future Span<T>/ReadOnlySpan<T>-like contract over existing storage, with validated lifetime and access permissions. A view does not own or free its backing storage |
| Longer-lived native owner | Separate ownership contract if a scenario requires one; not a copyable pointer descriptor whose aliases all appear to own the allocation |

This follows .NET's [NativeMemory API](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.nativememory?view=net-10.0)
and [owner/consumer and memory-view guidance](https://learn.microsoft.com/en-us/dotnet/standard/memory-and-spans/memory-t-usage-guidelines).
Those are existing .NET contracts, not proposals for merging native buffers into
arrays. Merely changing the current descriptor's name would preserve its mixed
Allocate/View/Free responsibilities. Do not promote that shape as a new standard
NativeArray or NativeBuffer API without resolving ownership first.

The selected generic managed-array identity and invariant mutable-array rule remain
intentional differences from .NET; this ownership clarification does not rescind
them. A managed array's eventual native address exposure would require an explicit
pin/lease or copy with a defined lifetime, not an implicit conversion to a native
owner. Stack-backed storage likewise belongs to the view/lifetime investigation,
not a second allocation mode of the ordinary managed array type.

Implementation order: resolve migration of the old native descriptor out of the
System.Array<T> name; connect managed array signatures to the generic definition;
then implement borrowed views only with the required runtime lifetime checks. Keep
existing native operations available through an explicitly low-level path during
migration. Exact replacement APIs and migration edits remain to implement. No new
Span, native owner, pinning API or automatic resource cleanup is claimed here.

## Review the operations before the variance annotations

| Candidate | Question to resolve |
| --- | --- |
| Iterable<out T> | A producer is plausible. Audit Iterator<T> covariance and ordinary reference returns before composing it; the legacy runtime GetIterator returns a byref |
| Collection<out T> | Count plus iteration can preserve covariance; adding Contains(T) or other element inputs changes the analysis |
| List<out T> | Count and a getter returning T can preserve covariance for reference T. No setters or writable element byrefs. Decide whether indexing promises efficient random access |
| Set<out T> | A normal Contains(T) consumes T, so this shape is not valid under CLR covariance. Decide whether an invariant membership contract or a smaller producer view serves the actual API |
| Map<K, out V> | Key invariance permits key inputs and outputs. Value lookup must be designed: TryGetValue(K, out V) has a byref output, and an invariant Option<V> result is not a covariant use of V. Our Result/Option policy makes this a real contract question |
| Mutable* | Mutation consumes elements/values, so keep these invariant. If they inherit a read interface, decide member ownership once, rather than accidentally duplicate Count/getter slots |

Do not change nullable/error policy, invent covariant value carriers or introduce
special dispatch conversions solely to preserve a desired `out` annotation. An
invariant interface is a valid outcome. Moving membership to an extension method may
also lose the set's lookup/equality contract; that needs evidence rather than cosmetic
consistency. `Option<T>`-based lookup can remain useful without a covariant Map.

The proposed .NET-style variance benefit applies to reference arguments. It does not
make a collection of value elements into a collection of Object without representation
conversion. Iteration-state mutation is separate: MoveNext/Dispose do not consume T
and are not by themselves reasons to reject producer variance.

## Implementation types and guarantees

A mutable ArrayList should expose the selected mutable list contract, and thereby
its inherited read contract, rather than only advertise List. A fixed-size array can
replace elements but cannot grow; this is a reason to review element replacement
separately from Add/Remove before making Array implement MutableList.

Linked-list indexing may be linear rather than constant time. Preserve iteration and
ordering without promising a random-access cost its implementation cannot provide.
Names must communicate useful capabilities; not every ordered sequence needs an indexer.

A live read view, an immutable collection and a frozen lookup-oriented structure can
have different guarantees and construction costs. Define observable mutation, whether
updates return a new collection, source-alias visibility and lookup requirements before
creating families for all three. Immutable collection contents still may contain
mutable referenced objects. A frozen type is not automatically deep immutability,
pinning, read-only native memory or an optimization claim.

Start with the consumer needing count, indexed reads and live aliases in the
[bounded adapter experiment](experiments/readonly-views/README.md). Defer immutable
and frozen additions until a concrete snapshot or repeated-lookup need distinguishes
them. Keep ordinary call, property, indexer and loop ergonomics in the evaluation.

## Comparison and next decision

The [experiment's source review](experiments/readonly-views/README.md#comparison-and-evidence)
includes .NET variance, mutable/read-only interface feedback, Java backed views and
independent immutable .NET collections. In particular, the .NET interface inheritance
proposal is not evidence of a shipped universal hierarchy. Our opportunity is to
choose a coherent contract before publication, not to assume every proposed branch
of the hierarchy is necessary.

Next define complete small member sets and their equality, mutation, lookup-result
and indexing-cost contracts. Then choose the minimal read/iteration contracts to
support with standard variance metadata. System.Array<T> is the selected array direction; the surrounding hierarchy and its
implementation remain open. No renamed List contract, implicit variance conversion or
immutable/frozen implementation is adopted in this document.

## Implementation follow-up

The author confirmed that compatibility with prior iterations is unnecessary and
asked whether Array<T> should implement interfaces. The active Raven profile now
removes the native descriptor entirely, introduces a bounded NativeMemory API and
declares Iterable<T> on the managed Array<T> definition. This supersedes the earlier
proposal to preserve native operations under a temporary descriptor name. The
current List<T> is not added because its Add operation requires growth. See the
[implementation and remaining limits](generic-managed-arrays.md). Earlier planning
statements above describe the state before that slice.


## 2026-09-13 capability prototype

The author approved continuing from generic arrays into a bounded collection review
and prototype, then observed that advertising supported capabilities is preferable
to a mutable default whose Add operation later fails. This section selects a small
experimental hierarchy; it does not adopt all the earlier candidate families.

| Interface | Parent | Declared operations |
| --- | --- | --- |
| Iterable<T> | — | GetIterator |
| Collection<T> | Iterable<T> | Count |
| Sequence<T> | Collection<T> | Item getter |
| MutableSequence<T> | Sequence<T> | Item getter/setter |
| List<T> | MutableSequence<T> | Existing Add operation |

Array<T> will declare MutableSequence<T>; ArrayList<T> will retain List<T> and inherit
all the read/replacement contracts. List is still a minimal growth contract, not a
completed .NET IList equivalent: Remove, Insert, Clear and membership APIs remain
future work. The names, especially Sequence versus List, remain provisional.
All parameters stay invariant until the runtime has validated generic variance.

Count is the current number of accessible positions, not capacity. Indices are
zero-based and must be less than Count; invalid positions retain the existing fault
behavior. Reads return T, not a writable element address. For reference elements,
the returned reference can still identify a mutable object. Sequence promises indexed
access but no general complexity bound; iteration order matches index order. No
concurrent mutation or thread-safety guarantee is added. Existing iterator behavior
is unchanged; iterate without structural mutation in this prototype.

A read interface restricts operations through that reference. Other aliases can
change the storage, and an explicit cast back may succeed when the concrete object
supports mutation. This is neither an immutable snapshot nor a security boundary.
A separate adapter remains useful when hiding the concrete object is required.

### Comparison and tradeoffs

Primary sources reviewed on 2026-09-13:

- [.NET 10 Array](https://learn.microsoft.com/en-us/dotnet/api/system.array?view=net-10.0)
  exposes generic collection interfaces on vectors despite fixed length. Reusing that
  contract would retain unsupported growth operations on arrays. The prototype instead
  expresses replacement separately from growth in ordinary interface metadata.
- [.NET 10 ReadOnlyCollection<T>](https://learn.microsoft.com/en-us/dotnet/api/system.collections.objectmodel.readonlycollection-1?view=net-10.0)
  is a live wrapper and also implements mutable interfaces whose mutation methods reject
  calls. neoCLR's read interfaces omit those members. A wrapper can still serve a
  different purpose, so this does not imply eliminating all read-only adapters.
- [dotnet/runtime #31001](https://github.com/dotnet/runtime/issues/31001) proposes
  making mutable interfaces inherit their read-only counterparts. It remains a
  proposal rather than evidence that .NET ships this hierarchy. neoCLR can experiment
  with that inheritance without preserving an earlier public interface graph.

The existing [comparison program](experiments/readonly-views/dotnet/Comparison.csproj)
was rerun with .NET SDK 11.0.100-rc.1.26425.128. Its alias and cast-back observations
remain applicable. Prior Java-view and independent .NET collection research stays in
[the earlier comparison](experiments/readonly-views/README.md#comparison-and-evidence).

Alternatives are retaining the .NET mutable contracts with runtime rejection, adding
only a read adapter, or splitting the contracts. The split lets parameters state what
they need and avoids allocating a wrapper just to obtain a smaller interface. Its costs
are more interface names, a changed metadata member-owner graph and compiler/runtime
coverage for inherited properties. No performance improvement is claimed.

Placement: declarations and implementation conformance belong in runtime-library
metadata; dispatch uses existing CLR-like calls. Raven should consume inheritance
normally. No new opcode, variance rule, ownership model, immutable/frozen family,
Map or Set API is needed. The legacy Neo library stays outside this profile migration.

Validation will cover array/list substitution, count, reads, replacement, growth,
live aliases, extension methods, source rejection of unavailable operations, low-level
forged member calls, invalid array conformance and invariant element types. Runtime
metadata must not let an array promise Add without an implementation. Compiler work
is limited to any ordinary interface-inheritance bugs the experiment exposes.


### Prototype implementation and next slice

Implemented in the Raven runtime profile: the three new capability interfaces,
List inheriting MutableSequence, and Array declaring MutableSequence. Count on an
array equals Length. Ordinary interface conformance checks and method dispatch now
handle the array's library-defined members; only iterator acquisition needs its
existing intrinsic mapping. No extra view allocation is required by the conversions.

The generated reference assembly and bridge catalog use the same inheritance graph.
Raven required a general inherited-indexer lookup correction; it does not need new
collection-specific language rules. ArrayList keeps its existing implementation and
acquires the read/replacement contracts transitively. This changes metadata member
owners: callers must regenerate target declarations and recompile against the new
profile. Installed .11 tools are unchanged by this source slice.

The sample is `library-collection-capabilities.rvn`; negative source checks are in
`verify_collection_capabilities.py`, and editor checks use `--collection-capabilities`.
Low-level tests cover direct interface dispatch, unchanged array allocation count,
forged setters, wrong element types and unimplemented growth declarations.

**Next, as directed by the author:** Map<K,V> and a dictionary-style implementation.
Use .NET Dictionary<TKey,TValue> as the behavioral baseline. Evaluate HashMap as the
concrete name, explicit read/mutation capabilities, key comparer ownership, equality
and hashing requirements, lookup via Option, duplicate-key Result behavior and
iteration order. No Map, hasher, comparer or dictionary implementation is included in
this slice. Decide these contracts before selecting a hash-table algorithm; do not
assume covariant V through an invariant Option<V> or nullable-key compatibility.


Source validation: 39 focused runtime tests, 22 Raven indexer/iteration tests,
60 saved-project cases, 28 query checks, six unavailable-capability cases and
56 editor completion/hover/diagnostic checks passed. All-target Clippy and the
runtime API inventory check passed. The editor checks used the source-built server;
no SDK, VSIX or installed demonstration was replaced.


## First Map implementation (2026-09-13)

The subsequent [Map slice](map-contracts.md) implements Map<K,V>, MutableMap<K,V>
and HashMap<K,V> with Option-returning Find, TryAdd, Set, Count and snapshot Keys.
It requires explicit equality/hash callbacks and does not yet supply a default
comparer, removal or pair enumeration. The two generic arguments remain invariant.
This extends the prototype; it does not settle the whole proposed collection tree.
LINQ terminal outcomes remain a separate slice.

## Interface-owned factory proposal — 2026-09-15

The author proposed `List<T>.Create()` returning an ArrayList through `List<T>`:
“I just want a list conforming to the list interface.” This is a proposed convenience,
not an implemented factory. In the current Raven profile, List remains mutable:
it extends MutableSequence (indexed replacement), adds Add, and inherits reading
and Count. Sequence is the separate read-access contract. This proposal does not
rename or change those contracts.

The assistant considers this useful if the factory promises a fresh empty mutable
list and guarantees the List contract. ArrayList can be the initial implementation;
callers needing its concrete operations should still construct it explicitly. The
tradeoff is a dependency from the contract's factory to a concrete implementation,
and reduced visibility of implementation-specific allocation/performance choices.
Keep the default deterministic rather than globally configurable.

Compared with .NET's familiar `new List<T>()` concrete construction, this moves
implementation selection into a library factory while preserving ordinary reference
and mutation behavior. Evaluate it alongside a separate factory namespace before
settling the API. This would be an ordinary static factory with a body, not a static
abstract member every implementation must supply, and does not require new runtime
allocation semantics. Raven binding, reference metadata and importer support for that
placement must be tested before promising the syntax. No factory was implemented in
this discussion.

## Queue and Stack review — 2026-10-10

The author selects a post-JSON review of useful collections, naming Queue and Stack,
and clarifies that TaskQueue is not necessarily connected. The
[bounded review and .NET comparison](experiments/queue-stack/README.md) recommends
small general-purpose FIFO/LIFO contracts and array-backed prototypes. These remain
proposals; no scheduler migration, new public collection or JSON admission is claimed.

## Sets added to the active review — 2026-10-10

The author explicitly requests sets alongside Queue and Stack. The
[set review and application-local prototype](experiments/sets/README.md) evaluates
Set/MutableSet capabilities and a hash-based implementation using existing comparers.
Membership, duplicate results, removal and clearing are in scope. Final storage,
set algebra, JSON mapping and production API publication remain separate steps;
this does not adopt the entire earlier consolidated collection proposal.

## Basic library slice and future concurrency — 2026-10-10

The author directs continued work until the library has a basic set of interfaces
and implementations, taking future concurrent variants into account. Development
Queue/ArrayQueue, Stack/ArrayStack and Set/MutableSet/HashSet now have ordinary source
implementations and a [native/interpreter consumer](experiments/native-collections/).
Public contracts remain pre-stable. The prior experiments remain historical evidence.

Queue and Stack inherit Collection, not indexed Sequence. Dequeue/Pop/Peek return
Option: empty is a normal result, distinct from a present nullable value. HashSet
requires an explicit EqualityComparer, suppresses duplicates without replacing the
stored representative, and returns bool for Add/Remove membership changes. It keeps
key-only storage, unlinks bucket chains on removal and reuses freed optional slots.
Equal values must hash equally and must not change equality/hash while stored.
Comparer reentry faults under the current terminal-fault policy.

Concrete implementations are unsynchronized. Iterators capture a shallow snapshot
in FIFO/LIFO order for queues/stacks and unspecified order for sets. Snapshot copying
costs time/storage and retains captured references; it does not make the collection
thread-safe. ArrayList's existing iterator policy remains unchanged. Optional slots
avoid manufacturing null/default values of arbitrary T. Native functional tests pass;
an interpreter heap-count test verifies queue element release. Native heap-count
services, hostile comparer callbacks and broader value/nullable shapes need further
qualification. No allocation or speed improvement is claimed.

For future concurrent implementations, separate operation results from sequences of
calls. Count followed by Dequeue/Pop is not atomic; callers should use the removal
result. Add/Remove results can express a concurrent implementation's linearized
membership change without redesigning the interface. The interface alone does not
promise linearizability: a future concurrent type must document its atomic operations,
memory visibility and iteration consistency. Count is an observation, not a capacity
reservation. Clear and enumeration must be defined by each concurrent type; do not
promise an atomic multi-operation transaction or snapshot merely through Iterable.
Bounded producer/consumer waiting and cancellation belong to distinct contracts.
These points guide future design; no concurrent collection is implemented here.

### Iterable Map and value pairs

The author explicitly requests Map<K,V> : Iterable<KeyValuePair<K,V>>, with a
record-struct pair. The development implementation now supplies KeyValuePair<K,V>
with read-only Key/Value, value copying and Deconstruct, and makes Map iterable.
HashMap captures pairs directly in a shallow snapshot, without a Keys snapshot or
additional lookups/comparer calls. Later inserts/replacements do not alter captured
pairs; referenced objects remain shared. Order is unspecified. This costs O(Count)
time and storage and retains captured references. Snapshot creation is not atomic
under concurrent mutation. Iterable itself makes no snapshot/concurrency promise.

.NET 10's [KeyValuePair source](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Collections/Generic/KeyValuePair.cs)
uses a readonly struct with named getters and an out-parameter Deconstruct method.
That library API, rather than C# record synthesis, is the ergonomic baseline.
[Rust HashMap iteration](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.iter)
returns borrowed pairs; neoCLR's current model instead copies pair values and shares
object references. Sources reviewed 2026-10-10. The earlier C5 and .NET interface
feedback review above supports keeping iteration separate from mutation and scheduling.
A live version-checked iterator avoids snapshot allocation but couples enumeration to
mutation; snapshots were selected for predictable current behavior, not a performance
claim. Future concurrent implementations must select and document their consistency.

Raven's explicit native positional-record storage capability is bounded: constructors,
copying, getters, init accessors and simple deconstruction work; equality, hashing and
display helpers are not exported. Metadata currently carries a normal value type, not
full record semantics. The .NET compiler target retains ordinary record behavior.
This is a temporary development restriction, with native equality/hash/record metadata
work still required. The [init follow-up](init-accessors.md) allows object-initializer
updates while retaining ordinary-assignment rejection. See the [integration contract](raven-cli-bridge.md#native-positional-record-storage--2026-10-10)
and [passing consumer](experiments/native-collections/). Adding Iterable is a source
compatibility change for custom Map implementations: they must implement GetIterator.
Map removal/clear and advanced operations remain subsequent API work.

## Iterable construction and ToMap — development, 2026-10-10

Author direction: collection initialization/copy constructors are a general convention,
following .NET. The initial slice implements HashMap; the follow-up below extends the convention
to the other basic concrete collections.
`HashMap<K,V>(items, comparer)` and `(items, equal, hash)` accept
`Iterable<KeyValuePair<K,V>>`. Sequence, arrays, queries and existing maps already
satisfy that capability; separate overloads would add no behavior.

`pairs.ToMap(comparer)` and `items.ToMap(keySelector, valueSelector, comparer)`
materialize eagerly in one pass. Selectors run key first, once per visited element.
Storage is independent; referenced keys/values remain shared. Empty input is valid.
The explicit comparer determines uniqueness and is never inferred from an input map.
The source must remain valid during enumeration; these unsynchronized APIs do not
promise an atomic snapshot under concurrent mutation.

Primary .NET 10 API contracts checked 2026-10-10:
[Dictionary constructors](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.dictionary-2.-ctor?view=net-10.0)
and [Enumerable.ToDictionary](https://learn.microsoft.com/en-us/dotnet/api/system.linq.enumerable.todictionary?view=net-10.0).
They support iterable initialization/materialization and reject duplicate keys.
This is a library convenience gap in neoCLR, not a .NET limitation. We retain the
existing explicit comparer policy rather than invent a default generic equality
contract. Allocation and hashing are ordinary HashMap costs; no performance gain
is claimed. Existing collection research above remains applicable.

Duplicate keys are rejected rather than silently choosing first/last wins. Unlike
.NET's recoverable ArgumentException, the current constructor and ToMap cause a
terminal System.Fail Fault; use an explicit TryAdd loop when duplicates are expected.
They dispose the iterator on normal completion and detected duplicates. Provider,
selector, comparer or Dispose Faults remain terminal, with no general cleanup or
side-effect rollback guarantee. A future TryToMap with a structured duplicate error
is an alternative if a consumer needs it; this slice does not add it. Selector
materialization follows the same policy as pair construction. No new VM intrinsic,
metadata representation or compiler capability is required.

Validation boundary: an attempted pair-array literal reached the existing AOT
`NewArray(record)` rejection. Array values satisfy the iterable contract, but native
record-array literal allocation is not qualified here. Pair ArrayList/Sequence,
query and Map inputs exercise the supported native path. Supporting value-record
array default allocation is a separate backend task, not a constructor restriction.
A focused .NET 10.0.0 console comparison confirms all three duplicate entry points
throw ArgumentException and that collection storage is independent while a List
value stays shared. This confirms the selected .NET library baseline, not equivalence
between .NET exception recovery and neoCLR terminal Faults.

### List, queue, stack and set constructors

The development follow-up adds `ArrayList<T>(items)`, `ArrayQueue<T>(items)`,
`ArrayStack<T>(items)` and `HashSet<T>(items, comparer)`, all accepting Iterable<T>.
Arrays, Sequence views, lazy queries and existing collections need no dedicated
conversion overloads. Construction enumerates once and disposes normally; storage
is independent and elements are copied shallowly. Provider/comparer Faults remain
terminal without a general cleanup guarantee. Inputs must remain valid while read;
this adds no concurrent snapshot or atomicity promise.

Primary .NET 10 comparisons, checked 2026-10-10:
[List](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.list-1.-ctor?view=net-10.0),
[Queue](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.queue-1.-ctor?view=net-10.0),
[Stack](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.stack-1.-ctor?view=net-10.0),
and [HashSet](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.hashset-1.-ctor?view=net-10.0).
The executable .NET 10.0.0 baseline confirms list/queue order, reversed stack order,
stack-to-stack reversal and case-insensitive set deduplication. neoCLR follows those
library semantics: queue removal starts with the first input, stack removal with
the last. A stack iterator visits top first, so constructing from another stack
reverses its pop order; there is no special case that changes behavior based on the
source's concrete type. HashSet ignores equivalent duplicates, unlike HashMap's
rejection of duplicate keys.

This is a missing library convenience, not a claimed .NET improvement. The set
keeps neoCLR's existing explicit comparer contract. Reusing insertion operations
keeps one set of capacity/hash policies and avoids extra metadata/runtime support;
it can grow storage several times instead of preallocating from Count. Capacity
and allocation counts are not promised, and no speed claim is made. Optimized
count-aware copying can be considered if measurement motivates it. A special
stack-preserving clone would be a distinct API, not an implicit constructor rule.

Implementation uses scoped `use` iterator resources. Compiler 494dede84 repairs
the native portable for cleanup omitted by b939cd696. Library loop simplification
is the next slice. Terminal Faults do not unwind use scopes.
