# Interpreter runtime-async experiments

**Current priority (author correction, 2026-10-11):** make managed programs compiled
to VM IL work end to end, including Raven runtime async, across interpreter, JIT and
native AOT. Current runtime-async evidence remains interpreter-only. Cross-platform operation
interfaces and native ABI design are deferred. The interface experiments below are
exploratory evidence, not selected public APIs or the next implementation milestone.


Current development status: [Raven language support](#raven-runtime-async-language-poc-2026-10-11)
now lowers ordinary async methods to retained bodies through an explicit native
compiler option. Executable samples and GC reclamation checks pass without emitted
state machines. This is interpreter-only and opt-in; production notification/lifetime
integration, broader lowering and default switching remain open. Earlier sections
record prior POC slices and their limitations at the time.

Development experiment, 2026-10-11. `Yield.rvn` calls a private runtime boundary
from ordinary Raven functions; no async state machine is generated for these calls.
`RuntimeAsyncYield` temporarily retains the current VM frame chain, dispatches at
most one eligible queued callback, and resumes after the call. An empty yield
returns immediately. It neither waits for an operation nor drains all ready sources.

The retained frames preserve locals, evaluation-stack values, heap roots, method
identity and resource charges. Callback faults terminate the invocation. Nested
yields resume their own callers. Task mutation and worker execution reject this
boundary. This is cooperative reentrant execution within one invocation, not a
detached activation, green thread, cross-thread resume or public Task await.

The exact private InternalCall `neoCLR.Runtime.RuntimeAsyncYield()` accepts a
neoCLR unit result or CLI no-result encoding. It requires TaskDispatch capability.
CLI no-result resumes without pushing a unit value. The Raven consumer declares
this service locally; there is no Runtime Contract option or automatic await
mapping. JIT/AOT support is absent and must not be inferred from compiler support.

## Validation

`cargo test --test runtime_async_yield --test scheduler_submission --test timers`
passes 13 focused tests covering retained state and GC roots, callback faults, nested empty yield, one callback
per boundary, frame/evaluation-stack limits, and exact service signatures.
[Recorded consumer commands](validation.json) compile the source with the existing
Raven bundle and execute it with the modified interpreter. Binary/source hashes
identify the tested artifacts. The compiled Raven consumer passes on macOS ARM64. No speedup is asserted and this is not yet the
state-machine versus runtime-suspension benchmark gate.

## Design boundary and next gate

Raven's existing CLR runtime-async lowering targets AsyncHelpers and CLR metadata;
it is not a target-neutral neoCLR insertion contract. Keep that .NET path intact.
A future explicit neoCLR target contract should carry method/await intent and
source information; the runtime should own activation retention, scheduling and
resume/result/fault delivery. Generated state machines remain supported.

Compared with compiler state machines, this probe retains ordinary runtime frames
without synthesized managed carriers. Its costs are retaining the full call chain,
reentrant execution and frame-budget pressure. It does not establish a native stack
strategy or complete logical async stack traces. Next, design operation-triggered
parking with one-shot resumption, atomic registration/completion, cancellation,
lifetime cleanup and logical await links before connecting Raven await lowering.
Detached interpreter activations and JIT/AOT frame/liveness contracts remain open.

The public Task/Promise implementation, API reference and website capability claims
remain unchanged; no public API is introduced by this experiment.

## Operation-triggered suspension POC — 2026-10-11

`Await.rvn` now demonstrates an ordinary nested function waiting for a producer.
The producer itself waits on a 20 ms timer, forces GC, updates its receiver and
completes the original operation with an integer result. Both ordinary call chains
resume with their state intact. A second case consumes an already completed result.
There are no async methods or generated state machines in this consumer. Explicit
private calls are intentional here; user-facing asynchronous code should still use
`await` with the supported managed Task implementation.

The private services are:

| Service | Result | POC behavior |
| --- | --- | --- |
| `RuntimeAsyncCreate()` | Int64 | Create a pending integer-result operation. |
| `RuntimeAsyncDelay(Int32 milliseconds)` | Int64 | Create a nonnegative monotonic timer; completion yields zero. |
| `RuntimeAsyncComplete(Int64 id, Int32 value)` | Boolean | Complete a pending manual operation once; false for invalid/terminal/timer handles. |
| `RuntimeAsyncCancel(Int64 id)` | Boolean | Cancel a pending manual/timer operation once. |
| `RuntimeAsyncAwait(Int64 id)` | Int32 | Retain frames until ready, consume once, or fault on cancellation. |

These are context-owned tokens, not Task replacements. In particular, public Task
multiple-await behavior must remain supported when projections are integrated.
Completed-before-wait results use the same boundary without parking. Pending checks,
callback completion and boundary installation serialize on the dispatcher; there is
no asynchronous host producer ABI yet. Unknown, consumed and recursively awaited
handles fault. Completion cannot overwrite cancellation. Cancelled operations fault
the invocation with ExecutionCancelled; no recoverable managed cancellation outcome
is introduced. Timer readiness is observed lazily by the wait; cancellation can win
before that observation. Timer handles reject explicit producer completion.

Pending execution returns `InstructionProgress::Waiting`, publishes roots and releases
the managed graph guard before host parking. The durable wake latch covers newly
submitted callbacks; waits use the smaller of the timer deadline and the existing
10 ms cancellation polling bound. Tests directly step the VM to prove that repeated
pending checks preserve frames without spending guest instruction budget.

### Validation

The compiled Raven consumer passes on macOS ARM64, alongside 25 focused checks:
22 integration tests and three library checks for host return/resumption, context
ownership, quotas and timer terminal states. See [operation-wait evidence](await-validation.json).
This qualifies the interpreter POC only; no Windows/native result is asserted.

### Reproduce

```sh
CARGO_TARGET_DIR=target/scheduler-wait-validation cargo build --bin neoclr
python3 docs/experiments/runtime-async/validate.py \
  --bundle target/data-cases-bundle \
  --runtime target/scheduler-wait-validation/debug/neoclr \
  --output target/runtime-async-await
```

The script requires a previously verified matching Raven bundle. It records commands,
source/runtime/compiler/reference hashes and output in its validation report. Focused
checks are `cargo test --test runtime_async_await --test runtime_async_yield
--test scheduler_submission --test timers`, plus library filters
`runtime_operation_await_returns_to_host` and `runtime_async::tests`.

### Boundaries and next step

This is an interpreter POC with integer results and at most 64 unconsumed operations
per context. Abandoned operations live until context teardown; consumed slots are
reusable. Worker execution and task mutation reject these services. A parked wait
runs submitted callbacks, but does not pump the legacy I/O/default-TaskQueue adapter.
It does not yet support real HTTP operation awaits or public Task projection.

Frames are retained in LIFO order. If a nested callback parks, its parent cannot
resume independently even when the parent's operation completes. This head-of-line
blocking is a concrete reason to investigate detached activations next. The probe
proves runtime-owned parking/resumption, not scalable green threads. It also does
not supply complete logical async traces, native/JIT frame maps or cross-thread
completion rules. No throughput/latency improvement is claimed. Comparative
state-machine benchmarks belong with an equivalent workload once these paths exist.

The bundled Raven compiler recursed during binding when a property initializer
called the source-local attributed intrinsic (an explicit property type did not
fix it). Passing the operation into a constructor avoids that form. The compiler
issue is tracked in both bridge documents for a separate minimized regression.
Public API reference and website capabilities remain unchanged.

The author subsequently clarified that green threads are the intended direction.
Independent activation scheduling is therefore the next POC gate; native stack
representation and preemption remain design questions, not established contracts.

## Detached interpreter activations — 2026-10-11

The private operation-wait POC now detaches a callback's frames when it parks and
can resume a ready parent independently. When no chain is active, ready parked activations are selected in
parking order at dispatch boundaries. Their frame permits, GC roots and array
payload accounting remain live while detached. Entry results are retained while
remaining activations finish; callback completion cannot overwrite the entry result.
The host wait considers deadlines across parked activations. Existing state-machine
and public Task behavior are unchanged.

The new handshake has a callback complete its parent's operation, then await a reply
from that parent. The parent resumes, sends the reply and awaits callback completion.
This cannot finish under the preceding LIFO-only implementation. The compiled Raven
consumer covers it, and focused tests additionally collect while the callback is the
only owner of its receiver and check detached array budgets and entry-result retention.

This is cooperative, interpreter-only scheduling of private integer-result operations.
No native stack strategy, preemption, migration across workers, stable foreign ABI,
public Task projection or compiler `await` lowering is selected by this implementation.
Uninterrupted guest code still runs until an existing execution boundary; fairness
and scalable work selection require measurement. Selection currently scans a bounded
operation/activation set; this is not a throughput improvement claim.

The preceding LIFO limitation is historical for the earlier POC revision. See
[detached-activation consumer evidence](detached-validation.json). Run the same
`validate.py` command with a fresh dedicated output directory to reproduce the
extended consumer. Earlier evidence files remain tied to their recorded source hashes.

The author's immediate goal is Raven runtime await by default, backed by stability
and improvement evidence. Next is the explicit compiler/managed-await integration
contract and equivalent comparison cases. This POC does not change Raven defaults.

## Async-body invocation ownership — 2026-10-11

A private interpreter `RuntimeAsyncStart(fn<Int32>) -> Int64` now runs a body
eagerly until it returns or first parks on a pending operation. Its caller receives
an operation handle at that point. Returning from the body completes that operation
with the integer result; completion belongs to the runtime, so manual Complete cannot
overwrite it. A cancellation outcome wins over a subsequent body return. Already
completed bodies and nested starts use the same mechanism. Frame budget admission
precedes operation publication; receiver roots travel with the detached body.

This mirrors the synchronous-prefix/returned-operation responsibility of .NET async
invocation. It is separate from suspension alone: Raven's existing CLR lowering
relies on the CLR to supply this responsibility through async method metadata.
neoCLR cannot safely enable that lowering just by omitting state-machine generation.
The POC uses a checked source-local delegate and private integer handle, not CLR
AsyncHelpers or the CLR async method flag. The next compiler contract must explicitly
connect body invocation, returned managed Task and awaited outcomes.

This remains an interpreter mechanics experiment. The source body is an ordinary
function, not a compiler-lowered Raven async method. Int32-only single-consumption
handles do not redefine Task semantics. Body faults remain terminal invocation faults,
not captured Task outcomes, and cancellation does not implement public Task semantics.
Managed result rooting, repeated Task awaits, fault/cancellation projection and
JIT/AOT support are still gates before ordinary await or default switching.
No performance improvement is claimed by this slice.

`Await.rvn` includes an eager body whose Phase is 1 when Start returns, then 2
when its awaited result becomes available. [Invocation evidence](start-validation.json)
records the compiled consumer. Focused tests additionally cover already completed
bodies, nested starts, frame limits, producer authority and late return after cancel.
The reproduction script is unchanged; use a dedicated output directory.

## Shared completed outcomes — 2026-10-11

The private operation bridge now retains successful outcomes for repeated awaits
and allows multiple parked activations to await the same operation. Completion is
still one-shot; delivery is no longer consuming. This aligns result observation
with .NET Task semantics. A timer becomes terminal when its ready outcome is read,
so subsequent cancellation cannot overwrite an observed success.

New private `RuntimeAsyncRelease(Int64) -> Boolean` explicitly releases a terminal
handle. It returns false for pending, foreign, unknown or already released handles,
and when an active/parked waiter or a running/parked body still uses the operation.
Release is serialized with dispatcher transitions; it cannot retract an admitted
waiter's outcome. Reads after successful release fault. The later pending-self-await
slice below rejects direct self-waits; general cycle detection remains unsupported.

This is experimental handle bookkeeping, **not a Task.Dispose requirement**. The
future managed projection needs lifetime ownership tied to managed reachability;
applications should not manage these handles. The 64-operation budget now counts
retained terminal outcomes until release/context teardown. Earlier consuming-handle
behavior and reports describe prior revisions. Results remain Int32 only; terminal
invocation faults/cancellation still prevent full Task outcome semantics, and no
compiler default or native/JIT capability changes in this slice.

The compiled consumer uses two eager bodies awaiting a shared completion, attempts
release while the second is parked, then re-awaits both results. Focused checks also
cover repeated reads, stale handles, capacity recovery only on release, and refusal
to release a cancelled body that is still parked. See
[shared-result evidence](shared-results-validation.json). Public API/reference and
website capability claims remain unchanged. No performance improvement is asserted.

## Ordinary Raven await compatibility — 2026-10-11

`AwaitLanguage.rvn` now connects the ordinary Raven awaiter protocol to private
runtime operations. Operation returns itself from GetAwaiter; IsCompleted queries
the runtime, GetResult retrieves the outcome, and OnCompleted submits an adapter
callback. That callback parks through RuntimeAsyncAwait and invokes the generated
continuation after resumption. The consumer uses async Main, awaits a custom operation
and a managed Task, and re-awaits the already completed operation.

This is deliberately a compatibility path: generated state machines still exist.
It does not implement compiler-native runtime await or change Raven defaults. It
proves that the current awaiter protocol can participate in runtime scheduling and
suspension. The adapter allocates an object and submits a callback for each pending
await; no performance improvement is claimed. The consumer exposes completion only;
its IsCancelled=false satisfies the current target cancellation protocol but does
not qualify cancellation adapters. Int32-only outcomes and invocation-terminal
faults remain POC limits. Custom awaiters are not automatically runtime-aware.

New private InternalCall `neoCLR.Runtime.RuntimeAsyncIsCompleted(Int64) -> Boolean`
returns terminal readiness without consuming the outcome. Unknown/released handles
fault. Observing an expired timer materializes completion, so later cancellation
cannot overwrite observed success. The existing 64-handle budget/lifetime limits
remain unchanged. No managed library API or separate public awaiter type is added.

The compiled consumer checks pending delivery, GC before resumption, the completed
fast path without another registration, and completion between a false readiness
check and callback publication. A submitted adapter rechecks the operation when it
runs, preventing a lost completion in this serialized interpreter path. Cross-thread
registration races and native/JIT execution are not qualified.

The older bundled compiler recurses when the readiness property calls an attributed
intrinsic in the same file. The general fix from Raven main `d4c5e5128` is now carried
to the separate native compiler line and qualified with the consolidated consumer.
Use the fixed compiler explicitly; the historical bundle itself is unchanged.
[Single-source evidence](single-source-validation.json) includes compiler dependency,
runtime, source and reference hashes. The earlier two-file report remains historical.
The next native-await compiler contract must
preserve this awaiter behavior while replacing generated continuations with runtime
activation resumption and supplying managed Task outcome/lifetime ownership.

[Recorded language-await evidence](language-await-validation.json) includes binary,
source and compiler hashes and the generated state-machine name markers. Reproduce:

```sh
python3 docs/experiments/runtime-async/validate.py --language-await \
  --bundle target/data-cases-bundle \
  --compiler /path/to/Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --runtime target/scheduler-wait-validation/debug/neoclr \
  --output target/runtime-async-language-await
```

Use a runtime built from this slice. The default script mode still runs the direct
runtime-intrinsic consumer. Public API reference/website capability claims remain
unchanged; these are source-local experimental services and adapters.


## Runtime operation cancellation delivery (2026-10-11)

Two private TaskDispatch InternalCalls extend the interpreter POC:
`neoCLR.Runtime.RuntimeAsyncIsCancelled(Int64) -> Boolean` observes cancellation
without waiting, and `RuntimeAsyncWaitCompletion(Int64) -> Boolean` parks until
terminal completion, returning true for success or false for cancellation. Both
reject unknown/released/context-foreign handles. Waiting retains the outcome and
shares the activation ownership/release guards of RuntimeAsyncAwait; it does not
read a cancelled result. Host invocation cancellation remains terminal.

The source-local awaiter now projects IsCancelled from the runtime and its explicit
OnCompleted adapter waits for terminal notification before invoking the generated
continuation. Existing Raven cancellation lowering marks the managed Task cancelled
without calling GetResult. This separates notification from outcome retrieval, as
in .NET's awaiter model; the boolean bridge and explicit IsCancelled protocol are
provisional neoCLR representation, not a new public Task contract or .NET exception
projection. Direct RuntimeAsyncAwait of a cancelled operation still faults.

Pending cancellation, already-cancelled fast paths and cancellation during
registration pass in the compiled consumer; a later independent await still works.
The interpreter has 21 focused operation tests plus five operation-state unit tests.
Evidence: [cancellation validation](cancellation-validation.json).
Runtime Contract configuration and compiler lowering are unchanged. This uses the
native compiler line `f51d0561f`; generated state machines remain. No public library
API, faulted-Task projection, managed lifetime, cross-thread race qualification or
JIT/AOT suspension support is introduced. Native activation/outcome metadata and
backend support must eventually replace these private integer-handle signatures.


## Pending self-await diagnostic (2026-10-11)

The private interpreter runtime-async POC now rejects a body's attempt to await its
own pending operation, including from ordinary helper calls. This applies to both
RuntimeAsyncAwait and RuntimeAsyncWaitCompletion. It raises a terminal RuntimeError
Fault at the offending call instead of parking forever. An independently dispatched
callback has its own activation boundary and may still await the submitting body.
A body may observe its own already-terminal cancellation through completion waiting.

This is a deliberate private POC diagnostic beyond .NET Task's general await
contract: it catches a provably unsatisfiable self-dependency before parking, at the
cost of a scan to the current activation boundary when establishing a runtime wait.
It is not general deadlock detection; indirect cycles and external missing producers
remain possible. No public Task guarantee, compiler lowering, Runtime Contract option,
metadata encoding or native/JIT capability changes. The author explicitly reaffirms
that Fault is non-recoverable: faults still end the invocation and are not stored as
recoverable task outcomes. Host cancellation and resource-limit faults remain terminal.

Validation: the regression previously required watchdog cancellation; 24 focused
runtime_async_await executable tests now pass, covering both wait forms, ordinary
helper attribution, terminal cancellation and independent callback waits. Existing
eager/nested body, repeated/shared result, GC, release and host cancellation checks
pass in that same suite. Compiler-native await/default switching and comparative
performance qualification remain open.


## Unit-result runtime async POC (2026-10-11)

The interpreter's private TaskDispatch bridge now accepts
`RuntimeAsyncStart(fn<Void>) -> Int64` alongside the existing integer body form.
`RuntimeAsyncCompleteUnit(Int64) -> Boolean` completes a pending manual operation
once with unit; runtime-started bodies retain completion authority.
`RuntimeAsyncAwaitUnit(Int64) -> Void` suspends until a unit outcome is ready and
retains it for repeated/shared observation. Unknown/released handles, result-kind
mismatches and cancelled-result reads fault. Existing cancellation notification,
release guards and pending-self-await rejection apply. Fault remains non-recoverable.
Timers still produce Int32 zero; they are not implicitly converted to unit.

This supplies the private operation foundation needed for Raven's `Task<()>`,
comparable to .NET Task's non-value completion. It does not introduce another public
Task type or select a generic-result ABI. The runtime stores a distinct unit outcome
rather than an integer placeholder. No managed-reference result/rooting contract is
added. General typed results and managed lifetime remain open.

Native intent is an inhabited unit result. The temporary compiler transport may
encode the intrinsic return as CLI no-result; that form resumes without pushing a
value. An inhabited Void result resumes with a unit value. The function-object start
contract remains `fn<Void>`, and the runtime owns completion and frame retention.
Compiler lowering and Runtime Contract configuration are unchanged; public Task
integration/default switching and JIT/AOT suspension are still not implemented.
Future native await metadata/backend integration must preserve both outcome kind
and stack effects rather than standardize these source-local integer handles.

The compiled Raven consumer starts a unit body, observes its first suspension,
awaits it through an ordinary `Task<()>` adapter, checks resumption/GC, repeats the
unit await and releases the terminal operation. Generated state machines remain in
the adapter; the ordinary unit body itself uses retained runtime frames. Evidence:
[unit validation](unit-validation.json).

The consumer exposed an entry-drain ownership bug: a saved frame depth could match
an unrelated helper returning within a restored activation, incorrectly injecting
unit into that helper's caller. Entry-drain resumption now marks the actual owning
frame, preserving identity through parking/restoration. The focused regression
reproduces the mixed entry-drain/runtime-body scenario. Validation: 30 operation tests,
six operation-state tests, eight yield tests, five entry-result tests, two scheduler
submission tests and three timer tests pass, plus the compiled consumer (54 checks).
No public API reference or website capability claims change in this private slice.

Author clarification: runtime async is canonical; generated state machines are a
migration/testing mechanism with eventual retirement intended. They remain useful
for finding compiler/runtime integration bugs. Defaults remain unchanged until the
runtime path is qualified; no performance improvement is claimed here.


## Managed Task boundary and idle dispatch (2026-10-11)

[ManagedTask.rvn](ManagedTask.rvn) is an executable, source-local candidate for
compiler lowering. `Start<T>` invokes an ordinary body eagerly through the private
unit-result runtime service and returns the body's existing managed `Task<T>`.
`Promise<T>` retains the result; the runtime retains the executing frames. The
body and its helpers contain no generated state machine. Async Main remains a
state-machine consumer, including an inline await in a short-circuit condition.
This is **not yet compiler-native lowering** or a public Task implementation change.

The candidate separates three responsibilities:

- A managed producer owns reference, value or unit results and cancellation.
- A private runtime operation owns the retained body's execution until it returns.
- A temporary notification adapter turns managed Task completion into a unit
  signal that can resume a retained frame. `OnCompleted` is used only at that
  protocol boundary and for explicit cancellation observation.

`Wait<T>` returns false on cancellation; the hand-written body immediately returns
`Cancelled`, without calling GetResult or treating a Fault as an outcome. The
compiler would need to generate that control transfer, including scope cleanup.
Fault stays invocation-terminal. The successful path releases the temporary signal
before reading the managed result. Body ownership is released after the runtime
publishes completion, not merely when the Promise becomes observable. Eager bodies
release immediately; suspended bodies currently need a separate cleanup callback.
References remain rooted by ordinary retained frames and managed Promise storage;
this does not add generic/reference values to the runtime's private handle table.

Compared with .NET's Task/TaskCompletionSource and generated async builder, this
keeps the managed operation/producer split while replacing body state fields with
retained runtime frames. Compared with introducing a generic runtime result ABI,
it reuses existing managed generic storage and tracing. Costs include a temporary
signal, callback and cleanup activation, plus dependence on the legacy TaskQueue
notification adapter. No allocation or speed improvement is claimed. Production
lowering must resolve lifetimes, cancellation cleanup, entry support and notification
routing before adopting this as a contract. A callback that itself suspends inside
legacy TaskQueue.Drain cannot depend on later work in that same serial queue; the
runtime prevents recursive draining, but does not turn that queue into concurrent
execution. This consumer's retained bodies run outside its queue callbacks.

The first run exposed starvation: when every activation was parked, the POC waited
before servicing managed notifications or host completions. Idle parked activations
now fall through the shared dispatcher. Completing a retained callback also marks
managed notifications for draining. A suspended legacy Drain is never entered
recursively; its outstanding drain request remains pending. Focused regressions
cover a managed completion waking a retained body and a queue callback that itself
suspends on a timer. Runtime Faults are not captured.

Validation: 32 runtime-operation tests, eight yield tests, five entry-result tests,
two scheduler-submission tests and three timer tests pass (50 focused checks).
The compiled consumer covers two bodies awaiting one managed reference Task, a
reference local retained across suspension/GC, repeated result reads, pending and
pre-existing cancellation, pending unit completion, eager integer/unit completion,
and 80 iterations of both eager and suspended bodies (beyond the 64-handle limit).
[Exact compiler/runtime/source hashes and commands](managed-task-validation.json).
The portable compiler fix `0c9a8881f` has 65 focused tests passing on the native
compiler line (`codex/source-object-metadata-resolution`); Raven main is unchanged.

Run with `validate.py --managed-task --bundle <bundle> --compiler <rvnc.dll>
--runtime <neoclr> --output <dedicated-directory>`. The compiler override is required
for this consumer until the selected bundle includes the portable condition fix.
No public reference snapshot or website capability claim changes. Interpreter-only;
JIT/AOT, automatic lowering/default selection and comparative benchmarks remain open.


## Raven runtime-async language POC (2026-10-11)

The native Raven compiler now supports ordinary `async`/`await` through
`--runtime-async-poc RuntimeAwaitProvider.Bridge`, with explicit
`--async-library System.Runtime` and the source-included
[provider](RuntimeAsyncBridge.rvn). The [language sample](RuntimeAwait.rvn) uses
ordinary async module functions and an instance method, including async Main.
There is no hand-written per-method async wrapper in that sample and metadata
inspection finds no AsyncStateMachine references. This meets the bounded
language-support POC milestone; it does not switch defaults.

The native backend creates an ordinary closure capturing parameters and a reference
receiver, invokes it through the selected Start helper, and rewrites awaits to a
single Task evaluation, a completion wait and a successful GetResult. Cancellation
returns a cancelled outcome immediately. The provider completes an existing managed
Promise<T>, so Task<T> remains the operation/outcome and Promise the producer.
The runtime retains frames and owns dispatch. Fault is never captured as an outcome.
The non-unit fallthrough guard remains terminal, including after System.Fail.

The explicit emission contract is separate from Raven's CLR runtime-async flag.
It selects a source-assembly type with generic static Start, Wait, Complete and
Cancel methods whose Task, callback and result signatures agree. The current
provider uses TaskOutcome<T>, existing private runtime operations and legacy managed
notifications. Default compilation still generates state machines. No public Task,
Scheduler, native ABI or API-reference addition is made by this experiment.

Supported qualification covers nongeneric module/static and reference-instance
methods on nongeneric owners, supported immutable parameter captures, Task<T>
awaits, explicit/implicit returns, unit and reference results. Generic methods and
owners, value receivers, nested async functions, other awaitable patterns, and
return expressions inside value expressions are rejected. Portable expression and
capture restrictions still apply. More complex cleanup/protected regions need
separate qualification. Unsupported shapes do not silently switch back to state
machines. Project/default selection, source-level async stack presentation and
JIT/AOT suspension remain open.

### Samples and GC qualification

- [Language sample evidence](runtime-language-validation.json): eager, pending,
  repeated and nested awaits; live references across GC; instance/parameter capture;
  unit completion and cancellation. Missing provider/configuration fail compilation
  without producing an artifact.
- [Fault sample](RuntimeAwaitFault.rvn) and [evidence](runtime-language-fault-validation.json):
  a fault after suspension terminates the invocation with the original UserFault;
  code after the awaited task never executes. No recoverable faulted Task is created.
- [GC sample](RuntimeAwaitGc.rvn) and [evidence](runtime-language-gc-validation.json):
  80 nested suspend/resume cycles, preserved live locals/results, positive reclamation,
  and no continuing growth beyond an eight-object tolerance after the midpoint.
- [Independent heap observations](runtime-language-gc-heap.json): two executions each
  allocate 4,736 objects, reclaim all 4,736 and finish with zero live objects; peak
  heap size is 63 and each records 242 collections. This checks reclamation during
  repeated execution and at invocation teardown, not just survival of live roots.

The GC sample exposed a second dispatch gap: an activation could post managed
notifications and park again after the previous drain had finished. Parking now
requests a fresh drain. If a legacy queue callback itself suspends, finishing that
Drain clears the request rather than starting an endless series of empty passes.
A watchdog regression covers repeated parking; the suspended-queue regression
continues to reject recursive draining. All 33 runtime-operation tests and 18
entry/scheduler/timer/yield checks pass (51 total).

Run `validate.py --runtime-await` with the explicit bundle/compiler/runtime/output
arguments above. Add `--runtime-sample gc` or `--runtime-sample fault` for those
samples. Each report records artifact hashes and checks the absence of generated
state machines. The private release host `runtime_async_interpreter` additionally
checks final heap state with metadata loading/verification outside invocation timing.
This is POC measurement scaffolding, not a new benchmarking-framework API.

Compared with .NET's TaskCompletionSource/builder split, managed outcomes stay in
Task/Promise while execution state moves to runtime frames. The source provider's
signals, callbacks, completion unions and lifetime objects add allocation overhead.
The direct benefits to qualify are simpler generated execution state and lower
execution cost; neither general throughput nor allocation superiority is assumed.
Further benchmark evidence should guide replacing that scaffolding before defaults.


## Matched language benchmark — 2026-10-11

[Raw results](runtime-await-benchmark.json) compare the same
[ordinary async source](RuntimeAwaitBenchmark.rvn), compiler and interpreter host,
changing only generated-state-machine versus runtime-async lowering. Compiler native
revision: `e96e9b53b`; runtime qualification: `dcfdef4a`. Exact artifact hashes,
commands, platform and individual observations are in the report.

| Workload | State-machine median | Runtime-async median | Allocations, state-machine / runtime |
| --- | ---: | ---: | ---: |
| 10 completed nested awaits | 114.19 ms | 95.70 ms | 324 / 365 |
| 100 completed nested awaits | 1,089.92 ms | 913.27 ms | 3,114 / 3,515 |
| 10 pending nested awaits | 232.47 ms | 186.23 ms | 504 / 596 |
| 100 pending nested awaits | 2,294.14 ms | 1,822.12 ms | 4,914 / 5,816 |

At 100 operations, runtime async takes about 16% less time for completed awaits and
21% less for pending awaits, while allocating about 13% and 18% more objects.
All measured invocations reclaim every allocated object and finish with zero live
objects. Peak live counts are 82 / 64 for completed and 96 / 100 for pending workloads.
The source provider improves these interpreter timings but does not improve allocation
counts. Notification, signal and lifetime scaffolding need optimization before defaults.

Method: two external process rounds with mode order reversed in the second round;
four fresh invocations per process, discarding the first as warmup, leaving six timed
observations per case/mode. Metadata loading and verification are measured separately
and excluded; invocation startup, GC and checksum validation are included. Counts of
10 and 100 expose workload scaling. This is a single-host exploratory experiment,
without confidence intervals, HTTP qualification, or JIT/AOT/.NET comparisons. It
measures object counts, not allocated bytes or peak process memory.

Reproduce with `benchmark.py --recipe <runtime-await validation.json> --host
<release runtime_async_interpreter> --output <fresh-directory>`. The recipe supplies
the exact compiler and bundle selection. The benchmark validates checksums and final
heap state; it is private measurement tooling, not a public framework addition.


## Independent operation proof — 2026-10-11

[RuntimeOperation.rvn](RuntimeOperation.rvn) supplies a second operation implementation
whose completion and Int32 result live in the runtime, with no Task/Promise backing.
Ordinary `await` suspends on it through an explicitly selected provider overload.
An Envelope supplies a separate awaiter, proving that the compiler calls GetAwaiter
once and reads the result from that returned object. OnCompleted deliberately faults:
the passing consumer does not route the operation through a managed callback adapter.
Async method *results* still use Task<T>; this proof generalizes awaited inputs only.

Raven native revision `0764064d4` admits exact source-provider Wait mappings for
reference awaiters (nongeneric, or one generic parameter constructed with the await
result type). Missing/ambiguous mappings fail compilation. No implicit conversions,
value awaiters or extension/static GetAwaiter are admitted. The existing binder's
TaskAwaiter and cancellation protocol remain required. These are bridge constraints,
not the final runtime metadata contract; no new public runtime API is introduced.

[Executable evidence](runtime-operation-validation.json) covers 80 cycles with two
parked readers, shared/repeated results, GC during suspension, exactly-once completion,
awaiter/result read counts, cancellation without reading an absent result and completion
immediately before waiting. Native metadata contains no generated state machines.
Both invalid mapping cases fail without output artifacts. The existing
[Task language consumer](runtime-operation-task-regression.json) also passes.
[Heap evidence](runtime-operation-heap.json) records two independent executions, each
with 3,028 allocations/reclamations, zero final live objects, peak 92 and 163 collections.
These timings are not a matched performance comparison with the previous workload.

A pending cancellation publishes terminal state before its suspended reader has
necessarily resumed. The test observes the reader's terminal outcome through an
explicit callback adapter and then awaits that observation; it does not assume that
Cancel executes continuations inline. Completed/cancelled handles are explicitly
released after readers finish. The sample exceeds the 64-operation budget over its
lifetime, checking handle reuse as well as collection of managed wrappers.

Run `validate.py --runtime-await --runtime-sample operation` with the explicit
bundle/compiler/runtime/output arguments. Reusing the dedicated output directory
now removes the old disassembly before inspection. Runtime implementation is unchanged
in this slice; the new behavior is native compiler mapping and executable qualification.

This establishes representation independence at the experimental provider boundary.
It does not yet establish an independently loadable interface implementation, native
producer ABI, multithreaded completion races, JIT/AOT suspension, or arbitrary async
return types. Next specify the recognized metadata contract, then replace explicit
per-type provider selection with validated implementation binding. Preserve these
samples and GC checks during that work. Relative to C#'s awaiter pattern, the language
shape is familiar while actual parking/resumption is runtime-owned; source-provider
mapping costs and final lifetime semantics remain explicit design work.


## Imported operation contract proof — 2026-10-11

[OperationContract.rvn](OperationContract.rvn) is compiled separately into
Operation.Contracts.dll; the [consumer](RuntimeContract.rvn) imports its generic
Operation<T> interface. A manual implementation keeps completion/results in runtime
storage, while an immediate implementation uses no runtime handle. Both return the
interface from GetAwaiter, so one generic provider mapping handles both concrete
types. An interface-typed consumer exercises both pending and ready implementations.
This uses existing metadata interface identity, conformance and runtime dispatch;
no compiler or runtime implementation change was needed beyond Raven `0764064d4`.

[Validation](runtime-contract-validation.json) covers the 80 shared/repeated wait
cycles, distinct awaiter, cancellation and GC cases plus both interface implementations.
Missing WaitCompletion is rejected with RAV0330; missing contract assembly fails
loading before execution. The consumer emits no generated state machines.
[Heap observations](runtime-contract-heap.json) show 3,064 allocated/reclaimed objects,
zero final live objects, peak 94 and 163 collections in each of two executions.
The private host accepts optional dependency assemblies; its original invocation
also passes the previous independent-operation sample. These are correctness/GC
observations, not a matched interface-dispatch performance benchmark.

Run `validate.py --runtime-await --runtime-sample contract` with the established
bundle/compiler/runtime/output arguments. It compiles the contract library before
the consumer, records its hash and loads it explicitly. For heap inspection pass
Operation.Contracts.dll after the repetition count to `runtime_async_interpreter`.

The experimental interface's members are IsCompleted, IsCancelled, GetAwaiter,
GetResult and WaitCompletion, plus the existing TaskAwaiter callback requirement.
WaitCompletion suspends through runtime services and returns false for cancellation;
only true permits result retrieval. These source members are documented here for
reproduction, not added to the public runtime library/API reference. The runtime
has not acquired special knowledge of this interface or arbitrary methods named
WaitCompletion. Implementations must still supply the tested runtime-service adapter.
Foreign/native ABI, thread races, hostile metadata validation and JIT/AOT remain open.

**Author clarification:** the desired shared runtime boundary is an awaitable-operation
interface, with concrete implementations supplied by integrating platforms. Managed
languages may project it as Task<T>. Interfaces are a primary cross-language mechanism,
not the only allowed form of interop. WinRT is a precedent for the boundary, not a
specification to copy. This proof qualifies ordinary metadata machinery; the final
runtime-facing contract must separate operation observation/ownership from Raven's
GetAwaiter syntax adapter and the private provider. No final member list is selected.


## Managed HTTP showcase and scoped cleanup — 2026-10-11

The existing [HTTP client](../native-http-client/Client.rvn) and
[HTTP server](../http-server/Server.rvn) showcase sources now pass with runtime-await
application lowering ([evidence](runtime-http-showcase-validation.json)). The client
resolves localhost and receives the exact UTF-8 greeting; the server runs queued work
while accept is pending, serves the greeting and closes. The peer uses HTTP/1.1, as
required by the tested library. Application artifacts contain no AsyncStateMachine
references. Imported runtime libraries keep their existing implementation; this is
mixed runtime-body/library execution, not a complete library migration.

[RuntimeHttp.rvn](RuntimeHttp.rvn) additionally runs eight loopback exchanges with
both sides inside one managed invocation. Its async handler awaits scheduled work;
GC runs during pending requests and inside the handler. HttpClient uses synchronous
`use` disposal across suspension. [Execution](runtime-http-validation.json) passes;
[two heap observations](runtime-http-heap.json) each allocate and reclaim all 4,657
objects, finish with zero live objects, peak at 326 and record 57 collections.
This is correctness/GC evidence, not a throughput benchmark or native-resource leak proof.

Raven native `beed357bf` fixes the exposed async-use admission gap behind explicit
RuntimeDisposalContract.AllowAsyncScopeExitCleanup, selected by --runtime-async-poc.
Retained bodies reuse ordinary scope-exit lowering. [RuntimeUse.rvn](RuntimeUse.rvn)
checks resources remain alive while suspended, reverse-order exactly-once disposal on
successful and cancelled exits, and implicit unit-return disposal. [Evidence](runtime-use-validation.json)
includes rejection without the runtime-async option. All 26 focused compiler scope-exit
tests pass, including default async rejection and continued iterator rejection.
Terminal Fault remains invocation-terminal: synchronous scope-exit cleanup does not
introduce managed fault unwinding or async Dispose support.

Reproduce with `validate.py --runtime-await --runtime-sample http` or `use`, using the
explicit compiler/bundle/runtime/output arguments. Then run `http_showcase.py --recipe
<passing http validation.json> --output <dedicated-directory>` to compile the unchanged
showcase sources and drive external peers. Reports include actual compiler/provider/
runtime/reference hashes, application hashes, commands and output.

Current limits: interpreter on this host; standalone *sources* are qualified, but
project-level runtime-await selection is the next slice. JIT/AOT suspension, Windows
qualification, timeout/cancellation network cases and a production provider remain
readiness gates. Keep defaults unchanged. The HTTP client/server projects are the
author-selected showcase and acceptance workload, not the interface experiments.


## HTTP showcase project integration — 2026-10-11

Native Raven `2ef670ff1` adds the explicit project property
RavenNeoClrRuntimeAsyncHelper. The existing client/server Native.rvnproj files include
the experimental provider only when this property is nonempty. Both projects now
build through `rvnc neoclr --project` and run their loopback acceptance cases with
runtime-await application bodies ([evidence](runtime-http-project-validation.json)).
No per-type operation-interface redesign is required.

Reproduce using `http_showcase.py --projects --recipe <passing http validation.json>
--output <dedicated-directory>`. The runner supplies NeoClrBundleRoot and
RavenNeoClrRuntimeAsyncHelper=RuntimeAwaitProvider.Bridge as project properties via
the environment, compiles the actual checked-in native project files, preserves the
resulting artifacts and checks both external peers. For manual project use, those
properties may instead be set in project configuration. The matched native compiler
is required; installing an older bundle compiler does not acquire this feature.

The provider property requires RavenNeoClrAsyncLibrary, which the existing bundle
props select. A separate cleanup project passes --project --run, including cancellation
and implicit-return use cleanup. Clearing the async-library property fails before
publishing output. A control build of the real server without opt-in still emits
state machines. Project-local configuration does not rebuild dependency libraries
under different lowering or switch default execution modes.

This completes the bounded interpreter project POC. Broader HTTP timeout/cancellation,
terminal-fault cleanup, production provider integration, performance remeasurement and
JIT/AOT suspension remain readiness work. Windows runtime-async qualification is also
open. Existing JIT/AOT state-machine HTTP support remains valid, but is not evidence
for runtime suspension in those modes. Runtime await is not ready to become default.


## HTTP lifecycle qualification — 2026-10-11

[RuntimeHttpLifecycle.rvn](RuntimeHttpLifecycle.rvn) passes timeout, cancellation,
malformed-response and terminal-Fault cases using retained application bodies.
The external peer checks request delivery and socket closure; recoverable outcomes
then make a second successful request. Synchronous use cleanup runs exactly once.
The Fault case remains invocation-terminal and does not claim managed unwinding.
[Evidence](runtime-http-lifecycle-validation.json) records all four interpreter cases
and verifies no generated async state machines in the application. Reproduce with
`http_lifecycle.py --recipe <passing HTTP validation.json> --output <new-directory>`.

The sample captures CancellationTokenSource rather than its value-type token:
retained-body closure capture of arbitrary structs is still a compiler limitation.
These checks establish lifecycle behavior, not throughput or complete native-resource
leak freedom. The author now explicitly requests continuation through JIT and AOT
runtime-suspension support; existing state-machine support does not satisfy that gate.

## Compiled runtime suspension: JIT and AOT — 2026-10-11

The explicit native POC now retains and resumes **compiled** Raven application
bodies in both JIT and native AOT on macOS ARM64. There is no interpreter fallback
and no generated application async state machine. Both actual HTTP Native.rvnproj
projects pass external-peer qualification, including client localhost DNS and server
work while accept is pending. The separate lifecycle consumer passes timeout,
cancellation, malformed-response recovery and terminal Fault in both modes.
[Reproducible qualification](native-validation.json) records compiler/runtime/source
hashes, project properties, commands and results.

The language and scoped-cleanup samples cover nested/shared/repeated waits,
receiver/reference retention, cancellation, eager completion and unit results.
The lifetime contract checks pending release rejection, exactly-once completion and
release, and retained outcome reads in interpreter, JIT and AOT. The existing
100-operation pending-await workload also runs as a checksum/stability test, **not**
a performance benchmark. It caught starvation of queued lifetime callbacks by ready
continuations; the pump now services deliveries even when continuations are ready. AOT repeats language and cleanup entries four times on the
same heap. Both matched native hosts require all parked stacks and operation roots
to be released before returning; after fault rendering and clearing fault roots,
final collection must empty the heap. This is checked teardown, not a general
native-resource leak proof.

Implementation layers:

- Raven native compiler `2ef670ff1` and the existing source provider are unchanged.
  The Runtime Contract still selects explicit Start/Wait/Complete/Cancel helpers,
  with synchronous use cleanup enabled only for retained bodies.
- The shared Cranelift JIT/AOT backend adds `--bind-runtime-async`, exact private
  service validation and a compiled entry pump. It also binds the existing explicit
  GC.Collect service for this experiment. Other managed GC counters remain a gap.
- The private host runs same-thread cooperative stacks for one active invocation. A body starts eagerly, parks
  only at a runtime wait, and resumes independently of notification/lifetime callbacks.
  One default-queue drain may remain parked; the pump does not recursively drain it.
- Published managed root chains are registered before switching away. GC visits all
  parked chains as well as the current chain. No arbitrary C-stack scan is introduced.
  Entry reset and exclusive-root checks reject parked guest frames. A focused native
  collector test checks two chains, out-of-order resumption and terminal abandonment.
  The stack-boundary test also retains its intended frame allocation at -O2, avoiding
  scalar replacement that previously invalidated that test.
- Scheduler delivery is acknowledged only after transferring a rooted callback to
  the retained activation. Callback ownership then lasts through callback completion.
  Fault stays invocation-terminal; teardown abandons other activations without managed
  exception unwinding and finishes before JIT code can be freed.

The temporary carrier uses macOS `ucontext`, which the SDK explicitly marks deprecated
and unsupported. This is **not the production stack choice**. Each activation reserves
1 MiB plus two guard pages; the private limits are 128 operations and 128 simultaneous
activations, within the existing host-root and I/O limits. Stack-budget checks use
registered activation bounds. These fixed reservations, context-switch cost, per-drain
stack allocation and lack of preemption are material costs. Windows, dynamic stack
sizing, multi-core execution, custom scheduler/context propagation across stacks,
logical async-parent trace stitching and performance comparison remain open. The backend
rejects this carrier option for Windows rather than implying support.

Task.Run remains a distinct concurrent-work service. The combined RuntimeHttp.rvn
consumer uses it and is still interpreter-only; it was not silently mapped to the
cooperative scheduler. The actual client/server projects and lifecycle workload do
not need it. Referenced libraries still use their existing state-machine implementation.
A production provider, library migration, native GC metric parity, CLI/build-kit
integration and default switching remain separate work. Existing compiler method-shape
and arbitrary-struct capture restrictions still apply.

Reproduce after building the current release `tools/aot-poc` backend and native text
archive, using the matched Raven compiler/bundle and the existing passing source and
macOS HTTP build recipes:

```sh
python3 docs/experiments/runtime-async/validate_native.py \
  --recipe target/runtime-http-use-final/validation.json \
  --native-build target/entry-owned-http/build.json \
  --output target/runtime-native-evidence-v2
```

The runner rebuilds sources and both projects. It treats the native build report as a
command recipe, rebuilding all C adapters and recording current hashes; old executable
artifacts are not accepted as new evidence. `--backend` and `--text-archive` select
explicit tool locations. Native generation fails without the opt-in service bindings;
this does not alter normal compiler or execution defaults.

### Architecture comparison and provisional decision

The existing [.NET comparison](../../runtime-scheduling-design.md) separates compiler
state machines, managed Task/Promise policy and runtime scheduling. This POC deliberately
moves suspended **application call stacks** into the runtime while keeping managed
operation results. The benefit demonstrated here is one ordinary compiled call chain
across an await with explicit GC ownership, not a measured speed improvement.

Alternatives remain compiler state machines (already supported, smaller captured state),
a maintained context library such as [Boost.Context](https://www.boost.org/doc/libs/latest/libs/context/doc/html/context/ff.html)
(ABI-aware register/stack preservation but an additional dependency), or a small
platform-specific carrier (more implementation and ABI validation responsibility).
[Windows fibers](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-switchtofiber)
are another carrier candidate, not an implemented port. Sources were reviewed
2026-10-11; the local macOS26.2 SDK ucontext.h also supplies the deprecation constraint.
Choose the production carrier only with stack/GC/fault tests and measured memory and
switching costs. No stable foreign ABI or public green-thread API is introduced.

## Matched interpreter, JIT and AOT measurement — 2026-10-11

`benchmark_modes.py` compiles the same nested-await source and provider with two
lowerings: compiler-generated state machines and opt-in runtime suspension. It uses
10 and 100 completed or pending operations, checks the sum, and requires the pending
path to remain incomplete until its producer completes. This compares neoCLR's two
implementations; it is **not a .NET runtime benchmark**. The architecture comparison
above remains the basis for the experiment.

```sh
python3 docs/experiments/runtime-async/benchmark_modes.py \
  --recipe target/runtime-http-use-final/validation.json \
  --host target/scheduler-wait-validation/release/examples/runtime_async_interpreter \
  --output target/runtime-modes-benchmark-final
```

The passing source recipe supplies compiler/reference paths; `--qualification`
defaults to `native-validation.json` and supplies native build commands. Both native
hosts are rebuilt from current sources. Current tool, source, reference and artifact
hashes are recorded, and inputs are checked again after measurement. A changed input
invalidates the run. Recipes retain absolute local paths and need matching local
builds; they are not portable binary bundles.

Each process executes four fresh invocations, discarding the first. Two rounds
reverse engine/lowering order, giving six measured invocations per cell. Native JIT
and AOT use the same C entry wrapper, including fresh heap/scope setup and terminal
cleanup. Interpreter timing also includes invocation teardown; its implementation is
not identical. Loading/verification and JIT generation are outside sample time.
Process wall time and JIT phase times are retained separately in the raw report;
AOT object generation/linking are build costs, not execution samples.

Every invocation must return zero with no guest output. The native host checks root,
operation and final-heap teardown and equal allocated/reclaimed GC block counts;
the interpreter host requires zero final live objects. Compare allocation counts
**within an engine**, not between interpreter objects and native GC blocks. These
counters exclude native stack mappings, RSS and reserved address space. They do not
establish memory efficiency of retained stacks.

These are small, single-threaded local microbenchmarks, with six samples and no
confidence interval. They do not measure HTTP throughput, concurrent loads, Windows,
production carrier performance or a fully migrated runtime library. Keep default
readiness and stack-memory qualification open even if an individual timing improves.

Recorded on Apple M1, macOS-27.0-arm64-arm-64bit. Median fresh-invocation times
for 100 operations ([raw report](runtime-await-modes-benchmark.json), including
10-operation results and min/max ranges):

| Workload | Engine | State machine (ms) | Runtime suspension (ms) | Runtime / state machine |
|---|---|---:|---:|---:|
| completed-100 | interpreter | 1176.91 | 989.33 | 0.84× |
| completed-100 | jit | 15.25 | 35.48 | 2.33× |
| completed-100 | aot | 14.35 | 32.26 | 2.25× |
| pending-100 | interpreter | 2514.89 | 2089.29 | 0.83× |
| pending-100 | jit | 31.41 | 108.01 | 3.44× |
| pending-100 | aot | 30.49 | 104.72 | 3.43× |

All 192 invocations passed result and reclamation checks; 144 remained after warmup
discard. For these 100-operation cases runtime suspension uses about 16–17% less
interpreter time, but 2.25–3.44× the native state-machine time. Native runtime
suspension is still much faster than interpreter execution; that does not make it
an improvement over native state machines. The preliminary run showed the same
native regression direction. Small sample counts and observed outliers limit the
precision of these ratios.

The native allocation comparison at 100 operations also exposes a cost: completed
work grows from 3,114 to 4,118 GC blocks, and pending work from 5,514 to 7,421. Both
compiled modes reclaim every block. Interpreter counts grow from 3,114 to 3,515
objects (completed) and 4,914 to 5,816 (pending), also with zero final live objects.
The native carrier creates a guarded stack per retained body/callback, while the
provider introduces notification/lifetime adapters. These are profiling candidates,
not an attribution of the measured slowdown. GC collection frequency is recorded
as another candidate. Measure these components and peak retained stack bytes before
choosing pooling, fast paths or a replacement carrier. The canonical runtime-async
direction remains intact, but the present native POC has not met the improvement gate.

## Bounded native stack reuse — 2026-10-11

The next experiment isolates stack-mapping churn. The private carrier caches up to
8 completed guarded stack mappings **within one invocation**. A completed fiber's
callback root and parked root chain are released before its mapping enters the cache;
the machine context, operation and callback identity are cleared. Reuse initializes a
fresh context and preserves guard pages. Excess mappings are unmapped immediately.
Normal and terminal-Fault shutdown unmap both cached and active stacks before code
can unload. No mapping survives into the next invocation.

This does not change managed Task/Promise semantics or the comparison with .NET
compiler-generated state machines described above. It is a bounded implementation
experiment in the existing deprecated macOS carrier, not a production stack-pool API.
Immediate unmapping remains available as the measurement baseline. The tradeoff is
keeping otherwise-idle stack address space and touched pages until shutdown; eight
cached 1 MiB stacks can retain 8 MiB plus guard pages. Reserved bytes are not RSS.

Run `benchmark_modes.py` twice with distinct output directories and respectively
`--stack-cache-limit 0` and `--stack-cache-limit 8`. Both configurations use identical
instrumentation and source. Private counters report fiber starts, resumes, mappings,
unmappings and reserved stack bytes. `peakReservedStackBytes` is the **thread's
high-water mark across process invocations**, including guards and cached stacks;
`reservedStackBytes` must be zero after every invocation. The managed allocation
counters still exclude stack memory. These counters and the paired experiment
isolate mapping reuse; they do not provide a sampled CPU profile or RSS measurement.

`RuntimeStackReuse.rvn` starts 24 readers before completing their shared input, checks
retained objects after collection, then repeats the wave three times. Qualification
also repeats this consumer across four invocations in one process and retains HTTP,
cancellation, scoped cleanup and terminal-Fault coverage.

On the same Apple M1 host, [cache-off](stack-cache-off-benchmark.json) and
[cache-on](stack-cache-on-benchmark.json) each pass 192 invocations across interpreter,
JIT and AOT. For 100 operations, native mapping counts drop from **202 to 3**
(completed) and **703 to 5** (pending). Fiber/resume counts, managed allocations,
collection counts and peak reserved bytes are identical across the variants.
Completed/pending peaks are 3,244,032 / 5,406,720 reserved bytes, including guards;
every invocation ends with zero reserved stack bytes. This workload does not
exercise the worst-case idle-retention cost of the cache.

An additional [alternating AOT comparison](stack-cache-paired-benchmark.json) runs
eight rounds in reversed order, four invocations per process, dropping the first.
That gives 24 measured samples per workload/configuration and 128 teardown checks:

| 100-operation workload | Cache off median | Cache on median | Time reduction |
|---|---:|---:|---:|
| Completed awaits | 32.21 ms | 18.90 ms | 41.3% |
| Pending awaits | 106.08 ms | 58.17 ms | 45.2% |

```sh
python3 docs/experiments/runtime-async/compare_stack_cache.py \
  --without-cache target/runtime-stack-cache-off/results.json \
  --with-cache target/runtime-stack-cache-on/results.json \
  --output target/runtime-stack-cache-paired.json
```

The fresh three-mode run puts cached runtime suspension at about **1.31–1.91×**
native state-machine time for 100 operations. This narrows the gap; it does not
meet the improvement gate over state machines. These remain local microbenchmarks,
not HTTP throughput or a claim about .NET performance. The paired comparison
supports keeping bounded mapping reuse in this POC. Profiling context switches,
notification adapters and GC costs remains the next performance investigation;
a maintained carrier, Windows and resident-memory measurements remain open.

The new fan-out consumer initially hit existing compiler diagnostic RAV2712 for
await inside `for`. The initial slice used explicit indexed iteration to isolate stack
reuse. The author explicitly requests fixing that restriction next, with cancellation
and iterator-disposal qualification in both async lowerings; it is not a platform
semantic rule or an accepted permanent workaround.

[Cache-enabled qualification](stack-cache-validation.json) passes 24 JIT/AOT cases,
including both HTTP projects, timeout/cancellation/protocol recovery, terminal Fault,
retained-object fan-out and native GC/stack contracts. The fan-out consumer also passes
four consecutive invocations using the same native host heap. Existing guards and
root/operation teardown checks remain enforced.


## Await inside for: both lowering paths — 2026-10-11

The blanket cancellation-policy rejection is removed in the Raven compiler, with
iteration lifetime fixed rather than merely disabling the diagnostic. General
fixes are on Raven main (`a23705567`, `262109b7f`, `0845f8db9`) and deliberately
carried to the separate native compiler line (`e914a4c6d`, `b4b998648`). Arrays/ranges retain position; saved
value-type iterators are mutated by address; .NET protected cleanup distinguishes
suspension from source-scope exit. Hoisted scoped resources are disposed and cleared
when a jump leaves their scope. Native state machines select the existing explicit
scope-disposal contract; retained bodies retain their existing cleanup path.

[Qualification](for-validation.json) passes **12 runs**: `RuntimeForAwait.rvn` and
`RuntimeStackReuse.rvn`, both with generated state machines and with runtime await,
in interpreter, macOS ARM64 JIT and AOT. The first covers pending awaits, ordinary
completion, break/continue/return, labeled outward exits to both `for` and `while`,
cancellation and GC. The
second replaces the indexed workaround with `for` and exercises three waves of
24 retained readers. Runtime-body disassembly must contain no application state
machines; the control builds must contain them. Source/compiler/runtime/bundle and
binary hashes accompany the commands. The existing bundle is unchanged.

```sh
python3 docs/experiments/runtime-async/validate_for.py \
  --recipe target/runtime-stack-cache-validation-v2/validation.json \
  --interpreter-recipe target/runtime-http-use-final/validation.json \
  --output target/runtime-for-await
```

Shared .NET tests cover reference/value iterators, arrays, ascending/descending
ranges, pending waits, cancellation/exceptions, nested exits, and disposal before
another await. The final shared regression run passes 74 loop/resource-lifetime,
runtime-async and generator-delegation cases, including hoisted-resource exits to
both `for` and `while`. Earlier adjacent label/goto and ref-safety selections also
passed; unaffected evidence is reused.

This restores familiar .NET loop lifetime; it does not introduce a new public API,
change awaiter/ref-safety checks, or make terminal neoCLR Fault recoverable. Fault
still has no managed-unwind guarantee. Native qualification uses the existing
bounded compiler/backend profile and macOS stack carrier, not Windows runtime
suspension. HTTP behavior and performance code were unchanged; prior HTTP and
benchmark evidence is reused rather than rerunning that matrix.


## Async-generator investigation — 2026-10-11

The author also requested an [async-generator investigation](async-generators.md).
Raven's .NET target already supports generator production, pending asynchronous
advancement, delegation and asynchronous disposal, with focused execution evidence.
neoCLR has no corresponding async-iteration runtime/library contract yet, and the
retained-body POC excludes generator methods. The report separates those gaps from
ordinary `for` with `await`, compares .NET and ECMAScript contracts, and proposes
qualifying a managed pull protocol before retained generator production. API names,
operation representation and abandoned-generator lifetime remain open; this is an
investigation, not newly implemented native generator support.

### Async iteration and yield/resume prototype

The managed AsyncIterable/AsyncIterator/AsyncDisposable contract is implemented.
The explicit retained-body compiler POC consumes it with `await for` and awaited
lexical cleanup. A source-local producer now tests demand-driven yield/resume,
early stop, cancellation and independent iterator state in interpreter/JIT/AOT.
See [contracts, execution evidence and remaining generator work](async-generators.md).
Subsequent checkpoints add native Raven `yield`/`yield from`, generic reference
owners and [managed suspension ownership](async-generators.md#managed-ownership-integration--2026-10-11).
Live consumer tasks and external completion callbacks retain their producers; pending
abandoned cycles release frames, operation slots and native stacks. Ready work retains
strong ownership through dispatch. No default-mode switch or performance improvement
is claimed; Windows carrier and performance qualification remain open.
