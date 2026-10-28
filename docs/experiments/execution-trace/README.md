# Bounded worker execution trace — 2026-10-28

Implemented private POSIX worker-pool instrumentation, the first slice of the
[execution visibility investigation](../../design/go-inspired-scheduling.md#execution-tracer-proposed-first-slice).
It does not yet trace production Tasks, async fibers, JIT or GC phases. Existing
pool scheduling, Task semantics and production defaults are unchanged.

## Recording contract

Build the runtime with `-DNEOCLR_WORKER_TRACE`, include
`runtime/native/workers/native-worker-trace.h`, and create a pool with
`neoclr_workers_create_traced(workers, job_capacity, heap_bytes, events_per_lane)`.
Normal builds compile out recorder storage and hooks entirely. In instrumented
builds the ordinary constructor disables recording. Configuration is immutable: zero
allocates no event buffers and emission returns before clock/TLS access; 1–65,536
reserves that many native records for each worker plus the external lane. Failure
to reserve buffers fails creation. There is no managed public API or CLI switch.

Each worker has a single-writer lane. External callers share lane zero, serialized
by the existing pool gate. Lane IDs are logical identities, not OS thread IDs.
Emission takes no additional lock and performs no allocation, formatting or I/O.
Some hooks execute inside the existing scheduler critical section, so even clock
and store costs can affect contention. Callback return records occur outside it.
Instrumented builds retain extra pool metadata and disabled branches: a longer
two-worker check showed about 10% higher wall time. This motivated build-time
opt-in instead of imposing that path on normal builds.

Records carry process-monotonic nanoseconds, a per-lane sequence, job slot and
generation, event kind and outcome/reason. Pool identity and clock origin accompany
each view. No application strings, managed references or native object addresses
are captured. Thread startup/exit, condition-wait boundaries, enqueue, dispatch,
callback return, settlement and cancellation are recorded. The callback interval
includes synchronous helper/lock/native blocking and OS descheduling: it is **not
CPU utilization**. Settlement follows callback return and can itself wait for the
gate. Queue delay begins at publication, not at the submit API call.

Buffers keep a prefix rather than wrap. Capacity exhaustion or a failed clock
read increments loss accounting; attempted sequences expose gaps. Full buffers
skip clock reads. No live readers, draining, signal-handler emission or reentrancy
from signals is supported. Once the owner closes/joins the pool and excludes
external callers, `neoclr_workers_trace_view` returns borrowed immutable views
until destroy. Export must finish before destroy; the tracer owns no GC roots.

The offline Python converter produces Chrome trace JSON with worker lanes, queue
flows, callback/condition-wait spans and raw transition points. Loss and incomplete
intervals remain explicit in `neoclr` summary metadata. It rejects malformed
schema/order/lane/loss accounting. Missing events cannot establish deadlock or
exact state durations. Cross-process merging and clock synchronization are not
implemented. Current schema is a private versioned experiment, not EventPipe
compatibility. Go/.NET and format comparisons are in the linked design.

## Reproduce and inspect

```sh
SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk python3 docs/experiments/execution-trace/validate.py --baseline-revision ad6ebc989 --output target/trace-check
python3 docs/experiments/execution-trace/export.py target/trace-check/optimized-1-1024.json target/trace-check/overview.trace.json
```

Open `overview.trace.json` in Perfetto's **Open trace file** action. This slice
checks the exported event structure; visual import is not an automated test.
The one-worker fixture includes a deliberately blocking callback, queued/running
cancellation, failed callback and reused slots. Four workers exercise concurrent
producers. One-record buffers force overflow. Truncated and invalid reports test
the converter, and disabled mode rejects trace reads.

The validator runs optimized and fail-fast UBSan variants; neither is a race
detector. Its tiny-job screen runs 8,192 checked jobs per invocation, three
alternating traced/untraced pairs at 1/2/4 workers. It also compares the default build (tracing compiled out), using 524,288 jobs per run,
with the pre-tracer source at the named revision using a disabled-only API
adapter. Setup/close/export are excluded from timings; allocations and first
buffer touches during execution are included. Retain all raw samples: short runs
and desktop load make this a diagnostic screen, not HTTP capacity evidence.

[Retained qualification and overhead measurements](../../validation/execution-trace-20261028/README.md).
The next [owner-thread async slice](async-README.md) now records suspension,
resumption, nested execution and abandonment, qualified with both collectors.
Shared session identity and correlated JIT/GC events remain next. Queue-policy comparisons must retain
untraced controls; tracing overhead must not be mistaken for scheduler behavior.
