# Native HTTP worker threads — investigation, 2026-10-23

Status: the [bounded native prototype](../experiments/native-worker-pool/README.md)
now implements a serial managed-access pool with explicit capture/result rooting,
mailbox wakeups and joined teardown. Eight explicit temporary-root slots per job
now retain participant locals across gate release and owner collection. Primitive CPU work overlaps across workers.
A generated Raven harness now qualifies forced collection, fault recovery and TLS
reuse in independent per-invocation contexts on four workers. Shared generated-code
parallel-mutator integration and public threading APIs remain unimplemented.
The isolated HTTP host and native read/mutation checkpoints below are implemented
experiments; their ownership models do not establish shared generated concurrency.
A synchronous shared callback bridge now publishes the pool root set to existing
compiler collection hooks under the heap gate. Synchronous faults now belong to
individual jobs, with rooted diagnostics copied before ticket consumption;
a bounded owner completion pump delivers tickets outside the gate. Generated
continuation wakeups and async/resource ownership remain open.
The architecture below remains a proposal beyond that checkpoint.
The author explicitly requested worker-thread investigation inspired by modern
web application frameworks.

## Recommendation and comparison

Investigate a bounded runtime worker pool for managed application execution,
while socket waits remain asynchronous. Preserve Task/await and shared capture
identity. HTTP should dispatch through the runtime substrate; application handlers
should not need to wrap ordinary requests in Task.Run.

| System | Shipped model | Lesson for neoCLR |
| --- | --- | --- |
| ASP.NET Core | App code runs on pool threads; async I/O releases execution capacity while waiting. Guidance discourages redundant Task.Run around request code. HttpContext is not safe for arbitrary simultaneous access. | .NET-oriented target: concurrent requests without a thread per connection or implicit thread safety for all objects. |
| Tokio | Separates I/O driver, timers and scheduler; multithread runtime uses work stealing. Current-thread execution also exists. | Separate readiness from execution. Work stealing is an optimization after ownership correctness. |
| Node.js | Workers execute JavaScript in parallel, mainly for CPU-heavy work; built-in async I/O handles waiting efficiently. Memory transfer/sharing is explicit. | Isolated workers are useful, but cannot silently replace shared Task.Run captures. |

Primary sources consulted 2026-10-23:
[ASP.NET Core](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/best-practices?view=aspnetcore-10.0),
[Tokio](https://docs.rs/tokio/latest/tokio/runtime/index.html),
[Node.js](https://nodejs.org/api/worker_threads.html).
These are design precedents, not performance comparisons.

The [existing measurements](../../benchmarks/http-server/serve/investigation-20261023/README.md)
show about 495/s for one native server at 800/s offered, versus 800/s aggregate for
two isolated processes. Separate heaps/listeners and doubled aggregate lane/client
limits mean this does not predict shared-pool scaling. Per-request allocation and
root bookkeeping remain important with multiple workers.

## Concrete native prerequisites

| Layer | Current implementation | Required work |
| --- | --- | --- |
| GC and roots | Host roots, resources and stack roots are thread-local; arena headers/extent are mutated without a shared-mutator protocol. | Invocation-wide participant registry, allocation synchronization, collection coordination and safepoints. |
| Task/Promise | Ordinary result slot, flags, callback list and mutable queue. | Exactly-once settlement, release/acquire publication and synchronized queues; invoke user callbacks outside locks. |
| Async fibers | Suspended stacks and parked roots belong to an owner thread/context. | Pin stacks initially and route wakeups to their owner; do not copy/migrate ucontext storage. |
| Socket scheduler | Scope, handles, pending operations and delivery are owner-local. | Bounded cross-owner command/completion delivery, or a qualified scope per worker. |
| Rust VM | Shared task work roots captures/results; graph access and trusted task regions are guarded. | Reuse semantic tests, not implementation assumptions: native generated code lacks the VM instruction gate. |
| HTTP context | Owns request/response streams, pending writes, cancellation and disposal. | Preserve one logical response writer and lifetime through handler completion. Concurrent requests do not authorize concurrent mutation of one context. |

Sources: [GC](../../runtime/native/gc/native-gc.c),
[roots](../../runtime/native/gc/root-probe.c),
[fibers](../../runtime/native/async/native-runtime.c),
[sockets](../experiments/aot-console/socket-listener.c),
[Tasks](../../runtime/raven/src/System/Concurrency/Tasks/Tasks.rvn),
[VM shared heap](../../src/shared_heap.rs),
[VM task work](../../src/task_work.rs),
[VM atomic regions](../../src/vm_task_atomic.rs).

The [native ownership probe](../experiments/native-worker-ownership/README.md)
confirms that an owner-issued host-root handle cannot be read or released from a
worker, while independently owned worker roots survive collection and the parent
retains its payload after join. It avoids concurrent shared-arena mutation. This
is evidence of today's boundary, not a permanent thread-affinity requirement.

Simply passing a callback pointer to pthread_create is insufficient. The existing
Rust VM work is not a drop-in collector/scheduler for generated native code.

## Proposed first pool architecture

Use a fixed small set of long-lived OS workers, configurable by the experiment
(one/two/four), without selecting a public default. This simpler first design
trades load balance and blocking tolerance for clearer lifetime reasoning compared
with dynamic pools/work stealing.

1. Root callback/captures before publication. A queued ticket carries invocation
   identity and generation. Reject full/closing admission before ownership transfer.
2. Register each worker and install its execution/fault, stack-root and runtime
   context. Share only qualified heap/services; do not concurrently reuse a mutable
   neoclr_aot_context. Parked roots remain registered.
3. Root results before publishing completion; the receiver adopts them before
   producer ownership ends. Completion and cancellation have one winner. Use an
   explicit memory-ordering protocol.
4. Await parks execution instead of blocking a worker on a socket. External events
   signal the destination owner; keep existing fiber stacks pinned initially.
5. Shutdown closes admission, requests cooperative cancellation, services remaining
   completions, joins workers, then releases roots/heap. Do not join while holding
   a worker-required lock or from the joined worker itself.

Bound both submissions and completion storage. Reserve delivery capacity so full
submission queues cannot block cancellation/reclamation. Invoke user callbacks
outside internal locks. A global managed-execution lock can be a serial correctness
reference, but cannot qualify CPU-parallel execution.

Blocking native work needs a separately bounded offload facility or a tested
compensation policy. A small fixed pool can starve when callbacks block; bounding
it does not solve nested-wait deadlocks. Cancellation does not forcibly interrupt
arbitrary native/user code. Shutdown latency remains a documented limitation.

No new promises about affinity, start order or ambient context flow. Fiber
migration, preemption, dynamic sizing and work stealing remain later options.
Targets without native threads need an explicit capability/semantic boundary.

## HTTP integration alternatives

**Handler-only first experiment:** retain transport ownership, run handlers on
workers, and marshal stream commands/completion back to the owner. This isolates
the change, but parsing, framing and GC may remain the bottleneck. Moving only
the handler does not necessarily improve the plain-response workload.

**Full connection ownership later:** assign accepted connections to workers before
managed request creation, keeping parsing/handler/framing/I/O with that worker.
An unregistered native socket needs exactly-once ownership transfer and close on
failed admission. Shared application services still require the shared-heap work.
Compare central acceptance with per-worker acceptance after qualifying ownership.

**Independent heaps:** message/connection transfer can exploit isolation, but
in-process heaps still need static-state/service and cross-heap-pointer auditing.
It changes state-sharing semantics; it is not the selected .NET-like Task.Run
implementation. The previous two-process experiment only motivates this alternative.

## First bounded implementation slice

Build a private native worker-ownership harness before connecting HttpServer:

- Two registered participants with rooted submission/result handoff.
- Bounded mailbox/wakeup, full-queue handling, close-before-start, and
  completion-versus-cancellation races.
- Collection while another participant is running, parked or blocked outside
  managed access; retain captures and unpublished results.
- Shared identity and write visibility after completion, covering reference,
  value and union results.
- Joined teardown with no remaining roots/resources, including early failures.

Begin with explicit synchronization, then qualify generated-code safepoints and
actual overlapping CPU work. Compiler/root-descriptor changes require matching
Raven and neoCLR bridge documentation; ordinary .NET behavior remains unchanged.
This investigation makes no compiler change.

After those gates pass, connect a private Task.Run adapter preserving current
Result/cancellation/Fault semantics, then compare HTTP integration alternatives.
Do not expose a worker-count HTTP API before ownership behavior is established.

## Validation and measurement

| Workload | Question |
| --- | --- |
| Fixed CPU handler plus unrelated fast requests | Do cores execute work concurrently, and how much head-of-line delay remains? |
| Delayed peer and slow upload/download | Does waiting release worker capacity without one thread per wait? |
| Explicit blocking host call | Does the owner progress; what happens when the pool is saturated? |
| Mixed I/O/CPU with cancellation and overload | Are queues/memory bounded, wakeups reliable and completion fair? |
| Existing plain and bulk fixtures | Do framing, parsing and GC remain serial bottlenecks? |

Compare one/two/four workers with equal aggregate offered load, lane/connection
limits, response bytes, GC policy and total memory budget. Report allocations
(including native malloc when instrumented), CPU/RSS, worker utilization, queue
delay, completion rate, misses, p50/p95/p99 and shutdown time. Retain failed/drained
requests and reverse trial order. Profiling and timing stay separate.

Use deterministic barriers for race tests, ASan/UBSan for memory errors and TSan
where supported. Passing sanitizers is not proof of race freedom. A pool saturated
by long callbacks can still have poor latency; green threads alone do not supply
CPU parallelism. Existing rates are motivation, not thresholds for new workloads.

Open decisions: shared native heap coordinator and safepoints; handler-only versus
connection-owner distribution; blocking compensation; wakeup backend and supported
targets; whether a separate isolated-worker API is useful. This proposal does not
select preemption, a public green-thread model, or a permanent roadmap reordering.

### Working isolated HTTP integration and shared-GC gate — 2026-10-23

The source HTTP host now has an explicit 1–4 application-worker experiment sharing
one kernel listener, with private managed heaps and owner-local I/O/Task lifetimes.
[Integration evidence](../experiments/http-worker-integration/README.md) covers real
requests, pending reads, failure propagation and joined teardown. This deliberately
uses isolation instead of .NET-style shared state: useful concurrency now, at the
cost of replicated application instances, statics and per-owner heap/lane limits.
It is not a change to default Task.Run semantics or a new web application framework.

The author's subsequent direction makes shared-object collection the correctness
gate before throughput optimization. The expanded
[graph qualification](../experiments/native-worker-pool/README.md#shared-graph-gc-gate--2026-10-23)
checks cycles, aliases, mutation, live displaced children and reclamation across
four workers under the heap gate. Parallel mutators remain unqualified. Preliminary
isolated-owner scaling is retained separately; matched comparisons with other
servers and allocation work follow correctness, not replace it.

### Concurrent native reads — 2026-10-23

The private pool now admits overlapping native read phases over rooted capture
graphs, while object mutation, allocation and collection wait for readers to exit.
Pending writers prevent new read admission. This starts an explicit quiescence
protocol without exposing uncoordinated generated mutators. Unlike .NET's managed
thread suspension/safepoints, native callers must delimit read phases themselves;
this is a temporary qualification tool, not a managed semantic divergence. It adds
synchronization and can delay GC if a reader stalls. Root release/cancellation
remain responsive through the metadata mutex, and active job captures retain data.
See [contracts and evidence](../experiments/native-worker-pool/README.md#concurrent-native-read-phases--2026-10-23).

### Cooperative native mutation checkpoint — 2026-10-23

The private pool now supports explicit native mutation phases with per-job local
roots and cooperative polling. Workers may mutate disjoint fields or synchronize
conflicting fields with application locks; the collector waits until all mutators
have yielded. Native read phases and mutation phases are mutually exclusive. Eleven
optimized/UBSan groups include detached cyclic graph roots, cross-worker updates,
GC, cancellation and joined shutdown; generated shared callbacks remain exclusive.
See [qualification and constraints](../experiments/native-worker-pool/README.md#cooperative-native-mutation-phases--2026-10-23).

Relative to .NET's transparent managed GC coordination, this is a private native
implementation experiment requiring explicit discipline. It establishes overlapping
writes and root publication without pretending that the current compiler ABI is
thread-safe. A stalled native phase delays collection; phase entry/polling adds
synchronization but no per-phase allocation. Generated integration requires a
shared heap coordinator distinct from thread-local fault/context state, complete
participant frame snapshots and audited allocation/static/intern helpers. Do not
copy the existing embedded arena cursor across contexts or remove the generated
callback gate as a substitute for those changes.

### Invocation state separated under the existing gate — 2026-10-23

The compiled callback adapter now transfers the arena/static root to a fresh
invocation context for the duration of each exclusive call, returning both before
releasing the gate. Job faults come from invocation-local diagnostics. This models
the distinction between shared heap ownership and per-execution failure state
without changing ABI v4 or allowing concurrent cursor copies. Relative to .NET's
shared heap/per-thread execution model, it is still an explicitly serialized
adapter; its benefit is ownership/fault isolation, with 1,080 bytes of additional
bounded stack context and initialization cost. It is not a throughput optimization.

[Validation](../experiments/native-worker-pool/README.md#per-invocation-context-ownership--2026-10-23)
covers static continuity, collection, faults, root-admission failure/retry and
teardown. Compiler-frame publication and genuinely shared allocation state remain
necessary before removing the gate. No compiler ABI or Raven encoding changed.


Concurrent compiled integration follow-up (2026-10-23): an opt-in macOS ARM64
backend now publishes participant compiler frames and coordinates collection on
one shared heap. Optimized/UBSan fixtures reach four active compiled participants,
retain aliases/cycles and detached locals across 24,544 collections, isolate faults,
and reclaim after cancellation. Allocation/native helpers remain serialized;
shared async Task/HTTP ownership and throughput qualification remain open.
See [scope and evidence](../experiments/native-worker-pool/README.md#concurrent-compiled-shared-heap--2026-10-23).


Coordination-cost follow-up (2026-10-23): isolated HTTP again scales from roughly
417–443 requests/s (one owner) to 1,525–1,584 (four), with saturated closed-loop p99
latency falling from 90–105 ms to 31–32 ms. A separate fixed-work compiled test
finds high shared-heap scheduling overhead. Separating safepoint notifications
from job-completion waiters reduces four-worker mean elapsed time about 47% in
two local trials without changing allocation/collection counts. Native and compiled
GC stress regressions pass. The concurrent graph path remains slower than exclusive
execution; lower-cost safepoints and pacing precede shared async HTTP migration.
See [cost evidence](../experiments/native-worker-pool/README.md#coordination-cost-and-completion-wakeups--2026-10-23).
