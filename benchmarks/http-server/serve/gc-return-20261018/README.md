# Return to GC and Serve benchmarking — 2026-10-18

After the primitive/enum ancestry work, the author directed returning to GC and
HttpServer benchmarking and updating the website. This repeat re-establishes the
native GC baseline before selecting another optimization.

## Contract

The qualified Serve application object is held fixed from the earlier root-validation
comparison, then relinked against current native sources. This is not a measurement
of the newly rebuilt primitive/enum library or compiler cohort, and does not estimate
an ancestry-change speedup. The relink manifest retains source, object and executable
hashes. No runtime optimization or collection-policy change is part of this slice.

The existing compare-go.py alternates neoCLR/Go then Go/neoCLR, reversing rate order
on the second round. Each rate runs for 30 seconds after a separate warm-up. The
matched workload returns ten bytes over loopback HTTP/1.1 with connection close,
awaits queued handler work, uses 16 server lanes and 64 load-generator slots.
neoCLR uses experimental paced GC; Go uses GOMAXPROCS=1 and GOGC=100. Rates are 300
and 400 requests/sec. Builds, tests and sampling are excluded from the timing runs.

The existing provisional target requires every offered request to validate, offered
p99 latency at most 250 ms and generator dispatch-lag p99 at most 5 ms. Arrivals
missed at the client cap remain losses; completions during drain are excluded from
timed throughput. Process CPU and RSS are sampled. Same-host scheduling and short
windows limit generalization; passing is a tested operating point, not production
capacity or a maximum throughput claim. Profiling runs are diagnostic only.

## Results

| Runtime/round | Offered/sec | Timed completions/sec | Missed arrivals | CPU ms/completion | Offered p99 ms | Generator lag p99 ms | Target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| neoclr-1 | 300 | 299.97 | 0 | 2.330 | 8.07 | 0.72 | Pass |
| neoclr-1 | 400 | 398.70 | 13 | 2.373 | 166.42 | 1.40 | Fail |
| neoclr-2 | 400 | 348.73 | 1536 | 2.499 | 733.61 | 325.44 | Fail |
| neoclr-2 | 300 | 299.97 | 0 | 2.455 | 7.76 | 0.52 | Pass |
| go-1 | 300 | 300.00 | 0 | 0.266 | 7.04 | 3.29 | Pass |
| go-1 | 400 | 400.00 | 0 | 0.235 | 5.41 | 1.90 | Pass |
| go-2 | 400 | 400.00 | 0 | 0.230 | 5.27 | 1.99 | Pass |
| go-2 | 300 | 300.00 | 0 | 0.298 | 6.32 | 2.73 | Pass |

Both neoCLR 300/sec runs validate all 9,000 arrivals and meet the provisional target.
One response in each finishes during drain. CPU/completion is 2.330–2.455 ms,
versus Go's 0.266–0.298 ms at that same rate (8.2–8.8× for the matched rounds).
No new speedup is claimed: this slice measures the existing root-search optimization.

Neither neoCLR 400/sec trial qualifies. The second has 325 ms generator-lag p99,
so the throughput deterioration is not attributable solely to server execution.
All admitted responses validate, server error counts remain zero and shutdown
accounting agrees. Go validates all 42,000 offered requests and passes all four
points. These results do not establish its maximum capacity.

Sampled peak RSS is 12,864–12,912 KiB for neoCLR and 15,008–15,376 KiB for Go.
This is process RSS, not bytes per request, and no memory improvement is claimed.
The earlier 120-second 400/sec qualification remains historical evidence; current
failures reinforce that it is not a repeatable guarantee. The new 300/sec repeats
are only 30 seconds each, so longer memory/latency stability remains unestablished.

[Run order and commands](evidence/comparison.json), [native relink inputs](evidence/native-build.json),
and full lossless reports accompany the compact summaries:
[neoCLR round 1](evidence/neoclr-1.json), [neoCLR round 2](evidence/neoclr-2.json),
[Go round 1](evidence/go-1.json), [Go round 2](evidence/go-2.json).


## Separate GC profile and next investigation

The 5-second, 1 ms wall sampler reports 3,676 main-thread samples. Top-of-stack
counts include allocation (766), indexed collection (733), stack-root publication
(660), strlen (236), and strstr (75). These are samples, not exact CPU costs or an
isolated estimate of an optimization's benefit. The diagnostic 300/sec run misses
5 of 6,000 arrivals while sampling; it is excluded from capacity qualification.

Allocation, collection and root publication remain the leading investigation areas.
Before changing allocation or validation, separate the cost of each operation from
its frequency and preserve root/descriptor rejection tests. Removing repeated strlen
requires a safe immutable-descriptor/lifetime contract or equivalent validation;
address-only caching is not a safe substitute. No collector replacement, GC-policy
change or further optimization is claimed here.

[Profile summary](evidence/sample-summary.json), [raw sample](evidence/sample.txt.gz)
and [diagnostic request outcomes](evidence/profile.json) are retained.

Website validation: 23 focused builder tests pass; the complete local build checks
4,449 pages. That build exposed a pre-existing Serve sidecar-ID mismatch in the
temporary CLI renderer, repaired with a scoped generic-arity ID adapter and a
regression preserving the native contracts. No deployment was performed.
