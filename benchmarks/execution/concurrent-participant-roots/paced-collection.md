# Separating worker mutation leases from collection frequency

2026-10-25. `NEOCLR_WORKER_PACED` makes collection scheduling independent of
exclusive mutation permission. It requires allocation-owner accounting and remains
opt-in; the unpaced compiled pool retains forced collection at every eligible
exclusive boundary as a safety stress control.

## Scheduling contract

At an exclusive boundary with an admitted invocation handle, request collection
when any one budget is exhausted since the last successful compiled collection:

- 256 successful dispatched allocations across the pool;
- requested payload bytes totaling one eighth of arena capacity;
- 4,096 eligible exclusive boundaries without a prior budget trigger.

The byte budget is an initial heuristic, not a measured optimum or a hard memory
limit. It counts requested payload only, not allocation headers/padding. Native
pool allocations outside the dispatched generated path are not included. Failure
does not count as a successful allocation or reset collection budgets. Successful
compiled collection resets all three observations; unrelated host collections may
leave a conservative extra future collection due. There is no wall-clock cleanup
guarantee. Pool service counters stay under its existing mutex; no atomic or
per-allocation request-object allocation is introduced.

Explicit collection and rooted allocation pressure still use `compiled_collect`
directly and bypass the opportunistic budget. The arena-only allocation entry is
still non-collecting; this option does not make unrooted helpers safe to collect
or guarantee that every allocation can succeed. Exclusive mutation leases,
participant rendezvous, copied/direct root validation and synchronous sweep remain
unchanged. The same option works with requester-side or service-side collection.

This reuses neoCLR's existing allocation/publication pacing principle rather than
using lock ownership as an allocation policy. As with the earlier .NET/service
comparison, internal trigger policy is provisional and not a managed API contract.
Delayed collection can retain more memory; this is not a claim of lower live heap
or fewer application allocations.

## Evidence

`paced-owner-validation.json.gz` passes optimized and UB-sanitized service-owned
compiled graph/fault/shutdown tests. The graph runs 20 collections instead of
59,456; the fault-only cohort needs none instead of 768. Fault/result/capture
cleanup still passes through the existing host collector. Both modes serve the
same 3,904 graph, 128 fault and 488 close-during-execution allocation requests.
The first harness incorrectly required positive compiled collection count even
for the tiny fault workload. `paced-initial.json.gz` preserves that rejection;
the harness now requires graph pacing to execute without reverting to forced GC,
while allowing zero fault-workload collections and still checking complete cleanup.

`paced-local-validation.json.gz` covers direct/copied-root requester-side collection
in optimized/sanitized builds. `paced-forced-validation.json.gz` retains the
service-owned forced control. Deliberate missing dispatch/root controls run with
forced collection so they test those protocols independently of scheduling budgets.

`pressure-validation.json.gz` adds a native adapter running in the real compiled
worker pool. Ordinary/UB-bounds builds each perform 32 explicit collections and
32 rooted pressure collections, observed through the installed collection hook.
A 2-KiB arena forces a failed 1,500-byte allocation followed by collection and a
successful retry; 96 allocation attempts and final cleanup are checked. Budget
collections at invocation exit may additionally occur. This validates the native
rooted helper/service path, not arbitrary generated provider admission. The first
fixture used a byte allocation as a callback capture and was correctly rejected;
the final fixture uses the existing 24-byte callback-object representation.

## Bounded performance comparison

`paced-cost.json.gz` compares byte-identical managed objects with the same ordinary
copied-root collection-owner configuration, one warmup and five measured runs in
alternating order. Each process runs 68 invocations including faults and shutdown.
The summary now parses per-workload collection counts instead of hard-coding the
old forced-collection count.

| Variant | Median elapsed | Median process CPU |
| --- | ---: | ---: |
| Forced collection | 1,065.9 ms | 1,012.9 ms |
| Budgeted collection | 79.1 ms | 82.2 ms |

This removes about 93% elapsed time and 92% CPU from the forced-collection stress
fixture. It does not establish ordinary HTTP throughput, optimum budgets, pause
distributions, peak retained heap or a concurrent-default advantage. Allocation
routing still has per-request synchronization; application allocation reduction is
not claimed. The synchronous forced and paced implementations remain comparisons
for the next live-epoch integration.

```sh
python3 benchmarks/execution/concurrent-participant-roots/validate-pool.py \
  --collection-owner --paced --output target/paced-owner-repeat
python3 benchmarks/execution/concurrent-participant-roots/validate-pressure.py \
  --output target/paced-pressure-repeat
```

Choose fresh destinations. Next coordinate epoch start/current roots/finish with
the explicit read-phase contract, preserving collection-before-mutation and all
host cleanup paths. Broader JIT/AOT, standalone/HTTP, CPU/memory/pause and TSan
acceptance remains open.
