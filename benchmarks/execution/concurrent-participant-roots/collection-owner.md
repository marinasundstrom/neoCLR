# Stable collector context for compiled workers

2026-10-25. `NEOCLR_WORKER_COLLECTION_OWNER` routes copied participant roots to the
pool's persistent service thread. It requires the allocation-owner and root-snapshot
options. This establishes a stable collector context for generated worker calls;
collections still run synchronously with all participants paused. It does not
remove the live shared-heap admission rejection or enable concurrent HTTP GC.

## Protocol and boundaries

The requesting participant holds the exclusive lease. It captures complete root
values using the existing validated helper, then publishes that buffer and its
current arena to the service under the pool gate. The request buffer remains valid
until acknowledgement. The service copies the arena into its stable context,
builds a lane frame belonging to that context and calls the native conditional
collector. It copies the resulting arena extent back before signalling completion.
The requester clears service pointers before freeing its root buffer.

No source worker frame or descriptor is reinterpreted as belonging to the service.
Fault/static roots are already included in the value capture. Existing restrictions
on worker TLS resources and host roots remain essential: this does not enumerate
foreign thread-local resources. No managed execution or cleanup callback is moved
to the service by this protocol. Allocation and collection share one bounded request
slot and dedicated condition variable; only one exclusive requester can submit.

Service collection diagnostics/statistics belong to the service thread; pool
coordinated collection counts remain the requester-visible completion count.
Allocation accounting remains on requesters. This is still the forced-collection
worker profile, not qualification of paced publications across migrating contexts.
Host-side pool collection and final cleanup continue using their existing paths;
they must be coordinated with any future retained live epoch. The external arena
is not acquired into a thread-owned allocator cache.

The [allocation-service research comparison](allocation-owner.md) still applies:
stable internal ownership is useful, but centralizing individual operations has
synchronization costs. This is a provisional implementation step, not a new managed
GC contract or a claim to reproduce CLR collector internals. Compiler emission,
Raven bridge and managed APIs are unchanged.

## Validation and observed cost

`collection-owner-validation.json.gz` rebuilds the managed sample and passes
optimized and UB-sanitized executions with copied roots. Each reports exactly
24,544 graph and 448 fault collections on the service, equal to the pool's completed
counts. Graph/fault invocations, cancellation, captures, results, close during four
running invocations and repeated close all pass. Removing copied root lanes causes
the expected managed-invocation assertion to fail. Disabling allocation dispatch
also fails its request-count assertion. Allocation-only direct/copied-root controls
are rebuilt and pass in `collection-allocation-control.json.gz`.

`collection-owner-cost.json.gz` compares ordinary copied-root executables with an
identical managed object, alternating order, one warmup and five measured runs:

| Variant | Median elapsed | Median process CPU |
| --- | ---: | ---: |
| Allocation service, requester collects | 124.3 ms | 168.9 ms |
| Allocation and collection service | 503.4 ms | 475.0 ms |

This forced-collection stress fixture performs a handoff for every coordinated
collection. The resulting roughly 4.0x elapsed/2.8x CPU cost is evidence against
using that request frequency for throughput. It is not an HTTP or ordinary paced-GC
result, and it establishes no concurrent performance gain. TSan, JIT worker paths,
forced allocation exhaustion and complete resource/finalizer lifetimes remain open.

## Scheduling follow-up

[Budgeted collection](paced-collection.md) now separates exclusive mutation
permission from forced collection frequency, retaining explicit/pressure paths and
the forced mode as a control. Its improvement is measured against this stress
fixture; live epochs and broad HTTP/memory/pause acceptance are still open.

## Next integration boundary

A stable service context is now available for a live epoch, but worker mutation
must not run beside an unguarded marker. Investigate starting an epoch as the pool
leaves an exclusive phase, overlapping it with admitted read-only execution, and
finishing with current copied roots before the next exclusive mutation. This needs
an audit of the compiler's operation classification, scheduler wakeups, allocation,
explicit collection, host pool operations and shutdown. It is a proposed next
integration, not an admitted mode. Wider overlap with managed mutations still
requires complete registered-barrier/provider integration.

```sh
python3 benchmarks/execution/concurrent-participant-roots/validate-pool.py \
  --collection-owner --output target/collection-owner-repeat
```

Use a fresh directory. The driver records commands, inputs and deliberate failures.
