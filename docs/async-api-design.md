# Asynchronous API design questions

Decisions and provisional recommendations, 2026-09-14. The author asked whether
APIs need synchronous and asynchronous forms, whether methods need an async marker,
and which .NET conventions or behaviors neoCLR should retain or change. The Task/Result direction and ordinary async naming policy are selected. Other
recommendations below remain assistant proposals, not author approval or implemented
contracts.

## Current development proof of concept

The [Task completion slice](task-contracts.md) now implements Raven-authored
Task<T>, TaskCompletionSource<T> and an explicit TaskQueue in the development Raven
profile. Generic payloads, queued continuations and producer/consumer access checks
run on neoCLR. This is not in Preview 8. Named compiler-generated async functions
and isolated workers now run in the development PoC; automatic host progress and
shared Task state across threads remain outstanding. The queue is provisional.

The updated [Task model proposal](proposals/task-model.md) is the target for the
next slices: State/Outcome, explicit cancellation and Map/Then composition. Follow
the [runtime-first alignment sequence](task-model-alignment.md); older open questions
below record how the design evolved, not alternatives to this latest direction.

## Post-release Task foundation — 2026-09-19

The author initially selected Task and compiler-generated async state-machine contracts as
the first post-release priority, then placed the conventional query-operator naming
pass ahead of it. The rationale is that upcoming APIs need a shared
completion contract; this is foundational API work, not merely an async syntax feature.
Runtime-owned suspension remains the later direction. The first implementation is
transitional, and should preserve the public Task/Result model when its execution
mechanism changes. No Task implementation is included in Preview 8.

Start by specifying completion, awaiter registration, compiler/builder responsibilities,
state ownership and continuation dispatch, then prove immediate and genuinely pending
operations through Raven and neoCLR. Keep recoverable outcomes in Result. Cancellation,
cleanup and terminal Fault handling must be explicit before dependent APIs rely on them.

The author identified ConfigureAwait as an opportunity to improve developer experience
without inheriting every .NET convention. The assistant proposes predictable defaults
that avoid routine per-await boilerplate; the exact policy remains open. In .NET,
ConfigureAwait(false) changes continuation context/scheduler capture, does not promise
a thread switch, and does not suppress ExecutionContext flow. These are separate
contracts, not one context switch. See the primary
[ConfigureAwait FAQ](https://devblogs.microsoft.com/dotnet/configureawait-faq/)
(consulted 2026-09-19).

Compare implicit context capture, explicit executor selection and scoped scheduling
before choosing defaults. Less ambient capture could simplify library code, but UI
and other affine resources still need a clear way to resume on their owner. Specify
logical context propagation separately. Validate completed/pending awaits, reentrancy,
UI-like single-executor behavior and library composition; do not claim an improvement
until the resulting experience and costs are demonstrated.

## Starting contract

Task<T> represents an operation and Task<Result<T,E>> carries recoverable outcomes.
Task<Void> represents no-payload completion; Task<Result<Void,E>> covers the fallible
case. Runtime-owned suspension is intended, with compiler state machines a possible
transition. Some APIs are better expressed as callbacks. Await yields Result and
Result propagation remains a separate operation.

## Selected naming and provisional behavior choices

| Question | Recommendation and tradeoff |
| --- | --- |
| Both sync and async? | Keep in-memory computations synchronous. Prefer tasks for operations that may wait on external activity. Add both forms only for concrete consumer needs and supported implementations. Avoid automatically wrapping blocking code in worker tasks or blocking on async code. This reduces redundant APIs but requires explicit offloading or blocking adapters where needed. |
| Must every task-returning method be marked async? | No separate public modifier is necessary to distinguish a returned Task. A method may forward an existing task or create a completed task. Whether a frontend requires async on a body containing await is a language decision. Runtime suspension may still require explicit metadata and verification. |
| Infer async from await? | Worth evaluating as frontend syntax, but retain the declared Task return contract. Inference must distinguish returning an existing Task from returning the awaited result and must preserve diagnostics. Do not infer a new public return type silently from a body edit. |
| Implicitly suspend ordinary calls? | Defer. Removing explicit await is a larger effect/continuation design affecting callers, locks, reentrancy, references and foreign calls. Runtime support alone does not settle those contracts. Explicit await keeps suspension points visible. |
| Async suffix? | Selected by the author: ordinary names for asynchronous operations, identified by their Task return type; no required Async suffix. Add explicitly named Blocking/Sync alternatives only when justified, using the term that accurately describes their behavior. Keep ordinary in-memory computations synchronous without extra naming. This reduces naming overhead but costs .NET familiarity and requires type/tooling information at unawaited call sites. Never distinguish pairs solely by return type. |
| When does work start? | Prefer activated operations on call and permit already-completed tasks, following TAP. Await waits; it does not restart the operation. Lazy work is a different useful contract that should be explicit. Eager work requires ownership even if its task is ignored. |
| How does completion resume consumers? | Propose queueing registered continuations for pending tasks to avoid arbitrary consumer execution inside completion. Already-completed awaits may continue inline, so await is not a guaranteed yield. Queueing costs scheduling overhead; specify executor/thread affinity and test reentrancy before adopting. Avoid assuming a hidden synchronization-context policy. |
| Allocation choices? | Start with one Task family and reusable completion semantics; measure before adding ValueTask-like public restrictions. Internal completed-value optimizations remain possible. Multiple awaits require explicit result copying/sharing rules under neoCLR's value model. |

The author's rationale is that .NET added async equivalents to an existing synchronous
surface. TAP's documented coexistence guidance supports the compatibility concern;
it does not establish that this was Microsoft's sole historical motivation. The
assistant initially proposed retaining Async, then reconsidered when the author
clarified the question, and the author selected ordinary names. See the
[API policy](api-policy.md#async-naming-decision-2026-09-14) for the resulting contract.
No existing APIs are renamed by this documentation change.

## Contracts to resolve before an API ships

1. **Cancellation and deadlines:** Decide whether cancellation is a case of E, a
   standard outcome wrapper or a task state. The selected await-yields-Result model
   must not accidentally gain an untyped exception channel. Distinguish cancelling
   a wait from cancelling shared work, and a timeout from rollback of side effects.
2. **Lifetime and abandoned work:** Decide who owns an operation that nobody awaits,
   how child operations are joined or cancelled, and how shutdown releases resources.
   Task scopes are a candidate; detached work should be explicit. Dropping a handle
   must have a documented meaning rather than silently implying cancellation.
3. **Buffers and references:** Define retention and mutation rights until completion.
   GC reachability alone does not make a caller-frame reference valid or prevent
   concurrent buffer mutation. Prefer returned values over delayed out writes;
   specify cleanup before signalling completion when buffers can be reused afterward.
4. **Errors and side effects:** Expected failures found immediately should use the
   same Result channel as delayed failures. Document partial reads/writes, retry
   safety and terminal Fault scope. Do not translate runtime Faults into ordinary E.
5. **Composition:** Task<Result<T,E>> completion and business success are different.
   Define whether combinators collect Results or propagate the first Err, what
   happens to sibling operations, ordering and whether cleanup is awaited. A task
   combinator cannot assume that generic T is a Result or that Err is a task fault.
6. **One result versus a sequence:** Task gives one completion. Streaming values
   need a separately designed iterator/channel or callback contract, including
   backpressure, cancellation and cleanup; progress callbacks may accompany a task.
7. **Target and host capabilities:** Define scheduler ownership and blocking limits
   for event-loop, desktop and embedded hosts. Async does not imply threads. A host
   with only blocking I/O needs an honest adapter/capability policy, not a false
   promise that a task-returning call promptly releases its executor.

## Primary comparisons and remaining research

Consulted 2026-09-14:

- [.NET TAP](https://learn.microsoft.com/en-us/dotnet/standard/asynchronous-programming-patterns/task-based-asynchronous-pattern-tap):
  documented library convention for task-returning names, activated tasks, possible
  synchronous completion and cancellation. Its exception channel differs from the
  selected Result model. Retaining task familiarity does not require every legacy API.
- [.NET sync-over-async guidance](https://learn.microsoft.com/en-us/dotnet/standard/asynchronous-programming-patterns/synchronous-wrappers-for-asynchronous-methods)
  and [async-over-sync guidance](https://learn.microsoft.com/en-us/dotnet/standard/asynchronous-programming-patterns/async-wrappers-for-synchronous-methods):
  documented advice against manufacturing wrapper pairs. This is support for keeping
  modern .NET guidance, not evidence that neoCLR invented the distinction.
- [Rust function reference](https://doc.rust-lang.org/reference/items/functions.html):
  async bodies produce futures whose execution is driven by polling. This is an
  alternative to TAP activation; adopting it would change when side effects occur
  and how callers schedule work. Rust ownership assumptions do not transfer unchanged.
- [Microsoft.VisualStudio.Threading VSTHRD200](https://microsoft.github.io/vs-threading/analyzers/VSTHRD200.html):
  an ecosystem library/analyzer's concrete policy enforces Async naming for awaitable
  methods. It demonstrates tooling value, not proof that suffixes are universally best.

This is initial comparison, not a completed survey of independent .NET libraries,
structured-concurrency designs or maintainer API-review discussions. Before settling
cancellation, scopes and scheduling, deepen those comparisons and pin implementation
revisions for source-dependent claims. No performance improvement has been measured.

Validation should pair an in-memory operation with a genuinely pending I/O/host probe;
exercise immediate and delayed Ok/Err, cancellation races, multiple awaiters, discarded
handles, nested completions, GC while pending, invalid references, early disposal,
combinator sibling cleanup and a host that cannot block. Compare both compiler and
runtime suspension paths against the same observable API contract when available.

## Provisional mechanism boundary — 2026-09-19

The author clarified that state machines are provisional and their supporting
contracts need not exist forever; adapting Raven is expected. Treat Task<T>'s
completion/result semantics as the public design, while builder selection,
state-machine interfaces, awaiter registration, state layout and ownership
adapters are replaceable compiler/runtime implementation contracts. Their initial
shape must not constrain future runtime-owned suspension. Even the public API is
still developing; this separation is not a promise of permanent binary ABI.

The [completion experiment](experiments/task-contract/README.md) tests queued
continuations and heap-owned saved state within one execution. It does not introduce
a shipped Task class or compile async/await. Its explicit single executor is a
provisional test mechanism, not the settled scheduling/context policy.

## Exception-free lowering requirement — 2026-09-19

The author explicitly requires no exceptions in neoCLR async: the runtime does not
support them, so Raven must adapt rather than introduce an exception compatibility
layer. The selected completion model remains Pending -> Completed(T). For
Task<Result<V,E>>, both Ok and Error are ordinary completed values. Await obtains
the Result; explicit patterns or ? handle its recoverable error. Runtime Faults
terminate the current execution and are not caught, boxed or stored in a task.

The compiler integration must select this behavior explicitly through the target
contract. Do not infer it merely from a missing Exception type or SetException
member: Raven currently constructs its async catch even when SetException lookup
fails. That would leave unsupported handlers and, on an exception-capable target,
could swallow failure without completing the task.

For the neoCLR policy, generate the dispatch/body/completion path without the
AsyncDispatchGuard exception wrapper or System.Exception/SetException dependency.
Preserve the ordinary .NET policy for .NET targets. The importer must continue
rejecting reachable exception handlers rather than removing them after emission.
Compiler-synthesized cleanup also needs review: retain cleanup on normal completion
and early Result returns, but do not claim finally/unwinding behavior on a terminal
Fault. Unsupported source constructs need diagnostics before execution.

The provisional builder needs completion with an arbitrary T and continuation/state
ownership. Completion does not classify the value as success or failure. It does not need a parallel
exception completion channel. Cancellation and operation abandonment remain
separate design questions; they must not acquire exception semantics accidentally.

Validation must execute immediate/delayed Ok and Error, Result propagation, normal
cleanup, and terminal faults before and after a pending resume. Check emitted
metadata for absent exception dependencies/handlers and rejected unsupported
constructs. Run unchanged .NET async failure behavior as a compiler regression
check. These are integration requirements, not a claim that the compiler path is
already implemented.

## Task and failure are independent — 2026-09-19

The author clarified that Result<T,E> is an ordinary type even when it is a task's
result. Async lowering must be uniform in T: completing Task<T> stores a T, and
awaiting it produces a T. It must not recognize Result, inspect Ok/Error, unwrap
a case, infer failure, or select a different builder path for Result payloads.
Task owns completion; Result independently models a recoverable outcome.

For Task<Result<V,E>>, substituting Result<V,E> for T is the entire relationship.
Patterns, operator calls and ? retain their ordinary language semantics after
awaiting. When ? causes an early return inside an async body, async lowering handles
that return through the same completion path as any other returned T; it does not
implement a second Result propagation mechanism. No Task-specific failure channel
or Result-aware scheduler is implied.

The exception-free target policy is independent of the payload type. Validate the
same immediate/pending completion behavior with int, unit, an unrelated user union
and Result payloads, so the compiler contract cannot accidentally depend on Result.

## Implemented PoC — 2026-09-21

[System.Tasks contracts](task-contracts.md) now execute named async functions with
pending awaits through an explicit TaskQueue. Result is still an ordinary payload;
no exceptions or ConfigureAwait policy are introduced. The builder/state protocol
is provisional and may be replaced by runtime suspension. Threading is a subsequent
slice, not a guarantee of this completion implementation.

## Promise-style composition direction — 2026-09-21

The author wants Task to resemble a Promise API, with methods for composing and
continuing tasks. .NET terminology is a reference, not a naming requirement.
Retain the project's initial-capital method convention. Exact operator names,
overloads and scheduling rules are still open; this records direction rather than
newly implemented operators.

Compared with .NET's
[Task<T>.ContinueWith](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.task-1.continuewith?view=net-10.0),
Promise-style chaining offers another vocabulary for expressing dependent work.
[ECMAScript 2025 Promise.prototype.then](https://tc39.es/ecma262/2025/multipage/control-abstraction-objects.html#sec-promise.prototype.then)
is a useful composition reference, but also includes rejection handling. Sources
reviewed 2026-09-21. neoCLR's existing decision remains: Task<T> supplies eventual
completion with an ordinary T; Result<T,E> independently models recoverable failure.
Promise-inspired composition does not introduce a rejection channel, exceptions,
implicit Result unwrapping or JavaScript scheduling semantics.

Evaluate value transformation, chaining a callback that returns another Task, and
combining multiple Tasks as distinct operations. Choose names for their meaning
and consistency with collection/Option/Result operators, not mechanical parity
with either .NET or JavaScript. Benefits sought are readable pipelines and direct
composition alongside await; costs include reduced .NET API familiarity and the
need to define callback ordering, reentrancy, queue ownership and pending/completed
behavior explicitly. Validate those contracts before implementing the operators.


## Task model takes priority — 2026-09-21

The author selected cancellation as Task state, with expected failure in Result.Error
and Faults outside the recoverable model, then directed alignment with the updated
Task model proposal before further Raven lowering work. Map and Then now have
proposed semantics; their scheduling and implementation still need validation.
Tokens request cancellation; operations decide whether to terminate as Cancelled.
The [alignment assessment](task-model-alignment.md) distinguishes the validated PoC,
unvalidated cancellation work and required public contracts. Raven may need a
separate lowering, but its implementation must follow the Task model.
