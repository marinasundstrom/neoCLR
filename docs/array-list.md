# ArrayList<T>: managed growable storage

`System.Collections.ArrayList<T>` is a platform-written value descriptor containing
private `Data: T[]&` and `Count: Int32` fields. Its backing array lives on the managed
heap and is traced by the GC. There is no native control block, raw storage pointer,
manual Free, or special collection opcode. It implements `System.Collections.List<T>`
through managed-reference interface dispatch.

| Member | Contract |
| --- | --- |
| Allocate(Int32 capacity) -> ArrayList<T> | Return an empty descriptor with a managed backing array; zero capacity is valid |
| Count: Int32 | Number of initialized logical elements |
| Capacity: Int32 | Length of the current backing array |
| Add(T value) -> Void | Append a copy of T; grow if full |
| Item[Int32]: T | Checked value get/set for indices below Count |

All instance methods use managed-reference receivers. Direct IL callers load an
address or existing `ArrayList<T>&`; interface callers use `List<T>&`. No receiver
boxing or implicit deep cloning is involved. The generic naming follows the
[API policy](api-policy.md).

## Copying and sharing

An `ArrayList<T>&` aliases the entire descriptor: mutation, Count changes and growth
are visible to all references to that descriptor. An ordinary `ArrayList<T>` copy
copies Count independently and copies the Data reference. The copies initially share
array elements, but growing one replaces only that descriptor's Data reference.
They can subsequently address different arrays. This is ordinary field-copy behavior,
not independent deep collection copying and not a shared hidden control block.

Use an explicit reference to share one coherent mutable list. A future explicit
Clone operation can provide independent collection copying. This slice does not
add implicit cloning, copy-on-write, or a collection destructor.

`ArrayList<Foo>` stores Foo values. `ArrayList<Foo&>` stores reference values:
Add takes Foo&, get_Item returns Foo&, and set_Item replaces the stored reference.
Access through the returned reference in Neo automatically operates on Foo. Mutating
Foo differs from replacing the reference stored in an element. Ordinary reference
copies retain target identity; they do not clone Foo.

## Capacity, growth and GC

`array.alloc T` reserves checked, uninitialized managed array slots. Unused capacity
contains no fabricated T, null reference, or copied filler value. Reading an
uninitialized slot faults. ArrayList checks indices against Count before element
access and initializes each slot with `stelem T` before incrementing Count.

Growth from zero chooses capacity four; otherwise capacity doubles with checked
Int32 arithmetic. Add allocates a replacement array, copies only Count live elements
with `ldelem`/`stelem`, stores the new item and updates the original descriptor.
Old arrays remain alive only while reachable, including through copied descriptors.
Capacity growth is a preview policy, not a reference invalidation promise.

GC traces initialized elements, including managed-reference values and references
inside records. Replacing an element drops that stored reference; other roots may
still retain its target. Fresh spare capacity contains no references. A copied descriptor can retain
initialized slots beyond its own Count because it shares the array. The public API
does not expose addresses of element slots. A Foo& returned from a reference-element
list addresses Foo itself and remains valid across list growth while it is reachable.

Strings, ordinary union carriers, numeric values, records, and managed references
can be stored without native payload layouts. Raw pointer values remain raw pointers:
the list neither roots managed targets through them nor frees their native targets.
Stored managed references must be heap-backed under the existing aggregate storage
rules. Add and indexed set fault on frame references even when verification is skipped.

Negative capacity, invalid indices, arithmetic overflow and resource limits produce
terminal Faults. Managed heap and array-payload limits apply, including simultaneous
old/new buffers during growth and retained arrays from descriptor copies. No thread
safety, enumeration-under-mutation, Remove, Clear, pinning or native layout is added.

## Migration and examples

This is a breaking storage/API change. Rebuild source/artifacts against the current
System library and remove ArrayList.Free calls. Native-buffer `System.Array<T>` remains
a separate API with explicit native allocation/free; it is not this backing store.

```sh
cargo run --locked -- verify examples/array_list.neoil
cargo run --locked -- run examples/array_list.neoil --gc-stats
cargo run --locked -- run examples/interfaces.neoil
cargo test --locked --test array_list --test library_references --test managed_arrays
```

The ArrayList example takes a managed-reference alias and appends squares through it.
It prints `ArrayList count:`, `5`, `0`, `1`, `4`, `9`, `16`, then returns Void.
Backing arrays are reclaimed by GC without guest cleanup.

This Neo helper demonstrates the substituted reference-element signature:

```swift
func Append(list: System.Collections.ArrayList<Foo&>&, item: Foo&) -> int {
    list.Add(item)
    let first = list.get_Item(0)
    first.Age = first.Age + 2
    return first.Age
}
```

Neo supports `System.Collections.ArrayList<Counter&>.Allocate(0)` and conversion
of its managed reference to `System.Collections.List<Counter&>&`. Run the complete
[Neo collection example](../examples/source/collections.neo) with
`cargo run -- run examples/source/collections.neo`; it prints 42, 1, 1, 2 and returns 42.
The example uses mutable value descriptors because collection receivers are writable
managed references, including Count observation; readonly contracts remain future work.
Library indexer syntax also
remains separate; direct accessor calls work. A unique library method signature now
provides parameter context, so existing references reach Add without an extra `&`.
Overloaded methods retain exact-signature selection in this compiler subset.
