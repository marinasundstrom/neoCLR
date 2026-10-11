# Runtime scheduling and async suspension

**Public Scheduler clarification (2026-10-11):** expose a managed `Scheduler`
contract, with scheduling implemented by the runtime. Task/Promise APIs stay in the
managed domain; queue storage and pumping remain implementation details. This selects
the public abstraction, not its exact members, namespace or customization model.
Earlier statements leaving the existence of a public scheduler open are historical.

**Queue visibility clarification (2026-10-11):** a task queue is an implementation
detail, not the scheduler architecture's application-facing foundation. The existing
public TaskQueue is a transitional dependency to remove through coordinated migration,
not a public abstraction to rename Scheduler. See the [migration boundary](#queue-independent-scheduling--2026-10-11).

**Latest author direction (2026-10-11):** the independent testing and benchmarking
project guides are complete. Proceed with scheduler work, then runtime-managed async
suspension; migrate more cases and extend framework features as needed. Test
parallelism/class grouping remain consumers to design with this work, not implemented
contracts. Earlier gates and deferrals below retain their historical dates.


**2026-10-11 sequencing update:** after completing and integrating JIT work, the
author requires the benchmarks before replacing the TaskQueue concept with schedulers, then implementing
runtime-managed suspension instead of requiring Raven-generated state machines.
The earlier “not yet” direction below is historical sequencing, not a prohibition
on that next milestone. Experimental contracts may break; the public scheduler
shape and saved-execution representation remain open. Task completion and I/O
ownership should remain independent of the suspension mechanism. Follow the
[roadmap](platform-roadmap.md) for current priority and compare alternatives before
selecting an implementation.

**2026-09-24 · Internal implementation direction; public API remains exploratory.**

The author asked whether ongoing socket work accounts for runtime-owned async and
whether a scheduler is needed instead of task queues. The recommendation is **yes to
an internal scheduling responsibility now**, without introducing a public Scheduler
class or committing to .NET's scheduler API. TaskQueue remains a compatibility adapter
while current Raven state machines are supported. Runtime suspension remains the target;
we should not make that future change depend on a particular callback representation.

This document refines the socket integration sequence in the
[platform roadmap](platform-roadmap.md). It does not require implementing full runtime
suspension before TCP echo. The initial private native-host completion driver is now
implemented; runtime suspension and a public scheduler are not.

## Scope and contract visibility — author clarification, 2026-09-24

The author explicitly directs us **not to implement runtime suspension yet**. Build
structure and contracts that permit it later, while implementing the application
capabilities we need now. Generated state machines remain the working execution
mechanism. Do not block usable I/O on a saved-frame implementation, a general context
framework or a public scheduler design.

| Contract tier | Examples | Compatibility expectation |
| --- | --- | --- |
| Application-facing | Task/Promise outcomes, await behavior, Result errors, I/O operation results, cancellation and resource ownership | Document and test observable behavior independently of execution machinery. Development APIs can still evolve, with explicit migration notes. |
| Transitional integration | Async builders/awaiters, generated continuation entry points, current explicit TaskQueue APIs | Some are publicly visible in metadata today; that does not make them a permanent application extension model. Keep matching compiler/reference/runtime artifacts and document migration or removal. |
| Private implementation | Scheduler source polling, wake latch, ready-slot representation, native operation registration and frame installation | May change without exposing their types as application contracts; preserve supported observable behavior and validate ownership/cleanup. |

Public visibility and intended stability are separate. Existing public TaskQueue and
compiler protocol members remain documented and functional; we are not silently making
them private. Future frame resumption must replace the integration adapter rather than
force applications to adopt internal scheduler handles. General affinity migration
remains explicit work, not an incidental change to the socket backend.

## Separate responsibilities

| Responsibility | Meaning | Current implementation / next boundary |
| --- | --- | --- |
| Task / Promise | Observable eventual outcome and producer authority | Preserve Task/Result/cancellation semantics while changing execution machinery. A Task is not a thread or necessarily a runnable job. |
| I/O operation | Resource, buffer, backend request, terminal completion and cancellation acknowledgement | Retain native/managed ownership independently of the consumer's continuation representation. |
| Scheduler | Admit runnable work, arbitrate completion sources, decide the next dispatch, park/wake or yield to the host, and determine quiescence | Private invocation-owned component; current VM code implements parts of this implicitly. |
| Resumption target | Where runnable work may execute | Initially the owning invocation. Future UI/affinity or isolate targets need explicit lifetime and capability rules. |
| Suspended execution | Saved execution state and its GC roots | Generated receiver/delegate today; runtime-owned frame/continuation later. Scheduling must not depend on builder or MoveNext names. |
| TaskQueue | Existing callback storage and explicit queue APIs | Transitional adapter, not the definition of all scheduling or all async work. |

Task.Run submits work for concurrent execution through platform capabilities. It is
separate from awaiting an I/O operation or making an already-suspended computation
runnable. A single-thread event loop can support async progress without CPU parallelism.
A worker-backed Task.Run additionally needs transferable captures/results or a supported
shared-memory model. An internal scheduler cannot make arbitrary closures transferable.
Thread remains explicit and platform-dependent; WebAssembly workers are not silently
relabelled as threads. No Task.Run overload or capture policy is selected by this slice.

## What the current implementation proves, and what it does not

The [TCP VM adapter](experiments/socket-completion/VM-INTEGRATION.md) checks real input,
managed-buffer roots, default-queue callback delivery, busy-queue progress, a pending
worker and terminal cleanup. At that original checkpoint, its receive binding and socket poller existed only in test
builds. Production worker waits now return control to the invocation every bounded
wait step. This is useful integration evidence, not a complete scheduler.

Current limitations are architectural, not just missing method names:

- The VM recognizes TaskQueue.Drain and Func<Void> return boundaries by System metadata.
  This compatibility behavior must not become the only place a future frame can resume.
- The initial source-order mismatch is resolved by the shared driver below. Its
  cooperative source rotation does not establish preemption or latency guarantees.
- Normal worker library adapters still have queued joins; the delayed-copy notification
  adapter is experimental. Merely adding a scheduler does not make those joins nonblocking.
- Promise captures its queue. A pending await registers MoveNext with the awaited Task,
  so it currently resumes on the producer's queue even if the result Promise belongs
  to the caller's queue. The [affinity fixture](task-contracts.md#scheduling-and-suspension-exploration--2026-09-23)
  records that mismatch; it is not an intended permanent contract.
- No supported runtime representation of suspended interpreter frames exists yet.

## First internal contract

These are proposed implementation rules and validation gates, not new public types.

1. Each invocation owns its scheduling state and all runnable/parked guest roots.
   Backend threads produce owned completion data and wake notifications; they cannot
   run guest code or mutate guest arrays. Initial dispatch stays on the invocation's
   owning thread. Multiple invocations and isolates remain separate ownership domains.
2. An admitted operation retains its buffer/resource state and a resumption registration.
   The registration identifies its owner and destination, independently of whether its
   runnable body is a generated callback or a future saved frame. Do not introduce a
   fake runtime frame type merely to fill out an interface before suspension exists.
3. Completion becomes runnable exactly once on the owner. Move all needed roots into
   ready ownership before releasing pending ownership; move them into active execution
   before removing ready ownership. Account for pending, ready and active resources,
   including completed results awaiting consumption. Completing I/O cannot bypass quotas.
4. Cancellation requests do not establish backend quiescence. An in-flight native
   operation retains resources until the backend acknowledges it or completion wins.
   Late events cannot resume retired work. Close and invocation shutdown use the same
   ownership rules; a noncooperative backend needs a documented shutdown strategy.
5. Source arbitration has one policy at idle and dispatch boundaries. Start by polling
   sources in rotating order, admitting a bounded amount of completion work per turn,
   and executing a bounded batch of ready work before checking completions again.
   This is cooperative fairness, not a latency promise or preemption. Existing callbacks
   must return; an instruction budget/cancellation check still bounds guest execution.
6. An empty ready queue is not completion while live operations can make progress.
   Wake notifications must be durable across the check-for-work / park boundary.
   Native blocking hosts may park; browser/embedded hosts must be able to return a
   pending execution to their event loop. The current 1/10 ms waits are transitional,
   not a portable scheduler contract. Do not use a blocking wait on the browser thread.
7. Continuation destination belongs to the awaiting execution, independently of the
   producer's location. Adopt the owning invocation as the initial supported target.
   Existing explicit TaskQueue behavior must receive a deliberate migration test and
   documentation update; do not silently change it in a socket backend. Whether public
   queue affinity survives, changes or is deprecated remained open at this proposal;
   the 2026-10-11 clarification below now directs migration away from visible queues.
8. Already-completed awaits may continue inline under the current language behavior.
   External pending completion must enqueue resumption rather than invoke guest code
   from a producer. Specify reentrancy separately from context propagation. Logical
   context, UI affinity and custom scheduler selection are distinct future decisions.

The names above describe responsibilities; they do not prescribe one public class per
row, an ambient scheduler, a thread pool, a work-stealing algorithm or a new namespace.

## Runtime suspension migration

Scheduling chooses runnable work; suspension determines what state is saved and how it
can safely resume. The interpreter should eventually report continue, suspend, return
or Fault to its execution driver. The suspended-state owner must preserve the needed
frames, PC, arguments, locals, evaluation state and logical context, with exact GC roots.
Byrefs, stack storage, cleanup and native-call boundaries require independent verifier
and lifetime rules before they can cross suspension. A callback ticket is not proof that
arbitrary interpreter frames can already be suspended.

The current adapter can turn an admitted runnable continuation into TaskQueue.Post/Drain
frames. A later adapter can enter a retained runtime frame. The I/O producer should
publish the same operation outcome in either case. Test the two adapters against the
same ordering, cancellation and root-transfer requirements when both actually exist.
Keep compiler-generated state machines and builders until that replacement is validated;
they can then be deprecated/removed deliberately. Public Task/Result behavior should not
inherit their storage or builder protocols.

## .NET comparison and tradeoffs

Primary sources reviewed 2026-09-24:

- [.NET 10 TaskScheduler](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.taskscheduler?view=net-10.0)
  provides task scheduling and permits custom implementations. We need its separation
  of policy from asynchronous results; we have not selected its Task-based public
  work-item/customization surface.
- [.NET 10 SynchronizationContext](https://learn.microsoft.com/en-us/dotnet/api/system.threading.synchronizationcontext?view=net-10.0)
  and the [ConfigureAwait FAQ](https://devblogs.microsoft.com/dotnet/configureawait-faq/)
  distinguish continuation placement from Task scheduling and logical context flow.
  An explicit initial owner avoids copying the whole ambient-capture surface now,
  but does not solve UI affinity or supply equivalent compatibility automatically.
- The [.NET runtime-async specification draft](https://github.com/dotnet/runtime/blob/main/docs/design/specs/runtime-async.md)
  describes runtime-supported suspension and constraints on preserved execution state.
  It is explicitly a design draft, not evidence here of a released .NET feature or a
  specification neoCLR has adopted. Its relevant lesson is that replacing compiler
  state machines still requires ownership and resumption rules; it does not remove
  scheduling responsibilities.

Keeping only TaskQueue is cheapest immediately, but extends dependencies on callback
and library names and leaves wakeup/affinity implicit. Copying .NET TaskScheduler plus
SynchronizationContext gives familiar extension points but commits a broad public
model before portable host requirements are known. A private execution scheduler with
an existing-queue adapter adds internal lifecycle machinery now while leaving that
public choice open. Runtime suspension first would avoid some transitional machinery,
but delays the application POC and still requires the same I/O ownership work.

The private scheduler is the recommended next step. Its cost is a temporary adapter and
new state/lifecycle tests; no performance or allocation improvement is claimed. Existing
GC and socket experiments are reusable evidence, not sufficient proof of the full model.

## Next implementation slices

1. **Initial native-host extraction complete:** one private source-arbitration and
   waiting boundary now serves idle and callback dispatch. TaskQueue remains the
   adapter. Focused rotation/wakeup checks and real guest TCP/worker progress cases
   pass. OS socket readiness and nonblocking host yielding remain backend follow-ups;
   there is no public scheduler API.
2. Make continuation destination and operation ownership explicit at that boundary.
   Cover both GC paths, pending-to-ready-to-active root transfer, bounded completed
   outcomes, cancellation/close orderings and explicit-queue migration behavior. Use
   real I/O and actual guest callbacks, not only synthetic scheduler tokens.
3. Attach reusable sockets and Task<Result<...>> producers through that boundary,
   refresh public reference metadata/docs, and run Raven TCP echo. Record how a future
   runtime-suspension adapter will replace the callback side without changing the
   transport producer. Full frame suspension is a later independently validated slice.

## Initial native-host driver — implemented 2026-09-24

[src/scheduler.rs](../src/scheduler.rs) now owns the worker registry, completion-source
arbitration and idle waiting for each invocation. The VM delegates root collection at
both GC paths and polls the same driver at queue quiescence and the existing safe
callback-return boundary. One completion is admitted per boundary. Source order rotates
after each successful delivery; an idle source cannot hold up a ready peer. Source lists
use stack storage rather than allocating a list on every poll.

Worker result publication or sender disconnection signals a shared wake latch after
the channel sender is dropped. Outcomes remain in their channels; the latch is only
a coalesced hint. A signal arriving between an empty poll and parking remains set,
and parking checks that state under the same mutex used by signal publication. The
condition variable may wake spuriously, so progress is always polled again. This is
a native-host implementation choice, not a browser parking contract.

The driver retains a 10 ms timeout because host cancellation has no wake subscription
and the test-only TCP adapter has no OS readiness notification. Thus neither native
socket readiness registration nor fully event-driven cancellation is claimed. The
worker registry's old wait loop remains only as a unit-test helper; production waiting
now belongs to the scheduler. Private worker submission/join access still uses its
existing registry; no generic public work-submission API has been added.

Root transfer into actual TaskQueue.Post frames and existing callback semantics remain
unchanged. Runnable work is still represented by a callback Value. No synthetic saved
frame variant is introduced before runtime suspension can implement it. The new shared
wake state adds a per-invocation allocation and synchronization cost; no speedup claim
is made. Pending/ready quotas, await-site affinity, nonblocking host yielding and real
suspended-frame ownership remain follow-up work.

Validation commands:

```sh
cargo test --lib scheduler::tests
cargo test --lib socket_vm_probe
cargo test --lib workers::tests
cargo test --test workers
```

Four focused scheduler cases check rotation, completion between poll and park, coalesced
signals retaining multiple outcomes, and a cross-thread wake. The real-TCP VM fixture
checks both collection paths, busy queues, progress alongside a pending worker and
terminal teardown. Existing worker tests cover output budgets, cancellation, default
versus explicit queues and callback/root ownership. A separate worker reply test
checks that publication/disconnection is observable before the durable wake. The next implementation slice is
explicit operation/resumption ownership and its affinity migration gates; the current
producer-queue affinity has not changed.

Checked locally on Darwin arm64: 4 scheduler cases, 5 real-TCP VM cases,
11 worker-registry cases (including the reply-order check), and 16 worker integration
cases pass. The combined website build validates 523 pages.

## Ready-to-active ownership — implemented 2026-09-24

The driver now has a single traced ready slot holding a callback and its explicit
legacy default-queue destination. Polling stops consuming source completions while
the slot is occupied. The VM installs the Post frame before acknowledging release
of that slot; a failed installation retains both root graphs. This makes the handoff
an explicit private contract instead of returning an unowned callback Value.

Pending sources still retain their callbacks/buffers until polling completes. The
source-to-ready transfer has no GC safepoint; both GC paths trace the ready slot
thereafter. Installation must not publish on failure and must publish to a traced
active owner before returning success. The current VM closure resolves/creates its
frame before pushing it, and contains no intervening collection. The adapter is
single-owner and admits at most one ready registration; additional source outcomes
remain with their existing owners and limits.

This is not a new suspended-frame representation or an await-site affinity change.
The destination is the existing invocation default queue at completion admission,
not newly captured from the await site. Source-operation quotas and completed results
awaiting consumption still need their own lifecycle accounting; a one-slot handoff does
not claim to bound all retained application graphs or the existing TaskQueue backlog.

Three new collector-backed/ownership checks cover independent callback/destination
roots across failed installation and successful active transfer, immutable destination
while occupied, exactly-once acknowledgement, and cancellation followed by teardown.
The existing real TCP and worker VM checks validate the actual Post-frame path.
Continue with the smallest reusable operation/socket bridge using these contracts;
full runtime suspension and broader execution-context policies remain later work.

Validation for this handoff slice: 7 scheduler tests, 5 real-TCP VM tests and
16 worker integration tests pass on Darwin arm64. API-reference checks and the
523-page combined website build also pass.

## Reusable operation source — 2026-09-24

The [private receive backend](socket-api-design.md#reusable-private-receive-backend--2026-09-24)
now separates socket ownership, pending buffer/callback roots, scheduler-ready
registrations and completed results awaiting consumption. Completed outcomes retain
operation admission slots until taken. The source is normal runtime code; only host
stream injection remains test-only. Public operation IDs, Task producers and error
mapping are not exposed yet. This uses generated callbacks and the existing default
queue adapter; no runtime suspension or new affinity behavior is introduced.


The subsequent [public socket client](socket-api-design.md#first-public-tcp-client--2026-09-24)
uses the same ready slot and default queue adapter. Connect and receive callbacks
complete library-owned Promises with Result values; no compiler builder protocol is
added. The private registry also owns pending native connects and retains their
callbacks as roots. Application methods expose neither registry IDs nor callbacks.

## Cross-runtime reassessment — author direction, 2026-10-08

While pursuing the native HttpServer POC, the author asks us to re-evaluate runtime
approaches, especially Function types and delegate-like objects, in anticipation of
runtime async suspension and green threads. This expands the design review scope;
it does not select a stack model, promise green threads in the POC, or reinstate
Delegate as a public type. Preserve the prior structural Function direction as the
current contract while reviewing its implementation and any future semantic changes.

The present interpreter FunctionObject keeps a checked target, receiver and separate
object identity. Structural equality uses signature/target/receiver identity. Native
AOT uses a private heap descriptor with an image-local selected-function number and
receiver pointer. Those are different representations of a callable contract, neither
of which should define suspended execution. Native host strong handles currently live
in thread-local storage; native published frames refer to live C stack slots. Both
need explicit reconsideration before execution can suspend or migrate between carriers.

### Boundaries to preserve in the POC

- **Callable contract:** retain signature checking, receiver/capture lifetime and defined
  equality independently of entry addresses and scheduler IDs. A callable may be invoked
  repeatedly; a particular suspended activation must resume at most once per wakeup.
- **Pending operation:** own I/O resources and retained buffers until terminal completion
  or cancellation acknowledgement. Completion makes an opaque work item runnable; its
  adapter may invoke today's callback or resume a future runtime activation.
- **Execution ownership:** associate roots, faults and logical context with an invocation
  or future execution unit. Current thread-local registries and quiescent host dispatch
  are bounded native adapters, not a requirement for all future schedulers.
- **Suspension:** never retain addresses into expired native frames. Interpreter frames,
  compiler-generated state machines and native saved stacks need different root maps and
  resume machinery behind the same outcome/cancellation contracts. Native foreign calls
  require an explicit blocking/pinning policy before carrier scheduling can work.
- **Diagnostics and reload:** preserve fault code/message and define logical stacks across
  suspension; raw callback IDs are private to a loaded image. Reload/version retention,
  active-frame lifetime, introspection and unwind behavior require their own tests.

### Alternatives and primary comparisons

Reviewed 2026-10-08: [.NET Delegate](https://learn.microsoft.com/en-us/dotnet/api/system.delegate?view=net-10.0)
binds methods and receivers; [.NET IAsyncStateMachine](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.compilerservices.iasyncstatemachine?view=net-10.0)
provides a compiler-generated async resumption protocol. These are useful separate
layers, not a reason to make a Function object itself a suspended stack.
[OpenJDK JEP 444](https://openjdk.org/jeps/444) provides the Java 21 virtual-thread
comparison, including GC-managed stack chunks. Its implementation is not a specification
for neoCLR or a claim about all subsequent JDK versions.

Retaining generated state machines costs least now and already supports the Task API,
but ties lowering and some diagnostics to compiler protocols. Runtime-owned stackless
activations could centralize ownership/rooting while still requiring explicit suspend
points and compiler/backend cooperation. Stackful green threads could suspend ordinary
call chains, at the cost of native stack capture, unwind metadata, GC maps, foreign-call
constraints and scheduling machinery. Neither option automatically supplies CPU
parallelism, fairness, thread-safe captures or a moving GC. No implementation winner or
performance advantage is selected by this review.

Use the HTTP app to build evidence: completion-before-registration, cancellation races,
exactly-once delivery, abandoned operations/shutdown, callbacks retaining captures through
GC, nested awaits, fault stack continuity and progress under a busy queue. Compare the
same workload in interpreter and AOT. Later add suspension memory per operation, root
scan cost, allocation rate and scheduler latency measurements; benchmark green-thread
alternatives only once implementations exist. Review these boundaries at each scheduler,
Function, GC, metadata/ABI or async-lowering change, without blocking the current POC on
a general suspension implementation.

The first AOT [TaskQueue scope adapter](experiments/aot-console/task-queue.md) now
preserves existing default and active explicit-queue behavior in a Raven consumer.
It uses a private published-frame convention and does not implement host queue pumping
or runtime suspension. Its passing affinity test preserves current behavior, not a
permanent choice of producer-queue affinity or scheduler API.

### Scheduler clarification — author follow-up, 2026-10-08

The author reiterates that TaskQueue may be replaced by a Scheduler, with the runtime
eventually scheduling green threads. Treat that as the direction to investigate,
not a commitment to preserve the current library queue as the scheduler abstraction.
The native host Drain entry is a compatibility adapter for today's library implementation.
It must not become a public scheduler ABI or encode TaskQueue fields into native hosts.

The proposed runtime Scheduler owns runnable/suspended activation state, admission,
wakeup/cancellation transitions and activation roots. A Function describes callable
work; invoking one can start work but does not make that object a green thread. I/O
completion should signal readiness through an adapter, leaving the scheduler to choose
when an activation runs. Task/Promise completion and application queue-affinity policy
remain distinct questions. TaskQueue could later become a facade, a serial executor,
or a deprecated API; that migration is still open. The native pthread stack probe and
TLS roots must be replaced or adapted for owned green-thread stacks/activations.

No stackful/stackless model, public Scheduler API, preemption or carrier-migration
policy is selected by this clarification. Continue the HTTP POC using current APIs,
then use workload evidence to evaluate scheduling and suspension implementations.

### Cross-cutting runtime services — author clarification, 2026-10-08

The author explicitly asks that native-compilation work benefit interpreted execution
and eventually JIT, summarizing the intended architecture as “cross-cutting runtime
services”. Use that as the design direction: shared service contracts for scheduling,
activation lifecycle, I/O readiness/cancellation, root ownership and logical fault
reporting, with execution-specific adapters. It does not require identical storage or
machine code across backends. Interpreter frames, native stack guards and future JIT
stack maps are backend mechanisms; their observable lifecycle and fault contracts
must remain compatible.

For each slice, identify shared invariants and reusable cross-mode consumer tests,
then identify the backend mechanism and its replacement boundary. Port a fix or
optimization to the interpreter when it applies and validate it there; do not claim
that a native-only adapter automatically improves interpreter performance. The current
Raven queue success/fault consumers exercise the same artifact in both modes. They
can later qualify a Scheduler adapter, while host Drain exports remain private.

### Co-evolution with native foundations — 2026-10-09

During the project-level native compilation/build-kit work, the author asks to
consider runtime suspension, runtime scheduling and green threads once the native
foundations are ready, and to let them co-evolve. This reinforces the shared-service
direction above. It does not require finishing every AOT feature or Windows port
before beginning suspension work, nor does it add suspension to the synchronous
console kit's qualification scope.

Continue the bounded native build and HTTP consumers while reviewing activation
ownership whenever adding host callbacks, GC roots, stack guards or scheduling.
The initial console host's one-entry heap lifetime and the current HTTP host's
queue pumping are explicit adapters, not permanent runtime scheduling contracts.
Do not make pthread identity/TLS or an uninterrupted native call stack the abstract
owner of suspended work; backend adapters must identify what would change when
activations own their state and roots.

An initial suspension experiment should exercise one activation suspending and
resuming on readiness, a captured value surviving collection, and cancellation or
shutdown releasing it exactly once. Reuse cross-mode lifecycle/fault consumers and
the .NET/JVM comparisons above. Assess stackless activation records and stackful
green threads against the same scenario before selecting storage, stack scanning,
foreign-call restrictions, carrier migration or preemption. Compare wakeup latency,
retained memory and rooting costs when executable alternatives exist; no performance
claim follows from the direction itself. Suspended activations and callbacks must
also retain the appropriate code generation for later reload.

These are proposed experiment boundaries, not implemented suspension or an approved
public Scheduler API. The next native HTTP/hosting slices should report their
replacement boundaries and any evidence that changes these choices.

The subsequent [HTTP project profile](native-poc-bundle.md#opt-in-http-project-profile-development)
packages the existing host unchanged. Its request and callback-fault checks validate
today's scope cleanup and diagnostic behavior, not suspended activations. Queue
draining/readiness dispatch, pthread stack guards and thread-local roots remain
the explicit replacement boundaries for the co-evolving scheduler work.

The [Windows native host prerequisites](windows-native-host.md) separate the heap
owner from OS thread identity. The private allocation probe has no TLS state, but
release still requires all activations, callbacks and roots to be gone; the raw
allocator does not enforce that precondition. The subsequent Windows collector
host now rejects teardown with registered host roots or published frames, and
from another thread. Its executable consumer verifies retention across root
handoff and reclamation after release. This establishes the current synchronous
lifecycle, not activation-owned roots: suspension still needs explicit ownership
that survives host returns before migration or deferred cleanup is allowed.

The Windows host stack guard now refreshes system bounds per call and explicitly
rejects fibers, including at collector-host admission. Small-stack and deep-call
consumers pass with GC-frame cleanup. This is a normal OS-thread contract; future
activation-owned stacks need their own bounds/root rules rather than bypassing
the rejection or retaining stale system-thread bounds.

The integer-only Windows generated-stack gate now passes actual generated recursion,
status-based fault return and host reuse with every published frame removed. This
qualifies the existing synchronous normal-thread path only. Inline page probes and
fresh system bounds do not supply suspendable frames, activation-owned roots or
native unwind/SEH metadata; those remain prerequisites for any stackful scheduling
experiment. See [execution evidence](windows-generated-stack-validation.json).

The Windows synchronous console project path now also runs managed array/text and
binary-I/O consumers as standalone EXEs, with fault diagnostics and checked cleanup.
It uses the same normal-thread host and status-return contracts; it does not turn
thread-local roots into activation ownership or qualify stack migration. Keep those
boundaries explicit when the next persistent-host/suspension experiment is selected.


## Scheduler-owned waiting — 2026-10-11

The first resumed implementation slice moves the interpreter invocation's fixed
parking timeout behind the private scheduler. `Scheduler::wait_timeout` returns a
non-consuming host wait hint: zero for ready work or an expired timer, otherwise
the smaller of the earliest timer deadline and the existing 10 ms polling fallback.
Cancellation/socket readiness still require that fallback; this is not an event-only
scheduler or an OS timer precision guarantee.

A contending context that cannot own dispatch keeps the fallback wait even when
another context has ready/expired work. It must not busy-spin on work it cannot
execute. Reading the hint does not claim dispatch ownership, consume timers, invoke
callbacks or release roots. The invocation drops its dispatch lock before parking;
VM managed graph access has already been released. The existing durable wake latch
continues to cover a notification between the check and park.

This reuses the .NET comparison above: scheduling policy, completion outcomes and
execution representation are distinct responsibilities. A deadline hint is an
internal host-loop decision, not a new application TaskScheduler/SynchronizationContext
contract. Its cost is a brief dispatch lock and a scan of the bounded timer registry
(currently at most 64 entries) before a VM wait. No throughput improvement is claimed.
It avoids deliberately choosing a timeout beyond an already-known nearer deadline;
OS dispatch can still run later. Concurrently admitted sources still rely on their
wake behavior or the polling fallback.

Scope: the interpreter's instruction driver and shared invocation contexts use this
policy. Existing JIT/AOT native host loops are separate adapters and are not changed
or claimed to inherit it. Their alignment remains part of the scheduler migration.
Existing private instruction-boundary VM suspension tests provide useful root and
ownership evidence; they do not implement compiler-independent async/await suspension.

Next: unify the completion/dispatch ownership contract across execution adapters,
resolve await-site versus producer destination, then design the saved-activation
boundary. Keep TaskQueue as a documented transitional adapter until consumers and
backend validation can move together; do not rename it and imply scheduler migration
is finished. Runtime suspension follows that contract work.


Focused validation on macOS ARM64: 11 scheduler cases, one dispatch-ownership
policy case, one real VM contention/cancellation case and three guest timer cases
pass. The policy checks use fixed instants rather than wall-clock latency thresholds;
timer integration checks delivery, return-frame preservation and cancellation.

```sh
cargo test --lib scheduler::tests
cargo test --lib invocation::tests::dispatch_wait
cargo test --lib vm::suspension_tests::contending_dispatch_context
cargo test --test timers
```

An isolated `CARGO_TARGET_DIR=target/scheduler-wait-validation` avoided the editor's
concurrent full-check lock. These checks do not claim native-host parity or a timing
benchmark. No public API reference snapshot changes are required for this private
interpreter implementation.


## Native completion-state boundary — 2026-10-11

Native JIT, HTTP AOT and retained-session hosts now call
`neoclr_scheduler_step_v1(context, callback, boundary)` instead of inspecting the
private operation table. This mirrors the interpreter's ready/pending/idle distinction
without requiring identical native/VM representations. See the
[consumer qualification](experiments/native-scheduler/README.md).

The step polls for one completion and returns ready (1), pending (2), idle
(0), or host failure (-3). It neither blocks nor invokes guest code. A supplied
boundary must be the exact published top frame for this context; NULL requires
quiescence. It retains current single-owner/thread and callback delivery contracts.
Delivered socket results remain pending until consumed. Advancing after a final
timer callback releases that delivery root before returning idle; previous host-side
empty-table shortcuts deferred this release until scope teardown.

The .NET TaskScheduler/SynchronizationContext comparison above remains the baseline:
separate scheduling decisions from asynchronous outcomes and continuation location.
This private entry point is not a public TaskScheduler implementation. Keeping three
host copies was cheaper initially but exposed operation storage and duplicated
quiescence policy. Moving the decision behind one C function preserves existing
callback semantics, adds no managed allocations, and centralizes future policy changes.
The source scan is still bounded to 64 operations; no speedup is claimed.

The host must consume/install a returned callback before stepping again. Roots remain
source-owned under that existing protocol; this slice does not add the interpreter's
explicit ready-to-active acknowledgment. Await-site destination, general suspended
activations, nonblocking OS readiness and native wait/deadline policy are still open.
Native hosts retain their 1 ms polling pause; interpreter waiting has its separate
10 ms fallback/deadline hint. Shared completion meanings do not imply identical timing,
parallel execution or a finished TaskQueue migration.

Six native C contracts pass with undefined-behavior/bounds instrumentation. Timer
success and callback-fault consumers pass across interpreter/AOT/JIT, the HTTP AOT
host passes ten timer scenarios, and the retained native HTTP consumer passes normal,
fragmented and terminal-fault cases. Qualification is macOS ARM64; the portable C
contract is included in the existing Windows workflow but has not been rerun there
for this revision. No compiler bridge or public API signature changes are involved.


## Runtime-owned asynchrony and language projections — 2026-10-11

**Author direction:** the runtime itself should understand asynchrony; native
languages and other runtimes should eventually be able to integrate and project
these fundamental concepts. This clarifies the scheduler-then-suspension direction,
without selecting a public API or foreign-runtime ABI.

**Subsequent author clarification:** managed bytecode languages are the primary
consumers, supported by interpreter and JIT execution; native AOT is the publishing
target. The common language runtime offers opt-in services, with future integration
points as outlined in the [architecture proposal](proposals/runtime-architecture.md).
A synchronous consumer need not acquire scheduling services merely to execute.
Consumers that opt into asynchrony must satisfy its ownership and root contracts.
This does not turn every proposed foreign-runtime projection into current scope.

The proposed semantic boundary has three layers:

- The runtime owns registration, completion publication, runnable-work ownership,
  suspension/resumption and the lifetime of retained execution state. Completion,
  scheduling destination and cancellation acknowledgment remain distinct concerns.
- Language implementations project their async constructs onto that lifecycle.
  Raven is one consumer. Task/Promise and compiler-generated callbacks are current
  projections; the underlying contract must not require Raven-specific method names,
  TaskQueue or a particular state-machine layout. Interpreter, JIT and AOT may retain
  different execution representations while enforcing the same lifecycle rules.
- Native hosts and other runtimes eventually bridge completion and resumption through
  explicit ownership and wakeup contracts. A foreign completion thread should publish
  readiness for the owning runtime to dispatch; it must not implicitly acquire guest
  execution rights or mutate the managed graph. Cross-runtime integration also needs
  explicit payload/error translation, cancellation and teardown acknowledgment.

This proposal reuses the [.NET comparison](#net-comparison-and-tradeoffs) above:
Task outcomes, scheduling and continuation placement remain separate. Runtime-owned
suspension changes the execution mechanism, not that separation. The cited runtime
async draft is comparative design evidence, not a claim that this direction is unique
to neoCLR or absent from .NET. Retaining callbacks alone would minimize immediate
implementation cost; making their lifecycle an adapter to a shared runtime contract
supports additional language consumers, at the cost of backend contracts, retained
roots/code lifetimes and interop validation. No performance benefit is established.

Opaque owned operation/resumption handles are a candidate for native integration,
not an adopted ABI. Stackful versus stackless suspension, green-thread scheduling,
metadata/instruction encoding, ambient context flow and foreign exception mapping
remain open. Not every library awaitable must become a built-in runtime type. Async
interoperability will require adapters; it does not imply automatic compatibility
with another runtime's Task, future or event loop.

The next bounded implementation gate remains explicit native ready-to-active
ownership acknowledgment. Design that transfer independently of whether the runnable
body is today's callback or a future retained activation. Test duplicate/stale
consumption, cancellation races, roots through dispatch and teardown before selecting
broader suspension machinery. A later non-Raven consumer should validate registration,
completion and resumption integration without depending on Raven-generated layouts.
These are validation requirements, not completed capabilities. Current native hosts
still use synchronous published-frame boundaries and source-owned callback roots;
existing instruction-step VM suspension does not implement runtime async/await.


## Owned native dispatch — 2026-10-11

The outer HTTP AOT and retained-session hosts now acquire a private dispatch ticket
and explicitly complete it after invoking the callback, on success or guest fault.
The ticket owns a separate strong root. Polling cannot advance while it is outstanding;
completion validates the owning context, ticket and original published boundary.
Operation outcomes and dispatch lifetimes can end in either order. Scope teardown
abandons both safely. See [contracts and HTTP evidence](experiments/native-scheduler/README.md#owned-dispatch-slice--2026-10-11).

This follows the existing .NET comparison's separation of outcome ownership and
scheduled execution. Keeping only source-owned roots avoids another registry slot
but couples dispatch lifetime to result consumption. The owned ticket costs one
bounded host-root slot, not a managed allocation; allocation/registry exhaustion is a
terminal host error with scope cleanup, not retryable delivery. No speedup is claimed.
The mechanism is private and callback-based; it does not select a future activation
representation or native interop ABI. Cancellation of a delivered source does not
revoke dispatch ownership; consumers still observe the operation's existing outcome.

The generated entry pump and JIT entry adapter retain their previous poll protocol.
Next migrate that entry lifecycle with explicit completion on normal and fault exits,
then resolve resumption destination and wait policy. This bounded outer-host slice
must not be described as completed acknowledgment parity across execution modes.


## Entry adapter acknowledgment — 2026-10-11

The JIT and HTTP AOT entry pumps now acquire owned deliveries, with completion routed
through an optional hook on the private task scope. The next valid entry-boundary
poll acknowledges the preceding callback, and entry end acknowledges the last one
on success or fault. Single ticket reads prevent duplicate entry dispatch; polling
before reading rejects skipped dispatch. Failure of the completion hook is sticky
and reported during scope teardown without replacing the guest diagnostic.

This extends the owned-dispatch lifetime contract above without compiler changes:
existing generated poll/read/end calls already express the required lifecycle. The
hook avoids coupling the task scope to socket storage and keeps poll-only adapters
usable. Its cost is an indirect completion call and retaining one ticket until the
next poll/end; no latency or throughput improvement is claimed. Source teardown owns
abandoned-ticket cleanup when acknowledgment fails. The runtime still executes Raven
state-machine callbacks, not retained async activations. See
[entry evidence](experiments/native-scheduler/README.md#owned-entry-delivery--2026-10-11).

Next settle continuation destination and shared wait policy. The interpreter and
native adapters have distinct representations; these checks establish native entry
ownership behavior, not identical scheduling or full suspension parity.


## Queue-independent scheduling — 2026-10-11

**Author:** “We shouldn’t base our scheduler architecture around a visible task
queue” and “Task queue is an implementation detail”. This settles the architectural
visibility direction; it does not claim today's public TaskQueue has been removed.

Applications should express asynchronous operations, await completion and observe
outcomes/cancellation. The runtime owns runnable work, continuation registration,
dispatch, progress and quiescence. It may implement these with queues, multiple queues,
work stealing or another mechanism. Queue contents, identity and manual draining must
not become the necessary application or language integration contract. Opting into
runtime scheduling means opting into its execution/lifetime guarantees, not owning a
queue. Future host/custom scheduling controls may describe policy or affinity if a
consumer requires them; their public shape is still unselected.

This reuses the .NET TaskScheduler/SynchronizationContext comparison above: scheduling
policy and continuation placement are meaningful contracts independently of storage.
A queue-shaped public abstraction is cheap to expose but couples applications to pump
and storage decisions. Hiding it permits backend evolution and reduces application
bookkeeping, while moving responsibility for progress, observability, fairness and
host integration into the runtime. These costs must be tested; hiding the queue alone
is neither a fairness nor a performance improvement.

The inspected dependencies define the next migration, rather than a cosmetic rename:

- `runtime/raven/src/System/Tasks/Tasks.rvn` captures `TaskQueue.Current` in Promise,
  exposes explicit queue construction, posts completion callbacks to that producer
  queue and forwards Task.Dispatcher to the producer. Move resumption registration
  toward the awaiting execution's runtime-owned destination, separately from outcome
  storage. Preserve explicit callback observation as an adapter with defined placement.
- `src/scheduler.rs` currently stages ready callbacks with a managed destination;
  `src/vm.rs` recognizes TaskQueue dispatch boundaries. Separate the runtime's execution
  destination from this managed queue adapter before removing its public surface.
- Native task scopes and the new owned-delivery hooks already separate dispatch-root
  lifetime from outcomes. Keep them private; they are not a public Scheduler API or
  proof that the queue dependency has been removed.

First qualify registration/routing with completed and pending awaits, different producer
and awaiting destinations, nested callbacks, cancellation and fault cleanup. Then migrate
compiler/runtime consumers and remove or internalize public queue APIs with matching
reference assemblies, API documentation and samples. Experimental compatibility does
not require preserving queue APIs, but changing source visibility alone would break
existing generated consumers without supplying runtime-owned progress. The destination
model must also admit later saved activations without depending on TaskQueue or Raven
state-machine layout. Cross-mode wait policy and runtime suspension follow this boundary.


## Invocation-owned completion admission — 2026-10-11

The interpreter scheduler can now admit and retain a ready source callback without
a managed TaskQueue destination. Invocation-owned work has no queue binding in its
ready record. The existing VM dispatch adapter resolves the invocation's default
queue only while installing the callback into a traced active frame. Explicit legacy
queue-bound work remains rooted and retains its selected destination; polling through
another adapter does not rewrite an occupied ready record.

Previously completion admission polled the source and then required a default queue
before storing its callback. An unavailable adapter could therefore reject already
consumed source readiness before the scheduler retained it. Now installation failure
leaves the callback rooted for retry or invocation teardown. A focused GC test proves
queue-free admission, failed installation retention, later transfer to an active
owner and no duplicate delivery. Shared-invocation ownership and guest timer checks
preserve existing dispatch behavior.

This is the first boundary in the queue-independent migration, not a new await-site
routing contract. Promise still captures its producer queue, the current VM callback
adapter still requires a default queue, and source registration may still require that
adapter. Native hosts are unchanged. The existing .NET comparison's separation of
completion and scheduling motivates this split: ready work need not own queue storage.
The cost is an optional legacy destination and adapter-side resolution; no performance
improvement is claimed. Public queue removal and runtime-owned continuation registration
remain next work; completed/pending await semantics have not changed in this slice.


Validation on macOS ARM64: `cargo test --lib scheduler::tests` (12),
`cargo test --lib invocation::dispatch_tests` (2), and `cargo test --test timers` (3),
using `CARGO_TARGET_DIR=target/scheduler-wait-validation`. These checks exercise the
changed interpreter ownership boundary; unchanged JIT/AOT adapters were not rerun.


## Managed Scheduler contract, runtime implementation — 2026-10-11

**Author clarification:** analogous to .NET exposing TaskScheduler, neoCLR should
expose Scheduler while its actual implementation resides in the runtime. Task remains
in the managed domain. This resolves the earlier open question of whether there is a
public scheduler abstraction; it does not select a copy of all .NET TaskScheduler members.

The intended layering is:

- **Managed Task/Promise:** outcomes, producer completion and consumer awaitability.
  They use the scheduling contract rather than owning dispatch policy or queue pumps.
- **Managed Scheduler:** the application/language-facing scheduling abstraction,
  projecting runtime services. Exact submission, selection, affinity and hosting
  members, as well as whether/how custom schedulers are supported, remain to be designed.
- **Runtime scheduling implementation:** runnable registrations, admission, dispatch,
  wake/park, lifetime/root ownership and eventual activation suspension/resumption.
  Interpreter, JIT and AOT adapters satisfy this contract using suitable representations.
- **Private storage:** queues and other scheduling data structures. Applications do
  not need a queue or manual Drain operation to participate in ordinary async execution.

The existing .NET comparison supplies the API baseline: separate awaitable outcomes
from scheduling and continuation placement. The author-selected name is Scheduler;
this document does not infer .NET custom TaskScheduler subclassing or inlining policy
as already supported. A managed facade preserves language-facing ergonomics while
runtime implementation centralizes execution ownership. It costs an explicit managed/
runtime service boundary and requires consistent cross-mode lifetime and error behavior.
No performance benefit or public API implementation is claimed by this clarification.

Next define the smallest runtime-backed Scheduler surface needed by managed task
continuations and hosting, then migrate TaskQueue consumers with matching runtime,
compiler bridge and reference documentation. Task remains managed; runtime-owned
suspension is not a direction to move the entire Task API into native code.


### Runtime ABI and projections — author clarification, 2026-10-11

These are different abstraction levels. The runtime exposes its own ABI hooks for
async lifecycle and scheduling; the managed environment projects those capabilities
through Task/Promise and Scheduler. Native languages may later provide native
projections without constructing managed Task objects merely to use runtime async
services. Task layout, managed builder types and queue identity must not define the
underlying ABI. Native projections still obey the selected services' ownership,
completion, cancellation and execution-entry contracts.

This selects the architectural boundary, not a stable ABI signature or universal
future representation. The current private host functions are experimental evidence,
not that final contract. Define and validate the minimum runtime hook contract together
with its first managed projection; do not start from managed Task layout and expose
it as the native ABI. Other language projections remain future consumers. Relative
to the .NET API baseline above, managed familiarity is preserved at the projection
layer; a reusable runtime ABI adds versioning, capability and lifetime validation
costs that must be addressed independently of managed API shape.


## First runtime submission hook — 2026-10-11

The private native scheduler now admits a rooted callable with
`neoclr_scheduler_submit_v1(context, callback_root, boundary)`, independently of
Task/Promise or managed queue construction. It retains its own root, defers execution,
and delivers through acquire/complete. The retained managed Raven HTTP callback is
its first consumer. [Contract, limits and evidence](experiments/native-scheduler/README.md#runtime-submission-prototype--2026-10-11)
distinguish admission from execution and input ownership from dispatch ownership.

Relative to the .NET scheduling comparison, this models runnable admission beneath a
managed API rather than taking a managed Task as the native submission payload. A
Task/Promise projection may still track completion entirely in managed code. Direct
host invocation is simpler, but bypasses deferred admission and common dispatch
ownership. The hook costs a retained root and a bounded admission slot; it does not
promise throughput gains, thread-pool semantics or FIFO order.

Scope remains narrow: owner-thread native hosts, an image-local managed callable
handle and current published-frame rules. The 64-slot table is shared with I/O/timers
inside the experimental socket scope. Extracting a runtime service scope, cross-thread
publication, cancellation handles, ABI version/capability negotiation and native-language
callable projections remain open. The managed Scheduler API and interpreter-facing
submission mapping are not implemented by this native prototype. Existing ScheduleTask
is a distinct interpreter concurrent-work service and is not silently aliased here.


## Managed projection with existing state machines — 2026-10-11

The author reiterates that scheduling must lead toward runtime async suspension/resume
while keeping compiler-generated state machines valid. The
[source-included Scheduler experiment](experiments/scheduler-projection/README.md)
now maps managed submission to a reserved runtime service and passes pending/completed
await and terminal-fault cases in interpreter, AOT and JIT. Task/Promise remain managed;
Raven lowering is unchanged. No public Scheduler members are added by this experiment.

The interpreter runs submitted callables in traced frames without a managed queue;
the native adapter projects the same admission intent into owned dispatch tickets.
The callback represents today's executable continuation, not the only future runnable
representation. Runtime-retained activations can later implement the dispatch boundary
without invalidating callback adapters or moving managed Task layout into the ABI.

This follows the existing .NET comparison's separation of outcomes and scheduling.
The experiment adds a private internal-call/native binding and retained runnable storage;
it preserves familiar managed await semantics but has bounded backend-specific capacity
and requires host progress. The prototype's Int32-zero admission acknowledgment is a
bridge detail, not a final public API. No performance advantage, suspension support,
parallelism or production-ready native interoperability is claimed.


## Scheduler and suspension benchmark requirement — 2026-10-11

The author requires ongoing scheduler/suspension benchmarks to measure improvement
from compiler-generated state machines to runtime suspension. Establish a retained
baseline for the current state-machine path before replacing its execution mechanism;
keep that path runnable for matched comparisons. Current projection validation is
correctness evidence, not performance evidence.

Use the independent benchmark framework and compare within each execution mode before
comparing interpreter, JIT and AOT. Fix workload, compiler/runtime revisions, build flags,
hardware, concurrency, payload, warmup and sampling policy. Measure startup separately
from warmed steady state; record host/polling/GC differences instead of attributing
all observed change to the suspension mechanism.

Initial cases should cover already-completed awaits, repeated pending suspend/resume,
continuation chains and fan-out, timers, and loopback HTTP client/server work. Include
cancellation/fault teardown as correctness gates. Compare operation throughput and
latency distributions, allocations/bytes per operation, retained memory per pending
operation, and suspension/resumption cost. Add instrumentation where these metrics
are not yet available; do not substitute process RSS for allocation counts. Use
in-process measures for dispatch costs and external-process measures for startup and
end-to-end workloads, with repeated samples and raw evidence.

Keep API/outcome semantics and source workloads equivalent. First compare scheduler
changes with state machines unchanged; later compare state-machine and runtime-suspended
execution on the same scheduler. The .NET-style state-machine path is the local baseline;
external .NET/Go/JavaScript comparisons are separate studies. Document regressions and
tradeoffs as well as gains; no percentage target or speedup is assumed in advance.

The first [retained state-machine baseline](../benchmarks/scheduler/README.md) now
executes completed and pending sequential awaits in interpreter/JIT/AOT on macOS ARM64.
It records external startup and batch timing, not allocation, retained-memory or
per-resumption latency measurements. General async benchmark lifecycle support and
runtime-suspension comparison remain open.

## Unified interpreter readiness — 2026-10-11

The private interpreter scheduler's completion query now reports Ready for admitted
submissions as well as staged source completions. Pending means sources remain but
no callback is runnable; Idle means neither kind of runnable work nor pending source
remains. Previously, a submission-only scheduler could report Idle (or Pending when
an unrelated source was pending); the VM compensated by checking submissions before
acting on that result. This slice removes that misleading private contract without
claiming a previously observed application hang.

Submission/source arbitration now reads the scheduler's own staged source state,
not a boolean supplied by its caller. Sources are still polled while submissions are
available, and a delivered submission yields to a staged source before another
submission. Failed frame installation retains work and its roots; cancellation is
checked before reporting readiness. The VM still owns frame installation, and source
callbacks still use the legacy queue adapter. This is not await-site routing migration
or runtime suspension.

The existing .NET comparison separates runnable scheduling from awaitable outcomes;
this internal correction provides a consistent progress/quiescence contract for future
execution adapters. No .NET API divergence or public managed API change is introduced.
Focused checks cover submission-only and mixed source readiness, installation failure,
fairness, cancellation/GC cleanup, queue-free execution and existing timer consumers.

Validation: 14 scheduler unit tests, two queue-free submission consumers and three
timer consumers pass. Six interpreter state-machine benchmark processes pass with
unchanged guest assembly/library inputs; [raw observations and limits](../benchmarks/scheduler/README.md#interpreter-readiness-regression-check).
No performance improvement is claimed. Native scheduling and public APIs are unchanged;
JIT/AOT/Windows qualification and the API snapshot were not repeated for this private
interpreter correction. Website review found no public behavior claim requiring change.

## Producer-independent continuation experiment — 2026-10-11

A [source-included awaitable](experiments/scheduler-continuations/README.md) now retains
outcomes in the existing managed Promise while submitting its own await continuations
through the runtime. An explicit producer queue contains a faulting pump sentinel;
pending/completed awaits, multiple awaiters, deferred observation and cancellation
succeed without pumping it. A fault after submission terminates without resumption.
Both cases pass in interpreter/JIT/AOT on macOS ARM64.

This qualifies current generated state machines against a different continuation
adapter without changing compiler lowering or public Task semantics. The registered
callback list is still managed; runtime admission owns each submitted callback.
Destination is implicitly the same invocation in this experiment. It is not yet an
await-site destination token, cross-invocation/race-safe registration or general
custom scheduler capture. Those rules must be settled before replacing production
Promise registration. The existing .NET comparison remains the design baseline;
producer independence alone does not provide context flow or ConfigureAwait semantics.
The benchmark workload and shipped library remain unchanged, so no routing speedup
or memory improvement is asserted.
