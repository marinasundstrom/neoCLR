# Existing GC walker on an isolated worker snapshot

2026-10-18. Proof of concept only: this reuses the existing native mark worklist,
address index and conditional-root fixed point on worker-private memory. The
[validated one-job worker](../concurrent-gc-worker/README.md) supplies submission,
publication, cancellation and joined teardown. Production sources and `--gc`
choices are unchanged. The POC now applies accepted results through owner-side
cleanup/sweep; see the [reclamation checkpoint](#live-reclamation-checkpoint).

## Integration boundary

`run.py` checks the production source hash and generates a private copy with small
experimental hooks. The ordinary synchronous path remains the comparison oracle.
Hooks translate candidate addresses into snapshot storage, stop before cleanup/sweep,
record conditional retention and check cancellation between worklist operations.
They do not replace the production graph walker with another tracing algorithm.

Capture runs while the owner is quiescent. It copies the heap (at most 1 MiB), context,
active frame storage/transient/stack words and host roots into job-owned storage.
Strong handles become ordinary captured root words. Owned handles remain conditional
owner/value pairs; **owners do not become strong roots**. Owner addresses in those
pairs refer to copied descriptors. Candidate addresses in payloads and root words
remain original numeric identities and are translated only during marking. Intern
seed pointers and conditional values originating in copied metadata are converted
back to those identities before marking. This also prevents a scalar that happens
to equal a snapshot address from becoming a false managed root.

The worker reads no original heap descriptors/payloads, stack slots or host-root list.
Its runtime TLS registrations must be empty. Copy-only marking does not invoke native
cleanup, expire live handles, sweep, or update the owner's allocation counters. Only
successful completion makes the mark results available to inspect; error/cancelled
jobs are discarded. Job storage remains alive until result pickup or joined shutdown.

Bounds: 4,096 captured frame/strong-root words and 1,024 conditional handles, with
explicit capture failure on overflow. Up to 128 parked registrations are captured with private flattened frames. Unowned
registrations remain strong; owned registrations keep a copied owner and participate
in the existing conditional fixed point. Expired registrations are skipped; unrelated
owner contexts are excluded. Parked words share the 4,096-word budget and are not
added to the unconditional frame roots. Explicit caller-supplied conditional arrays are not yet
a capture input. Fault/static roots are copied through the context and traced by the
existing walker. Registered resource owners are validated during capture, but no
resource callbacks are copied or run on the worker.

## Executable validation

With the repository Xcode SDK environment:

```
python3 docs/experiments/concurrent-gc-walker/run.py --output NEW_DIRECTORY
```

The final UBSan/bounds run passes:

- 100 deterministic 64-object graphs and one 3,000-object graph (dense-index path),
  with object edges, atomic leaves, strong/conditional host roots, interior references
  and owner-attached cleanup. Compare each original allocation's snapshot mark with
  its survival under synchronous collection, and compare conditional retention.
- Every graph comparison uses `mprotect(PROT_NONE)` on the original heap throughout
  worker execution. An accidental dereference faults. Restore access only after
  taking the worker result, then run the synchronous oracle. Cleanup occurs solely
  on the owner during synchronous collection and exactly once per resource.
- Three snapshots of an unrooted conditional owner cycle leave every member unmarked.
  Adding one independent root retains the entire chain through the fixed point,
  including atomic owners. This addresses the earlier snapshot's strong-promotion
  [counterexample](../concurrent-gc-snapshot/README.md) at the tracing stage.
- Intern roots (including seed-cache overflow), static/fault roots, initialized record
  and string-array slots, captured storage/transient roots and skipped uninitialized
  array entries match the synchronous oracle.
- Mutating the original graph and allocating after capture leaves the snapshot result
  unchanged. The current live graph can therefore have a different correct result;
  the test deliberately discards snapshot marks instead of applying them.
- Cancellation before tracing, malformed copied descriptors, successful worker reuse
  after failure, and (in the initial slice) explicit rejection of parked-frame capture.

The first fixture attempt aborted because it passed null empty-storage metadata and
an invalid spill-plan marker to the existing root-probe API. The fixture was corrected;
its failed command/input hashes are retained separately. It was not a collector pass.
Final command/input hashes and output are in `ubsan-results.json`.

ThreadSanitizer remains unavailable on this host as documented by the worker and
empty-control evidence; it was not rerun here. UBSan and protected-heap checks do not
substitute for exhaustive race detection. This is a native POSIX experiment, not
interpreter/JIT/AOT qualification. Capturing/validating roots and copying the heap are
still synchronous. Cancellation checks are between worklist operations, not every
scanned word or index step; worst-case cancellation latency has not been measured.

## Integration limits

The worker reuses the current walker without consulting live heap storage for
these captured cases. The reclamation checkpoint below supplies conservative
invalidation and owner-side application for the bounded POC. Broader runtime/native
access paths, execution modes and Serve remain qualification work. Keep the
synchronous collector as the lifetime oracle.

The existing [.NET/Go/alternative collector research](../../design/concurrent-gc-evaluation.md)
remains applicable; no managed semantics change or new collector design is selected.
No throughput, pause reduction or Go comparison is claimed by this correctness slice.
Copy cost, worker CPU, retained memory and sustained Serve behavior remain unmeasured
for this integration.


## Parked-frame capture checkpoint

`parked-results.json` records the updated passing UBSan/bounds run, including all
previous graph/root/error checks. Three additional cases compare two parked frames
with synchronous collection: unrooted owners, a rooted owner activating another
owner through its parked frame, and an unowned strong frame activating an owned
frame. Interior references are preserved. Both original heap pages and frame/slot
storage are unreadable during worker tracing. Owner-side synchronous collection
confirms retention/expiry; a second capture skips expired registrations. A 129th
registration fails capture without leaking the copied heap. These checks do not yet
qualify resumption or handle reads while the worker result is outstanding.

## Publication counterexamples

`publication-results.json` retains three additional executable counterexamples,
with all previous checks still passing under UBSan/bounds. The success of this test
means the unsafe publication assumptions were disproved; it does not qualify live
reclamation.

1. **Conditional host read:** capture an otherwise unreachable owner/value pair,
   then read its still-valid handle and publish the value as an active root before
   worker completion. The snapshot marks neither object. Synchronous collection
   retains the value and expires the owner. Owner allocation and collection counters
   are unchanged between capture and completion.
2. **Parked resume:** capture an unrooted owner with a conditionally parked frame,
   then resume that frame. The snapshot marks neither owner nor frame value.
   Synchronous collection retains the now-active value. Again, no intervening
   allocation or collection is necessary to change the required retained set.
3. **Address reuse:** capture an unrooted allocation, collect it synchronously, then
   allocate and root a replacement at the same address with the same descriptor
   shape and arena extent. Old marks would reclaim the replacement. Comparing only
   addresses, sizes, kinds and extent cannot detect this; the collection epoch changes.

These use current native handle/park/resume/allocation APIs, not invented graph
mutations. The original prototype's descriptor checks alone are therefore not a safe
publication gate. A practical next implementation should cancel/invalidate an
outstanding result before conditional-handle access and parked resumption can expose
previously unmarked objects, and invalidate it across synchronous reclamation/arena
reuse. Registration/replacement, native access and lifetime transitions still need
an audit; the examples are not an exhaustive list of invalidation points. Cancellation
must preserve job storage until worker completion, as required by the existing worker.

Do not add a generic mutation framework just to mask this issue. Start with the audited
host/root transitions and explicit cycle identity, keep cleanup/reclamation on the
owner, and test that stale results are refused. Ordinary graph changes are a separate
snapshot-at-capture correctness question; these failures specifically demonstrate
reachability emerging through conditional roots and identity reuse. At this checkpoint there was no live publication operation. The following
reclamation slice adds one and rejects these stale results.


## Live reclamation checkpoint

The bounded native POC is complete with **demonstrated arena reclamation**, not just
marking. `reclamation-results.json` records a passing UBSan/bounds run: **319 applied
cycles, 10 rejected results and 245,776 reclaimed payload bytes** through the new
owner-side apply path. These totals exclude bytes reclaimed by synchronous oracle
calls. Storage is returned to the arena allocator and reused; this is not an OS
resident-memory reduction or a throughput measurement.

`reclaim.inc` coordinates one outstanding job. Capture and result application run on
the owner; tracing runs on the persistent worker. After a successful result pickup,
the owner checks cycle identity and the original allocated descriptors, marks new
allocations as retained, imports marks for captured allocations, and runs the existing
cleanup/expiry/sweep body. The generator extracts that body without changing its
algorithm; the worker never invokes it. No synchronous retracing is hidden in apply.

### Safety argument and supported scope

- Capture occurs at a complete, quiescent root boundary. All managed references must
  be represented by captured active/parked roots, host handles, fault/static state or
  the graph reachable from them. One mutator owns the live heap. Foreign code cannot
  retain unregistered pointers, fabricate references or reset an arena behind the
  coordinator. This is a POC precondition, not a newly qualified runtime-wide API.
- The worker traces immutable captured payload/root values and writes private marks.
  Mutex publication/result pickup transfers ownership of the job; cancellation does
  not permit freeing job storage before completion/join.
- Under that root contract, ordinary reference transfers and graph changes cannot
  create a genuine reference to an old unreachable object. The exceptions exposed
  by conditional handles and parked frames are covered by invalidation hooks.
- An owner-thread monotonic epoch invalidates results on host-handle creation/read/
  replacement/release, callback reads, allocation-info queries, resource attach/release,
  park/resume/discard, entry/teardown checks and synchronous collection. Hooks run
  before effects, including failed calls. The epoch is shared across contexts on
  that owner, so unrelated transitions may discard useful work. Overflow aborts
  rather than allowing stale identity to match. Root-probe and collector source
  hashes are pinned before hook generation.
- No intervening reclamation can reuse a captured allocation identity and still pass
  the epoch check. New allocations, including ones occupying previously free spans,
  are retained for the current cycle. Later cycles reclaim them if unreachable.
- The owner applies marks only after successful result pickup, unchanged epoch,
  matching arena identity and validated original descriptors. Cancelled, failed or
  stale jobs change no live heap bytes, cleanup state or collection counters. Cleanup
  runs once on the owner through the existing sweep path; cleanup callbacks must
  obey the existing no-reentry/no-heap-mutation collection contract.

This is an invariant argument plus executable evidence for that scope, **not a formal
proof of all runtime behavior**. Arbitrary native/FFI pointer access, multiple mutators,
external arena resets, uninstrumented host paths and full interpreter/JIT/AOT/Serve
integration are not qualified. Invalidated work is discarded; callers can retry or
use synchronous collection under pressure. There is no automatic pacing/fallback
policy in this coordinator.

### Reclamation evidence

- 100 deterministic graph pairs produce identical survivor sets and reclaimed
  allocation/byte counts under POC versus synchronous collection. Each fixture ends
  with an empty arena after dropping roots.
- Live parent/child contents survive reference transfer and allocation while work is
  outstanding. Unreachable conditional owners/values are freed, handles expire,
  payload poisoning is observed, and the freed address is reused. Native cleanup is
  exactly once on the owner.
- A deterministic interleaving stops the worker after 63 graph objects have been
  scanned. The owner transfers the root, removes an edge and allocates; the worker
  resumes, and apply frees old garbage while retaining survivors/new allocations.
  Following cycles reclaim floating garbage and reach an empty arena.
- Allocation into a pre-existing free span survives the current cycle, then becomes
  reclaimable later. Unrooted parked owners and frame values are reclaimed and the
  registration is expired.
- Ten stale/error/cancel cases preserve live heap bytes and collection/cleanup
  counters. These include the actual conditional-handle rescue, parked resume and
  same-address/same-extent replacement counterexamples, handle mutation, resource
  attachment, malformed worker input and cancellation of an already-completed job.
  Destroying an outstanding job joins it without applying marks.
- All earlier guarded snapshot/oracle, array, conditional fixed-point and parked-root
  tests still pass. The test-only midwalk gate is disabled for ordinary jobs.

ThreadSanitizer/ASan startup limitations remain as previously recorded; this slice
was validated with UBSan/bounds, deterministic interleavings and protected-memory
checks. It does not claim production safety or a speedup. Copy pauses, owner sweep
pauses, worker CPU, conservative invalidation frequency and memory overhead need
measurement before deciding whether to integrate this approach into Serve.


### Capture initialization follow-up

Capture now initializes counts and control flags without clearing unused bounded
array storage. Each used entry is assigned before publication. Poisoned storage and
repeated capture after removing the only root verify that stale tails do not retain
objects or preserve the test pause flag. [Fresh UBSan/bounds qualification](capture-reuse-results.json)
passes alongside all existing reclamation/oracle checks. The [paired performance
comparison](../../../benchmarks/gc-concurrent-poc/README.md#snapshot-initialization-and-larger-graph-2026-10-18)
shows lower capture cost but no convincing overall throughput improvement.


### Single import walk

Owner application now imports captured marks once and marks only new allocations
in captured free spans or the appended tail. All validation remains before mutation.
The expanded split-free-span/tail regression and existing UBSan/bounds suite pass:
319 applied cycles, ten discards and 246,304 reclaimed payload bytes. See
[qualification](import-results.json) and the [paired performance evidence](../../../benchmarks/gc-concurrent-poc/README.md#single-mark-import-walk-2026-10-18).
Mark-import time roughly halves on the larger graph; overall elapsed gains are
small and concurrent GC still trails sync. This remains an isolated bounded POC.


### Combined validation walk

Validate live integrity and captured identity together, with explicit bounds for
subdivided free spans and full appended-tail validation. All validation still
precedes mark mutation. Five new corrupt-live-state cases reject without changing
heap bytes or GC statistics, then restore the injected corruption and drain the
heap. [Fresh qualification](validation-results.json) passes 324 applied cycles,
15 discards and 249,024 reclaimed payload bytes, including existing graph oracles.
[Paired measurements](../../../benchmarks/gc-concurrent-poc/README.md#combined-validation-walk-2026-10-18)
show validation near half its former cost and lower owner interruption. Large-graph
elapsed approaches sync, but CPU remains higher and the small graph remains slower.
No production integration or general safety/performance claim follows.


### Indexed registered-root capture

Reuse the existing ephemeral block index during capture instead of validating the
whole heap for every host/resource/parked owner. Index construction still validates
all descriptors before exact-base lookups; no index survives capture. Zero-registration
capture keeps its original integrity walk. New pointer-boundary/corrupt-tail cases
and the existing reclamation suite pass [UBSan/bounds qualification](root-index-results.json).
[Matched measurements](../../../benchmarks/gc-concurrent-poc/README.md#indexed-registered-root-capture-2026-10-18)
show 47–49% less elapsed and 46–47% less CPU with 64 stable roots, confirmed by a
longer repeat, and about 10% less elapsed with eight. Zero-root behavior is essentially
unchanged. This fixes POC overhead; concurrent remains more CPU-expensive than sync.


### Unchanged strong-handle reads

Successful ordinary host reads from unchanged strong handles now preserve the POC
epoch: their values were captured as strong roots. Creation/replacement/release
still invalidate; subsequent reads cannot revive that result. Conditional reads,
failed reads and callback-specific hooks retain conservative invalidation. This
exception applies only to the audited owner-thread POC, not arbitrary native reads.
[Qualification](strong-read-results.json) passes 329 applied cycles, 18 discards and
249,088 reclaimed bytes, including new read/mutation regressions and existing
conditional-resurrection cases. [Paired evidence](../../../benchmarks/gc-concurrent-poc/README.md#preserve-snapshots-across-unchanged-strong-handle-reads-2026-10-18)
shows unnecessary read-only fallbacks fall from 2000 to zero and owner interruption
falls about 37%; mixed replacements still invalidate exactly 500 cycles. Overall
elapsed gains remain modest, and production collector behavior is unchanged.


### Callback-read audit completed

Successful ordinary/suspended callback reads now preserve the POC epoch after their
existing strong-handle, heap, callback-shape, fault and entry-state checks succeed.
They do not invoke callbacks. Failed reads still invalidate; handle mutations and
parked transitions remain protected. This extends the preceding strong-read exception
without changing production sources or API semantics. Eleven added reclamation
cases and existing tests pass [UBSan/bounds qualification](callback-read-results.json):
342 applied cycles, 27 discards, 249,608 reclaimed payload bytes.
[Matched evidence](../../../benchmarks/gc-concurrent-poc/README.md#callback-read-invalidation-audit-2026-10-18)
removes 2000 read-only fallbacks with about 37% less owner interruption; mixed
replacements still discard exactly 500 cycles. These synthetic callback-object
reads do not qualify callback execution or Serve integration.


### Native callback body and ongoing live-object replacement

A shared C fixture now allocates replacement nodes, publishes through rooted state,
and retires old nodes. An 80-invocation qualification replaces 1360 objects, including
an invocation while tracing is paused midwalk. Verify payload order, bounded arena
extent during the run, and complete final reclamation. [Qualification](callback-churn-results.json)
passes 423 applied cycles, 27 discards and 285,344 reclaimed payload bytes alongside
all earlier tests. The callback-shaped state is passed to a known C body; this is
not Raven delegate dispatch or scheduler integration.
[Matched measurements](../../../benchmarks/gc-concurrent-poc/README.md#invoked-native-fixture-with-live-object-replacement-2026-10-18)
show a repeated elapsed advantage at eight replacements per collection, at higher
CPU cost; 64-replacement results remain mixed. Managed invocation, async/fault paths,
variable live-set size and pressure-triggered allocation remain integration gaps.


### Rooted allocation pressure during tracing

[UBSan/bounds qualification](rooted-pressure-results.json) now pauses the worker
midwalk with a full, bounded arena and calls the existing rooted allocator:

- A satisfiable request synchronously collects, preserves an otherwise unregistered
  borrowed value through explicit helper roots, and reuses a reclaimed address.
- An arena-sized but currently unsatisfiable request collects once and returns the
  memory-limit status without publishing an allocation.
- An impossible single block returns the memory-limit status without collecting;
  the pending snapshot remains applicable.

Both actual pressure collections invalidate the outstanding snapshot. Consuming
its result changes neither heap bytes/extent nor reclamation counts. Each case
checks the rooted chain and ultimately reclaims every allocation. The complete
qualification reports 427 applied cycles, 29 discards and 294,776 reclaimed payload
bytes. This exercises the existing native allocation boundary, not a Raven callback
or scheduler. It changes no production code or collection policy, adds no speedup,
and does not remove the need to measure pressure-heavy workloads. As with CLR
allocation-triggered collection, callers must expose live references; this POC uses
explicit native helper/frame roots rather than CLR-generated GC maps.

Next integration gaps remain managed invocation, suspension/fault paths and pressure
under sustained application load. The earlier fixed-size fixture results are still
the performance evidence; these deterministic pressure cases are safety checks.


### Native suspension and fault lifetimes

[UBSan/bounds qualification](lifecycle-results.json) exercises two complete native
lifetimes: park/resume/fault/unwind/complete and park/discard/complete. Each park,
resume/discard and successful completion occurs while tracing is paused midwalk.
The pending result is discarded without changing heap bytes, extent or collection
and reclamation counters. A separate collection while parked verifies that a
strongly rooted owner keeps the suspended message alive.

Transferring an already-rooted message into the context fault, then unwinding its
frame, permits the existing snapshot to apply safely. Completion removes static
storage and intern ownership. After releasing the callback owner, only the fault
message survives; clearing the fault allows complete reclamation. Both paths check
exactly-once owner-thread resource cleanup and a clean final entry check. The
collector is noncompacting: assertions count survivors, not just arena extent.
The complete qualification passes 433 applied cycles, 35 discards and 301,032
reclaimed payload bytes. An initial test-only assertion incorrectly equated one
surviving object with compacted arena extent; correcting that assertion was the
only repair needed. No collector change was needed.

Like CLR exception and suspended-state lifetimes, live fault/continuation values
must remain roots until released. These tests exercise neoCLR's explicit native
frames, parked ownership and fault slots; they do not execute Raven async state
machines, exception dispatch or the scheduler. This closes those native boundary
checks, not managed integration. Performance evidence is unchanged; sustained
pressure and managed execution remain the next validation work.
