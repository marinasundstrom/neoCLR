# Native root observation — development comparison, 2026-10-13

GC builds previously loaded every published root into a volatile diagnostic
accumulator before the collector loaded it again to trace reachability. They now
validate the descriptors and publish the same snapshots, leaving value traversal
to the collector. Probe-only builds retain their diagnostic observations. Every
existing collection boundary and fault/descriptor check remains; this does not
promote the rejected pacing experiment or reduce collection frequency.

This follows the [existing CLR/Go/V8 and GC research](../http-scheduling/README.md#collector-research-direction--2026-10-13):
root enumeration belongs to the runtime collection machinery. This removes an
extra neoCLR diagnostic pass, not a .NET API divergence or a new GC algorithm.
The benefit is less repeated memory traffic and volatile accumulation. The cost
is a conditional path that must remain qualified in both GC and probe-only builds;
malformed descriptor rejection now has explicit coverage for both.

## Matched JIT and AOT measurements

[Raw report](root-observation-comparison-20261013.json). Baseline `9cd74b374`;
only `root-probe.c` changes between each matched pair. Guest assemblies and AOT
objects are the retained plain/JSON benchmark fixtures from the existing reports.
They are fixed fixtures, not a newly rebuilt SDK or qualification of the latest
managed HTTP library. Both hosts use the same remaining native sources.

Five alternating rounds per case/mode, one warmup process per variant, 32 validated
responses per process, first eight requests excluded from timing. This is client
wall time for a fresh loopback connection through response EOF, without routing.
Build/startup is excluded. No diagnostic instrumentation or concurrent builds
ran during these timings. Interactive macOS ARM64 host; no Windows claim.

| Workload | Mode | Baseline median requests/s (range) | Candidate median requests/s (range) | Gain |
| --- | --- | ---: | ---: | ---: |
| Plain text | JIT | 64.42 (64.01–66.42) | 86.80 (79.13–88.02) | 34.7% |
| Plain text | AOT | 68.79 (61.59–69.48) | 88.80 (84.72–89.90) | 29.1% |
| Object JSON | JIT | 35.99 (32.89–36.19) | 48.58 (47.24–48.79) | 35.0% |
| Object JSON | AOT | 37.51 (36.54–38.01) | 49.27 (46.03–49.71) | 31.4% |

All 1,536 responses passed; measured rounds contain 960 timed requests. Candidate
and baseline ranges do not overlap in these runs. These are local development
comparisons, not release numbers, statistical confidence intervals, or evidence
of parity with Go/.NET. Website publication remains on hold.

## Correctness and allocation evidence

[Sanitizer report](root-observation-validation-20261013.json) passes eight existing
collector/host-root/retained-body/idle-drain checks and ten expected-abort cases.
The rejection cases cover bad addresses, widths, flags, fault-frame counts and
transient descriptors in an ancestor belonging to another heap. Both probe-only
and GC builds reject them before collecting the current heap.

Separate instrumented runs validate 192 more HTTP responses and complete final
reclamation with zero collector failures:

- [Plain text](root-observation-plain-profile-20261013.json): 17,908 allocations and
  reclamations per process; peak arena extent 13,584 bytes.
- [JSON](root-observation-json-profile-20261013.json): 40,214 allocations and
  reclamations per process; peak arena extent 19,640 bytes.
- [Concurrent runtime suspension](root-observation-runtime-profile-20261013.json):
  four runs, including stalled peers, cancellation and shutdown checks, using the
  verified runtime-lowered fixture from the matched-lowerings experiment.

Plain/JSON boundary validation now costs approximately 51–54/86–90 ms per
instrumented process. These timings are diagnostic, not paired throughput data;
do not compare them numerically with earlier profiles using different guest builds.
The collector still dominates. Exact lifetime, root and owner correctness matters
before enabling pressure-based collection; helper temporary roots remain the next
pacing prerequisite. Generational/concurrent GC remains a separate design decision.

## Reproduce

```sh
python3 benchmarks/http-server/compare-gc.py \
  --plain benchmarks/http-server/results-20261013.json \
  --json benchmarks/http-server/json-results-20261013.json \
  --host target/release-callback-bundle-20261013/preparation.json \
  --baseline 9cd74b374 --source root-probe.c --output NEW_DIRECTORY
```

The preparation report and linked guest objects must still be available. Raw
reports retain commands and input hashes. Use `validate_conditional_roots.py
--owned-bodies` for focused native checks, and `benchmarks/http-scheduling/profile.py
--gc-phases` with matching native build reports for reclamation/phase diagnostics.

## Streaming-library follow-up

The [separate streaming-fixture comparison](root-observation-streaming-comparison-20261013.json)
uses the newer `http-concurrent-plain/json-20261013` native build records, including
the streaming HTTP library. These workloads are still sequential plain/JSON request
processing, not a concurrent throughput benchmark. The same five-round protocol
and pre-optimization baseline apply; only `root-probe.c` differs within each pair.

| Workload | Mode | Baseline median requests/s (range) | Candidate median requests/s (range) | Gain |
| --- | --- | ---: | ---: | ---: |
| Plain text | AOT | 79.30 (72.98–80.10) | 101.11 (86.99–102.76) | 27.5% |
| Object JSON | AOT | 40.93 (36.91–41.05) | 53.36 (48.79–53.67) | 30.4% |

All 768 responses passed, including 480 timed requests. This confirms the improvement
with the newer library. The retained full benchmark reports for these newer fixtures
are not passing JIT qualifications, so this follow-up deliberately consumes their
passing native build records with `--aot-only`. It makes no new JIT claim; the earlier
matched JIT result above remains valid for its own fixtures. No failed aggregate
report is relabeled as passing. MacOS ARM64 only; website publication remains on hold.

```sh
python3 benchmarks/http-server/compare-gc.py --aot-only \
  --plain target/http-concurrent-plain-20261013/native/build.json \
  --json target/http-concurrent-json-20261013/native/build.json \
  --baseline 9cd74b374 --source root-probe.c --output NEW_DIRECTORY
```

The controller now supports explicit AOT-only comparisons from native build reports,
requires the selected source in each link command, and records measured modes and
round count. JIT comparisons still require a passing host preparation report.

Separate instrumented [plain](root-observation-streaming-plain-profile-20261013.json)
and [JSON](root-observation-streaming-json-profile-20261013.json) runs validate another
128 responses. All 16,340 plain / 38,358 JSON allocations per process are reclaimed,
with zero collector failures. Peak arena extents are 13,216 / 19,672 bytes, matching
the prior stress-policy fixture evidence. The optimization reduces root observation
work rather than the number of allocations or collection boundaries.
