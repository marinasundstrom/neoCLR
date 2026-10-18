# Reusable TaskQueue storage — 2026-10-18

Author-approved follow-up to [lazy callback storage](../callback-storage-20261018/README.md).
Replace TaskQueue's fresh ArrayList per detached batch with the existing ArrayQueue
FIFO. Dequeue clears the stored callback reference before invoking it. Enqueued
callbacks stay behind existing work, including when a callback posts more work;
recursive pumping remains rejected. Public Task/Promise identity, deferred delivery
and completion/cancellation contracts are unchanged. No Serve API change is included.

This reuses the library's array-backed ring queue rather than introducing a custom
allocator or dispatcher. Compared with the familiar .NET Task model, it changes
only private scheduling storage; it does not introduce ValueTask restrictions or
pool user-visible task identities. The existing [Task comparison](../task-completion-20261018/README.md#task-and-http-api-comparison)
continues to apply. The queue retains its high-water capacity until its owner dies,
although consumed slots release their callback references. This is a deliberate
memory/CPU tradeoff, not a universal reduction in memory use.

## Validation

The shared [contract consumer](../task-completion-20261018/Contracts.rvn) adds queue
growth, wraparound, empty-drain reuse and reentrant posting checks. Interpreter,
JIT and AOT pass using the rebuilt four-library cohort. The compiler/backend/native
host remain fixed; the public reference is regenerated with no API change.
[Three-mode evidence](evidence/contracts.json.gz).

Separate diagnostics use 200 measured, 20 warm-up and one stop request, including
startup/shutdown in totals. Every measured response validates; allocation-site
instrumentation independently agrees on the total. GC timing from diagnostic runs
is not a controlled capacity comparison and is not used to claim a speedup.

| Counter | Lazy-callback baseline | ArrayQueue | Change |
| --- | ---: | ---: | ---: |
| Allocations | 73,787 | 65,150 | −11.71% |
| Allocated payload bytes | 3,783,060 | 3,495,480 | −7.60% |
| Collections | 18,393 | 18,578 | +1.01% |
| Root publications | 709,118 | 718,247 | +1.29% |
| Peak live payload bytes | 46,112 | 49,283 | +3,171 bytes |
| Peak arena bytes | 69,696 | 74,416 | +4,720 bytes |

All 65,150 allocations are reclaimed by shutdown, with zero GC failures.
[GC counters](evidence/gc-counts.json), [request diagnostics](evidence/diagnostic.json),
[caller allocation attribution](evidence/allocation-summary.json),
[attribution requests](evidence/allocation-requests.json). Compressed build and
library manifests retain compiler commands and hashes. Use the preceding slice's
build.py, verify.py and allocation-sites.py with this library.

## Sustained load

The separate uninstrumented comparison uses 400 offered requests/sec for 60 seconds,
16 server lanes, 64 client slots and reverse order: baseline/candidate/Go then
Go/candidate/baseline. Baseline is the lazy-callback candidate. Both neoCLR builds
use experimental paced GC; Go uses GOMAXPROCS=1 and GOGC=100. The generator shares
the host. No builds or profiling run during timing. All offered arrivals count,
including admission misses; timed throughput excludes drain completions. The
provisional target requires all arrivals valid, offered p99 ≤250 ms and generator
lag p99 ≤5 ms. These are local tested points, not production or maximum capacity.

| Arm | Timed req/s | Misses | CPU ms/completion | Offered p99 ms | Lag p99 ms | Target |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| [baseline-1](evidence/baseline-1.json) | 399.98 | 0 | 2.156 | 32.68 | 0.73 | pass |
| [candidate-1](evidence/candidate-1.json) | 399.98 | 0 | 2.092 | 43.80 | 0.65 | pass |
| [go-1](evidence/go-1.json) | 400.00 | 0 | 0.265 | 5.40 | 1.91 | pass |
| [go-2](evidence/go-2.json) | 400.00 | 0 | 0.254 | 7.41 | 3.50 | pass |
| [candidate-2](evidence/candidate-2.json) | 399.98 | 0 | 2.095 | 48.16 | 0.77 | pass |
| [baseline-2](evidence/baseline-2.json) | 399.98 | 0 | 2.127 | 20.69 | 0.85 | pass |

Both candidate trials validate all 24,000 arrivals. CPU/completion is lower in
both pairs; latency is higher than baseline, so this is not a latency improvement.
The queue allocation reduction is useful but modest relative to the remaining Go
CPU gap. No rate above 400/sec was tested and these short local trials do not
establish long-duration production capacity. Earlier noisy failures are retained
in the preceding investigation rather than replaced by this cleaner run.
