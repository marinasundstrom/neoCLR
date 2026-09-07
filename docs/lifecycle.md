# Value lifetimes, destruction and resource cleanup

Status: proposed next foundation following the prototype review. Deterministic
destruction and explicit ownership are the intended direction; the detailed rules
below are recommendations for implementation. Destructor metadata, lifetime
instructions and counted Ref remain unimplemented. Ordinary
[Disposable/Closable interfaces](disposal.md) and [Clonable](cloning.md) are now
implemented separately; they do not enable automatic destruction.

The platform direction is values by default, explicit passing by reference and
deterministic lifetimes. Pointers provide low-level memory access, including native
interop; ordinary resource APIs should use values and explicit managed references.
The existing pointer APIs remain available, but are not the default resource model.
[System.Clonable<T>](cloning.md) is implemented separately for explicit cloning.

## Three distinct operations

| Operation | Trigger | Result | Meaning |
| --- | --- | --- | --- |
| Destruction | Runtime ends an owned value's lifetime | Void | Run its destruction body and destroy its owned fields |
| Disposable.Dispose | Explicit call or language-generated scope cleanup | Void | Release a resource while leaving a valid disposed value |
| Closable<E>.Close | Explicit call whose result is handled | System.Result<Void,E> | Complete a potentially fallible close protocol |

A type defines a value's behavior, including a declared destruction contract; it
does not select stack/heap placement or universal reference counting. Disposable
and Closable are ordinary interfaces. Implementing either must not by itself
register a destructor. A destructor may call Dispose, but the lifecycle metadata
must identify that behavior explicitly, independently of a method name.

The implemented Dispose and Close interfaces use byref receivers so state changes
affect the original value. Existing interface views dispatch them without boxing.
System.Disposable and System.Closable<E> are the initial library contracts.
Specific E preserves recoverable error information;
heterogeneous callers can use an explicit adapter to a shared error type later.

Dispose returns no recoverable error. That is an API obligation, not proof that
arbitrary IL cannot Fault. Close exposes expected failures such as flushing output.
Destruction must not silently claim successful flushing, committing or publication.
An explicit Close caller handles the result before ordinary lifetime cleanup.

For the initial resource protocol, Dispose is idempotent. Close after successful
close succeeds without repeating effects, even following subsequent disposal.
Otherwise, Close on a disposed resource reports a documented error. A failed Close
leaves a valid value that can still be disposed.
Whether retry is supported depends on the resource and must be documented; do not
promise rollback or repeat a partially completed external operation automatically.
Destruction after either successful Close or Dispose releases nothing twice.

## Ownership and copying must precede automatic destruction

| Form | Copy/transfer contract | End of lifetime |
| --- | --- | --- |
| Plain copyable T | Copy its contents under their declared contracts | Destroy owned fields where applicable |
| Unique owner | Transfer explicitly; copying is rejected | Destroy its payload and release its storage |
| Shared owner | Copy explicitly retains ownership | Release one owner; the final owner destroys the payload and frees storage |
| T& or interface view | Borrow existing storage | Release no ownership; target lifetime rules still apply |
| T* or Void* | Copy an address | Do not destroy or free the target |

Destruction of a shared-owner wrapper and destruction of its shared target are
different events. Only owning references contribute to the target's reference count.
Borrowed references must remain checked if an owner can be released through an
alias; they must not silently retain an allocation. Shared target projections need
a specified liveness mechanism before they are supported. Cycles and weak owners
are later work; reference counting alone does not collect cycles.

Ordinary value semantics remain the default. A destructor declaration alone must
not silently turn a type into a reference type or change ordinary assignment into
an explicit Clone call. Resource-owning fields need a declared copy/release protocol
that preserves those semantics. For explicitly unique ownership wrappers, an opt-in
move-only contract is a possible bounded first experiment, not a universal rule.
A record containing such a field inherits that restriction. Generic operations must
preserve the selected capabilities after substitution, including when verification
is skipped. The ownership-aware copy protocol remains an implementation prerequisite.

Clonable<T>.Clone is explicit, separate from ordinary copying and moving. An owning
wrapper might copy by retaining shared storage while Clone duplicates its payload;
an ordinary record can implement Clone by copying its fields. Document sharing and
independence per type. Do not make the interface an implicit VM copy hook.

Current ArrayList descriptors are explicit aliases to shared storage. Adding a
freeing destructor to each copied descriptor would double free that storage.
Keep that API's existing contract until an explicit ownership migration. Likewise,
the current Ref arena must not silently become a counted owner in existing artifacts.

## Lifetime events in IL

The compiler can hide cleanup syntax, but the emitted IL must describe transfers
and lexical lifetime endings. The runtime invokes destruction at those events;
Rust Clone/Drop and host Rc counts are not guest lifecycle operations.

| Event | Proposed behavior |
| --- | --- |
| Load or dup | Copy only when the closed type supports copying |
| Explicit take/move from a slot | Transfer its value and leave the source uninitialized |
| Store into an empty slot | Transfer the incoming value into that slot |
| Replace an initialized slot | Validate incoming value, destroy the old value, then install the new value |
| Discard an owned evaluation-stack value | Destroy it |
| Call | Transfer arguments into callee slots; do not destroy transferred arguments |
| Return | Transfer the result first, then destroy remaining callee-owned values |
| Explicit lexical lifetime end | Destroy the live value and mark its slot uninitialized |
| Final owning-reference release | Destroy target, then free its allocation |
| Raw heap.free | Release raw storage under its existing contract; do not infer live objects from bytes |

These are semantic operations, not selected opcode spellings. A function frame
cannot infer lexical block exit from an evaluation-stack pop. Branches and early
returns must lower through explicit cleanup paths where a language promises scope
cleanup. Returning an owner requires moving it out before destroying the frame's
other values. An uninitialized or moved-from slot has no value to destroy.

Recommend reverse successful-initialization order for frame cleanup. Replacing a
slot establishes a new value lifetime and updates its cleanup registration. A
destruction body observes its fields before the runtime destroys those fields in
reverse declaration order. The body must not manually destroy a still-owned field;
an explicit take can transfer a field once partial-move rules exist. Initially
reject partial moves and destruction bodies that replace/move their whole receiver.

Lifetime-ending operations must invalidate references to the ended lifetime, even
if the physical slot is reused. Current weak slot identity distinguishes frames
but does not distinguish successive value lifetimes within one live slot. Add a
generation or equivalent runtime check before introducing take/end-lifetime.
Define replacement invalidation consistently for byref parameters and interface
views; a destructor receives a restricted view of the value being destroyed.

## Construction, native storage and Faults

Only fully initialized values receive their destruction body. The first slice
retains whole-value construction. Failed construction must account for initialized
owned temporaries without pretending an incomplete receiver is a valid T. Partial
field initialization later needs explicit field-state cleanup records.

Raw byte allocation and typed value ownership remain separate. Neither heap.free
nor localloc teardown currently has enough information to discover live resource
objects inside arbitrary bytes. Owned containers must track initialized elements,
destroy those elements, and then free the buffer. cpobj/cpblk and pointer loads/stores
cannot become an unchecked copying route for move-only values; reject unsupported
placements in the first slice until typed native lifecycle operations exist.

Recommend guaranteed deterministic destruction on normal lifetime endings only
for the first slice. Result.Error is a normal return and follows the same cleanup
rules as success. A terminal Fault, cancellation or exhausted execution budget
continues to terminate guest execution; host teardown reclaims tracked allocations
but is not a promise to execute guest destructors or close foreign resources.

A destructor runs as guest IL, consumes the existing instruction/frame budget and
appears in logical Fault traces. If it Faults, preserve that Fault and stop guest
execution. Do not retry it, continue arbitrary user cleanup, or introduce guest
catch/unwind semantics implicitly. A stronger bounded cleanup phase would require
a separate budget, secondary-Fault reporting and host/native failure contract.

This qualifies automatic cleanup: applications needing observable completion must
call Close explicitly; host resources needing release even after guest termination
need host-owned registrations or another specified teardown mechanism.

## Concrete first implementation slice

1. Introduce closed-type lifecycle capabilities, explicit transfer and lifetime-end
   operations, slot generation tracking, and runtime/verifier enforcement. Preserve
   ordinary value copying; specify owner-aware copying before admitting resource
   fields. An explicitly unique wrapper can be an initial test case. Reject copying
   or erasing such wrappers, legacy arena storage, host import/export and native
   payload placement until each boundary supports its lifecycle contract.
2. Add an explicitly identified Void-returning IL destruction body with a restricted
   byref receiver. Schedule it through guest frames on discard, replacement, normal
   return and explicit lifetime end. Preserve existing plain-value behavior.
3. Build on the implemented ordinary Disposable and Closable<E> interfaces with
   byref receivers. Their value-backed draft sample establishes explicit dispatch
   and state contracts. Next demonstrate a resource owner with idempotent Dispose
   and actual release tracking. Test fallible Close separately through an injected
   resource service; a native byte buffer has no natural flush error and should not
   invent one merely to implement Closable.
4. Cover nested owning fields and active union payloads before generalizing native
   containers and replacing System.Value. Add shared owning Ref and final-release
   destruction afterward, with a deliberate artifact/API migration.

Acceptance cases must count actual resource release and destructor calls: explicit
scope exit, early return, move through a function, overwrite, discarded temporary,
nested owners, rejected copies, stale byrefs after slot reuse, Dispose twice, Close
success/error followed by Dispose, and a destructor Fault. Include source/artifact
round trips, verification and execution without optional verification. Backend
reachability must include implicit destruction targets and required services.

Before accepting the first owner type, audit every current copying path: slot get/set,
dup, field extraction/update, arguments/returns, value.pack/unpack, heap.load/store,
typed native access, interface value receivers and host input/output. A partial
audit is not sufficient to make resource ownership safe.

The resulting lifecycle contract must be shared by interpretation, JIT and AOT.
Encode new metadata/operations with an explicit compatibility decision; do not
reinterpret previously serialized plain values as resource owners.
