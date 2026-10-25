# Concurrency, scheduling and GC architecture investigation

2026-10-25, author-directed extension of the standard benchmark/profile procedure.
Status: [initial owner profiles](initial-profiles.md) pass; [tuned owner and shared-pool profiles](pool-results.md) also pass. Wider scaling
and task/fiber attribution remain pending. Concurrent
GC's advantage under concurrency is a hypothesis, not an assumed acceptance result.

## Keep different kinds of concurrency separate

| Dimension | Current evidence and boundary |
| --- | --- |
| HTTP clients/connections | Many pending requests on one owner do not prove parallel managed execution |
| Independent owners | Existing AOT HTTP hosts run one/four OS workers with independent heaps; Task.Run stays owner-local |
| Shared heap | Compiled AOT pool admits coordinated execution; concurrent marking overlaps reviewed read phases and ends before exclusive mutation |
| Tasks/fibers | Cooperative native fibers are pinned to their owning thread; not a general migrating virtual-thread scheduler |
| Collector threads | Count separately from application workers; CPU competes with mutators |

Both native worker hosts currently cap application workers at four. Start with
supported one/four-owner configurations; add two and then higher counts only with
matching admission, resource, shutdown and stress qualification. Do not silently
substitute multiple processes for shared-heap workers or classify restricted read
phases as arbitrary concurrent mutation. JIT/shared-worker and suspended-task
capabilities require separate qualification; unsupported cells stay explicit.

## Required measurements

For each collector, workload, execution mode and topology, retain its own history
and compare throughput, total user/system CPU, CPU per checked operation, allocation
rate, retained/peak memory, collection counts and pause distributions. Count useful
application work that completes during collection, not just epochs started.

Scheduling profiles must distinguish:

- OS voluntary/involuntary context switches from task/fiber switches;
- runnable queue delay from blocked I/O and mutex/condition waits;
- enqueue/dequeue, resume, wakeup and fiber context-switch costs;
- root publication, safepoint rendezvous and exclusive execution lease waits;
- collector service dispatch, marker work, join, remark and sweep;
- per-worker completed work, CPU and starvation/tail latency.

The owner harness now records wait4 process context-switch counters and separate
user/system CPU, over the whole server lifetime. These include every server thread
and exclude the client/profiler process. Zero counters can reflect OS reporting
limits. macOS sample reports wall-stack observations, not precise scheduling delay
or CPU attribution; blocked stacks must not be reported as consumed CPU. Missing
metrics need instrumentation, not inferred values or zero placeholders.

## Initial profile cohorts

1. Complete the current three-round fixed-work HTTP matrix unchanged.
2. Run existing one/four-owner plain HTTP sync, guarded-sync and concurrent hosts
   under separate loaded stack profiles. Guarded-sync helps isolate runtime store
   and root overhead from marking. Preserve its older policy/fixture provenance.
3. Profile the tuned watermark/index-only concurrent hosts separately to avoid
   mistaking older configuration costs for current costs. Existing owner fixtures
   close connections; they are not the new persistent-driver benchmark.
4. Profile shared-heap compiled graph and allocation workloads, separating useful
   reads from exclusive mutation, allocation service and collection coordination.
   Short lifecycle tests need a sustained workload before stack-sample percentages
   can support architectural conclusions.
5. Add fixed-total-work and fixed-work-per-worker sweeps, then mixed I/O/CPU and
   suspended-task cases. Run uninstrumented timings separately. Record heap-per-owner
   versus total heap budgets and CPU oversubscription, including collector threads.

An architectural rethink is warranted if coordination serializes useful execution,
frequent allocation/lease transitions destroy overlap, or service/queue contention
outweighs reduced pauses. First locate and measure that cost; do not assume adding
threads, migration, work stealing or a different collector resolves it.

Existing evidence: [independent owners](../../execution/concurrent-http-owners/README.md),
[guarded CPU profiles](../../execution/concurrent-http-owners/cpu-results.md),
[shared-heap read epochs](../../execution/concurrent-participant-roots/read-epochs.md),
[worker design and .NET comparison](../../../docs/design/native-http-worker-threads.md).

The author clarified that virtual threads are not required: measure context-switch
and handoff cost, then decide whether tuning or redesign is justified.
