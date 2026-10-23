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
There is no compiler safepoint integration, automatic collection on allocation
pressure, stack/fiber migration or concurrent Promise support. Existing TLS
host-root/socket/fiber APIs must not be used on the pool context. Callbacks may
not retain and dereference unregistered managed pointers outside access intervals.

Capture roots are adopted before job publication. Workers publish result reference
slots before releasing managed access, including while the callback remains
unsettled. Temporary references can be explicitly published to job-local slots
before leaving the gate. Owner collection traces all temporary/capture/result/consumer slots using the
canonical collector. Workers may allocate under the gate but cannot initiate
collection or invoke ordinary runtime GC hooks. Thread-local allocation counters
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
