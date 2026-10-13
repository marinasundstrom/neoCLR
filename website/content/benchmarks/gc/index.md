# Garbage collector benchmarks

GC work targets general runtime workloads as well as HTTP. Recent changes remove
repeated heap walks during allocation and host-handle admission while preserving
complete validation. Earlier series measure object graphs, roots and scheduler waits.

These are development measurements, not release guarantees. See
[HTTP benchmarks](/benchmarks/http/) for application results and
[garbage collection](/features/gc/) for the runtime model. Each historical series
retains its own conditions and raw evidence.

## Host-handle admission — 2026-10-13

Creating a native host root formerly validated the heap, then searched it again
for the referenced object and owner. One walk now does both. It still validates
every trailing header before registering the root, even when the objects are found
early. This helps native lifetime integration without changing collection policy.

Local macOS ARM64, five alternating pairs after warmup; baseline/candidate elapsed
time ratios. Each timed operation creates, reads and releases one root. Heap
construction and final reclamation are excluded.

| Case | 16 objects | 256 objects | 4,096 objects |
| --- | ---: | ---: | ---: |
| Strong root, first object | 1.00× | 1.03× | 1.00× |
| Strong root, last object | 1.13× | 1.77× | 1.76× |
| Owner and value first | 1.03× | 1.00× | 1.00× |
| Owner last, value first | 1.06× | 1.90× | 1.87× |
| Owner last, value middle | 1.21× | 2.22× | 2.22× |
| Owner and value last | 1.40× | 2.59× | 2.57× |

Early-object controls remain near baseline; later objects benefit from fewer heap
walks. A separate AOT HTTP comparison passes 768 responses with plain/JSON medians
within 0.3% of baseline. **This is not an HTTP throughput improvement.**
[Protocol, raw rounds and correctness checks](https://github.com/marinasundstrom/neoCLR/blob/ca995dc2e/benchmarks/gc-handles/README.md).

Hosted macOS/Windows GC, socket and native-package checks pass. At 256/4,096
objects, later-entry gains are 1.56–2.60× on hosted macOS and 1.43–2.26× on Windows.
Windows early-entry controls are 0.974–1.004×; hosted macOS controls are
0.882–1.110×, including slower medians with overlapping ranges. All rounds remain
in the linked record. These are within-host comparisons, not a platform ranking
or a universal non-regression guarantee. Windows ARM64 remains unmeasured.

## Allocation throughput — 2026-10-13

Allocation formerly walked heap headers once for validation, then again to find
free space. It now selects the first suitable hole during the validation walk.
Every header is still checked before the heap changes, including headers after a
usable hole. Failure outputs, corruption detection and collection policy remain.

The table shows baseline/candidate elapsed-time ratios from five alternating
pairs after warmup. Construction of fragmented heaps and final collection are
outside timing. Each column is a separate paired run, not a platform ranking.

| Workload | Allocations per batch | Local macOS ARM64 | Hosted macOS ARM64 | Windows x64 |
| --- | ---: | ---: | ---: | ---: |
| Objects | 64 | 1.66× | 1.37× | 1.37× |
| Objects | 1,024 | 1.88× | 1.32× | 1.70× |
| Objects | 4,096 | 1.89× | 1.60× | 1.82× |
| String-sized blocks | 1,024 | 1.87× | 1.34× | 1.84× |
| Byte buffers | 256 | 1.52× | 1.58× | 1.39× |
| Fragmented refill | 1,024 | 1.49× | 1.18× | 1.32× |

String-sized blocks are 128-byte atomic allocations; this does not measure encoding
or string API throughput. Object payloads are 16 bytes and buffers 8,192 bytes.
All workers check retained contents and final reclamation. Full validation is still
linear per allocation, so this does not establish constant-time allocation.
[Protocol and all raw rounds](https://github.com/marinasundstrom/neoCLR/blob/524ca0b22e6ca772010ae791da7099399b326329/benchmarks/gc-allocation/README.md).

<details>
<summary>Experimental follow-up: collection pacing</summary>

A separate closed native workload protects every allocation with rooted admission.
Its local object/string runs are 3.98×/3.57× faster with less frequent collection,
while peak arena extent rises about 23%. The pressure counterexample that rejected
the previous reserve-based policy now passes, alongside cleanup and suspended-owner
checks. This is **not enabled for general managed or HTTP execution**. Remaining
helper-root coverage and real scheduler idle integration are required first.
[Experimental evidence and limitations](https://github.com/marinasundstrom/neoCLR/blob/524ca0b22e6ca772010ae791da7099399b326329/benchmarks/http-gc-pacing/README.md#rooted-closed-workload-pacing-follow-up-2026-10-13).

</details>

## Dense object graphs — 2026-10-13

The native collector now uses a denser address index above 2,048 heap spans.
This reduces repeated block-header searches while keeping exact payload checks,
root ownership and collection frequency unchanged. The extra index costs 32 KiB
of temporary native stack and one additional validated heap walk on the dense path.

Initial local candidate: macOS ARM64, Xcode Clang `-O2`; five alternating pairs after warmup.
Values below are milliseconds per full collection: median (observed minimum–maximum).
Graph construction and final reclamation are outside the timed interval.

| Shape | Live objects | Baseline ms | Adaptive ms | Ratio |
| --- | ---: | ---: | ---: | ---: |
| Chain | 64 | 0.0008 (0.0008–0.0015) | 0.0008 (0.0008–0.0012) | 1.01× |
| Chain | 1,024 | 0.0155 (0.0154–0.0163) | 0.0155 (0.0155–0.0155) | 1.00× |
| Chain | 2,048 | 0.0397 (0.0397–0.0397) | 0.0396 (0.0396–0.0398) | 1.00× |
| Chain | 2,049 | 0.0398 (0.0398–0.0433) | 0.0295 (0.0294–0.0320) | 1.35× |
| Chain | 4,096 | 0.1178 (0.1177–0.1185) | 0.0622 (0.0621–0.0656) | 1.89× |
| Chain | 16,384 | 1.3975 (1.3513–1.3992) | 0.3137 (0.3036–0.3324) | 4.46× |
| Wide | 16,384 | 1.2973 (1.2179–2.0204) | 0.2475 (0.2323–0.4342) | 5.24× |
| Holes | 4,096 | 0.2199 (0.2197–0.2200) | 0.1017 (0.1011–0.1025) | 2.16× |
| Atomic | 64 | 0.0006 (0.0006–0.0006) | 0.0007 (0.0007–0.0007) | 0.98× |

The 16,384-object chain and wide graph collect **4.46× and 5.24× faster**.
Separate diagnostic runs reduce chain lookup-header visits from 70.7 million to
5.2 million over 100 collections. Small/sparse controls remain near baseline;
their slight changes are not evidence of a gain. All workers verify their live
graphs and complete final reclamation.

This is a collection-only test of the shared JIT/AOT native collector. It does not
measure allocation throughput, interpreter GC, concurrent load or parity with
.NET/Go. A separate local HTTP regression comparison passes all 1,536 responses:
plain/JSON JIT/AOT median throughput stays within 2% of baseline, with overlapping
ranges. The graph improvement does not translate into a material HTTP gain here.

[Raw graph rounds](https://github.com/marinasundstrom/neoCLR/blob/00e34b5c5/benchmarks/gc-index/macos-20261013.json)
and [methodology and costs](https://github.com/marinasundstrom/neoCLR/blob/00e34b5c5/benchmarks/gc-index/README.md).
The final bookkeeping refinement also passes hosted macOS/Windows GC and socket
contracts. Its large-chain/wide speedups are **4.48×/4.90× on macOS ARM64** and
**4.63×/5.19× on Windows x64**. These are within-machine comparisons, not platform
rankings. Final Windows small-control costs are under 2%; hosted macOS short
controls include up to a 12.5% slowdown. Longer local controls cost roughly
0.3–1.5%. The benchmark record retains all rounds: the dense-graph gain does not
promise improvement for every workload.
[Final evidence and limitations](https://github.com/marinasundstrom/neoCLR/blob/81da188cc/benchmarks/gc-index/README.md#qualification-and-retained-controls).

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

## Development: dense collection refinement

A 2026-10-14 macOS ARM64 comparison reuses validated heap spans when building the
larger native GC lookup table. Seven alternating pairs with longer timing intervals
show 2.1–3.3% higher collection throughput on dense object graphs; small/sparse
controls are within 1%. Windows paired results improve 3.6–6.9% on dense cases. Both platforms pass
correctness checks; noisy hosted macOS results are retained separately.
Full validation and collection policy remain unchanged.
This is a collection-only result, not an HTTP speedup. The accompanying HTTP
regression check validates 768 responses with overlapping timing ranges.
[Methodology, correctness checks and raw results](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/gc-index#reuse-validated-spans-for-the-dense-index--2026-10-14)
are retained separately from published release benchmarks.

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
