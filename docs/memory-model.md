# Memory model layers

neoCLR is a low-level virtual machine. Rust is an implementation language, not the
guest memory model. Rust allocation, borrowing, aliasing, or destruction rules must
not become guest semantics accidentally. Implementations in another host language
should be able to preserve the same explicit VM contract.

## Current priority

Focus on heap allocation and executable pointers. Defer reference counting, GC,
automatic destruction, ownership-aware copy/move behavior, and language lifetime
integration. The ideas below are recorded architectural direction, not requirements
to finish before the raw allocation/pointer slice.

That slice should define allocation size/alignment, initialization, pointer values,
addressing and indirect access, explicit release, and invalid-access behavior. Exact
opcodes and pointer representation are still open; accepting pointer signatures
alone does not implement these operations.

## Core VM

Types, storage, addresses, loads/stores, layout, and call frames belong in the VM
model. Raw pointers are fundamental, even when a higher-level language discourages
or restricts their use. A pointer carries no automatic ownership or lifetime policy.

`Ptr<T>`/`T*` are now recognized in signatures. The executable pointer layer remains
to be designed: address width, alignment, layout, pointer arithmetic, zero-sized
values, null addresses, validity, allocation/deallocation, and boundary failures.
These contracts must be explicit. They should not be inferred from Rust references
or require a Rust borrow checker. Checked borrowing can later be an additional
facility for languages that want it.

The original default-local-storage direction remains separate from physical native
stack placement. The current interpreter models frame-owned values but uses host
allocations internally. It does not establish native layout or ABI guarantees.

## Deferred philosophy: explicit VM memory operations

Memory management should be explicit in metadata and IL. Languages may hide Ref
wrappers or generate lifetime operations automatically, but their compiled output
must make the required behavior explicit. The runtime must not infer ownership from
a plain value's type or silently choose retain/release behavior. Host-language cloning
and destruction are implementation scaffolding, not the guest lifetime specification.

Keep these concepts distinct:

- `T` describes a value and its representation.
- `Ptr<T>` supplies address access without ownership.
- `Ref<T>` explicitly represents counted ownership when that abstraction is built.
- The environment supplies allocation services and may supply optional collection
  facilities. Programs need not select a concrete allocator at every allocation site.
- Construction, copy, move, retain, release, and destruction have explicit contracts
  where applicable. A compiler can emit calls or operations implementing them.

The environment may choose where and how allocation occurs, but its implementation
choices must preserve the declared access and lifetime contract. Environment-provided
services are not a reason to make guest ownership implicit.

## Deferred ownership abstractions

A plain value should not automatically contain a reference count. Ref<T> is an
explicit candidate for counted ownership: creation establishes a reference, copying
retains it, release decrements its count, and final release destroys the value and
returns storage through its allocation environment.

Reference counting need not be built into the runtime as the universal object model.
The VM can supply counters/atomics and construction/destruction facilities, or
specialized retain/release operations, while a generic library wrapper owns the
policy. How much is implemented in the library versus optimized by the VM remains
open. Real generic definitions are needed; the current special-cased Ref signature
is not that general generic facility.

A record containing Ref fields would need explicit copy/destruction behavior for
those fields even though the record itself is not counted. That behavior must not
emerge accidentally from Rust Clone or Drop. Moves, weak references, cycles,
threading/atomicity, destruction ordering, and foreign ownership all remain deferred.

Optional collection services must preserve explicit ownership contracts. No ambient
tracing GC, Rust-style borrow checker, or count header on every object is implied.

## Current implementation

Current `Ref<T>` values index an execution-owned arena. They support explicit shared
identity, loads, and stores. Allocations are retained until the execution result is
dropped. **There is no reference counting**, per-allocation release, or raw guest
address today. This arena is scaffolding, not the final managed memory model.

Instance method receivers are currently read-only value snapshots. They do not
establish a borrow model or mutable receiver semantics. No guest pointer operation
is implemented merely because pointer signatures are accepted.

The next implementation target is heap allocation and pointer operations. General
generic metadata and lifetime-aware wrappers can be revisited afterward; reference
counting and GC are not prerequisites for that target.

## Deferred environment and allocator integration

The environment chooses allocation implementation and may integrate an allocator
with a collector. Explicit allocator selection may be useful later, but is not
required at every source or IL allocation site. A frame allocator was an earlier
proposal, not a committed universal default for this heap/pointer milestone.

Movable storage needs roots/handles, relocation support, or pinning; a naked stable
pointer contract cannot silently become a movable reference. Ref counting could own
storage from a suitable allocation environment, whose lifetime must outlast those
allocations. Cleanup on construction failure and avoiding conflicting reclamation
policies also need contracts. These integration questions are recorded for later.
See [allocation encoding](allocation-encoding.md).
