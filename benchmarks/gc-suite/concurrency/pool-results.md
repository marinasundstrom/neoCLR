# Shared-heap handoff profile — 2026-10-25

Six diagnostic runs pass the unchanged compiled four-worker graph, fault and
shutdown consumers using identical retained generated objects: three paced sync
and three restricted concurrent read-epoch runs. Each has 3,904 graph, 128 fault
and 488 shutdown allocation requests. Concurrent graph runs complete 20 epochs.
No worker, GC, task or fiber policy changes are made by the profiling tool.

`profile-pool.py` copies the worker source and wraps condition waits and mutex
acquisition with monotonic wall and calling-thread CPU clocks. Original lifecycle,
root, allocation-count, fault and shutdown assertions remain active. Instrumented
waits preserve the original return code and locking semantics. Atomic counters and
clocks add overhead: these are attribution diagnostics, not throughput benchmarks.

Median across three invocations; time is aggregate milliseconds across threads:

| Site | Sync calls | Sync wall / thread CPU ms | Concurrent calls | Concurrent wall / thread CPU ms |
| --- | ---: | ---: | ---: | ---: |
| Allocation requester wait | 4,520 | 82.16 / 10.90 | 4,520 | 80.03 / 9.85 |
| Collection requester wait | 22 | 0.36 / 0.05 | 44 | 1.19 / 0.11 |
| Allocation/collection service idle wait | 4,544 | 218.99 / 29.35 | 4,565 | 221.01 / 28.62 |
| Compiled execution boundary wait | 33,191 | 640.33 / 143.16 | 36,251 | 644.73 / 147.19 |
| Mutex acquisition | 120,503 | 91.26 / 40.29 | 120,675 | 92.06 / 42.61 |

Whole-process medians are 232/233 ms wall and 324/330 ms CPU for sync/concurrent.
Wait wall times overlap between threads and must not be summed as process elapsed
or called CPU time. Condition wait includes time reacquiring its mutex. These
short tests include startup, deliberate cancellation/faults and shutdown, not a
steady application load; service idle time is not useful work or a defect by itself.

## Architectural implication

Both collectors incur thousands of allocation handoffs and tens of thousands of
execution-boundary waits, despite only tens of collections. Current allocation
requests retain the exclusive execution lease while waiting on a separate service.
The restricted concurrent path also ends each read epoch before the next exclusive
operation; the first profiled graph run admits only 20 readers over 20 epochs.
This limits overlap in allocation-heavy code independently of marker efficiency.

The evidence warrants testing fewer handoffs, not assuming virtual threads are
necessary. A bounded next candidate is allocation by the requesting participant
while it already holds the exclusive lease, retaining the dedicated collection
owner and existing accounting, pressure, roots and cleanup. First verify that the
external-arena allocator needs no owner-thread cache authority. Compare unchanged
policies and workloads, retain missing-root/dispatch negative controls, and profile
before/after separately from timing. This is a proposal, not implemented behavior.

Broader concurrent mutation, per-worker allocation regions, batching and work
stealing have larger ownership/proof requirements. Do not bypass admission to
make the graph appear parallel. Task enqueue/resume and fiber-switch costs remain
to be instrumented; the measurements here are OS synchronization, not task switches.

Reproduce with `python3 benchmarks/gc-suite/concurrency/profile-pool.py --output
 target/new-pool-profile` (one shell line), using the retained builds named in the
script. `pool-waits.json.gz` retains commands, checks, outputs and source hashes.

## Tuned independent-owner follow-up

Eight additional loaded profiles pass 513,811 checked plaintext responses using
the qualified watermark/index-only concurrent owner builds, one/four owners and
two repetitions. Median CPU microseconds/response:

| Policy | One owner | Four owners |
| --- | ---: | ---: |
| Watermark | 255.2 | 328.8 |
| Index-only | 250.4 | 318.9 |

The more recent tuning reduces cost relative to older qualified hosts, but these
are separate cohorts with different code/policies, not a controlled attribution
of the entire difference. Four-owner CPU/response still rises. `tuned-profile.json.gz`
and eight `tuned-*.sample.txt.gz` files retain diagnostics. No capacity, precise
scheduler-delay or concurrent-default claim is made.
