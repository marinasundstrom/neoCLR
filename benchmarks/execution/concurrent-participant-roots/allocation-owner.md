# Compiled worker allocation service

2026-10-25. `NEOCLR_WORKER_ALLOCATION_OWNER` connects generated native allocation
calls to a pool-owned service thread. It requires `NEOCLR_WORKER_GC` and
`NEOCLR_GC_ALLOCATION_DISPATCH`; all three remain opt-in. This is actual compiled
worker integration of allocation routing, **not concurrent shared-heap collection**.

## Ownership and lifecycle

The requesting compiled participant retains its existing exclusive execution
lease while waiting. Its arena pointer remains valid until the response arrives.
The service checks the participant, exact context arena and sole-running lease
under the pool gate, then invokes the checked, non-counting storage allocator.
Only the requester publishes its output and updates allocation/pacing statistics.
One embedded request slot is sufficient because the existing lease permits one
request at a time; it introduces no per-allocation native request object.

The service uses the pool's external-arena allocation contract. It does not acquire
an owned-heap free-list/cache scope that could become stale when another thread
collects. It neither traces roots nor collects, accesses foreign TLS resources,
invokes generated code, or installs a dispatch callback of its own. Synchronous
collection still runs on the existing exclusive compiled participant with direct
or copied roots. Rooted allocation pressure still follows that collector path;
forced allocation-pressure routing is not separately qualified here.

The service starts lazily on the first admitted compiled invocation. A failed
thread start destroys its condition variable and rejects the invocation before
participant registration. Each invocation installs the allocation adapter and
clears it on return, including managed faults. Pool close requests cancellation
and joins execution workers first, leaving allocation service available while
those workers drain. It then stops/joins the service and destroys its private
condition variable before pool destruction. OS thread/condition creation failure
injection is not covered by the tests.

The service has its own condition variable. Only the requester and allocation
service wait there; allocator traffic does not broadcast to every compiled-lease
waiter. Managed execution retains its existing cooperative scheduling contract.

This continues the [allocation-service comparison and tradeoffs](../concurrent-worker-epoch/README.md#owner-serviced-allocation-requests).
It does not prescribe a central allocation thread as a permanent departure from
.NET. The concrete benefit is explicit allocator/worker ownership while integrating
the collector; the measured synchronization cost below is substantial. No managed
API, Raven emission or Runtime Contract changes are involved.

## Qualification

`allocation-owner-validation.json.gz` rebuilds the managed sample object and runs
four configurations: optimized/UB-sanitized, each with direct/copied participant
roots. Each executable passes graph/GC, fault isolation, cancellation, capture
release, result lifetime and cleanup. It serves 3,904 graph and 128 fault allocation
requests, with the existing 24,544 graph and 448 fault synchronous collections.
A new shutdown scenario closes the pool with four invocations running, drains 488
allocation requests, releases results/capture and repeats close safely. The
control matrix (`allocation-control-validation.json.gz`) passes the same cases
without the service option. Removing adapter installation makes the request-count
assertion fail; the deliberate failure is retained in the owner report.

These tests qualify 4,520 generated allocation requests per executable. They do
not claim concurrent marker epochs, JIT integration, HTTP throughput, memory
pressure behavior or a complete scheduler/resource-root protocol. Existing native
live-epoch tests supply separate marker evidence; that is not interchangeable
with this compiled synchronous-pool evidence. TSan remains an open gap.

## Diagnostic cost

All measurements use the same byte-identical managed object, ordinary optimized
copied-root builds, one warmup and five measured executions per variant. Each
execution runs 68 managed invocations including fault and shutdown cases. CPU is
whole-process user+system time (all threads); elapsed time includes process startup.
This short correctness workload is not a calibrated application benchmark.

Initial separate runs had materially different control timings. Preserve those
reports, but do not use them to infer the effect of the dedicated condition.
`allocation-three-way.json.gz` interleaves all three executables in rotating order:

| Variant | Median elapsed | Median process CPU |
| --- | ---: | ---: |
| Local allocation control | 60.0 ms | 93.4 ms |
| Allocation service, dedicated wakeup | 122.4 ms | 163.5 ms |
| Allocation service, general broadcast | 173.8 ms | 349.1 ms |

Dedicated wakeups use about 53% less CPU and 30% less elapsed time than broadcasting
in this run. Even so, the service uses about 75% more CPU and twice the elapsed
time of local allocation. This is integration scaffolding with an observed cost,
not a performance improvement over the existing allocator or justification for a
default change. Results and source/executable hashes are retained. The initial
broadcast sources are archived separately; the default remains local allocation.

## Stable-context follow-up

The [collection-owner option](collection-owner.md) now executes copied-root
synchronous collection on the service's stable context, with matching compiled
controls and deliberate lost-root failures. It has substantial per-collection
handoff overhead and does not yet retain a live epoch across worker execution.

## Next boundary

Connect collector ownership and final remark to the compiled scheduler without
moving live TLS state. The allocator service currently receives each participant's
external arena; a persistent concurrent epoch instead needs a stable collector
context and complete initial/current roots across participant lifetimes. Collection
requests, new-object publication, provider writes, faults, cancellation and heap
exhaustion must follow that protocol. Per-allocation handoff is already costly;
consider allocation batching or worker-local allocation regions only with explicit
header/publication and accounting rules, then measure the whole workload again.

```sh
python3 benchmarks/execution/concurrent-participant-roots/validate-pool.py \
  --allocation-owner --output target/owner-repeat
python3 benchmarks/execution/concurrent-participant-roots/validate-pool.py \
  --output target/control-repeat
python3 benchmarks/execution/concurrent-participant-roots/measure-owner.py \
  --control target/control-repeat --owner target/owner-repeat \
  --output target/owner-cost-repeat.json
```

Use fresh destinations. The validator records source hashes and build/run
commands; the measure driver requires passing manifests and matching managed
objects. No website build or publication is needed for these private native changes.
