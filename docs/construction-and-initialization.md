# Construction, mutation, and initialization proposal

Status: implemented foundation plus future contracts for discussion. The implemented
[constructor subset](constructors.md) supports invocation and whole-value receiver
initialization with `starg this`. [Managed slot references](reference-slots.md),
byref receivers and out/out(true) contracts are also implemented. Managed field
references, readonly access, partial initialization and placement construction
remain proposals. The [verifier](verification.md) documents current static checks
and runtime obligations; this document discusses their future extensions.
The proposals below do not change current aggregate newobj, stfld, pointer, or
ownership behavior.

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

The [addressed-access design](addressed-access.md) records the original proposal.
The implemented whole-slot subset and its restrictions are specified in
[reference contracts](reference-slots.md); field paths and readonly permissions
in the broader design below remain future work.

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

Current `newobj instance Type::.ctor(...)` invokes a constructor with a whole-value
initialization receiver and returns the completed value. `newobj Type` continues
to construct a record from its fields. These are distinct normalized operations;
see [constructor invocation](constructors.md). Placement construction and partial
initialization remain open. No current module should acquire different behavior
merely by loading it again.

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

Option/Result now use temporary System.Value storage and ordinary typed extraction.
Their TryGet methods use `out(true) Case&`: a successful direct Boolean branch
establishes initialization, while false supplies no new initialization guarantee.
Execution checks assignment on required returns; the verifier's propagation is
conservative. See [verification](verification.md). Remaining work is payload layout
and lifetime migration, followed by richer initialization proofs when needed;
unconditional out is not a substitute for the conditional contract.

## Remaining dependency order and open choices

1. Preserve the implemented value/byref receiver distinction, whole-value construction,
   visibility, bound member identities and call-scoped reference checks.
2. Specify field-path identity and invalidation, readonly permissions and partial
   initialization before extending addressed access or placement construction.
3. Define payload layout, copying and lifetime contracts for String, errors and
   nested carriers before retiring System.Value. Keep extraction behavior stable.
4. Bring up a small language compiler and migrate a few library functions/types,
   comparing its output with handwritten IL fixtures.

Decisions still requiring discussion include whether verified read-only access is
a signature modifier or a distinct reference form, placement initialization, the
Fault containment boundary, and eventual copy/move contracts for owning values.
Current receiver metadata and constructor visibility are implemented; final binary
encoding, ownership hooks and high-level source syntax remain open.
