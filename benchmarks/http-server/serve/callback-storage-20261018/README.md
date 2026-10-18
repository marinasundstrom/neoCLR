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

Uninstrumented reversed-order 60-second comparisons at 400 offered requests/sec
are recorded in the next measurement slice. No throughput or latency gain is inferred
from allocation counts alone. The public reference was regenerated; its API is unchanged.
