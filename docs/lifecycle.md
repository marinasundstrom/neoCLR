# Managed references, lifetimes and cleanup

Status: agreed platform direction with implementation decisions still open. Values
are the default; reference semantics are chosen explicitly; the runtime manages
reference lifetimes. Clonable, Disposable and Closable are implemented ordinary
interfaces. T& locals, managed field addresses and checked guest reference returns
are implemented. Returning a reference into the current frame faults. Heap allocation now returns T& directly, with tracing through field and interface
views. Stored references must be heap-backed. Guest destruction remains future work.

Ref<T> was a proposal, not a selected public reference abstraction. The source
direction is T&. Prefer .NET CLR instructions and semantics wherever they fit;
justify deviations by the improved value/reference/lifetime contract. Preview
artifacts and APIs may be broken to implement that model. Obsolete encodings need
not be preserved behind compatibility wrappers.

Reuse CLR managed references, represented by T&/ByRef, as the explicit reference
feature. Extend the existing metadata, verification and address/load/store paths
rather than introduce a parallel public ownership type. Frame-backed and retained
storage implement the same reference abstraction. Automatic retention and escapes
beyond a defining frame require documented semantic extensions; they do not imply
compatibility with execution on an unmodified CLR.

## Values and two reference uses

| Form | Programmer's choice | Runtime responsibility |
| --- | --- | --- |
| T | Pass or store a value | Preserve value-copy semantics and clean up its contained state at the appropriate lifetime boundary |
| Managed heap reference to T | Allocate a value with shared identity and hold a reference to it | Keep the value alive while reachable through live retaining references; reclaim unreachable storage through tracing GC |
| T& parameter or receiver | Access an existing value, including one in a caller's stack frame | Preserve identity and mutations through nested calls, with automatic validity checks and no manual reference cleanup |
| T* or Void* | Use low-level memory or native interop | Enforce the declared raw-pointer boundary; a raw address does not automatically retain its target |

Heap allocation in the managed programming model produces a managed reference.
The referenced value has the same type T that could otherwise be held directly;
there is no class/struct bit that forces allocation policy onto the type.
heap.alloc/free remain raw memory operations. heap.new directly produces a GC-backed
T&; format 5 removes Ref and heap.load/store. See [heap references](heap-references.md).
The selected high-level spelling is new T(...); its CLI-based IL lowering remains open.

A byref parameter can refer to a caller's local and be passed further down the call
chain. The programmer explicitly chooses reference access, then uses that reference
without retain/release calls, allocator annotations, moves or manual invalidation.
Multiple writable aliases remain permitted. A language may offer stronger checks,
but exclusive borrowing and Rust-style lifetime annotations are not platform requirements.

An ordinary by-value copy copies embedded managed references as references: it does
not clone their targets. The runtime accounts for any required retention. Inline
value fields retain their value semantics. Physical stack/heap location and whether
reference access retains storage are separate concepts; a byref can also access a
value backed by managed heap storage when that access contract is implemented.

## Choose storage to match the intended lifetime

The developer chooses whether an object is local to a method/block or allocated on
the managed heap. Many objects are temporary working values that need only exist
within one method or block. Ordinary local value construction expresses that scoped
lifetime; explicit managed heap construction expresses an independent lifetime.
Both support explicit reference access. Choosing a reference does not itself request
heap allocation or extend the lifetime of its owner.

A reference to a scoped value can be passed to code that completes within the
owner's lifetime. It cannot survive that scope. A heap reference can outlive the
allocating method and remains valid while reachable. Developers reason about the
required lifetime without manually freeing managed objects or invalidating aliases.

The prototype enforces frame lifetime boundaries today. Lexical block lifetimes,
earlier local cleanup and their runtime representation remain future work for the
language/compiler and runtime contract; a block is not currently a distinct runtime
scope. Physical native-stack placement also remains a backend implementation task.
Optimizations may change placement while preserving the declared lifetime rules.

## High-level allocation syntax

The selected source direction distinguishes construction, reference formation and
explicit managed heap allocation:

```swift
let value = Counter(0)          // Counter: ordinary value construction
let reference = &value         // Counter&: reference to the existing value
let allocated = new Counter(0) // Counter&: new value in managed heap storage
```

| Expression | Meaning |
| --- | --- |
| T(...) | Construct a value; physical placement is chosen by the implementation |
| &value | Refer to that existing value, preserving its identity and required lifetime |
| new T(...) | Explicitly construct a new value in managed heap storage and return T& |

Automatic placement is the default. A value may live in registers, stack storage
or other suitable storage; T(...) is not a promise of a native stack allocation.
new explicitly requests managed heap allocation, with automatic reference retention
and cleanup rather than a raw pointer or manual free. No type declaration chooses
class-versus-struct allocation semantics. This source syntax is selected direction,
not implemented compiler syntax or a change to neoIL newobj.

## Explicitly copy an existing value to the heap

A future language operation may copy a scoped value into a new managed heap object.
This supports starting with a local value and choosing independent storage later.
The original remains in its scope; the heap copy has a new identity and GC-managed
lifetime. Existing references continue to address the original, so this operation
must not silently promote storage or retarget aliases.

Use ordinary value-copy semantics: inline fields are copied and embedded managed
heap references preserve their targets. Deep or custom copying remains an explicit
Clonable policy. Reference-containing copies still need lifetime validation: a
reference into scoped storage cannot become a longer-lived heap edge merely because
its containing value was copied. Reference-valued fields currently require heap-backed targets.

The prototype can already load a local value and pass that copy to heap.new, yielding
T& directly. The future source spelling remains to be defined. This does not authorize returning &local or automatically
converting a scoped reference into a heap reference.

## Storage and escape

A reference can depend on storage in an active outer frame. Passing a caller's
local by reference does not require managed heap allocation or manual lifetime
handling. Returning that reference, or a reference to one of its fields, to the
caller is valid because the caller still owns the storage:

```swift
func MakeCounter(counter: Counter&) -> int& {
    return &counter.Age
}
```

The same function must not return an address into its own frame. This includes
its ordinary locals, by-value argument copies and fields nested inside either.
Returning &local does not implicitly promote that local. The future high-level
language should reject this escape; the runtime must validate the actual target
and fault even when optional verification is skipped. A helper or reference-valued
local cannot hide the target's owning frame. Each frame validates again on return.

If a function creates an object whose reference must survive that function, the
object needs explicit managed heap-backed storage. The selected source direction
is new T(...), producing T&; its managed allocation lowering is still future work.
Ordinary by-value returns transfer a value into the caller and do not impose this
heap-allocation requirement. A new stack slot containing a reference to a heap
object does not make that object's lifetime depend on the slot.

The interpreter represents ordinary slots using allocated host cells. That physical
implementation detail does not turn guest locals into managed heap objects or make
their addresses eligible to escape their owner. Runtime return checks follow the
root slot and field path. The [reference-return example](../examples/reference_returns.neoil)
implements the caller-owned field case using ldloca, ldflda and ret.

A managed field reference continues to address the same field path when the owner
is replaced by a value of the same type. It does not become a detached copy.
Raw native pointers remain a separate capability and do not inherit these retention
or relocation rules. Future pinning and interop bridges need their own contracts.

## Value lifetime, collection and destruction

Frame-owned values end their storage lifetime when the frame exits. Scoped byrefs
preserve access through active callers; returning an address into the current frame
faults. Managed heap storage instead follows reachability and tracing GC. Unreachable
objects, including cycles, are eligible for collection; the last reference loss does
not promise immediate reclamation or resource cleanup.

Copies, stores and returns must preserve reference identity and root visibility.
There is no source-level retain/release or manual invalidation requirement. The
implemented [collector](garbage-collection.md) scans active frames and heap graphs;
heap-backed T& now uses the same checked address feature.

Guest destructors, heap finalizers and automatic scope cleanup are not implemented.
Any deterministic value destruction needs ordering, partial initialization, reentrancy
and failure rules. Heap finalization would need additional resurrection and scheduling
contracts and must not be presented as timely cleanup. Dispose/Close remain the
explicit resource protocols. Clonable is not an implicit substitute for value copying.

## Cloning, disposal and closing

| Operation | Contract |
| --- | --- |
| Ordinary value copy | Copy the value, automatically preserving the lifetime of embedded managed references |
| Clonable<T>.Clone() | Explicitly produce a clone under the type's documented duplication/sharing policy |
| Disposable.Dispose() | Explicitly release resources or discard state, leaving a valid disposed value; repeated disposal is harmless |
| Closable<E>.Close() | Complete a resource-specific operation with System.Result<Void,E> for expected failure |
| Destruction | Runtime-triggered end-of-lifetime cleanup under declared destruction metadata |

The [cloning](cloning.md) and [cleanup](disposal.md) interfaces use byref receivers
so calls access the original value without first copying it. Their implementation
requires ordinary IL and explicit conformance. It does not yet provide destruction
hooks, using syntax or automatic scope cleanup. Managed heap reachability is
handled separately by the collector.

A destructor may share release logic with Dispose, but implementing Disposable
does not alone register that hook. Closing or disposing a reference target may
change state visible through its aliases without destroying their managed handles.
The library must define subsequent operations on the closed/disposed value.
Reference liveness is not a promise that an external resource remains open.

Close can report a failure such as flushing output. Destruction must not silently
stand in for successful flushing, committing or publication. For the initial close
protocol, a previously successful Close remains successful on repetition, including
after later disposal. Otherwise, Close on a disposed value returns a documented E.
A failed Close leaves a value that can be disposed; retryability is resource-specific.

## Fault and native boundaries

Result.Error is an ordinary return and participates in normal cleanup. Terminal
Faults and cancellation need a separate cleanup execution policy before automatic
user destructors are promised there. The prototype currently reclaims tracked host
allocations on teardown but does not execute guest cleanup handlers.

Specify destructor instruction budgets, secondary failures, native failures and
host-resource teardown without introducing catchable guest exceptions implicitly.
Deterministic lifetime management must not imply successful external side effects
or rollback. Raw memory release remains distinct from destroying typed managed values.

## Next implementation gates

1. Specify managed allocation and reference identity/retention, keeping ordinary
   value copies and explicit T& calls as the default contracts. Audit all paths that
   copy, store, return or erase reference-containing values and cross host boundaries.
2. Extend the implemented collector to unified heap-backed T& and safe call-scoped
   access. Test reachability through aliases, nested fields and returns before
   considering guest finalization.
3. Build on checked T& locals, field addresses and caller-backed guest returns.
   Introduce managed heap targets without weakening the rule against returning
   addresses into the current frame. Allocation optimizations must preserve that rule.
4. Add destruction metadata, initialization tracking and execution/failure rules.
   Demonstrate real resource release alongside Disposable/Closable and explicit Clone.
5. Build on implemented cycle collection; define weak references, concurrency and
   native pinning before exposing those features. Unique ownership may be a later optional
   capability; it is not the foundation users must adopt to use managed references.

Interpretation, JIT and AOT must preserve the same observable value/reference and
lifetime behavior. Existing Ref arena artifacts and raw pointer-based collections
may be replaced or redesigned. Declare incompatible format changes and require
reassembly instead of preserving proposals that no longer fit.

The [implementation gate](managed-reference-implementation.md) records current code
gaps, reference identity/retention invariants and the first acceptance workloads.

The proposed [managed heap strategy](managed-heap-strategy.md) separates frame and heap
roots behind T&, and reviews initobj plus construction into supplied storage before
introducing a new value-construction instruction.
