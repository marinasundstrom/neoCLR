# Binding immutability and readonly managed references

Decision updated 2026-09-08: immutable bindings remain a high-level language feature.
Neo enforces let/var and its assignment rules. There is no current requirement for
runtime-protected immutable local slots, write-once storage or initialization regions.
Those mechanisms are removed from the immediate plan.

Readonly managed-reference access is a runtime contract because it crosses calls,
returns, storage and language boundaries. The [input/receiver implementation](readonly-parameters.md)
and [stored/returned signatures](readonly-storage.md) enforce this capability.

## Separate responsibilities

| Characteristic | Meaning | Responsibility |
| --- | --- | --- |
| Binding immutability | Whether a source binding may be reassigned | Language compiler and semantic model |
| Readonly managed reference | Whether this access path may write the target or call a writable receiver | Declared reference signature, verifier and runtime capability |
| Nullability | Whether a declared value/reference can be in a null state | Proposed explicit type-signature contract; not implemented |
| Deep immutability | Whether an entire reachable object graph can change | Separate potential feature, not implied by these contracts |

Ordinary object/member reflection does not expose local bindings as fields. Debugger
inspection is a separate tooling concern and does not justify a general runtime
immutable-slot mechanism. A compiler can enforce bindings without requiring every
language to share its source-level binding abstraction.

A local containing a readonly reference can be assigned another compatible reference
by IL. Neo may prohibit that rebinding for let, while permitting it for var under its
existing reference-assignment rules. The runtime still rejects writing through the
readonly reference. A let holding a writable reference can mutate its target.

Readonly remains shallow. A readonly view of a record or array cannot write owned
fields/elements through derived addresses, but loading a separately stored reference
preserves that reference's own access permission. Other writable aliases can change
the same object. Readonly is not purity, exclusivity, thread safety or stable data.

## Runtime enforcement

Writable references may narrow to readonly without changing the addressed location,
owner, lifetime or allocation. Restricted references cannot satisfy writable stores,
calls or returns. Interface projections and generic substitution preserve the access
contract. Current-frame escape checks and heap-only embedded references remain in force.

Existing output contracts still concern assignment of an addressed value. They do
not grant permission to write through a readonly reference, and do not imply that
locals are immutable. No new scope region or local.reset policy is introduced.

A future JIT must derive optimization facts from validated IL/contracts or its own
analysis. It cannot assume that all languages enforce Neo let rules. Runtime readonly
permissions also do not justify treating aliased memory as constant.

## .NET comparison and scope

The [readonly research](readonly-parameters.md#net-comparison-and-decision) compares C#
language rules, CLR controlled-mutability addresses and runtime-enforced access.
Source binding restrictions can remain language-level while cross-language reference
permissions are enforced below the compiler. This avoids implementing universal
storage protection without a concrete consumer.

Revisit stronger runtime storage contracts only if a demonstrated platform requirement
needs them. They are not prerequisites for inheritance, nullability, closures or a JIT.
The current sequence continues with reference contracts and shared type relationships,
as described by the [groundwork review](runtime-groundwork-review.md).
