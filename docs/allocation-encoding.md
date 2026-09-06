# Allocation encoding: proposals and current scope

Current priority: heap allocation and pointer operations. Ownership management,
reference counting, GC, and allocator-policy composition are deferred. The environment
should be able to choose allocation implementation; an explicit allocator operand at
every allocation site is not a requirement. The earlier alternatives below remain
recorded proposals, not settled encodings or additional work for this milestone.

Separate storage allocation, object construction, and ownership policy. None of
these should be selected by a permanent value/reference bit on the type.

- Stack storage: a core operation such as `localloc`, with defined frame lifetime,
  size, and alignment semantics.
- Heap storage: an explicit allocator call returning an unmanaged pointer (or a
  Result carrying that pointer and a recoverable allocation error). The allocator
  is an implementation/ABI contract, not an ambient managed-object heap.
- Construction: initialize the selected destination and execute the type's constructor.
  Pointer receiver and constructor contracts still need to be implemented.
- Ownership: optionally wrap an allocation in a library abstraction such as a
  reference-counted `Ref<T>`. Allocation does not imply retain/release behavior.

`newobj` can be a convenient typed operation that produces an owned value in the
current frame. This intentionally changes the usual heap-allocation assumption of
ordinary CLR `newobj`. An explicit heap shorthand could lower to allocator plus
construction and produce `Ptr<T>`, rather than automatically producing Ref. Its exact
spelling and whether it is an opcode or assembler lowering remain undecided.

A single permanent `newobj.stack`/`newobj.heap` pair is not enough by itself: the VM
also needs placement construction for preallocated memory, arenas, and foreign
storage. Keep the lower-level operations accessible, whatever shorthand is chosen.

Stack allocation cannot escape a dead frame. Heap allocation alone does not imply
a valid initialized object. Raw zeroing is not sufficient initialization for all
neoCLR types. Construction failure needs explicit ownership of cleanup; object
layout, alignment, partial initialization, allocation-error behavior, and constructor
receiver semantics remain open contracts before implementation.

Today `newobj Name` constructs a record from field values and produces an owned
interpreter value. `heap.new` transfers that value to an arena and produces Ref.
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
