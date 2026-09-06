# Construction, mutation, and initialization proposal

Status: proposed contracts for discussion, not implemented or approved semantics.
Runtime implementation is paused after the marker-attribute slice. This document
identifies prerequisites for ordinary library types and a future high-level language.
It does not change current newobj, stfld, receiver, pointer, or ownership behavior.

## Separate the contracts

| Concept | Proposed meaning |
| --- | --- |
| Value of T | Fully initialized data with T's identity; no allocation policy implied |
| Storage for T | Space which may not yet contain a valid T |
| Construction | Establish a valid T in selected storage |
| Assignment | Replace a value in existing storage under the type's copy/ownership contract |
| Addressed access | Observe or mutate a particular storage location |
| Ownership | Responsibility for cleanup, independent of raw address access |

Value semantics remain the default. Physical native-stack placement is an execution
choice consistent with observable lifetime and identity, not a universal promise.
Plain pointer copies continue to alias without acquiring ownership. Host-language
Clone/Drop must not become the future guest copy/destruction contract.

## Addressable values and receivers first

The [addressed-access proposal](addressed-access.md) develops this boundary, its first
scoped subset, and the checks needed before implementation.

Recommend a CLI-like by-reference signature, provisionally written T&, alongside
native T*. Its meaning is addressed access to existing storage, not counted Ref<T>
and not a GC-selected allocation. Begin with addresses of locals, arguments, and
fields. Keep indices as canonical slot identities; source names only map to them.

The initial verified subset should make these references scoped to calls: they may
be forwarded to callees but not stored in heap records or returned from a function.
Returning caller-owned references can be added when provenance rules can express it.
This conservative first subset is a staging restriction, not the final lifetime
model. Raw pointers remain available with their explicit low-level contracts.
Taking an address need not require a native layout for a value: interpreter locals
can be addressed through internal location identities. A T& to a local containing
String must not accidentally require String's native-memory representation.

Read-only and writable access need explicit contracts. No Rust-style exclusive
mutable borrow is implied: aliasing can be permitted, and a read-only access path
alone does not prove that another alias cannot modify the value. Byrefs should not
implicitly convert to stable native addresses; pinning/movement and foreign access
remain separate concerns. Define escape checks and invalidation before introducing
an unchecked conversion.

For methods, recommend explicit receiver modes: value, read-only addressed access,
and writable addressed access. Existing value-receiver methods preserve copies.
A mutating method requires an addressable receiver, so a call on an original local
and a call on a copied local have predictable, different targets. The future
language can offer familiar member-call syntax and emit the appropriate receiver.
The metadata must retain that distinction even when source syntax hides it.

Current stfld returns an updated record value. The eventual addressed field-store
operation should mutate the addressed destination. Preserve value-update operations
as an authoring convenience where useful, but do not silently change serialized
stfld semantics. Final instruction spelling and migration/versioning are open.

## Initialization is a verifier state

Recommend distinguishing uninitialized storage, partially initialized storage, and
a fully initialized value in verification. Do not add a general null/default value
to T merely to represent these states. A normal value read, copy, method call, or
return requires full initialization. Void fields are logical values even when they
occupy no bytes; native zero-sized storage does not eliminate type identity.

A reference to uninitialized storage grants an initialization capability, not
permission to read a T. Its exact metadata representation is open. During verified
construction, field writes update initialization state. All reachable successful
exits must establish every required field; branch merges retain only guarantees
established on every incoming path. The partially initialized receiver must not
escape or be passed to an ordinary method expecting a valid T.

This is a limited verification contract, not a proof of arbitrary raw pointer code.
Raw memory operations retain their explicit unsafe capabilities and diagnostics.
A general control-flow verifier should first establish stack compatibility, local
assignment, and returns; field-level initialization and reference escape checks
then extend that foundation. Visibility is also needed before a library can protect
its invariants from ordinary callers.

Zeroing remains an explicit representation operation. It can construct a valid value
only where the type/representation contract permits it; do not infer a universal
zero default for future types. The current native subset has supported zero forms,
but that is not a promise for resource wrappers or union carriers.

## Constructors and recoverable failure

Recommend keeping .ctor as a Void-returning initializer of a selected destination,
with successful completion establishing a valid value. It must not choose stack
versus heap allocation. A constructor reference can eventually be the operand for
value construction, while placement construction initializes separately obtained
storage. This restores a familiar constructor-based operation without restoring a
class/struct allocation distinction.

Current .ctor members supply constructor identity for attributes only; invoking
newobj Type still constructs a record from its fields. Constructor invocation,
initialization receivers, and a versioned transition from this helper remain open.
No current module should acquire different behavior merely by loading it again.

Use ordinary factories returning Result<T,E> for expected construction failure,
for example Parse or TryCreate. Validate inputs before acquiring resources where
possible. On the error path, explicitly release any acquired resources; do not
silently introduce exception unwinding or automatic destruction. Reserve Fault for
violated execution contracts, with its containment boundary still to be decided.
A future ownership system can improve generated cleanup, but is not a prerequisite
for constructing plain records.

This deliberately avoids making .ctor return either Void or Result depending on the
type. It separates the familiar initializer convention from an ordinary fallible
API. Whether placement initialization may itself expose a recoverable protocol is
a later question, particularly for foreign resources.

## Library and language implications

The future compiler and assembler should share the platform metadata/IL contract.
The compiler need not emit assembly text: both front ends can produce the same
normalized representation. Start with an externally implemented small compiler;
rewriting that compiler in its own language is not a prerequisite for writing the
runtime library in the language. The native runtime/kernel boundary remains explicit.

Use paired handwritten-IL and language fixtures to check the same observable
semantics: copied values, addressed mutation, completed construction, error paths,
and field access. Begin migrating a few library functions, then types, while keeping
native interop/InternalCall declarations in their existing role. Avoid privileged
language-only library operations or hidden runtime behavior.

The first useful tests should exercise:

1. Mutating one local through an address without mutating a previously copied local.
2. Passing a field address to another method with correct receiver type and lifetime.
3. Rejecting a read or normal receiver call before initialization on every path.
4. Constructing the same type as a value and in separately allocated native storage.
5. Returning a recoverable factory error with explicit resource cleanup.
6. Preserving these behaviors when the caller is compiled from the future language.

Option/Result remain a motivating library workload. Their carrier convention still
needs inactive storage and safe typed extraction. In particular, a Boolean query
that writes an output only on success is not equivalent to an unconditional out
parameter: false must not fabricate a valid T. Conditional initialization would
need an explicit contract and control-flow support. Do not rush it into the first
byref implementation or assume ordinary out rules solve it.

## Proposed dependency order and open choices

1. Settle receiver modes, initialization rules, and the boundary between verified
   addressed access and raw pointers. Define member/field identity requirements.
2. Introduce a basic control-flow verifier and stable definition references, including
   generic overload identity. Add visibility so construction invariants can be hidden.
3. Implement scoped addressed access and mutable receivers with verifier support.
4. Implement constructor invocation and field initialization checking, with a declared
   compatibility path for current field-based newobj and value-update stfld.
5. Bring up a small language compiler and migrate a few library functions/types.
6. Use Option/Result to settle carrier storage and typed extraction; extend compiler
   and library together rather than designing a complete source language in advance.

Decisions still requiring discussion include the exact receiver encoding, whether
verified read-only access is a signature modifier or a distinct reference form,
construction visibility, the Fault containment boundary, and eventual copy/move
contracts for owning values. No opcode numbers, binary schema, ownership hooks,
constructor migration, or final source syntax are selected by this proposal.
