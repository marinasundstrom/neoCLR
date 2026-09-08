# Addressed access: design history and remaining proposals

Status: original design proposal, partly superseded by the implemented
[managed slot-reference contract](reference-slots.md) and [verifier](verification.md).
Use those documents for current behavior. This document preserves the rationale
and broader proposals; its recommended first scope is not the implemented subset.

Implemented: T& parameters and locals, ldloca/ldarga, managed record-field ldflda,
ldobj/stobj, checked caller-backed reference returns, explicit byref receivers,
out/out(true) assignment contracts and managed interface views. Runtime return
checks reject current-frame roots, including field and interface views.
Deferred: readonly permissions, reference-valued aggregate fields, managed heap
allocation, array-element references and native-address bridges. Taking the address
of an uninitialized local is permitted; reading it or forming a field reference
requires initialization. The broader rules below preserve earlier proposals; use
[slot contracts](reference-slots.md) for implemented field-path behavior.

## Recommended boundary

Keep native T* and introduce checked addressed access, provisionally T&, as distinct
capabilities. T& does not own T, retain it, choose its allocator, or imply a moving
collector. It does not impose exclusive access. Its purpose is to name existing,
typed storage while retaining enough information to enforce access and lifetime rules.

The VM should enforce the meaning of this capability. A language can impose stronger
rules but must not be the only component preventing malformed IL from returning a
reference into a dead frame. Raw pointers retain their separate low-level contract;
introducing a checked facility does not make all pointer code verifiable.

| Form | Copying it does | Permitted access | Lifetime responsibility |
| --- | --- | --- | --- |
| T | Copies a plain value under its value contract | Operates on that copy | Follows the value's storage |
| T* | Copies an address | Existing native pointer operations | Explicit; no ownership is acquired |
| Writable T& | Copies an access path to the same location | Read and replace T, or access its fields | Must not outlive the referenced location |
| Read-only T& | Copies a read-only access path | Read T or derive read-only field access | Same lifetime restriction |
| Future Ref<T> | Not yet specified as executable counted ownership | Governed by that wrapper | Separate future ownership contract |

Read-only means the access path cannot write. It does not freeze the underlying
value: a writable alias may change it. It also does not make native pointees stored
inside T read-only. Converting writable access to read-only access is reasonable;
the reverse must not be implicit. Neither form implies a no-alias optimization promise.

## First implementation scope

Recommend starting with references to fully initialized locals, by-value argument
slots, and their fixed-layout record fields. Here fixed layout means a stable field
shape for a closed type; an interpreter-backed String field need not have a native
byte layout. Generic record fields use their substituted type identities.

References may be held in local reference slots and passed to ordinary callees,
including further forwarding. A callee receiving T& accesses the caller's location.
Taking the address of a by-value parameter instead accesses the callee's own copied
parameter slot. These are observably different operations.

For this first subset, reject reference returns, references inside aggregate fields,
heap/global storage of references, and references to reference slots. Do not allow
converting T& to/from T*, integer conversions, pointer arithmetic on T&, or byref
arguments to native imports. These are staged omissions, not claims that such
features can never be supported. In particular, returning a reference to caller-owned
storage should eventually be expressible with a suitable lifetime contract.

The initial slice excluded byref-containing generic arguments. The managed-array
collection slice now permits managed-reference type arguments and array elements,
with heap-provenance checks on aggregate stores; direct nested managed references
and native pointer storage of reference-bearing signatures remain unsupported.
Existing generic methods whose receiver is addressed, or whose argument is a reference
to a closed generic value, remain useful without that broader feature. Longer term,
restrictions should follow storage/lifetime capabilities instead of recreating a
permanent struct/class category.

References to native heap locations can follow after invalidation and explicit-free
interaction are specified. Existing raw heap pointers remain available throughout.
No collection, retention, or pointer pinning is needed for the initial local subset.

## Location identity and aliasing

A proposed interpreter representation is a frame identity, slot category/index,
a field path, and access permission. This is an implementation suggestion, not a
native ABI or required runtime object layout. Frame identities must distinguish
successive invocations so a stale location cannot revive when a slot is reused.

A reference names the slot/location rather than a detached copy of its contents.
Replacing a local's value with another value of the same closed type preserves that
location. An existing reference to a field observes the corresponding field of the
replacement. This relies on a stable field shape; active union storage and references
to conditional payloads require additional invalidation rules and are excluded here.

Two writable references to the same location are permitted. Writes through either
one are visible to subsequent reads through either one in single-threaded execution.
The future compiler must not infer exclusivity from writable access. Threading,
atomics, races, and cross-thread references need a separate memory-model contract.

A read-only reference can likewise observe changes through another alias. It cannot
be treated as a permanently cached value. Void locations remain distinct logical
locations even though their payload occupies no bytes; no native-address uniqueness
or reference-equality instruction is promised by this proposal.

## Receiver semantics and language lowering

Use explicit value, read-only addressed, and writable addressed receiver modes in
metadata. Source syntax can remain familiar. Illustrative source, not a selected
language grammar:

```text
point = Point(1, 2)
copy = point
point.SetX(10)
```

The proposed lowering is:

1. Construct a complete Point value and store it in point's local slot.
2. Copy that value to copy's slot.
3. Pass writable addressed access to point's slot as SetX's receiver.
4. Replace the X field at that location.

The observable result is point.X = 10 and copy.X = 1. Passing a copy of point to a
value-receiver method would not provide permission to modify point's original slot.
Calling a mutable method on a temporary requires an explicit compiler rule: either
materialize a temporary location or reject the call. The VM should see the chosen
location, not guess a write-back destination.

This is a semantic lowering contract only. Do not assign opcode encodings or silently
repurpose current value-update stfld and field-based newobj. Existing module semantics
need a versioned migration path if their normalized operations change.

## Verification and initialization

Before addresses, implement stack states at control-flow joins, definite assignment
of locals, argument/local bounds, and return checks. Add reference permissions and
origins to those states when addressed access is introduced. A merge must retain all
possible origins and may permit later access only when each remains valid. It must
not upgrade a read-only path to writable access or lose a shorter lifetime.

Normal T& access starts with a fully initialized T. Address-taking must not convert
an uninitialized slot into a readable value. Begin by rejecting ordinary address-taking
of uninitialized slots; write-only initialization access is a later, explicit contract
needed for constructors and output parameters. Do not infer it merely from the callee's
name or pretend that a zeroed allocation initialized an arbitrary T.

Static checks should reject forbidden escapes and invalid permissions before execution.
The interpreter should retain defensive checks for live frame identity, path validity,
exact target type, and permissions. These checks are not a proof about arbitrary raw
pointer programs or hostile native code.

The first tests should demonstrate ordinary aliasing, copy independence, read-only
write rejection, forwarded reference access, separation of caller and callee value
slots, references surviving same-type slot replacement, generic field types, and
rejection of escapes and uninitialized reads. A String local should be addressable
without a native String layout. Future language-generated and handwritten IL fixtures
should exercise the same contracts.

## Next decisions and implementation gate

The control-flow verifier, bound member identities and whole-slot addressed access
are implemented. Extend them together when introducing field paths, permissions or
longer lifetimes, with runtime enforcement for obligations the verifier cannot prove.

Still open: the signature encoding for read-only access; partial initialization;
reference returns and containment; native-address conversion; and field-reference
invalidation. Current receiver syntax and out/out(true) behavior are specified in
[reference contracts](reference-slots.md).
The decisions proposed here do not select an ownership system or commit the future
language to a borrow-checking discipline.
