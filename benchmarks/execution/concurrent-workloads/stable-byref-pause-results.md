# Optimized JSON JIT and remaining collector costs

2026-10-25 follow-up to the [matched cost comparison](stable-byref-cost-results.md).
The candidate now passes JSON managed-JIT execution. The new profiles do **not**
show a concurrent pause advantage. Keep both collectors and the explicit byref
prototype; concurrent-default acceptance remains open.

## JIT qualification

The HTTP execution qualifier now accepts `--stable-byrefs` only for managed JIT
with the existing direct/forwarded stack proofs. It clears an ambient byref switch
when not selected and records the actual compiler environment. Production compiler
options and defaults do not change.

Speed-optimized JSON JIT, paced collection and inline exact indexing pass 512
checked responses. There are 519 begun epochs, 518 completed, one cancelled and
none active at shutdown; index metadata is released. The JIT removes 31,311
eligible stores, matching the candidate AOT audit. Exit status and expected UTF-8
JSON content pass. This is execution/lifecycle qualification, not a JIT throughput
or compile-speed benchmark. The prior AOT and root/sanitizer evidence is retained.

## Uninstrumented guarded-synchronous control

Four alternating-order rounds, four clients, 2,048 measured requests and 16 warmups
per process validate another **24,768 responses**. The guarded control uses the
candidate generated object and native guard path but disables asynchronous epochs;
its six candidate counters are all zero. The live control completes epochs and all
modes pass teardown. These runs are separate from the diagnostic profiles below.

| Mode | Median requests/s | Process CPU us/response |
| --- | ---: | ---: |
| Sync | 3,080.1 | 326.33 |
| Guarded sync | 2,600.1 | 385.63 |
| Concurrent candidate | 2,306.9 | 463.32 |

CPU includes startup, warmup and shutdown, divided by all 2,064 responses. About
43.3% of the excess over sync persists in guarded sync. This comparison includes
compiler/native guard costs and collector policy interactions; the remaining 56.7%
is not an attribution solely to the marker thread. It includes scheduling,
snapshot/remark/reclamation and differences in object lifetime. No new optimization
or allocation saving is introduced by this control.

## Owner-operation distributions

Use the separate **owner-only** cohort for these distributions: three alternating
rounds, two modes, 1,024 requests plus 16 warmups give 6,240 checked JSON responses.
Three rounds for each of tasks/collections/arrays and two modes give 18 standalone
runs of 4,096 iterations each. Expected output, standalone allocation equality and
teardown pass.
Histograms pool three runs; each bucket is a lower-inclusive/upper-exclusive range.

| Workload | Sync GC-operation p99 us | Concurrent p99 us | Observed max us, sync / concurrent |
| --- | ---: | ---: | ---: |
| JSON | [32.768, 65.536) | [32.768, 65.536) | 59 / 90 |
| Tasks | [4.096, 8.192) | [16.384, 32.768) | 10 / 25 |
| Collections | [4.096, 8.192) | [16.384, 32.768) | 13 / 36 |
| Arrays | [4.096, 8.192) | [16.384, 32.768) | 11 / 66 |

These are owner-thread elapsed times of outermost GC operations, excluding guarded
writes, not request latencies or global safepoint pauses. Instrumentation itself
costs clocks/counters and the host scheduler can affect maxima. No global shared-
heap pause or uninstrumented production tail guarantee follows. Even with those
limits, this cohort supplies no evidence for the proposed pause advantage.

## Detailed attribution, kept separate

A more instrumented cohort adds per-phase owner wall/CPU clocks: three modes
(sync/previous-live/candidate), three rounds, 9,360 checked JSON responses and 27
standalone runs. All counters, histogram bounds, event sums and teardown pass.
The detail clocks perturb short operations, particularly live collection; do not
use this cohort as an unbiased cross-collector throughput or pause comparison.

Final owner collection consumes 92.8% of measured finish CPU in JSON and about
89.7%/91.8%/89.8% in tasks/collections/arrays. Initial plus remark wait uses only
0.34% of JSON finish CPU; background marker CPU is outside this owner measurement.
The pooled JSON final-collection breakdown is:

| Component | Share of final-collection owner CPU |
| --- | ---: |
| Snapshot seeding | 38.6% |
| Sweep | 20.3% |
| Index build | 9.9% |
| Frame roots | 8.2% |
| Unattributed remainder, including instrumentation | 23.0% |

These percentages are nested within final collection, not total request CPU.
They must not be added to the enclosing finish totals. The checked summaries retain
the components from each same-event slowest finish rather than adding unrelated
maxima. The previous-live and candidate JSON final-collection costs are similar,
consistent with the byref change reducing ordinary executed guard calls rather
than changing the collector algorithm.

## Decision and next work

Do not promote concurrent GC based on throughput gains against its older version;
it still trails sync in CPU and this pause cohort. Do not spend effort optimizing
marker *waiting* based on these profiles. Measure begin/snapshot capture and actual
marker-thread CPU alongside whole-request costs, and continue isolated-owner
scaling qualification. Owner-side seed/sweep work remains a concrete candidate,
but the [existing direct-seed experiment](direct-seed-results.md) already showed
that a local phase improvement can have little whole-workload effect. Preserve
that option without promoting or repeating it solely because it dominates this
nested profile. Thread-sanitizer, shared heaps and moving GC remain unqualified.

Evidence is in [stable-byref-evidence](stable-byref-evidence/): `json-jit-*`,
`json-guarded-*`, `json-owner-only-*`, `ordinary-owner-only-*`, `json-pauses-*`,
`ordinary-pauses-*` and the checked owner/finish/parts summaries. Reports retain
commands, inputs, compiler environment where applicable, all trials and counters.
The existing profiler and checked parsers are reused unchanged. No public API,
Raven semantics/emission, Runtime Contract, metadata encoding, ownership boundary
or native ABI changes are made by this qualification.
