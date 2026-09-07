# Managed reference implementation gate

Status: proposed implementation plan for the agreed [lifecycle model](lifecycle.md).
This is a representation and invariant review, not executable reference retention,
new IL, a selected native ABI or an additional source-level ownership system.

Use the .NET CLR instruction set, metadata concepts and semantics as the baseline.
Reuse existing instruction meanings where they fit, and document the concrete
improvement when a deviation is needed. Ref<T> is a historical proposal and current
prototype encoding, not a selected future type. Preview changes may replace or
remove it and break artifacts/APIs; preserving the experiment is not an acceptance
requirement. Reject incompatible artifacts explicitly rather than adding an
unnecessary compatibility layer.

Reuse CLR managed references, represented by T&/ByRef, as the explicit reference
feature. Extend the existing metadata, verification and address/load/store paths
rather than introduce a parallel public ownership type. Frame-backed and retained
storage implement the same reference abstraction. Automatic retention and escapes
beyond a defining frame require documented semantic extensions; they do not imply
compatibility with execution on an unmodified CLR.

## Observable contract

T(...) constructs a value, &value refers to the same value, and new T(...) constructs
a managed heap value and returns T&. A reference can be passed down call frames
without manual handling. Returning a reference requires the referenced value to
outlive its creating frame. Source syntax does not select a retaining-wrapper type.

Every alias must observe the same logical storage location. Taking another reference
or changing physical placement cannot duplicate the referent. A value copy copies
its inline fields and its embedded reference handles; it does not recursively clone
their targets. Clonable remains an explicit operation with a separate contract.

Distinguish a reference variable from its referent: assigning another reference to
the variable changes its binding; writing through the reference changes the target.
A reference to an existing local continues to observe same-type assignment to that
local under the current whole-slot semantics. Promotion must preserve that behavior.

## Current implementation gaps

| Area | Current implementation | Required change |
| --- | --- | --- |
| Stack slots | Slot owns an optional Value in an Rc cell; SlotReference holds Weak plus an output-write baseline | Preserve stable target identity while adding retention where references outlive their frame |
| Managed heap prototype | Value.Reference contains an arena index and Type::Ref target | Supply retaining managed references with the selected T& semantics |
| Execution roots | Execution.heap owns every arena value until the result is dropped | Distinguish live references from registry bookkeeping so dead allocations can be released |
| Copies | Value derives Rust Clone; slot reads recursively copy value trees | Make reference-copy retention a specified guest contract across every copy path |
| Metadata and verification | ByRef locals, fields, returns and erasure are rejected | Admit supported retained references only after representation, escape and initialization checks exist |
| Hosting | Inputs accept owned values; pointer/Ref/byref transfer is rejected | Define opaque rooted results and execution-context lifetimes before persistent reference invocation |

See [slots.rs](../src/slots.rs), [value.rs](../src/value.rs),
[vm.rs](../src/vm.rs) and [input.rs](../src/input.rs). Existing host Rc cells are
implementation scaffolding: they do not prove physical stack placement or a guest
reference-counting contract. Replacing Weak with Rc alone would leave metadata,
copying, outputs, cycles and host cleanup obligations unresolved.

## Candidate representation

Use one logical managed location with a stable identity, exact closed type,
initialization state and payload. A managed reference identifies that location;
retention is automatic. Physical representation can differ across backends.

For a first interpreter experiment, a stable retained cell is a reasonable correctness
baseline. Direct frame storage owns its normal lifetime claim; surviving reference
values keep the same cell alive. Heap allocations enter this model directly.
An allocation registry must not accidentally retain every cell merely to track it.
Limits, diagnostics and identity lookup can use separate bookkeeping.

Proven call-scoped references can use ordinary frame storage without allocating
another managed heap object. Optimizing retention away is valid when the active
frame already guarantees the referent's lifetime. Conversely, a reference stored
or returned beyond that guarantee requires retained storage before the frame ends.
The programmer sees the same T& contract in either case.

Choose conservative retained placement for known escaping values before implementing
general late promotion. A later promotion mechanism must redirect all aliases through
the same logical identity. Raw addresses cannot be redirected this way; native
pinning/stable-address rules remain a separate gate.

## Invariants to establish before enabling escapes

1. Every accessible reference has a live target of the exact declared type. An
   allocation identifier cannot be fabricated or reused to revive an old reference.
2. Copying a reference preserves target identity and required lifetime. Discarding
   one reference releases only its own retention; the target survives other roots.
3. Reads copy the stored T according to its value semantics. Replacing T preserves
   location identity, accounts for embedded references and does not invalidate
   unrelated aliases merely because a host container was replaced.
4. Allocation and construction are separate internally. Uninitialized storage may
   participate in checked output initialization, but must not escape as a readable
   value. Preserve per-invocation out/out(true) write obligations across aliases.
5. Returning a reference installs retention in the caller/result before releasing
   the callee's lifetime claim. It never returns an address into a dead ordinary frame.
6. Disposal may change the target's resource state without invalidating its managed
   references. Final reference release and explicit Dispose are distinct events.
7. Failed stores, invalid types and failed promotion leave existing live references
   valid or terminate with a defined Fault; no partially published target is observable.

For replacement, retain incoming referenced state before releasing outgoing state.
Self-assignment and overlapping aliases must not reclaim a target in the middle of
the operation. Temporary interpreter copies must not become observable extra guest
lifetimes when destructor timing is introduced.

## Boundaries that cannot be inferred from host reference counts

An acyclic reference-counted graph is a useful first test workload, but is not a
complete cycle policy. Determine how unsupported cyclic stores are handled before
claiming general deterministic reclamation. Weak references or supplementary cycle
collection affect the public lifetime guarantee and need their own decision.

The allocation limit must distinguish live allocation count from an identity budget.
The current heap_objects check uses the monotonically growing arena length. Under
reclamation, repeated allocate/release should not exhaust a live-object budget while
identity reuse must still be safe. Native pointer-byte limits remain separate.

A host-visible reference must keep its execution context and required metadata/code
alive, or be restricted to a context-owned handle with checked lifetime. Rust Drop
on an arbitrary host thread must not unexpectedly execute guest destructors. Guest
cleanup scheduling, reentrancy, cancellation and secondary Faults need a deliberate
context contract before host releases can trigger user code.

Reachability/service planning must eventually include implicit destruction targets,
and native compilation needs equivalent roots, reference-copy and release behavior.
No interpreter-only ownership mechanism should become the portable ABI by accident.

## First bounded implementation and acceptance cases

First implement and test automatic retention of managed heap values, with scoped
byref access to the same target. Then enable a returned T& using retained placement
and verify that it survives the producer frame. Keep runtime reference handling
automatic; no manual endloc/retain/release source operations are prerequisites.
User destructor dispatch follows once these reference-copy/release invariants hold.

Acceptance cases should cover scalar and String-containing records; new T(...)
aliases; a local passed through multiple byref calls; returned &local; alias-visible
mutation before and after return; copies of records containing references; field and
local replacement including self-assignment; last-reference release; construction
failure; repeated allocation under a live-object limit; and rejected unsupported
native/host escape paths. Count target allocations and releases independently of
host implementation temporaries. Later add destructor-order and failure tests.

The final IL encoding remains open. Review CLI-style newobj, ldloca/ldarga and
ldobj/stobj before proposing any additional instruction. Their current prototype
meanings are not a reason to invent a parallel operation for an otherwise unchanged
concept. Reference escape and automatic retained storage need an explicit semantic
contract; do not imply ordinary CLR execution compatibility for those extensions.

Breaking preview artifacts and host APIs is permitted. Replace obsolete ByRef/Ref
representations where the selected model requires it, without assuming Ref<T> must
survive as a wrapper or a renamed type. Bump the format when old encodings would
otherwise be misinterpreted, require reassembly, and update validation, verification,
runtime services, host boundaries and examples together.
