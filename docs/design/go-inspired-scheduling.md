# Go-inspired concurrency and scheduling investigation

**Author direction, 2026-10-28. Status: investigation proposal, not an adopted runtime architecture.**

Explore lightweight execution contexts, worker-local and global runnable queues,
work stealing, configurable parallelism and suspension that releases workers. Adopt
pieces only when they improve measured efficiency, scalability or useful runtime
capabilities without disproportionate complexity. This becomes the next bounded
investigation after the interface-view optimization, not a commitment to copy Go.

## Execution roles and shared runtime direction

**Author clarification, 2026-10-28:** JIT is the development execution path;
Native AOT is the publish/distribution target. Both use GC and scheduling services,
with concurrent GC the preferred direction. The author expects the main lazy-JIT
and concurrent-GC gains to emerge from scheduler/work-queue optimization. This is
a performance hypothesis to test, not an established gain or a collector-default
change.

Treat scheduling and collection as shared runtime investments. Keep AOT code
generation/packaging optimization and JIT cold-compilation costs distinct from
common steady-state execution costs. Representative published HTTP/load tests
must include Native AOT with sync and concurrent GC, alongside development JIT
startup and warm-run controls. Use matching source, resource budgets and current
codegen settings; improvements should be attributed to the layer actually changed.

After the shared-tracing commit, the next bounded slice compares queue policies
and scheduling latency against the existing FIFO baseline. Use the current
recorders and add provider hooks where measurement requires them. Tiny native-job
results alone cannot select the shared scheduler or establish HTTP/GC gains.
Compiler wait/publication tracing remains open; lazy JIT's synchronous first-call
wait is a separate blocking limitation that queue policy alone cannot fix.

## Existing foundation and gaps

| Area | Current evidence | Constraint for the proposal |
| --- | --- | --- |
| Native async contexts | `runtime/native/async/native-runtime.c` parks roots and uses owner-thread fibers; contexts/stacks can be reused | Scheduler state, stack bounds, roots and operation ownership are thread-local. Existing suspended machine contexts cannot simply be copied or stolen. |
| Native worker pool | `runtime/native/workers/native-workers.c` admits bounded rooted jobs, scans queued slots in submission order under a shared gate, and joins workers on teardown | Current pool supports 1-4 workers and at most 16 job slots. It has no worker-local ready queues or work stealing. Arbitrary callback blocking can occupy a worker. |
| Shared compiled execution | Private `NEOCLR_WORKER_GC` and compiler worker safepoints support parallel pure regions, coordinated collection, exclusive helper boundaries and job-local faults | Tested subset excludes parked async roots, TLS services and unaudited helpers. Earlier shared-heap graph tests scale poorly despite coordination improvements. This is a useful prototype, not a general Task/HTTP backend. |
| Ordinary concurrent collector | Standard HTTP/JIT path uses an owned heap with a background marker and owner reclamation | Multiple managed mutators require explicit additional coordination. A background GC thread is not parallel managed execution. |
| Lazy JIT | A compiler worker owns compiler state; the requesting managed thread waits for a result through `recv()` | Queueing compilation on another thread has not made the caller nonblocking. Suspending that caller must publish arguments/roots, preserve faults, and resume exactly once. |
| HTTP parallelism | Isolated HTTP workers/processes and owner-local async I/O already have evidence | Separate heaps do not implement shared Task.Run capture identity. Keep them as controls, not proof of shared-heap scheduling. |

See [existing scheduling contracts](../runtime-scheduling-design.md),
[concurrency direction](../concurrency-direction.md), and the
[worker experiment's later shared-heap checkpoints](../experiments/native-worker-pool/README.md#concurrent-compiled-shared-heap--2026-10-23).
Earlier statements that no shared compiled execution exists are historical; the
remaining gap is broad async/service integration and useful scaling.

## Runtime comparisons

Go separates goroutines, OS threads and execution resources. Its scheduler has
local/global runnable work, stealing and parking/wakeup coordination. This supports
separating runnable execution from OS-thread identity; it does not establish that
NeoCLR's current fibers or GC roots can migrate. Sources: [Go runtime structure](https://go.dev/src/runtime/HACKING)
and [scheduler implementation](https://go.dev/src/runtime/proc.go), reviewed 28 October 2026.

Go 1.25's container-aware parallelism direction also shows why host logical CPU
count alone is an insufficient automatic policy. NeoCLR should report effective
affinity/quota and account for compiler/GC CPU demand when evaluating defaults.
An explicit process override is required; setting names and quota-rounding rules
remain provisional. Source: [container-aware GOMAXPROCS](https://go.dev/blog/container-aware-gomaxprocs).

.NET remains the API/behavior comparison: a Task is an eventual outcome, not a
thread or necessarily a runnable context. CoreCLR's thread-pool implementation
already combines local work-stealing queues with shared queues; local queues are
not unique to Go. Preserve Task.Run shared captures, async unwrapping, exceptions,
cancellation and awaited completion semantics while changing execution machinery.
ExecutionContext/AsyncLocal flow and affinity must be specified separately rather
than assumed. Sources: [.NET work queues](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Private.CoreLib/src/System/Threading/ThreadPoolWorkQueue.cs),
[thread-pool configuration](https://learn.microsoft.com/en-us/dotnet/core/runtime-config/threading),
and [existing NeoCLR Task.Run contract](../concurrency-direction.md#taskrun-with-shared-captures--author-direction-2026-09-27).

## Proposed architecture boundaries

Separate the completion object (Task/Promise), the resumable execution context,
and a runnable scheduler record. Both compiler-generated await continuations and
future implicit suspension should feed the same runnable-record contract. Do not
require a new language keyword, expose queue internals, or treat every Task as a
new fiber. The previously selected public Scheduler abstraction remains distinct
from its still-open API shape.

Use an explicit context state machine: runnable, running, parking, parked,
runnable-again, terminal. Completion/cancellation races settle exactly once;
a completion arriving during parking must become a durable wakeup. The context
owns fault state, root registration, stack/continuation storage, cancellation,
logical destination and its pending operation. Queue publication acquires runnable
ownership before pending/active ownership is released. A worker parks only after
rechecking runnable work and wake state under the coordination protocol.

Start with pinned stackful contexts and steal only eligible unstarted work or
explicitly migratable logical continuations. Distinguish stealing runnable work
from migrating a live native stack. Reusing the present `ucontext` or Windows
fiber by moving its bytes is forbidden. Migrating suspended contexts is a separate
gate covering TLS, byrefs, stack bounds, GC root chains, exceptions, native frames,
I/O handles and thread-affine resources. No broad shared-payload isolation or
thread safety follows from GC coordination.

Use worker-local queues plus a global injection queue. Begin with auditable
bounded locking and explicit publication rules; compare against the current FIFO
pool. Measure local hits, successful/failed steals, global admissions, oldest
ready age and worker idle time. Add bounded global polling and aged-work service
to prevent local reuse from starving external completions. Lock-free deques,
async preemption, stack copying and automatic worker growth are later alternatives,
not prerequisites or assumed improvements.

Suspending on async I/O or compilation should leave the worker available for
other runnable work. I/O completion enqueues to the permitted destination, never
runs arbitrary managed code on the completion thread. Keep blocking native calls
on an audited boundary or bounded blocking executor. Classify pinned/noncooperative
calls explicitly; the first cooperative prototype cannot promise preemption or
nonblocking behavior for arbitrary native code. Cancellation still requires
backend acknowledgement before releasing buffers and registrations.

GC and JIT are part of admission, not afterthoughts. Collection must see queued,
active, parked and compiler-waiting roots, and coordinate all relevant mutators.
Compare sync and concurrent collection using the same work and memory budgets,
while retaining their independently tuned policies. Lazy compilation must have
once-only publication, reentrancy/recursion rules, failure fan-out, shutdown and
code-lifetime ownership. Do not compile unused methods merely to hide contention.

## Bounded sequence and adoption gates

0. **Establish execution visibility.** Implement the bounded opt-in tracer described
   below against the current runtime before interpreting queue-policy changes.
   Start with worker lifecycle/queue transitions, then join JIT, async and GC
   events as their hooks are qualified. Preserve untraced timing controls.
1. **Establish runnable-work measurements.** Refresh the actual FIFO worker pool
   baseline and add deterministic short/long/skewed jobs, admission-to-first-run
   latency, queue age and per-worker work counts. Compare a separately selectable
   local/global/stealing prototype at 1/2/4 workers first; expand limits deliberately
   before 8-worker tests. Native-only jobs isolate scheduling policy costs. They
   do not measure fiber switching, managed throughput or GC behavior.
2. **Qualify suspension on one worker.** Reuse the native async carrier. Test yield,
   timer/socket wait and compiler wait while an independent ready task progresses.
   Record roots-before-park, completion-during-park, cancellation, faults and joined
   teardown. Measure actual suspend/resume CPU and latency separately from queue
   dispatch. Preserve an owner-pinned control.
3. **Integrate multiple managed workers.** Build on the existing compiled shared-heap
   coordinator. Admit specific services and roots; initially pin contexts. Validate
   shared captures, displaced references during collection, fault isolation and
   cancellation under sync GC before qualifying the concurrent collector. Keep
   isolated-heap workers as a separate control.
4. **Measure representative applications.** Run the matrix below before selecting
   worker defaults, migration or preemption policy. Adopt one proven component at
   a time; retain reference modes and report losing cases.

The [bounded queue-policy prototype](../experiments/queue-policy/README.md) now
passes ordering, cancellation/reuse and existing worker qualification. Two short
screens are variable; longer two-worker controls show about 1% lower throughput.
Keep FIFO as the default. The prototype retains the shared pool/heap gate and
establishes no application or collector advantage.

The next bounded investigation is real I/O completion-to-resume latency and
coordination costs, retaining the owner-pinned control and shared async/GC trace.
Predeclare workload, repetitions, acceptable tail/error budget and host constraints
before evaluating a candidate. Queue ordering alone cannot remove first-call JIT
blocking or qualify shared managed execution.

**Author clarification, 2026-10-28:** General-purpose includes future desktop and
mobile UI applications, as well as HTTP and other communications. RabbitMQ
bindings should be feasible and competitive; no binding or performance parity is
claimed. Include low-load input/continuation responsiveness, owner/main-thread
affinity, frame deadlines, idle wakeups, CPU/energy and memory alongside saturation
throughput, backpressure and tail latency. Mobile and desktop need actual target
measurements; a busy desktop worker microbenchmark cannot qualify them. Follow
.NET-facing scheduling/context contracts; do not infer unrestricted continuation
migration from an internal queue policy. Compare broker clients only with matched
protocol, payload, acknowledgement/durability, prefetch and resource constraints.
These are workload criteria, not a commitment to a UI framework or broker API.

The author also requests optional purpose-specific JIT, GC and scheduling tuning,
including high request loads, while accounting for the executed logic. Aim for
balanced defaults and explicit, documented controls rather than a universal server
preset. Evaluate CPU work, allocation/live-set size, downstream waits, blocking
calls, available cores and admission limits together; report throughput, tails,
memory and idle/energy tradeoffs. Existing controls remain as documented; this
requirement does not announce new knobs or select values. Benchmark candidate
settings against representative handler logic before adding presets.

## Execution tracer: proposed first slice

**Author follow-up, 2026-10-28:** Suggested implementing an execution tracer to
provide an overview of execution. The assistant proposes making bounded runtime
tracing the first implementation slice. The first [bounded worker recorder and offline export](../experiments/execution-trace/README.md)
are now implemented in the private native pool. The broader correlated runtime
timeline below remains intended behavior. A separate [owner-thread async recorder](../experiments/execution-trace/async-README.md)
now captures suspension/resumption, nested execution and abandonment with sync and
concurrent GC qualification. The [shared native session](../experiments/execution-trace/session-README.md) now
correlates async transitions, owner collection requests and published concurrent
epoch lifetime. Compiler/worker registration and fine-grained GC phases remain
open; epoch duration is not a pause or marker CPU measurement.

The central question is why ready application work is not progressing. Existing
startup timings, compiler-wait diagnostics, aggregate GC measurements and CPU
samples are useful but do not provide a correlated context-state timeline. Keep
CPU profiling alongside tracing: a context marked running can still be descheduled
by the OS. Runtime events alone cannot establish actual CPU occupancy, kernel
blocking, throttling or instruction-level cost.

| Timeline / identity | Initial events or intervals | Question answered |
| --- | --- | --- |
| Worker and OS thread | Start/stop, dispatch, idle enter/exit, audited blocking boundaries | Are workers available while runnable work waits? |
| Runnable context / job generation | Admit, enqueue, dequeue, run, park reason, ready, complete/cancel/fault | Is time spent executing, waiting for service, or waiting for a dependency? |
| Queue | Depth, oldest-ready age, local/global source, steal attempt/outcome when implemented | Does placement or fairness delay work? |
| I/O / timer operation | Submit, completion observed, runnable publication, resume; stable operation ID | Is latency in the operation, completion delivery or subsequent scheduling? |
| JIT method / request | Request, compiler dequeue, lowering/codegen, publication/failure, caller wait/resume | Does compilation delay the caller, compiler queue or unrelated work? |
| Heap / GC cycle | Pause request, mutator acknowledgement, roots, mark, remark/reclaim as applicable, pause end | Which phases stop application progress and which overlap it? |
| Application work | Optional bounded app/route IDs and request correlation | Which applications or handlers suffer under shared load? |

Logical context IDs must survive suspension and differ from worker/thread IDs.
Use session-scoped IDs plus generations where slots are reused; connect requests,
operations, compiler requests and GC cycles explicitly. Record transition reasons
and event sequence numbers. Use one process-wide monotonic clock shared by Rust
and C hooks, declaring units and clock origin in the file. Cross-process clocks
require synchronization metadata before merging; equal wall-clock labels alone
are insufficient. Never infer an exact causal ordering solely from timestamp ties.

Begin with preallocated bounded per-producer buffers and fixed-size native event
records. The disabled path must avoid clock reads, formatting and allocation.
Enabled recording must not allocate managed objects, retain managed references,
perform file I/O or acquire a global logging lock at GC/scheduler transition sites.
Define producer registration, nested/reentrant events, buffer lifetime and shutdown
before enabling hooks on compiler, I/O or collector threads. Full buffers drop
records with explicit per-producer loss counts and sequence gaps; they must not
block application progress. Export after producers are quiescent in the first
slice. Crash capture, live draining and a continuous flight recorder are later work.

Record numeric identities and bounded static labels initially, not request bodies,
headers, arbitrary URLs or object addresses. Method/route dictionaries are optional
and resolved outside hot paths. Version the schema and include build/input hashes,
JIT/GC modes, configured workers, effective CPU budget, capture bounds and loss
counts. Partial capture must report unknown/incomplete intervals rather than
inventing state durations or treating missing completions as deadlock.

Use an offline Chrome trace JSON export for an initial Perfetto overview: separate
worker/context, compiler and collector tracks, causal flows, and queue/heap counters.
Do not draw a suspended context as occupying a worker, or a background GC span as
a stop-the-world pause. Keep overlapping logical operations on appropriate tracks
rather than emitting invalid nested thread spans. Preserve raw records for analysis;
export format is not the runtime storage format. A custom viewer and a public
managed tracing API are not prerequisites.

Validation must exercise deterministic enqueue/run/park/wake order, reused IDs,
completion-during-park, cancellation, thread exit, overflow and truncated exports.
Check exactly-once work outcomes independently of trace completeness. Compare
tracing disabled/enabled on tiny jobs, mixed waits and allocation pressure, then
cold/warm HTTP under sync and concurrent GC. Publish event rate, lost records,
buffer memory, CPU cost and throughput/tail changes; no low-overhead claim before
measurement. Keep performance conclusions grounded in untraced interleaved runs.
A first useful artifact is a captured current-pool timeline plus a checked summary
of queue wait and worker activity; deeper GC/JIT/I/O coverage follows incrementally.

Research comparison, reviewed 28 October 2026:

- [Go execution tracing](https://pkg.go.dev/runtime/trace) captures scheduler,
  blocking and GC activity and supports task/region annotations. This motivates
  correlating runtime state with application work; it does not prescribe NeoCLR's
  storage format or make the current fibers migratable.
- [.NET EventPipe](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/eventpipe)
  aggregates runtime events, including JIT and GC, with application events. Use
  that as the diagnostics comparison while retaining .NET Task semantics. A small
  NeoCLR recorder is not EventPipe/EventSource API or tooling compatibility.
- [Perfetto's supported formats](https://perfetto.dev/docs/getting-started/other-formats)
  include Chrome JSON slices, counters and flows. Reusing this viewer avoids an
  initial UI project; native TrackEvent export remains an alternative if JSON
  volume or correlation limits justify it.

## Required comparison matrix

| Workload | Pressure / controls | Measurements |
| --- | --- | --- |
| Runnable-only | Tiny jobs, long jobs, skewed queues, nested submissions; 1/2/4 then 8 workers | Median throughput, CPU/job, p50/p95/p99/max ready latency, oldest queue age, steals, worker utilization |
| Suspension | Yield ping-pong; timers; real socket completions; completion-before/during/after park | Switch CPU cost, wall latency, wakeups, parked memory, exactly-once outcomes |
| Mixed work | CPU hogs alongside latency-sensitive I/O; bounded cooperative polls; deliberate blocking native calls | I/O tails, starvation, fairness across work groups, CPU/idle time, cancellation delay |
| Managed allocation | Short-lived and retained graphs, shared immutable captures and disjoint mutable graphs; explicit memory caps | Sync/concurrent throughput, pause/owner-stall distribution, live/peak/reserved memory, root publication cost |
| Lazy compilation | Cold routes amid warmed traffic; recursive and duplicate first calls; compiler contention | Warm-route progress during cold compilation, first response, startup median, compiled/uncompiled counts, compiler waits |
| HTTP production shapes | Native AOT publication target plus JIT development controls, plain/JSON and complex handlers, downstream I/O, multiple apps sharing resources; paced and saturation loads | Offered/completed/error rates, p99 latency, CPU/request, per-app fairness and sustainable capacity |
| Resource limits | Available CPU/affinity/quota, oversubscription and memory limits; fixed and automatic worker policies | Effective parallelism, throttling/idle time, total threads, GC/compiler competition and tail behavior |

Use repeated interleaved runs with fixed work, warmup, offered-load semantics and
input hashes. Separate correctness/sanitizer/profiler runs from timing. Capture
failed/time-out runs as failures, not lower-throughput successes. Microbenchmark
wins alone cannot select the HTTP architecture. Keep the existing benchmark and
TechEmpower scope review: database/network endpoints and realistic persistence
must be admitted before making external benchmark-comparability claims.

## Initial evidence

[28 October baseline refresh](../validation/scheduling-direction-20261028/README.md)
passes the existing native pool's optimized/UBSan safety cases and repeats its
copied-primitive 1/2/4-worker scaling screen. It confirms the benchmark foundation,
not work stealing, context migration, nonblocking JIT waits or shared HTTP scaling.
The scheduling prototype and the expanded matrix above remain to be implemented.
