# Complete owner-operation pause diagnostics (2026-10-25)

## Implemented measurement

Scope: the retained macOS ARM64 owned-heap experiment.

The [managed-JIT investigation](stack-jit-results.md) found that the existing
collection timer excludes portions of concurrent finish and cancellation.
`owner-pauses.py` instruments a copied native collector, preserving the original
entry points and wrapping complete collect calls, begin, finish, cancel and
admitted guarded writes. Synchronous and guarded-sync controls use the same clock
and histogram implementation. No production runtime or compiler behavior changes.

`--profile-owner-pauses` enables this in the ordinary and HTTP measurement tools.
Generated hosts report after cleanup on the invoking owner, not a destructor or
marker thread. All results are marked diagnostic-only. This uses existing runtime
profiling conventions rather than introducing a .NET semantic/API divergence.

Seven rows are reported:

| Row | Scope |
| --- | --- |
| collect-call | Inclusive native collection entry, including nested live operations |
| begin | Complete candidate begin |
| finish | Complete wait/drain/remark/reclamation/disposal sequence |
| cancel | Complete cancellation/disposal sequence |
| guarded-write | Admitted write including validation/gate access; not pure mutex wait |
| outermost | Non-overlapping outer calls across all five operations |
| outermost-gc | Non-overlapping outer GC operations, excluding guarded writes |

Inclusive rows overlap and **must not be summed as total pauses**. The first
prototype combined outer GC operations and writes; frequent tiny writes diluted
the percentile. Retain that first cohort separately and use the final seventh row
for GC comparisons. Internal phase recursion is counted in inclusive rows, while
outer histograms count each selected owner operation once.

These scopes are not every possible GC-related execution cost: ordinary root
publication, allocation/lookup work outside these functions, and final worker
shutdown outside active cancellation are excluded. The guarded-write row does not
cover early store paths that never reach `live_write_known`. Wall time includes
scheduler delays; it is not collector CPU. No cross-worker stop-the-world or
request-latency meaning is inferred from an owner-operation histogram.

Counters use owner TLS, no heap allocations: 3,808 counter bytes plus 16 bytes of
tracking state before toolchain TLS metadata/alignment. Disabled cohorts do not
contain the wrappers/counters. Sixty-four log2 nanosecond buckets retain counts,
sums, maxima and nonzero-result counts. The first bucket covers 0–1 ns; quantiles
are reported as intervals, not fabricated exact percentiles. Nonzero results may
include handled gate cancellation, not necessarily a guest failure.

## Validation and evidence

Deterministic-clock C tests, compiled with UB/bounds sanitizers, verify inclusive
and outer duration accounting, status counts, histogram boundaries, owner TLS
isolation and duplicate/missing-definition rejection. Parser tests reject missing
rows, duplicates, inconsistent counts and impossible totals. Real summaries check
bucket sums/bounds, maximum/total consistency and outer/inclusive consistency. Successful begin/finish/cancel call counts also
match the independent candidate lifecycle counters.

The final diagnostic cohort passes **27 ordinary executions and 12,480 HTTP
responses**, with synchronous, guarded-sync and inlined concurrent controls.
Tasks/collections/arrays use the transformed retained objects; HTTP uses the same
plaintext/JSON candidate objects. Ordinary allocation equality passes. HTTP keeps
scheduling-dependent allocation variation rather than forcing equality. The
matching uninstrumented control cohort passes the same counts. An initial
six-row exploratory cohort also passes those counts but is not used for final
pause conclusions.

The default local clang/SDK combination initially failed at link time on an SDK
architecture entry. Retain the failure; using the existing matched Xcode compiler
and SDK recipe passes the deterministic tests. This was not a collector failure.

## Observed distributions

Pooled final distributions (three ordinary rounds, two HTTP rounds; bounds are
microseconds, lower inclusive / upper exclusive):

| Workload | Sync outer-GC p99 | Concurrent outer-GC p99 | Sync / concurrent maximum |
| --- | ---: | ---: | ---: |
| Tasks | [4.096, 8.192) | [16.384, 32.768) | 12 / 42 µs |
| Collections | [4.096, 8.192) | [16.384, 32.768) | 12 / 54 µs |
| Arrays | [4.096, 8.192) | [16.384, 32.768) | 10 / 55 µs |
| Plaintext HTTP | [32.768, 65.536) | [65.536, 131.072) | 77 / 162 µs |
| JSON HTTP | [32.768, 65.536) | [32.768, 65.536) | 85 / 7,411 µs |

Concurrent finish p99 lies in [65.536, 131.072) µs for both HTTP cases. JSON's
7,411-µs outer maximum coincides with a 7,410-µs finish maximum. This locates the
outlier within finish, but does not separate marker waiting, log processing,
remark/reclamation and OS descheduling. Preserve the outlier; do not ascribe it
to a specific cause without additional measurement. This cohort does **not** show
a concurrent pause advantage. Sync remains faster and the production default.

Outermost GC totals across pooled trials are 55.506 / 72.740 ms for plaintext
sync/live and 119.036 / 149.672 ms for JSON. Different collection schedules/counts
mean these are measured totals, not equal-work per-collection comparisons. In this cohort, outer
non-GC writes account for 28.934 / 33.491 ms for plaintext/JSON; inclusive rows cannot be
added again to these outer totals.

## Instrumentation overhead and next step

Matching uninstrumented controls ran before the final profiled cohorts, not in a
balanced interleaving. Ordinary profiled/control CPU median ratios range about
0.94–0.99, while live HTTP ratios are 1.057 plaintext and 1.030 JSON. The apparently
faster instrumented ordinary runs expose drift/noise; this does **not** establish
precise profiler overhead or justify treating it as free. HTTP process CPU also
includes report emission, unlike the ordinary invocation CPU boundary. Retain the
raw control reports and ratio calculations as exploratory overhead evidence.
Use the earlier balanced, uninstrumented cohorts for throughput conclusions.

Next split finish into marker waits, log draining/marking and final remark/disposal,
with owner CPU alongside wall time where practical. This can distinguish waiting
from active work before changing synchronization or scheduling. Keep phase sums
and nesting explicit. Shared heaps, cross-worker safety, broader JIT pause coverage
and concurrent-default acceptance remain open.

`owner-pauses-summary.json.gz` contains final pooled histograms and quantile bounds;
`owner-pauses-overhead.json` contains exploratory CPU ratios. Raw cohort reports,
invocation commands, generated first/final collectors and test logs accompany this
report. Reproduce with the retained commands and `summarize-owner-pauses.py`; these
recipes require the retained compiler bundle, native host recipes and objects.

The subsequent [finish subphase investigation](finish-phases-results.md) adds
owner CPU/wall attribution and shifts the next target toward active final
collection work; it does not retrospectively explain the outlier above.
