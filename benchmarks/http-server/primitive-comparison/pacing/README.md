# Plain HTTP GC pacing balance — 2026-10-23

The author asked for a good balance between worker throughput and CPU use.
This slice tunes the existing **opt-in** `NEOCLR_GC_PACED` policy to collect after
256 allocations or 1,024 root publications, replacing 16/64. Forced collection,
root validation, allocation-pressure retry and explicit idle/shutdown collection
remain intact. No public API, compiler bridge or worker ownership model changes.

## Decision and scope

A fixed budget is provisional. The benefit is less repeated tracing per request;
the cost is potentially longer retention of unreachable objects and native resources
between collections. Publications are execution events, not milliseconds: this is
not a timed cleanup guarantee. Heap capacity remains bounded and rooted admission
can collect before the budget expires. Retain explicit disposal and shutdown.

.NET uses generational collection with thresholds adjusted during execution,
including a balance between working set and collection frequency; neoCLR's bounded
nonmoving collector and fixed private budget are materially different. This change
amortizes its existing tracing work without introducing an adaptive or generational
collector. See Microsoft's [GC fundamentals](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/fundamentals#conditions-for-a-garbage-collection).
Larger/adaptive budgets, longer offered-load runs and richer handlers remain future
qualification, not inferred improvements.

## Evidence and methodology

`measurement.json` contains the final 12 uninstrumented trials, including forward
and reverse ordering: three budgets times one/four isolated owners, two seconds
warmup and six seconds measured. It uses the same retained Raven application object,
1 MiB per owner, 16 Serve lanes per owner, 32 clients and EOF-aware compiled driver
as the [primary baseline](../README.md). CPU per response uses whole-process CPU
and all completed warmup/window/drain responses. These are short closed-loop local
observations, not sustainable offered-load limits or production capacity claims.

`profiles.json` records separate instrumented four-second diagnostics on the
main owner. GC counters are thread-local; an exit hook on the main thread would
incorrectly see zero counters for an isolated worker host. These runs deliberately
omit the worker-host macro and are not used for capacity scores. Peak arena extent
is not process RSS, and a sampled maximum pause is not a latency bound.

`validation.json` checks all three budgets in optimized and UBSan/bounds builds:
collection just below/at the allocation budget, publication-budget cleanup with
no new allocations, root survival, stale/mismatched boundary rejection, external
collection epoch handling, rooted pressure retry, exhaustion and final reclamation.
The selected source copy is byte-identical to the runtime source. Native network
validation additionally covers the paced scheduler and ordinary socket contracts.

The larger budget reduces repeated tracing, but does not remove the large remaining
per-request runtime cost. The reference-server gap remains substantial. More worker
threads are not a substitute for reducing this work; there is no new shared-heap
async HTTP integration in this slice.

## Final uninstrumented results

| Allocation/publication budget | Owners | Requests/s (two trials) | CPU µs/response | p99 ms |
| --- | --- | --- | --- | --- |
| 16/64 | 1 | 484.5, 505.5 | 2,070.2, 1,983.6 | 67.05, 67.30 |
| 16/64 | 4 | 1,689.5, 1,898.3 | 2,288.5, 2,092.0 | 21.48, 32.26 |
| 64/256 | 1 | 577.8, 584.7 | 1,734.6, 1,712.1 | 62.49, 64.26 |
| 64/256 | 4 | 2,154.5, 1,995.2 | 1,810.5, 1,919.8 | 28.40, 28.11 |
| 256/1024 | 1 | 618.7, 625.0 | 1,605.6, 1,589.5 | 65.41, 58.90 |
| 256/1024 | 4 | 2,135.7, 2,274.2 | 1,777.5, 1,717.9 | 30.40, 26.35 |

The selected 256/1,024 setting improves mean throughput by about 26% with one
owner and 23% with four, while reducing estimated CPU per response by about 21%
and 20%, respectively. Four-owner p99 varies: the fastest baseline tail sample
is better than the candidate; no uniform latency improvement is established.
The intermediate budget helps but saves less CPU in these final trials.

## Separate GC diagnostics

| Allocation budget | Collections/response | GC µs/response | Peak live payload B | Peak arena extent B | Maximum observed GC pause µs |
| --- | --- | --- | --- | --- | --- |
| 16 | 58.05 | 566.5 | 49,369 | 73,200 | 169 |
| 64 | 14.37 | 171.8 | 49,657 | 79,904 | 79 |
| 256 | 3.60 | 58.2 | 49,657 | 82,224 | 74 |

All three report zero GC failures and all allocated objects reclaimed at shutdown.
The selected budget raises peak arena extent by 9,024 bytes (about 12%) in this
small main-owner workload. It reduces collection frequency substantially, while
roughly 3,678 root publications per response remain. That count includes startup
and shutdown and is a diagnostic lead, not proof that all publications are avoidable.

## Reproduction

Use the matching retained source build and client described in the parent README.
The scripts validate its inputs and permit only a recorded native collector source
change; generated code and other runtime inputs must still match. Each experimental
source copy differs only in the two pacing thresholds. All executables are linked
before serial measurement; profiling and tests run afterwards, without overlapping
performance trials. Output directories must be new for measurement.

```sh
python3 benchmarks/http-server/primitive-comparison/pacing/measure.py \
  --build target/primitive-comparison-build/build.json \
  --client target/primitive-comparison-eof/client --output target/plain-pacing-final
python3 benchmarks/http-server/primitive-comparison/pacing/profile.py \
  --build target/primitive-comparison-build/build.json \
  --client target/primitive-comparison-eof/client --experiment target/plain-pacing-final
python3 benchmarks/http-server/primitive-comparison/pacing/validate.py \
  --experiment target/plain-pacing-final
python3 scripts/validate-native-network.py --output target/plain-pacing-network
```

On this machine select Xcode's macOS 26.2 SDK explicitly through `DEVELOPER_DIR`
and `SDKROOT`. No website build or public API snapshot refresh is needed for this
private runtime policy change; the existing feature description remains accurate.
