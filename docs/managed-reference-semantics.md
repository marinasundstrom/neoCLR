# Managed references: runtime behavior and Neo projection

This describes the implemented model. A type T defines a value; T& chooses managed
reference access to a value. The same T can have frame-owned or managed heap storage.
The reference's type does not select allocation, require a universal Object base,
or impose manual retain/release, invalidation or raw-pointer operations.

## Storage owns the lifetime

| Storage | Owner and lifetime | Reference return rule |
| --- | --- | --- |
| Frame-owned value | The declaring call frame owns its storage; references do not extend that frame's lifetime | A callee may return a reference into an active caller's storage, but cannot return an address into its own locals or by-value argument copies |
| Managed heap value | The collector owns its storage independently of the allocating frame; reachability keeps it alive | A heap-backed reference may survive the allocating function and be returned |

An interior reference such as `&counter.Age` follows the owner's root. For a heap
object, that field reference keeps the entire allocation and its reachable graph
alive, even without another reference to the object. For a frame value, it remains
bounded by the owning frame. Forwarding or copying the reference preserves this
provenance; it neither promotes storage nor changes the lifetime rule.

The lifetime of a local reference binding is distinct from its target's lifetime.
Dropping one heap reference does not invalidate other aliases. Losing the last path
to a heap object makes it eligible for GC, not necessarily immediately reclaimed.
Runtime checks enforce target type, initialization and validity; each return checks
that the actual target does not belong to the returning frame, even through aliases.
The optional verifier and Neo catch some failures earlier, but do not replace runtime
validation. Stored references in record fields/erased payloads currently must be
heap-backed, preventing indirect frame escapes.

Here, “stack value” means guest frame-owned storage. The interpreter's host allocations
are an implementation detail, not permission to escape. Native register/stack placement
is a backend choice. Lexical blocks currently control name visibility, not separate
runtime storage lifetimes. Neo conservatively rejects taking addresses of block-local
values while allowing references to existing outer values and managed heap targets.
Automatic guest destructors and deterministic block cleanup remain future work.

## How Neo projects the runtime

Managed references behave transparently at the source level: reads access their value,
assignments update their target, and forwarding preserves aliasing. Neo emits the
underlying address/load/store operations; users never need unary `*` for a managed T&.

| Neo operation | Existing runtime mechanism |
| --- | --- |
| `var local = Counter(0)` | Construct an ordinary value with newobj and store it in frame-owned local storage |
| `let shared = new Counter(0)` | Construct the value, then heap.new establishes a managed heap root and returns Counter& |
| `let age = &local.Age` | ldloca and ldflda form an int& with the original frame root |
| `let age = &shared.Age` | Load the existing Counter& and use ldflda, retaining its heap root |
| `age = age + 2` | Load the reference, read with ldobj, calculate, then update the target with stobj |
| `let copy: int = age` | Read through int& and store an ordinary copied int |
| Pass/return T& | Forward the reference, preserving its target and runtime lifetime checks |
| Pass/return T | Copy the value, reading through a managed reference when needed |

For example, this function can refer to either kind of storage:

```text
func Age(counter: Counter&) -> int& {
    return &counter.Age
}
```

When called with `Age(&local)`, the result still depends on local's frame. When called
with `Age(shared)`, it retains the managed heap allocation. Both results can be used
as `age = age + 2`. The [runnable counter example](../examples/source/counter.neo)
demonstrates value copies, caller-owned references and returned heap field references.

A factory that returns a reference uses explicit heap storage:

```text
func MakeCounter() -> Counter& {
    return new Counter(40)
}
```

Returning `&local` from the function that owns local is invalid. Returning local by
value remains valid and does not require heap allocation.

Inferred bindings preserve reference types. `let` prevents rebinding but allows target
mutation through T&. Ordinary reference-to-reference assignment copies the right
referent into the left target. Retargeting a `var` reference binding is explicit:
`reference = &other`. `&reference` forwards its existing reference; it is not T&&.
See the [Neo access rules](neo.md#managed-references-are-transparent-pointers-are-explicit)
for argument, inference and assignment details.

## Pointers and remaining boundaries

Raw T* pointers are different. They have explicit low-level validity, access and
allocation obligations, do not automatically retain managed objects, and are not
subject to Neo's automatic managed-reference reads. Neo does not yet expose pointer
syntax; the runtime already has a separate pointer/interop path. Future native sharing
of managed objects needs a defined pinning/handle contract.

GC manages memory, not timely external-resource completion. Dispose/Close are explicit
resource protocols; reference liveness does not mean a resource remains open. The
current host can inspect a returned heap reference through its owning Execution,
but a copied host reference is not an independent GC root registration, and reference
inputs across executions remain unsupported.

Further details: [slot operations](reference-slots.md), [heap references](heap-references.md),
[collector](garbage-collection.md), [lifecycle and cleanup](lifecycle.md),
[native pointers](heap-and-pointers.md), and [Neo grammar](neo-grammar.md).
Implemented behavior is covered by the [Neo access tests](../tests/neo_managed_access.rs)
and the [counter regression](../tests/neo.rs).
