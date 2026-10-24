# Indexed old-object stores — 2026-10-24

The experimental scalar adapter now uses the active epoch's detached allocation
index to find an old allocation containing an interior destination. It checks the
live descriptor's validity, allocated state, span, payload size and kind against
the saved identity, then passes the validated extent to a private grouped-write
helper. This removes the adapter's linear scan and the public grouped-write hook's
second allocation lookup. The public grouped-write entry still validates callers.
Epoch rules prohibit moving/reclaiming/retiring old allocations; this optimization
uses that existing invariant rather than adding an address cache or extra memory.

New allocations, inactive epochs and index misses retain the checked linear
fallback. All width, payload-extent, published-array-shape, logging and overflow
cancellation behavior remains. This changes only the experimental generated native
collector. Synchronous defaults and ordinary compiler output are unchanged.

## Matched measurement

`measure-stores.py` builds indexed and `NEOCLR_LIVE_LINEAR_STORE` reference variants
from one generated source with the same compiler flags. The reference retains the
original linear adapter plus public write lookup. Runs are linear/indexed/indexed/
linear at each size. Each trial retains a linked graph, runs 100 epochs and writes
32 pseudo-random old-object scalar lanes per epoch (3,200 stores). The marker has
finished its first pass before timed stores, and 64 logged words fit the buffer.
This intentionally isolates lookup without worker contention or cancellation.

| Objects | Linear store CPU, two runs | Indexed store CPU, two runs | Store CPU reduction | Whole workload CPU reduction |
| --- | --- | --- | --- | --- |
| 1,024 | 16.46 / 18.29 ms | 0.342 / 0.299 ms | 98.2% | 70.1% |
| 16,384 | 194.86 / 188.03 ms | 0.426 / 0.397 ms | 99.8% | 73.6% |

Reductions compare arithmetic means. Store CPU is owner-thread CPU around the
write loop, including clocks and value checks; process CPU includes begin, wait,
stores and finish, but excludes initial graph allocation and final teardown. Wall
times are retained separately. Short indexed intervals are timer/noise sensitive;
these are paired observations, not confidence intervals. There is no synchronous
comparison here because this isolates two versions of the same live-store adapter.
The earlier whole-collector synchronous baseline remains relevant and is not
superseded by these numbers. No HTTP, overlap, pause-tail or RSS claim follows.

## Correctness and evidence

Optimized and UBSan/bounds native suites pass, including 97 narrow scalar ranges,
new array construction, immutable published shapes, noncollecting overflow recovery,
12,000 differential object checks, native scan fixtures and the missing-deletion
negative control. A new idle-worker test changes an old descriptor's size/kind,
verifies rejected writes and untouched bytes, then restores identity before remark.
The real generated-code consumer also passes optimized/UBSan AOT (200 cycles each,
100 framed) and framed JIT (100 cycles), including rejected-write root cleanup.

- `store-index-validation.json.gz`: native commands, outcomes and source hashes.
- `store-index-generated.json.gz`: generated JIT/AOT consumers against the new host.
- `store-index-costs.json.gz`: compiler commands, paired raw times and input hashes.
- `store-index-native-gc.c.gz`: exact generated collector for the cost runs.

Reproduce with `validate.py --sparse-index --output <fresh-directory>` and
`measure-stores.py --output <fresh-directory>` in this directory, followed by the
concurrent-store harness with its qualified Rust runner. Existing CLR/Go barrier
research and the mutation audit remain the design basis; this reuses the native
collector's detached index, not a new managed or metadata contract. Further work
must qualify all writers/hosts and measure managed standalone and HTTP workloads.

[Follow-up synchronous and overlapping-marker comparison](store-overlap-results.md)
shows the remaining whole-collector cost after lookup optimization.
