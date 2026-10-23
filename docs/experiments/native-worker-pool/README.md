# Bounded native worker pool experiment — 2026-10-23

The author authorized continuing the worker experiment directly in main runtime
code. The implementation is under
[runtime/native/workers](../../../runtime/native/workers/native-workers.h).
It is an opt-in POSIX prototype, not linked into Task.Run or HttpServer and not a
public runtime ABI. No existing scheduler/GC default changes.

## Implemented boundary

One pool owns a bounded shared arena, 1–4 long-lived pthread workers, up to 16 job
slots, eight temporary-root slots per job and 32 consumer-root slots. The mutex protects FIFO admission, cancellation,
completion publication, roots, allocator access and collection. User callbacks
run outside the mutex and enter explicit managed-access intervals. Computation on
copied primitives and blocking outside those intervals may overlap across workers.

This is the **serial managed-access reference implementation** proposed by the
investigation. It does not execute arbitrary generated Raven code in parallel.
The synchronous bridge below uses existing compiler root hooks while holding the
heap gate; there is no parallel-mutator safepoint integration, automatic collection on allocation
pressure, stack/fiber migration or concurrent Promise support. Existing TLS
host-root/socket/fiber APIs must not be used on the pool context. Callbacks may
not retain and dereference unregistered managed pointers outside access intervals.

Capture roots are adopted before job publication. Workers publish result reference
slots before releasing managed access, including while the callback remains
unsettled. Temporary references can be explicitly published to job-local slots
before leaving the gate. Owner collection traces all temporary/capture/result/consumer slots using the
canonical collector. Workers may allocate under the gate. Ordinary native callbacks cannot initiate
collection; the synchronous generated bridge below admits existing compiler GC
hooks while publishing all pool roots. Thread-local allocation counters
therefore do not describe whole-pool allocation volume.

Each submission reserves its completion slot. Ready results stay rooted if consumer
root capacity is full; take does not discard or partially publish them. Handles
carry pool identity, generation and root/ticket domain, rejecting foreign, stale
and wrong-kind handles. Consumer adoption precedes producer release. Reference
payloads are returned as roots, not unrooted pointers.

Queued cancellation settles without invoking the callback. Running cancellation
requests cooperative stop and wins only before settlement under the gate. Close
stops admission, cancels queued/running work, wakes waiters and joins outside the
gate. It cannot interrupt arbitrary native work. Destruction rejects outstanding
tickets/consumer roots and requires complete reclamation. The owner must ensure
there are no concurrent external callers during destruction.

Work and completion condition variables provide wakeups with bounded stored state.
The owner-side timed wait uses the POSIX realtime clock and is diagnostic, not an
HTTP deadline contract; clock changes can affect it. Worker callbacks cannot wait
on their own pool through this API. Other arbitrary blocking can still starve a
fixed-size pool. No preemption or work stealing is claimed.

## Correctness evidence

[Optimized and fail-fast UBSan runs](validation.json) pass six test groups:

- Full admission preserves caller outputs; two callbacks can be active while
  owner collection runs. Shared capture mutations are visible through both result
  handles; releasing the original consumer root does not lose queued captures.
- Collection retains newly allocated reference payloads before settlement and
  after consumer adoption. Tagged scalar/reference and scalar-only results survive.
  These are native test payloads, not qualification of all Raven union layouts.
- Queued/active cancellation, empty close, repeat close, reserved completion drain
  and joined teardown leave no roots or arena extent.
- Consumer-root saturation retains ready results; foreign pool, stale generation
  and wrong handle domain are rejected without partial outputs.
- One hundred completion/cancellation contests reconcile the winning operation
  with exactly one observed terminal outcome.
- Allocation exhaustion reports callback failure; later work still completes.

This covers manual access/root intervals. It is not proof that generated-code
roots, shared statics, runtime services or arbitrary callback captures are safe.

The [TSan attempt](tsan-incomplete.json) exits with SIGSEGV without diagnostics.
An independent minimal [pthread control](tsan-control.c) has the same
[result](tsan-control.json). This suggests a sanitizer/toolchain/host problem,
but its cause is unresolved and no TSan success/race-freedom claim is made.
That attempted build predates final cleanup/recovery-test additions; hashes retain
the precise identity. ASan was not repeated after the preceding ownership probe's
timeout. Windows support is not implemented.

## Parallel compute measurement

[Uninstrumented samples and build commands](measurement.json) execute eight
identical 20-million-iteration integer jobs with 1/2/4 workers, then reverse order.
Each job reads a rooted scalar under the gate, computes on a copied primitive
outside it, and publishes a checked scalar result. Pool creation, expected-result
calculation, close and destruction are outside timing; submission/completion waits
are inside. Managed arena capacity stays 8 KiB. Additional workers have additional
native stacks/resources, not included in that arena budget.

| Trial | Workers | Wall seconds | CPU seconds | CPU/wall |
| --- | ---: | ---: | ---: | ---: |
| 1 | 1 | 0.3206 | 0.3203 | 1.00 |
| 2 | 2 | 0.1619 | 0.3226 | 1.99 |
| 3 | 4 | 0.0881 | 0.3385 | 3.84 |
| 4 | 4 | 0.0895 | 0.3414 | 3.82 |
| 5 | 2 | 0.1611 | 0.3220 | 2.00 |
| 6 | 1 | 0.3189 | 0.3190 | 1.00 |

These short local samples demonstrate CPU overlap, not HTTP throughput or a
stable production speedup. Total process CPU divided by elapsed time exceeds one
with multiple workers; the table is generated from retained JSON. No profiler or
sanitizer ran concurrently with these measurements. Desktop contention and small
sample size remain limitations.

Reproduce optimized/UBSan tests with validate.py --output FRESH using the same
Xcode 26.2 SDK described in the preceding ownership experiment. Optional
--thread-sanitizer and --address-sanitizer retain unsuccessful outcomes.
Run measure.py --validation VALIDATION_JSON --output FRESH afterward, with no
other builds/tests active.

## Channels and next integration gate

The author identified channels as a possible later safe communication abstraction.
The internal bounded submission/completion mailboxes establish useful ingredients:
backpressure, wakeups, lifetime retention, settlement, close and cancellation.
A public Channel<T> remains unimplemented and undesigned.

This prototype shares payload identity; a channel would still need to specify
sharing versus copying/ownership transfer, producer/consumer counts, draining,
close errors and cancellation. Sending an object reference does not make later
concurrent mutations safe. Rooting guarantees lifetime, not general synchronization.

Next qualify generated-code participant/safepoint integration and task publication
against this serial reference before removing the managed-access gate. Shared
statics, task queues, resources and fault contexts require explicit ownership;
do not route existing HTTP delegates into this pool yet. Only after those gates
pass should handler-only versus whole-connection distribution be measured.
No HTTP speedup is inferred from the primitive-compute result.

## Generated-code boundary qualification — 2026-10-23

[Generated.rvn](Generated.rvn) and [generated-host.c](generated-host.c) now exercise
compiled Raven through the existing invocation ABI on all four pool threads.
Each callback owns a separate 2 KiB arena and context on its native stack. It reads
only a copied integer from the pool and publishes only a scalar completion. No
managed reference crosses between these invocation heaps or into the pool heap.
This is a qualification harness, not a new runtime dispatcher or Task binding.

Eight batches rendezvous four callbacks before executing, guaranteeing participation
by every long-lived worker. Each callback invokes the same generated entry 20 times,
alternating invalid-cast faults and successful calls in the same context. Every call
allocates at least 501 objects, forces collection, and retains the original boxed
input across allocation pressure. The host checks result/fault isolation, unchanged
output on failure, empty root stacks, complete post-invocation reclamation and an
arena boundary sentinel. A thread-local visit count proves worker reuse over all
eight batches. The owner also collects the separate pool heap while jobs are pending.

[generated-validation.json](generated-validation.json) records 640 invocations per
build (1,280 total), passing optimized and UBSan host/runtime builds on macOS ARM64.
The generated machine code itself is not UBSan-instrumented. The report records
compiler, codegen, library, source and output identities; large codegen diagnostics
remain in the local output directory with a recorded digest. Prior TSan/ASan
limitations still apply. There is no throughput measurement in this test.

Reproduce with generated.py --bundle BUNDLE --libraries MATCHING_LIBRARIES
--validation POOL_VALIDATION_JSON --output FRESH, using the same Xcode SDK as above.
The validation argument supplies the successful native build recipe. Raven source
is compiled afresh, its exact native function name is discovered from metadata,
and the existing codegen invocation ABI is linked with the canonical runtime.
No compiler semantics or CLI bridge representation changed.

This result narrows the risk to shared execution ownership: private-context
allocation, fault cleanup and TLS reuse work in this fixture. Shared captures,
shared statics, resources, Task settlement, suspended stacks and HTTP/socket scopes
remain unqualified. Unlike the intended .NET-style shared Task model, these heaps
cannot exchange references. Do not substitute this isolation model silently for
existing Task.Run semantics. The next integration step must publish participant
roots to shared collection and preserve one logical owner for suspended I/O before
routing existing HTTP callbacks to workers. Public channels remain a future design.

## Shared participant roots — 2026-10-23

The serial reference now includes eight fixed temporary-root slots per running
job. Store/load require that job's managed-access gate. Store validates an exact
allocation base in this pool (or null for release), copies the reference into
pool-owned storage and rejects invalid indices/pointers without replacing a root.
No pointer to a worker stack slot escapes. Owner collection includes every job's
published roots while the gate excludes heap mutation. Settlement clears temporary
roots on success, callback failure and cooperative cancellation; result publication
retains its own root before temporary ownership is released.

This supplies a bounded native participant boundary, not arbitrary generated stack
registration. Callbacks must publish all live temporary references before leaving,
then reload/dereference only after re-entering. Registration costs no per-object
allocation: it adds eight pointer slots per job (1 KiB across the fixed 16-job
storage on this 64-bit target) and up to 128 root entries per collection. The
collector's temporary root description array also grows by 2 KiB on the owner's
stack. These costs are deliberate correctness overhead, not an allocation or
throughput improvement. Slot count remains a private experimental limit.

[participant-validation.json](participant-validation.json) passes all seven test
groups in optimized and UBSan builds. Four callbacks publish 32 newly allocated
temporaries, all unreachable from captures/results, and pause while the owner
collects three times. On return each slot is checked for value and a still-valid
allocation base. Invalid stack/interior pointers and indices preserve prior roots.
The test repeats for success/failure/cancellation, checks result adoption across
collection, clean job reuse and full final reclamation. Existing saturation,
shutdown and settlement-contest tests also pass.

[participant-generated-validation.json](participant-generated-validation.json)
repeats all 1,280 private-context generated invocations against the updated pool.
The independent heaps remain separate; this does not qualify generated callbacks
against shared pool storage. The copied-primitive performance probe is repeated in
[participant-measurement.json](participant-measurement.json); it does not exercise
collection or HTTP and cannot measure the added collector root-scan cost.

.NET-style shared Task captures remain the intended contract. This explicit root
boundary protects lifetime while callbacks yield the gate; it neither makes
concurrent mutations safe nor replaces a compiler safepoint protocol. Next attach
complete generated root snapshots and task settlement through an explicit runtime
contract, including fault/static/resource ownership, before dispatching HTTP.

## Synchronous shared generated callbacks — 2026-10-23

This records the initial bridge checkpoint. Its pool-wide fault poisoning is
superseded by the job-local fault ownership below; historical evidence is retained.

The private runtime now bridges the existing generated bootstrap and callback
exports into a shared pool heap. Owner-only bootstrap is admitted once on an empty
pool, adopts the returned function descriptor into a pool root, and releases the
owner thread's temporary host root. Both supplied exports must belong to the same
compiled image. No compiler emission or CLI bridge encoding changed.

A worker invokes its captured descriptor under the heap gate. The adapter publishes
all consumer, capture, result and participant root slots in a temporary parked
frame on that worker's thread, creates a temporary thread-local callback handle,
and invokes the generated callback without resetting the heap. Existing compiler
snapshots and allocation retries can now collect safely: the parked frame supplies
the other jobs' roots. On return the adapter removes its frame and handle and
checks that generated execution left no stack frames, parked stacks, host roots or
resources behind. Violating this trusted synchronous contract fails fast; it is
not a general sandbox or async adapter.

Shared statics remain attached to the pool context. A guest fault is retained and
blocks further generated invocations; recovery is not silently inferred. After
joined workers and explicit draining of all jobs/consumer roots, destruction drops
static/intern/fault ownership and collects the remaining heap. Cancellation remains
cooperative at job settlement; it cannot interrupt a generated invocation holding
the gate. Socket scopes, async fibers, resources and Task/Promise are unsupported.

[Shared.rvn](Shared.rvn) closes over an immutable reference to a mutable counter
(the current compiler rejects direct mutable-local capture). Four pool workers
invoke the same closure 80 times, each allocating 500 boxes and forcing collection.
The 81st invocation faults, proving the counter was shared rather than copied;
seven later invocations are rejected. Every callback first publishes a separate
temporary, with all four participants rendezvousing before invocation. Pool roots,
paused participant roots and completion roots survive generated/owner collection.
Tests also reject a non-function capture, an uninitialized image, repeated/nonempty
bootstrap, and verify output preservation and teardown after bootstrap failure.

[shared-validation.json](shared-validation.json) records optimized and UBSan
host/runtime passes; generated machine code itself is not sanitizer-instrumented.
[bridge-validation.json](bridge-validation.json) repeats all seven native pool test
groups. [bridge-private-validation.json](bridge-private-validation.json) repeats
the 1,280 private-context generated invocations against this bridge.
Reproduce shared validation with generated.py --shared and the same bundle,
libraries, validation recipe and fresh output arguments used above.

This is the serial correctness reference for .NET-style shared capture identity.
It does not provide parallel managed execution or an HTTP speedup. Each invocation
adds temporary root registration and scans the bounded pool root set during GC;
we have not optimized that overhead. Next separate per-invocation fault ownership,
qualify task publication and suspended I/O ownership, then introduce coordinated
mutator safepoints before lifting the heap gate. Do not dispatch current HTTP
handlers through this synchronous-only bridge.

## Job-local callback faults — 2026-10-23

Generated callback faults now belong to their job. Before releasing the heap gate,
the bridge copies the fault header and frames into fixed job storage, then clears
the shared context's diagnostic state. The same job cannot invoke again after its
fault, but other jobs can use the shared closure/heap. A guest fault forces failed
settlement even when the native adapter returns success; cancellation that wins
before settlement takes precedence and discards diagnostics. Mutations performed
before a fault are not rolled back. Bootstrap failures still prevent use of that
image/context; there is no general recovery promise for corruption or native crashes.

The collector traces message/function-name pointers for outstanding job faults,
including during subsequent generated invocations. Pending diagnostics stay private.
For a terminal ticket, fault_copy copies metadata and UTF-8 message bytes into
caller storage under the gate; it never exports managed pointers. A too-small
buffer, invalid ticket or pending job preserves outputs. Callers choose a buffer
capacity; a successful no-fault read reports zero code/length. Take releases all
fault ownership, and stale tickets cannot read a recycled job's diagnostics.

Fixed fault storage adds 1,048 bytes per job (16,768 bytes across the 16-slot pool),
with no per-fault native allocation. The worst-case collector slot array grows by
16,640 stack bytes; actual scans include only active fault messages and recorded
frames, so successful jobs add no diagnostic slots. These are provisional costs
for a correctness baseline, not a claim of improved allocation or throughput.

[fault-shared-validation.json](fault-shared-validation.json) passes optimized and
UBSan runtime/host builds with 87 successful invocations and one isolated failure
on the same shared closure. The failure constructs its message at runtime. Owner
collection and subsequent callbacks preserve that text until copying/take. Tests
check same-job reinvocation rejection, forced failed settlement, insufficient-buffer
output preservation, no stale access, successful job reuse and full teardown.
A separate synchronized test holds a faulted job before settlement, collects,
cancels it, and verifies a cancelled outcome with no retained diagnostic.
[fault-pool-validation.json](fault-pool-validation.json) repeats seven native groups;
[fault-private-validation.json](fault-private-validation.json) repeats 1,280 private
context invocations. Generated machine code is not UBSan-instrumented. Earlier
TSan/ASan limitations remain unchanged.

This closes synchronous job fault isolation only. Exceptions in .NET are normally
attached to a Task's failure rather than poisoning unrelated Tasks; this private
bridge follows that ownership direction but does not yet implement public Task
exception semantics. Native status and copied diagnostic bytes are provisional
adapter contracts. Async Task/Promise publication, suspended I/O ownership and
parallel mutator coordination remain the next boundaries before HTTP dispatch.

## Owner-side completion delivery — 2026-10-23

The private pool now has a bounded owner-side dispatch operation. Workers publish
a settlement sequence under the heap gate; dispatch selects completed, unnotified
tickets in that order and calls a native consumer after releasing the gate.
Notification is at most once per ticket generation. Results/faults remain rooted
and the slot remains occupied until explicit take. A consumer can therefore copy
diagnostics, adopt a result, collect, or submit follow-up work without running
those actions inside the producer's critical section.

Dispatch processes at most the requested 0–16 notifications, including newly ready
work if budget remains. No background thread, extra queue allocation, or implicit
consumer scheduling is added. Nested dispatch and non-owner calls return -1.
Destroy is rejected while a dispatch callback is active, even if it closes/drains
the pool. Closing does not discard completed tickets: the owner may still dispatch
and consume them. Existing polling take remains supported and suppresses a later
notification. An external concurrent take may invalidate a notified ticket; use one
consumer when notification and consumption must stay together. Notification alone
never bypasses consumer-root backpressure or promises automatic retry.

[delivery-validation.json](delivery-validation.json) passes eight optimized/UBSan
test groups. The new group verifies settlement order differing from submission,
strict budgets, no duplicate notifications, retained tickets, owner identity,
reentrant submission, collection/take outside the gate, and close/destroy lifetime.
[delivery-shared-validation.json](delivery-shared-validation.json) routes all 88
compiled Raven job completions (87 successes and one isolated fault) through this
pump in two-notification batches. The consumer checks diagnostics and adopts/releases
result roots under collection pressure. Cancellation/fault tests still pass.
[delivery-private-validation.json](delivery-private-validation.json) repeats 1,280
private-context generated invocations. UBSan instruments the runtime/host, not the
generated machine code; prior TSan/ASan limitations remain.

The added per-job fields are a completion sequence and delivered flag, plus pool
settlement/dispatch state. Selection scans the fixed at-most-16 slots, trading a
small bounded scan for no new queue nodes. This is not an HTTP throughput claim.
The copied-primitive probe is repeated in
[delivery-measurement.json](delivery-measurement.json); it exercises ordinary take,
not dispatch or async continuation scheduling.

The .NET-oriented aim remains deferred continuation execution after synchronized
result publication. Current Raven Promise/TaskQueue fields and native async wakeups
are owner-local; this pump establishes a native delivery boundary without declaring
them thread-safe. It does not resume a generated continuation, bind Task.Run, add
public channel APIs or migrate fibers. Next connect an explicitly owned runtime
completion signal to this pump and qualify owner wakeups/teardown before HTTP use.

## Shared graph GC gate — 2026-10-23

The author clarified that collection while objects are accessed across workers is
the correctness gate before throughput work, with server primitives rather than a
full web framework as the scope. `Shared.rvn` now retains an aliased cyclic graph
through its single shared closure. Every invocation checks the previous worker's
mutation and reference identity, replaces an edge, then allocates 500 boxes while
retaining both the displaced child and replacement through collection. Four native
participants rendezvous each batch, so four distinct workers access that same
closure. Owner collections also run between batches and while consuming results.

[Optimized/UBSan evidence](graph-validation.json) passes 87 successful invocations
and one isolated fault per build, preserving native participant temporaries and
fault text. After releasing the closure and sentinel roots, an explicit collection
reports zero occupied extent, including reclamation of cycles. Managed access
remains serialized by the heap gate; this does not qualify concurrent mutators,
shared async Task settlement or off-thread use of existing I/O resources.

The separate [HTTP integration](../http-worker-integration/README.md) runs complete
application owners on isolated heaps. It supplies working HTTP/async-I/O integration
and a multicore comparison, not evidence of shared-object GC safety.

## Concurrent native read phases — 2026-10-23

The next private correctness step permits multiple native callbacks to read the
same rooted capture graph at once. `neoclr_worker_read_begin` returns the capture;
`read_end` ends that thread's phase. The caller must not mutate, allocate, call
other pool APIs, invoke generated code, or retain pointers beyond the phase.
Blocking on another pool operation or reader admission inside a phase can deadlock
and is prohibited. Cancellation does not revoke a phase or its roots.

Mutation, allocation (including root creation), bootstrap and collection wait for
all readers. Waiting writers prevent new readers entering, so fresh readers cannot
indefinitely postpone GC. Metadata operations keep their existing mutex and can
still cancel jobs or release consumer roots while readers are active. Job captures
retain the graph until callback settlement. Close remains cooperative and joins
callbacks; an application must eventually end every phase.

This is a runtime-owned native synchronization primitive because it coordinates
arena lifetime and root publication. It is not a new managed public API or a
replacement for .NET's transparent managed-thread/GC coordination. The benefit is
a small testable boundary for actual overlapping object reads; the cost is explicit
native discipline, two synchronization operations per phase, and GC latency bounded
by how soon all readers exit. Generated callbacks still require exclusive access,
and concurrent managed writers/compiler safepoints remain open. Existing design
research is linked in the [threading comparison](../../design/native-http-worker-threads.md).

[Native validation](read-phases-validation.json) passes nine test groups in both
optimized and UBSan builds. The added group performs 25 rounds with four overlapping
readers. A native controller requires each reader to make progress while the owner
requests collection, then releases them. The fixture checks that collection never
finishes while a reader holds the object, serialized writes appear in the next
round, cancellation and external-root release preserve active captures, and the
final heap extent is zero. The existing generated shared graph test also
[passes both builds](read-phases-shared-validation.json).

The isolated HTTP listener group [passes optimized/UBSan](read-phases-http-native-validation.json).
The [relinked HTTP host](read-phases-http-relink.json) reuses the previously qualified
application object with current native sources; [240 responses plus pending-read
shutdown and guest-fault exit](read-phases-http-validation.json) pass. These checks
establish regression coverage, not a new throughput result. TSan remains unqualified.
