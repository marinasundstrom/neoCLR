# Binding immutability and readonly managed references

This describes the broader design. The first [readonly input-parameter slice](readonly-parameters.md)
and readonly instance receivers are implemented and compared with .NET. Runtime-protected
immutable slots and general readonly storage/return declarations remain planned. Neo still enforces let/var
binding rules in the compiler.

## .NET comparison and decision status

The [initial research](design-research.md#starting-evidence-and-limits) supports the
separation of binding/storage and target mutability, which also appears in C# readonly
behavior. The [parameter slice](readonly-parameters.md#net-comparison-and-decision)
now selects explicit metadata plus live capability narrowing, backed by a bounded
.NET comparison and alias tests. Broader receiver, storage and initialization rules
below remain candidates to evaluate. This is not a general claim of deficiencies in
CLR enforcement or of improved performance.

## Two independent contracts

Binding immutability controls replacement of a stored value. Readonly reference
access controls which operations a particular reference permits. Neither by itself
means deep immutability, exclusive access or an unchanging object graph.

| Concept | Neo responsibility | Runtime and metadata responsibility |
| --- | --- | --- |
| Immutable binding | let syntax, initialization analysis, diagnostics and preventing rebinding | Represent protected slot storage and enforce its initialization/write policy, including writes through aliases |
| Readonly managed reference | Reference syntax and conversions, member selection, rejecting writes and writable receiver calls | Represent a restricted reference capability in signatures and live references; preserve it across calls, returns, storage and projections; reject forbidden writes even without verification |
| Readonly instance receiver | Select methods that permit readonly access and diagnose mutating calls | Publish receiver capability in method contracts and enforce it at dispatch, including overrides and interface implementations |
| Deeply immutable data | Possible later language/library feature | A separate graph/type contract if needed; not implied by let or a readonly reference |

Keep familiar API behavior where it fits, while making these contracts enforceable
across languages. The exact readonly source spelling and metadata encoding remain
open. Do not introduce a CLR-style value/reference type classification to encode
mutability: this is a storage/access property independent of allocation.

## Preserve the distinction between a reference and its target

An immutable binding holding a writable T& cannot be retargeted, but can still
mutate its target. This preserves Neo's current behavior for `let shared = new
Counter(0)`. A mutable binding holding a readonly reference can be retargeted but
cannot write through that reference. The four combinations of binding mutability
and reference permission must remain expressible.

For an immutable owned value, protect the slot and its owned field/array-element
storage from replacement or mutation through derived addresses. A reference stored
inside that value is a separate access boundary: copying it preserves its declared
target permission. Thus protecting a record containing a writable reference does
not freeze the referenced object. Document the same distinction for collections
whose descriptors contain references to managed backing arrays.

A readonly view of mutable storage may observe changes made through another writable
alias. It provides no stability or thread-safety promise. Copying an ordinary value
through a readonly view produces an independent value that may be stored in a mutable
binding; it must not manufacture a writable alias to the original protected storage.

## Enforceable runtime rules

- Permit capability narrowing from writable to readonly. Ordinary casts, interface
  or base projections, generic substitution, erasure and host boundaries must not
  turn the same restricted reference into a writable one.
- Derived addresses into owned fields and array elements retain the restriction.
  Loading a stored reference preserves that reference's own declared permission;
  distinguish this from forming an address of the field that stores it.
- Reject writes through indirect stores, field/element operations, output contracts,
  or mutating receivers when the reference or target storage forbids the operation.
  Reflection, dynamic hooks and future delegate invocation must use the same checks.
- A readonly receiver cannot be passed as writable this. Specify override/interface
  compatibility so dispatch cannot strengthen the caller's required capability.
  Prefer explicit readonly APIs over hidden defensive copies that alter receiver
  identity or behavior.
- Slot initialization and immutability require a checked transition: distinguish
  first initialization from replacement, and ensure aliases cannot bypass protection.
  Specify output-based initialization and constructor behavior before exposing them.
- Define declaration re-entry before protecting loop locals. The existing local.reset
  instruction must not become an unrestricted escape hatch for immutable storage.
  Decide how fresh declaration storage is represented and validated, while retaining
  the existing prohibition on invalidating live references.

Readonly access does not extend lifetimes, promote frame values, or change GC roots.
Reference identity remains the addressed location, independent of its access view.
Native pointers stay a separate low-level interop contract; no managed mutability
guarantee should be claimed for arbitrary native writes.

## Implementation order

1. Specify the capability matrix, owned-storage boundaries, metadata and conversions.
   Include reference fields, reference-array elements and list backing storage in the
   examples so shallow protection is unambiguous.
2. Implement readonly reference/receiver enforcement and propagation in the runtime,
   verifier and reflection. Test raw IL/artifacts that attempt to recover write access.
3. Add protected-slot initialization and declaration re-entry rules, including aliases,
   outputs and loops. Preserve frame provenance rather than adding silent promotion.
4. Project both contracts into Neo with diagnostics and examples for all four binding/
   reference combinations. Review predicates, equality, inspection and other library
   receivers for readonly contracts as their actual implementations permit.

Explore this foundation before inheritance dispatch and closure capture contracts
are fixed. Inheritance, nullability, delegates, async and dynamic binding must preserve
these permissions rather than invent separate mutability systems. See the
[platform roadmap](roadmap.md), [reference semantics](managed-reference-semantics.md)
and [API design](api-design.md).
