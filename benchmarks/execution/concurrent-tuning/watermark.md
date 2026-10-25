# Complete epochs before mutation-log exhaustion

2026-10-25. Follow-up to [concurrent-specific tuning](README.md). The largest budgets
were faster but often canceled marker work after mutation-log exhaustion. This
prototype uses occupancy as a scheduling input instead of only enlarging the log.
It is opt-in, owner-only, and preserves synchronous/default behavior.

## Implemented boundary

`NEOCLR_LIVE_FINISH_WATERMARK=1..99` selects a percentage of the mutation log.
After a successful admitted write, the gate records a sticky owner-written hint
when occupancy reaches the threshold. The live adapter requests the existing
publication pacing slow path. **No collection occurs inside the store**, where
intermediate inputs/results may not yet be published as roots. The next admitted
root publication completes the epoch using existing remark, validation and sweep.

Marker draining never accesses the hint; only the owner writes/reads it. Reset is
at the next gate begin. No extra mutex or atomic is added to the common store path.
The marker may drain after the threshold is reached: this deliberately does not
retract the owner's request. Overflow before the next safe boundary still cancels
and retries safely. The threshold is not a guarantee against an oversized bulk
write. Shared-writer builds reject the option rather than applying owner-local
pacing across participants. Heap reclamation remains on the owner.

This is an occupancy-triggered finish rule, not a full adaptive controller, a new
managed API, concurrent sweep or a default change. It follows the general industry
lesson of adapting collection timing to resource pressure; it does not reproduce
G1 or Go pacing algorithms. See the existing
[primary-source comparison](../concurrent-workloads/industry-gc-comparison.md).

## Interleaved HTTP evidence

Five rounds, four clients, 4,096 measured requests plus 16 warmups. Same compiled
objects, 1 MiB arena, 4,096-word mutation log and 4096/65536 allocation/publication
budgets for all three concurrent modes. 123,360 responses pass. No instrumentation
is enabled beyond the existing common runtime statistics.

| Sample / policy | Requests/s | Process CPU s | Completed / canceled epochs | Store cancellations | Peak arena bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Plain / control | 4,324 | .9447 | 25 / 193 | 192 | 1,048,576 |
| Plain / 50% | 4,356 | .9475 | 232 / 1 | 0 | 608,864 |
| Plain / 75% | 4,316 | .9524 | 215 / 1 | 0 | 675,312 |
| JSON / control | 2,887 | 1.4180 | 39 / 343 | 342 | 1,048,576 |
| JSON / 50% | 2,866 | 1.4378 | 401 / 1 | 0 | 691,456 |
| JSON / 75% | 2,817 | 1.4439 | 366 / 1 | 0 | 814,592 |

Values are separate medians, so median completed + canceled need not equal median
begun; every individual trial balances. One cancellation remains at shutdown.
The 50% policy lowers arena occupancy about **42% plain / 34% JSON**, with no store
cancellations. Plain throughput +0.7% / CPU +0.3%; JSON throughput -0.7% / CPU +1.4%.
This is a completion/memory tradeoff at roughly similar performance, **not a CPU
or throughput win**. Do not confuse occupancy with allocated bytes, retained native
metadata, or total process RSS. Managed allocation statistics remain in the reports.

Sixty standalone AOT runs (tasks, collections, arrays; ten interleaved rounds at
ordinary default budgets) also pass. CPU changes are +0.6%, +0.1%, -0.3% respectively:
approximately flat, with no general speedup claim. Their shorter budgets differ
from this HTTP experiment intentionally.

## Safety, JIT and pause evidence

- Optimized/UB-bounds live fixtures pass mutation, deletion, delayed writes, arrays,
  record/object scanning, overflow recovery and randomized liveness comparisons.
  Existing deliberately broken deletion logs and cached proofs fail.
- New deterministic test verifies the threshold, no collection before publication,
  a sticky hint after draining, retention of a deleted edge through finish, and
  next-epoch reset. Removing the pacing request fails. Shared-writer admission and
  invalid thresholds fail compilation. Disabled gate object is byte-identical to
  the previous committed control in the same ARM64 optimized build.
- JSON managed JIT passes 512 responses with immutable-byref/stack-elision options
  retained: 518 begun, 517 completed, one canceled, zero active at teardown. This
  verifies execution/lifetime compatibility; the JIT harness has frequent explicit
  collection/idle boundaries, so these counts do not establish watermark frequency
  or JIT throughput improvement.
- Separate owner-only profiles validate 12,480 responses. Both policies have p99
  owner-GC operations in [131.072, 262.144) microseconds. Observed maximum is
  175→162 us plain and 377→327 us JSON. Small diagnostic cohorts and coarse buckets
  do not prove a tail-latency win; no throughput claims use these runs.

The initial pause cohort exposed a profiler omission: `candidate_cancel` was
instrumented, but `live_store_cancel_epoch` was not. The histogram/epoch validator
correctly rejected it. Instrumentation now includes both cancellation paths, with
an added deterministic UB/bounds test. The rerun balances every cancellation. The
original `watermark-pauses-*` reports are retained only as rejected diagnostic
evidence; use `watermark-pauses-v2-*` and the validated summary. Earlier reports
without store cancellations are unaffected; cancellation-heavy old pause reports
must be regenerated, not inferred correct from request success.

## Reproduction and next decision

HTTP: add `--compare-watermarks 50 75 --allocation-budget 4096
--publication-budget 65536 --modes live-inline-index live-watermark-50-inline-index
live-watermark-75-inline-index` to the established HTTP command. Ordinary driver
supports `--compare-watermarks 50` with modes `live-hybrid-inline` and
`live-watermark-50-hybrid-inline`. Native validation and JIT qualification accept
`--finish-watermark 50`; JIT qualification also exposes the pacing budgets.
Exact commands, hashes, errors, output and counters are archived under
[watermark-evidence](watermark-evidence/).

Retain 50% as an experimental candidate. Next measure log high-water/finish causes
and combine the rule with the moderate budget rather than assuming one universal
percentage. Qualify sustained multi-owner loads before default selection. The
larger structural task remains bounded owner-safe reclamation: split sweep work
without racing allocation, losing roots or bypassing cleanup/ownership contracts.
