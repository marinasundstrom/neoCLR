# Existing GC walker on an isolated worker snapshot

2026-10-18. Proof of concept only: this reuses the existing native mark worklist,
address index and conditional-root fixed point on worker-private memory. The
[validated one-job worker](../concurrent-gc-worker/README.md) supplies submission,
publication, cancellation and joined teardown. Production sources and `--gc`
choices are unchanged. No result is applied to the live heap.

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

## What this permits next

The worker can reuse the current walker without consulting live heap storage for
these captured cases. Next, define how host-handle reads,
root changes and allocation epochs invalidate or preserve a result before attempting
live reclamation. Repeated snapshots fixing a retention bug does not establish that
applying old marks is safe. Keep the synchronous collector as the lifetime oracle.

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
