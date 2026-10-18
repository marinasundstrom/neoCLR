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
