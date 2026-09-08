# Arrays and pointers: direction and remaining work

Managed owned and heap arrays are implemented separately; see [managed arrays](managed-arrays.md).
This page describes the existing native `System.Array<T>` pointer/length buffer API.
It retains explicit native allocation/free and frame-backed `localloc` storage.

## Implemented first subset: explicit buffer descriptors

System.Array<T> is an ordinary generic interface over a typed memory region. Its
representation is a `T*` plus a length; the pointer may refer to frame or heap
storage. This is analogous to `System.Int32`, which provides methods around a
primitive integer value without changing that value's underlying representation.
Array<T> is a non-owning view over the region.
The ordinary indexed `Item(Int32) -> T` property maps to `Get` and `Set`; it adds no
implicit bounds, allocation or ownership behavior.
This descriptor uses ordinary generic metadata and methods. Its native buffer is not
traced by the managed GC and has no reference counting. It is distinct from T[] managed arrays.

| Member | Contract |
| --- | --- |
| static Allocate(Int32 length, T initialValue) -> System.Array<T> | Allocate heap storage, initialize each element by copying the supplied value, and return a view |
| static View(T* data, Int32 length) -> System.Array<T> | Construct a view over existing storage without allocating or taking ownership |
| instance get_Length() -> Int32 | Read descriptor length; exposed through the ordinary Length property |
| instance Get(Int32 index) -> T | Bounds-check and return an element value |
| instance Set(Int32 index, T value) -> Void | Bounds-check and write an element |
| instance GetElementAddress(Int32 index) -> T* | Bounds-check and compute its raw address using checked native-integer arithmetic |
| instance Free() -> Void | Explicitly release the backing allocation |

Copying the descriptor copies its pointer and length; both copies access the same buffer.
The view does not record whether storage came from the heap or the current frame. The
caller must keep the storage alive and must call `Free` only for allocations made by
`Allocate`.

This separation is part of the type-safety model: the view remains `Array<T>` and
element operations remain typed regardless of allocation location. Allocation
provenance and lifetime are checked by the pointer/memory subsystem rather than
encoded as separate array types.

## Stack and heap allocation

Both storage locations are available in the low-level instruction set. The existing
`System.Array<T>.Allocate` method uses `heap.alloc` and requires an explicit `Free`.
For frame-lifetime storage, a program computes `length * sizeof(T)`, calls `localloc`,
casts the resulting `Byte*` to `T*`, and constructs the same `System.Array<T>` descriptor.
The frame releases that storage automatically when the invocation returns. The
descriptor and indexer are identical in both cases; only the pointer's allocation
provenance differs.

`examples/arrays_stack.neoil` demonstrates a two-element frame-local array and prints
`42`. A pointer to that storage must not escape its owning frame, and the stack form
has no `Free` operation.
It does not copy elements or acquire ownership. Element Get/Set and initialization use
existing value/storage rules: records copy, small scalar storage remains precise, and
pointer-containing elements copy addresses without acquiring pointee ownership. Free
must be called exactly once for an allocation. It does not run element destructors or
free pointees. Remaining aliases cannot safely access freed storage; the interpreter's
existing allocation diagnostics reject dangling accesses and repeated free.

Length is fixed by the API after allocation, but the descriptor has ordinary metadata
fields and no encapsulation guarantee. Handwritten IL can construct or modify descriptors;
these methods do not prove that arbitrary Data/Length pairs describe a valid buffer.
Raw element pointers obey the existing pointer lifetime/alignment rules. Native interop
retains its separate unsafe boundary. This is a low-level buffer facility, not checked
borrow provenance or a permanent array ownership model.

The initial subset supports T only where native layout and typed loads/stores exist:
numeric types, Boolean, Char, Void, pointers, and supported closed records. String, Error,
managed references, and erased carrier elements fail allocation layout checks, even for empty arrays.
Nested descriptors store pointer/length values; they do not deep-copy inner buffers.

Negative lengths and out-of-range indices produce terminal Faults. Access is invariant,
zero-based, and excludes the one-past-end index. Empty arrays are valid but have no valid
indices. Array<Void> retains its logical length and bounds with zero payload bytes;
valid element addresses can coincide. A zero-byte allocation still has a tracked lifetime.
Recoverable Option/Result accessors and bounds-error types can be layered on later.

Allocation uses existing byte/allocation limits, and initialization consumes ordinary
IL frames and instructions. A Fault aborts execution and releases its remaining storage
through existing execution teardown. Returning a descriptor within guest calls preserves
its execution-owned allocation; returning it to a host does not make it transferable
into another invocation. Keep the Execution alive while its memory is needed. Host
invocation continues to reject pointer-bearing input schemas.

`cargo run -- run examples/arrays.neoil` prints 10, 42, 10 after updating a shared buffer,
then explicitly frees it. `examples/array_bounds.neoil` deliberately faults on index 2
of a two-element array and presents GetElementAddress -> Get -> Main in its stack trace.

## Further work

The [managed-array contract](managed-arrays.md) supersedes the earlier owned-array
proposal. `newarr` creates managed heap storage; `array.create` creates owned values.
Checked slices and native pinning remain future work. Async suspension will need
explicit lifetime rules for references into caller-owned arrays.
