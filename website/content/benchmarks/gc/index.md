# Garbage collector benchmarks

Runtime profiling separates collection work from socket processing and scheduler
waits. The latest matched Windows experiment improved HTTP throughput by 1.58–2.23×
by waiting for socket readiness instead of sleeping after an empty poll. Collection
time stayed largely unchanged, making it the next substantial measured cost.

These development application benchmarks measure collector and scheduling changes,
not isolated GC pauses or release guarantees. See [HTTP benchmarks](/benchmarks/http/)
for the application baseline and [garbage collection](/features/gc/) for the runtime
model. Earlier collector-path results remain below for comparison within their own
recorded conditions.

## Windows GC versus waits — 2026-10-13

Same guest objects and runtime sources, with a build switch selecting the old sleep
policy or the new bounded readiness wait. Windows Server 2022 x64 on one GitHub-hosted
AMD VM; five alternating rounds, with ranges shown alongside medians. This is a
separate series from the root-observation experiment below.

| Workload | Mode | Sleep requests/s (range) | Readiness requests/s (range) | Median ratio |
| --- | --- | ---: | ---: | ---: |
| Plain text | JIT | 40.10 (37.07–42.30) | 89.30 (82.63–93.46) | 2.23× |
| Plain text | AOT | 41.77 (37.47–47.44) | 87.19 (58.55–91.94) | 2.09× |
| Object JSON | JIT | 29.95 (26.94–33.21) | 47.36 (45.84–49.51) | 1.58× |
| Object JSON | AOT | 28.45 (25.12–33.21) | 46.73 (46.25–47.74) | 1.64× |

All 1,536 responses passed, including 960 timed requests. Observed ranges do not
overlap; the slower plain-text AOT candidate round remains included. The short,
sequential protocol measures neither saturation nor parity with .NET or Go.
[Immutable raw run](https://github.com/marinasundstrom/neoCLR/blob/4c6573147/benchmarks/http-scheduling/windows-waits-20261013.json).

Separate instrumented AOT runs explain the improvement. These are elapsed intervals
for the whole process, including warmup and shutdown, not per-request pauses or an
exclusive CPU breakdown.

| Measured interval | Plain text, sleep → readiness | JSON, sleep → readiness |
| --- | ---: | ---: |
| Scheduler waits | 424–443 → 24–27 ms | 386–489 → 51–63 ms |
| Collection | 191–196 → 195–198 ms | 359–361 → 355–356 ms |
| Socket polling | 1.95–2.13 → 1.66–1.77 ms | 2.11–2.20 → 1.79–1.82 ms |

All 256 diagnostic responses passed. Every process reclaimed all its tracked
allocations: 16,340 for plain text and 38,358 for JSON, with no GC or scheduler errors.
Socket/GC contracts and the HTTP/HTTPS execution checks also pass. The
[diagnostic record](https://github.com/marinasundstrom/neoCLR/blob/4c6573147/benchmarks/http-scheduling/README.md#matched-windows-results--2026-10-13)
links every raw profile and correctness report.

The Windows host now waits for socket readiness with a requested timeout of at most
one millisecond. DNS-only, timer-only and legacy TLS waits retain their fallback;
macOS waiting is unchanged. Callbacks still pass through the same scheduler, and
GC policy is unchanged. A follow-up needs to investigate collection pressure and
complete temporary-root coverage before reducing collections.

## Root observation — 2026-10-13

Previously the native host read every published root into a diagnostic accumulator
before the collector read it again. The candidate removes that duplicate pass while
preserving descriptor validation, roots and every existing collection boundary.
Only the root observer differs within each baseline/candidate pair. This change
does not reduce allocations or introduce a new collection policy.

Tables show median requests per second, with the minimum–maximum across five
alternating rounds in parentheses. Ranges are observations, not confidence intervals.

### macOS ARM64

Interactive workstation; fixed guest assemblies and native objects. These fixtures
are separate from the newer streaming-library follow-up below.

| Workload | Mode | Baseline requests/s (range) | Candidate requests/s (range) | Median change |
| --- | --- | ---: | ---: | ---: |
| Plain text | JIT | 64.42 (64.01–66.42) | 86.80 (79.13–88.02) | +34.7% |
| Plain text | AOT | 68.79 (61.59–69.48) | 88.80 (84.72–89.90) | +29.1% |
| Object JSON | JIT | 35.99 (32.89–36.19) | 48.58 (47.24–48.79) | +35.0% |
| Object JSON | AOT | 37.51 (36.54–38.01) | 49.27 (46.03–49.71) | +31.4% |

All 1,536 responses passed validation; 960 requests contributed to measured rounds.
[Immutable raw results](https://github.com/marinasundstrom/neoCLR/blob/858f6ca7f/benchmarks/http-server/root-observation-comparison-20261013.json).

### Windows x64

Windows Server 2022, GitHub-hosted AMD virtual machine; tested revision `88f522c1c`.
The same paired protocol uses freshly built Windows guests. Compare within this
table, not against the macOS host or its guest fixtures.

| Workload | Mode | Baseline requests/s (range) | Candidate requests/s (range) | Median change |
| --- | --- | ---: | ---: | ---: |
| Plain text | JIT | 25.81 (24.33–29.26) | 26.01 (24.39–27.36) | 0.8% |
| Plain text | AOT | 24.72 (23.63–26.48) | 24.80 (23.17–26.88) | 0.3% |
| Object JSON | JIT | 18.06 (16.41–19.52) | 19.88 (19.40–20.47) | 10.1% |
| Object JSON | AOT | 18.07 (17.29–18.72) | 19.68 (18.65–20.95) | 8.9% |

All 1,536 responses passed validation; 960 requests contributed to measured rounds.
Plain-text differences are within variability. JSON medians improved, but ranges
overlap: more evidence is needed to establish the size and consistency of the gain.
[Immutable raw results](https://github.com/marinasundstrom/neoCLR/blob/37c5cb698/benchmarks/http-server/windows-root-observation-comparison-20261013.json)
· [Windows execution](https://github.com/marinasundstrom/neoCLR/actions/runs/38405932978).
Windows ARM64 remains unmeasured.

### Newer streaming library — separate macOS AOT series

A follow-up uses newer managed HTTP fixtures. It changes only the root observer
within each pair and validates another 768 responses, including 480 timed requests.

| Workload | Baseline requests/s (range) | Candidate requests/s (range) | Median change |
| --- | ---: | ---: | ---: |
| Plain text | 79.30 (72.98–80.10) | 101.11 (86.99–102.76) | +27.5% |
| Object JSON | 40.93 (36.91–41.05) | 53.36 (48.79–53.67) | +30.4% |

This series qualifies AOT only; it makes no additional JIT claim.
[Immutable raw results](https://github.com/marinasundstrom/neoCLR/blob/5390a6f83/benchmarks/http-server/root-observation-streaming-comparison-20261013.json).

## How these runs are measured

Each worker receives 32 sequential HTTP/1.1 requests over fresh loopback connections,
with connection close after each response. The first eight requests are excluded
from timing. Each variant has a separate warmup process before five alternating
measured rounds. Client wall time includes receiving and validating the response;
build, process startup and JIT compilation are outside request timing. Profiling
is disabled during throughput measurements.

These short, concurrency-one runs do not measure saturation, sustained service
capacity or parity with .NET and Go. Raw reports retain commands, source/input hashes,
individual timings and startup measurements. Reproduction instructions and exact
fixture constraints are in the [experiment record](https://github.com/marinasundstrom/neoCLR/blob/37c5cb698/benchmarks/http-server/root-observation.md).

## Correctness and allocation diagnostics

Separate instrumented macOS runs report complete final reclamation and zero collector
failures. These counts cover entire processes, including setup and shutdown; they
are not allocations per request or total operating-system memory consumption.

| Guest fixture | Plain allocations / reclaimed | JSON allocations / reclaimed | Peak arena extent, plain / JSON |
| --- | ---: | ---: | ---: |
| Fixed fixtures | 17,908 / 17,908 | 40,214 / 40,214 | 13,584 / 19,640 bytes |
| Newer streaming fixtures | 16,340 / 16,340 | 38,358 / 38,358 | 13,216 / 19,672 bytes |

These are separate fixture series, not an isolated allocation-reduction comparison.
Root/lifetime tests also cover suspended work, abandoned operations and invalid
root descriptors. Windows passes collector, admission, profiling and root-rejection
checks. Passing these checks supports correctness for the tested scenarios; it does
not prove every lifetime path is correct. The experiment record links the raw
profiles and checks. GC pause distributions and Windows allocation-per-request
measurements are not yet presented as a comparable benchmark series.

## What the results guide

The matched wait experiment above separates a large Windows waiting cost from
collector work. Collection-pressure changes still need complete temporary-root
coverage first: an earlier pacing experiment was rejected
because it could fault on an allocation accepted by the existing collector.
Generational or concurrent collection remains a design option, not an implemented
result.

Dated runs, including slower samples, remain in [benchmark history](/benchmarks/history/).
New workloads or measurement conditions form separate series; results are not
silently replaced when performance changes.
