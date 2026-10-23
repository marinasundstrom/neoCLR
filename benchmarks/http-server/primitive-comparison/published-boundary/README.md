# Reuse the current-head check at publication — 2026-10-24

## Candidate

The root publication hook checks that its frame equals the current thread's root
head before validating and publishing the snapshot. Ordinary paced execution then
calls `neoclr_gc_boundary_v1`, which retrieves the same thread-local head again.
A private `neoclr_gc_published_boundary_v1` handoff reuses the first proof. The
checked native entry remains unchanged in its admission checks; both entries share
one pacing implementation. Context, nonnull published boundary and cleanup-active
checks remain in the private entry. Budget thresholds, explicit collections, worker
rendezvous, root tracing and descriptor/fault/plan validation are unchanged.

The proof is local to this call, not cached by pointer or across guest execution.
Only root-probe.c uses the private handoff. Between the initial head check and
handoff, the ordinary path performs no guest execution, collection or root mutation.
Worker hooks return through their existing separate path. Site-profiling builds
retain the checked handoff because an external diagnostic sink intervenes after
the first check. Forced-GC and diagnostic-probe modes retain their existing paths.
General native clients must continue using `neoclr_gc_boundary_v1`; the private
hook is not an alternative unvalidated hosting API.

This follows the existing [CLR/runtime research](../../../../docs/design/allocation-metadata-runtime-comparison.md)
principle of distinguishing validated runtime transitions from general native
admission. It does not remove a safepoint or infer that a managed call is
noncollecting. The cost is another private cross-translation-unit function with an
explicit caller obligation; no frame/header growth, metadata cache, compiler
change, managed API change or new generated-object import is required.

## Validation

Extend the maintained pacing fixture to reject a copied/stale frame at the checked
entry, reject null/mismatched contexts at the private entry, and reject both entries
inside resource cleanup. A subsequent publication verifies cleanup guard restoration.
The existing allocation/publication thresholds, cross-context epochs and optional
two-thread isolation remain checked. Six optimized/sanitized GC/pacing/profile
executions pass. Checked-root rejection cases, threaded heaps, recompiled shared
worker graph/fault stress, and eight forced-GC HTTP responses pass.

Four object import checks verify that ordinary paced publication uses the private
hook, site-profiling uses the checked hook, and forced/probe-only builds use neither.
The first validation builds preceded the site-profile fallback; the dedicated
four-variant check and HTTP runs use that final source. Existing async qualification
remains separate evidence; no new full async or Windows matrix is claimed.

An eight-response paced run (two complete invocations, including startup/teardown)
retains the preceding cohort's 38,352 boundaries, 10,132 transient publications,
2,753 allocations, 282,270 frame visits and 96,832 storage-descriptor validations.
These are whole-run counts, not per-response increments. It has 56 collections
versus 57 in the earlier run. The exact interleaving cause was not measured; do not
claim identical collection counts. Focused tests check unchanged budget semantics.

## Measurement and reproduction

The paired runner extracts both root-probe.c and native-gc.c from baseline
`10a7942aa`. Baseline/candidate share the generated application object, host,
owned heap, context reuse, pacing budgets and ten-byte checked response. One/four
isolated owners run forward/reverse: 32 connections, two-second warmup and ten-second
measured windows. CPU/response includes startup/warmup/drain and subtracts client
CPU. No build, test or diagnostic capture overlaps timed load. Short desktop trials
are not sustained capacity, JSON measurements or comparison against Kestrel.

```sh
python3 benchmarks/http-server/primitive-comparison/gc-state/validate.py --output NEW_DIRECTORY
python3 benchmarks/http-server/primitive-comparison/published-boundary/check-hooks.py --output NEW_DIRECTORY
python3 benchmarks/http-server/primitive-comparison/published-boundary/measure.py \
  --build target/primitive-comparison-build/build.json \
  --object target/owned-transient/candidate.o \
  --client target/primitive-comparison-refresh/client --output NEW_DIRECTORY
```

Use the recorded Xcode SDK environment. Compressed raw reports preserve commands,
source/object identities and outcomes. No unrelated website build is required.

## Result: retained

| Isolated owners | Baseline req/s | Candidate req/s | Change | Baseline CPU µs/response | Candidate CPU µs/response | Change |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 2,747.10 | 2,840.00 | +3.4% | 359.78 | 348.00 | -3.3% |
| 4 | 8,159.55 | 8,292.10 | +1.6% | 447.48 | 437.20 | -2.3% |

Means cover two samples per configuration in eight forward/reverse trials. Both
candidate samples outperform both respective baseline samples on throughput and
CPU/response. This is a modest incremental gain. Latency tails are not uniformly
better: one-worker candidate p99 is 14.08/17.18 ms versus baseline 13.45/15.64 ms;
four-worker results are mixed. The change is retained for CPU/throughput, not a
latency claim. Raw trials are in measurement.json.gz. Do not multiply these means
with older cohorts into a claimed cumulative speedup.
