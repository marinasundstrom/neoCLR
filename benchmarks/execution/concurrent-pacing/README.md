# Industry-inspired pacing screen and runtime/collector split

2026-10-25. The author requested a visible concurrent-over-sync advantage before
wrapping up, and then emphasized investigating runtime bottlenecks before choosing
industry techniques. This screen is a first implemented pacing mechanism, not the
required final advantage or an adaptive policy.

Private owned-heap allocation/publication budget defines now default to the existing
256/4096 values. The boundary fast path and trigger classification use the same
allocation constant. The HTTP harness applies selected values to **both** collectors.
The separate compiled shared-worker pool's pacing policy is unchanged. There is no
new managed API, relocation, ownership exemption or production-default change.

Three alternating sync/live rounds per budget, four clients, 2,048 measured requests
plus 16 warmups validate 24,768 plaintext responses. Both use the same 1 MiB arena
capacity. A 1024/16384 budget is compared with 256/4096. Each cohort interleaves modes,
but the two budget cohorts ran sequentially: this is a screening result, not a
paired statistical estimate across budget settings.

| Budget | Mode | Median requests/s | Process CPU seconds | Peak arena bytes |
| --- | --- | ---: | ---: | ---: |
| 256/4096 | Sync | 4,978 | 0.419 | 99,224 |
| 256/4096 | Concurrent | 3,530 | 0.624 | 146,752 |
| 1024/16384 | Sync | 5,358 | 0.385 | 170,120 |
| 1024/16384 | Concurrent | 3,956 | 0.532 | 471,640 |

Concurrent throughput rises about 12.1%, process CPU falls 14.7%, and managed
allocations remain 429,856. Its arena high-water mark grows about 3.2x. Sync also
improves and stays ahead. Larger budgets reduce concurrent epoch frequency to about
one per eight responses. Do not present this as concurrent beating sync, allocation
savings, or a free speedup. Native allocation bytes and longer-run retention remain
unmeasured. Pressure recovery, idle handling and forced collection are unchanged.

UB/bounds tests cover default/custom thresholds, status propagation, context and
collection-epoch slow paths, and rejection of a zero budget. All sampled requests,
epoch accounting and teardown checks pass.

## Current runtime/collector evidence

A separate default-budget diagnostic cohort samples sync, guarded-sync and concurrent
execution twice each, with 8,192 measured requests plus 16 warmups: 49,248 checked
responses. All three use the optimized generated candidate where applicable.
The checked macOS stack parser retains thread identities and TLS caller offsets.
These are wall-stack observations, including waits, not CPU percentages or throughput.

Both sync and guarded-sync prominently show root-frame entry, stack-root publication,
TLS access, native stack checks and allocation/mark helpers. Guarded sync additionally
shows the store adapter. Concurrent runs add marker lookup/scanning and synchronization.
The largest waiting category is a sleeping marker condition variable; it is not
evidence that marker waits consume most CPU. Examples pooled from two samples:

| Observed top frame | Sync | Guarded sync | Concurrent |
| --- | ---: | ---: | ---: |
| enter_owned | 186 | 86 | 91 |
| stack_roots | 119 | 70 | 66 |
| experiment_gc_store | not reported | 86 | 79 |
| TLS lookup | 177 | 136 | 129 |

Sample populations/durations differ; the row counts are not normalized cost ratios.
TLS caller attribution points to native stack checks, frame entry, store adapters
and marking. Source inspection confirms the stack helper reads retained per-thread
bounds, with an OS-stack fallback. Removing safety checks or caching beyond an
activation's ownership would not be justified by those observations.

This complements the [guarded CPU control and pause evidence](../concurrent-workloads/stable-byref-pause-results.md)
and [begin/marker CPU profiles](../concurrent-workloads/begin-phases-results.md):
the performance gap spans runtime bookkeeping **and** collector work. The
[industry comparison](../concurrent-workloads/industry-gc-comparison.md) suggests
pacing, persistent metadata/local allocation, precise tracing/locality and eventually
generations. Neither more marker threads nor repeated small seed tweaks alone is
supported as the next solution.

## Next bounded decision

Keep defaults unchanged. Use this budget sensitivity as a control, quantify native
snapshot allocations, and test reusable epoch metadata against the current copied-
header implementation. Separately investigate proven-safe elimination/grouping of
runtime TLS/root checks. A claim of success needs an actual sync comparison on the
same workload and stated memory/CPU constraints, including latency and longer-lived
graphs. The author has not accepted these screening improvements as completion.

Evidence: 1x-results.json.gz, 4x-results.json.gz, summary.json, tests.json and
runtime-* reports/raw samples. The existing measure-plaintext.py accepts
--allocation-budget and --publication-budget; private native defines are
NEOCLR_GC_ALLOCATION_BUDGET and NEOCLR_GC_PUBLICATION_BUDGET. Use fresh output paths.
