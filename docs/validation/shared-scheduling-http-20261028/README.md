# Generated HTTP shared-capture and carrier profiling qualification

Development evidence, macOS ARM64, 28 October 2026. This slice establishes real
HTTP execution with shared ready storage and starts profiling the native carriers.
It does not establish an HTTP performance improvement or close the scheduling goal.

## Application boundary

`Http.rvn` runs persistent HTTP with two handlers. Both await Task.Run; the plain
handler mutates a captured object, and JSON allocates 64 temporary objects and
mutates through aliases before returning the checked typed response. Identity and
mutation assertions execute in the generated application. The shared scheduler
preserves owner affinity; this does not move HTTP captures to a worker pool heap
or qualify simultaneous managed mutators. No HTTP-specific scheduling policy is added.

[Eight untraced smoke cases](http-smoke.json) pass: two handlers × baseline/shared
ready storage × sync/concurrent GC, 32 measured requests plus 4 warmup requests,
two clients, keepalive. The concurrent runs start and finish marking epochs;
shutdown cancellation is recorded separately. [Eight diagnostic cases](http-trace.json)
also pass with process-memory snapshots and bounded owner async/GC tracing.
All traces have zero lost events. These are short qualification runs, not medians
over repeated performance trials, saturation tests or sufficient throughput evidence.

Both modes have 1 MiB heaps and the same 1024-allocation/16384-publication pacing
budgets. The same optimized codegen test executable supplies lazy JIT. The current
generic-contract Raven compiler/libraries compile the guest, and a freshly built
matching runtime reads its metadata. The older bundle's inspection binary cannot
read the new generic-parameter token metadata. A [debug-runner attempt](debug-runner-timeout.json)
reached readiness but timed out compiling the first response; a stack sample showed
lazy compilation, not a demonstrated scheduling deadlock. It is excluded from results.

Build commands/source hashes: [smoke](smoke-build.json), [trace](trace-build.json).
The trace host subsequently adds optional trace export; the ordinary host binary
used by the smoke cohort is retained separately in its build output. Raw timings
include qualification overhead and first-call compilation; do not rank runtimes
or infer startup trends from this cohort.

## Actual native carrier and memory inventory

[Compiler layout probe](carrier-layout.json), using actual private definitions:

| Item | Bytes / count |
| --- | ---: |
| Carrier record (includes machine context and parked roots) | 1,056 B |
| Machine context, included above | 880 B |
| Per-owner TLS state, including fixed carrier/operation arrays | 138,512 B |
| Carrier capacity | 128 |
| Usable stack per mapping | 1,048,576 B |
| Two guard pages per mapping on this host | 32,768 B |
| Completed-stack cache limit | 8 |

The ordinary HTTP runs report 229 carrier activations, 230 resumes, two mappings
and two unmaps, with a 2,162,688-byte peak stack reservation and zero reservation
after shutdown. A new managed Task does not imply a new stack mapping. Reuse is
material, and reservation is not resident memory. The process snapshots include
compiler/JIT, collector and native memory; they cannot identify retained captures.
The managed-retention and per-context resident-cost gates remain open.

## Trace findings and limits

[Summary](trace-summary.json) retains event counts, GC request intervals, native
park/finish-to-return intervals, completion-to-carrier-entry intervals, OS switch
counters and process memory. Each diagnostic run reserves 6 MiB for the two bounded
event buffers, excluding recorder headers. This is instrumentation cost, not
production task memory. OS switch counters include JIT/GC helper threads and must
not be interpreted as cooperative task switches.

The observed finish-to-return median is 1 µs in these traces, but timestamps are
quantized at microsecond granularity and include trace emission. Some intervals
are zero. This is insufficient to claim a pure context-switch cost or a small
optimization. Root parking occurs before that interval begins.

Only one native parked-carrier transition appears per run despite many managed
Task.Run invocations. The carrier trace does not cover every generated state-machine
await/queue interval. Consequently it cannot establish Task.Run scheduling latency,
fairness or retained-state costs. Extend shared scheduler admission/dispatch
instrumentation and use a sufficiently precise clock plus untraced controls before
making those claims. Traced GC intervals likewise include diagnostic overhead and
do not establish a collector winner.

## Required next evidence

Measure queue residence, lock/wakeup and real managed suspension/resumption;
account for shared capture retention and idle/peak resident memory. Extend handlers
to many endpoints with file and downstream HTTP/socket I/O, mixed short/long work,
fixed resource limits and offered-load pressure. Compare AOT distribution controls
and lazy JIT, both collectors, with repeated untraced timings and separate profiles.
The [acceptance gate](../../experiments/worker-delivery/http-plan.md) remains open.
