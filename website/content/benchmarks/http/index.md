# HTTP request-processing benchmarks

Plain-text and typed-JSON Raven servers exercise the request path in interpreter,
JIT and AOT. These **18 October 2026, Apple M1/macOS 27.0** measurements use the
Preview 16 package with Raven `c0854cdda`. All **1,152 responses across
36 processes** pass validation, with zero errors. Windows results follow below as a separate hosted-runner series.

<a id="plain-text-baseline"></a>
<a id="publication-status"></a>

## Workloads and results

Plain text returns ten UTF-8 bytes, `Café 🌍`. Typed JSON creates a `Person` and
serializes `id: 42`, `name: "Café team"` and `active: true` for every request,
including object construction and reflection. Every response checks framing,
status and content before it contributes to the result.

| Workload | Mode | Median requests/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| http-plain | interpreter | 2.16 | 2.11–2.19 | 2.901 |
| http-plain | jit | 142.79 | 137.54–150.22 | 5.364 |
| http-plain | aot | 147.21 | 143.48–155.82 | 0.006 |
| http-json | interpreter | 1.17 | 1.16–1.18 | 2.782 |
| http-json | jit | 79.87 | 79.55–81.48 | 8.432 |
| http-json | aot | 83.18 | 78.17–83.39 | 0.005 |

## Windows x64

The same package passes on Windows Server 2025 (AMD Family 26 Model 2), with
55 CPU/JSON processes and 1,152 HTTP responses validated across the full run.
Shared hosted-runner load is uncontrolled; these numbers are a separate series,
not a comparison with the macOS machine.

| Workload | Mode | Median requests/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| http-plain | interpreter | 2.08 | 1.90–2.21 | 3.389 |
| http-plain | jit | 62.35 | 45.59–69.80 | 6.353 |
| http-plain | aot | 57.90 | 51.11–64.95 | 0.007 |
| http-json | interpreter | 1.21 | 1.16–1.24 | 3.434 |
| http-json | jit | 44.89 | 42.93–54.00 | 8.808 |
| http-json | aot | 45.67 | 36.61–50.93 | 0.007 |

[Windows raw reports and package identity](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/release-20261018/windows)

## Measurement boundaries

Each mode has one whole-process warmup and five measured fresh processes, with
rotating mode order. Each process validates 32 responses; the first eight are
excluded from timing. Startup measures process launch to listening-port readiness,
including metadata loading and JIT preparation. Builds complete before measurement.

The Python client shares the machine with the server. Requests are sequential,
use HTTP/1.1 loopback and close each connection. Host indexing was active. These
short, fixed-count workstation runs do not measure concurrent server capacity or
establish a controlled regression threshold. Request latency samples, host-load
observations and artifact/input hashes are retained in the raw reports.

The optional PostgreSQL workload has not been rerun for this release. Its
[historical measurements](/benchmarks/history/) do not qualify the current package.
HTTPS, keep-alive, RSS and managed allocation rate are outside these measurements.

[Raw plain/JSON samples and package identity](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/release-20261018/macos) ·
[Reproduction instructions](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server) ·
[Garbage-collection measurements](/benchmarks/gc/)

[General execution](/benchmarks/execution/) · [All benchmark reports](/benchmarks/)


## Development load investigation

After Preview 16, the [offered-load investigation](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/load)
separates offered arrivals, successful completions, client admission misses and
errors, with latency and CPU/RSS samples. It also compares worker-count and GC-policy
costs. These local development experiments do not replace the release measurements
above or establish production capacity; paced collection remains experimental.

The [callback-path follow-up](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/lazy-accept)
compares `ServeOne` handlers that await queued work, measuring deferred queued-accept
storage separately from the earlier explicit-accept fixture. Public API semantics
and collection defaults are unchanged.

Development `Serve(handler)` now owns bounded concurrent dispatch and handler
completion. Its [matched load comparison](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/serve)
uses the same asynchronous handler and shutdown control as application-managed
`ServeOne` loops, retaining overload misses and latency alongside throughput.

## Development Serve sustained-load follow-up

After reducing concurrent desktop activity, the connection-close Serve fixture
passes two 120-second trials at 400 offered requests/sec with experimental paced
GC: all 96,000 responses validate, with p99 latency 115.8/98.6 ms. A 500/sec check
saturates one CPU core and loses 2,606 of 15,000 arrivals at the client admission
limit. These are local tested points, not default-GC or production capacity claims.
[Profiles, noisy counterexamples and full measurement contract](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/serve/capacity-20261018).

A [matched Go net/http comparison](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/serve/go)
now alternates 400/500 offered requests/sec under a single Go execution processor
and the same response, connection and client budgets. At 400/sec, neoCLR paced GC
uses 8.6–8.9× the sampled server CPU per completion; Go passes 500/sec while neoCLR
records admission misses. Go also passes two separate 30-second 2,000/sec checks.
A separate 1,000/sec Go repeat fails the generator-lag criterion despite valid
responses. These are explicit local configurations and tested lower bounds, not
framework maximum-throughput rankings; the full reports retain latency, misses and limitations.

## Development GC baseline repeat

After the ancestry fixes, a separate repeat holds the qualified Serve application
object fixed and relinks current native GC adapters. It does not measure the new
compiler/library cohort. Two 30-second 300/sec trials validate every response with
7.8–8.1 ms offered p99 latency. Both 400/sec trials fail qualification: 13 and 1,536
arrivals are missed; the second also has 325 ms generator-lag p99. Go passes both
rates in both rounds. At 300/sec, neoCLR uses 2.33–2.46 ms sampled CPU per completion,
versus Go's 0.27–0.30 ms under the explicit single-processor configuration.

These are short same-host checks with experimental paced GC, not a new speedup or
production capacity claim. The earlier 400/sec qualification is historical, not a
guarantee. Separate sampling still identifies allocation, collection and root
publication as the leading areas to investigate.
[Full contract, losses, build hashes and profile](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/serve/gc-return-20261018).

## Development: collection seed-walk optimization

A follow-up gathers a bounded set of intern roots during GC header validation,
avoiding a second full-heap walk when marks are clean and the set fits. At 400
requests/sec the final candidate uses 2.212/2.207 ms CPU per timed completion,
versus 2.305/2.506 ms for baseline. The first pair is about 4% lower; deterioration
in the second baseline prevents attributing its larger difference wholly to the
patch. The candidate still misses arrivals in two of four trials, and a Go control
fails the generator-lag target. No higher reliable sustained rate is established.

The same guest object and experimental paced-GC policy are fixed. Separate collector
benchmarks show gains and an overflow-path regression; all results, including rejected
experiments, remain [in the evidence](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/serve/seed-walk-20261018).


## Development: Task completion allocations

Promise settlement now skips replacing an empty continuation list. A separate Serve
diagnostic records 0.78% fewer allocations and 0.47% fewer allocated payload bytes;
Task identity and callback timing are unchanged. Interpreter/JIT/AOT contract checks pass.

Two reversed-order 60-second comparisons at 300/400 offered requests/sec show no
repeatable CPU gain: a small first-pair reduction reverses in the second pair.
All four candidate points miss arrivals, and one suffers 388 ms generator-lag p99.
Go validates every arrival but fails one of four generator-lag targets. No higher
reliable sustained rate or lower process RSS is established. Both neoCLR arms use
the same current compiler/native runtime and separately built library cohorts;
this is not the older fixed-guest comparison.
[Full results, allocation counts and failures](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/serve/task-completion-20261018).

## Development: lazy Task callback storage

Lazy Promise callback lists and indexed dispatch reduce diagnostic allocations by
13.37% and root publications by 4.94%, preserving three-mode callback contracts.
An unchanged repeat at 400 offered requests/sec passes both candidate trials,
using 5.6–7.2% less sampled CPU per completion than baseline. Candidate CPU remains
about 7.8–8.0 times the matched single-processor Go control.

The initial comparison includes losses in both candidate trials and severe generator
lag in one. Both comparisons are retained: the cleaner repeat does not establish
consistently lossless 400/sec or a higher sustained rate. These are same-host,
60-second trials with experimental paced GC.
[All trials, settings and allocation evidence](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/serve/callback-storage-20261018).

## Development: reusable TaskQueue storage

TaskQueue now reuses ArrayQueue storage. Focused interpreter/JIT/AOT tests cover
FIFO posting, growth and wraparound. A separate diagnostic records 11.71% fewer
allocations, but 1.29% more root publications and about 3.2 KB more peak live payload.

Both 60-second candidate trials validate every arrival at 400 offered requests/sec.
Sampled CPU per completion falls 1.5–3.0% against the lazy-callback baseline; p99
latency rises, so no latency gain is claimed. The same-host, experimental paced-GC
configuration remains far behind Go in CPU cost. Higher rates were not tested.
[Tradeoffs, all trials and raw evidence](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/serve/task-queue-20261018).

## Development: bounded receive-buffer reuse

Buffered HTTP requests now reuse at most one 256-byte and one 4 KiB receive array
per server. Return happens after read completion; streaming-owned buffers remain
separate, and shutdown releases cached references. Inactive exchanges no longer
allocate empty arrays. This is an internal cache, not a public pooling or Span API.

Separate diagnostics record 0.70% fewer allocations and 3.43% fewer allocated
payload bytes, but more collections. Both 60-second candidate trials validate all
arrivals at 400 offered requests/sec. The cleaner pair differs by only 0.3% sampled
CPU/completion; the first baseline has severe generator lag and 717 missed arrivals.
No convincing CPU speedup or higher sustained capacity is established. Candidate
CPU remains roughly 7.2–7.6 times the corresponding Go control.

Ownership regressions pass interpreter/JIT/AOT. The streaming correctness fixture
uses a longer test budget after the original interpreter run fails on both baseline
and candidate; production timeout defaults remain unchanged.
[All results, ownership rules and validation limits](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/serve/receive-pool-20261018).

## Development: encoding provider reuse

Encodings.Utf8/Ascii now reuse stateless providers within runtime-owned invocation
storage. Encoder and decoder sessions remain independent, with strict errors and
BOM handling preserved in interpreter/JIT/AOT tests. Separate diagnostics record
3.07% fewer allocations, 3.78% fewer collections and 0.55% fewer root publications;
peak live payload rises by 48 bytes.

All six matched 60-second Serve/Go trials pass at 400 offered requests/sec. Candidate
CPU per completion is 1.1–1.2% lower than baseline in both orders—a small measured
improvement, without establishing a higher sustained rate. Latency is mixed. The
candidate still uses about 9.7–9.9 times the corresponding single-processor Go
control's CPU. These remain same-host checks with experimental paced GC.
[Tests, limitations, failed experiment and all measurements](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/serve/encoding-cache-20261018).

## Development: request-header reservation

The request decoder now reserves up to 128 header bytes initially, capped by the
configured limit, to avoid repeated small-array growth. Larger headers still grow
normally. Exact-limit and overflow tests pass in interpreter, JIT and AOT. This
trades a small initial reservation for fewer allocations; it adds no pooling API.

Diagnostics show 1.76% fewer allocations and 0.43% fewer root publications, but
3.57% more collections. All six alternating 60-second Serve/Go trials pass at 400
requests/sec. Candidate CPU/completion is 0.9–1.0% lower in both orders, with lower
p99 latency. Two short pairs and variable baseline latency do not establish a
general latency improvement or higher sustained capacity.

The candidate uses roughly 7.4–7.7 times the matching Go controls' CPU. This ratio
must not be compared directly with earlier sessions: Go CPU also varies between
runs. The small within-session neoCLR improvement is the relevant comparison.
[Boundary tests and complete measurement evidence](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/serve/header-reserve-20261018).

## Development: caller profiling and first-callback reservation

A fresh CPU sample still places allocation, root publication and collection at the
top. Caller attribution identifies Promise callback lists that allocate empty
storage immediately before their first addition. Reserving one slot removes that
allocation: 4.36% fewer allocations, 3.47% fewer payload bytes and 1.13% fewer root
publications in the fixed diagnostic workload. Collections increase 2.26%.
Interpreter/JIT/AOT tests preserve deferred completion/cancellation and exercise
multiple callbacks; multiple registrations may incur extra growth from capacity one.

The load results do **not** establish a repeatable CPU speedup or a newly qualified
300/400 requests/sec rate. Initial 400/sec trials and a 300/sec fallback retain all
missed arrivals and generator-lag failures. After the author reports stopping
competing activities, a separate 400/sec repeat still loses arrivals in both neoCLR
builds; both Go controls pass. Candidate CPU/completion is higher in one pair and
lower in the other. Earlier successful trials remain historical results, not a
qualification of this candidate.

The next investigations are scheduler/GC correlation around admission stalls,
better load-generator isolation, and Promise result storage, which accounts for
about 1.12 MB of 3.33 MB allocated in the preceding caller diagnostic. That last
figure identifies where bytes are allocated; it is not a promised saving from
moving the payload elsewhere.
[CPU/caller evidence](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/serve/caller-profile-20261018) ·
[All comparisons, condition changes and limitations](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/serve/callback-reserve-20261018).
