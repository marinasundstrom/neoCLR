# Owner-affine I/O and CPU offload across runtimes — 2026-10-28

For an independent CPU job taking roughly 6 ms, explicit offload reduces median
TCP acknowledgement latency from about 5.8 ms to 116–130 µs in all four tested
configurations, without a material reduction in total batch duration. This supports
separating CPU work from an I/O owner, not making computation itself faster. The
tiny-job control has mixed tails and CPU costs: do not infer universal offloading
or a collector/runtime winner.

## Comparable contract and remaining differences

One persistent loopback TCP connection, one outstanding byte receive and one
independent CPU job per round. The server sends readiness, then performs no CPU
work, performs the work on its owner, or offloads it. A common external Python peer
waits 200 µs after readiness, timestamps immediately before sending `x`, and measures
until it receives `A`. The owner acknowledges I/O before waiting for any remaining
CPU work; the next round starts only after both finish. The CPU result is independent
of that acknowledgement: this is not a handler whose response depends on the job.
Every implementation checks payloads and uses the same unsigned 64-bit xorshift
loop and initial value; Python independently computes and checks the final result.
TCP_NODELAY is enabled on both ends.

- **neoCLR:** native C fixture at runtime revision `c5a4542ca`; real socket scheduler
  and rooted async carrier, ordinary sync collector versus the owned concurrent
  configuration. Offload uses the existing private FIFO pool, one worker/one slot,
  copied primitive computation and no shared managed pointers. Worker completion is
  joined only after the I/O pump returns. This is not generated Raven/JIT/AOT code,
  general managed Task.Run, async worker completion integration or UI support.
- **.NET:** SDK/runtime 10.0.100/10.0.0, Release C#, owner queue implemented through
  SynchronizationContext, async Socket receive and Task.Run offload. A continuation
  checks owner-thread identity. Tiered compilation is disabled for a stable optimized
  compute method; workstation concurrent GC is requested. This custom context is an
  affinity control, not ASP.NET's normal dispatch configuration or a UI toolkit.
- **Go:** 1.26.3, compiled normally, GOMAXPROCS=2, LockOSThread on the application
  owner, a receive goroutine delivering to that owner and a compute goroutine for
  offload. This deliberately retains owner affinity, rather than letting the receive
  goroutine acknowledge independently during the inline control.

All permit at most one CPU job plus an owner; **OS-thread counts and CPU quotas are
not equalized**. Runtime I/O, GC and housekeeping threads differ, and Go's two Ps
are not equivalent to a .NET pool limit. The native pool uses another heap for its
rooted capture, without managed computation. Concurrent neoCLR includes existing
pacing/profiling policy defines absent in its unpaced sync control. Thus this is an
application-pattern comparison, not a scheduler-only cost or fair overall runtime
ranking. Source/compiler/GC/lifetime differences must remain visible.

The host is macOS ARM64 with uncontrolled desktop load and core placement. Three
repetitions rotate runtime order and reverse mode order. Each process runs 32 warmup
plus 256 measured rounds; client latency drops the warmup. Server wall/CPU include
all 288 rounds and exclude startup, compilation, socket setup and final teardown;
.NET may still perform runtime work during those rounds. Client-process CPU is
excluded. Tables show medians of per-run values, not pooled percentiles. All raw
latencies and observed peer delays are retained. No memory, energy, cold-start,
HTTP throughput, large-live-heap or sustained-capacity claim follows.

## Long work: 3,200,000 iterations

| Runtime/configuration | Work placement | Ack p50 / p99 µs | Server batch wall ms | Server CPU ms |
| --- | --- | --- | --- | --- |
| neo-sync | idle | 104.1 / 141.3 | 106.7 | 8.1 |
| neo-sync | inline | 5771.8 / 6227.2 | 1744.9 | 1743.6 |
| neo-sync | offload | 130.4 / 167.8 | 1742.9 | 1747.9 |
| neo-concurrent | idle | 113.4 / 148.0 | 109.7 | 17.7 |
| neo-concurrent | inline | 5797.6 / 6466.8 | 1757.3 | 1755.2 |
| neo-concurrent | offload | 126.4 / 207.7 | 1770.5 | 1766.5 |
| go | idle | 112.5 / 172.4 | 109.7 | 17.0 |
| go | inline | 5779.2 / 6087.9 | 1746.0 | 1752.3 |
| go | offload | 128.1 / 225.1 | 1750.9 | 1759.3 |
| dotnet | idle | 97.6 / 140.6 | 109.7 | 58.1 |
| dotnet | inline | 5777.0 / 6137.3 | 1758.3 | 1790.7 |
| dotnet | offload | 115.8 / 179.5 | 1752.9 | 1821.6 |

## Tiny control: 3,200 iterations

This follow-up was selected after observing the long-work result. It is a separate
cohort, not a pure within-run size sweep. Its higher idle latency also cautions
against comparing absolute figures across cohorts.

| Runtime/configuration | Work placement | Ack p50 / p99 µs | Server batch wall ms | Server CPU ms |
| --- | --- | --- | --- | --- |
| neo-sync | idle | 115.0 / 234.7 | 111.6 | 8.8 |
| neo-sync | inline | 124.3 / 241.5 | 113.8 | 10.7 |
| neo-sync | offload | 120.9 / 228.7 | 112.9 | 11.6 |
| neo-concurrent | idle | 124.1 / 259.9 | 116.7 | 17.9 |
| neo-concurrent | inline | 122.9 / 230.4 | 114.6 | 20.0 |
| neo-concurrent | offload | 121.3 / 268.7 | 114.3 | 24.2 |
| go | idle | 136.4 / 255.9 | 118.1 | 18.0 |
| go | inline | 134.3 / 229.9 | 116.2 | 22.1 |
| go | offload | 121.8 / 211.5 | 113.4 | 21.4 |
| dotnet | idle | 121.7 / 191.8 | 117.4 | 55.2 |
| dotnet | inline | 122.0 / 206.9 | 119.0 | 59.4 |
| dotnet | offload | 116.1 / 242.7 | 117.7 | 59.9 |

The 200 µs intentional peer delay usually lets tiny work finish before input arrives.
This control asks whether offloading buys anything in that situation; it cannot
locate a general crossover threshold. In neoCLR sync, offload CPU rises from about
10.7 to 11.6 ms per batch; concurrent rises from 20.0 to 24.2 ms, while latency tails
are mixed. No preset or automatic scheduling threshold is selected.

## Correctness and profiling

Both cohorts pass 18 control invocations (neoCLR optimized and fail-fast UBSan,
both collectors/all placements; Go/.NET all placements) and 36 timed invocations.
Each timed cohort checks 10,368 acknowledgement rounds, including warmup. neoCLR
also checks parked roots, exactly-once resumption, worker results, released operations
and empty heap teardown; concurrent epochs actually start and finish. An initial
fixture bug checked byte-array length storage instead of payload (offset 8 rather
than 16); this was corrected before the retained passing cohorts. The failed report
and manual diagnostic are documented; no runtime fix was required.

Separate two-second macOS `sample` captures run 1,024 long offload rounds for each
configuration. Native stacks show compute on a worker while the owner spends much
of its time waiting for that round's job, after acknowledgement. Go shows compute
and runtime parking; .NET managed frames are largely unsymbolicated in this sampler.
Thread counts differ and waits include off-CPU time: do not compare sample counts
as CPU percentages. Profiled durations are excluded from the tables. No trace-based
causal claim or managed .NET CPU attribution is made.

## Architectural decision and next bounded slice

Retain owner-affine dispatch as a first-class requirement. Investigate bounded
worker-to-owner completion delivery so an owner can service several independent
operations instead of joining one job at a round barrier. Start with copied native
payloads and explicit lifetime/cancellation; test completion-during-park, exactly-once
settlement, queue saturation, shutdown and both collectors before considering managed
captures. Compare batching/cooperative slicing as alternatives; preserve an inline
path for work too small to justify handoff. This is the next investigation, not an
adopted replacement for .NET Task semantics or a new public API.

Then use paced arrivals independent of completion, multiple connections/apps and
mixed handler logic to measure sustainable capacity and tails with generated JIT
and Native AOT. Larger graphs, true shared capture semantics, UI deadlines, mobile
energy and broker protocol behavior remain separate gates. The current evidence
supports progress isolation, not a wholesale scheduler rewrite or GC-default switch.

## Primary-source comparison

.NET documents Task scheduling on its thread pool and configurable synchronization
models; this fixture explicitly adds owner affinity through SynchronizationContext.
Go separates goroutines, OS threads and execution resources; limiting GOMAXPROCS
does not limit total OS-thread count. Reviewed 28 October 2026:
[.NET tasks](https://learn.microsoft.com/en-us/dotnet/standard/parallel-programming/task-based-asynchronous-programming),
[SynchronizationContext](https://learn.microsoft.com/en-us/dotnet/api/system.threading.synchronizationcontext?view=net-10.0),
[Go runtime structure](https://go.dev/src/runtime/HACKING).
These implementation concepts motivate the comparison; they do not prove performance.

[Sources and reproduction](../../experiments/runtime-handoff/README.md).
Compressed raw evidence is indexed by decompressed SHA-256 in `artifacts.json`.
