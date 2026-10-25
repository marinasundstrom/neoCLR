# Isolated-owner concurrent cost attribution

2026-10-25. Follow-up to [transport diagnostics](close-results.md), using the retained
plaintext managed objects and native recipes. This is an attribution result, not a
collector optimization or default promotion. The synchronous control remains faster.

`guarded.py` adds a control using the live generated object, checked stores, native
providers and heap metadata, but disables automatic epoch starts (both allocation/
publication and idle paths). It uses ordinary synchronous collection. Ownership,
mutation validation and allocation handling remain intact. Existing sync/live
binaries are copied byte-for-byte; manifests hash those binaries and guarded link
inputs. This reuses the control design from the earlier
[plaintext/JSON attribution](../concurrent-workloads/forward-gap-results.md).

## Unprofiled measurements

Three rotating-order rounds, one owner, 32 Go client lanes, one second warmup and
four-second windows validate **159,593 responses** through EOF. CPU and allocations
include startup, warmup, drain and shutdown; throughput counts window completions.
No failed cohort is omitted from these measurements.

| Mode | Median requests/s | Median CPU us/response | Allocations/response | Observed peak RSS bytes |
| --- | ---: | ---: | ---: | ---: |
| Sync | 5,095.75 | 194.37 | 208.022 | 9,469,952 |
| Guarded sync | 2,908.25 | 341.75 | 208.038 | 11,370,496 |
| Live | 2,605.75 | 393.19 | 208.041 | 12,042,240 |

About **74.1% of excess CPU** persists with marking disabled, calculated as
`(guarded - sync) / (live - sync)` from these per-response medians. This is not an
isolated barrier measurement: generated code, collection schedules and live sets
also differ. Tiny allocation differences include lifetime amortization; no allocation
reduction is established. Native allocation traffic is not counted by this metric.

The guarded trials require all six candidate counters to be zero, while live
requires completed epochs and balanced cleanup. Substituting the live executable
under the guarded name deliberately fails that validation. Both retain zero index
metadata after teardown. The test rules out an accidentally active marker control.

## Separate loaded profiles

Three three-second macOS wall-stack samples during five-second load windows validate
another **56,713 responses**. Profiled timings are marked diagnostic-only and excluded
from the table. `summarize-cpu.py` verifies raw stack hashes and retains TLS callers.
The summary input adds only `pair = repeat` to fit that existing summarizer's schema.

| Top-frame observations | Sync | Guarded sync | Live |
| --- | ---: | ---: | ---: |
| TLS resolution | 356 | 519 | 349 |
| Scalar-store adapter | not reported | 448 | 317 |
| Owned frame entry | 388 | 183 | 138 |
| Stack root publication | 307 | 108 | 100 |
| Condition wait | 2,389 | 2,329 | 3,472 |

These are all-thread wall observations, not CPU percentages or comparable per-request
costs. Different modes complete different amounts of work. Waits must not be counted
as busy CPU. The scalar-store adapter accounts for 329 guarded and 185 live TLS
caller observations at offsets 80/304. This makes repeated store admission/TLS work
a stronger next target than marker-only tuning. Common entry/publication work remains
visible. Earlier JSON/task/collection/array evidence supports investigating the same
services, but these fresh measurements cover only this plaintext AOT combination.

## Next qualification

Investigate eliminating repeated store admission with an explicit bounded ownership
proof or stronger compiler stack provenance. Do not simply merge heap and root
ownership or skip checks when the marker is inactive: malformed writes must still
be rejected. Compare any candidate with this guarded control and ordinary sync,
then run existing standalone tasks, collections and arrays plus plaintext/JSON,
with JIT/AOT safety checks. A reduced number of helper calls is insufficient evidence
without CPU/throughput and allocation measurements. Existing failed borrowing and
header-cache experiments remain relevant controls, not reasons to repeat them.

Shared-heap/JIT HTTP, complete pause distributions and concurrent-default acceptance
remain open. Four-owner transport stalls are kept separate from this stable one-owner
cost question. Website/public API claims are unchanged.

## Reproduction

```sh
python3 benchmarks/execution/concurrent-http-owners/guarded.py \
  --build target/concurrent-http-owners-plain --output target/guarded-new
python3 benchmarks/execution/concurrent-http-owners/measure.py \
  --build target/guarded-new --go-client target/http-close-client --owners 1 \
  --modes sync guarded live --seconds 4 --rounds 3 --output target/guarded-new/cost.json
python3 benchmarks/execution/concurrent-http-owners/measure.py \
  --build target/guarded-new --go-client target/http-close-client --owners 1 \
  --modes sync guarded live --seconds 5 --profile-seconds 3 --rounds 1 \
  --output target/guarded-new/profile.json
```

Use fresh paths and the current checked client. `--profile-seconds` requires macOS
`sample`; profile failure fails the run, and the profiler is reaped on load failure.
[cpu-evidence](cpu-evidence/) archives manifests, raw reports, stacks, summaries and
the deliberately rejected guarded control. No production source or default changes.
