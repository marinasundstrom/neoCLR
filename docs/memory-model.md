# Memory model layers

neoCLR is a low-level virtual machine. Rust is an implementation language, not the
guest memory model. Rust allocation, borrowing, aliasing, or destruction rules must
not become guest semantics accidentally. Implementations in another host language
should be able to preserve the same explicit VM contract.

The platform direction is value semantics by default, explicit reference access and
runtime-managed deterministic lifetimes. Managed heap allocation produces a reference;
byref parameters can access existing caller values without requiring heap allocation.
Programmers do not manually retain, release or invalidate ordinary managed references.
The compiler/runtime arranges storage and retention to satisfy the reference contract.
Raw pointers and explicit memory operations remain low-level capabilities, including
native interop. See the [managed lifecycle direction](lifecycle.md) for the intended
heap-reference, stack-byref and checked reference-return distinctions and current limits.

This makes neoCLR managed without making it high-level. The VM validates types,
signatures, layouts and pointer operations, while explicitly permitting programs
and native integrations to address memory when their contract requires it.

## Values and explicit capabilities

Values are the default. A type defines the value's shape and available operations;
it does not decide that every instance must have reference identity, be heap-allocated,
or participate in one universal ownership scheme. Passing T copies its value under
the declared copy rules. Allocation, reference access and ownership are separate choices.

| Form | Contract |
| --- | --- |
| T | Ordinary typed value; copied by value |
| T& | Explicit managed access to a live, initialized slot when reading; output contracts can initialize it |
| Interface& | Explicit dispatch view over an implementing value's slot |
| T* or Void* | Low-level address access with explicit validity and lifetime obligations |
| Future retained T& | The same CLR-style managed-reference feature, with automatic lifetime retention |

Capabilities are expressed through declared types, interfaces, parameter/receiver
contracts and explicit operations. Current interfaces are declared on types; this
is not a promise of dynamic per-instance interface attachment. An interface view
adds access to an existing contract without changing its storage lifetime.
Retained managed references extend T&/ByRef without turning all T values into
references. Ref<T> remains a historical proposal and current arena encoding.

A Rust-style borrow checker is not a platform requirement. In the current managed-reference
subset the runtime checks reference identity, liveness, exact type, initialization
and output assignment. Multiple writable aliases are allowed, and sequential reads
observe preceding writes. There is no exclusive-borrow or no-alias guarantee.
The optional IL verifier supplies additional static checks; it is not a source-language
ownership checker. A language may impose stronger rules, but those rules are not
silently inherited by every language targeting neoCLR.

This separates the required behavior from how it is enforced. An interpreter may
track slots dynamically; a future compiler may eliminate checks it can prove redundant.
Both must preserve the reference contract. Longer-lived or stored references still
need specified invalidation/retention rules, and cross-thread access needs concurrency
rules. Avoiding a borrow checker does not remove those design obligations. Neither
case requires selecting exclusive borrowing as the only solution.

## Current priority

The next lifecycle foundation is [managed references and destruction](lifecycle.md).
It prioritizes automatic heap-reference retention alongside stack-backed byref calls
and checked caller-backed reference returns, then user destructor execution. The implemented preview
contracts below remain unchanged. Earlier ownership-operation proposals later in this
document are implementation options, not requirements for manual reference management.

Heap allocation and native pointers are implemented, as are retaining managed
slot references, guest reference returns and interface views. Stabilize their contracts and the Preview 1 programs.
Slot references, field addresses and checked caller-backed returns are implemented;
explicit managed heap allocation,
reference fields and guest destruction are not implemented yet.
Their next implementation gate is described in the
[managed-reference plan](managed-reference-implementation.md); cycle policy and
concurrency remain open decisions.

The precise native allocation contract is in [heap and pointers](heap-and-pointers.md),
and the managed-reference contract is in [slot references](reference-slots.md).
The ideas below remain architectural direction rather than additional preview gates.

## Core VM

Types, storage, addresses, loads/stores, layout, and call frames belong in the VM
model. Raw pointers are fundamental, even when a higher-level language discourages
or restricts their use. A pointer carries no automatic ownership or lifetime policy.

`Ptr<T>`/`T*` values are native addresses, with interpreter side tables for checked
access to its own allocations. Side tracking is a prototype diagnostic mechanism,
not a language borrow checker or mandatory future ownership abstraction.

The original default-local-storage direction remains separate from physical native
stack placement. The current interpreter models frame-owned values but uses host
allocations internally. It does not establish native layout or ABI guarantees.

## A standard low-level surface

The VM should standardize low-level capabilities directly instead of forcing each
language through a policy box. Typed values, fixed and dynamic array views, raw
`Ptr<T>` addresses, explicit allocation regions, layout queries, and checked loads
and stores are platform primitives. A language may expose them safely, restrict
them, or make them ergonomic, but those choices belong to the language profile.

Reference counting, cycle collection, nullability and escape analysis are distinct
design concerns. The selected public reference feature is CLR-style T&/ByRef; the
runtime manages its lifetime automatically. Ref<T> is not a required library wrapper.
Preserve CLR metadata and instruction semantics where they fit, documenting the
extensions needed for retained and escaping references.

## Earlier implementation proposals: explicit VM memory operations

Metadata and IL must preserve the chosen value/reference semantics. The runtime can
automatically implement retention for managed references; explicit retain/release
opcodes are not a settled requirement. Compiler-generated lifetime operations remain
an implementation option. Host-language cloning and destruction are implementation
scaffolding, not the guest lifetime specification.

Keep these concepts distinct:

- `T` describes a value and its representation.
- `Ptr<T>` supplies address access without ownership.
- `T&`/ByRef supplies managed reference access with automatic lifetime handling.
- `Ref<T>` is a historical ownership proposal and prototype arena encoding; it may
  be removed as the managed-reference implementation develops.
- The environment supplies allocation services and may supply optional collection
  facilities. Programs need not select a concrete allocator at every allocation site.
- Construction, copy, move, retain, release, and destruction have explicit contracts
  where applicable. A compiler can emit calls or operations implementing them.

The environment may choose where and how allocation occurs, but its implementation
choices must preserve the declared access and lifetime contract. Environment-provided
services must preserve explicit reference semantics and automatic lifetime handling.

## Retention implementation decisions

A plain value does not inherently require a reference count. Managed references
must preserve their target and identity automatically; reference counting is a
candidate implementation, not a separate source-level Ref<T> obligation. The older
generic ownership-wrapper proposal is superseded as the platform reference model.

Copies and replacement of records containing managed references need specified
retention and cleanup behavior. Those semantics must not emerge accidentally from
Rust Clone or Drop. Weak references, cycles, threading/atomicity, destruction order
and foreign ownership still need explicit contracts. No ambient tracing GC or
Rust-style borrow checker is implied. See the
[implementation gate](managed-reference-implementation.md).

## Current implementation

Current `Ref<T>` values index an execution-owned arena. They support explicit shared
identity, loads, and stores. Allocations are retained until the execution result is
dropped. **There is no reference counting** or per-allocation release within that
arena. This arena is scaffolding, not the final managed memory model.

Instance methods use copied receiver values unless they declare a byref receiver.
Byref receivers access the original initialized slot through the same retaining
managed-reference contract as parameters. Updating a copied receiver does not
write back to the caller. Native pointers have a separate allocation store and
explicit access operations.

Native allocations stay live until explicit free or execution teardown. Copies of
pointers neither retain nor release storage. General generic metadata and lifetime-aware
wrappers can be revisited afterward; reference counting and GC are not prerequisites.

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

See the [managed slot-reference contracts](reference-slots.md) for implemented
reference parameters, output assignment and explicit reference receivers, and the
[Raven-like pseudocode guide](references-in-pseudocode.md) for their language projection.
