# Whole-request cost after seed optimizations

2026-10-25. The previous [direct-seed experiment](direct-seed-results.md) removed
24–41% of one phase's CPU with only small mixed application gains. Refresh the
whole-request view before selecting further collector micro-optimizations.

Six one-second macOS stack profiles pass 49,248 checked responses: plaintext and
JSON, each with ordinary sync, guarded-sync and live concurrent modes. Generated
objects include the qualified stack-argument optimization; live and guarded use
direct seeding and the inlined hybrid allocation index. These are diagnostic runs,
not throughput evidence. The profiles include all threads and sleeping observations;
counts below are not CPU percentages. Short profiles cannot prove absent costs.

| Top-frame observations | Plain sync | Plain guarded | Plain live | JSON sync | JSON guarded | JSON live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| TLS resolution | 90 | 118 | 100 | 78 | 127 | 107 |
| Scalar store | — | 94 | 74 | — | 100 | 60 |
| Owned entry | 89 | 73 | 37 | 84 | 52 | 41 |
| Stack roots | 58 | 47 | 26 | 60 | 31 | 30 |
| Root publication | 21 | 16 | 14 | 23 | 12 | 13 |

The scalar-store entry is absent from the sync collapsed top-frame table; the
dash denotes no reported observation, not a measured cost. Live condition
waits contribute 499/497 observations, which must not be interpreted as busy CPU.
The marker is a separate thread; all-thread totals are not a single-thread budget.
TLS caller counts reconcile exactly, with zero unattributed observations in all six
profiles. Heap-owner and candidate-owner resolutions remain visible at scalar-store
return offsets 80 and 304, consistent with the earlier [instruction audit](worker-tls-results.md).
For live plaintext their counts are 19/22; JSON 31/36. No new instruction mapping
is inferred from an offset belonging to an unrelated function.

## Uninstrumented control gap

Reuse the independently measured, balanced direct-seed cohort instead of treating
profiled durations as performance evidence. Median process CPU for the fixed
2,048-request plus 16-warmup work (seconds):

| Workload | Sync | Guarded-sync | Live | Guarded share of live excess over sync |
| --- | ---: | ---: | ---: | ---: |
| Plaintext | 0.406307 | 0.605143 | 0.704961 | 66.6% |
| JSON | 0.662920 | 0.946710 | 1.098137 | 65.2% |

Share = `(guarded - sync) / (live - sync)`, using medians, not paired causal
estimates. Guarded and live use the same generated managed object; guarded disables
concurrent autostart/idle triggering while retaining checked stores and synchronous
collection. Sync uses its separately qualified compilation. Different generated
stores, native paths, collection schedules and live sets prevent assigning the
entire difference to barriers. Nevertheless, most of the gap persists without
concurrent marking, so optimizing finish alone cannot explain it away.

The summary validates response counts, success, balanced pair IDs, work/client counts,
zero guarded epochs, positive live completions, clean index/store shutdown and exact
TLS accounting. Four negative controls reject failed responses, missing pairs,
active guarded epochs and TLS mismatch. Existing direct-seed summary checks separately
verify matched object/build differences. Memory/allocation ranges remain in the
underlying reports; no new reduction or pause advantage is claimed.

## Next implementation target and boundaries

Prioritize reducing repeated internal store dispatch/owner-state resolution through
an explicit lifetime-bounded owner capability, alongside common frame/root setup.
This is a design target, not a new contract or implemented exemption. The current
store API validates heap ownership and active candidate context before even an
outside-arena write. Moving that write ahead of validation would change rejection
semantics and is not justified by these profiles. Disabling checks in inactive epochs
also loses destination validation. Keep heap ownership separate from collector state;
an explicit handle would need context, thread and epoch/lifetime rules plus revocation
and cross-owner tests before use. The existing generated stack proofs remain useful.

This reuses the [.NET/Go metadata and ownership research](allocation-index-results.md)
and prior compiler-provenance work; no managed API, Runtime Contract, compiler or
Raven bridge changes are implemented here. Shared-worker and concurrent-default
acceptance remain open. Sync remains faster. This evidence redirects work toward
costs present outside collection rather than assuming a more concurrent marker will
remove the entire throughput gap.

Reproduction: `whole-profile-commands.json.gz`; raw reports
`whole-profile-{plain,json}.json.gz`; raw stacks
`whole-profile-{plain,json}-{mode}.txt.gz`; checked stack summaries
`whole-profile-{plain,json}-summary.json`; combined `whole-gap-summary.json` and
`whole-gap-negative.json.gz`. Commands need the retained bundle/object paths.
Invoke `summarize-whole-gap.py`
with `--root target/concurrent-workloads --output <path>`. Inputs are hash recorded;
raw stack hashes are verified by `summarize-cpu.py` before aggregation.
