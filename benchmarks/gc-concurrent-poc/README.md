# Concurrent versus synchronous GC POC performance

2026-10-18, macOS ARM64, local process-isolated comparison. The author explicitly
prioritized reaching a performance comparison before covering every special case.
The [bounded reclamation POC](../../docs/experiments/concurrent-gc-walker/README.md#live-reclamation-checkpoint)
is sufficiently functional for this supported workload; production/Serve integration
is not a prerequisite for these first measurements.

## Result

The current snapshot implementation **loses on this workload**. Arithmetic means of
the two 10,000-batch trials per mode:

| Work per batch | Sync elapsed | Concurrent elapsed | Change | Sync process CPU | Concurrent process CPU | Change |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Allocate only | 889.40 ms | 1,041.96 ms | +17.2% | 886.51 ms | 1,144.77 ms | +29.1% |
| Allocate + 50,000 scalar iterations | 1,363.01 ms | 1,538.18 ms | +12.9% | 1,362.47 ms | 1,639.66 ms | +20.3% |

Every trial completes exactly 640,000 measured allocations and 10,000 measured
collections. Checksums match, and post-window cleanup proves all managed allocations
are reclaimed with an empty arena. A prior 2,000-batch pilot also loses in both
workloads; all pilot and repeat results are retained, not selectively discarded.

Owner GC-interruption time (sum, not one continuous pause) in the repeat:

- Allocate only: sync 77.1–77.4 ms; concurrent capture 48.4–50.0 ms plus
  wait/application 70.2–70.3 ms, totaling 118.7–120.1 ms.
- With scalar work: sync 76.6–76.7 ms; concurrent capture 52.3–55.0 ms plus
  wait/application 70.0–77.2 ms, totaling 122.3–132.2 ms.
- Individual interruption p99: sync 8 µs; concurrent 8 µs without scalar work,
  9–11 µs with it. Concurrent has two interruption samples per batch, sync one;
  these percentile distributions alone are not a latency win. Batch p99 is worse:
  109–110 versus 92–93 µs, and 161–166 versus 141 µs respectively.

Peak managed arena extent is 49,152 bytes concurrent versus 38,912 synchronous
(+26.3%). This excludes snapshot storage and the worker stack. Both reserve the same
512 KiB managed backing arena. Per-process peak RSS (includes startup/warmup) is
about 2.21–2.36 MB concurrent versus 1.88–1.90 MB synchronous. These RSS measurements
are macOS bytes and are not portable units for Linux's `ru_maxrss`.

## Protocol and comparability

One rooted chain of 512 reference-bearing objects; each batch allocates 64 unreachable
128-byte buffers and optionally executes a deterministic scalar loop. Root contents
are used and a checksum is observable. The live graph does not change. Each mode
collects once per batch at the same logical boundary:

- Sync collects at batch start, then executes application work.
- Concurrent captures at batch start, executes the same application work while the
  worker traces, then waits for/applies its result before starting the next batch.

Thus the collection cadence and application work are matched; no skipped collections,
lossy arrivals or different allocation totals explain the outcome. Concurrent keeps
new allocations through the cycle. It temporarily retains previous garbage during
allocation, increasing peak extent and potentially allocation-search work. Final
cleanup is outside timing and checks complete reclamation.

The executable is compiled once with `-O2`, no sanitizer, and both paths use the same
validated generated collector. The sync comparator includes POC hook branches/epoch
increments; it is not a separately built untouched production binary. The concurrent
path includes its current per-cycle snapshot allocation/initialization/copy costs,
worker scheduling, mark import, validation, cleanup and sweep. It does not include
startup costs in the measurement window; each fresh process warms up 100 batches.

Trial order is sync/concurrent/concurrent/sync for each work size. Reports include all
commands, hashes, qualification input, process/thread CPU, monotonic elapsed time,
load averages and trial results. The host is uncontrolled; these are short local
microbenchmarks, not confidence intervals or sustained HTTP/Go capacity results.
Timing instrumentation adds overhead, and 1 µs clock granularity limits tiny pauses.
No requests are generated and no HTTP request rate is inferred from batch rate.

## Next practical work

The measurements justify optimizing the current POC before broadening special cases.
Owner capture plus result handling already exceeds the full synchronous collector's
cost here. Snapshot initialization/allocation, mark import/validation and longer
allocation searches are concrete candidates; the latter is an inference from extra
retained blocks, not separately attributed CPU evidence. Measure these costs before
selecting a change. Larger live graphs and different garbage/live ratios can reveal
whether enough tracing work exists to amortize copying and handoff. A stable small
live chain does not predict Serve's invalidation rate or GC costs.

No result here shows that concurrent GC in general is slower or that this approach
cannot be improved. It establishes a measured baseline with actual reclamation and
supported, identical workloads while broader safety coverage remains unfinished.

## Reproduce

Use the repository Xcode SDK environment:

```
python3 docs/experiments/concurrent-gc-walker/run.py --output NEW_QUALIFICATION
python3 benchmarks/gc-concurrent-poc/benchmark.py --validated-build NEW_QUALIFICATION --output NEW_RESULTS --rounds 10000
```

The benchmark checks every qualification input hash before building, so a changed
POC requires refreshed validation. Only the supported POSIX/macOS workload and
reclamation contract are qualified by this experiment.

## Snapshot initialization and larger graph (2026-10-18)

Replace whole-structure zeroing with explicit count/flag initialization. Capture
assigns every used root, conditional pair, parked registration and frame before the
worker reads it; array tails are excluded by counts. A poisoned-storage/reuse test
checks root removal and flag reset, alongside the existing guarded and reclamation
qualification. No production collector or .NET-facing semantics change.

The harness now accepts `--live-count` (default 512) and `--compare-build` pointing
to a prior benchmark directory. The latter records the prior report and binary hash,
requires identical workload source, and alternates baseline/candidate sync/concurrent
trials in forward then reverse order. All workloads retain equal work/checksums and
complete final reclamation. The baseline executable predates the initialization edit.

Arithmetic means of two fresh-process trials, milliseconds:

| Live objects / batches | Scalar iterations | Baseline concurrent elapsed | Candidate concurrent elapsed | Candidate sync elapsed | Candidate concurrent CPU | Candidate sync CPU |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 512 / 10,000 | 0 | 1043.59 | 1044.20 | 880.72 | 1139.00 | 880.26 |
| 512 / 10,000 | 50,000 | 1531.25 | 1514.79 | 1364.47 | 1617.18 | 1364.08 |
| 4096 / 2,000 | 0 | 1425.29 | 1424.43 | 1364.71 | 1533.55 | 1364.49 |
| 4096 / 2,000 | 50,000 | 1532.28 | 1518.15 | 1470.94 | 1627.72 | 1470.35 |

The edit reduces small-graph capture totals from 47.37 to 42.14 ms (allocation-only)
and 48.71 to 37.84 ms (scalar work). End-to-end gains are inconclusive: essentially
zero without scalar work, about 1% with it, and unchanged sync controls also vary.
Keep the simpler initialization, but do not claim a meaningful throughput gain.

At 4096 objects concurrent remains 4.4%/3.2% slower and uses 12.4%/10.7% more CPU
than candidate sync. Total owner interruptions are 133.73/131.68 ms concurrent versus
112.67/113.14 ms sync. Larger graphs narrow the relative gap without producing a win.
Concurrent wait/application totals 99.68/97.81 ms, about 49 microseconds per cycle;
separating worker wait from descriptor validation, mark import and sweep is the next
useful attribution step. Allocation header scanning also grows with the live set,
so the narrowing total-runtime gap alone does not prove better tracing overlap.

Raw [small](snapshot-clear-small.json) and [large](snapshot-clear-large.json) reports
include earlier baseline trials as nested evidence. The initial large baseline
briefly overlapped qualification compilation; retain it, but use the later alternating
comparison above for conclusions. Timed paired runs did not overlap our builds/tests.
These remain short local microbenchmarks, not sustained Serve/Go measurements.

Example after building a baseline with the same workload source:

```
python3 benchmarks/gc-concurrent-poc/benchmark.py --validated-build NEW_QUALIFICATION --compare-build BASELINE_RESULTS --output NEW_RESULTS --rounds 2000 --live-count 4096
```

## Result-application attribution (2026-10-18)

`--phase-timing` enables POC-only owner timestamps; the default compiles the hooks
out. `phaseNs` reports worker take/wait, validation, mark import, cleanup/sweep and
scratch release in that order, excluding warmup and final drain. This diagnostic
workload completes every cycle successfully; it does not characterize rejection
paths. Timestamps include their own overhead and 1 microsecond clock granularity.

For 4096 live objects / 2000 batches, [diagnostic trials](phase-large.json) record:

- Worker take/wait: 0.058–0.068 ms total; worker tracing is already complete here.
- Validation: 40.00–40.56 ms; import: 40.24–40.37 ms.
- Cleanup/sweep: 20.24–20.35 ms; scratch release: 0.45–0.51 ms.
- Combined outer wait/apply: 101.10–101.97 ms.

The [uninstrumented control](phase-control.json) retains all work and reclamation
checks. Diagnostic concurrent elapsed is 1466/1468 and 1564/1555 ms, control
1463/1462 and 1567/1563 ms; small differences change direction. Do not interpret
phase timing as a throughput improvement. The evidence points to repeated owner
heap walks, rather than worker completion wait, for this stable workload. Next
combine mark import's blanket marking and overwrite walks while keeping validation
before mutation and retaining allocations in old free spans or beyond the snapshot.
The collector still follows the existing reachability/cleanup contract; these are
internal POC diagnostics, not a .NET API divergence or production runtime change.

## Single mark-import walk (2026-10-18)

Import now writes captured allocation marks once, retaining new allocations in
captured free spans and beyond the captured extent. It no longer marks every live
allocation and then overwrites captured marks in a second walk. Validation still
finishes before any live state changes; invalidation and owner cleanup semantics
are unchanged. This internal POC optimization preserves the existing reachability
contract and introduces no .NET-facing API divergence.

Fresh UBSan/bounds qualification passes 319 applied cycles, ten rejected results,
100 reclamation-oracle graph comparisons and all prior guarded snapshot tests.
The expanded free-span regression allocates two objects inside one captured free
span, leaves a free remainder, and appends another object: all survive apply, retain
their payloads, and are reclaimed on the next cycle after roots are removed.

Uninstrumented paired means (two trials each), milliseconds:

| Live objects / batches | Scalar iterations | Baseline concurrent elapsed | Candidate concurrent elapsed | Candidate sync elapsed | Baseline owner interruption | Candidate owner interruption |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 4096 / 2000 | 0 | 1459.03 | 1452.47 | 1409.88 | 133.79 | 118.22 |
| 4096 / 2000 | 50,000 | 1559.80 | 1546.78 | 1522.73 | 134.20 | 117.87 |
| 512 / 10000 | 0 | 1089.33 | 1081.65 | 912.38 | 155.82 | 144.10 |
| 512 / 10000 | 50,000 | 1585.00 | 1581.44 | 1410.83 | 151.96 | 145.64 |

At 4096 objects, owner interruption falls 11.6%/12.2% and process CPU falls
0.8%/1.0%, but elapsed gains are only 0.4%/0.8%. Candidate concurrent still takes
3.0%/1.6% longer than candidate sync and consumes 10.1%/8.4% more process CPU.
Small-graph elapsed gains are 0.7%/0.2%; CPU gains 0.8%/0.4%. Sync controls vary,
so these short trials do not establish a substantial sustained throughput gain.

The paired diagnostic confirms the intended phase reduction: import totals fall
from 40.26–40.38 ms to 19.70–20.67 ms. Validation remains about 40 ms and sweep
about 20 ms in most trials. One candidate allocation-only diagnostic takes 1562 ms
versus 1442 ms on its repeat; it includes increased wait/validation/sweep and a
wall/CPU gap. Two baseline diagnostics also record roughly 7 ms worker-wait totals.
All trials are retained; these observations limit precise latency claims. Default
uninstrumented measurements, rather than clock-instrumented elapsed times, determine
the table above. Every trial matches checksums, allocations and collection counts
within its workload and drains the managed heap completely.

Raw evidence: [large graph](import-large.json), [small graph](import-small.json),
[phase comparison](import-phases.json), and [qualification](../../docs/experiments/concurrent-gc-walker/import-results.json).
Each paired report includes baseline provenance. No timed run overlaps our builds
or tests. The next measured owner cost is validation's separate heap-integrity and
captured-descriptor walks. Investigate combining those without weakening rejection
before mutation; do not infer that worker wait is negligible for other workloads.
Production GC and Serve/Go capacity remain unchanged and unmeasured by this slice.

## Combined validation walk (2026-10-18)

Validate live block integrity and captured allocation identity in one read-only
walk. Captured free spans may contain multiple new blocks, but none may cross the
captured span boundary. Check the entire appended tail too. Only after all checks
succeed does import change any live marks. Epoch rejection remains unchanged.
This preserves the existing reachability/cleanup contract and introduces no .NET
API divergence; production collector code is untouched.

Fresh UBSan/bounds qualification records 324 applied cycles, 15 rejected results
and 249,024 reclaimed payload bytes. Five new live-corruption regressions cover an
invalid appended descriptor, valid-but-changed captured payload size, changed mark
state, a new block crossing a captured free-span boundary, and used size exceeding
capacity. Every rejection preserves heap bytes and all GC statistics. Restore the
injected corruption and prove later complete reclamation. Existing split-span/tail
retention, guarded snapshots, graph oracles and transition invalidation checks pass.

Uninstrumented paired means, milliseconds (two trials per version/mode/workload):

| Live objects / batches | Scalar iterations | Baseline concurrent elapsed | Candidate concurrent elapsed | Candidate sync elapsed | Baseline owner interruption | Candidate owner interruption | Candidate sync interruption |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 4096 / 2000 | 0 | 1453.13 | 1429.44 | 1424.70 | 115.64 | 99.69 | 115.62 |
| 4096 / 2000 | 50,000 | 1546.69 | 1529.49 | 1528.17 | 114.95 | 95.44 | 115.59 |
| 512 / 10000 | 0 | 1080.96 | 1069.38 | 916.36 | 143.76 | 131.74 | 79.94 |
| 512 / 10000 | 50,000 | 1579.05 | 1577.85 | 1414.58 | 139.58 | 133.92 | 79.34 |

For the larger graph, owner interruption falls 13.8%/17.0%, elapsed 1.6%/1.1%
and process CPU 1.7%/1.0% relative to the previous concurrent POC. Concurrent elapsed
is within 0.3% of candidate sync here, but process CPU remains 7.7%/7.3% higher
(1533.95/1637.89 ms versus 1423.73/1526.76 ms). Total owner interruption is now
lower than sync on this supported workload; sums of two interruptions per batch
are not a guarantee about individual or HTTP tail latency.

For the small graph, elapsed gains are 1.1%/0.1%, with concurrent still 16.7%/11.5%
slower than sync. These short uncontrolled-host runs do not establish a general
concurrent-GC advantage or a sustained server capacity. Sync controls also vary.

Paired phase diagnostics show validation at 20.71–20.78 ms in three candidate
trials versus 40.39–40.63 ms in three baseline trials. Retain the fourth trial of
each: candidate validation 24.29 ms, baseline 44.44 ms, with increased wait and
other phase costs. One candidate diagnostic takes 1566 ms versus its 1422 ms repeat;
a baseline diagnostic takes 1634 ms versus 1550 ms. Do not selectively discard
these or substitute instrumented elapsed for the uninstrumented comparison above.
Import and sweep are each roughly 20 ms in the cleaner diagnostic trials.

Raw evidence: [large graph](validation-large.json), [small graph](validation-small.json),
[phase comparison](validation-phases.json), [qualification](../../docs/experiments/concurrent-gc-walker/validation-results.json).
All checksums, work and collection counts match within each workload; final
reclamation succeeds. Timed trials did not overlap our builds or tests. Existing
ASan/ThreadSanitizer startup limitations remain; UBSan/bounds is not race detection.

The next useful evaluation is varying collection cadence and graph/root mutation,
including invalidated cycles, before extrapolating this stable chain to Serve.
The repeated owner walks are reduced; capture copying, allocation searches while
garbage awaits reclamation, and invalidation/fallback costs remain relevant.

## Indexed registered-root capture (2026-10-18)

The author asked to continue until there were real improvements. Inspection found
that capture validated the entire heap separately for every host handle, resource
owner and owned parked registration. This was a POC scaling defect: the synchronous
collector already uses a temporary block index for these checks.

Capture now builds that existing 256-bucket index once when registrations exist,
validating the full heap during construction. Exact-base checks use the index until
capture returns. The owner performs no allocation or collection during this scope;
the index never escapes or becomes worker-owned. The no-registration path retains
its original full integrity walk without building the index. Costs are bounded
scratch storage (256 block pointers plus 32 intern-seed pointers, 2304 bytes on this
host), index construction and bounded bucket lookup. No retained pointer cache,
managed API, production collector or .NET semantic contract changes.

New regression cases preserve exact-base rejection for interior/header/end/outside
pointers and invalid conditional owners. A valid registered root cannot hide a
corrupt trailing descriptor. Valid strong and conditional captures pass, alongside
the prior protected-memory, parked-owner, invalidation and reclamation suite.
Fresh UBSan/bounds reports 324 applied cycles, 15 rejected results and 249,024
reclaimed payload bytes. ASan/ThreadSanitizer startup limitations remain.

The benchmark now accepts `--host-roots` (default 0, at most 256). Both modes register
the same strong handles at evenly spaced graph nodes before timing and release them
after timing, then prove complete reclamation. The live graph, allocation batches,
collection cadence and optional scalar work remain identical. Handles are stable;
this measures root-count scaling, not task churn, invalidation frequency or Serve.
The baseline is the immediately preceding POC with the same new workload source,
compiled before the capture change. All compared binaries use `-O2` without phase
clocks or sanitizers. Each group runs baseline/candidate sync/concurrent in forward
then reverse order. Timed runs do not overlap our builds/tests.

Uninstrumented arithmetic means of two trials each, milliseconds:

| Roots | Live nodes | Batches | Scalar iterations | Baseline concurrent elapsed | Candidate concurrent elapsed | Candidate sync elapsed | Baseline concurrent CPU | Candidate concurrent CPU |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 4096 | 2000 | 0 | 2787.44 | 1431.91 | 1429.55 | 2897.89 | 1539.09 |
| 64 | 4096 | 2000 | 50000 | 2930.75 | 1529.74 | 1533.18 | 3019.56 | 1640.05 |
| 64 | 4096 | 5000 | 0 | 6979.61 | 3575.50 | 3569.66 | 7253.58 | 3848.44 |
| 64 | 4096 | 5000 | 50000 | 7246.11 | 3818.80 | 3826.82 | 7517.52 | 4092.87 |
| 8 | 4096 | 2000 | 0 | 1601.96 | 1431.74 | 1422.99 | 1710.39 | 1537.73 |
| 8 | 4096 | 2000 | 50000 | 1692.86 | 1528.73 | 1521.81 | 1801.14 | 1634.39 |
| 0 | 512 | 10000 | 0 | 1064.44 | 1063.08 | 914.45 | 1155.12 | 1151.61 |
| 0 | 512 | 10000 | 50000 | 1574.07 | 1575.03 | 1424.14 | 1665.16 | 1663.75 |

The longer 64-root repeat reduces elapsed 48.8%/47.3% and process CPU 46.9%/45.6%
for allocation-only/scalar work. Owner capture totals fall from 3497/3501 ms to
89.1/88.6 ms; owner interruption totals fall from 3654/3661 ms to 244/245 ms.
The shorter 64-root run agrees. Eight roots already reduce elapsed 10.6%/9.7%.
The zero-root control changes elapsed by -0.1%/+0.1%, consistent with no material
regression and ordinary host variation. No trial is excluded.

This is a substantial improvement over the preceding concurrent POC, not evidence
that concurrency itself is faster than synchronous collection. In the longer run,
candidate concurrent elapsed is approximately equal to candidate sync (3576 versus
3570 ms; 3819 versus 3827 ms), while process CPU is still 8.0%/7.2% higher. The
zero-root small graph remains slower than sync. These are short, local GC workloads;
no HTTP requests or Go baseline were measured and no sustained server rate follows.

Every group matches work, checksum and collection counts; final heaps are empty.
The improvement removes redundant full-heap validation per registered root rather
than reducing work, deferring collections, skipping reclamation or adding a hidden
synchronous retrace. Do not interpret 64 registered roots as 64 HTTP requests: actual
Serve root counts, lifetimes and invalidation patterns require measurement.

Raw reports: [64 roots](root-index-64.json), [longer repeat](root-index-repeat.json),
[8 roots](root-index-8.json), [zero-root control](root-index-0.json), and
[qualification](../../docs/experiments/concurrent-gc-walker/root-index-results.json).
Reports retain baseline provenance and all outputs. Example:

```
python3 benchmarks/gc-concurrent-poc/benchmark.py --validated-build NEW_QUALIFICATION --compare-build BASELINE_WITH_MATCHING_WORKLOAD --output NEW_RESULTS --rounds 5000 --live-count 4096 --host-roots 64
```

Next test root churn/invalidation and realistic collection cadence before runtime
integration. The capture scaling defect is fixed; it does not establish support
for every interpreter/JIT/AOT access path or a production concurrent collector.

## Rotating graph and invalidation cost (2026-10-18)

Add opt-in `--churn-every N`. With N > 0, move the first node to the tail of the
rooted chain each batch, preserving the same live objects, and replace one strong
host-root value every N batches. A host replacement invalidates the POC epoch;
ordinary managed pointer/root transfers do not. Both sync and concurrent perform
the same application operations. Default zero leaves the original stable workload.

The benchmark now explicitly falls back to a synchronous collection after a rejected
concurrent result, charging all waiting, discarded work and fallback time inside
the measurement window. This is benchmark policy, not production integration or a
new runtime API. Successful collections remain exactly one per batch: either an
applied snapshot or a fallback. Report replacement, discard and fallback counts;
assert exact correspondence. A skipped failed collection cannot inflate throughput.
Sync collects at batch start; invalidated concurrent cycles collect at batch end,
so transient garbage lifetimes differ. Final managed allocation/reclamation totals
must still agree and the heap must be empty.

After timing, traverse and verify the complete graph in its expected rotated order,
including its null terminator, then release handles and prove final reclamation.
This catches lost edges or cycles that checking only the first/last objects could
miss. Five focused boundary runs cover a one-node graph, stable no-handle sync and
concurrent modes, and a churn period not dividing the 100 measured batches.
No runtime collector code changed; existing qualification is reused. This preserves
managed reachability semantics, with no .NET-facing divergence.

4096 live nodes, 64 strong handles, 2000 measured batches, 128,000 allocations:
means of two trials per mode/workload, milliseconds:

| Discarded cycles | Scalar iterations | Sync elapsed | Concurrent elapsed | Sync CPU | Concurrent CPU | Sync owner interruption | Concurrent owner interruption |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0/2000 | 0 | 1429.27 | 1425.26 | 1428.08 | 1535.94 | 119.88 | 96.99 |
| 0/2000 | 50000 | 1526.97 | 1525.93 | 1526.09 | 1636.17 | 120.20 | 97.17 |
| 500/2000 | 0 | 1430.40 | 1438.47 | 1429.69 | 1548.15 | 120.54 | 111.83 |
| 500/2000 | 50000 | 1535.65 | 1573.85 | 1534.29 | 1669.74 | 120.68 | 118.44 |
| 2000/2000 | 0 | 1455.17 | 1474.74 | 1451.20 | 1581.90 | 120.28 | 153.66 |
| 2000/2000 | 50000 | 1549.33 | 1576.08 | 1548.63 | 1683.21 | 119.11 | 153.54 |

Rotation without measured host replacement (`N=3000`; one replacement occurs during
warmup) retains the earlier large-graph result: near-equal elapsed, about 7–8% more
CPU, and about 19% less total owner interruption. This demonstrates supported
pointer/root mutation, not changing live-set size or arbitrary native access.

Every-fourth-batch replacement yields exactly 500 discards and 500 fallback
collections; every-batch replacement yields exactly 2000 of each. In the latter,
concurrent costs 1.3%/1.7% more elapsed, 9.0%/8.7% more CPU, and 27.8%/28.9% more
owner interruption than sync. Wasted worker work and fallback eliminate the pause
benefit. A quarter-invalidated scalar trial takes 1599.65 ms versus 1548.05 ms on
its repeat; retain both, so no precise tail-latency advantage is claimed.

The initial quarter-invalidated run preceded the explicit full-chain assertion and
is retained separately, not substituted for the checked repeat. Reports:
[initial quarter](churn-quarter.json), [checked quarter](churn-quarter-checked.json),
[all invalidated](churn-all.json), [rotation only during measurement](churn-rotation.json),
[boundary checks](churn-boundaries.json). No trial overlaps our other builds/tests.
These short local runs are not sustained HTTP/Go capacity evidence, nor a measurement
of how often actual Serve invalidates snapshots. Raw reports retain source hashes.

Next audit unnecessary invalidations narrowly. Current POC hooks invalidate even a
read from an unchanged strong handle, whose value was already a snapshot root.
Conditional reads can rescue snapshot-white values and must remain protected;
creation/replacement/release, teardown and root ownership changes require separate
reasoning and tests. Removing a hook without demonstrating safe ownership and
publication is not an optimization. Root churn data motivates this investigation,
but this slice changes no invalidation rule and claims no new speedup.

Example:

```
python3 benchmarks/gc-concurrent-poc/benchmark.py --validated-build VALIDATED_BUILD --output NEW_RESULTS --rounds 2000 --live-count 4096 --host-roots 64 --churn-every 4
```

## Preserve snapshots across unchanged strong-handle reads (2026-10-18)

The generated POC no longer invalidates a snapshot on a successful
`neoclr_gc_host_root_read_v1` from an unchanged strong handle. Its value was captured
as a strong root, so exposing that value cannot rescue a snapshot-white object.
Any creation, replacement or release still advances the epoch; a later read does
not reset it. Conditional reads can rescue otherwise unreachable values and still
invalidate, as do failed reads. Callback-specific read hooks and all other audited
transitions remain unchanged. This is a scoped native POC rule, not production
integration or a managed API change. It preserves the familiar strong-handle
retention semantics rather than changing reachability or ownership contracts.

New reclamation regressions demonstrate successful strong reads with garbage freed,
rejection after replacement/creation followed by a strong read, and conservative
failed-read rejection. Existing conditional-resurrection, parked-resumption,
same-address reuse, error/cancel and protected-memory cases remain passing.
Fresh UBSan/bounds records 329 applied cycles, 18 discards and 249,088 reclaimed
payload bytes. Existing ASan/ThreadSanitizer startup limitations are unchanged.

The workload adds `--read-every N`, default zero. Both versions/modes read the same
handles and verify returned values against expected handle contents. Reads and
replacements are separately counted. Expected discard counts follow the epoch;
five additional boundary checks assert explicit numeric outcomes for stable/no-read,
read-only, every-cycle replacement and non-dividing read/replacement periods.
Fallback collections are still fully charged; all work, graph order and final
reclamation checks remain. This benchmark uses strong handles, not conditional
reads, and does not establish real Serve read/invalidation frequency.

4096 nodes, 64 handles, 2000 batches, one read per batch. `paired` and `repeat`
perform no handle replacement; `mixed` also rotates the graph and replaces a handle
every fourth batch. Baseline always discards 2000 cycles. Candidate discards zero
for read-only runs and exactly 500 for mixed runs. Arithmetic means of two trials,
milliseconds:

| Run | Scalar iterations | Candidate discards | Baseline concurrent elapsed | Candidate concurrent elapsed | Candidate sync elapsed | Baseline owner interruption | Candidate owner interruption |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| paired | 0 | 0 | 1491.39 | 1430.66 | 1447.38 | 170.42 | 97.00 |
| paired | 50000 | 0 | 1554.15 | 1527.42 | 1521.95 | 153.34 | 97.20 |
| repeat | 0 | 0 | 1453.74 | 1426.95 | 1419.05 | 153.69 | 96.93 |
| repeat | 50000 | 0 | 1554.83 | 1528.81 | 1521.12 | 153.77 | 97.39 |
| mixed | 0 | 500 | 1464.23 | 1445.41 | 1429.82 | 154.26 | 112.47 |
| mixed | 50000 | 500 | 1561.27 | 1539.60 | 1531.10 | 154.82 | 111.92 |

The read-only repeat reduces owner interruption 36.9%/36.7%, elapsed 1.8%/1.7%
and process CPU 1.6%/1.5%. Mixed runs reduce owner interruption 27.1%/27.7%, elapsed
1.3%/1.4% and CPU 1.2% in both cases. The first allocation-only baseline contains
a 1521 ms run versus a 1461 ms companion, inflating its mean improvement; retain
both and use the repeat to describe the more modest elapsed effect. Sync controls
also vary. No substantial throughput or precise tail-latency claim follows.

Candidate concurrent still consumes about 8% more process CPU than candidate sync
in the repeat (1537/1639 ms versus 1418/1520 ms), with nearly equal elapsed.
Avoiding unnecessary fallback restores the owner-interruption benefit for this
supported read pattern; it does not make all host access safe or concurrent GC
universally faster. Source generation changes only the isolated POC; installed
collector selection and production behavior remain unchanged.

Raw evidence: [first pair](strong-read-paired.json), [repeat](strong-read-repeat.json),
[mixed operations](strong-read-mixed.json), [boundary checks](strong-read-boundaries.json),
[qualification](../../docs/experiments/concurrent-gc-walker/strong-read-results.json).
Reports retain baseline qualification and binary provenance, all outputs and counts.
Timed measurements did not overlap our builds/tests. No HTTP/Go rate was measured.
Further read-hook changes need separate publication/ownership reasoning; callback
reads remain conservative until audited. Example with a matching baseline build:

```
python3 benchmarks/gc-concurrent-poc/benchmark.py --validated-build NEW_QUALIFICATION --compare-build BASELINE_WITH_MATCHING_WORKLOAD --output NEW_RESULTS --rounds 2000 --live-count 4096 --host-roots 64 --read-every 1 --churn-every 4
```

## Callback-read invalidation audit (2026-10-18)

Successful ordinary and suspended callback reads now preserve the generated POC's
snapshot epoch. The shared callback reader already rejects conditional handles and
validates a strong handle's value, full heap integrity, object kind/size, callback
tag and fault state. These functions return an object; they do not invoke a callback
or resume parked roots. The value was captured as a strong root, and creation,
replacement or release still invalidates independently. All failed callback reads,
including rejected entry states, continue to invalidate. No epoch is restored or
reset on a successful read. All production sources and APIs remain unchanged.

This reuses the familiar strong-reference retention argument without changing
managed/.NET-facing semantics. Native callback entry restrictions remain intact:
ordinary reads reject active frames for the context; suspended reads require the
current head frame and matching context. Keeping the same checks is part of this
change, not a deferred optimization.

Eleven new reclamation cases cover both successful read variants, ordinary reads
with an active frame, missing/mismatched suspended boundaries, wrong tag/size,
conditional handles, null output, fault state, and replacement followed by a valid
read. Successful cases preserve the callback object and reclaim unrelated garbage;
rejected cycles leave heap bytes and collection counts unchanged. Fresh UBSan/bounds
qualification passes 342 applied cycles, 27 discards and 249,608 reclaimed payload
bytes, including previous conditional-resurrection and stale-result regressions.
ASan/ThreadSanitizer startup limitations remain.

`--callback-reads` selects suspended callback reads in the benchmark, which already
has a published frame. Graph nodes receive the required 24-byte object shape and
callback tag. These are synthetic callback-shaped objects: no function is invoked,
so this is not a callback-dispatch or handler benchmark. Both versions perform the
same reads and validate returned handle values. Default ordinary host reads are
unchanged. Six explicit boundary runs cover default modes, a one-node graph,
every-cycle replacement, non-dividing periods and ordinary strong reads.

4096 live nodes, 64 handles, 2000 batches, one callback read per batch. `mixed` also
rotates the graph and replaces a handle every fourth batch. Means of two trials per
mode/workload, milliseconds:

| Run | Scalar iterations | Candidate discards | Baseline concurrent elapsed | Candidate concurrent elapsed | Candidate sync elapsed | Baseline owner interruption | Candidate owner interruption |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| paired | 0 | 0 | 1482.68 | 1455.52 | 1432.96 | 155.85 | 97.23 |
| paired | 50000 | 0 | 1582.14 | 1547.64 | 1542.34 | 153.75 | 96.75 |
| mixed | 0 | 500 | 1489.99 | 1461.61 | 1458.59 | 155.56 | 111.48 |
| mixed | 50000 | 500 | 1582.45 | 1564.62 | 1555.61 | 154.61 | 112.00 |

Baseline always discards 2000 cycles; candidate discards zero in read-only runs and
exactly 500 in mixed runs, each with a fully charged synchronous fallback. Read-only
owner interruption falls 37.6%/37.1%; elapsed falls 1.8%/2.2% and CPU 1.6%/2.0%.
Mixed interruption falls 28.3%/27.6%; elapsed falls 1.9%/1.1% and CPU 1.7%/1.0%.
These modest elapsed changes are subject to host variation; one mixed baseline
sync run takes 1549.83 ms versus 1458.87 ms on its companion, and both are retained.
No precise tail-latency or substantial throughput advantage is claimed.

Candidate concurrent remains more CPU-expensive than candidate sync, despite
reduced interruption. Every group matches allocations, successful collections,
application checksum and expected graph order, and drains all managed objects.
No timed run overlaps our builds/tests. No HTTP/Go rate or production concurrency
claim follows. This closes the callback-read audit only; actual callback execution,
live-set churn and runtime integration still need their own evidence.

Raw reports: [read-only](callback-read-paired.json), [mixed](callback-read-mixed.json),
[boundary checks](callback-read-boundaries.json), and
[qualification](../../docs/experiments/concurrent-gc-walker/callback-read-results.json).
Reports retain baseline qualification/binary provenance and every trial. Example:

```
python3 benchmarks/gc-concurrent-poc/benchmark.py --validated-build NEW_QUALIFICATION --compare-build BASELINE_WITH_MATCHING_WORKLOAD --output NEW_RESULTS --rounds 2000 --live-count 4096 --host-roots 64 --read-every 1 --callback-reads --churn-every 4
```

## Invoked native fixture with live-object replacement (2026-10-18)

Move beyond reads and rotation of permanently live objects. A known C callback body
is invoked with validated callback-shaped state. That state holds the head and tail
of a 4096-node live window. Each invocation consumes the oldest node's expected
sequence number, allocates a replacement, publishes it through the rooted state,
unlinks the retired node and updates an observable checksum. Object identities
change continually; live-set size stays fixed. This is an explicit native test
fixture, not dispatch through a function pointer stored in a Raven delegate, not
managed callback execution, and not scheduler/HTTP integration.

The same fixture body is shared with qualification. Eighty callback invocations
replace 1360 objects in a 128-node window. The first invocation runs while the worker
is deterministically paused after 63 scanned objects. After each invocation and
application, verify the full surviving sequence. Heap extent is bounded throughout
by `(live nodes + 2 * replacements per callback + 1) * 56` bytes; this prevents a
collector that only reclaims during final cleanup from passing. Release the handle
and prove an empty heap and matching lifetime allocation/reclamation counts.
Fresh UBSan/bounds qualification passes 423 applied cycles, 27 discards and 285,344
reclaimed payload bytes, including all earlier oracle/rejection tests. Existing
ASan/ThreadSanitizer startup limits remain.

`--callback-churn --replace-count N` selects the new workload. Both modes invoke the
same body, allocate the same number of nodes, collect once per invocation, and run
the same optional scalar loop. Sync collects at invocation start; concurrent captures
there and applies after the body. Every concurrent cycle must apply: no invalidated
cycles or skipped/fallback collections explain the results. After timing, verify
the full sequence and exact survivor count, then drop the callback root/handle and
drain completely. Peak extent is checked each batch and recorded. Fixed 512 KiB
managed capacity and a 100-batch warmup are shared. Six boundary checks include a
one-node window, more replacements than nodes and non-dividing window sizes.
A short default-workload run verifies the harness's original path still works;
it is functional validation, not a new capacity measurement.

All trials use one `-O2` executable without sanitizers or phase clocks, in
sync/concurrent/concurrent/sync order. 64 replacements uses 2000 measured invocations
(128,000 allocations); eight replacements uses 10,000 (80,000 allocations) and has
an independent repeat. These configurations differ in GC cadence per allocation;
only compare sync/concurrent within each row. Means of two trials, milliseconds:

| Replacements / run | Scalar iterations | Sync elapsed | Concurrent elapsed | Sync CPU | Concurrent CPU | Sync owner interruption | Concurrent owner interruption |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 0 | 1448.83 | 1485.28 | 1443.13 | 1573.70 | 117.29 | 101.93 |
| 64 | 50000 | 1550.38 | 1537.18 | 1545.02 | 1644.15 | 117.49 | 94.99 |
| 8 | 0 | 1540.64 | 1384.35 | 1505.65 | 1906.55 | 595.34 | 466.16 |
| 8 | 50000 | 2007.66 | 1883.36 | 1996.77 | 2407.22 | 582.77 | 463.35 |
| 8-repeat | 0 | 1498.47 | 1386.19 | 1492.06 | 1908.88 | 584.11 | 466.23 |
| 8-repeat | 50000 | 1997.33 | 1882.21 | 1992.05 | 2406.33 | 581.61 | 462.52 |

The eight-replacement repeat is 7.5%/5.8% faster in elapsed time, with 20.2%/20.5%
less total owner interruption, but uses 27.9%/20.8% more process CPU. This is a
measured concurrency tradeoff on a supported frequent-GC workload: parallel tracing
can shorten elapsed execution while doing more total work. It is not a CPU-efficiency
win. Initial eight-replacement results also favor concurrent elapsed, but include a
1594.68 ms sync run versus 1486.59 ms on its companion; retain it and use the repeat
for the more modest headline. The allocation-heavy 64-replacement results are mixed
(2.5% slower without scalar work, 0.9% faster with it), not a consistent elapsed win.

Peak managed extent is 230,328 versus 229,880 bytes at eight replacements, and
236,600 versus 233,016 at 64: concurrent retains an extra batch of floating garbage.
These figures exclude the copied heap, capture metadata, index and worker stack;
raw process RSS includes native overhead. Allocation searches still walk retained
blocks, and all concurrent copy/import costs remain in the measurement window.
No timed run overlaps our builds/tests. Checksums, allocations and successful
collections match within every group; all final heaps are empty.

This demonstrates concurrent reclamation with ongoing native fixture execution and
live-object replacement. The next integration gap is the real managed execution
boundary: Raven invocation, async suspension/resumption, fault/cancellation paths
and memory-pressure admission are not exercised here. Allocations use the existing
non-collecting native allocator within a prechecked budget; automatic pressure
fallback and variable live-set size remain unqualified. No sustained Serve/Go rate
or production collector eligibility is established, and no production GC code changed.

Evidence: [64 replacements](callback-churn-64.json), [eight](callback-churn-8.json),
[eight repeat](callback-churn-8-repeat.json), [boundary checks](callback-churn-boundaries.json),
[default harness check](callback-churn-default-control.json),
[qualification](../../docs/experiments/concurrent-gc-walker/callback-churn-results.json).
The qualification hashes the shared callback body; reports also hash the workload.
Reproduce:

```
python3 docs/experiments/concurrent-gc-walker/run.py --output NEW_QUALIFICATION
python3 benchmarks/gc-concurrent-poc/benchmark.py --validated-build NEW_QUALIFICATION --output NEW_RESULTS --rounds 10000 --live-count 4096 --callback-churn --replace-count 8
```
