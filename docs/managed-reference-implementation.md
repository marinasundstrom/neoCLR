# Managed reference implementation gate

Status: the first checked reference-return slice is implemented for the agreed
[lifecycle model](lifecycle.md). T& locals, managed record-field addresses and guest
reference returns reuse ByRef, ldloca/ldarga, ldflda, ldobj/stobj and ret. Runtime
checks reject references into the returning frame, including field/interface views
and aliases. Explicit managed heap allocation and destruction remain future work.

Use CLR metadata and instruction semantics as the baseline. Ref<T> is a historical
proposal and current arena encoding, not a selected future wrapper. Preview changes
may replace it and break artifacts/APIs. Reject incompatible artifacts explicitly;
unnecessary compatibility layers are not an acceptance requirement. New allocation
and lifetime behavior must have explicit semantics rather than imply compatibility
with execution on an unmodified CLR.

## Observable contract

T(...) constructs an ordinary value, &value refers to that value, and the selected
future new T(...) constructs a managed heap value and returns T&. Passing references
requires no manual retention, release, ownership annotations or invalidation.

A function can return a reference into caller-owned storage, including a field of
a byref argument. It cannot return a reference into its own ordinary local or
by-value argument storage. No implicit promotion on return is selected. The future
high-level language should enforce this dependency direction and the runtime must
fault on violations even without verification. A function-created referent that
must outlive its owner needs managed heap-backed storage. Ordinary by-value returns
transfer values to the caller without inherently requiring heap allocation.

The [reference-return example](../examples/reference_returns.neoil) implements:

```swift
func MakeCounter(counter: Counter&) -> int& {
    return &counter.Age
}
```

Every alias observes the same root and field path. Replacing an owner with another
value of the same type preserves the field's logical location. Rebinding a T& local
changes only that reference binding. Ordinary value copies do not clone reference
targets; Clonable remains explicit.

## Current implementation gaps

| Area | Current implementation | Next gate |
| --- | --- | --- |
| Slots | Stable host cells; references identify a root and field path; every ret rejects current-frame roots | Introduce explicit managed heap targets with independent lifetime provenance |
| Managed heap prototype | Value.Reference is an index into execution-owned storage with Type::Ref | Replace with the selected T& semantics and automatic reclamation |
| Copies | Reference handles retain host cells; slot reads copy values; fields preserve inline semantics | Define copy/release behavior for records containing managed references |
| Metadata and verification | T& locals, record-field addresses and caller-backed returns supported | Define reference-valued fields, broader escape analysis and safe initialization |
| Output obligations | Root/path write history tracks field or ancestor replacement; sibling writes do not satisfy outputs | Extend to additional storage kinds without weakening per-invocation obligations |
| Hosting | Owned inputs only; T& results rejected before guest execution | Define context-rooted handles before persistent reference invocation |

See [slots.rs](../src/slots.rs), [value.rs](../src/value.rs), [vm.rs](../src/vm.rs)
and [input.rs](../src/input.rs). The interpreter's use of host allocations does not
make ordinary guest locals managed heap objects. Strong host cells alone are not
an escape policy; explicit frame-root validation controls returned references.
Reference-valued fields, nested references and erasure remain rejected, preventing
slot-reference cycles in this slice. Array-element references are not implemented.

## Invariants for the next managed heap slice

1. Every reference has one initialized target of the exact type. Field paths and
   interface views preserve the root's storage lifetime. Reused storage cannot
   revive stale references or change the identity observed by aliases.
2. Returning a reference into the current frame always faults. A heap root has a
   separately established lifetime; physical host placement cannot substitute for
   that semantic distinction. Forwarding through another function changes neither.
3. Copying a heap reference preserves its target and retention. Discarding an alias
   releases only its own claim. Bookkeeping registries must not retain dead objects.
4. Reads copy T under ordinary value semantics. Replacing a value accounts for its
   embedded references while preserving aliases to the same logical location.
   Retain incoming state before releasing outgoing state, including self-assignment.
5. Construction and allocation are separate internally. Uninitialized storage must
   not escape as a readable reference. Preserve out/out(true) obligations and field
   identity across ancestor replacement; failed stores must not satisfy outputs.
6. Guest return transfers a valid reference before callee teardown. Host results
   remain rejected until their context and cleanup contracts exist.
7. Dispose/Close resource state is independent of reference liveness. Guest
   destructor dispatch requires separate metadata, ordering and failure rules.

## Boundaries and acceptance cases

Automatic reference counting is a candidate for managed heap reclamation. Cycles,
weak references, concurrency and pinning require decisions before claiming general
deterministic reclamation. Reference-valued fields introduce new retention graphs;
the current acyclic slot subset does not resolve those policies.

The heap_objects limit currently bounds the legacy arena length. A reclaiming
managed heap must separate live-object budgets from identity bookkeeping. Repeated
allocation/release should not exhaust a live-object budget; identity reuse must
remain safe. Frame cells and native pointer-byte budgets remain separate.

A host reference must retain required context/metadata/code or use a checked
context-owned handle. Releasing a Rust handle on an arbitrary thread must not
unexpectedly execute guest destructors. Cleanup scheduling, reentrancy, cancellation
and secondary Faults need explicit contracts. Native backends need equivalent
provenance, roots and copy/release rules; the interpreter layout is not a portable ABI.

Current tests cover caller-root forwarding, field mutation, nested generic String
fields, parent replacement, reference-local rebinding, output aliases, and Faults
for direct/indirect current-frame escapes. They also cover uninitialized targets,
invalid field indices, native result signatures and host-result rejection. Host-cell
release is tested independently of guest readback.

Next add explicit managed allocation producing T&, safe heap-root returns, reference
fields and live-budget tests. Then add destructor ordering and failure tests. Review
CLI-style allocation/construction lowering before adding instructions; no manual
retain/release operations are source prerequisites. Bump the format if obsolete
encodings would otherwise be misinterpreted, and update validation, services, hosting
and examples together.

The next [managed heap strategy](managed-heap-strategy.md) develops storage provenance,
initobj/constructor reuse, explicit heap placement and automatic lifetime handling.
