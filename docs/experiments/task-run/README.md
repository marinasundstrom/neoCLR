# Task.Run — shared captures and runtime-owned submission

**Development, 2026-09-27.** Public Run overloads now cover shared captures,
completion-only and typed results, and async unwrapping. See the [public integration
checkpoint](#public-overloads-and-async-unwrapping) and its compiler limitations.
The sections below preserve focused evidence for each prerequisite checkpoint. The [concurrency design](../../concurrency-direction.md#taskrun-with-shared-captures--author-direction-2026-09-27)
owns its selected sharing/progress contract, .NET comparison and future green-thread
direction. [Runtime tracking](../../tracking/runtime-language.md#author-selected-taskrun-work--2026-09-27)
owns implementation status.

## Implemented boundary

Managed storage changes from Rc/RefCell to Arc and a mutex per slot. Cloning a
reference retains location identity; heap references remain weak capabilities whose
lifetime is governed by tracing. Storage access and its write counters/payload cache
are protected together. A whole-value read returns a coherent copy. A guest
read/modify/write sequence is still multiple operations, not an atomic increment.
The managed-slot slice introduces no unsafe Send/Sync implementations; the
native-buffer ownership slice below has a separate allocator-ownership proof.

Debugger snapshots copy the slot payload before following its references, avoiding
holding storage locks during graph traversal. Existing frame-escape, readonly,
construction and output-assignment rules remain enforced. Native tests access the
same object through interior aliases on two threads; no serialization or object
copy is used to substitute for sharing.

This is a storage prerequisite only. The interpreter still runs one execution per
invocation. The coordination prerequisite below now protects its heap access, but
it does not yet interleave multiple guest VMs. The new
native tests keep the heap alive while accessing it, and only collect after the
threads join. They **do not** establish safe concurrent collection. Promise,
TaskQueue, general collections and host-resource ownership still need their own
concurrency contracts and implementation. Arc alone does not supply GC roots.

.NET's shared-object Task.Run behavior motivates preserving identity, but this
per-slot lock is a neoCLR implementation choice, not a claim about CLR internals.
It trades atomic reference-count operations and lock overhead for safe native
storage access. A purpose-built atomic/borrow protocol could cost less but would
introduce more correctness obligations; it is not required for this slice.

## Focused validation

```sh
cargo test --lib slots::tests -- --test-threads=1
cargo test --test reference_slots --test heap_references --test delegates \
  --test debugger --test class_constructors --test runtime_gc
```

Results: 10 slot tests and 99 selected integration tests pass. Coverage includes
cross-thread alias mutation, whole-record replacement/read consistency, payload
cache invalidation, weak-handle expiration, captured receiver retention, explicit
GC, construction roots, frame provenance and debugger inspection. No full suite,
website build or Raven/API snapshot regeneration is needed for this internal
storage change; no Raven signatures changed.

## Single-thread cost check

[measure-storage.rs](measure-storage.rs) runs 200,000 class-field reads/writes with
local counter updates per observation. Module loading and verification occur
outside timing; each execution checks its result. This specifically exercises the
ordinary-code path affected by slot locks. It is not a Task.Run throughput test,
HTTP qualification or comparison with .NET performance.

Build the probe against separate baseline and candidate release libraries:

```sh
cargo build --release --lib
rustc --edition=2024 -O docs/experiments/task-run/measure-storage.rs \
  -L dependency=target/release/deps \
  --extern neoclr=target/release/libneoclr.rlib -o /absolute/path/to/runner
python3 docs/experiments/task-run/compare-storage.py \
  --baseline /absolute/path/to/baseline-runner \
  --candidate /absolute/path/to/candidate-runner \
  --baseline-commit 5522cf1a --output /absolute/path/to/results.json
```

On this machine, use SDKROOT pointing to the installed Xcode macOS 26.2 SDK if the
Command Line Tools SDK cannot be linked by the selected compiler. The checked-in
[observations](storage-cost.json) retain runner/source hashes, compiler/platform,
all observations and medians. The order is baseline/candidate/candidate/baseline,
with the first of six observations per process excluded as warmup. This is a small
local regression check with background machine activity, not a release threshold.
The recorded medians are 0.4331 seconds for the baseline and 0.4497 seconds for the
candidate (1.0383×, about 4% slower). This observed cost does not justify a separate
optimization project; repeat only if subsequent shared-execution changes or a
supported workload raise a material regression question.

## Shared heap/root coordination

The runtime now creates an invocation-owned heap coordinator. A participant has a
registered root snapshot and acquires exclusive graph access. Root publication,
new participant registration and collection happen under that same gate. Collection
combines the current participant's roots with every retained participant snapshot;
transitive references are still traced by the existing collector. Registered roots
are invocation-local VM allocation IDs, not guest handles or a cross-heap import API.

Registration precedes transfer of queued captures. A completion can retain its
participant until the receiving owner publishes the result. Registrations are weak
in the coordinator, so dropping an abandoned submission never reacquires the gate
and cannot deadlock a submitter holding it. Expired entries are pruned on admission
and collection. Admission has an explicit participant bound; exporting the final
heap requires all participants to have been released. Invalid root publication
preserves the previous snapshot and does not partially sweep.

The VM's automatic, explicit and final collections now use this coordinator. The
current VM uses a one-participant budget. The subsequent suspension slice below
now publishes roots and releases access at instruction boundaries; public concurrent
guest submission is still pending. Blocking boundaries require separate integration. No GC can make progress
while a participant holds access, so holding that guard during a blocking host call
is not a viable final Task.Run scheduling policy. Slot locks alone remain insufficient.
All VM graph access must obey the gate; the internal ManagedHeap helper interface
must not be used to bypass registered-root collection.

Focused checks for this slice:

```sh
cargo test --lib shared_heap::tests
cargo test --test runtime_gc --test gc_diagnostics --test heap_references --test entry_results
```

Six coordinator tests and 25 integration tests pass. They cover queued capture
retention, native mutation/result handoff, exclusion of collection during mutation,
transitive graph retention, abandoned admission, invalid publication, bounded
participants and export ordering. Existing VM collection/entry-dispatch behavior
passes. No new per-instruction scheduling or slot operations were introduced; the
previous storage cost check is retained, not rerun as a scheduler benchmark.

## First native work-submission slice

`src/task_work.rs` implements an internal, bounded native work owner. It is tested
with runtime-side callbacks and is **not yet connected to the public guest delegate
submission path or Task.Run**. The subsequent suspension probe runs a guest function
through this work owner. One dedicated native thread per admitted job is the initial
backend primitive; pooling remains runtime policy. Handles are single-use and never
reused. The bound is total submissions per owner, not a public Task.Run option.

Submission registers capture roots under heap access before starting the thread.
A worker can release access while blocked and publish temporary execution roots
between access intervals. Producing a result and publishing its roots happen in
one interval; completion retains that registration until the receiving owner
publishes the result. Invocation identities reject foreign submissions/completions,
and reference-result provenance checks reject handles from another heap even if
allocation IDs coincide. Existing frame-escape checks also apply. Root IDs passed
by the internal driver remain trusted invocation-local root-walker output.

The owner provides nonblocking readiness inspection and explicit join. Teardown
requests a private stop token and joins all owned jobs, including after a fault;
it does not cancel the host's token merely because this owner is disposed. Workers
observe both host cancellation and local stop. Waiting for heap access is cancellable,
so a stopped worker need not acquire the gate to exit. Blocking host operations
still require their own cooperation; this does not forcibly interrupt arbitrary
native code. Normal teardown must inspect faults; Drop joins as an unwinding
fallback. Faults remain invocation failures, not new TaskOutcome cases.

The driver must normally join/teardown outside heap access. The cancellation test
also covers cleanup of a worker already waiting for that access. Public Promise
completion, queue publication, wake notifications, shared host resources, aggregate
instruction/frame budgets, guest safepoints and async unwrapping are not supplied
by this component. Do not enable a public facade until these are connected and
validated. The VM now pauses between instruction intervals and releases graph
access during scheduler waits and prepared file/console I/O, worker joins and native imports. The runtime-side blocking probe is
not proof that guest blocking calls have already been offloaded correctly.

```sh
cargo test --lib task_work::tests
cargo test --lib shared_heap::tests
cargo test --test heap_references host_can_inspect_returned_fields_only_in_the_owning_execution
```

Nine native work tests, six coordinator tests and the host provenance regression
pass. The work tests cover caller progress during a blocked callback, shared
mutation, parked temporary roots, fresh reference results, bounded admission,
single-use handles, cancellation/join, fault cleanup and foreign-heap rejection.
These are contract checks, not throughput benchmarks or Raven API examples.

## Guest instruction-boundary suspension

The normal interpreter now runs in internal 1,024-instruction intervals. It retains
frames and execution state across intervals: remaining instructions, GC threshold,
array-accounting state, file services, intern pool, scheduler, default queue,
invocation result and entry-drain state. Before releasing graph access it publishes
all interpreter roots. Interval resumption does not reset the instruction budget.
A zero interval is rejected. Source teardown runs after graph access is released.
The interval is a private runtime choice, not a new application API or promise of
fairness, parallelism or thread identity.

A native-work acceptance probe executes a guest function with a shared captured
object and reference result. It pauses after every instruction while the caller
forces collection, then verifies identity, mutation and reclamation. A separate
small linked System fixture checks queue construction, explicit entry draining,
final draining and the returned queue across the same collection-at-every-pause
schedule. These probes do not imply Raven Task.Run overloads or general concurrent
Promise/collection safety.

```sh
cargo test --lib vm::suspension_tests
cargo test --test entry_results --test runtime_gc --test cancellation \
  --test file_streams --test debugger
```

All four suspension tests and 47 selected integration tests pass. A separate run
of the legacy `tests/tasks.rs` fixture performed its broad library preparation;
six tests passed, and the remaining access-control test was intentionally stopped
with SIGINT to avoid repeated unrelated whole-library verification. The focused
queue fixture replaces that broad validation for the changed resumption behavior;
this is not a claim that the complete legacy Task suite passed. No website build,
full suite or API snapshot regeneration was run.

The existing release-build class-field probe was reused because root publication
now occurs during ordinary long-running execution. [Suspension observations](suspension-cost.json)
compare the fingerprint-checked storage-slice runner at `16a4784c` with this slice.
Medians were 0.5116 seconds and 0.5036 seconds (0.9845×). This small, noisy local
sample shows no observed regression; it does not establish a speed improvement,
contention behavior or Task.Run throughput. The comparison also includes the
intervening heap-coordination commits and does not isolate a single lock cost.

## Completion waits outside graph access

Completion dispatch now retains its boundary value when registered host work is
pending. The scheduler polls under graph access; the driver releases that access
before parking, then resumes dispatch even when the last guest frame has returned.
The pending boundary, invocation result, queue and source callbacks remain rooted.
Waiting does not consume guest instruction fuel. The existing wake latch covers a
notification between poll and park; cancellation and sockets retain bounded polling.

This removes one obstacle to .NET-like caller progress during Task.Run work. It
is not a thread-pool implementation or a promise that arbitrary guest/native blocking
calls already permit concurrent managed progress. Serial graph access remains the
correctness baseline, with sharing and budgets still requiring integration.

```sh
cargo test --lib vm::suspension_tests
cargo test --lib scheduler::tests
cargo test --test entry_results --test cancellation
```

Six suspension tests, nine scheduler tests and eleven entry/cancellation tests pass.
The pending loopback-accept probe permits another native participant to collect
while no guest frame exists, preserves an independent reference result, verifies
unchanged fuel across repeated waits, and resumes with exactly one notification
post and the final queue drain. A separate cancellation case checks a parked
completion boundary. Existing wake tests cover poll/park notification races.
No public signature changed; the website's future Task.Run status remains accurate.
No website build, API snapshot regeneration, full suite or new benchmark was needed.

## Prepared blocking guest calls

The interpreter now yields a prepared call to its native driver. File services,
console reads/writes/flushes, isolated-worker joins and trusted native imports run
outside graph access. The caller frame, arguments and pending read destination are
published first. Native pointers remain owned by the executing context; this change
does not yet share native allocations between contexts.

Byte writes use a detached snapshot. ReadInto validates and roots its destination,
performs the bounded read outside access, and commits only transferred elements
after reacquiring access. This preserves aliases and changes to the untouched tail;
it does not make conflicting application writes to the same range safe. Faults
retain the caller location and host calls are not replayed. Cancellation remains
cooperative: an in-progress host Console or native call cannot be forcibly interrupted.
This supplies the blocking boundary needed for .NET-like Task.Run progress without
adopting .NET's thread-pool implementation or exception-bearing Task contract.

Seven suspension/guest-call probes, two buffer handoff probes, and 33 focused
file-stream/native/debugger integration tests pass. The guest console probe admits
collection and capture mutation by a different participant during the host call.
A broader invocation was stopped when its console-stream fixture began whole-library
preparation; it is not counted as completed coverage. No website build was run.

## Shared instruction and live-frame budgets

An invocation budget now supplies atomic, non-resetting instruction fuel and
live-frame permits. VM contexts can share this owner; a paused or blocked frame
keeps its permit, while return/teardown releases frame capacity. New frames are
admitted before executing or parking, including initial callback frames. Waiting
consumes no fuel, and exhausted fuel never wraps or refunds on task completion.
This retains neoCLR's bounded-host contract; .NET Task.Run does not supply equivalent
per-invocation instruction or frame quotas by default.

Two budget tests cover concurrent native consumers and unwind cleanup. Eight VM
suspension tests include two guest contexts sharing one budget, rejecting excess
live frames, reusing released capacity and exhausting the same fuel. Nineteen
focused cancellation/startup/GC integration tests also pass.

[Budget cost observations](budget-cost.json) reuse the existing release class-field
probe and a fingerprint-verified runner from `891bf8d4`. Median times were 0.6638s
and 0.4939s (0.7441×). No slowdown was observed, but the noisy comparison spans the
intervening wait/host-call changes and does not establish a speedup or isolate the
cost of atomic accounting. No broad suite or website build was run.

## Shared file and interning services

VM contexts now take one invocation owner containing immutable limits, the shared
budget, file table and intern pool. Passing that owner preserves open-handle identity,
file position, close state and the existing 64-file cap, as well as canonical string
owners and intern quotas. These services outlive an individual context and are disposed
when the invocation owner is released. Isolated Thread workers still create their own
invocation owner and retain their existing isolation contract.

File operations take the file-table mutex only outside graph access; file operations
never reacquire the graph gate. The initial lock serializes even unrelated file
operations. This is a correctness baseline with a concurrency cost compared with
independent .NET FileStream instances, not a claim of better I/O scheduling. Interning
holds only its own short-lived lock and performs no host I/O.

Two native-context service probes and nine VM suspension/ownership probes pass,
including shared file position/close state and guest interning identity/quota checks.
The twelve file-stream integration tests pass. An older interning fixture was stopped
when its whole-library preparation became apparent; its partial run is not evidence
for this slice. The small System fixture validates the actual guest String.Intern
binding across two contexts without rebuilding or verifying the entire library.

## Shared scheduler and default-queue ownership

The invocation owner now holds the completion scheduler and registered default queue.
A service-root registration keeps queue, pending callbacks and staged completions
alive independently of the context that submitted them. It is bound to one heap,
blocks heap export until released, and does not consume an executable participant
slot. Source/root updates follow graph-then-dispatch lock order. Collection inside
an instruction interval still includes the current service values before their next
published snapshot.

One context owns completion draining across instruction pauses and host calls.
Other contexts wait without consuming instructions and still observe cancellation.
The claim is released on completion, entry-drain exit, terminal fault or context
disposal. This protects runtime-driven draining; it does not yet make arbitrary
concurrent guest TaskQueue.Post/Run or lazy TaskQueue.Default construction atomic.
Those library mutations remain part of the Promise/queue publication slice.

Worker joins detach their receiver and dedicated-thread ownership before waiting;
the scheduler stays available for cancellation requests and other sources. A consumed
handle cannot be joined twice. Detached teardown requests cancellation and joins the
dedicated producer outside graph access. Parking uses the invocation wake directly,
without taking the dispatch lock. Final execution collection now follows service
teardown, preserving the existing single ExecutionCompleted collection and reclaiming
service-only objects before heap export.

Like .NET's runtime scheduler, submission lifetime does not define callback-root
lifetime. neoCLR retains invocation-scoped limits and one completion pump, rather
than adopting .NET's process-wide pool or promising parallel callback dispatch.
The shared graph/dispatch locks are a correctness baseline, not a throughput claim.

Focused validation passes: two dispatch-service tests, eleven VM suspension/ownership
tests, twelve worker tests, seven heap-coordinator tests, nine scheduler tests and
22 GC/startup/cancellation integration tests. New probes cover source roots after
submitter exit, exact-once handoff, shared queue identity, dispatch contention and
cancellation during detached joins. Existing final-GC diagnostic counts still pass.
No whole-library fixture, full suite, API snapshot regeneration or website build was
needed; the public Task.Run API remains unimplemented.

## Shared native-memory owner and foreign-call handoff

Native buffers and loaded native-library handles now belong to the invocation.
Contexts use the same allocation identities, live-byte limit and cumulative
allocation-identity limit; starting another context cannot reset those quotas.
The native-buffer owner is transferable because it uniquely owns a global-allocator
allocation. Safe byte access is serialized by the shared memory mutex; the buffer
itself is not Sync. Execution still exports its ordinary PointerHeap after all
contexts and foreign-call leases have released ownership.

Trusted native calls borrow all directly supplied tracked allocations before leaving
for native code. Duplicate aliases share one lease, including raw addresses that
resolve to tracked storage. Admission validates every argument before marking any
allocation busy. Until the lease ends, conflicting VM reads, writes, frees and
foreign calls fault; debugger snapshots omit busy bytes. Independent buffers remain
usable. The lease keeps storage alive and releases the borrow on return, error or
wrapper unwinding. Prepared calls retain their library handle, and actual native
execution holds no graph, scheduler, library-table or memory mutex. Library loading
still serializes under the library-table lock, including trusted initializers.

This protects direct tracked arguments only. Indirect/global pointers and foreign
allocations remain the trusted unsafe ABI caller's responsibility. It does not make
an arbitrary native library thread-safe or infer ownership of returned pointers.
The per-operation memory lock and exclusive foreign borrow favor bounded ownership
and diagnosable conflicts; they add synchronization overhead and reject conflicting
access instead of waiting. This is a neoCLR implementation boundary, not a claim that
.NET Task.Run synchronizes user buffers or that this is faster than CLR interop.
The existing [concurrency comparison](../../concurrency-direction.md)
continues to define the public ergonomic target.

Focused validation: four native-memory tests and twelve VM suspension tests pass,
including a guest buffer shared with a native-thread context after submitter exit,
aggregate quota exhaustion, stale identities, borrow rollback/unwinding and unrelated
buffer progress during a foreign borrow. The existing native-import, pointer,
memory-operation, stack-allocation and debugger suites add 57 passing tests (73
focused tests total). Real C-ABI imports retain scalar/pointer round trips and lazy
library loading. No throughput claim, full suite, API regeneration or website build
is included; Task.Run remains a future public capability on the feature page.

## Aggregate arrays across guest contexts

Each heap participant now publishes the logical array payload in its private frames,
evaluation stacks, retained completion and prepared host-call arguments before
releasing graph access. The active VM combines its current payload with the parked
participants' snapshots and measures shared heap storage once. Reference aliases do
not duplicate heap charges; owned inline copies do. Participant teardown releases
its private charge, and pressure collection still reclaims unreachable heap arrays
before rejecting an aggregate allocation. Existing element and logical-byte limits
apply across contexts, including contexts that receive arrays without executing an
array-creation instruction.

Snapshots contain counts rather than values or additional GC roots. Publication and
checking use the graph gate; parked private frames must not be mutated outside that
protocol. Detached write buffers are charged as owned copies. Host-side transient
allocation and the copies within an instruction remain outside the exact accounting
model, as before; these are logical payload limits, not an OS memory cap.

This preserves neoCLR's bounded invocation contract while approaching .NET's familiar
shared-reference semantics. It does not adopt a CLR process-wide memory quota or
claim that Task.Run in .NET provides per-invocation array limits. The implementation
adds per-participant summaries and boundary scans; existing slot caches continue to
avoid remeasuring unchanged slot payloads at every instruction.

Validation passes: two aggregate-accounting probes, fourteen VM suspension probes,
two prepared-I/O probes, and 29 array/GC integration tests (47 focused tests total).
Cases include native-thread admission against a parked inline array, byte-budget
aggregation, shared aliases, collection, participant release, completed inline results
and detached write payloads. No full suite or website build was needed. The public
Task.Run feature page remains correctly marked as future work.

## Atomic task-library scheduling regions

The interpreter now retains graph access across instruction quanta inside the
closed System.Tasks library's Promise Complete/Cancel/Register/Read and state tests,
Task State/Outcome snapshots, TaskQueue Post/Run/Drain bookkeeping and lazy Default
creation. Identification uses the System definition owner and selected member names;
application methods with the same names receive no special policy. This is internal
runtime/library coupling, not a public synchronized-method attribute or general lock.

A callback frame entered by TaskQueue Run/Drain ends the enclosing atomic region.
Callbacks can yield and perform prepared blocking I/O; subsequent queue bookkeeping
regains the region when the callback returns. Nested task-library mutations inside
callbacks establish their own region. Instruction fuel, cancellation checks, GC and
resource limits remain active. Host-call suspension or invoking a user callback from
another protected mutation faults rather than silently releasing the region. A
terminal fault/cancellation does not roll back a partially performed mutation: it
fails the invocation under the existing Fault contract.

The generated library retains first-terminal-transition behavior, callback posting
and its existing rejection of recursive/overlapping queue pumps. The runtime does
not make arbitrary ArrayList operations or application read/modify/write sequences
atomic. Shared captures still require appropriate application synchronization.

.NET's [TaskCompletionSource<T>](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.taskcompletionsource-1?view=net-10.0)
documents concurrent member use (reviewed 2026-09-27). That is the ergonomic baseline
for Promise completion and observation. neoCLR reuses its graph gate for this first
implementation; it does not claim to reproduce CLR internals. Per-object monitors
would permit finer scheduling but require retained monitor ownership, waiters and
unwind integration. Native replacements for Promise/queue storage would duplicate
Raven library behavior. The selected approach keeps that behavior in Raven at the
cost of longer graph-lock intervals and explicit coupling to these method bodies.
There is no throughput or fairness improvement claim. A future compiler/backend or
a library change that introduces new callback/I/O paths must revisit the policy.

Focused validation passes: six new probes plus the sixteen existing VM suspension
and prepared-I/O probes (22 tests). The new fixture executes checked-in generated
Tasks, ArrayList and required contracts with one-instruction quanta; it does not
recompile Raven or verify the whole library. Object formatting is an explicit
faulting fixture stub because reflection/formatting is outside these scenarios.
Checks cover completion versus cancellation, callback registration versus completion,
posting during a yielding drain callback, one Default identity, application-name
isolation, instruction exhaustion, cancellation and forbidden host suspension. The
older queue suspension probe now expects bookkeeping to remain within one interval.
No public signature, API snapshot or website build changed.

## Rooted, accounted capture and result handoff

Production submission now takes owned capture values and passes those same values
to the native callback. Under graph access, admission validates heap provenance,
walks nested captures (including delegate receivers), registers their roots and
charges inline array payload before spawning. Failed quota/provenance admission
starts no worker and consumes neither participant capacity nor a submission handle.
The older raw-ID submission helper is now restricted to coordinator tests.

Completion replaces the worker's private roots and payload charge with its result.
The completion registration retains them after native-thread exit. Receiving the
result publishes its roots into the receiver and transfers the private charge under
one graph-access interval, so the result is neither temporarily unaccounted nor
counted twice. Oversized results fail and release the worker registration. Pressure
collection may reclaim unreachable heap arrays before rejecting aggregate usage.
Callers must publish current private roots and usage before submission/receipt and
republish after changing their live values; the future public driver must enforce
that protocol. No public host ownership transfer is implied.

Array measurement now follows owned delegate receivers as well as records, erased
values and arrays. This closes an omission for inline arrays inside captured value
receivers; managed-reference aliases still charge only the heap-owned payload.
Such payloads can now reach the existing ArrayLimitExceeded boundary where they were
previously omitted. This remains logical accounting rather than exact host memory.

The .NET comparison remains shared reference identity with retained captures/results,
as described in the [selected concurrency contract](../../concurrency-direction.md).
neoCLR additionally enforces its invocation budgets and heap provenance. Moving owned
values avoids a serialization/deep-copy substitute, while validation/publication adds
boundary work. No throughput claim or general-purpose host sharing API is introduced.

Validation: thirteen native-work tests, four array-measurement tests, one guest
native-callback/forced-GC probe and 26 delegate integration tests pass (44 focused
tests). New cases cover parked captures, completed-but-unreceived results, exact-limit
transfer, rejected admission rollback, foreign captures, oversized-result cleanup and
inline delegate-receiver payloads. No whole-library rebuild, full suite or website
build was needed; public API signatures and snapshots are unchanged.

## Invocation-owned work lifecycle and wakeup

The invocation now lazily owns its bounded native work service. Submission uses its
limits and host cancellation token. Completion, cancellation and callback failure
signal the existing scheduler wake latch; ready outcomes remain rooted until taken.
The latch is a hint, so polling retains the existing bounded timeout when a signal
arrives just before the native thread finishes. Handles remain single-use.

Non-cancellation callback faults and panics leaving the native callback are recorded
as invocation failures and request sibling cancellation. The VM checks the retained
failure before executing another instruction interval. Consuming a failed job cannot
hide the invocation failure. This uses the existing Fault model rather than adding
a TaskOutcome error case. Host operations remain cooperatively cancellable; a
noninterruptible native call can still delay shutdown.

The root driver explicitly closes admission, removes the work owner from its service
mutex, and joins outside graph/service locks before releasing shared services or
exporting either heap. A root-driver scope repeats that cleanup during unwinding,
breaking the job-to-invocation lifetime dependency. An existing guest failure keeps
precedence; an otherwise successful driver reports a retained worker failure.
Normal invocation exit stops outstanding work; this does not yet define the public
Task.Run awaiting/entry-drain adapter. Only the root driver owns this shutdown scope.

The initial internal service reuses Limits.frames as its total native-submission cap,
while the existing shared frame budget independently bounds live guest frames. This
is a bounded prerequisite, not a newly published task-limit contract. The .NET
comparison remains runtime-owned scheduling with shared lexical captures; neoCLR's
invocation lifetime and host quotas are deliberate boundaries, not a reproduction
of .NET's process-wide pool. Detaching the owner before joining avoids a lock-cycle;
no scheduling-throughput improvement is claimed.

Validation passes: thirteen native-work tests, five invocation-lifecycle tests,
fifteen guest suspension tests and fourteen GC/cancellation integration tests
(47 focused tests). New coverage includes completion wakeup and single consumption,
closed admission, retained faults/panics, sibling cancellation, shared host cancellation,
root unwinding while a worker retains the invocation, service-lock availability during
join, and fault observation before any further guest instruction. Existing final-GC
behavior still passes. No full suite, website build or API snapshot change was needed.

## Guest delegate execution and native entry draining

An internal adapter now invokes zero-argument guest delegates on native work contexts,
including captured receivers and typed or completion-only results. Frames are built
on the worker; instruction/frame budgets, heap, scheduler, files, interning and native
memory retain their invocation owners. A callback returns without claiming the root
completion pump or waiting for its own native job. Public Task.Run is still absent.

The root completion boundary parks while native jobs run. A completion revision
requests another default-queue drain, including when completion races the initial
status read. Repeated waits spend no guest instruction fuel and do not repeatedly
execute queue drains. Ready host notifications still progress while native work runs.
The root closes and joins work on faults or unwinding; normal completion first waits
for pending work through this boundary.

Guest cancellation observes both host cancellation and work-service shutdown/sibling
failure, including supported host waits. Noninterruptible host/native operations can
still delay shutdown. Captured output is handed to the root once at execution/host
boundaries and callback return. Live console output retains the existing embedding
contract and is not replayed. Ordering follows flush batches, not a cross-thread
wall-clock ordering guarantee. Each job currently reuses worker_result_bytes to bound
captured output (UTF-8 line bytes plus newline and raw stream bytes); typed values
continue to use shared heap/array limits. These internal quota choices are provisional.

This follows the selected .NET comparison: shared captures and native-thread progress
while callbacks block, with runtime-owned scheduling. Invocation-scoped lifetime and
quotas remain deliberate differences from .NET's process-wide pool. No throughput
claim or green-thread implementation is implied.

Focused validation covers shared capture identity and mutation, forced collection,
typed/completion-only results, shared fuel, return while another context owns dispatch,
shutdown of a running guest loop, captured-output forwarding/limits and live console
behavior. The entry-drain probe verifies fuel-preserving waits and observes queue work
after native completion. Existing VM, native-work/lifecycle, linked cancellation and
console/GC/cancellation checks pass. No full suite or website build was needed. No
public API signatures or snapshots changed.

## Private native submission boundary

The VM now admits `neoCLR.Runtime.ScheduleTask(Func<Void>)`, classified as the
existing TaskDispatch service. It publishes the submitting frame's roots/payload
before creating the native context. The root heap allows one caller plus the existing
bounded total job count (Limits.frames); finished jobs retain their registration until
invocation shutdown. This is conservative retention, not a pool or a live-job quota.
Callbacks cannot invoke the entry-drain service recursively. A terminal guest callback
fault fails the invocation rather than becoming a successful task outcome.

The focused generated-library probe schedules a captured guest receiver through the
actual native binding, completes Promise<int>, and observes its registered continuation
before root exit. Its failure variant verifies propagation of a guest Fault. All 29 VM
checks and 12 runtime-service checks pass. This is internal runtime plumbing; no public
Task.Run reference, Raven compiler policy or API snapshot changed.

## Public overloads and async unwrapping

The development static System.Tasks.Task owner now has three Run overloads:
Func<unit>, Func<T> and Func<Task<T>>. They create a Promise on the default queue and
submit a retained helper delegate through ScheduleTask. Typed results preserve
reference identity. The async helper registers the existing OutcomeTransfer against
the returned inner task; its completion or cancellation becomes the outer outcome.
It neither blocks a native thread for the inner result nor adds a Task error case.
No per-call cancellation-token overload or .NET ExecutionContext/AsyncLocal flow is
claimed. Invocation cancellation, faults and host quotas retain the earlier contract.

The selected .NET comparison is the familiar submission/result shape and one-level
async unwrapping. The first backend uses invocation-owned native threads rather than
.NET's process-wide thread pool. Managed instruction intervals serialize through the
heap gate; blocked host work releases it. No CPU-parallelism or throughput claim is
made. Shared arbitrary read/modify/write sequences still require coordination.

The bridge admits exact public overload signatures and keeps ScheduleTask confined
to bootstrap RuntimeServices. Source export keeps Task and Task<T> as distinct nominal
owners. The public-overload slice changed no Raven Runtime Contract setting or compiler
source; the subsequent capture correction is recorded below. Constructor-
assigned helper storage uses explicit fields for the already-recorded private-var
compiler limitation. Only the Tasks implementation was regenerated; other generated
manifests refresh their shared source fingerprint while reusing unaffected artifacts.

[Main.rvn](Main.rvn) exercises captures, reference identity, completion-only work,
nested submission and an async callback that itself awaits native work. Its expected
output is 42, True, 41, 21 and 7 on separate lines. Five compiled consumer cases
pass: that sample, unobserved-work entry draining, unit-result async unwrapping,
cancellation from an inner task and a terminal callback fault. Mutable-local sharing
now has separate positive regressions below. Run with matching --runtime, --bridge,
--system and --reference artifacts;
--case selects one check. Eleven bridge signature checks and 29 VM tests pass. The API
reference includes the static owner and all overloads; matching API/library snapshot
checks pass. Website source/downloads are updated without building the website.

Two [compiler integration gaps](compiler-gaps/README.md) remain: short-name Task
lookup and block-lambda result inference. The passing consumer
uses an explicit alias and a typed callback local. These
workarounds do not redefine the API contract.

## Shared mutable-local capture checkpoint

Raven main `dc7b87eff` was developed on `codex/async-mutable-captures` independently
of neoCLR and integrated individually as `08815ceaf` on the experimental branch.
The original compiler copied a mutable scalar into callback storage while its async
caller read a separate local or state-machine field. The fix retains one shared
closure per invocation through suspension and completion, and redirects captured
local reads/writes/addresses to it. Local-function calls resolve the host closure.
This restores the expected .NET-style shared-variable semantics; it adds private
state-machine closure storage, not atomicity for concurrent read-modify-write work.
Runtime Contract configuration, reference signatures and native runtime code are
unchanged. No importer checks were relaxed.

Seventeen focused ordinary .NET runtime checks pass, covering direct and Task.Run
callbacks, forced suspension, caller writes, sibling reads, per-invocation lifetime,
receiver/parameter capture and local functions. The original two suspension cases
failed before the fix; ordinary synchronous capture neighbors already passed.
The rebuilt neoCLR bridge uses the same fix. [MutableCapture.rvn](MutableCapture.rvn)
and [MutableCaptureInline.rvn](MutableCaptureInline.rvn) are positive regressions,
requiring 42 rather than accepting the old incorrect 0. The former exercises native
submission; the latter removes Task.Run and invokes the callback directly in async
Main. Both compile, pass typed-stack verification and exit with 42 using the unchanged
development System library/reference. The existing shared-object/async-unwrapping
consumer was also recompiled, verified and run, preserving its expected output
(42, True, 41, 21, 7). These three cases are the neoCLR checks rerun for this correction;
unaffected fault/cancellation and native-runtime checks reuse their earlier evidence.
The library has no authored async functions, so this correction changes consumer
emission without requiring library regeneration. The API snapshot refresh changes only the XML
documentation fingerprint and reuses the unchanged reference assembly.

A separate generic-method closure metadata failure reproduces before and after this
fix on ordinary Raven/.NET. It is [retained as a compiler follow-up](compiler-gaps/README.md#separate-general-raven-follow-up-generic-method-closure-metadata),
not hidden by lowering the required result or weakening the importer. This slice
does not establish async-lambda-owned locals or iterator capture planning.

## Direct unit-await checkpoint

Raven `c08f343b1` on `neoclr` now recognizes the configured inhabited unit representation
when an imported generic method returns it. Previously `Task<System.Void>.GetResult()`
left a value on the stack because the compiler classified it as a no-result call.
The correction discards that value once in statement position. Unlike CLR's `void`,
neoCLR's configured System.Void is a unit value in generic storage; ordinary CLI
no-result returns keep their existing treatment. No Runtime Contract setting, runtime
API, or importer stack check changes. This target-specific correction stays on
Raven's `neoclr` branch; it is not a general main-branch extraction.

[UnitAwait.rvn](UnitAwait.rvn) is now a positive regression. Main and MutableCapture
also await completion directly, without Map. Focused checks compile, typed-verify
and execute these three consumers against the unchanged development library/reference.
Twenty-four neighboring Raven async/unit contract tests pass. Assignment of an awaited
unit to a local also compiles in the reduced probe; runtime validation here covers
statement-position awaits. No full suite, performance suite or website build is needed.
The API snapshot update is XML-only; no public signatures change.

## Main integration checkpoint

The completed Task.Run slices are integrated with main's routing, typed JSON arrays,
retained reflection constructors and RavenDoc checkpoint. Native ScheduleTask dispatch
and reflection adapters are retained in the merged interpreter; the combined reference
assembly is regenerated and the API snapshot refreshed. Fifteen VM suspension checks,
nine reflection checks and eleven Task.Run signature checks pass. The shared-capture/
identity/unwrapping consumer compiles, typed-verifies and executes with the combined
runtime, reference and library. Source/generated-library fingerprints match. No full
suite or website build is used for this integration.

## Next bounded work

Resolve short-name lookup and block-lambda inference in isolated Raven work with
independent CLI tests. Keep the
remaining negative fixtures until corrected. Public release readiness still requires
reviewing these integration gaps and the separate generic-method capture candidate;
no broader platform matrix or website build is required for this correction.
