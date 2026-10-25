# Stable participant roots at the worker rendezvous

2026-10-25. Add private root-value capture and an opt-in integration into the existing
compiled worker pool. This is a synchronous equivalence step toward concurrent
shared-heap marking; it does not enable the concurrent collector on shared heaps.

## Contracts and service ownership

The GC service owns private `runtime/native/gc/participant-roots.[ch]`. Root capture
is now independent of `NEOCLR_WORKER_GC`; collector hooks still require that flag. The worker pool owns its participant
rendezvous and buffer lifetime. Neither service owns the other's state. There is
no managed API, object-layout, compiler-emission or Runtime Contract change.

Capture requires valid, quiescent source contexts, frame chains, descriptors and
slot/lane values for the entire call. Count validates metadata without reading
slot/lane values. Capture validates capacity before writing and copies conservative
64-bit values into caller-owned storage. It retains no frame, context or descriptor
pointer. Four-byte discriminator slots are excluded; borrow addresses are copied
as values, never followed. Fault roots use the existing runtime enumerator. Static
storage anchors are included for each participant; duplicate anchors are permitted.

The helper rejects foreign arenas, mismatched frame contexts, cyclic frame chains,
malformed slot widths/flags/addresses, missing lanes, invalid fault frames, excessive
participant count and insufficient capacity. Nonoverlapping output and valid native
input pointers remain caller obligations; this is not memory protection against
arbitrary native pointers. No helper allocation, root publication or guest execution
occurs. Outputs change only on success under the required quiescence contract.

Foreign TLS handles, resources and parked task roots are not enumerated here. The
compiled pool's existing worker-root checks still reject unsupported foreign state;
pool capture/local/result roots are supplied by `pool_slots`. Broader worker modes
must establish their own complete root contract before using this helper.

This follows the existing research distinction between published root metadata and
changing root values. The [.NET GC design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/garbage-collection.md)
coordinates suspension around required phases of background collection; immutable
root-value capture is neoCLR's experimental implementation choice, not a claim to
copy CLR internals or to eliminate final root synchronization.

## Compiled-pool integration

`NEOCLR_WORKER_ROOT_SNAPSHOT` selects capture at `compiled_collect_locked`, while the
exclusive execution lease still keeps all other participants paused. Link
`participant-roots.c` and enable `NEOCLR_WORKER_GC` consistently. The snapshot includes
the invoking worker's frames, other attached participants and pool roots. Collection
uses the copied lane buffer, then frees it before releasing the lease. The original
participant-descriptor collector remains the default/control.

The first integration run rejected a registered worker with no heap arena yet.
A diagnostic confirmed a NULL arena and empty frame chain: it was waiting for its
first exclusive lease. The pool now explicitly records first attachment. It omits
only never-attached invocation contexts, asserting that they have no frames, arena,
static storage or fault; their captures/locals remain represented in pool roots.
The helper's foreign-arena rejection is preserved. Attached participants never use
that omission path.

This version deliberately adds one native buffer allocation per coordinated
collection and repeated metadata passes for sizing/validation/copy. Allocation
failure or sizing overflow fails collection without entering the collector; the
buffer is freed after capture/collection failure or success. Allocator-failure
injection and reusable-buffer optimization remain open. No allocation, CPU,
throughput or pause improvement is claimed.

## Validation

- Ordinary and UB/bounds-sanitized root tests each perform 400 actual worker-stack
  captures and 6,400 native-collector live/dead checks. Four workers expose storage,
  transient, lane, fault and static roots plus cross-worker object edges. After
  capture, they overwrite and return from their source stacks. The real native
  collector retains the expected objects through copied roots, excludes a merely
  borrowed pointee, and reclaims everything after removing the snapshot.
- Malformed inputs and insufficient capacity are rejected without publishing output.
  Deliberately zeroing copied roots or following a borrow address makes the live/dead
  test fail, demonstrating sensitivity to both missing and spurious retention.
- The generated Raven parallel worker sample is rebuilt from retained inputs with
  the current explicitly recorded code generator. Four native executables cover
  snapshot/control and optimized/UB-sanitized modes. Each passes graph/GC and fault
  isolation (32 jobs per cohort), cancellation, capture release, result lifetime and
  cleanup. Each performs 24,992 coordinated collections; graph execution reaches
  four active participants. These are synchronous collections, not concurrent epochs.
- An initial harness incorrectly expected an object hash in the older manifest and
  stopped before execution. The final driver rebuilds and hashes the object and
  compiler inputs instead. The subsequent waiter rejection and diagnostic remain
  archived separately from final passing evidence.

This does not qualify JIT worker integration, native concurrent marking of this
snapshot, root acquisition after resume, allocation/reclamation overlap, or complete
HTTP worker handling. The earlier TSan test/control startup failures remain an open
gap; no new TSan/ASan claim is made. Source worker stacks returning is a targeted
lifetime test, not proof of every scheduling interleaving.

## Compiled allocation routing follow-up

The [owner allocation service](allocation-owner.md) now routes actual generated
worker allocations through a persistent service under the existing exclusive
lease. Direct/copied-root controls and shutdown/fault tests pass; collection is
still synchronous. Measured per-allocation routing remains more expensive than
local allocation, so this is opt-in integration scaffolding, not a default choice.

## Next integration boundary

The [native live-epoch integration](../concurrent-worker-epoch/README.md) now connects
copied roots and registered object writes with owner-only allocation and joined
mutators. Persistent compiled-worker admission remains open.

Connect stable initial roots and the [registered writer protocol](../concurrent-mutation-gate/workers-results.md)
to one shared epoch while preserving allocation identity and payload lifetime.
Before resuming mutators, define how new allocations and newly introduced roots
remain visible. Final remark must rendezvous again, copy current roots and drain
mutation logs, then join the marker before exclusive reclamation. Cancellation,
overflow, worker exit and faults must revoke epoch authority before recovery.
Shared-heap concurrent admission stays disabled until that complete protocol is tested.

## Reproduction

```sh
python3 benchmarks/execution/concurrent-participant-roots/validate.py \
  --output target/participant-roots-final
python3 benchmarks/execution/concurrent-participant-roots/validate-pool.py \
  --output target/participant-roots-pool-final
```

Choose fresh directories for repeat runs. `validation.json.gz`, `pool-validation.json.gz`
and `negative.json.gz` retain commands, outputs and hashes; the compressed negative
driver describes deliberate faults. The final pool recipe needs the retained Raven
assembly, bundle modules and native code generator listed in its manifest.
`manifest-initial.json.gz`, `pool-initial.json.gz` and `waiter-diagnostic.json.gz`
retain failed setup/admission evidence. Root tests and pool tests use matching private
native contracts; no published platform default changes.
