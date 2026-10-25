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
and joins every mutator, then finishes marking and reclaims on the owning thread.
The source root frames have returned before finish. Descriptors are trusted native
data, not roots or a protection boundary against arbitrary native code. No token
or descriptor may be used after finish disposes the gate. Cancellation similarly
requires all users to stop/join before disposal and synchronous recovery.

The private root-capture service is now independent of `NEOCLR_WORKER_GC`; linking
it does not install worker collector hooks. The live generator still rejects that
shared-heap configuration. `NEOCLR_GATE_WORKERS` selects the new experimental API;
without it, the appended helpers compile out. Target creation currently accepts
only ordinary objects. Arrays and grouped element/marker updates need separate
worker qualification; this introduces no new managed record semantics.

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

Two ordinary negative controls zero deletion or insertion log values while keeping
log counts/overflow behavior intact. Both abort at the final object-liveness
assertion. Thus successful overflow tests cannot mask an ineffective barrier.
These are sensitivity tests, not proof of all thread interleavings.

`root-validation.json.gz` repeats the existing copied-root safety tests with the
shared-worker macro enabled. `owner-validation.json.gz` repeats live owner-only
collector/store/array controls with allocation indexes enabled. The initial owner
validator stopped after all positive cases because it assumed exactly two logging
sites; adding the worker implementation made four. It now explicitly selects the
owner implementation for its deliberate fault. The initial report is retained as
`owner-initial.json.gz`, not counted as a passing validation run.

TSan remains unqualified after the previously recorded test and minimal pthread
control startup failures. No new ASan/TSan, JIT, AOT application, HTTP performance,
pause, allocation reduction or default-collector claim is made by this test.

## Remaining integration and measurement

Connect the protocol to generated worker execution without crossing service
ownership: allocation handoff, provider store emission, all managed payload forms,
complete suspended/task/resource roots, scheduler rendezvous and fault/exit cleanup
remain to be qualified. The test's owner-only allocation and joined native mutators
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
