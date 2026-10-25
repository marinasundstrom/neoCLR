# Live concurrent epochs with native mutators

2026-10-25. This connects the live heap marker, registered writer gate and copied
participant roots in an explicit native integration test. It does **not** admit
compiled shared-worker pools or HTTP workers to concurrent collection.

## Ownership and scope

The owner captures initial root values while four mutators are paused, begins an
epoch, and issues immutable native target descriptors containing validated object
bounds. Registered threads mutate ordinary object fields through logged grouped
writes. They never consult the owner's TLS allocation index. Only the owner
allocates; no participant reclaims objects during the epoch. New owner allocations
are handled by the existing final new-object closure.

At final rendezvous the owner copies current roots, revokes registrations, releases
every mutator and waits for invocation completion, then finishes marking and
reclaims on the owning thread. Completion is proven by joining temporary workers
or by a synchronized acknowledgement after returning from `mutate` on persistent
workers.
The source root frames have returned before finish. Persistent dispatchers retain
no outstanding job and perform no further epoch access after acknowledging return. Descriptors are trusted native
data, not roots or a protection boundary against arbitrary native code. No token
or descriptor may be used after finish disposes the gate. Cancellation similarly
requires all users to complete before disposal and synchronous recovery.

The private root-capture service is now independent of `NEOCLR_WORKER_GC`; linking
it does not install worker collector hooks. The live generator still rejects that
shared-heap configuration. `NEOCLR_GATE_WORKERS` selects the new experimental API;
without it, the appended helpers compile out. Target creation accepts ordinary objects and existing reference/value array storage.
Array shape headers remain immutable; element and initialization-marker writes
share one logged transaction. Text, byte and intern payloads remain rejected by
this helper. This introduces no new managed record semantics.

This reuses the [.NET background-collection research and root-service comparison](../concurrent-participant-roots/README.md#contracts-and-service-ownership).
The current protocol preserves explicit root rendezvous and owner reclamation;
it does not copy CLR internals or claim equivalent thread/allocator capabilities.

## Evidence

`validation.json.gz` retains commands, input hashes and results. Ordinary and
UB/bounds-sanitized builds each pass:

- 100 successful live epochs with four native mutators per epoch and 2,900
  expected live/dead checks, then complete reclamation after roots are cleared.
- Deletion before a deliberately delayed marker, insertion after the marker goes
  idle, new owner allocations during the mutation phase, and changed current roots.
- Rejection of foreign-thread descriptor creation and writes to object identity.
- Ten bounded-log overflow cycles: the cancelling write has no partial effect,
  later writes remain cancelled, and synchronous recovery retains the current graph.

Two object negative controls zero deletion or insertion log values while keeping
log counts/overflow behavior intact. Both abort at the final object-liveness
assertion. Thus successful overflow tests cannot mask an ineffective barrier.
These are sensitivity tests, not proof of all thread interleavings.

The subsequent array qualification (`array-validation.json.gz`) adds 300 epochs
per ordinary/sanitized build across plain reference, reserved reference and two-lane
value arrays. It checks delayed deletion, late insertion, descendant liveness,
header protection, all-or-nothing rejection of an out-of-bounds grouped patch and
complete reclamation. Six additional negative controls (both logs, each layout)
fail at the expected liveness assertion. Ordinary object/overflow cases also rerun.

The first array harness retained the array as a current root. Its missing-insertion
control unexpectedly passed because final tracing found the new edge. The corrected
test removes that current root before finish and relies on the initial snapshot to
retain the array. The failed sensitivity result is archived in
`array-initial.json.gz`; it is not counted as qualified evidence.

`root-validation.json.gz` repeats the existing copied-root safety tests with the
shared-worker macro enabled. `owner-validation.json.gz` repeats live owner-only
collector/store/array controls with allocation indexes enabled. The initial owner
validator stopped after all positive cases because it assumed exactly two logging
sites; adding the worker implementation made four. It now explicitly selects the
owner implementation for its deliberate fault. The initial report is retained as
`owner-initial.json.gz`, not counted as a passing validation run.


`persistent-validation.json.gz` additionally keeps four native dispatcher threads
alive across 100 epochs in each ordinary/UB-bounds build. Every epoch retains the
same initial/current root, liveness, revocation and teardown checks. The owner
waits for acknowledged invocation return before disposing the gate or reclaiming;
only after all epochs does it join the dispatcher threads. Both deliberately
missing logs also fail on this persistent path. Temporary-worker and array tests
rerun alongside it. Acknowledgement replaces thread destruction, not quiescence.
No thread is permitted to reuse the old descriptor after acknowledging completion.

This selects a fixed collector owner for the native integration experiment. It
avoids transferring TLS allocator/epoch state while establishing reusable worker
lifetime. It does not settle the final managed-pool design: workers still receive
owner-allocated inputs, and allocation requests from generated execution, complete
resource roots, persistent-worker fault/cancellation and scheduler routing remain
unqualified. No throughput improvement follows from this correctness test alone.

## Owner-serviced allocation requests

The private `NEOCLR_GC_ALLOCATION_DISPATCH` build option adds a thread-local adapter
at the existing `neoclr_gc_allocate_v1` entry, before it inspects arena metadata.
Absent the option, there is no hook branch. With no installed callback, the existing
allocator runs unchanged. The adapter is an internal service seam, not a new
managed allocation API or permission to share an arena without coordination.

The callback must preserve non-collecting allocation, zeroed successful storage,
error codes and output publication only on success. On success it also returns the
authoritative used extent for requester-local peak profiling. It must coordinate arena
identity/lifetime and allocator/sweep access. It must not recursively dispatch to
itself. The test uses one bounded request slot per worker and a fixed owner using the private non-dispatching
storage primitive. Workers use empty local arenas as request identities, ensuring they
cannot accidentally allocate against copied owner metadata. Requests contain size
and storage kind; the owner selects its authoritative arena. Mutex/condition
handoffs publish the result; the worker initializes it before final rendezvous.

`allocation-validation.json.gz` adds 100 epochs per ordinary/UB-bounds build with
four workers, 1,600 requests, 800 successful allocations and 1,200 live-object
checks. Oversized and invalid-kind requests preserve outputs; success returns
zeroed objects, worker publication through the registered barrier succeeds, and
removing roots reclaims everything. Bypassing dispatch fails at the successful
allocation assertion. Several workers may report that same assertion before
abort; the verifier requires one to four copies and rejects any other diagnostic.
The initial overly strict verifier report is retained in `allocation-initial.json.gz`.
Existing temporary/persistent mutator and array tests, including negative controls,
also rerun with the current source.

This follows the previously recorded .NET comparison at the internal allocation
service layer while retaining the existing managed contracts. Routing requests to
one owner is a provisional neoCLR implementation choice: it avoids migrating TLS
allocator/epoch state, but adds synchronization and can serialize allocation.
The test services workers sequentially; it is not a fairness or throughput design.
No per-request native message allocation is needed in this fixture, but no
application allocation/CPU improvement is established.

The accounting follow-up (`allocation-accounting-validation.json.gz`) corrects a
pacing issue in the initial seam: physically allocating on the service thread also
attributed the allocation to that thread. The requester now counts exactly one
successful allocation, its bytes and the returned arena extent. The owner calls
`neoclr_gc_allocate_dispatch_storage_v1`, which retains all allocation validation,
zeroing, free-list/index updates and output semantics but neither dispatches nor
counts. Failure counts no successful allocation. This is a private experimental
callback signature change; rebuild adapters with the matching header.

Each test worker verifies two allocations/48 bytes and a nonzero reported extent;
the owner verifies its own allocation/byte counters do not change when serving
requests. Omitting the requester count fails deliberately. Ordinary and
UB/bounds-sanitized full native integration fixtures pass. Clang ARM64 assembly
with the dispatch option disabled remains byte-identical to `4701e799c`, both with
and without profiling (`allocation-accounting-assembly.json.gz`). These facts
protect existing pacing/measurement behavior; they do not prove scheduling or
throughput gains. The service primitive must only serve a corresponding public
requester; using it for an unrelated allocation would bypass accounting.

The [compiled-pool adapter](../concurrent-participant-roots/allocation-owner.md) now
installs and serves this hook under its existing exclusive execution lease. Its
collector remains synchronous; the live shared-worker integration is still open. Rooted allocation pressure must
coordinate all participant roots before collection; this non-collecting seam alone
does not authorize that path. Provider intermediates, owner scheduling, cancellation,
worker faults and complete persistent dispatch lifetimes still require integration.
The live shared-heap admission rejection remains in place.

TSan remains unqualified after the previously recorded test and minimal pthread
control startup failures. No new ASan/TSan, JIT, AOT application, HTTP performance,
pause, allocation reduction or default-collector claim is made by this test.

## Remaining integration and measurement

Code inspection identifies a concrete ownership mismatch: `compiled_boundary`
transfers the pool's exclusive execution lease and context between threads, while
`candidate_active`, `shadow_owner` and the gate's owner identity belong to the
thread that began the epoch. Merely enabling `NEOCLR_WORKER_GC` would not transfer
those resources. The pool also forces collection at exclusive boundaries, so the
current scheduling policy would not preserve a long-lived concurrent epoch.

The next implementation must explicitly choose a persistent collector owner with
requests/rendezvous, or a transferable collector state with a validated handoff.
A persistent owner avoids transferring gate authority but needs scheduling and
allocation-service requests; handoff requires removing TLS identity assumptions
without weakening root/resource lifetime checks. Neither option is implemented or
accepted as a default by this checkpoint. Preserve the rejection until end-to-end
worker entry, allocation, mutation, final remark, cancellation and exit tests pass.

Connect the protocol to generated worker execution without crossing service
ownership: allocation handoff, provider store emission, all managed payload forms,
complete suspended/task/resource roots, scheduler rendezvous and fault/exit cleanup
remain to be qualified. The test's owner-only allocation and explicitly completed native invocations
are narrower than a persistent managed worker pool.

After that integration, compare synchronous and concurrent collectors using the
existing tasks, collections, arrays, plaintext and JSON consumers, in JIT/AOT and
relevant interpreter controls. Track throughput, process CPU per operation and
managed/native allocation volume together, with pause and memory correctness
checks. Optimize measured common costs before assuming HTTP-specific causes.

```sh
python3 benchmarks/execution/concurrent-worker-epoch/validate.py --output target/worker-epoch-repeat
```

Use a fresh directory. The driver also builds and runs both deliberate-fault cases.
