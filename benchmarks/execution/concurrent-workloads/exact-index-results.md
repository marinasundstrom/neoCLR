# Exact-base allocation index experiment (2026-10-25)

## Scope and safety

Extend the [owned allocation-start index](allocation-index-results.md) behind
`NEOCLR_LIVE_EXACT_INDEX` (requires `NEOCLR_LIVE_ALLOCATION_INDEX`). The existing
index is a complete owner-local set of live payload bases. Use exact bit membership
after a miss in the 256-entry positive cache in `known_base`.
A direct-membership control bypasses cache reads via
`NEOCLR_LIVE_DIRECT_EXACT_INDEX`; it retains cache maintenance to isolate lookup.
Null retains its admitted-root meaning; nonnull pointers must be aligned and within
the current owned used extent before reading the index. An interior address does
not become a base merely because its surrounding allocation is live.

This reuses the same allocation/reclamation/reset/release protocol and adds no
metadata or allocations. Keep the existing base cache and its updates. The fast cache hit remains useful;
the bitmap resolves misses without a checked full-heap walk. The marker does not
read the index. Ordinary unowned heaps retain checked walks. Public APIs, object
headers, compiler lowering, shared-worker admission and production defaults do not
change.

The positive proof has the same trust boundary as the existing positive cache:
allocator-owned metadata certifies current allocated bases. It is not a whole-heap
corruption scan. Existing store paths still validate selected headers and bounds;
helper admission still validates the current arena. No arbitrary input pointer
can create an index entry. Missing membership still falls back to the existing
checked path where the caller requires it.

This is a follow-up to the .NET/Go indexed-location research linked above, not a
new managed API divergence or a claim to implement those collectors. Constant-time
exact membership is distinct from the predecessor summary search used for interior
stores, whose worst-case work still depends on arena size.

## Validation

The expanded independent model checks exact allocation information in addition to
predecessor lookup and checked stores for 72,000 sampled addresses per optimized
and undefined/bounds-sanitized build. It also checks all retained live bases,
including more objects than the old cache can hold. Reclamation, reuse, reset,
reacquisition, large holes, allocation failure and zero-capacity handling remain
covered. Both deliberately omitted-revocation controls fail as required.
Native safety tests and stale-hint/deletion controls pass. Generated code passes
600 entry and 840 record-array JIT/AOT fixture cycles. ASan remains the previously
documented host-initialization gap; it was not rerun or counted as passing.

## Direct-membership findings

The first version replaced every cache read with bitmap membership. Ninety ordinary
runs pass, but live CPU medians rise: tasks 0.024953 to 0.026012 seconds,
collections 0.145056 to 0.147549 and arrays 0.059110 to 0.059727. The collection
sample's 125 exact-base walks / 68,749 visited spans fall to zero. Lower lookup
work alone therefore does not establish lower total execution cost.

The direct version validates 86,688 HTTP responses across both handlers and seven
modes. Live plaintext improves from 2,415 to 2,572 req/s (CPU 0.882987 to 0.836182
seconds); JSON improves from 1,518 to 1,629 (1.406545 to 1.321843 seconds). These
comparisons use the preceding predecessor-index variant in the same cohort.
Live managed allocations match. One plaintext guarded-direct trial adds one
528-byte object; JSON sync trials add one/two, and one guarded-direct trial adds
one. Retain the raw variation rather than asserting whole-cohort equality.

This motivated the hybrid cache-hit/index-miss variant. Both variants remain
opt-in experiments; the hybrid is the meaning of `--exact-allocation-index`, and
the direct variant is a reproducible control.
Its generated source is archived as `exact-index-direct-source.c.gz`; original
direct reports predate the explicit direct-control flag. Later reproduction uses
`--direct-exact-allocation-index` in the measurement tools.

## Hybrid ordinary findings

A fresh ninety-run baseline/hybrid comparison passes outputs, cleanup and exact
managed allocation equality. Live CPU medians are tasks 0.025173 / 0.026331 seconds,
collections 0.144633 / 0.148825, arrays 0.059297 / 0.060351. These observed increases
of about 4.6%, 2.9% and 1.8% do not establish a general CPU improvement. Ordinary
baseline/candidate cohorts run sequentially, with mode order alternated within each;
small effects can include between-cohort drift. HTTP retains both variants in one
alternating cohort. The hybrid also eliminates the collection sample's remaining
exact-base walks. Keep that mechanism evidence separate from timing conclusions.

## Hybrid HTTP findings and decision

The hybrid validates another 86,688 responses, giving 173,376 checked responses
across the direct and hybrid comparisons. Four-client medians against the
predecessor-index control in the same cohort:

| Handler | Prior indexed req/s | Hybrid req/s | Prior CPU | Hybrid CPU |
| --- | ---: | ---: | ---: | ---: |
| Plaintext | 2,449 | 2,609 | 0.880401 s | 0.826707 s |
| JSON | 1,497 | 1,625 | 1.403337 s | 1.326125 s |

Throughput rises 6.5%/8.5%, CPU falls 6.1%/5.5%. Guarded-sync also benefits:
plaintext CPU falls 0.745980 to 0.723535 seconds, JSON 1.209585 to 1.159863.
Ordinary sync still serves about 5,071/3,042 requests/s, well ahead of the candidate.
These closed-loop loopback measurements include connection-close/client costs;
they do not establish saturation throughput or framework parity.

Plaintext client p99 trial medians are 1.811 / 1.634 ms (prior/hybrid). JSON client p99 trial medians are 2.810 / 2.616 ms (prior/hybrid). These are response latencies, not GC-pause distributions.

Live allocations match in every comparison: 429,856 objects / 33,057,881 bytes
for plaintext and 943,817 / 54,065,775 for JSON, per 2,064 responses. The hybrid
cohort's earlier guarded-sync plaintext control has one/two additional 528-byte
objects; one JSON sync trial adds one. All indexed runs report one native metadata
allocation, 16,640 bytes peak, zero retained at shutdown. There is no new allocation
or index-memory cost relative to the previous indexed implementation. RSS and
per-trial timings remain in raw reports; requested metadata bytes are not RSS.

Both variants pass native safety, 144,000 independent model comparisons each and
1,440 JIT/AOT fixture cycles each. The two ordinary comparisons total 180 runs.
Keep the hybrid and direct variants opt-in: the HTTP benefit is useful evidence,
but the ordinary CPU cost requires further profiling and tighter alternating
baseline/candidate measurement. Neither the index nor exact-base reuse is promoted
to the production default. Shared-worker qualification, larger arenas, sustained
JIT workloads and pause distributions remain open. Preserve synchronous comparison
and consider reusable lookup improvements for both collectors where applicable.

## Reproduction

Add `--allocation-index --exact-allocation-index` to native safety, entry,
record-array and ordinary-workload validators; the independent index model only
needs `--exact-allocation-index` because it always uses the index. HTTP uses
`--compare-allocation-index --compare-exact-allocation-index --include-guarded-sync`
with four clients, 2,048 measured requests, 16 warmup requests and three rounds.
This retains seven modes: ordinary sync, the previous anchored guarded/live pair,
the predecessor-index guarded/live pair and the exact-index guarded/live pair.
