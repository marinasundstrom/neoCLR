# Private queue-policy experiment

`NEOCLR_WORKER_QUEUES` selects a bounded alternative to the existing FIFO scan.
External submissions use global FIFO; same-pool callbacks submit locally, consume
newest local work and steal oldest work from other locals. A worker services
waiting global work after at most eight local dispatches. This bounds dispatch
count, not waiting time: callbacks are not preempted. Queued cancellation unlinks
before settlement/slot reuse. The existing shared gate still protects all queues,
job states and GC roots. Limits remain four workers and sixteen outstanding jobs.
No stack migration, general Task.Run or parallel HTTP capability is introduced.

This tests the local/global/stealing idea discussed in the existing
[Go/.NET design comparison](../../design/go-inspired-scheduling.md), without
adopting either runtime's implementation or changing .NET-facing Task semantics.
Extra metadata and changed local order are costs; locality/less scanning are
hypotheses. The [results](../../validation/queue-policy-20261028/README.md) do not
justify switching the default. Native copied-work timings cannot compare GC modes.

Run from the repository root on macOS with the Xcode SDK selected:

```sh
python3 docs/experiments/queue-policy/validate.py --output target/queue-comparison
python3 docs/experiments/queue-policy/measure_long.py --binaries target/queue-comparison --output target/queue-long
python3 docs/experiments/queue-policy/profile.py --binaries target/queue-comparison --output target/queue-profiles
python3 docs/experiments/native-worker-pool/validate.py --local-queues --compiled-workers --trace-workers --output target/queue-safety
```

Output directories must be new. Profiles are separate from timing; the native
sleep shape is a blocking control, not asynchronous I/O. Owner timestamp/result
collection and batch barriers are part of these synthetic workloads. Preserve
both short screens and the longer follow-up, rather than selecting a favorable run.
