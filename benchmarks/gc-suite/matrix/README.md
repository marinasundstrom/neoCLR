# Execution/collector benchmark matrix — 2026-10-25

The author requested fresh Interpreter/JIT/AOT measurements with both collectors
after persistent HTTP integration. This first completed cohort covers the unchanged
six standard standalone workloads in native AOT and JIT. HTTP close/persistent/pipelined plain/JSON measurements now also pass; remaining
standalone interpreter results and matrix pause profiles are still pending. This is not completion of the requested matrix.

## Available combinations

| Execution | Collector | Evidence/status |
| --- | --- | --- |
| Interpreter | mark-sweep | Four full samples pass; retained-window times out at 900 seconds |
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
with native invocation-only timings. The full fixed-work first pass did not reduce loop counts: compute, reuse-buffer,
steady and bursts pass; retained-window hits the 900-second timeout and the
fail-fast runner does not reach mixed-phases. Timeout/instruction-budget failures
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

### Full interpreter first pass

Single invocation per sample, unchanged work counts. CPU here is whole-process
seconds, including startup, unlike the native invocation-only table above.

| Sample | Wall seconds | Process CPU seconds | Outcome |
| --- | ---: | ---: | --- |
| compute | 637.43 | 633.90 | Checksum passes |
| reuse-buffer | 95.56 | 95.11 | Checksum passes |
| steady | 52.51 | 52.24 | Checksum passes |
| bursts | 92.80 | 92.57 | Checksum passes |
| retained-window | 900.02 | 896.90 | Timeout, no completed checksum |
| mixed-phases | — | — | Not reached |

`interpreter-full-first-pass.json.gz` preserves the failed cohort and successful
preceding trials. Its archive note distinguishes input hashes captured after the
timeout from the original runner's incomplete result. No builds or competing
benchmarks were started during this cohort. The retained-object cost needs a
separate profile; these timings alone do not identify its cause.

## Private persistent HTTP matrix procedure

`prepare-http.py` builds one exact-source System.Web fixture with the canonical
three-field `Person` JSON model retained for native reflection. It prepares AOT
executables and JIT hosts for both native collectors before timing. The interpreter
uses the same assembly and its mark-sweep implementation. Public server APIs and
collector defaults are unchanged.

```sh
python3 benchmarks/gc-suite/matrix/prepare-http.py \
  --output target/gc-matrix-http-build-new
python3 benchmarks/gc-suite/matrix/http.py \
  --build target/gc-matrix-http-build-new --output target/gc-matrix-http-smoke-new \
  --requests 16 --interpreter-requests 16 --warmup 4 --concurrency 2 \
  --pipeline-depth 4 --rounds 1 --smoke
```

Run qualification first, then invoke `http.py` with fresh output and without
`--smoke` for measurements. Defaults are three rounds, 4,096 native/64 interpreter
requests per invocation, 16 warmup requests, four clients and pipeline depth 16.
Those different counts must remain explicit in reports. Use `--modes`, `--samples`
and `--protocols` to select a bounded cohort. Run cohorts sequentially.

Close, sequential keep-alive and pipelined traffic all exercise the new private
persistent driver. The close column is not the historical public Serve driver.
Every response validates status, content type, body, length, connection policy and
expected EOF. Shutdown must report all warmup/measured requests and no failures;
live collector accounting and JIT exit diagnostics must pass. No forced collection
occurs in this fixture.

Request throughput times only measured client traffic. Native CPU/allocations cover
the entire invocation (warmup, readiness and cleanup included); interpreter CPU
includes process startup. JIT compilation is recorded separately. Client CPU is
also retained. Pipeline latencies are observed from each batch start, not independent
request latency or GC pause measurements. These Python loopback clients do not
establish saturation, TechEmpower compatibility or an external framework ranking.

Qualification evidence: `http-build.json.gz` and `http-smoke.json.gz` pass all
30 cells (600 responses including warmup), with 95 completed concurrent epochs
and no invalid-state reports. These small runs are not throughput evidence.
`http-development-failures.json` preserves two preparation failures: the first
incorrectly required explicit sync budget macros where the retained recipe used
identical defaults; the second omitted the JSON model's reflection roots. The final
build supplies explicit sync budgets and standard Person reflection retention;
no compiler admission guard was relaxed. Three-round measurements now pass all 90 trials and 297,504 responses; see
[HTTP results](http-results.md). Pause and worker-scheduling profiles remain pending.


## Stack profiling the current native matrix

On macOS, add `--sample-stacks --requests 32768 --rounds 1` and select native
modes. The harness starts a one-second `sample` capture after warmup, records
its command and report hash, validates the capture, and cleans up the sampler
on failure. Plain and JSON, sync/live, JIT/AOT and connection policy remain
separate report dimensions. Build inputs must match before execution.

These reports are diagnostic, never benchmark evidence. Stack observations
include sleeping threads and are not CPU percentages; omitted frames are not
zero. `wholeProcessCpuSeconds` includes sampler child CPU in this mode, while
`invocationCpuSeconds` remains server-reported (with sampling interference).
Use `benchmarks/execution/concurrent-workloads/summarize-cpu.py --measurement
PATH/results.json --output PATH/summary.json` to verify capture hashes and extract
top frames and TLS callers. Keep unprofiled throughput runs separate.


For the opt-in small-frame validation experiment, prepare a fresh standard HTTP
build, then run `prepare-root-variant.py --build PATH --output NEW_PATH`. It retains
original recipes and adds `split-sync-aot`, `split-live-aot`, `split-sync-jit` and
`split-live-jit`, using identical managed objects. Run qualification and timings
separately through `http.py`. The default root validator and collector policy stay
unchanged. See [safety and measurements](../../../docs/validation/split-storage-validation-20261025/README.md).

For matched standalone JIT root-validation measurements, use
`compare-scalar-jit.py --root-validation --build <standard-samples>
--aot <completed-root-validation-cohort> --runner <qualified-runner>
--output <fresh-directory>`. This reuses the timing/output checks while keeping
typed stores and qualified stack policy identical in both modes. It validates
current AOT input hashes and rejects differences in host/object recipes beyond
the root-validation flag and output path. Compilation and invocation costs remain
separate, and the first round is excluded as warmup.


For experimental JIT placement, `prepare-jit-placement.py --build <qualified-http>
--runner <current-test-runner> --output <fresh-directory>` pairs the same native
host with system and 64 MiB arena placement. Run `http.py` with `sync-jit`,
`arena-sync-jit`, `live-jit`, and `arena-live-jit`. The harness clears the ambient
placement switch; only arena recipes set it. For standalone work use
`compare-scalar-jit.py --arena-placement` with the completed root-validation AOT
cohort: both modes share its unchanged control host and typed-store compiler policy.
The arena is a test-only JIT code/static-data reservation, not managed-heap sizing
or a production default. See [qualification and current results](../../../docs/validation/jit-arena-current-20261025/README.md).

Use `prepare-jit-placement.py --policy fallback` to select the separate bounded
reservation/system-fallback experiment. Its modes are `fallback-sync-jit` and
`fallback-live-jit`; `sync-jit`/`live-jit` remain the matched system controls. The
manifest records policy identity and the fallback implementation hash. Fixed-arena
performance reports do not automatically qualify fallback performance or memory.


Use `http.py --process-memory` on macOS for separate whole-process RSS and virtual
size snapshots at readiness, after warmup and after traffic. These include the
compiler/test runner, metadata and libraries; they are neither allocator attribution
nor peak-memory measurements. The report is diagnostic-only, and child CPU includes
`ps`. Run this separately from throughput cohorts. Validate the helper with
`python3 -m unittest discover -s benchmarks/gc-suite/matrix -p test_process_memory.py`.

For compilation lifetime, use `prepare-jit-lifetime.py --build <qualified-http>
--runner <current-test-runner> --output <fresh-directory>`. Run its
`retained-sync-jit`, `released-sync-jit`, `retained-live-jit`, and `released-live-jit`
recipes with `http.py`. Each uses identical fallback placement; only retained
recipes set the test-only compilation-retention switch. The harness checks the
reported release policy. Use `compare-scalar-jit.py --compilation-lifetime` for the
six standalone patterns with the existing root-validation AOT control host.
Keep memory diagnostics separate from timing. Compilation, early release and guest
invocation have separate timings; release does not eliminate cleanup cost.
See [qualification and limitations](../../../docs/validation/jit-compilation-lifetime-20261025/README.md).

Current macOS JIT memory profiles also require test-runner malloc-zone snapshots
before preparation, before/after compilation-state release, and after host cleanup.
`sizeInUse` counts live malloc storage; `sizeAllocated` counts allocator reservation.
Neither is RSS or direct-mapping attribution. The harness sets the test-only native
memory switch only for memory profiles and clears it for ordinary runs. Rebuild
older runners before requesting JIT `--process-memory`; missing stages are errors.
See [native accounting](../../../docs/validation/native-malloc-accounting-20261025/README.md).

For stack-bound TLS layout, use `prepare-stack-bounds.py --build <qualified-http>
--baseline-source <preserved-native-stack.c> --output <paired-build>`. The preserved
source must match the base manifest; all other inputs are checked unchanged.
Run split/grouped sync/live AOT recipes with `http.py`. Add current JIT recipes via
`prepare-stack-jit.py --build <paired-build> --runner <pinned-runner> --output
<jit-build>`; both layouts use fallback placement. For six standalone patterns use
`compare-root-validation.py --stack-bounds`, keeping root validation unchanged.
See [qualification and mixed results](../../../docs/validation/grouped-stack-bounds-20261025/README.md).

To test composition, add `--combine-validation` to `prepare-stack-bounds.py`; the
variant becomes `combined-{sync,live}-aot` and also enables the split small-frame
storage validator. JIT preparation follows the manifest's layout names. Standalone
composition uses `compare-root-validation.py --stack-bounds --combine-validation`.
For a separate pause diagnostic, `prepare-paired-owner-profile.py --build <paired>
--output <profile-build>` instruments every AOT recipe without changing its options.
Run `http.py` on that build, then validate `OWNER_PAUSE` rows with
`benchmarks/execution/concurrent-workloads/summarize-owner-pauses.py`.
See [combined qualification](../../../docs/validation/combined-runtime-layout-20261025/README.md).
