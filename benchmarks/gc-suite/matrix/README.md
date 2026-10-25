# Execution/collector benchmark matrix — 2026-10-25

The author requested fresh Interpreter/JIT/AOT measurements with both collectors
after persistent HTTP integration. This first completed cohort covers the unchanged
six standard standalone workloads in native AOT and JIT. HTTP close/persistent/
pipelined plain/JSON measurements, full interpreter results, and matrix pause
profiles are still pending. This is not completion of the requested matrix.

## Available combinations

| Execution | Collector | Evidence/status |
| --- | --- | --- |
| Interpreter | mark-sweep | Full steady-allocation pilot passes; full six-sample first pass started |
| Interpreter | concurrent native candidate | Unsupported: src/gc.rs exposes only MarkSweep; CLI rejects concurrent |
| AOT | synchronous native | 18 measured invocations pass |
| AOT | concurrent native | 18 measured invocations pass |
| JIT | synchronous native | 18 measured invocations pass |
| JIT | concurrent native | 18 measured invocations pass |

Interpreter mark-sweep and the synchronous native collector have distinct
implementations. Do not treat these as six interchangeable policy switches.
No collector defaults change. The retained optimized native compiler, source
assemblies and reference bundle match the standard suite; this is a new timing
cohort, not a rebuild of unchanged standalone sources. Source hashes are checked.

## Standalone native CPU

Median invocation CPU **milliseconds**, three rounds per cell. Compilation and
startup are excluded; collector worker CPU, output and cleanup are included.
All 72 checksums pass. JIT/AOT allocation counts and allocated bytes match for
every corresponding sample/collector, using the same linked host sources/policies.
Hardware and raw commands are archived. Both phases ran sequentially, without
competing benchmark or build processes started by this task. No affinity/isolation
or external machine-load exclusion is claimed.

| Sample | AOT sync | AOT concurrent | JIT sync | JIT concurrent |
| --- | ---: | ---: | ---: | ---: |
| compute | 64.74 | 64.69 | 64.87 | 64.53 |
| reuse-buffer | 18.04 | 111.19 | 19.20 | 111.78 |
| steady | 98.23 | 151.80 | 200.41 | 256.33 |
| bursts | 55.49 | 82.88 | 105.38 | 133.23 |
| retained-window | 222.62 | 327.09 | 390.20 | 525.50 |
| mixed-phases | 106.62 | 156.70 | 205.43 | 257.79 |

Policies: sync allocation/publication budgets 256/4096; concurrent 1024/16384,
50% completion watermark and index-only store lookup. The four qualified live
stack/byref optimizations are enabled consistently in JIT/AOT. Repetitions alternate
collector order. This retains independent policies rather than assuming identical
CPU/memory budgets.

## Findings and limits

- Compute is effectively equal across native modes/collectors. The reusable-buffer
  sample has only 14 managed allocations; concurrent CPU is about six times sync
  in both execution modes. Remaining store/runtime admission work is a stronger
  lead here than collection throughput.
- Allocation-heavy JIT workloads are substantially slower than AOT for both
  collectors despite identical allocation work. Steady and burst collection counts
  also match between modes. This identifies a mode-specific cost to profile; it
  does not prove whether generated calls, dynamic-host/TLS access, heap lookup,
  root publication or another service is responsible.
- Concurrent remains slower than synchronous for the allocation-heavy cases.
  This cohort establishes neither a concurrent-over-sync advantage nor default
  readiness. Retain the previous same-collector history rather than replacing it.
- JIT metadata loading, selection/verification and compilation/finalization times
  are retained separately in each trial. Native host PROFILE measures invocation;
  whole CLI duration is not used as the JIT throughput denominator.
- No pause-distribution claim is made. Arena/live peaks and collection counts are
  in the archives; timed runs do not enable detailed phase/pause instrumentation.

## Interpreter qualification and timing scope

The unchanged steady sample performs 1,048,576 managed Reading allocations and
passes its exact checksum. It takes 53.60 seconds wall / 53.50 seconds whole-process
CPU; GC reports 1,048,585 allocated/reclaimed objects, peak 64 and 19,419 collections.
These metrics include metadata loading/startup and are **not** interchangeable
with native invocation-only timings. The full fixed-work six-sample first pass is
being measured without reducing loop counts. Timeout/instruction-budget failures
must remain failed measurements, never successful throughput rows. Three-round
interpreter statistics remain pending.

## Reproduction

```sh
python3 benchmarks/gc-suite/run.py benchmark --suite standalone \
  --build target/gc-patterns-standard-build --index-only --rounds 3 \
  --output target/gc-matrix-aot-new
python3 benchmarks/gc-suite/matrix/native-jit.py \
  --aot target/gc-matrix-aot-new --build target/gc-patterns-standard-build \
  --rounds 3 --output target/gc-matrix-jit-new
python3 benchmarks/gc-suite/matrix/interpreter.py \
  --build target/gc-patterns-standard-build --rounds 1 --timeout 900 \
  --output target/gc-matrix-interpreter-new
```

Run measurement phases sequentially. `native-jit.py` links matching shared hosts
before measuring and retains JIT compilation stages separately. The interpreter
harness proves selector rejection for concurrent and records full commands/output.
Existing standard-suite retained-toolchain prerequisites apply; these are developer
experiments, not a clean-checkout benchmark bootstrap or portable installed CLI API.

Archives: `aot-run.json.gz`, `aot-results.json.gz`, `jit-results.json.gz`, and
`interpreter-pilot.json.gz`. Native data is complete; the pilot is one interpreter
sample/run, not a complete matrix cell set.
