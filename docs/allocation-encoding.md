# Allocation encoding: proposals and current scope

Current memory milestone: format 5 uses direct heap-backed T&. Ref and heap.load/store
are removed; heap-only references can be stored in fields/erased payloads and returned
to the host for context-bound inspection. See [the current contract](heap-references.md).

The selected [high-level allocation direction](lifecycle.md#high-level-allocation-syntax)
uses T(...) for value construction, &value for a reference to an existing value, and
new T(...) for explicit managed heap allocation returning T&. Reference retention
is automatic. A function may return a caller-backed reference or, in the future,
a managed heap reference. Returning an address into its own frame faults, including
when a helper or alias obscures the origin. There is no implicit promotion of a
local merely because its address is returned. High-level syntax is separate from the final IL
encoding, which remains open.

The sections below describe implemented raw-memory operations and earlier encoding
proposals. They do not define new T(...) as returning a raw pointer. Managed heap
allocation through T& and allocator-policy composition remain future work.
[Tracing GC](garbage-collection.md) now manages the transitional Ref heap.
An explicit allocator operand at every high-level allocation site is not required.

The first implementation now uses `heap.alloc T` (element count to `Ptr<T>`) and
`heap.free`, with `stobj T` for copying a constructed value into storage. It uses the
host native allocator; pluggable allocator services remain deferred. See
[implemented contract](heap-and-pointers.md). The alternatives below are historical
design options. `localloc` is now implemented as frame-local byte storage; see the
[frame-local contract](heap-and-pointers.md#frame-local-allocation).

Separate storage allocation, object construction, and ownership policy. None of
these should be selected by a permanent value/reference bit on the type.

- Stack storage: `localloc`, with explicit frame lifetime, byte size, and primitive
  alignment. The interpreter backs it with host buffers.
- Raw heap storage: an explicit allocator call returning an unmanaged pointer (or a
  Result carrying that pointer and a recoverable allocation error). The allocator
  is an implementation/ABI contract, not an ambient managed-object heap.
- Construction: initialize the selected destination and execute the type's constructor.
  Pointer receiver and constructor contracts still need to be implemented.
- Ownership: optionally wrap an allocation in a library abstraction such as a
  reference-counted `Ref<T>`. Allocation does not imply retain/release behavior.

`newobj` can be a convenient typed operation that produces an owned value in the
current frame. This intentionally changes the usual heap-allocation assumption of
ordinary CLR `newobj`. An earlier low-level heap shorthand proposal would produce
Ptr<T> through allocation plus construction. That proposal is distinct from the
selected high-level new T(...), which returns a managed T&. Its managed allocation,
construction and retention lowering has not yet been encoded in neoIL.

A single permanent `newobj.stack`/`newobj.heap` pair is not enough by itself: the VM
also needs placement construction for preallocated memory, arenas, and foreign
storage. Keep the lower-level operations accessible, whatever shorthand is chosen.

Stack allocation cannot escape a dead frame. Heap allocation alone does not imply
a valid initialized object. Raw zeroing is not sufficient initialization for all
neoCLR types. Construction failure needs explicit ownership of cleanup; object
layout, alignment, partial initialization, allocation-error behavior, and constructor
receiver semantics remain open contracts before implementation.

Today `newobj Name` constructs a record from field values and produces an owned
interpreter value. `heap.new` transfers that value to the GC-managed heap and produces transitional Ref.
Neither instruction proves physical native-stack layout, raw allocation, placement
construction, or reference counting. This proposal does not silently change those
existing instructions.

## Earlier allocator-selection proposal (deferred)

Make allocation select an allocator instead of baking a stack/heap choice into every
type or a permanent pair of construction opcodes. An allocator decides where/how
storage is obtained and participates in an explicit lifetime/access contract. A
frame allocator can be the default. Other choices include unmanaged native memory,
arenas/pools, and memory registered with a collector.

Conceptual assembly, not implemented syntax:

```text
alloc Widget using allocator
construct Widget in storage
```

Exact operands, stack effects, failure behavior, and result types remain open.
The important separation is allocator selection, valid construction, and ownership.
`newobj` may lower through the default allocator; source conveniences must not hide
an incompatible lifetime or identity transition.

An allocator alone does not define all managed-memory behavior. Its allocation
result must have a specified access contract: stable raw pointers, relocatable
handles, or another explicitly supported representation. A moving collector needs
root registration, relocation-aware references, and potentially write barriers or
pinning. Returning an ordinary unrestricted Ptr to movable memory is not sufficient.
These are explicit capabilities/protocols of the selected memory system, not a
universal requirement that every neoCLR value is GC-managed.

Reference-counted Ref can own allocations from a suitable allocator and return
storage through its release contract. The allocator must stay alive for as long as
those allocations need it. Construction failure must release or abandon storage
according to that allocator's rules. Avoid multiple independent lifetime owners
that could reclaim the same allocation.

A managed collector, reference counting, explicit free, or region reset can therefore
be separate choices, provided their contracts compose. The raw VM retains pointer
and memory operations. Higher-level languages can restrict allocator selection or
pointer use without changing the core model.

## Next managed allocation review

The [managed heap strategy](managed-heap-strategy.md) compares the MSIL patterns
for initobj, constructor calls and newobj before selecting explicit heap placement.
Review managed initobj and constructor destinations before adding newval or making
newobj heap-only. Current newobj still produces ordinary values; heap placement
producing T& remains proposed direction.
