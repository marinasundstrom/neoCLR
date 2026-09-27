# Task.Run prerequisites — shared managed storage

**Development, 2026-09-27.** The public Task.Run API is not implemented by this
slice. The [concurrency design](../../concurrency-direction.md#taskrun-with-shared-captures--author-direction-2026-09-27)
owns its selected sharing/progress contract, .NET comparison and future green-thread
direction. [Runtime tracking](../../tracking/runtime-language.md#author-selected-taskrun-work--2026-09-27)
owns implementation status.

## Implemented boundary

Managed storage changes from Rc/RefCell to Arc and a mutex per slot. Cloning a
reference retains location identity; heap references remain weak capabilities whose
lifetime is governed by tracing. Storage access and its write counters/payload cache
are protected together. A whole-value read returns a coherent copy. A guest
read/modify/write sequence is still multiple operations, not an atomic increment.
No unsafe Send/Sync implementations are introduced.

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

## Next prerequisite

Integrate native allocation ownership and aggregate array/native-memory accounting
before general guest work
submission. Then establish atomic Promise/queue publication and expose the
Run overloads, with async callback unwrapping validated at that API boundary.
Instruction/live-frame budgets, file handles and interning now share an invocation
owner. Array/native-memory and remaining host-resource accounting still require
integration; a new task must not reset those limits. Retaining every object until
invocation exit is not a substitute for bounded live-object accounting.
