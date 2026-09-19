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
