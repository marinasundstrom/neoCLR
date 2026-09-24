# Compiler-generated async feasibility

Source assessment, 2026-09-14: neoCLR `5da27a7`, Raven `ee7b2e5af` on
`codex/neoclr-namespace-metadata`. No Raven files were changed. This is an inspection,
not a successful compilation or execution probe.

## Post-release priority — 2026-09-19

The author selected these compiler-facing contracts and Task as foundational work after
Preview 8, because upcoming APIs need the completion abstraction. A subsequent
direction places the query-operator naming pass before this work. Runtime suspension
remains a later step. The assessment below is historical source evidence; re-audit
the current compiler/runtime before implementation. See the updated
[API design priorities](async-api-design.md#post-release-task-foundation--2026-09-19).

## Finding

Compiler-generated async is a feasible transitional direction, but neoCLR does not
currently provide everything needed to execute Raven's existing async output.
A state machine saves live values and a resume position in heap-owned state;
MoveNext can return normally and be called again when the awaited operation completes.
This avoids requiring runtime-owned suspended interpreter frames for a first slice.
It does not eliminate the need for safe state ownership and continuation scheduling.

Existing building blocks include generic records, fields, branches, instance and
interface calls, heap allocation, tracing GC and managed delegates/closures. Their
presence supports feasibility; it does not prove that all emitted CIL, initialization
patterns or generic builder calls are accepted by the Raven-to-neoCLR path.

## Concrete gaps

- **Task library and compiler-facing contracts:** No guest Task/awaiter/builder
  implementation was found in the current tree. Raven's
  `SynthesizedAsyncStateMachineTypeSymbol.DetermineBuilderType` selects .NET
  AsyncTaskMethodBuilder/AsyncTaskMethodBuilder<T> (and other .NET builder families).
  A neoCLR implementation and matching reference declarations/projection are needed.
- **Error lowering:** Raven's `AsyncLowerer` wraps MoveNext in an AsyncDispatchGuard
  try/catch and generates a System.Exception path to SetException. neoCLR's
  [error contract](errors.md) has Result values and terminal, uncatchable Faults.
  Existing emitted handlers cannot simply be discarded. Define target-aware lowering
  for Result-bearing completion and preserve terminal Fault behavior; this need not
  introduce CLR guest exceptions. The author's follow-up selects Task<Result<T,E>>
  with an explicit error type: both Ok and
  Err complete the task with a Result value. This resolves the recoverable-error
  direction, while compiler adaptation and cancellation remain to be implemented
  or specified. Await itself yields Result rather than implicitly propagating Err.
- **State identity and initialization:** Raven synthesizes a struct state machine,
  initializes it, and passes it by reference to builder Start. Pending continuations
  must retain the correct heap-owned state, not a reference into a returned frame or
  an accidentally independent copy. Audit initialization, builder sharing, emitted
  generic calls and Task<Void> result/return handling against neoCLR's contracts.
  Current delegates reject frame-backed receivers.
- **Completion and execution ownership:** Implement pending/completed state, awaiter
  registration and continuation dispatch. A guest-owned queue pumped inside one live
  execution is a candidate first model, using ordinary GC roots. Host-driven completion
  after an invocation returns needs additional persistent execution/rooted-handle
  support: current [invocations](loaded-program.md) start fresh execution state and
  do not support moving live guest capabilities between executions.

## .NET comparison and bounded validation proposal

Primary .NET 10 API references consulted 2026-09-14:
[IAsyncStateMachine](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.compilerservices.iasyncstatemachine?view=net-10.0)
and [AsyncTaskMethodBuilder<T>.AwaitUnsafeOnCompleted](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.compilerservices.asynctaskmethodbuilder-1.awaitunsafeoncompleted?view=net-10.0).
These document the compiler-facing state-machine and continuation contracts; they
are not dependencies that can be satisfied by loading the host's .NET library into
neoCLR. Local Raven evidence is in `src/Raven.CodeAnalysis/BoundTree/Lowering/AsyncLowerer.cs`
and `src/Raven.CodeAnalysis/Symbols/Synthesized/SynthesizedAsyncStateMachineTypeSymbol.cs`
at the revision above.

Proposed first probe: implement a minimal Task<T>/Task<Void> and compatible lowering,
then run one Raven async method that saves a local, awaits an initially incomplete
task, returns control to a guest queue, and resumes after that queue completes it.
Verify output, shared state identity and GC retention while pending; also cover an
already-completed await, no-payload completion, repeated completion rejection,
illegal frame capture and a Result error. Define cancellation and terminal Fault
handling before claiming general async support. An already-completed await alone
would not demonstrate suspension/resumption.

This could deliver task-based async before runtime frame suspension, at the cost of
compiler/library adaptation that may later be replaced. It does not require a thread
pool or parallel execution for the first demonstration. Runtime-owned suspension
remains the intended long-term direction; no performance claim or implementation
commitment is made by this assessment.

## Re-audit and executable model — 2026-09-19

Raven revision ddf10eca80d599ede28f9da59f145497a21e854e on neoclr still has the
builder-selection and AsyncDispatchGuard/SetException assumptions identified above.
The [new completion experiment](experiments/task-contract/README.md) replaces only
the historical lack of execution evidence for a manual heap-owned continuation
model. It does not establish compatibility with emitted compiler state machines.

Following the author's clarification, compiler-facing builders, state-machine
interfaces and awaiter adapters are provisional. Compiler adaptation is allowed
and expected; they are not permanent public Task API requirements. The experiment
records the next integration steps and keeps general Raven fixes separate from
neoCLR-specific lowering policy.

### Why omitting SetException is insufficient

The 2026-09-19 inspection of Raven ddf10eca found that CreateExceptionCatchClause
always creates an exception local and catch body. CreateBuilderSetExceptionStatement
returns null when the builder has no SetException member, but the catch still sets
the machine to completed state and returns. CreateMoveNextBody still wraps dispatch
and the rewritten body in AsyncDispatchGuard. Therefore exception-free lowering
must bypass construction of that wrapper and catch explicitly, not only substitute
a smaller builder. Follow the [exception-free requirement](async-api-design.md#exception-free-lowering-requirement--2026-09-19);
no Raven code was changed by this inspection.

### Payload-independent lowering

The author's follow-up clarifies that the async transformation must not know about
Result: Task<T> completion and await operate on an arbitrary T. A Result payload
does not select a special builder, error path or scheduler behavior. Ordinary
Result handling/propagation may appear in the user's body, but its resulting return
is lowered exactly like any other return. The manual probe's Result match is
application logic, not part of the proposed compiler contract.

## First compiler adaptation — 2026-09-19

Raven's neoclr branch now has the provisional compiler API
`CompilationOptions.WithAsyncExceptionCapture(false)`. It skips construction of the
async exception wrapper and SetException member discovery. Default .NET compilation
retains capture. The policy survives option copies and prevents incompatible
incremental-state reuse. No project/CLI switch is exposed yet: this is compiler
integration groundwork, not an enabled neoCLR async language feature.

The tests execute real generated state machines on .NET with immediate and pending
awaits. Ordinary ? propagation before and after await returns a Result value through
the normal completion path and skips later side effects. An unrelated union also
completes through the same mechanism. No Result-specific async code was added.
The opt-out machines in these bounded cases have no exception handlers; explicit
handlers and disposal regions are not silently removed.

neoCLR still needs Task/builder/awaiter metadata and implementations, safe state
ownership, target selection and diagnostics for unsupported constructs. This slice
does not prove their execution on neoCLR. Separately, Task<unit> with explicit
`return ()` currently reports RAV2705 under both policies; that return-binding gap
needs independent resolution before the uniform Task contract is ready.

Compiler implementation: Raven [b99025680](https://github.com/marinasundstrom/raven/commit/b9902568043f52c4d2ab8616386a92d6007591de),
kept on neoclr. Validation passed 61 focused tests and the 119-test functions/async
selection on .NET 11 (overlapping sets). The neoCLR manual completion probe now
uses ordinary ? inside its resumed application transformation; its state machine
and executor do not inspect Result cases.

## Generic unit return correction — 2026-09-19

An independent main-based Raven reproduction confirmed the generic-unit bug on
ordinary .NET. The fix distinguishes nongeneric task shapes from generic unit
payloads in return binding, supplies a unit value to generic builder completion,
and lowers awaitless/arrow bodies through generic completion. It adds no Runtime
Contract option or neoCLR policy. Immediate, pending, tail-expression and explicit
unit returns are covered for Task<unit> and ValueTask<unit>, with nongeneric return
diagnostics retained. This resolves the compiler gap above; it does not prove
neoCLR's System.Void payload ABI or supply guest Task/builder implementations.

General fix: Raven main [fb8eb77bb](https://github.com/marinasundstrom/raven/commit/fb8eb77bb),
cherry-picked to neoclr as b67d1e11b. All 145 tests in the selected async/resource
suite pass on .NET 11, including ten new unit-task execution cases and two
nongeneric diagnostic checks. The temporary main-based branch is removed after
integration; the long-lived neoCLR branch remains separate.

## Value state-machine priority — 2026-09-24

The author asks whether true value support can make async state machines efficient
in Release and directs focusing on that path if Raven supports it. This is now the
next bounded foundation investigation, ahead of the Path consistency case. The
library-wide Object/GC consistency work remains required, not cancelled.

Raven does support value state machines. Current target-branch code in
SynthesizedAsyncStateMachineTypeSymbol chooses Struct/System.ValueType by default;
UseHeapAsyncStateMachines explicitly selects Class/System.Object. That selection is
not itself conditioned on Release. neoCLR's props select heap states in both build
configurations. Six existing Raven policy/runtime tests pass on .NET 11. The new
[Release target probe](experiments/value-async/README.md) establishes a successful
heap baseline and a struct import rejection, not working target value async yet.

### Why changing the flag is insufficient

The current neoCLR builder is a class holding a shared Promise. Start and
AwaitOnCompleted accept IAsyncStateMachine by value; SetStateMachine is a no-op.
Raven therefore boxes the struct at startup and again at continuation registration.
Import currently rejects a reference cast along this path. Struct field/boxing/GC
support is a prerequisite, but does not establish a correct, efficient builder
ownership protocol. Repeated boxing might preserve some simple results while still
allocating and copying state at every suspension. Do not claim allocation savings
from a metadata type change.

The [.NET 10 builder source](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Runtime/CompilerServices/AsyncTaskMethodBuilderT.cs)
(reviewed 2026-09-24) starts through a generic ref state parameter, stores suspended
state in a typed box and reuses that box. Its builder/task initialization order avoids
copies building distinct tasks. This is an implementation comparison, not a mandate
to copy ExecutionContext, CLR exception handling or the exact Task layout into neoCLR.

### Implementation gates, in order

1. **By-reference startup:** prove a generated non-generic struct machine runs a
   ready await against the target builder without boxing its state at Start. Keep
   result ownership shared with the caller; check unit and Result payloads. Separate
   target Task recognition from heap selection where necessary. Review importer
   generic/byref/constrained-call support instead of silently coercing references.
2. **First suspension ownership:** transfer saved state into one traced heap owner.
   Retain that owner for subsequent MoveNext calls. Do not retain a frame-backed
   managed reference or capture one in a continuation. Check pending completion
   after kickoff returns, and two independently pending awaits with observable
   mutations so stale copies cannot pass unnoticed.
3. **GC and completion:** keep reference fields, awaiters and the result source alive
   under forced collection between resumes. Verify aliases see the same state,
   completion/cancellation fires once, and completion releases unnecessary roots.
   Preserve terminal Fault and Task<Result<...>> semantics; do not import CLR catches.
4. **Measured Release gate:** compare identical ready, one-pending and two-pending
   programs against the existing heap policy. Record state-specific allocations,
   total allocations, copied payloads, peak live objects and collections before any
   speed claim. Promise/task/continuation allocations still count. Only select a
   new Release default after correctness and a demonstrated benefit; retain debugging
   behavior deliberately rather than assuming Debug must use classes.

The first experiment may retain the existing reference-type builder to isolate
state ownership. A value builder and lazy/fused task-state storage are subsequent
choices, not prerequisites for every slice. The cost of byref generic contracts and
promotion machinery must be compared with keeping the existing simple heap state.
Runtime-owned suspension remains a longer-term alternative. None of this selects a
public Scheduler API or requires networking work.
