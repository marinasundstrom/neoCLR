# Standard GC benchmarking and profiling procedure

Version 1, 2026-10-25. Concurrent GC is intended to become the general-purpose
default and improve overall application performance. Synchronous GC remains the
current default and a useful option for resource-constrained/single-thread hosts.
Neither intent implies that concurrent wins every workload. We retain **both
collectors' own histories** as well as cross-collector comparisons.

## Fixed sample set

`samples.json` defines the six non-HTTP cases, operation counts and checksums.
Every case is an ordinary Raven source, with no explicit collection requests.
The sources deliberately vary allocation cadence and survival, not only totals.

| Sample | Work and allocation shape |
| --- | --- |
| HTTP plain | Fixed plain-text response through the existing HTTP primitives |
| HTTP JSON | Fixed typed JSON response through the existing HTTP primitives |
| compute | 67,108,864 scalar items; no allocations in the work loop |
| reuse-buffer | 4,194,304 updates to one fixed 16-element array |
| steady | 1,048,576 newly allocated readings, consumed immediately |
| bursts | 4,194,304 items; allocate during the first 128 of every 1,024 items |
| retained-window | 1,048,576 replacements in a rolling window of 1,024 readings |
| mixed-phases | 2,097,152 items; alternate eight allocation-heavy and eight compute-only phases |

Each standalone program checks execution through its computed output; the harness
requires the exact checksum and exit status. Initial pilot loops were too short
for useful timing; the standard lengths above replace those pilots. Different
rows do different work: compare collectors or revisions **within a sample**.
Phase schedules describe work progress, not a wall-clock arrival-rate generator.

## Environment and policies

This is a developer harness for the qualified macOS ARM64 native candidate, not an
installed CLI or a portable clean-checkout benchmark package. It uses existing
compiler/binding recipes and qualified build artifacts under `target/`, including
`gc-mode-comparison-final`, `primitive-comparison-build`, `native-host-ownership`,
and the HTTP selections. Provision those with the existing
[standalone setup](../execution/concurrent-workloads/README.md) and
[HTTP experiment setup](../execution/concurrent-store/README.md) before running.
The first retained environment uses the optimized native test runner
`target/release/deps/neoclr_codegen-12f33f26d2ff4df0`; pass `--runner` for another
qualified build. Missing recipes are a setup failure, not an omitted sample.

Use the matching Xcode developer directory and SDK. Current measurement environment:
Apple M1, 8 logical CPUs, 16 GiB RAM, macOS ARM64. The runner records hardware and OS;
raw child reports record objects, source hashes, compiler commands and policies.
Keep machine load stable, avoid competing builds during timed runs, and retain
failures. No process affinity/core isolation or cross-framework ranking is implied.

Standard policies are intentionally different:

- **Sync:** 256 allocations / 4096 publications between pacing boundaries.
- **Concurrent:** 1024 / 16384, with completion requested at 50% mutation-log occupancy.
- HTTP uses a 1 MiB arena and concurrent 4096-word log. Standalone retains its
  qualified host capacity, recorded in compilation commands. Worker-count studies
  are separate from these one-owner fixed-work runs.

These are experimental comparison profiles, not production defaults. Tune a
collector with the lower-level drivers and record its changed policy; do not force
identical resource use merely to make the comparison look symmetric.

## Benchmark mode

From the repository root, in the qualified toolchain environment:

```sh
python3 benchmarks/gc-suite/run.py benchmark --output target/gc-runs/baseline
```

The runner builds the six allocation samples with matching sync/concurrent
selection, retaining qualified immutable-byref and stack-store optimizations.
It runs both HTTP samples and all six standalone cases, five alternating rounds.
HTTP performs 4096 measured requests plus 16 warmups, four clients, checking status,
body and EOF. Standalone CPU covers invocation and cleanup, excluding startup and
compilation; HTTP CPU covers the server process, including worker/startup/cleanup.
Do not directly compare these different CPU scopes.

A qualified build can be reused:

```sh
python3 benchmarks/gc-suite/run.py benchmark \
  --build target/gc-runs/baseline/build --output target/gc-runs/next \
  --baseline target/gc-runs/baseline/run.json
```

Build reuse rejects changed sample sources. Rebuild when runtime/compiler changes
require new objects. `--suite http` or `--suite standalone` supports focused work;
full-suite acceptance should retain all eight cases. `--rounds` and `--requests`
are recorded, not silently changed. Benchmark reports include CPU, throughput,
managed allocation count/bytes and peak arena occupancy. These are distinct from
native allocation volume, RSS and live-object residency.

## Profile mode

```sh
python3 benchmarks/gc-suite/run.py profile \
  --build target/gc-runs/baseline/build --output target/gc-runs/profile \
  --rounds 3 --requests 1024
```

The same sources/policies run with diagnostic instrumentation. HTTP records owner
operation pause histograms. Standalone additionally records begin/capture and
joined-marker work, finish phases and disjoint final-collection components. The
runner validates counts and nesting through the existing summary tools; it never
adds nested capture to begin or seed to whole collection as independent costs.
Histograms provide bucket intervals, not exact p99 values. These are owner-operation
pauses, not HTTP end-to-end latency or whole-process CPU samples.

Profile timings **must not become benchmark speedups**. The comparison tool rejects
diagnostic runs. Use the existing symbolized runtime/CPU profiling tools when the
phase evidence points outside GC; low allocation does not imply low runtime overhead.
Per-phase allocation-rate time series are not yet collected: these samples provide
known cadence plus aggregate counts, not a continuous allocation timeline.

## Compare and retain

`run.json` is the normalized index; all raw child reports remain in its directory.
`compare.py old/run.json new/run.json --output comparison.json` compares **sync with
sync and concurrent with concurrent**, reports policy changes, and rejects different
sample definitions, hardware/OS, workload sizes, collector sets or instrumentation.
Historical runs are sequential and remain sensitive to OS load; ratios are evidence,
not automatic statistical regression verdicts. Do not compare across changed sample
lengths or against old release benchmarks as though only GC changed.

Always retain failures and exclusions; the standard suite fails if any case is
missing. Preserve output correctness, balanced epochs, no active marker at exit,
index cleanup and allocation equality where required. A disabled-GC control is not
part of version 1: it would require separate bounded-memory/lifetime semantics.

The [first standard results](results-20261025/README.md) are the baseline for future
same-collector comparisons. They include a separate profiler run. Earlier GC studies
remain separate series. Refresh website tables/chart only from a successful full
benchmark run:

```sh
python3 benchmarks/gc-suite/site-results.py target/gc-runs/next/run.json --date YYYY-MM-DD
python3 -m unittest discover -s scripts -p test_build_website.py
python3 scripts/build-website.py
```

Website publication is the repository's separate manual workflow. A local build is
not a deployment. Default promotion additionally needs sustained workload coverage,
resource/latency evidence, supported execution modes and ownership qualification;
a single fast sample is not that decision.

### Store-lookup policy experiment

Pass `--index-only` to either `benchmark` or `profile` to use the existing
allocation index directly for concurrent managed stores, without recent-range
cache bookkeeping. Synchronous GC remains unchanged. The normalized report records
this policy, and history comparisons flag the policy change. This option does not
change compiler emission, ownership rules, barriers or the runtime default.
See [qualification evidence](../execution/concurrent-workloads/index-only-results/README.md).

[TechEmpower comparison boundary and protocol gaps](techempower-comparison.md) records the author-supplied Round 23 reference and the prerequisites for a meaningful external comparison.

## Execution-mode expansion

The author-requested [execution/collector matrix](matrix/README.md) records a fresh
three-round AOT/JIT standalone cohort, interpreter capability and pilot evidence,
and the still-pending interpreter/HTTP/profile cells. Keep its timing scopes distinct
from this original AOT procedure; unsupported combinations are explicit.

## Concurrency and scheduling extension

The author requested dedicated worker/thread contention profiles and an architectural
review of concurrent GC. Follow the [concurrency procedure](concurrency/README.md),
which separates independent owners, shared heaps, tasks/fibers and collector threads.
The single-owner sample matrix does not establish many-worker scaling.

## Workload review — 2026-10-26

The [coverage and TechEmpower review](workload-review-20261026.md) distinguishes
mechanism tests from production-shaped pressure, records fresh production-default
lazy-JIT reruns, and identifies real database/pooling, mixed cold routes, resource
quotas and longer recovery runs as remaining gaps. Keep historical release scores
separate from changed workloads and codegen configurations.
