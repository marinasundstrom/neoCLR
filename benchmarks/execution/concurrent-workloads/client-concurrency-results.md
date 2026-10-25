# HTTP client concurrency and GC scheduling (2026-10-25)

## Why broaden the comparison

The [within-store lookup work](snapshot-reuse-results.md) improves CPU modestly,
but sequential requests still leave a large concurrent-GC gap. The retained
plaintext trial recorded roughly 2,078 synchronous versus 6,220 live collection
attempts for 2,064 responses. Automatic live GC performs idle cleanup and then
starts another epoch. Sequential clients commonly reach idle between requests.
That observation warrants a queued-work comparison before changing GC pacing.

The existing fixed-work runner now accepts `--concurrency 1..128`, capped by the
measured request count. Each client loop opens a fresh connection per request and
checks the complete status/body. Warmup remains sequential. A barrier and event
start pre-created client threads together; request counts are distributed exactly,
including remainders. All clients finish before shutdown. One client preserves the
original serial execution path. Worker count here means **client threads**, not
server workers or shared-heap GC admission.

Every trial retains response latency p50/p95/p99/max and now records client-process
CPU for the measured interval. Server CPU includes startup, warmup and cleanup.
Client latency includes connection setup, server queueing, validation and client
scheduling. Python's GIL, the local OS and connection-close overhead limit the
interpretation: this is a closed-loop load experiment, not a maximum-capacity or
cross-framework benchmark. Request latency is not a GC pause distribution.

No runtime, compiler, collection policy, ownership or managed code changes are
made in this slice. The prior compiler/JIT/AOT and ordinary-workload safety evidence
is unaffected. A 37-request, three-warmup, four-client smoke test covers uneven
partitioning, exact responses, allocation equality and clean shutdown across all
three modes (120 responses). The summary utility rejects duplicate cohorts and
mixed work sizes; raw validation output is retained.

## Matrix results

The successful six-cohort matrix validates 111,456 responses. Three trials per
mode use 2,048 measured plus 16 warmup responses. Median rates and process CPU:

| Handler | Clients | Sync req/s | Guarded-sync req/s | Live req/s | Sync CPU s | Guarded CPU s | Live CPU s |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Plaintext | 1 | 4,840 | 1,895 | 1,867 | 0.389 | 1.009 | 1.129 |
| Plaintext | 4 | 4,963 | 1,625 | 1,294 | 0.408 | 1.259 | 1.610 |
| Plaintext | 16 | 5,085 | 1,640 | 1,296 | 0.405 | 1.269 | 1.625 |
| JSON | 1 | 2,439 | 1,181 | 1,095 | 0.651 | 1.659 | 1.878 |
| JSON | 4 | 3,069 | 1,024 | 818 | 0.669 | 1.984 | 2.568 |
| JSON | 16 | 3,080 | 1,054 | 840 | 0.672 | 1.960 | 2.518 |

Live end-to-end p99 trial medians rise from 0.61 to 3.42 to 13.00 ms for plaintext
and 0.98 to 5.79 to 20.25 ms for JSON. Sync p99 is 0.27/0.90/3.46 ms and
0.45/1.43/5.73 ms respectively. These are closed-loop response latencies, not GC
pauses or open-loop tail-latency guarantees.

Plaintext live epochs fall from 2,075 at one client to 959 at four, with idle
restarts falling from 2,074 to 34. Instrumented collection attempts fall from
6,226 to 1,954, **yet CPU increases**. JSON epochs fall from 2,079 to 1,895, idle
restarts from 2,078 to 40, and collection attempts from 6,238 to 3,832. Thus idle
restart frequency alone cannot explain the performance gap.

One/four-client allocations match exactly across modes and trials: plaintext
429,856 / 33,057,881 bytes; JSON 943,817 / 54,065,775 bytes. At 16 clients, the
initial plaintext cohort completed all 18,576 responses and clean shutdown but
failed the existing equality assertion: allocations ranged from 429,856 to
429,862, with each extra object contributing 528 bytes. Preserve that report as a
failed strict comparison, not a runtime correctness failure or a passing benchmark.

An explicit `--allow-allocation-variation` mode retains exact response/lifecycle
checks while recording equality status and ranges. Repeated 16-client plaintext
trials range from 429,856 to 429,865 allocations; JSON from 943,817 to 943,823.
Counts and byte ranges are retained per mode in the summary. The server's single
cached receive buffer per size class (`HttpServer.rvn`, `RentReceiveBuffer`) can
miss under overlap. This is a plausible source of small scheduling-dependent
variation, not proven attribution. Exact equality remains the runner default.

At four clients, live peak arena use grows from 81,624 to 146,936 bytes for
plaintext and from 100,344 to 144,936 for JSON. Plaintext checked store walks rise
from about 467,065 to 517,705; JSON walks slightly fall, from 753,986 to 739,927.
Walk counts alone do not measure their lengths or explain total CPU. The increase
in guarded-sync CPU points to costs common to the guarded store path and queued
execution, in addition to concurrent marking. Do not infer that more client
threads test the unqualified shared-worker heap.

## Diagnostic profile

A separate four-client plaintext cohort with macOS `sample` checks another 24,624
responses. It is marked diagnostic-only; none of its timing values enter the
performance table. Raw stack profiles, hashes and a checked summary are retained.
Top-frame observations (reporting threshold five):

| Frame | Sync | Guarded-sync | Live |
| --- | ---: | ---: | ---: |
| `block_valid` | not reported | 187 | 177 |
| `experiment_gc_store` | not applicable | 166 | 150 |
| `_tlv_get_addr` | 74 | 98 | 63 |
| `enter_owned` | 94 | 33 | 18 |
| `stack_roots` | 65 | 23 | 12 |
| `validated_bases` | not reported | 24 | 20 |

Main-thread observations total 674/687/585 respectively. The live worker has 585
observations, mostly condition waiting (570 reported top-frame observations).
These are wall-stack observations, including waits, **not CPU utilization
percentages**. Missing frames are not zero. The profiles support prioritizing
store/header lookup work under overlap rather than assuming the marker consumes
most CPU. They do not isolate how much time belongs to header validation per
allocation versus repeated heap walking; that is the next diagnostic question.

## Decision

Keep sync as default. Prioritize the cost of store/heap lookup under a larger live
working set and common queued execution/root work. Do not change pacing solely
because sequential idle restarts look frequent; this matrix contradicts that being
the whole problem. Any next optimization must retain a queued HTTP comparison as
well as sequential and ordinary-workload evidence.

## Evidence and reproduction

Use `concurrent-store/measure-plaintext.py --concurrency N --requests 2048
--pairs 3 --include-guarded-sync`, separately for plaintext and JSON, with retained
`optimizer-http-build` objects. Cohorts run sequentially; each compiles all hosts
before timing. The server modes are ordinary sync, guarded-sync (same admitted
store path without concurrent epochs), and live. Allocation totals/ranges, failures, collector
lifecycle and response counts remain checked in every trial. Use
`--allow-allocation-variation` explicitly for the 16-client range cohorts.

`summarize-concurrency.py --measurements ... --output ...` verifies matching work,
mode sets, cohort uniqueness and successful non-profiled inputs. It reports median
trial values and medians of trial latency percentiles, not pooled percentiles.
Raw `client-concurrency-*.json.gz` reports retain commands, hashes and individual
trials. The aggregate JSON and this report distinguish request latency, collector
instrumentation and CPU measurements; none alone proves mutator pause behavior.
