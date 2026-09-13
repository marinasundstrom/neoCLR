# Collection contracts: candidates for review

Recorded 2026-09-13. The [first generic-array slice](generic-managed-arrays.md) now implements the managed shape and its explicit Iterable<T> declaration in the Raven profile. The author wants to review collection/enumerable APIs before
accumulating readonly, immutable and frozen contracts. The following proposals are
inputs, not selected library declarations. Keep the implemented List<T> and ArrayList<T>
contracts unchanged during this review; mutable arrays remain invariant.

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
