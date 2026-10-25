# Remove the allocation-service round trip — 2026-10-25

`NEOCLR_WORKER_LOCAL_ALLOCATION` is an opt-in native shared-pool configuration.
It requires the existing collection-owner contract. The requesting participant
already holds the sole exclusive execution lease and now calls the checked storage
allocator directly under the pool gate. The separate service still owns collection.
No virtual threads, task migration, shared mutation admission or default GC change.

## Ownership and accounting

The participant must be the current exclusive owner, the only running participant,
and attached to the exact context arena. The collection service must be started,
not stopped and have no request pending. In read-epoch mode allocation additionally
rejects any active epoch: existing exclusive-boundary logic must finish it first.

This pool uses external arenas, without thread-owned allocation cache authority.
The storage allocator does not collect, invoke managed code or recurse through the
allocation dispatch hook. Rooted pressure still collects/retries through the existing
owner protocol. The requester publishes output only on success; allocation attempt,
successful count/byte pacing and per-thread GC statistics preserve their existing
owners and meanings. Collection roots, faults, cancellation and cleanup are unchanged.

The .NET comparison is the existing [allocation ownership research](../../execution/concurrent-participant-roots/allocation-owner.md):
central dispatch was an integration choice, not a required managed allocation
semantic. This step removes a cross-thread rendezvous while retaining serial heap
access. It does not yet provide independent per-worker allocation regions or full
concurrent mutators. The requester pays storage allocation CPU and still holds the
exclusive lease; contention at execution boundaries remains a cost.

## Validation

Sync and concurrent graph/fault/shutdown consumers pass optimized and UB/bounds
sanitized builds. Deliberately removing allocation dispatch and copied collection
roots still aborts both configurations. Separate sync/live pressure checks pass
optimized/sanitized runs: 32 explicit and 32 pressure collections, 96 allocator
attempts, output/liveness and cleanup assertions. Generated graph objects match
all comparison builds byte-for-byte. No TSan or arbitrary shared-provider admission
is claimed. Source changes are native runtime only; Raven emission and Runtime
Contract remain unchanged.

## Uninstrumented comparison

Fresh service controls were rebuilt from current sources after an initial retained-
control screen. Seven interleaved rounds per configuration, first discarded as
warmup. Each invocation includes the unchanged graph, fault and shutdown fixture;
this is a short four-worker lifecycle workload, not HTTP capacity.

| Collector / allocation | Median elapsed ms | Median process CPU ms |
| --- | ---: | ---: |
| Sync / service | 121.48 | 181.28 |
| Sync / local | 29.54 | 44.15 |
| Concurrent / service | 115.59 | 140.86 |
| Concurrent / local | 34.95 | 54.88 |

Concurrent elapsed falls 69.8% and CPU 61.0%; sync falls about 75.7% in both metrics.
Variation is material: concurrent control CPU ranges 108.85–279.22 ms versus
43.57–70.76 ms locally. Thus this establishes a substantial same-collector gain,
not a stable cross-collector ranking or concurrent-default acceptance.
`local-cost.json.gz` is the matched-source cohort; `local-cost-retained-control.json.gz`
preserves the preliminary screen and is not the final comparison.

## Separate wait diagnostics

Six instrumented candidate runs preserve the same 4,520 allocation requests per
invocation while allocation-service waits become **zero**. Median execution-boundary
wait calls fall to 13,293 sync / 14,655 concurrent, versus the earlier instrumented
33,191 / 36,251. Collection handoffs remain 22 / 44. These scheduler-dependent counts
are diagnostics, not deterministic invariants. Clock/atomic instrumentation adds
substantial overhead; compare uninstrumented times above, never profile durations.

## Reproduction and next work

Use `validate-pool.py --local-allocation` with/without `--read-epochs`, and
`validate-pressure.py --local-allocation` with/without `--read-epochs`, under
`benchmarks/execution/concurrent-participant-roots/`. Retain service controls with
`--collection-owner --paced` or `--read-epochs` without the new flag.
`measure-local.py` names the retained builds used here; `profile-pool.py
--local-allocation` measures waits separately. All commands and hashes are archived
in the `local-*-validation.json.gz`, cost and wait reports beside this document.

Next investigate remaining exclusive-boundary transitions, service collection
handoffs and task/fiber switching. HTTP owners currently have independent heaps
and do not use this allocation path, so no HTTP improvement is claimed. Do not
claim the shared-heap concurrent architecture is ready for general use from this
one successful scheduling change.
