# Garbage collector benchmarks

Removing a duplicate root-reading pass improved macOS HTTP throughput by 29–35%
in matched JIT/AOT runs. The Windows comparison showed little change for plain
text and 9–10% higher JSON medians, with overlapping ranges. These development
measurements help identify where to profile next; they are not release guarantees.

These are application benchmarks of a collector-path change, rather than isolated
GC pause benchmarks. Each request exercises allocation, collection, HTTP processing
and scheduling. See [HTTP benchmarks](/benchmarks/http/) for the application baseline
and [garbage collection](/features/gc/) for the runtime model.

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

The Windows result warrants separating collector CPU time from socket and scheduler
wait time before choosing another optimization. Collection-pressure changes need
complete temporary-root coverage first: an earlier pacing experiment was rejected
because it could fault on an allocation accepted by the existing collector.
Generational or concurrent collection remains a design option, not an implemented
result.

Dated runs, including slower samples, remain in [benchmark history](/benchmarks/history/).
New workloads or measurement conditions form separate series; results are not
silently replaced when performance changes.
