# HTTP throughput and execution continuity

Author-directed roadmap, 2026-10-31. This refines the active HTTP performance work
in the [platform roadmap](../platform-roadmap.md); it does not reprioritize unrelated
platform milestones. Research and proposed gates below are distinct from shipped
capabilities. No GC or execution-mode default is changed by this document.

## Outcome and baseline

The author wants direct request/response throughput competitive with established
modern runtimes, with sustained throughput in the thousands as an explicit goal.
Four-digit throughput is an initial floor, not a ceiling or a claim of current
capacity. The primary metric is successful **HttpServer requests/second at a stated
latency target and CPU/memory budget**. Report errors, timeouts and queue growth;
unbounded queuing cannot count as throughput progress.

The primary application uses `System.Web.Http.HttpServer`, without Neo.Web, DI,
configuration or logging setup. The new [primitive fixture](../../benchmarks/http-server/primitive/README.md)
is a starting point. Neo.Web receives a separate matched overhead study after the
primitive baseline is established. A combined framework benchmark image remains
useful for incremental binding comparisons, but is not the primitive startup baseline.

Use the same response bytes, client behavior, connection policy, hardware and
resource envelope when comparing HttpServer and ASP.NET Core. Start with plaintext;
then distinguish pre-serialized JSON from actual object serialization. Record all
runtime versions and settings. A real ASP.NET Core comparison is planned, not yet
performed here. Match connection-close behavior first; also measure native
keep-alive behavior separately as public persistence support is qualified.

One sequential Python consumer is a correctness/latency screen, not a capacity
measurement. Establish a load generator with spare capacity and report offered
versus completed load. Sweep concurrency, measure warm steady traffic, then add
cold routes and bursts. Choose numerical latency and memory gates with the reference
workloads; do not invent a cross-framework ranking from unlike configurations.

## Lessons from other runtimes

Primary documentation checked 2026-10-31. These mechanisms are references, not an
instruction to reproduce their entire architecture or tune neoCLR with their numbers.

| Runtime mechanism | Relevant interaction and implication |
| --- | --- |
| CoreCLR tiered compilation | Quick initial code or ReadyToRun gives execution a body while optimized code is produced in the background. A fast executable path matters as much as having a compiler worker. [Compilation settings](https://learn.microsoft.com/en-us/dotnet/core/runtime-config/compilation) |
| CoreCLR compilation scheduling | Tiered compilation can pause during startup-like activity, and exposes pending/jitted method counts. Instrument demand and queue delay before choosing a background-work budget. [Tiered compilation events](https://learn.microsoft.com/en-us/dotnet/fundamentals/diagnostics/runtime-tiered-compilation-events) |
| CoreCLR background GC | Generation-2 work overlaps execution, but foreground collections still suspend managed threads. Background does not mean pause-free. [Background GC](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/background-gc) |
| CoreCLR DATAS | Heap/collection budgets adapt to live data and throughput, with heap count also changing. This is a reference for workload feedback rather than a fixed CPU-count policy. [DATAS](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/datas) |
| HotSpot tiering, JDK 25 | Early compiled code collects profiles before more optimized compilation. Startup, profiling quality and eventual throughput are separate objectives. [Tiered compilation](https://docs.oracle.com/en/java/javase/25/vm/java-hotspot-virtual-machine-performance-enhancements.html) |
| HotSpot G1/ZGC, JDK 25 | G1 trades application CPU for shorter pauses. ZGC requires allocation headroom during concurrent work; its soft heap limit can be exceeded to avoid allocation stalls. CPU and memory reserves must be evaluated together. [G1](https://docs.oracle.com/en/java/javase/25/gctuning/garbage-first-g1-garbage-collector1.html), [ZGC](https://docs.oracle.com/en/java/javase/25/gctuning/z-garbage-collector1.html) |
| Go GC | Allocation assists expose GC debt on application paths. Idle marking uses spare CPU; a soft-memory-limit CPU limiter resists GC thrashing, potentially exceeding that limit. Go supplies GC/scheduler lessons, not a JIT comparison. [GC guide](https://go.dev/doc/gc-guide) |

Compiler/collector coordination also has a correctness dimension: JIT-provided
stack roots and optimized lifetimes affect tracing. Code publication, barriers,
safepoints and code-version lifetime therefore need explicit contracts before
introducing tier replacement or resuming execution during compilation. CoreCLR's
[root description](https://github.com/dotnet/docs/blob/main/docs/standard/garbage-collection/fundamentals.md)
is a useful comparison; neoCLR's current nonmoving collector and execution ABI
remain the implementation baseline.

**Inference for neoCLR:** optimize total request progress, not the utilization of
individual workers. Compiler queues, GC debt, ready requests and native I/O share
CPU/cache/memory bandwidth. Reserving application headroom and deferring optional
optimization may help, but collection cannot be deferred without accounting for
allocation headroom. JIT/code-cache/native compiler memory must be included in the
process budget separately from the managed heap. No universal worker percentage
is selected by this research.

## Bounded sequence and acceptance evidence

| Stage | Work and current boundary | Exit evidence |
| --- | --- | --- |
| 1. Establish the primitive baseline | Standalone HttpServer image; direct close-response plaintext, then JSON. Separate interpreter, eager/lazy JIT and AOT. Keep synchronous/concurrent policies explicit. | Repeatable sustained-load baseline with client headroom; exact settings, failures, throughput, latency distribution and memory. Matched ASP.NET Core run. |
| 2. Remove the largest steady-state costs | Profile allocation, root publication, span checks, response writes and task/scheduler work. Qualify existing owned-heap fast paths before considering defaults. | One attributable change per comparison; native ownership/corruption/lifetime tests and an HTTP-level improvement beyond observed noise. Retain slower cases. |
| 3. Improve transport and scheduling | Trace accept/read/write completion-to-resume, queue delays and owner work. Qualify public persistence and framing/body/cancellation lifetimes; do not treat private pipeline tests as a public contract. | Higher direct-server throughput under concurrent clients, bounded queues and acceptable tails; disconnect, timeout and shutdown tests. |
| 4. Keep warm traffic progressing during cold compilation | Current lazy JIT defers compilation but its requesting execution path waits. Evaluate scheduler-aware suspension, prewarming and a fast initial tier as alternatives. A compiler thread alone does not solve this. | Warm-route throughput and p99 remain acceptable while cold routes activate; measure compiler queue/wait and CPU. Safe roots, recursion, failures, publication and shutdown remain qualified. |
| 5. Coordinate concurrent services under load | Measure GC pauses, allocation waits, marker progress/cancellation, compiler demand and process headroom together. Test fixed budgets before adaptive policies. | Improvement at realistic offered loads, including saturation, CPU limits, allocation bursts and retained objects. Avoid starvation, runaway queues and memory growth. |
| 6. Bound framework overhead | Add Neo.Web routing, context, middleware, DI, query/body binding and typed results on top of the qualified primitive baseline. | Separate absolute throughput and per-request cost, with registration-time and request-time work distinguished. Keep descriptor/reflection work out of repeated execution where possible. |

Stages are a dependency-oriented sequence, not permission to ignore a stronger
profile. Transport or compilation stalls can take precedence when they prevent the
chosen workload from progressing. Keep each implementation slice bounded and update
this record with measured evidence. Sustained server performance, including the
thousands-of-requests goal, remains an open acceptance gate.

## Heavy-load experiments and diagnostics

Use a factorial eager/lazy × synchronous/concurrent matrix where supported, with
AOT as a compiled control and interpreter results reported separately. Include warm
routes while previously untouched routes compile; allocation/retention pressure;
I/O waits; constrained CPU; and multiple processes sharing a machine. Keep cold
readiness, first-request latency, warm throughput and tail latency separate.

Trace request admission/completion, ready-queue delay, I/O completion-to-resume,
compiler queue/compile/wait/publication, GC owner pauses and worker activity on a
common clock. Record CPU for the actual compiler/execution process, not merely its
launcher. Sample native/Rust/compiler allocations as well as managed live bytes.
These diagnostics and production-shaped load tests are next work, not capabilities
established by the short fresh-connection screen.

The [current investigation](../validation/neo-web-jit-layout-20261031/README.md)
retains the initial layout experiment, final measurements and limitations. It
archives an unselected static-layout candidate with unchanged generated code and
no decisive startup gain. It does not establish the server-throughput goal or
nonblocking concurrent compilation.

## Release direction following the screen

The author requests lazy JIT and paced synchronous GC as release defaults and
explicitly permits enabling experimental changes with demonstrated throughput
benefit. Include the owned-heap fast path in matched qualification rather than
excluding it solely because it is experimental. Concurrent GC remains research
into the current or alternative implementations; it is not the selected release
policy or a commitment to one future collector. Record enabled settings and target
qualification in the release record. This direction supersedes retaining eager or
stress-host defaults solely because broader production-capacity work is unfinished.
