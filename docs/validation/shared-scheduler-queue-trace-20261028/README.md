# Shared scheduler queue residence and precise trace clock

Development qualification, macOS ARM64, 28 October 2026. This extends the
[HTTP carrier profile](../shared-scheduling-http-20261028/README.md) to actual
runnable submissions, without adding HTTP-specific scheduling policy.

## Instrumentation contract

The existing bounded session recorder now uses `mach_absolute_time`, converted
with the platform timebase using widened integer arithmetic, on macOS. Other
POSIX platforms retain CLOCK_MONOTONIC. The clock name is explicit in exports;
timestamps from the different clocks must not be combined. Existing independent
worker/async recorder controls retain their original clocks. Production builds
without tracing do not acquire a recorder or clock dependency.

Shared owner scheduling emits admission, dequeue, dispatch-root transfer, release
and queued-abandonment events. Keys are per-scope diagnostic identities and opaque
root handles, never object addresses. Both linked-list control and shared ready
storage use these hooks. Admission-to-dequeue is queue residence; dequeue-to-dispatch
is root-transfer preparation. Release marks the end of dispatch-root ownership,
which can precede callback execution; it is not a callback completion timestamp.
Failed admissions are not yet classified by reason. No queue policy changes occur.

## Validation

- [Session checks](session-validation.json.gz): optimized/UBSan producer lifetime,
  bounded loss, malformed/missing records and sync/concurrent async/GC correlation
  pass. Its short recording-overhead controls overlapped host-build activity and
  are not accepted timing evidence; retain them as raw validation output only.
- [Scheduler lifetime checks](lifetime.json): existing root, fairness and teardown
  checks pass with recording under both collectors, optimized/UBSan. Each observes
  513 admitted runnables, 386 dequeues and 127 queued abandonments, with zero loss.
- [Compiled-out check](default-assembly.json): baseline and shared queue adapter
  assembly is identical to the preceding revision with tracing disabled, ignoring
  only the source `.file` directive.
- Eight HTTP diagnostic runs pass (plain/allocating JSON × baseline/shared × both
  collectors): 128 requests plus 4 warmup, four clients, keepalive. Every run has
  132 traced admissions and matching dequeue/dispatch/release lifecycles, no pending
  queue/ticket at shutdown and no dropped events. GC activity is correlated in the
  same owner clock. [Raw traces](http-traces.json.gz), [summary](summary.json),
  [build inputs and commands](build.json).

## Observed queue residence

One diagnostic run per cell; values describe distributions in that run, not
repeated-trial medians. Includes cold warmup and trace/process-memory overhead.

| Handler | GC | Control median | Shared median | Shared p99 | Queue high-water |
| --- | --- | ---: | ---: | ---: | ---: |
| Plain | Sync | 4.291 µs | 4.125 µs | 8.042 µs | 1 |
| Plain | Concurrent | 5.417 µs | 5.291 µs | 9.167 µs | 1 |
| Allocating JSON | Sync | 4.584 µs | 4.500 µs | 19.875 µs | 1 |
| Allocating JSON | Concurrent | 5.958 µs | 7.500 µs | 129.417 µs | 1 |

The concurrent JSON candidate regresses in this short run; retain it rather than
selecting only favorable cells. Maxima are roughly 9.5–17.1 ms and include first-use
compilation effects. This cohort cannot establish a scheduler or collector winner.
The high-water mark of one in **every** run proves these cases do not exercise queue
pressure. Many-endpoint mixed-I/O/long-handler traffic and offered-load testing are
still needed to assess fairness, capacity and sustained throughput.

Carrier finish-to-return medians now resolve at 875–1,125 ns rather than whole
microseconds. Those intervals include tracing and context return and exclude earlier
root parking; they remain unsuitable as a pure switch-cost claim. Recorder buffers
still reserve 6 MiB per diagnostic process; the scope identity adds 8 native bytes
in trace builds. Shared-capture retained memory, queue lock/wakeup costs, dispatch
execution time, trace overhead under representative HTTP load and AOT controls
remain open profiling gates. No production default or throughput table is replaced.
