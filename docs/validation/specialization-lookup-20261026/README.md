# Immutable specialization-source lookups — 2026-10-26

This follows the [selection-index optimization](../metadata-selection-20261026/README.md), whose CPU captures still identify generic expansion, interface closure and variance/assignability as repeated type-lookup callers. Baseline is `7396dfe74`, including selection indexes; this is an incremental comparison, not a return to the original lazy-JIT baseline.

## Contract

After existing source inventory limits pass, generic expansion runs inside the existing immutable source lookup scope. The growing expanded module remains separate and mutable. A separately supplied original conformance module retains its distinct identity and existing validation path; it is not silently replaced by projected source metadata. When source and proof are the same module, they naturally share the scope. No conformance result, specialized body, signature decision or constraint result is cached.

This reuses the [.NET/CLR shared metadata-resolution research](../../design/allocation-metadata-runtime-comparison.md#scoped-method-owner-candidates--2026-10-24), preserving generic constraint and interface semantics while amortizing candidate lookup. A permanent cache would require invalidation; indexing the mutable expanded inventory would require synchronizing every insertion/rewrite. The bounded immutable scope avoids those obligations, at the cost of temporary index construction and potential allocator reservation. It does not change managed APIs, metadata formats, GC roots, code publication, first-call scheduling, or the interpreter path.

## Qualification

The static Self/generic test compares complete expanded metadata and reports with the linear implementation for both identical source/proof and distinct projected-source/original-proof modules. It verifies/runs the selected result and compares diagnostics for four invalid conformance, visibility, return-type and constraint mutations. Existing lookup lifetime/thread-isolation evidence remains applicable because the index implementation is unchanged.

All 208 code-generation tests pass, with 35 manual probes ignored. All paired measurements and separate diagnostics pass. The study uses production-default codegen, unchanged JSON/native hosts, eager/lazy JIT, sync/concurrent GC, and the previous three-pair alternating HTTP procedure. Host caches are warm; these are local development measurements rather than deployment capacity or a claim of special concurrent-GC interaction.

## Measurement procedure

The existing HTTP harness runs three alternating baseline/candidate pairs per mode, reversing order on the middle round. Each invocation checks 4,096 JSON responses after 16 warmups with four clients and pipeline depth 16. It checks guest exit, clean shutdown and concurrent collector invariants. The lazy method counts must remain 870 compiled and 729 uncompiled out of 1,599 selected methods. Both runners receive an untimed `--list` loader qualification before measurement.

An additional pair per mode captures process RSS and malloc-zone snapshots; those timings are not pooled into the primary medians. Two ten-second startup CPU captures follow the timing/memory cohorts. Only CPU tables are exported; raw trace bundles remain local because other trace tables include environment metadata. No task-owned builds or profiling overlap the timing cohort. Background desktop and independent work remain uncontrolled.

The candidate uses the same Xcode SDK as the preceding slice. A separate RavenDoc-only commit landed during work; it does not change the runtime baseline. Exact manifests, source snapshots, commands and report hashes are retained with the completed results. Website publication remains a separate manual operation.

## Paired startup and HTTP results

All 24 timed invocations pass, including 98,688 responses with warmup. Every lazy run retains the expected 870 compiled / 729 uncompiled methods. Three-pair medians:

| Mode / GC | READY before → after (s) | Selection + verification before → after (s) | Warm req/s before → after |
| --- | ---: | ---: | ---: |
| Eager / sync | 12.681 → 11.848 | 8.607 → 7.432 | 1,859 → 1,831 |
| Lazy / sync | 9.887 → 8.798 | 8.687 → 7.617 | 2,130 → 2,014 |
| Eager / concurrent | 14.475 → 13.234 | 8.175 → 6.918 | 1,743 → 1,760 |
| Lazy / concurrent | 9.327 → 8.097 | 7.985 → 6.802 | 1,483 → 1,528 |

All 12 paired readiness changes are improvements. Pairwise reductions (median; observed range):

- Eager / sync: 6.57%; 2.82–11.54%.
- Lazy / sync: 12.10%; 8.21–12.84%.
- Eager / concurrent: 8.63%; 4.40–17.43%.
- Lazy / concurrent: 13.01%; 8.76–13.19%.

The selection/verification reduction occurs before managed invocation and benefits both collectors; it does not establish a special concurrent-GC interaction. Lazy first-response medians remain about 1.7 seconds (sync) and 2.9 seconds (concurrent): first-call compilation still blocks the managed execution thread. Warm throughput is mixed; the short lazy/sync group median is 5.5% lower. A separate longer lazy/sync cohort investigates that possible regression rather than pooling it into these medians.

## Memory diagnostics

All eight separate memory invocations pass (32,896 checked responses including warmup). Live malloc bytes and block counts match exactly at `afterRelease` and `afterInvoke`; the lazy/sync pre-release snapshot differs by three blocks/208 bytes. This is consistent with temporary index release, not retaining new metadata with the lazy compiler.

| Mode | RSS at READY before → after (MiB) | Allocator reservation after invocation before → after (MiB) |
| --- | ---: | ---: |
| Eager / sync | 953.6 → 954.1 | 936.0 → 924.0 |
| Lazy / sync | 930.0 → 846.2 | 920.0 → 836.0 |
| Eager / concurrent | 1058.3 → 1025.0 | 1072.0 → 1032.0 |
| Lazy / concurrent | 930.4 → 847.0 | 924.0 → 836.0 |

Allocator reservation is 12–88 MiB lower in these snapshots. This is one pair per mode and not peak-memory or managed-heap attribution. The earlier selection-only slice had higher reservation in its cohort; do not pool these independent observations or promise the same reduction on every host.

## Longer warm-throughput check

Because the short lazy/sync cohort showed a 5.5% lower group median, run three additional alternating pairs with 65,536 measured responses per process (393,312 checked responses including warmup). Do not pool these runs into the short cohort. Median warm throughput is **2,104.8 → 2,099.4 req/s** (−0.26%); paired changes are −2.00%, −0.26%, and +2.74%. The larger short-run gap is not reproduced, but these local measurements do not prove exact throughput equivalence or production capacity.

Invocation CPU medians, including first-call compilation and warmup, are 33.081 → 32.912 seconds. Every process reports the same 24,033,963 managed allocations and 1,374,175,035 allocated bytes; all retain 870 compiled and 729 uncompiled methods. Keep the short-cohort regressions visible alongside this focused check. No warm-throughput gain is claimed for this optimization.

## CPU attribution and decision

Separate ten-second lazy/concurrent startup captures contain 19,784 baseline and 19,578 candidate CPU samples. Specialization occupies 35.30% → 21.14% of sampled cycles (inclusive); type-definition lookup occupies 18.61% → 2.57% (self). These overlapping shares are diagnostic attribution, not exact phase speedups. Timings from profiler runs are excluded from medians.

Retain the bounded source-index change: all 12 readiness pairs improve; source/proof and diagnostic equivalence pass; the longer throughput check does not reproduce the short-cohort gap; and allocator reservation is lower in the diagnostic pairs. First-call blocking, collector/JIT defaults and managed semantics remain unchanged.

The remaining startup profile emphasizes selection (15.8% self), comparison (9.2% `memcmp` self), hashing (6.3% self) and allocation. Repeated call-target matching is a next investigation candidate, not a proven attribution of all those samples or an implemented follow-up. The separate first-call scheduling and warm GC/root-bookkeeping boundaries remain open.

## Reproduction

`prepare-pairs.py`, `measure.py`, `summarize.py` and `long-warm-command.json` retain the exact local commands. Rebuild the baseline runtime at `7396dfe74` with the recorded SDK and retain its runner before building the archived candidate patch. Reuse the hash-verified guest/native hosts from the preceding cohort, then run the paired harness. Raw results, source snapshots, test/build logs and safe CPU-table exports are hashed in `evidence-index.json`. Full trace bundles remain local. No push or deployment is part of this slice.

Website validation passes all 26 contract tests and the full 6,137-page build, using the pinned publisher recorded in `website-toolchain.json`. Existing API-description warnings remain. Updated tables await the next manual publication; no deployment was performed.
