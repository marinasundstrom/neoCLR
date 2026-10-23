# HTTP allocations, async I/O and execution owners — 2026-10-23

The current plain HTTP fixture is limited by work on one native execution owner,
with substantial GC/root bookkeeping and allocation costs. Two independently
owned server processes sustain more aggregate work. This supports investigating
multicore execution alongside reducing per-request work; it does not qualify a
shared-heap thread pool or identify a single cause for all HTTP workloads.

This is author-directed investigation following MediaType and URI work. A value
object may use multiple allocations; there is no one-allocation requirement.
Zero allocations is an exploratory question, not a correctness or release gate.

## Measured workload and limits

Apple M1/macOS, matching development Raven 1128b98cf, source libraries including
the URI optimization, AOT plain HTTP/1.1 connection-close, 10 response bytes,
16 Serve lanes, 64 client slots, explicitly paced synchronous GC, 1 MiB native
heap. The retained Server fixture is the existing Serve workload migrated from
Result.Error to Failure. Its handler deliberately awaits Task.Run, constructs a
byte array and returns a response. This extra scheduling is part of the fixture,
not advice to wrap ordinary async I/O in Task.Run.

Use [summary.json](summary.json) and [compressed raw evidence](evidence/manifest.json).
All response bodies, successful server totals and clean shutdowns were checked.
All builds/profilers finished before uninstrumented timing. Twelve-second trials
are exploratory local operating points, not production maximum capacity. No TLS,
persistent connection, JSON, database, Windows or JIT performance claim follows.
The URI change is not an A/B throughput comparison: this incoming path already
keeps primitive URI components.

## Allocation accounting

Independent Python peers validate nine client and nine manual Accept/Complete
server exchanges. The first exchange is retained separately; the following eight
give identical stage counts within each fixture.

| Stage | Warm client tracked objects | Warm server tracked objects |
| --- | ---: | ---: |
| Construct request | 18 | — |
| Send/receive or Accept/read | 199 | 88 |
| First Content-Type projection | 9 | 9 |
| Repeated Content-Type projection | 0 | 0 |
| Configure text response | — | 7 |
| Complete response | — | 46 |
| Dispose | 0 | 0 |
| Total measured stages | 226 | 150 |

Cold client counts are 19/201/9/0/0; cold server counts are 92/9/0/7/46/0.
Startup, logging, loop bookkeeping and final client/server shutdown are outside
these stage boundaries. These are interpreter GC.AllocatedObjectCount results,
not all host/native allocations or complete string storage. They are also a
different consumer path from native Serve below; do not compare them as an engine
speed or allocation ratio. Await scheduling may occur across stage boundaries.

Native caller capture compares 200 with 400 paced requests, each with identical
20-request warmup and one stop exchange. Whole-process totals are 48,271 versus
91,471 managed allocations, and 3,551,017 versus 6,724,817 payload bytes. Subtracting
the runs gives **216 allocations and 15,869 payload bytes per additional request**.
No caller records were dropped; counts and bytes reconcile.

| Native allocation kind | Increment/request | Payload bytes/request |
| --- | ---: | ---: |
| Text | 4 | 55 |
| Objects | 166 | 9,352 |
| Byte arrays | 17 | 954 |
| Reference-slot arrays | 12 | 300 |
| Record arrays | 17 | 5,208 |
| Intern entries | 0 | 0 |

This delta removes common startup/shutdown overhead, but is one pair of diagnostic
runs. It includes application payloads/tasks and response processing. Native
payload bytes exclude allocator headers, stacks, OS allocations and C malloc.
In particular the socket adapter's transfer buffers and runnable queue nodes
allocate outside this managed counter. This is not an all-allocator zero test.

Promise result arrays account for 2,909 allocations and 1,144,237 payload bytes
(32.2% of the shorter whole-process payload total). They retain generic result
state; removing their cost must preserve repeated awaits, reference/value/union
results, pending roots and cancellation. The previous inline-result experiment
regressed bulk p99 despite saving allocations; do not simply reapply it.

## Throughput, latency and profiling

| Run | Offered/s | Completed within window/s | Unsent slot-limit misses | Offered p99 |
| --- | ---: | ---: | ---: | ---: |
| Single process A | 400 | 400 | 0 | 3.94 ms |
| Single process overload | 800 | 495.33 | 3,593 / 9,600 | 134.77 ms |
| Single process B | 400 | 400 | 0 | 6.97 ms |
| Two simultaneous processes, each | 400 | 400 | 0 | 9.75 / 9.56 ms |

At 800 offered/s, 6,007 admitted responses validate; 5,944 finish within the window
and 63 during drain. The unsent requests are load-generator admission misses, not
HTTP error responses. CPU consumed is 11.98 seconds over the 12-second window,
consistent with one busy execution owner. At 400/s the two single-process runs
consume 9.35/9.36 CPU seconds and peak at 13,168 KiB sampled RSS.

The two process windows begin within 7 microseconds of each other and each complete
4,800 requests, giving **800/s aggregate** over their overlapping 12-second
windows. They consume 9.88/9.86 CPU seconds. This is evidence of available multicore
headroom with independent ownership, not evidence that two shared-heap threads
will achieve the same gain. There are two heaps/listeners, 32 total lanes and 128
total client slots; no load balancer or connection handoff is measured.

A separate five-second wall-stack sample under 400/s load gives these exclusive
main-thread shares (3,449 samples):

| Symbol/work | Share |
| --- | ---: |
| Root publication | 19.9% |
| Indexed collection | 18.6% |
| Managed allocation | 16.4% |
| Heap-base validation | 8.4% |
| String-length checks | 8.0% |

These are stack samples, not exact CPU durations or removable percentages.
Inclusive callers overlap and are not added. The sampled run has admission misses;
its throughput is diagnostic only. The leading symbols are in the native C
GC/root layer; this measurement does not implicate Rust interpreter execution.
String-length checks here often validate root descriptors, not HTTP string copies.

## Async I/O and threading findings

The audited native HTTP path already uses nonblocking sockets:
[socket-os.h](../../../../docs/experiments/aot-console/socket-os.h) sets O_NONBLOCK
(or FIONBIO). Pending accepts/transfers are progressed by
[socket-listener.c](../../../../docs/experiments/aot-console/socket-listener.c).
The owner waits with poll on macOS or select on Windows, bounded to one millisecond
because DNS, timers and legacy TLS lack one common wakeup event. It scans a bounded
64-operation table and dispatches ready callbacks. Socket waits are not implemented
as one blocked OS thread per request.

DNS is already offloaded to a detached OS worker in
[socket-client-internal.h](../../../../docs/experiments/aot-console/socket-client-internal.h).
That worker is separate from managed request execution. Receive/send adapters still
snapshot through native transfer storage; its ownership/copy costs deserve separate
measurement, especially for larger bodies.

Task.Run in this native host submits a rooted runnable to the same owner queue.
The private [async provider](../../../../runtime/native/async/native-runtime.c)
parks/resumes cooperative fibers on that owner. Thus there is already a private
fiber mechanism, but no qualified multicore managed HTTP scheduler in this path.
Adding more lanes or another Task.Run around a handler does not add CPU parallelism.
An arbitrary blocking or long-running callback can still occupy the owner.

Do not generalize that limitation to every backend. The Rust VM has separate
[shared-context work](../../../../src/task_work.rs), an
[isolated worker pool](../../../../src/workers.rs) and asynchronous DNS machinery.
Their existence is not proof that the AOT HTTP host supports shared managed
parallelism. Likewise, background GC tracing is not parallel handler execution.

.NET distinguishes asynchronous waiting from CPU execution: async/await does not
itself create threads, and Task.Run submits computational work to the thread pool.
That is the semantic baseline, not a reason to wrap each nonblocking socket call
in a task or create a thread per connection.
[Primary .NET explanation](https://github.com/dotnet/docs/blob/main/docs/csharp/asynchronous-programming/task-asynchronous-programming-model.md).
Treat the native Task.Run parallelism gap as a backend qualification gap, not a
new permanent semantic definition.

## Recommended bounded follow-ups

1. **Qualify native parallel execution ownership.** Reuse the existing shared-task
   research and compare two execution owners with a bounded shared-heap pool.
   Establish captured-object identity, cross-thread completion, cancellation,
   resource affinity, joined shutdown and roots from running/parked stacks.
   Isolated heaps can scale today experimentally, but silently copying captures
   would violate the intended Task.Run contract. Do not send the current native
   context or mutable queues to arbitrary threads without this work.
2. **Qualify event-driven wakeups.** Measure idle CPU, syscall count and wake-to-run
   latency with many idle connections, fragmented reads, slow receivers, timers
   and DNS completion. Compare the current bounded poll loop with a unified wakeup
   source and platform readiness/completion backends. kqueue/epoll/IOCP are design
   candidates, not implemented features or guaranteed speedups. Keep deadlines,
   partial transfers, cancellation and fairness behavior.
3. **Reduce per-request task/root work independently.** The existing
   [runtime completion bridge proposal](../../../../docs/design/runtime-task-completion-bridge.md)
   targets redundant Promise/result adapters. Avoid repeated root-descriptor
   strlen/marker parsing using an explicit validated immutable-metadata contract;
   a pointer-only cache is not sufficient. Native allocation and owner lookup still
   validate heap spans: an index requires maintained integrity/invalidation rules,
   not deletion of corruption checks.
4. **Measure bulk-copy and construction opportunities.** Request construction
   initializes empty header/content storage that incoming factories replace;
   context response defaults may be replaced by a handler response. Test avoiding
   that discarded work. Response snapshot loops still copy bytes individually;
   a checked bulk copy must retain snapshot isolation and partial-send semantics.
   These are observed candidates, not adopted optimizations.

For threading experiments, compare one/two/four owners under separate CPU-heavy,
await-heavy and deliberately blocking handlers. Include an unrelated latency probe
to detect head-of-line stalls; report CPU, allocations, misses, latency and shutdown.
Preserve the connection-close baseline and qualify keep-alive separately. Green
threads may reduce suspension overhead, but need scheduling across OS threads to
provide multicore execution. No pool, preemption or collector default is selected.

## Reproduction

Build with task-completion-20261018/build.py using this Server.rvn, the matching
URI bundle libraries/compiler/codegen and the retained link recipe. Run
caller-profile-20261018/capture.py for 200 requests; run diagnose.py again on its
instrumented app with rate 20, duration 20 and NEOCLR_ALLOCATION_CALLERS set for
the 400-request trace. Run diagnose.py on the uninstrumented app-paced at 400,
800 and 400 requests/sec for 12 seconds each; launch two separate 400/s runs
simultaneously for the independent-owner check. The retained manifests contain
exact commands, source identities and toolchain paths.

Run allocations.py --bundle BUNDLE --output FRESH for public-stage counters.
Recompute the report with summarize.py --evidence evidence --output summary.json.
No public API, threading implementation, runtime default or published performance
claim changes in this investigation.
