# Scalar stores with synchronous and concurrent collection — 2026-10-24

The indexed adapter removes its repeated linear lookup cost, but the full live-GC
candidate remains more expensive than synchronous collection in this small-work
fixture. At 16,384 objects, two cohorts put candidate process CPU at about 2.6–2.8×
synchronous and wall time at about 2.5–2.9× when writes need not wait for the marker.
These results do not establish an application throughput gain or lower pause tails.

## Experiment and scopes

`store-overlap.c` keeps a linked graph live, performs 32 pseudo-random scalar writes
and one collection per epoch, for 1,000 epochs. Initial allocation and final teardown
are excluded from process CPU/wall intervals. No allocation or object death occurs
inside that interval. Final traversal validates every graph edge and object identity;
clearing the root then collects the entire graph. This is a GC cost probe with very
little independent application work, not a representative request handler.

- `sync`: ordinary direct scalar stores, then the production synchronous collector.
- `idle`: begin a concurrent epoch and wait for its first pass before stores, then
  finish. Its begin timing includes that deliberate wait.
- `overlap`: begin and immediately perform guarded stores, then finish. Scheduling
  permits overlap but does not prove that the marker runs during every write.

The same native compiler options and graph checks apply. The driver builds before
measurement, runs UBSan/bounds validation for all three modes, then executes
sync/idle/overlap/overlap/idle/sync at each heap size. Cost runs are optimized without
sanitizers or per-gate profiling. Per-epoch wall samples cover begin, the complete
32-store window and finish; the store window includes application work and checks,
so it is not a pure blocking-pause measure. Quantiles are checked against retained
raw arrays. They are observed process timings, not HTTP latency or confidence bounds.

## Scoped cohort

Arithmetic means of two runs, milliseconds for the full 1,000-epoch interval:

| Objects | Mode | Process CPU ms | Wall ms | Marker CPU ms |
| --- | --- | --- | --- | --- |
| 1,024 | sync | 15.88 | 16.49 | 0.00 |
| 1,024 | idle | 49.42 | 50.15 | 17.80 |
| 1,024 | overlap | 49.88 | 49.49 | 18.23 |
| 16,384 | sync | 271.40 | 273.45 | 0.00 |
| 16,384 | idle | 702.67 | 704.08 | 266.48 |
| 16,384 | overlap | 761.18 | 796.35 | 291.20 |

At 16,384 objects, the overlapping mode's 32,000 writes consume 12.13–13.26 ms
of owner CPU, about 1.7% of total process CPU. Marker CPU is the sum of measured
worker tracing passes, not all collector CPU. About 470 ms remains outside that
scope; it includes owner capture/completion and unscoped scheduling/runtime work.
It must not be attributed entirely to metadata without further phase measurement.

Every concurrent trial completes 1,000 valid epochs with 64,000 logged candidate
words. At 16,384 objects it copies 524,288,000 header bytes and reads 262,144,000
payload bytes; at 1,024 the counts are 32,768,000 and 16,384,000 respectively.
These are accumulated traffic counts, not retained memory or allocation bytes.
The prior indexed-store gain remains valid, but does not remove these lifecycle costs.

At 16,384 objects, observed finish p99 is 360–387 µs for synchronous collection
and 1,069–1,988 µs for the overlapping candidate. Its write-window p99 is 62–97 µs,
versus 7–17 µs with an idle marker. The first cohort had smaller tails and somewhat
lower candidate CPU (about 707 ms). Both cohorts are retained; scheduling noise is
material, and the comparison does not prove improved tail behavior. Percentiles of
separate phases must not be summed into a fictitious per-epoch percentile.

macOS whole-process maximum RSS in the scoped large-graph runs is about 2.70 MiB
for synchronous, 3.47–4.13 MiB for idle and 4.13–4.14 MiB for overlap. This includes
initial allocation, diagnostic arrays, worker stack and final reporting; it is not
peak managed live bytes or isolated collector memory. No memory-policy conclusion
is justified from these small process high-water measurements alone.

## Evidence and next action

- `store-overlap-results.json.gz`: scoped runs, raw phase samples, validated quantiles,
  build/sanitizer commands, counters and input hashes.
- `store-overlap-pilot.json.gz`: earlier cohort before reporting marker/work counters.
- `store-overlap-build-failure.json.gz`: initial compile failed because strict POSIX
  feature selection hides macOS RSS fields; enabling Darwin declarations fixes the
  macOS-specific fixture. No runtime or collector change was needed.
- Generated candidate source matches `store-index-native-gc.c.gz` byte-for-byte.

Run `python3 benchmarks/execution/concurrent-live/measure-overlap.py --output <fresh>`.
The synchronous collector remains the default. Next separate owner snapshot-capture
and completion CPU/work, then reduce duplicated metadata/tracing work. In parallel
with that design path, complete generated/native writer admission so these results
can be tested against managed standalone and plain/JSON HTTP workloads. This fixture
cannot replace that full objective or prove the candidate ready as the default.
