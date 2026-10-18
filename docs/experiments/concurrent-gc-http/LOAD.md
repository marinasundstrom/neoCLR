# Actual Serve: applying snapshots versus synchronous GC and Go

2026-10-18, following the [application qualification](APPLY.md).
**Concurrent result application works, but this policy does not demonstrate a
throughput improvement.** It costs about 23–24% more CPU per completion than the
paced synchronous baseline. Tail latency varies in both neoCLR modes. Keep the
adapter experimental; do not promote it as a faster HTTP collector.

## Contract and retained evidence

`compare.py` runs fresh servers with identical 16 server lanes / 64 client slots,
ten-byte responses, a queued asynchronous operation, HTTP/1.1 connection close and
no TLS on the same Apple M1/macOS host. neoCLR uses the same retained qualified AOT
guest object, 32 MiB heap and static archives. This is a current native relink, not
a fresh managed-library/compiler rebuild. The applying C adapter is uninstrumented
`-O2`, with the same qualified source as the sanitizer checks. Go uses the existing
matched net/http executable with GOMAXPROCS=1 and GOGC=100. The concurrent worker
can use an additional core: process CPU includes both threads.

Two 30-second points at each of 200/300 offered req/s reverse runtime and rate order
on round two. Because generator p99 lag exceeded the 5 ms reporting limit in two
adjacent 300/sec trials, a separate two-round 300/sec confirmation follows. All
initial and confirmation results remain available; none are discarded or replaced.
The runs are sequential, with no builds/profilers or synthetic competitors running.
Desktop/OS activity remains uncontrolled; loopback generation is not an independent
load machine. The comparison changes collection scheduling and invalidation/copying
along with tracing placement, so it does not isolate the cost of concurrency alone.

[Initial manifest](evidence/load-comparison/comparison/comparison.json),
[initial summary](evidence/load-comparison/comparison/summary.json),
[confirmation manifest](evidence/load-comparison/confirmation/comparison.json),
[confirmation summary](evidence/load-comparison/confirmation/summary.json).
Each directory contains browsable reports and lossless `.json.gz` originals with
hashes. `summarize.py` derives the rows and CPU peaks from those reports. Build and
guest identities are retained in the manifests and nested reports.

All admitted responses validate, every server/client completion count matches,
and all servers exit cleanly. An admission miss is an **unsent generator arrival**
at the 64-slot cap, not a server rejection. Success p99 excludes misses, so read it
with the miss count. Completed/sec includes only completions inside the window;
a final response during drain is not added to timed throughput. The provisional
target is all offered requests successful, response p99 <=250 ms, and generator
p99 lag <=5 ms. It is not a product SLO or a maximum-capacity measurement.

## Initial comparison

CPU is process milliseconds per timed completion. Peak CPU is cores consumed over
adjacent approximately half-second samples inside the window; it misses shorter
spikes and includes sample timing/precision error. One core means one CPU-second
per wall-second, not 100% of the whole machine.

| Mode / round / offered rate | Timed req/s | Misses | CPU ms/request | Response p99 ms | Generator p99 ms | Sampled peak cores | Target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Sync / 1 / 200 | 200.00 | 0 | 2.262 | 4.58 | 0.39 | 0.50 | pass |
| Concurrent / 1 / 200 | 200.00 | 0 | 2.773 | 43.74 | 0.63 | 0.61 | pass |
| Go / 1 / 200 | 200.00 | 0 | 0.213 | 3.39 | 0.87 | 0.06 | pass |
| Sync / 1 / 300 | 300.00 | 0 | 2.208 | 110.20 | 0.65 | 0.91 | pass |
| Concurrent / 1 / 300 | 300.00 | 0 | 2.733 | 5.61 | 0.50 | 0.89 | pass |
| Go / 1 / 300 | 300.00 | 0 | 0.190 | 3.05 | 0.75 | 0.08 | pass |
| Go / 2 / 300 | 300.00 | 0 | 0.196 | 44.79 | 22.50 | 0.08 | fail |
| Concurrent / 2 / 300 | 295.03 | 149 | 2.891 | 328.91 | 35.40 | 1.21 | fail |
| Sync / 2 / 300 | 300.00 | 0 | 2.291 | 42.76 | 0.83 | 0.83 | pass |
| Go / 2 / 200 | 200.00 | 0 | 0.195 | 5.73 | 1.74 | 0.06 | pass |
| Concurrent / 2 / 200 | 200.00 | 0 | 2.758 | 4.57 | 0.40 | 0.63 | pass |
| Sync / 2 / 200 | 200.00 | 0 | 2.258 | 79.69 | 0.76 | 0.56 | pass |

## Targeted 300/sec confirmation

| Mode / round | Timed req/s | Misses | CPU ms/request | Response p99 ms | Generator p99 ms | Sampled peak cores | Target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Sync / 1 | 299.97 | 0 | 2.235 | 4.70 | 0.50 | 0.71 | pass |
| Concurrent / 1 | 299.97 | 0 | 2.751 | 110.70 | 1.09 | 1.15 | pass |
| Go / 1 | 300.00 | 0 | 0.191 | 8.24 | 3.74 | 0.08 | pass |
| Go / 2 | 300.00 | 0 | 0.206 | 5.86 | 2.14 | 0.08 | pass |
| Concurrent / 2 | 300.00 | 0 | 2.752 | 5.50 | 0.51 | 0.89 | pass |
| Sync / 2 | 299.93 | 2 | 2.226 | 139.08 | 0.90 | 0.87 | fail |

The initial mean CPU/request is 2.255 ms sync versus 2.789 ms concurrent (+23.7%);
the confirmation is 2.230 versus 2.752 ms (+23.4%). Go averages about 0.198 ms in
both series: these matched tiny-response points show roughly 11–14 times lower
CPU cost than neoCLR, not a corresponding capacity ratio. This is a net/http
runtime comparison; its collector is not forced to do equivalent work and no Go
maximum is measured. Sampled RSS in the initial series is about 12.6 MiB (values
12,896–12,912 KiB) sync, 13.9 MiB concurrent (14,192–14,240 KiB), and 14.9–15.1 MiB
Go (15,216–15,424 KiB); these are process samples, not exact managed live bytes.

The 200/sec points pass in both rounds for all three runtimes. Across all four
300/sec trials per neoCLR mode, neither passes every point; the large initial
concurrent miss cannot be assigned solely to GC because generator lag also failed.
The confirmation establishes neither a throughput win nor uniformly lower latency.
Longer independent-generator tests are still needed for a sustained-capacity claim.

## Work performed by the concurrent adapter

The initial four concurrent runs collectively start 890,496 snapshots: 591,529
apply and 298,967 discard (33.6%). They reclaim 356,218,922 payload bytes through
the coordinator, with zero trace errors/capture rejections. All 11,521,507 native
allocations are reclaimed by process teardown. These counts include warm-up and
control traffic; they are not timed-only collections. The extra worker therefore
is doing real work, not merely being enabled. About 48 million deferred boundary
visits and 3.49 million fallback visits also indicate how often the adapter is
consulted; visits are not the same as full synchronous collections.

The next investigation is a code/algorithm review of repeated capture, heap walks,
mutex polling, discarded work and root/scheduler transitions, informed by existing
collectors. Counts identify candidates, not a phase-level CPU attribution. Review
pacing and cheap completion observation before adopting barriers or a new heap
layout. Preserve the synchronous baseline and full safety qualification.

## Reproduce

```sh
python3 docs/experiments/concurrent-gc-http/compare.py \
  --sync-build target/http-concurrent-readiness/native \
  --apply-build target/http-apply-release \
  --go-build target/http-concurrent-readiness/go \
  --rates 200 300 --duration 30 --output target/http-gc-comparison-new
python3 docs/experiments/concurrent-gc-http/summarize.py target/http-gc-comparison-new
```

Repeat with only `--rates 300` in another fresh directory for the targeted control.
Production source, APIs and collector selection remain unchanged.
