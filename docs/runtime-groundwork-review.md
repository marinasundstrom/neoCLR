# Runtime groundwork review

Reviewed 2026-09-08 against readonly receiver milestone `0e1b2e1`. This is a code-backed design
assessment and proposed sequence, not implementation of the capabilities below.
Feature expansion is paused while these foundations are considered.

## Recommendation

Finish the storage/reference contract first, then establish shared type relationships
and complete-owner projections before implementing inheritance. Define activation
ownership and GC roots before escaping closures or suspension. A JIT needs these
contracts, but it does not require a moving collector or every future language feature
to be implemented first.

The missing groundwork is mainly agreement between existing subsystems, not absence
of a runtime. Preserve the loaded-program boundary, definition identities, managed
slot capabilities, frame escape checks, tracing GC, typed verifier, interface dispatch,
reflection, debugger/source maps, target layout and runtime-service planning.

## Findings and dependencies

### 1. Storage and reference contracts are incomplete

Evidence: [Function metadata](../src/metadata.rs) records readonly inputs and receivers,
but locals and returns still carry ordinary Type signatures. [SlotReference](../src/slots.rs)
retains its runtime permission; [the verifier](../src/verifier.rs) loses permission
facts when references pass through ordinary locals and returned signatures.
Slot::set has no immutable-slot contract. Neo let checks are compiler rules.

A readonly return can therefore look writable to its consumer and fault only when
used. This is enforced, but not yet a complete, predictable API contract. Immutable
storage is a different missing capability: restricting one alias cannot protect a
slot against every other writer.

Groundwork: specify a shared access/storage model covering local, parameter, field,
array-element and return positions; initialization, out writes and loop re-entry;
and conservative permission joins. Keep target mutability separate from rebinding.
Choose whether qualifiers belong on use-site signatures or separate contract records
before extending the current parallel parameter lists. Do not equate readonly with
purity, non-aliasing, thread safety or a stable value.

First bounded implementation after the pause: readonly local/return contracts and
verifier propagation, followed by protected-slot initialization. Demonstrate a readonly
reference returned by one method, stored locally and forwarded without losing its
declared restriction; raw IL must not upgrade it. Protected-slot tests must include
pre-existing aliases and repeat execution of a declaration in a loop.

.NET comparison: the [existing readonly investigation](readonly-parameters.md)
distinguishes language diagnostics, CLR controlled-mutability references and neoCLR's
runtime permission. Extending the same permission offers consistent cross-language
enforcement, at the cost of richer signatures, checks and migration.

### 2. Type relationships need one shared contract

Evidence: TypeDef currently has fields, interfaces and generic parameters, but no
base type or virtual slot declarations. [Interface implementation lookup](../src/interfaces.rs)
matches concrete methods against contracts. [Value storage](../src/value.rs),
[verification](../src/verifier.rs), [reflection](../src/reflection.rs) and layout each
consume type information for their own operations. Definition identities already exist;
they should be extended and used, not replaced with another naming scheme.

Groundwork: define common assignability and projection rules for values, references,
interfaces and future base views, including readonly permissions and generic substitution.
Define method-slot identity and override compatibility independently of textual names.
Resolve the difference between copying a derived value as a base value and viewing the
original through Base&. Do not let an incidental record-copy path decide slicing.

A base view must preserve the complete derived owner's identity, lifetime and GC
reachability. Field addressing, layout and reflection must agree on inherited members.
Decide constructor order, partial initialization and whether virtual calls are permitted
during construction before implementing the first inheritance chain.

.NET comparison: CoreCLR's [type-system design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/type-system.md)
connects type information with GC layout and stack walking. Reuse that separation of
responsibilities, without copying its object header or requiring System.Object.
A shared prepared descriptor is a candidate; mandate semantic agreement first, then
measure whether caching/layout changes are worthwhile.

Exit case: a base/interface view is the sole reference to a derived heap owner whose
derived-only field holds another heap reference. Collection, dispatch, identity and
reflection must all preserve the complete object correctly.

### 3. Absence and initialization need distinct representations

Evidence: [Type](../src/metadata.rs) has no nullable managed form. [Value](../src/value.rs)
and slots represent uninitialized storage; Option/Result are ordinary library unions.
A native null pointer is a separate construct.

Groundwork: define null versus uninitialized versus Void, nullable references versus
references to nullable values, default initialization, arrays and clearing GC edges.
Specify whether a nullable view can be narrowed and what runtime check establishes
non-null access. Decide the relationship to Option<T> explicitly.

This must precede not-null generic constraints and broad nullable APIs. Constraints
also need common validation at definition, substitution and invocation boundaries;
not-reference must describe addressing mode rather than a class/struct split.

.NET comparison: [C# nullable references](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/null-safety/nullable-reference-types)
are compile-time annotations and analysis, without a distinct runtime type.
[C# notnull](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/generics/constraints-on-type-parameters#notnull-constraint)
violations produce warnings in a nullable context. neoCLR can choose enforceable
runtime slots and constraints; that improves cross-language guarantees but requires
representation, default-value, conversion and artifact decisions.

Exit case: clearing a nullable heap reference removes a root, accessing an absent value
faults predictably, and reading an uninitialized slot remains a different error.

### 4. Execution ownership must precede escaping callbacks and suspension

Evidence: [Frame](../src/vm.rs) owns arguments, locals and evaluation state.
Frame::check_reference_return rejects current-frame escapes. Value::ensure_heap_references
rejects frame-backed references embedded in stored values. The current host invocation
model starts fresh execution state, as documented in [LoadedProgram](loaded-program.md).

Groundwork: specify execution-context identity and the owners of active and suspended
activations, managed captures and external handles. Decide which references may cross
a suspension boundary and how cancellation/completion releases retained state.
Do not silently promote a referenced local or treat retaining a Rust slot as permission
to extend its guest lifetime.

A typed callable can start as a function identity plus an optional managed receiver;
raw function pointers are not a prerequisite for that semantic model. Escaping
captures require a separate decision. Keeping the current heap-only capture rule is
the conservative initial option; retained activations would be a larger runtime feature.

This is our proposed dependency, not a claim that neoCLR must copy .NET's async lowering.
Compare compiler-generated state machines against runtime-owned continuations when
choosing an async slice. Suspension does not itself imply threads or parallel execution.

Exit case: a callback retains an eligible heap receiver across calls; an illegal frame
capture faults. Before async, specify one suspended activation's roots, resume-once
behavior, cancellation and debugger visibility.

### 5. GC and native execution need an explicit root protocol

Evidence: [the collector](../src/gc.rs) traces interpreter Value graphs and whole
allocation identities. [The interpreter](../src/vm.rs) enumerates frame roots at
collection points. [Pinning](pinning.md) is a documented proposal, not a native-address
bridge. Rust cells are not the guest's native ABI.

Groundwork: define root enumeration for active/suspended frames, returned host handles,
future static storage and native registrations. Before a JIT, specify safepoints,
root maps, interior-owner reporting and allocation/call helper contracts. Define what
a backend may optimize only after equivalent lifetime and Fault behavior is established.

.NET comparison: the [CoreCLR ABI](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/clr-abi.md)
ties GC-safe stopping and debugger locations to code-generation conventions. neoCLR
needs equivalent agreements, not necessarily CoreCLR's calling convention or machinery.

Keep the existing nonmoving collector for now. Moving GC, generations and barriers
are later policy choices. Pinning additionally requires a stable supported native
layout, rooting, release and replacement rules; nonmoving collection alone is insufficient.

### 6. Cleanup and persistent state remain separate missing foundations

Evidence: [Disposable and Closable](disposal.md) are explicit library protocols, with
no automatic scope cleanup or destructor dispatch. [Field metadata](../src/metadata.rs)
has no static-storage contract. Current executions have fresh state; the
[execution architecture](execution-architecture.md) leaves persistent-context Fault
containment and a stable invocation ABI open.

Groundwork: distinguish ordinary Result errors, terminal Faults, cancellation,
construction failure and resource cleanup. Decide cleanup ordering and what happens
if cleanup itself faults. GC reclamation must not acquire deterministic resource
semantics by accident.

Before broader framework APIs, specify per-context static storage, generic-static
identity and initialization order/failure/re-entry, or deliberately keep state in
explicit context objects. Before native callbacks or persistent hosting, define
handle lifetime, reentrancy and which failures leave a context reusable.

Compare .NET's familiar disposal and initialization behaviors during those dedicated
slices; this review does not select CLR exception unwinding or type-initializer policy.
No threading, atomics or memory-order guarantees should be inferred from the current
single-execution interpreter or readonly permissions.

## Proposed order and scope control

1. Review and settle the reference/storage capability matrix; implement local/return
   permission contracts and protected-slot initialization as separate small slices.
2. Establish shared assignability, complete-owner projections and dispatch contracts;
   use them to implement one inheritance chain.
3. Specify nullable storage and then enforce the relevant generic constraints.
   Enums/flags remain an independent small feature if a concrete consumer needs them.
4. Define persistent execution ownership and root registration, then add typed callbacks.
5. Resolve cleanup and suspension contracts before async or retained native callbacks.
6. Specify one native backend's ABI and GC safepoints before machine-code generation.

Each slice should include one Neo demonstration, hostile IL/artifact cases, and
verifier/runtime/GC/reflection/debugger agreement where applicable. Keep published
semantics separate from prototype implementation details. A focused conformance matrix
is more useful now than a wholesale runtime rewrite or a premature universal backend API.

This review is based on the cited implementation and design documents, with primary
.NET sources checked 2026-09-08. It is a dependency assessment, not a performance study
or a complete safety proof. The readonly regression evidence establishes the current
capability behavior; future foundations require their own executable acceptance cases.
