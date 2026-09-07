# Arrays and pointers: direction and remaining work

Native `Ptr<T>`/`T*` values and a first heap/access subset are implemented; see
[heap and pointers](heap-and-pointers.md). An explicit native-buffer System.Array<T> descriptor is now implemented; owned array
values remain a proposal. Raw pointers are foundational VM capabilities; managed
ownership policies are separate. The current Ref arena is not reference-counted.
See [memory layers](memory-model.md) for the updated architectural direction.

Arrays should follow the same separation between data type and storage as every
other type. Making every array implicitly reference-allocated would reintroduce
the distinction neoCLR intends to remove.

## Implemented first subset: explicit buffer descriptors

System.Array<T> is an ordinary generic interface over a typed memory region. Its
representation is a `T*` plus a length; the pointer may refer to frame or heap
storage. This is analogous to `System.Int32`, which provides methods around a
primitive integer value without changing that value's underlying representation.
Array<T> is a non-owning view over the region.
The ordinary indexed `Item(Int32) -> T` property maps to `Get` and `Set`; it adds no
implicit bounds, allocation or ownership behavior.
There is no new array signature category, opcode, intrinsic member dispatch, implicit
GC, or reference counting. This subset intentionally differs from the owned-array
proposal below and from a .NET managed array.

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
Ref, and erased carrier elements fail allocation layout checks, even for empty arrays.
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

## Remaining array value design

An owned, runtime-length array value is separate future work. Settle its copy/move,
allocation, and return-lifetime contracts before adding newarr, ldelem/stelem, or a
special signature encoding. Do not silently reuse the descriptor's pointer aliasing
as the semantics of owned array values. The earlier candidate below remains a proposal.

## Array ownership and shape

Start with invariant, zero-based, one-dimensional `Array<T>` with a runtime length
and fixed size after construction. Its elements belong to the owning value's
region. `newarr T` would create a frame-owned array by default; placing it in a
shared heap region would require an explicit operation, producing `Ref<Array<T>>`.
A frame arena can hold runtime-sized payloads without requiring an unbounded native
stack allocation. It should fault or return a declared allocation error at a
resource boundary rather than silently changing observable ownership or identity.

For consistency with current record semantics, assignment would copy the array's
elements; copying elements containing `Ref<T>` would preserve those references.
That can be expensive. Before implementing large arrays, decide whether to retain
implicit copies, require explicit copies for variable-sized aggregates, or adopt
moves with borrows. Do not silently make array assignment alias because it is
cheaper. Returning an array must transfer/copy owned storage into the caller's
region, never leave a pointer into a dead callee frame.

Every element must be initialized before safe access. Allocation cannot presume
that zero bits are a valid default for arbitrary `T`, since most types have no
null/default inhabitant. An initial `Array.Create<T>(length, initialValue)` can
copy an explicit value into each slot; an initializer function can come later.
Negative lengths and byte-size multiplication overflow must be handled before
allocation. An empty array is valid and distinct from an absent array.

Bounds-checked `ldelem`/`stelem` and `ldlen` would be the natural IL-like surface.
A violated low-level index invariant can Fault; `Get(index) -> Option<T>` provides
a recoverable lookup API, and mutation can return `Result<Void,BoundsError>`.
Bounds policy should be explicit rather than introducing catchable index exceptions.
Element type is invariant: no array covariance and no delayed array-store type errors.

`Array<Void>` should be legal. It still has a length, bounds, and logical elements,
but it need not store per-element payload bytes. This must work without confusing
zero-sized payloads with null or empty arrays. A pointer model must also define
zero-sized-element stepping before allowing raw pointers to such arrays.

An eventual `InlineArray<T,N>` can express compile-time shape/layout where needed.
It should not force runtime-length arrays to have reference semantics. Jagged
arrays can use nested arrays or explicit references; rectangular arrays and nonzero
lower bounds can wait until migration requirements justify them.

## Distinct access capabilities

| Form | Proposed contract | Current status |
| --- | --- | --- |
| `T` | Owned data; copied under the prototype's rules | Implemented for scalars/records/unions |
| `Ref<T>` | Non-null shared heap identity with checked typed access | Implemented with execution-owned arena |
| `Borrow<T>` / `BorrowMut<T>` | Temporary checked access; cannot outlive owner | Design only; naming provisional |
| `Span<T>` / `SpanMut<T>` | Borrow plus length, with bounds checks | Design only |
| `Ptr<T>` | Unmanaged address type, with no implied ownership | Native heap/access subset implemented |

`Ref<T>` is not an address into the native stack, and `Ptr<T>` must not be an
unchecked alias for it. Managed references may need stable handles or relocation
tracking if GC is introduced. Converting one to a native address would require an
explicit pin or stable allocation, an explicit lifetime, and an unsafe boundary.
A foreign pointer cannot become managed ownership without a declared ownership
transfer and deallocator contract.

Checked borrows would need lifetime metadata or verifiable provenance, an escape
rule, and an aliasing contract. Rust-style exclusivity is not a default VM rule;
checked borrowing should be an explicit facility rather than a host-language assumption. A span can view stack or heap storage; its origin
must stay live. Returning a borrow to callee-owned storage must fail verification.
Array element access through borrows must retain bounds and allocation provenance.
These rules are prerequisites for address-taking instructions such as `ldelema`.

Raw pointers belong to the core low-level VM, independently of whether a high-level
language restricts them to unsafe or interop code. The design needs alignment,
address width, bounds/provenance, null, arithmetic overflow, and deallocation
contracts; pretending a raw address is safe does not supply those contracts.
BCL absence should remain `Option`. A future foreign ABI may represent an actual
null pointer directly, but it must not become the general missing-value convention.
If a `Ptr<T>` requires a storage-bearing element, native `void*` should map to an
opaque address type, not automatically `Ptr<Void>`: neoCLR's inhabited, zero-sized
`Void` is a different concept.

## Async interaction and next experiment

A future runtime-async function must keep owned values live across suspension.
Its logical frame may reside in persistent storage. A borrow into an ordinary
caller stack frame cannot simply survive suspension; it needs a proven enclosing
lifetime or must be rejected. Physical “always native stack” placement is therefore
not a sustainable universal promise. Frame ownership is the semantic promise;
backend placement and explicit shared identity are separate concerns.

The explicit buffer descriptor now serves the initial runnable milestone. The owned-array
experiment, checked spans, and broader ownership policies remain separate later work.
Tests cover empty arrays, bounds, descriptor aliasing versus element copies, invalid
lengths, and zero-sized elements. The heap/pointer implementation remains independently
usable.
