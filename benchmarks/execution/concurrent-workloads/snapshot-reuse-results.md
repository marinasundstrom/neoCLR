# Reuse snapshot classification within a store (2026-10-25)

## Finding and lifetime argument

The preceding [cached-header experiment](header-proof-results.md) showed no
convincing gain from maintaining another validation cache. Review instead found
two identical snapshot lookups in the scalar store adapter: the first selects and
validates an old allocation; the second decides whether to gate the write. Reuse
the first result in a local boolean. No extra TLS, object header, detached metadata
or managed allocation is needed.

The first lookup still precedes allocation admission. Snapshot headers, offsets
and index remain immutable for the active epoch; marker work changes mark/queue
state, not this identity. Between lookup and write, the owner performs header,
range and width checks and updates private range hints. It performs no callback,
allocation, collection, cancellation or epoch transition. The scalar adapter's
noncollecting contract and isolated owner scope make the result valid for this
call. The result is not retained across calls or used as a shared-heap proof.

All full header and detached identity checks remain. Old allocations still enter
the mutation gate, and overflow cancellation still joins/discards work before the
store. New allocations remain subject to current owned allocation and extent
checks and are traced at remark. Legacy linear lookup and the explicit
`NEOCLR_LIVE_REPEAT_SNAPSHOT_LOOKUP` comparison retain the repeated search.

This is an internal lookup elimination, not a CLR API divergence or a new
lifetime/ownership contract. The existing [research context](packed-tls-results.md)
and native boundary guarantees apply. Production compiler and GC defaults are
unchanged; only the private candidate store adapter changes.

## Validation

Native optimized and undefined/bounds-sanitized suites pass for reuse and repeated
lookup. Existing negative controls still fail when deletion logging or newest/older
hint ownership admission is removed. Existing tests cover old/new classification,
old descriptor corruption, immutable array shape, unaligned stores, reclamation,
reuse, cancellation and independent owner isolation. Current generated code passes
600 entry and 840 record-array JIT/AOT cycles using the retained private `speed`
backend setting. These results reuse unchanged compiler/full-server JIT evidence;
current HTTP runs relink fresh native collectors to the same retained AOT objects.
They do not qualify shared heaps or every possible thread interleaving.

## Measurements

Ninety ordinary executions pass exact outputs, cleanup and allocation equality.
Live CPU medians (reuse / repeat) are 0.077589 / 0.079077 seconds for tasks,
0.393366 / 0.397851 for collections, and 0.090089 / 0.091160 for arrays: reductions
of about 1.9%, 1.1% and 1.2%. These are separate fixed-work cohorts.

Three interleaved rounds across five modes and two handlers validate 61,920 HTTP
responses, including warmup:

| Handler | Repeated lookup requests/s | Reuse requests/s | Repeated lookup CPU | Reuse CPU |
| --- | ---: | ---: | ---: | ---: |
| Plaintext | 1,825 | 1,871 | 1.148822 s | 1.120775 s |
| JSON | 1,068 | 1,084 | 1.900772 s | 1.880114 s |

Observed throughput rises 2.5%/1.5% and CPU falls 2.4%/1.1%. Guarded-sync CPU
moves from 1.006892 to 1.003526 seconds for plaintext and 1.668348 to 1.657545 for
JSON; these small changes without active snapshots indicate measurement/layout
variation, so do not attribute every timing difference to the removed lookup.
Ordinary synchronous GC remains faster at 4,628/2,376 requests/s.

All HTTP modes allocate the same counts/bytes: plaintext 429,856 / 33,057,881 and
JSON 943,817 / 54,065,775 per 2,064 responses. Standalone allocations also remain
identical. Reuse adds no persistent native memory. Sampled process peak RSS is
unchanged for plaintext live (11,354,112 bytes), and JSON differs by one 16-KiB page
(13,238,272 versus 13,221,888), not evidence of an allocation improvement.

The retained ARM64 disassembly confirms **two `shadow_find` call sites become
one** in `experiment_gc_store`; its instruction count falls from 436 to 421.
This verifies removal of actual native work rather than relying solely on timing.
Raw extracted assembly and binary hashes accompany the report.

**Decision:** retain within-call classification reuse in the private candidate.
It removes redundant work without extra state or weaker validation; measured gains
are modest and do not close the synchronous/concurrent gap. Further work should
address common store entry, root publication/descriptor and metadata costs, while
retaining correctness and separate pause/scaling qualification. Do not relax
ownership or mutation protection to chase the remaining throughput gap.

## Reproduction

Native: `concurrent-live/validate.py --sparse-index [--repeat-snapshot-lookup]`.
Standalone: `concurrent-workloads/measure.py --guarded-control
[--repeat-snapshot-lookup]` against `optimizer-build`, five rounds. HTTP:
`concurrent-store/measure-plaintext.py --include-guarded-sync
--compare-snapshot-lookup --pairs 3 --requests 2048`, using `optimizer-http-build`.
Each HTTP trial adds 16 warmup responses. `live` reuses classification;
`live-repeat-lookup` is the prior algorithm. Guarded-sync modes admit the same
stores but never start concurrent epochs. Timing cohorts run sequentially after
correctness compilation; they contain no sampling profiler.

Raw `snapshot-reuse-*` and `snapshot-repeat-*` compressed reports preserve commands,
hashes, outputs and individual trials. `snapshot-reuse-summary.json` provides
medians. Sequential loopback requests and fixed-work ordinary samples do not
establish saturation capacity, framework parity or pause distributions. Sync stays
the platform default; concurrent default/worker/scaling acceptance remains open.
