# Moderate budgets plus log-pressure completion

2026-10-25. Follow-up to the [watermark experiment](watermark.md), retaining all
previous compiler/runtime optimizations. No production/default or shared-heap
admission change. The author requested recurring plaintext/JSON comparisons, a
standard benchmark/profile sample set, website results, and independent collector
histories. See the resulting [standard suite](../../gc-suite/README.md).

## Fixed-work HTTP

Three interleaved rounds per mode, 8192 requests + 16 warmups, four clients, per sample:
147,744 checked responses. Same generated objects; baseline 256/4096 budgets,
moderate 1024/16384, optional 50% watermark. A 1 MiB arena and 4096-word log remain.

| Sample / policy | Requests/s | Process CPU s | Peak arena bytes | Store cancellations |
| --- | ---: | ---: | ---: | ---: |
| Plain / baseline | 3,592 | 2.4067 | 147,584 | 0 |
| Plain / moderate | 4,046 | 2.0387 | 472,112 | 34 |
| Plain / moderate + watermark | 4,026 | 2.0709 | 309,272 | 0 |
| JSON / baseline | 2,278 | 3.8242 | 144,952 | 0 |
| JSON / moderate | 2,621 | 3.1282 | 354,128 | 0 |
| JSON / moderate + watermark | 2,645 | 3.1375 | 353,088 | 0 |

The combination improves plain throughput/CPU about 12.1%/14.0%, JSON 16.1%/18.0%,
against the original concurrent settings. Memory occupancy increases. It is not a
cross-collector win. Sixty additional ordinary task/collection/array runs compare
watermark-enabled default versus moderate budgets: CPU falls 2.1%, 7.9%, 2.8%,
respectively, with larger arena peaks and identical allocations.

## Isolated application owners

`build-owner-policies.py` relinks the qualified immutable-byref plaintext object
under the three policies for one/four owners. Each owner retains its own heap,
root state and marker. No managed objects are shared between application heaps.
`concurrent-http-owners/measure.py` accepts the named policy modes and now explicitly
rejects nonzero GC failure counts as well as bad responses, epoch balance and cleanup.

Three rotated five-second windows per mode/owner count, one-second warmups, 32 Go
client lanes: **733,711** responses pass. CPU is server process CPU per total response,
including worker threads and warmup/cleanup. Per-owner process counters are not summed.

| Policy | 1 owner rps | 1 owner CPU µs/response | 4 owners rps | 4 owners CPU µs/response |
| --- | ---: | ---: | ---: | ---: |
| Baseline | 3,647 | 289.2 | 9,418 | 406.9 |
| Moderate | 4,156 | 241.1 | 10,182 | 339.3 |
| Moderate + watermark | 4,047 | 249.1 | 9,320 | 351.6 |

Four-owner throughput varies materially across rounds (watermark 8836–10157 rps).
Moderate improves median throughput about 8% and CPU/response 17%. The watermark
eliminates observed store cancellations and lowers the sum of per-owner arena peaks
from about 1.88 to 1.42 MB, but gives up the median throughput gain in this cohort.
That sum is an upper bound from independent owner peaks, not simultaneous occupancy.
No universal best policy or framework ranking follows.

A separate 30-second four-owner watermark run passes **327,153** responses including
warmup/drain, 10,543 measured requests/s. Each owner's peak arena stays below 379 kB,
each ends with zero active epochs and one shutdown cancellation. This is bounded
stability evidence, not a long-duration leak or capacity guarantee. UB/bounds
one/four-owner runs pass another **25,003** responses; sanitized timing is not used
as performance evidence.

A fresh 49,344-response sync comparison and a later complete standard baseline are
retained separately. Sync remains ahead. The [standard baseline](../../gc-suite/results-20261025/README.md)
now provides the website's current table and future self-comparison anchor. Earlier
shorter phases and additional paused diagnostics remain distinct series.

## Decision

Keep policy selection explicit. Moderate budgets are a useful concurrent improvement;
watermark completion trades some CPU/throughput for fewer discarded epochs and less
arena occupancy. The standard suite's low-allocation buffer case directs the next
optimization toward repeated managed-store checks. Concurrent GC is still the intended
general-purpose default; sync retains its resource-constrained use case. Promotion
requires broader runtime/ownership and resource evidence, not identical tuning or a
single favorable metric. Exact commands and raw reports are in `combined-evidence/`.
