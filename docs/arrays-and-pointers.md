# Arrays and pointers: direction and remaining work

Native `Ptr<T>`/`T*` values and a first heap/access subset are implemented; see
[heap and pointers](heap-and-pointers.md). Array storage remains a proposal. Raw pointers are foundational VM capabilities; managed
ownership policies are separate. The current Ref arena is not reference-counted.
See [memory layers](memory-model.md) for the updated architectural direction.

Arrays should follow the same separation between data type and storage as every
other type. Making every array implicitly reference-allocated would reintroduce
the distinction neoCLR intends to remove.

## Initial runnable milestone

Arrays are now a near-term fundamental alongside primitive-backed types, strings, and
Error/Fault diagnostics. Start with a small program that creates an initialized array,
reads its length, reads and updates elements, and demonstrates a bounds failure with
a Fault trace. Empty arrays, explicit element values, and generic element types belong
in the first contract; extensive OOP and the full collection library are not prerequisites.

The ownership proposal below remains a candidate, not an accepted encoding. Before
implementation, settle owned element storage versus a descriptor over separately allocated
storage, copy/alias behavior, construction/allocation instructions, and return lifetime.
Do not make allocation or copying silently imply GC or the bootstrap Ref arena. Raw
pointer buffers already exist, but lack the array value's shape and checked-access contract.
String APIs may later use array/buffer facilities without making the String representation
or Unicode indexing depend on a particular array-storage policy.

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

The initial heap allocation and pointer operations are now available. The array
experiment above is promoted to the initial runnable milestone. Checked spans and
broader ownership policies remain later work. Initial array tests should cover empty arrays, bounds, aliasing versus
copies, invalid lengths, and zero-sized elements; the existing heap/pointer implementation remains independently usable.
