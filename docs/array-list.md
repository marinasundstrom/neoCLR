# ArrayList<T>: managed growable storage

`System.Collections.ArrayList<T>` is a value wrapper containing a private reference
to managed state holding Data: T[]& and Count: Int32. The GC traces that state and
its buffer. Assignment shares the whole collection, including count and growth,
through ordinary field copying. There is no new runtime copying rule or opcode.
It implements System.Collections.List<T> through managed-reference dispatch.

| Member | Contract |
| --- | --- |
| Allocate(Int32 capacity) -> ArrayList<T> | Return an empty descriptor with a managed backing array; zero capacity is valid |
| Copy() -> ArrayList<T> | Independent state and buffer, shallow copies of live elements; capacity equals Count |
| Count: Int32 | Number of initialized logical elements |
| Capacity: Int32 | Length of the current backing array |
| Add(T value) -> Void | Append a copy of T; grow if full |
| Item[Int32]: T | Checked value get/set for indices below Count |

All instance methods use managed-reference receivers. Direct IL callers load an
address or existing `ArrayList<T>&`; interface callers use `List<T>&`. No receiver
boxing or implicit deep cloning is involved. The generic naming follows the
[API policy](api-policy.md).

## Copying and sharing

Ordinary assignment shares the complete collection. An explicit ArrayList<T>& also
aliases the wrapper slot; replacing that slot with a separately allocated list is
visible through that reference but does not rebind other copied wrappers.

Use `list.Copy()` for an independent sequence. It copies only live elements into
new state and a new buffer with capacity equal to Count. T values follow normal
field-copy rules; T& elements still reference the same targets. Element replacement,
Add and growth in the new list do not change the original sequence. This is shallow
copying, not recursive cloning or copy-on-write. Copy has a readonly managed receiver;
mutable methods still require writable receivers. Readonly access remains shallow.

`ArrayList<Foo>` stores Foo values. `ArrayList<Foo&>` stores reference values:
Add takes Foo&, get_Item returns Foo&, and set_Item replaces the stored reference.
Access through the returned reference in Neo automatically operates on Foo. Mutating
Foo differs from replacing the reference stored in an element. Ordinary reference
copies retain target identity; they do not clone Foo.

## Scope and resource ownership

The wrapper remains an ordinary value whose storage belongs to its containing scope
or owner. Leaving scope ends that local value's lifetime. Its managed state remains
alive while reachable through another wrapper, a managed reference, or other roots;
GC reclamation need not happen at the scope boundary. Under the intended future
value-destructor model, the value's destructor would run at scope exit even if its
managed backing state remains reachable elsewhere. Scope destruction and GC reclamation
are distinct events. Destructor support is not implemented by this slice, and this
library currently calls neither Dispose nor a destructor at scope exit. Managed backing
memory needs no manual cleanup.

A value facade around native resources would need an additional ownership contract:
GC reachability alone does not make disposing a shared native handle safe when one
facade leaves scope. Deterministic disposal and future guest destructors are separate
from this managed collection policy; the change neither introduces nor removes them.

## Capacity, growth and GC

`array.alloc T` reserves checked, uninitialized managed array slots. Unused capacity
contains no fabricated T, null reference, or copied filler value. Reading an
uninitialized slot faults. ArrayList checks indices against Count before element
access and initializes each slot with `stelem T` before incrementing Count.

Growth from zero chooses capacity four; otherwise capacity doubles with checked
Int32 arithmetic. Add allocates a replacement array, copies only Count live elements
with `ldelem`/`stelem`, stores the new item and updates the shared state.
Old arrays remain alive only while reachable, including through existing iterators.
Capacity growth is a preview policy, not a reference invalidation promise.

GC traces initialized elements, including managed-reference values and references
inside records. Replacing an element drops that stored reference; other roots may
still retain its target. Fresh spare capacity contains no references. Copied wrappers observe the same Count. The public API
does not expose addresses of element slots. A Foo& returned from a reference-element
list addresses Foo itself and remains valid across list growth while it is reachable.

Strings, ordinary union carriers, numeric values, records, and managed references
can be stored without native payload layouts. Raw pointer values remain raw pointers:
the list neither roots managed targets through them nor frees their native targets.
Stored managed references must be heap-backed under the existing aggregate storage
rules. Add and indexed set fault on frame references even when verification is skipped.

Negative capacity, invalid indices, arithmetic overflow and resource limits produce
terminal Faults. Managed heap and array-payload limits apply, including simultaneous
old/new buffers during growth and retained arrays from iterators. No thread
safety, Remove, Clear, pinning or native layout is added. Iterators retain their initial
buffer and extent; this contract is unchanged. Each independent list now costs one
additional managed state object, including empty lists.

## Migration and examples

After Preview 3 this is a breaking library storage and assignment-behavior change.
Rebuild against the matching System library. Code relying on independent counts or
growth detachment must call Copy explicitly. Existing published release notes describe
the old behavior and remain unchanged. Native System.Array<T> remains a separate API.

```sh
cargo run --locked -- verify examples/array_list.neoil
cargo run --locked -- run examples/array_list.neoil --gc-stats
cargo run --locked -- run examples/interfaces.neoil
cargo test --locked --test array_list --test library_references --test managed_arrays
```

The ArrayList example takes a managed-reference alias and appends squares through it.
It prints `ArrayList count:`, `5`, `0`, `1`, `4`, `9`, `16`, then returns Void.
Backing arrays are reclaimed by GC without guest cleanup.

Neo uses indexer syntax for reads and writes, including through List<T>& views.
`list[i] = item` calls the setter and replaces the stored T; for Foo& this replaces
the reference. `list[i].Age = value` mutates the referenced Foo instead.

This Neo helper demonstrates the substituted reference-element signature:

```swift
func Append(list: System.Collections.ArrayList<Foo&>&, item: Foo&) -> int {
    list.Add(item)
    let first = list[0]
    first.Age = first.Age + 2
    return first.Age
}
```

Neo supports `System.Collections.ArrayList<Counter&>.Allocate(0)` and conversion
of its managed reference to `System.Collections.List<Counter&>&`. Run the complete
[Neo collection example](../examples/source/collections.neo) with
`cargo run -- run examples/source/collections.neo`; it prints 42, 1, 2, 2 and returns 42.
The example uses mutable wrappers for Add; Count, Capacity, Item getters and Copy
support readonly receivers. Neo indexers select the getter/setter automatically.

## Design comparison (2026-09-08)

The [application experiment](experiments/reference-experience/README.md) reproduced
capacity-dependent partial sharing. Keeping that representation costs less indirection
but makes ordinary collection assignment surprising. Implicit independent copying
would add allocation/copying to assignment; copy-on-write requires controlling every
mutation path. Shared state is implemented entirely in System IL and preserves the
runtime's existing value-copy and reference rules. It costs an additional managed
object and indirection per independent list; no speedup is claimed.

The executed .NET 10 comparison shows whole-list assignment sharing. Microsoft's
[List documentation](https://learn.microsoft.com/en-us/dotnet/fundamentals/runtime-libraries/system-collections-generic-list%7Bt%7D)
and [GetRange contract](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.list-1.getrange?view=net-10.0)
(consulted 2026-09-08) describe explicit shallow sequence copying. NeoCLR's Copy name
is a bounded library adaptation, not a new CLR primitive or a claim that .NET List
exposes that method. It does not implement Clonable or change its receiver contract.
Iterator buffer/extent capture continues to differ from .NET mutation invalidation.

Tests cover assignment across growth, Copy independence in both directions, empty
reference lists, shared element identity, readonly copying, frame-reference rejection
and GC/resource limits. The collection policy remains provisional while the application
experiments continue; retained-reference diagnostics are a separate investigation.
