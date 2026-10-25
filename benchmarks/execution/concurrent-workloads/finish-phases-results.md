# Concurrent finish: owner CPU and wall attribution (2026-10-25)

The [complete-operation profiler](owner-pauses-results.md) located a 7.4-ms JSON
outlier inside finish but could not attribute its components. Add an opt-in
`--profile-finish` extension, requiring `--profile-owner-pauses`, to the retained
macOS ARM64 owned-heap diagnostics. No collector algorithm, synchronization,
compiler code, public contract or production default changes.

## Scope and validation

Split successful finish into initial marker wait, mutation-log drain/mark,
optional marker redispatch/wait, final collection and disposal. Record owner-thread
CPU (`CLOCK_THREAD_CPUTIME_ID`) and monotonic wall time. The final collection phase
includes index construction, current-root tracing, snapshot identity validation
and mark seeding, worklist processing and sweeping. It is not exclusively sweep.
Disposal includes final state changes and native snapshot/index/gate release.

Whole-finish timing includes the instrumented phase bookkeeping. Phase intervals
do not overlap; whole time also contains the gaps. Keep the components of the
**same slowest whole-finish event**, not a fictional combination of per-phase
maxima. Optional phases carry an execution mask; zero time is distinguishable
from a phase that never ran. Wall quantiles remain histogram intervals. Thread
CPU excludes marker CPU; short wall/CPU intervals may invert due to clock
resolution and their different sampling boundaries. Do not interpret every
wall-minus-CPU difference as mutex blocking.

Deterministic-clock tests with UB/bounds sanitizers verify CPU/wall sums, optional
phase counts, histogram bins and same-event selection when a different call owns
a phase maximum. Parser negative controls reject missing/duplicate rows, count
mismatches and impossible attribution. Injection rejects absent/duplicate finish
definitions and changed phase anchors. Final injection reproduces the measured
collector byte-for-byte after tightening definition checks.

The cohort passes **27 standalone executions and 12,480 HTTP responses**, including
sync and guarded-sync controls. CPU/wall diagnostics apply only to live/guarded
candidate hosts; guarded-sync has zero finishes. Every phase count agrees with
whole-finish and independent lifecycle counters. Histogram totals/bounds, phase
sums, event maxima and enclosing owner-pause maxima are checked. Standalone
allocation equality passes; HTTP scheduling-dependent variation is retained.
The optional second marker wait is exercised by the synthetic accounting test,
but occurs **zero times in these real workload trials**; this cohort does not
qualify its runtime timing under pressure.

The diagnostics add 3,412 bytes of owner TLS counters/event state before toolchain
alignment/metadata, plus local timestamp storage, on top of owner-pause profiling.
No profiler heap allocations occur. Multiple clock reads and reporting add cost;
these runs are not throughput or precise overhead evidence. Keep the earlier
uninstrumented performance cohorts for performance conclusions.

## Results

Pooled live finish work across three ordinary rounds and two HTTP rounds:

| Workload | Finishes | Whole wall / owner CPU | Final collection wall / owner CPU |
| --- | ---: | ---: | ---: |
| Tasks | 384 | 7.854 / 7.770 ms | 6.908 / 6.827 ms |
| Collections | 1,392 | 31.828 / 31.567 ms | 29.023 / 28.758 ms |
| Arrays | 504 | 9.398 / 9.288 ms | 8.286 / 8.178 ms |
| Plaintext | 967 | 48.907 / 48.373 ms | 44.015 / 43.655 ms |
| JSON | 1,899 | 94.026 / 93.318 ms | 86.509 / 86.208 ms |

Final collection consumes about **88–92% of finish wall time**, with owner CPU
close to wall time. Initial waits total 0.395 ms plaintext / 0.763 ms JSON;
log drain/mark totals 2.784 / 3.615 ms, and disposal 1.114 / 1.912 ms.
These observations favor investigating active final collection work before
changing worker synchronization based on an assumed dominant wait.

The slowest JSON finish in this cohort is 116 µs wall / 64.958 µs owner CPU.
Its initial wait is 54 µs wall / 0.291 µs CPU; final collection is 56 µs wall /
56.167 µs CPU. The paired event supports a low-CPU delay within that wait, but
cannot distinguish mutex/condition blocking from OS scheduling. The prior 7.4-ms
outlier did **not** recur; do not retroactively assign it this event's cause.
The slowest plaintext finish is 101 µs wall / 99.834 µs owner CPU, dominated by
91 µs of final collection.

## Next target and evidence

Inspect the cost inside final collection: `build_index`, current roots,
`candidate_seed` and sweep. The current seed validates each old allocated object
through `indexed_block` before walking current allocations to apply snapshot
marks and trace new objects. This is a concrete lookup/validation candidate,
not yet proof that it dominates final collection or permission to skip checks.
Measure these parts before selecting reuse or a linear validation strategy.
Preserve current root values, snapshot identity, old/new allocation safety and
cleanup. Concurrent/default, shared-worker and broader pause qualification remain
open; synchronous GC stays the default.

`finish-phases-evidence.json` contains compact pooled data, same-event outliers
and tool/report hashes. Compressed raw cohorts, full distributions, commands and
generated collector accompany it. Reproduce using the retained commands with
`--profile-owner-pauses --profile-finish`; run `test-finish-phases.py` and
`summarize-finish-phases.py` for validation. Existing bundles/native recipes and
objects are required. This reuses the existing .NET/CLR profiling comparison;
it introduces no managed API divergence.
