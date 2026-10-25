# Bitmap seed proof reuse screen — 2026-10-25

Test whether final collection can validate snapshot allocation boundaries through
its existing owned allocation bitmap instead of sparse-index lookup. Preserve all
old span/size/kind/state checks before snapshot marks. Caller-managed arenas retain
the sparse path: the first native run exposed that boundary and failed; the
corrected candidate passes all 34 native validation commands including UB/bounds.

A differential fixture compares 18,000 fragmented-layout/invalid-identity cases in
each optimized and sanitized build. Forged interior headers reject, and deliberate
removal of bitmap membership or kind checks makes the fixture fail. No metadata,
managed representation or compiler contract changes. The archived patch contains
implementation, measurement switches and the differential test; apply from repo
root to reproduce `test-index-seed.py` and the commands in the reports.

Five interleaved rounds per mode validate 60 standalone invocations and 82,240
HTTP responses, on top of index-only store lookup and 1024/16384 pacing/50% watermark.

| Workload | Control CPU s | Bitmap CPU s |
|---|---:|---:|
| Compute | .065848 | .065908 |
| Reuse buffer | .113689 | .113753 |
| Steady | .158152 | .158035 |
| Bursts | .083844 | .083482 |
| Retained window | .339352 | .334047 |
| Mixed phases | .159040 | .159095 |
| Plain HTTP | 1.039755 | 1.032592 |
| JSON HTTP | 1.525094 | 1.516314 |

HTTP median requests/s: 4023.7→4069.3 plain and 2711.9→2724.0 JSON. Whole-workload
CPU reductions are under 1% except retained-window (1.6%). These results do not
justify another enabled policy branch; archive without promotion. No pause or
concurrent-default improvement is claimed. Reuse the safety proof if a future
larger index simplification needs it. Existing .NET/Go allocation metadata research
in `concurrent-workloads/allocation-index-results.md` remains the design basis.

The header's ownership comment was expanded during measurement; generated sources
and command input hashes remain in raw reports. No algorithm changed between modes.
The broader lesson is to measure whole execution: local proof reuse alone does not
remove the owner-side walks or sweep. Proceed with the author-requested separate
HTTP persistent-connection/pipelining investigation while preserving GC regression
coverage and this negative performance evidence.
