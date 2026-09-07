# Managed references, deterministic lifetimes and cleanup

Status: agreed platform direction with implementation decisions still open. Values
are the default; reference semantics are chosen explicitly; the runtime manages
reference lifetimes. Clonable, Disposable and Closable are implemented ordinary
interfaces. Managed heap retention, escaping stack references and automatic guest
destruction remain future work. The current Ref arena is a prototype limitation.

## Values and two reference uses

| Form | Programmer's choice | Runtime responsibility |
| --- | --- | --- |
| T | Pass or store a value | Preserve value-copy semantics and clean up its contained state at the appropriate lifetime boundary |
| Managed heap reference to T | Allocate a value with shared identity and hold a reference to it | Keep the value alive while reachable through live retaining references; release it under the selected deterministic lifetime contract |
| T& parameter or receiver | Access an existing value, including one in a caller's stack frame | Preserve identity and mutations through nested calls, with automatic validity checks and no manual reference cleanup |
| T* or Void* | Use low-level memory or native interop | Enforce the declared raw-pointer boundary; a raw address does not automatically retain its target |

Heap allocation in the managed programming model produces a managed reference.
The referenced value has the same type T that could otherwise be held directly;
there is no class/struct bit that forces allocation policy onto the type.
Current heap.alloc/free remain raw memory operations, while heap.new/Ref expose
an execution-retained arena. Neither existing mechanism is silently redefined by
this proposal; a managed allocation spelling and artifact transition remain open.

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

## Storage and escape

Call-scoped references to caller locals do not require heap allocation merely
because a callee uses them. When the call returns, there is no reference handle
cleanup for the caller to perform. This is the implemented T& subset today.

The intended broader direction permits a reference to keep a value alive beyond
its creating scope. The compiler/runtime must then arrange suitable storage, for
example by allocating an escaping local in retained storage from the start or
promoting it while preserving the identity seen by every existing reference.
Escape analysis may avoid unnecessary allocation; correctness cannot depend on
requiring programmers to choose the physical placement themselves.

Conceptual source, not implemented syntax:

```text
func MakeCounter() -> reference Counter {
    let counter = Counter(0)
    return reference counter
}
```

The result refers to the same counter, whose lifetime outlasts the function.
The encoding of an escaping/retaining reference versus a call-scoped T& remains an
implementation decision. Current T& returns and stored byrefs are still rejected;
accept them only when retained storage and alias-preserving promotion exist.

Raw pointers require a stable-address or pinning contract at native boundaries.
Ordinary managed references should continue to work without exposing those details.
An unrestricted native address must not be relocated underneath foreign code.

## Deterministic destruction

Scope exit ends that scope's claim on a value. A local with no surviving references
is destroyed there; a retained value is destroyed after its last lifetime-retaining
reference is released. Passing a scoped byref down an active call chain preserves
access to the caller's value until the call completes.

Destruction belongs to the value's stored instance. Releasing one reference does
not destroy the target while another live reference keeps it alive. Replacing an
inline field or local must account for the outgoing value and any referenced
storage separately. Calls and returns transfer their operands/results without
premature destruction. There is no requirement to call an explicit invalidation
instruction in source code or manually manage reference handles.

The runtime and compiler may use internal lifetime metadata or generated operations
to implement these rules. Their spelling and representation are not selected yet.
The uncommitted endloc experiment was set aside; it is not a prerequisite or a
published opcode. Build the managed-reference behavior before choosing additional
lifetime instructions.

Automatic reference counting is a candidate implementation for deterministic
release, not an obligation to maintain a count on every ordinary stack value.
Counters or equivalent retention mechanisms belong to managed shared storage.
Cycles remain a real open decision: simple counting cannot reclaim a cycle.
Weak references, restrictions or supplementary collection must be assessed against
both usability and the promised timing of destruction. Do not claim universal
deterministic reclamation before the cycle policy is resolved.

A destructor observes its still-valid fields before their automatic cleanup.
Destruction ordering, partial initialization, reentrancy and resurrection need
explicit rules before executing user-defined destruction bodies. Ordinary value
copying and reference retention must work for nested records, strings and active
union payloads. Clonable is not an implicit substitute for those runtime contracts.

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
hooks, using syntax, managed heap retention or automatic scope cleanup.

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
2. Implement automatic lifetime retention/release for managed heap references and
   safe call-scoped access to their values. Test final-reference release through
   aliases, nested fields and returns before attaching user destructor bodies.
3. Define stack-reference escape representation and implement retained placement or
   promotion without changing alias identity. Preserve simple stack-backed byref calls.
4. Add destruction metadata, initialization tracking and execution/failure rules.
   Demonstrate real resource release alongside Disposable/Closable and explicit Clone.
5. Resolve cycles, weak references, concurrency and native pinning before broadening
   the lifetime guarantee to those cases. Unique ownership may be a later optional
   capability; it is not the foundation users must adopt to use managed references.

Interpretation, JIT and AOT must preserve the same observable value/reference and
lifetime behavior. Existing Ref arena artifacts and raw pointer-based collections
need an explicit migration rather than silently acquiring different ownership rules.
