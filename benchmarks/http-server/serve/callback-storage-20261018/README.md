# Lazy callback storage and indexed dispatch — 2026-10-18

Follow-up to the [empty-settlement allocation investigation](../task-completion-20261018/README.md),
prioritized by the author before the proposed Serve API work.

## Change

Promise allocates its private callback list only on the first pending registration.
Settlement detaches and releases that list instead of allocating a replacement.
Callback posting and TaskQueue's detached-batch draining use indexed traversal,
avoiding iterator objects over private lists that application callbacks cannot mutate.

Results are still stored before callbacks are posted; completed/cancelled promises
send later registrations directly to the queue. The queue detaches each batch before
invoking it, so work posted by callbacks remains deferred to the next batch. FIFO
order, duplicate-settlement behavior, Task identity and queue ownership are unchanged.
The nullable callback field keeps one reference slot, with normal compiler-emitted
GC roots; this is not pooling or reusing externally observable Task objects.

As with .NET's Task, a reference-type awaitable does not require every internal
collection to be allocated eagerly. This changes implementation overhead, not the
public async contract. Keep the [existing Task/ValueTask comparison](../task-completion-20261018/README.md#task-and-http-api-comparison)
and defer a new awaitable ABI.

## Validation and diagnostics

The [shared contract consumer](../task-completion-20261018/Contracts.rvn) now checks
multiple registered callbacks and work enqueued by a callback, as well as completion,
cancellation, duplicate settlement and deferred late observation. Interpreter/JIT/AOT
pass against the rebuilt library. [Results](evidence/contracts.json.gz).

Candidate-only diagnostic runs validate 200 requests, 20 warm-up requests and the
stop exchange, matching the preceding slice's diagnostic workload. Totals include
startup/shutdown and are not isolated per-request counters.

| Counter | Previous candidate | This candidate | Reduction |
| --- | ---: | ---: | ---: |
| Allocations | 85,172 | 73,787 | 13.37% |
| Allocated payload bytes | 4,145,372 | 3,783,060 | 8.74% |
| Collections | 19,638 | 18,393 | 6.34% |
| Root publications | 745,931 | 709,118 | 4.94% |
| Peak live payload bytes | 46,260 | 46,112 | 0.32% |

All allocations reclaim by shutdown and GC failures are zero. Diagnostic GC elapsed
time increased (209 ms to 577 ms), but this run overlapped tooling/build activity;
it is not valid comparative timing and is explicitly excluded from capacity claims.
The allocation-site run independently agrees on the 73,787 allocation total.
[GC counters](evidence/gc-counts.json), [diagnostic requests](evidence/diagnostic.json),
[caller attribution](evidence/allocation-summary.json),
[attribution requests](evidence/allocation-requests.json).

Builds reuse the same compiler/backend, native link recipe, Serve source and current
native GC as the previous candidate; only the source-built Task library changes.
Use the preceding slice's `build.py`, `verify.py` and `allocation-sites.py` with this
library. Compressed build/library manifests retain commands and hashes.

The public reference was regenerated; its API is unchanged.

## Sustained-load comparisons and requested rerun

Unchanged 400/sec offered load, 60 seconds per trial, 16 server lanes and 64 client
slots; order baseline/candidate/Go then Go/candidate/baseline. Baseline is the
preceding empty-settlement fix. Both neoCLR builds use experimental paced GC.
Go uses GOMAXPROCS=1 and GOGC=100. No build or profiling ran during these trials.
The generator shares the host. Every offered arrival remains in the denominator;
timed throughput excludes drain completions. Qualification requires all arrivals
valid, offered p99 ≤250 ms and generator-lag p99 ≤5 ms.

| Run | Arm | Timed req/s | Misses | CPU ms/completion | Offered p99 ms | Lag p99 ms | Target |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| comparison | [baseline-1](evidence/comparison-baseline-1.json) | 398.42 | 94 | 2.258 | 159.63 | 1.59 | fail |
| comparison | [candidate-1](evidence/comparison-candidate-1.json) | 375.88 | 1444 | 2.303 | 420.63 | 182.74 | fail |
| comparison | [go-1](evidence/comparison-go-1.json) | 400.00 | 0 | 0.225 | 10.91 | 4.84 | pass |
| comparison | [go-2](evidence/comparison-go-2.json) | 400.00 | 0 | 0.220 | 10.84 | 4.61 | pass |
| comparison | [candidate-2](evidence/comparison-candidate-2.json) | 399.10 | 14 | 2.212 | 156.05 | 2.86 | fail |
| comparison | [baseline-2](evidence/comparison-baseline-2.json) | 391.22 | 518 | 2.358 | 250.43 | 35.75 | fail |
| rerun | [baseline-1](evidence/rerun-baseline-1.json) | 398.57 | 84 | 2.292 | 175.55 | 1.55 | fail |
| rerun | [candidate-1](evidence/rerun-candidate-1.json) | 399.98 | 0 | 2.164 | 37.93 | 0.69 | pass |
| rerun | [go-1](evidence/rerun-go-1.json) | 400.00 | 0 | 0.271 | 5.70 | 1.98 | pass |
| rerun | [go-2](evidence/rerun-go-2.json) | 400.00 | 0 | 0.270 | 5.87 | 2.29 | pass |
| rerun | [candidate-2](evidence/rerun-candidate-2.json) | 400.00 | 0 | 2.115 | 30.25 | 0.66 | pass |
| rerun | [baseline-2](evidence/rerun-baseline-2.json) | 399.98 | 0 | 2.280 | 37.58 | 0.64 | pass |

The rerun candidate passes both trials and uses 5.6%/7.2% less sampled server CPU
per completion than the corresponding baseline rounds. Its 2.115–2.164 ms remains
about 7.8–8.0 times the Go control. The initial comparison has a CPU regression in
the first pair and losses in both candidate trials, including severe generator lag.
Thus the rerun supports a CPU benefit under these conditions, but the combined
evidence does not establish consistently lossless 400/sec or a higher sustained
capacity. Allocation savings are independently demonstrated; latency improvement
and process-memory savings are not guaranteed. All twelve trials are retained,
including lossless compressed request/resource records.
