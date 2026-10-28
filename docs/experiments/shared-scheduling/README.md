# Shared scheduling qualification and profiling

Development experiment, 28 October 2026. `NEOCLR_SHARED_READY_QUEUE` gives the
owner-affine runnable scheduler (including Task.Run and HTTP consumers) and native
workers the same bounded ready storage. It preserves each caller's execution
affinity, admission/root lifetime and completion contract. This is not yet a
unified execution-context lifecycle or general cross-thread managed Task.Run.
There is no HTTP-specific queue or scheduling policy in this change.

The default implementation remains the control. The candidate replaces owner
submission-node allocations with fixed storage and worker slot scans with a FIFO;
the existing optional local/global/stealing policy uses the same queue primitive.
Fixed capacity costs idle memory and cancellation removal is linear. Measure those
costs rather than assuming allocation avoidance is a win. Identities in queues are
root handles or native slot/generation identities, never unrooted managed pointers.

## Required profile before closing the HTTP goal

Author direction: measure the execution context itself, its memory, switching cost
and worker-queue behavior. Keep scheduling/resources constant across sync and
concurrent GC. The [HTTP gate](../worker-delivery/http-plan.md) remains authoritative;
native queue measurements alone cannot satisfy it.

| Layer | Required measurement | Interpretation |
| --- | --- | --- |
| Fixed native metadata | Actual compiler `sizeof` for context, job, pool, owner scope, root records, queue entries/storage | `layout.py`; nested records overlap and must not be summed. Record flags/architecture. |
| Managed context and captures | Allocated bytes/count for Task, continuation/state machine and captures; reachable live bytes at parked high-water and after completion | Compare empty, small capture and shared large graph. Shared graphs count once, not once per root. Include success, cancellation and fault cleanup. |
| Worker resources | Configured/active workers, native stack reservation and resident/high-water process memory, idle memory/CPU | Report per-process shared overhead separately from marginal per-context memory. RSS deltas do not identify managed retention. |
| Suspension/resumption | Pending-await registration, root parking, readiness, enqueue, dispatch start and first resumed execution | Attribute setup, external wait, queue residence and resume overhead separately. An already-completed await is a control, not a suspension. |
| Switching | Repeated cooperative continuation handoff plus owner/worker completion handoff; OS-thread switches separately | No claim of stackful fiber switching for a state-machine/callback transition. Include timestamp-loop control and an untraced timing cohort. |
| Queues | Occupancy/time, high-water, admission/rejection, enqueue/dequeue/removal costs, lock wait/hold, wakeups, batch sizes | Split runnable, running, parked and completed-but-undelivered work; queue size is not total outstanding work. |
| Stealing/fairness | Local/global dispatches, attempts/successes, oldest-job wait and per-class latency | Compare worker counts and mixed long/short work; preserve fairness regressions, not just aggregate throughput. |
| GC/JIT interference | Collection/marker/safepoint intervals and first-call compilation correlated with queue/continuation events | Distinguish collector time from waiting on ownership or compilation. Verify actual concurrent epochs. |

Use bounded traces with correlation identities, monotonic timestamps and explicit
dropped-event counts. Record enabled event classes and instrumentation memory.
Measure tracing overhead separately; do not mix traced and untraced headline
timings. If the available profiler cannot attribute a category, label it unmeasured
and keep that gate open rather than infer it from wall time. Reuse the existing
async/worker trace service before adding another recorder. A trace should make it
possible to reconstruct one request's scheduling/await/GC lifecycle.

Run controlled no-I/O handoffs to isolate scheduler costs, then real HTTP with
many endpoint methods, file reads/writes and downstream HTTP/socket waits. Include
cached/cold-file conditions where controllable and distinguish blocking file APIs
from true asynchronous I/O. Mix short requests with slow/allocation-heavy handlers;
vary active contexts, worker counts and offered load while keeping resource limits
matched. Separate first-use lazy-JIT costs from warm service. Use AOT controls.

The resulting report must connect memory and switching costs to HTTP capacity,
p95/p99, fairness and CPU/request under both collectors. Record the best-supported
tradeoff and remaining limitations; no general GC winner or runtime ranking follows
from a queue microbenchmark. This profiling requirement does not authorize an
unbounded scheduler redesign.

## Reproduction

Use an explicit valid macOS `SDKROOT` when required by the local Xcode setup:

```
python3 docs/experiments/shared-scheduling/validate.py --output target/shared-contract-new
python3 docs/experiments/shared-scheduling/layout.py --output target/shared-layout-new
```

Output directories must not already exist. Reports retain commands, results and
source hashes. The layout probe includes the actual private worker definitions;
it must not be linked alongside another native-workers translation unit.
`Http.rvn` is an in-progress shared-capture Task.Run consumer, compiled with the
same internal Web-source approach as the existing persistent HTTP benchmark.
It is not yet the many-endpoint or capacity qualification.
