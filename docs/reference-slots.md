# Managed slot references and reference receivers

Status: T& parameters, out and out(true) contracts, ldloca/ldarga, ldobj/stobj slot
access, reference receivers and safe interface slot views are implemented.
Existing pointer and InterfaceRef behavior is unchanged.

## Purpose

A callee should be able to write a typed value into storage supplied by its caller,
without allocating an owner or exposing a raw pointer. This supports output
parameters and methods that mutate their original receiver. The same mechanism
must work for ordinary interpreter values, including String and generic records;
it must not depend on the native-layout subset supported by pointer memory.

Use `T&` as the CLI-shaped spelling for a typed slot reference, with a
separate ByRef signature node. It is not `T*`, not an interface view and not the
bootstrap Ref<T> arena handle. Leave Ref<T>'s future counted-ownership meaning
separate from this non-owning access mechanism.

A slot reference grants access to one exact T slot. It has no null state, integer
conversion, pointer arithmetic or unchecked cast. Copying the reference aliases
the same slot; loading through it copies the stored value. Storing through it
replaces that slot immediately. There is no implicit copy-back on method return.

## Parameters and explicit access

Assembly syntax keeps type before the optional name:

```text
.function Assign(out Int32& destination, Int32 value) -> Void
    ldarg destination
    ldarg value
    stobj Int32
    ldvoid
    ret
.end

.function Main() -> Int32
    .local Int32 result
    ldloca result
    ldc.i4 42
    call Assign(Int32&,Int32)
    pop
    ldloc result
    ret
.end
```

`out` is a parameter contract, not part of Int32 or a separate reference type.
An unqualified `T&` parameter is read/write and requires initialized storage at
call entry. An out parameter accepts uninitialized storage, cannot be read until
assigned during that invocation and must be assigned before every normal return.
Even if the caller supplied an initialized slot, the callee must fulfill the out
contract; a prior value is not proof of assignment by this call.

Both directions use the same normalized T& signature. Parameter contracts are
preserved in metadata and checked at calls, not used to create overloads that
differ only in out versus ordinary reference access. Names remain authoring aliases
for indices. A future readonly parameter contract must restrict the capability,
including forwarding, rather than merely attach an advisory annotation.

| Operation | Implemented behavior |
| --- | --- |
| ldloca index/name | Form T& for that local; does not read or initialize it |
| ldarga index/name | Form T& for a by-value argument slot in the current frame |
| ldarg on a T& parameter | Load the passed reference, without dereferencing it |
| ldobj T on T& | Copy the initialized value from the referenced slot |
| stobj T on T& | Store an exact, storage-normalized T value into that slot |
| starg on a T& parameter | Rebinding is rejected in the first slice; stobj updates the caller's slot |

Existing ldobj/stobj pointer behavior remains an explicitly different operand path.
Neither instruction needs to imply native memory layout when its operand is T&.
Taking ldarga of a T& parameter would create a reference-to-reference slot and is
excluded initially; forwarding uses ldarg instead. Taking ldarga of an ordinary
by-value parameter refers to the callee's own copy, not its caller's argument.

The illustrative language projection uses `Assign(out &result, 42)`. It can also
hide this lowering behind assignment or multiple-result syntax. An ordinary return
followed by stloc remains preferable when a single returned value is sufficient.

## Reference receivers

Methods record their receiver mode explicitly: existing value receiver versus byref
receiver. Authoring syntax:

```text
.type Counter
    .field Value Int32
    .method instance byref Set(Int32 value) -> Void
        ldarg this
        ldarg this
        ldobj Counter
        ldarg value
        stfld Counter::Value
        stobj Counter
        ldvoid
        ret
    .end
.end
```

The receiver at argument zero is Counter&, not a copied Counter. Existing stfld
still produces a modified value; stobj explicitly writes it into the receiver slot.
This avoids changing existing field instructions or hiding a dereference. A caller
passes ldloca counter before the explicit arguments. Receiver mode belongs to the
resolved method contract and does not introduce overloads distinguished only by
receiver mode. Byref receivers require an initialized concrete value.

Future field references can make individual field mutation more direct. The first
slice needs only whole-slot access; nested field paths, arrays and native allocations
must receive their own validity rules before they become safe-reference targets.
Constructors retain their existing initialization convention initially.

## Lifetime, aliasing and enforcement

Start with a call-scoped subset: byrefs may be formed on the evaluation stack,
passed as arguments/receivers and forwarded to nested calls. They cannot be returned,
placed in locals or record fields, erased into System.Value, retained by runtime
helpers, or imported/exported through host calls or P/Invoke. Ordinary local values
remain addressable. Reference-valued locals and longer-lived references can be added
once their escape contracts exist; this first restriction is not a permanent VM limit.

The interpreter represents each slot with a stable host cell. Managed references
hold weak identity to that cell, its exact type and an optional output-write baseline.
They neither keep the frame alive nor alias a later slot when frame storage is reused.
Every indirect access checks that the slot is still live, the slot type matches,
and the access is permitted. Host reference counting supports this implementation;
it does not impose reference-counted ownership on guest values. A native backend can lower proven call-scoped references
to ordinary addresses, while preserving these rules for paths it cannot prove safe.
No fixed fat-pointer layout or stable native ABI is implied.

Typed verification rejects forbidden reference storage/escape and invalid operand
shapes. Execution enforces the same safety boundaries even when optional verification
was not requested. Initialization tracking follows the actual slot, not a copied
reference descriptor. Out parameters also need per-invocation assignment obligations:
stores through aliases and successful forwarding must satisfy the affected obligations.
A normal return with an unwritten out parameter faults before returning to the caller.
The verifier may prove those obligations where possible; runtime checks cover the rest.

Aliasing is allowed. Two writable parameters can designate the same slot, and reads
observe preceding writes in execution order. This is not Rust-style exclusive
borrowing and does not imply a no-alias optimization promise. Readonly access, when
introduced, will not imply that another alias cannot mutate the target. Threads and
concurrent access rules remain separate work. A borrow checker is not required by
this contract: runtime validation may enforce it, while compiler proofs may remove
redundant checks. Languages may choose stronger borrowing rules independently.
Stored/escaping references will still require explicit validity and retention rules.

The first safe subset has no raw-pointer-to-byref conversion. A pointer is not proof
of a valid slot or lifetime. Explicit unsafe bridges can be designed later without
weakening ordinary slot references.

## Output contracts and errors

Writes happen when stobj executes, including writes before a terminal Fault. There
is no rollback, cleanup handler or implicit transactional result. An ordinary Error
returned through Result is a normal return and must still fulfill every out contract.
Replacing a value invokes no automatic destructor or release: existing explicit
ownership rules remain the caller/library's responsibility.

Unconditional out is distinct from the implemented `out(true) T&` contract for
Boolean-returning methods. Conditional outputs must be assigned on true, and provide
no new initialization guarantee on false. The verifier recognizes direct success
edges from brtrue/brfalse on a call result; runtime assignment obligations cover
forwarding and aliases. It does not infer success through Boolean locals or arbitrary
comparisons yet. The union TryGet methods use this contract without inventing defaults.
The union API extracts case values; pointer-returning TryGet variants were removed.
See the [pseudocode and IL guide](references-in-pseudocode.md).

## Interface integration and implementation order

InterfaceRef retains its typed native pointer and value receiver copy. It is separate
from the implemented managed I& view formed from Concrete& by interface.borrow.
The managed view preserves slot lifetime and initialization, and conformance matches
receiver modes and output contracts.

Migrating List<T> contracts and implementations to reference receivers must be an
explicit change. Preserve or separately spell the raw-pointer interface path; do
not silently bless it as safe. A receiver reference cannot escape inside an interface
view any more than it can escape directly. Allocation and Free remain outside the
borrowed interface contract.

Implementation slices (implemented and committed separately):

1. ByRef signatures, ldloca/ldarga, direct read/write parameters, slot access through
   ldobj/stobj, stable slot identities and non-escape enforcement. Test String and
   records as well as primitives, nested forwarding, aliasing and invalid lifetimes.
2. Out parameter metadata, initialization/assignment obligations and diagnostics.
   Test uninitialized callers, alias writes, forwarding, normal-return failures and
   writes before Faults. Define the verification report's conservative limits.
3. Byref receiver metadata and explicit receiver mutation. Keep value receiver tests
   as compatibility controls, including overload resolution and host boundaries.
4. Interface slot views and a deliberate library/sample migration, with conformance,
   dispatch, lifetime and backend reachability coverage.

Each slice must update the opcode/deviation reference, executable samples and runtime
service planning. No compiler, GC, reference counting or full borrowing language is
required to establish these contracts.
