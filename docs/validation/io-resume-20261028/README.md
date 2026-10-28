# Real TCP completion-to-resume baseline — 2026-10-28

The existing owner-local path resumes promptly once runtime completion is published.
A long non-yielding callback delays observation of incoming I/O: splitting the
same loop count across callbacks gives lower latency but slightly longer batch
time. No scheduler/default change or universal GC winner follows from this screen.

## Method and qualification

macOS ARM64, runtime source at `5c5260e8b`, native C callbacks using the actual
socket adapter, scheduler and async stack carrier. This is not generated JIT/AOT
code, HTTP, messaging or UI execution. One persistent loopback TCP connection,
one outstanding one-byte receive per round, 256 rounds, three alternating collector
pairs per shape. A native peer thread waits for admission, sleeps 200 µs, timestamps
immediately before send and sends one byte. It never touches managed state.
The owner parks a context holding a traced live local, completes the receive via
ordinary scheduler callback, publishes the awaited operation, resumes and checks
the local, then finishes the batch. Sixteen callbacks each run 20,000 xorshift
iterations; the long control runs one callback with 320,000 iterations. Per-callback
seed updates and scheduling overhead differ. Allocation adds 32 discarded 24-byte
objects per short callback. Live graph size stays small; this is churn, not a large
survivor-heap collector test. Heap budget is 1 MiB.

Both collectors pass all four shapes optimized and fail-fast UBSan. The owned
concurrent candidate reports actual started/finished epochs. Separate traced runs
export successfully through the existing async trace converter with no dropped
events. Callback/result consumption, parked-root preservation, complete cleanup
and zero remaining heap usage are checked. The initial build failed because a
POSIX feature macro hid INADDR_LOOPBACK; using its numeric loopback address fixed
it. Retain that diagnostic, earlier screens, traces and separate profiles.

The synchronous baseline is the unpaced native collector; concurrent uses the
existing owned-candidate policy (including pacing/profiling defines). This compares
those configurations, not marker concurrency alone. No tracer or sanitizer is
compiled into timed binaries. Peer polling/sleep CPU is included in process CPU.
Desktop load/core placement is uncontrolled; runs are short and p99 has only 256
observations per run. Quantization is visible. Values are medians of three per-run
statistics, not pooled percentiles or capacity estimates.

| Shape | Collector | Batch wall ms | Process CPU ms | Send-to-resume p50 / p99 µs | Completion-to-resume p99 µs |
| --- | --- | --- | --- | --- | --- |
| Idle I/O | sync | 86.8 | 11.9 | 55 / 83 | 3 |
| Idle I/O | concurrent | 100.8 | 21.0 | 62 / 82 | 3 |
| 16 CPU callbacks | sync | 177.0 | 182.9 | 46 / 86 | 2 |
| 16 CPU callbacks | concurrent | 179.8 | 186.7 | 42 / 73 | 2 |
| 16 CPU + allocation callbacks | sync | 180.0 | 186.1 | 47 / 85 | 2 |
| 16 CPU + allocation callbacks | concurrent | 184.2 | 191.1 | 47 / 80 | 2 |
| 1 long CPU callback | sync | 168.4 | 173.2 | 365 / 455 | 2 |
| 1 long CPU callback | concurrent | 168.0 | 174.4 | 355 / 396 | 2 |

Send-to-resume includes send syscall, kernel/network delivery, readiness observation,
callback dispatch and resumption. The timestamp is not a kernel readiness timestamp.
Completion-to-resume starts immediately after CompleteUnit returns in the callback;
it excludes socket polling/transfer and callback admission. Neither metric is a
pure context-switch instruction cost. No request-rate claim is inferred from a
closed-loop fixture with intentional peer delay and a per-round drain barrier.

Earlier three-shape cohorts have mixed collector tails (one allocation cohort had
concurrent p99 252 µs versus sync 140 µs); preserve them rather than treating the
last cohort as stable superiority. The one-long-callback comparison was added
after the first screen to test handler-shape sensitivity; it was not a predeclared
independent confirmation. Final validation includes it under both collectors.

## Profiles and interpretation

Separate two-second macOS stack samples use the earlier three-shape binary and
4,096 rounds of CPU/allocation work. Sync/concurrent top-of-stack samples include
invoke (1017/677), peer sleep (1137/858) and concurrent condition waits (1001).
They include off-CPU time and different thread counts, so cannot be compared as CPU
percentages. Carrier setup (`sigprocmask`, `sigaltstack`, `getrlimit`) is also visible.
Raw stacks and process results are retained; profiled timings are excluded above.

The best next investigation is bounded cooperative execution or offloading CPU
work while retaining owner-thread affinity, with explicit fairness/admission and
root ownership. Do not shorten collector pauses or redesign completion queues on
the assumption that they caused the long-callback latency. First expand to several
connections, paced arrival independent of completions, cancellation/backpressure,
larger live graphs and generated HTTP handlers. Compare shared runtime behavior
in JIT development and AOT publication configurations with both collectors. UI
frame/input and mobile idle-energy goals still need target-specific qualification.

[Fixture and reproduction](../../experiments/io-resume/README.md).
`artifacts.json` records decompressed hashes for archived raw evidence. No website
publication or production scheduler change is included.
