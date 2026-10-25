# Repeated store-cache insertion screen — 2026-10-25

After the standardized benchmark commit, investigate redundant allocation-base
cache insertion in the concurrent scalar-store adapter. The candidate skips
reinsertion when the payload equals the last range hint. It retains current
ownership admission, header validation, bounds checks and mutation barriers.
The patch is archived here rather than enabled: this screen does not establish
a useful performance improvement.

Three trials per case in each of two sequential cohorts, using the same qualified
allocation-pattern objects, moderate concurrent pacing and 50% watermark:

| Sample | Control CPU ms | Candidate CPU ms | Change |
|---|---:|---:|---:|
| Compute | 62.284 | 62.345 | +0.1% |
| Reusable buffer | 103.101 | 102.097 | -1.0% |
| Steady allocation | 159.286 | 157.587 | -1.1% |
| Bursts | 85.486 | 84.601 | -1.0% |
| Retained window | 330.312 | 330.893 | +0.2% |
| Mixed phases | 161.098 | 162.009 | +0.6% |

All 36 workload invocations validate their outputs. Native validation passes
36 recorded commands, including optimized and UBSan/bounds fixtures plus negative
controls. Exact invocations and source hashes are in the compressed reports.
Cohorts were sequential, not interleaved; these small changes are insufficient
for a speedup claim. No HTTP qualification or default change was made.

## Next viable direction

The larger issue remains per-store admission for scalar storage: the reusable
buffer executes about 8.39 million checked stores for 4.19 million updates, with
negligible marker work. Removing one cache insertion leaves that cost intact.
Investigate preserving compiler-known scalar-array provenance through lowering,
so data and initialization-marker writes can use an appropriate narrow contract.
Before bypassing synchronization, prove which array kinds the marker reads and
which metadata remains immutable. The current marker conservatively scans object
and record payloads; scalar value type alone is not sufficient permission to
omit a barrier. Keep reference arrays, interior byrefs, construction, reclamation
and ownership-change negative tests in the qualification boundary.

This follows the compiler/runtime barrier coordination already researched in
[the industry comparison](../../concurrent-workloads/industry-gc-comparison.md).
The potential benefit is avoiding unnecessary calls and metadata admission; the
cost is a stronger compiler/runtime storage contract and added proof obligations.
No compiler or public contract has changed in this screen.
