<a id="possible-platform-directions"></a>

# Project direction

These are open areas of development and exploration. For what you can use today,
start with the [guides](../guides/). The repository's
[roadmap](https://github.com/marinasundstrom/neoCLR/blob/main/docs/platform-roadmap.md)
tracks priorities and implementation work; the ideas here are not release commitments.

## Concurrency and scheduling

The execution direction is JIT for development and Native AOT for publishing and
distribution. Both use the runtime’s GC and scheduling services. Concurrent GC is
the preferred direction, with adoption/default choices governed by correctness
and representative measurements. Scheduler improvements should benefit both
paths; JIT compilation and AOT code-generation costs remain separate concerns.

A new investigation explores Go-inspired lightweight execution contexts,
worker-local/global queues, work stealing and configurable parallelism. The goal
is better throughput, scheduling latency and resource use while preserving Task
semantics and supporting explicit await or future implicit suspension.

Existing owner-local async fibers and a private parallel worker experiment provide
a foundation. General context migration, nonblocking lazy-JIT waits and shared-heap
HTTP execution still need ownership and performance qualification with both sync
and concurrent GC. Adoption will be incremental and measurement-driven; this is
not an announced runtime capability or a commitment to replicate Go.

The first private execution-tracer slice records worker activity, queue transitions,
callback intervals and cancellation with bounded buffers and offline timeline
export. It reports lost events and requires an instrumented build plus per-pool opt-in:
tiny-job overhead is material, so normal builds compile out the recorder.
A second private macOS slice records owner-thread async suspension/resumption,
nested execution and abandonment, qualified with sync and active concurrent GC.
Shared native sessions now correlate async transitions with owner GC requests and
concurrent epoch lifetimes, including cancellation. Epoch lifetime is not a GC
pause measurement. Compiler/worker integration, real I/O and finer GC phases
remain planned; these fixtures do not establish production HTTP performance.
[Implementation and qualification](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/execution-trace/README.md).

The private local/global/stealing queue experiment now passes focused correctness
checks, but repeated timings show no broad advantage; FIFO remains the default.
[Measurement tables and limitations](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/queue-policy-20261028/README.md)
retain both short screens and longer controls. These are native-job measurements,
not HTTP or collector comparisons. Next work targets real I/O completion-to-resume
latency and coordination costs. A subsequent
[real TCP baseline and tables](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/io-resume-20261028/README.md)
now compare both collectors: long callbacks delay I/O observation even when
post-completion resumption stays short. Next investigate cooperative execution,
offloading and admission. These native fixtures do not establish HTTP capacity,
UI responsiveness or a collector winner.

The intended general-purpose scope also includes future desktop/mobile UI apps:
main-thread affinity, input/frame responsiveness and idle CPU/energy matter alongside
server throughput and backpressure. Competitive messaging bindings, including
RabbitMQ, are a future evaluation case; no binding, UI platform support or performance
parity is announced. JIT/AOT and both collectors remain application-level controls.
The direction includes optional workload-specific JIT/GC/scheduling tuning with
balanced defaults. Settings must account for actual handler CPU work, allocations
and waits; no universal high-load preset or new configuration API is announced.

The intended model is **Task.Run schedules; await suspends the awaiting execution**.
Task.Run returns the completion handle for independently scheduled work. A submitted
function can itself await; shared captures remain part of the intended contract.
A Task and its execution context have distinct lifetimes.

A development comparison uses one I/O owner plus one independent CPU job. For
roughly 6 ms of CPU work, offloading lowers median TCP acknowledgement latency:

| Configuration | CPU work on owner | CPU work offloaded |
| --- | --- | --- |
| neoCLR native fixture, sync GC | 5,772 µs | 130 µs |
| neoCLR native fixture, concurrent GC | 5,798 µs | 126 µs |
| Go 1.26.3, owner-affine fixture | 5,779 µs | 128 µs |
| .NET 10, custom owner context | 5,777 µs | 116 µs |

Batch time stays nearly unchanged; tiny-job benefits/costs are mixed. These are
common-peer closed-loop measurements, with different helper threads, collector
policies and implementation languages. They are not generated neoCLR JIT/AOT, HTTP
capacity, strict equal-core rankings or a general Task.Run qualification.
[Full tables, tiny controls, profiles and limitations](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/runtime-handoff-20261028/README.md).
An opt-in bounded completion candidate now wakes the owner directly from native
worker settlement, preserving existing shared pool roots and owner-only resumption.
A wait-heavy native/TCP fixture reports these accepted-candidate medians:

| Collector | Polling resume latency | Wake-driven resume latency |
| --- | --- | --- |
| Sync | 479 µs | 21 µs |
| Concurrent | 482 µs | 39 µs |

Batch duration falls 26.0%/17.4%, with CPU increasing 8.0%/24.4% (sync/concurrent).
These accepted-candidate results include a source-order correctness fix; earlier
cohorts remain archived separately. These are worker
return-to-owner-resume times, not HTTP request latencies. General managed captures
across the HTTP owner/worker boundary remain unqualified.
[Correctness, traces, profiles and all cohorts](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/worker-delivery-20261028/README.md).
A subsequent compiled Raven check passes shared graph/static mutation, rooted
closure handoff, collection, fault delivery and recovery across 64 owner resumptions
under both collector configurations (optimized and UBSan runtime builds). Owner
concurrent epochs run, but compiled worker callbacks still collect synchronously:
the restricted pool bridge lacks owned-heap registration. Concurrent worker marking
and HTTP owner-heap integration remain open.
[Generated closure qualification](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/worker-delivery-generated-20261028/README.md).
A separate native prototype now checks exclusive heap transfer and marker join.
Rebuilding ownership metadata costs about 1 µs for 128 blocks and 7.4–7.6 µs for
1,024 blocks in a short microbenchmark. It is not enabled for generated callbacks:
their compiler profile must first admit concurrent store barriers.
[Lease checks, cost table and limitations](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/heap-adoption-20261028/README.md).
Adoption requires proving the change in the real HTTP path and running a **new
HTTP benchmark after integration**, with both collectors and matched workload/resource
controls. Existing HTTP performance tables are not replaced by these native results.

[Investigation and benchmark plan](https://github.com/marinasundstrom/neoCLR/blob/main/docs/design/go-inspired-scheduling.md)

<a id="http-poc"></a>

## HTTP and networking

The [HTTP library](../features/web/) supports bounded, cleartext HTTP/1.1 exchanges.
Broader streaming, TLS, persistent connections and newer HTTP protocols would support
more applications. Each needs suitable resource ownership, cancellation and error
contracts.

[HTTP design and capabilities](https://github.com/marinasundstrom/neoCLR/blob/main/docs/http-capabilities.md)

## Text

[UTF-8 strings](../features/strings/) distinguish graphemes, scalars and bytes.
Further work includes normalization, richer text positions and language-sensitive
operations. These need to fit the existing text model and make scanning and allocation
costs understandable.

[Text design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/text-model.md)

## Contiguous memory views

Development APIs include `Slice<T>` and `ReadOnlySlice<T>` for borrowed memory,
and persistent `Memory<T>` and `ReadOnlyMemory<T>`. These views share storage
without copying and preserve checked bounds and readonly access. The development
[Raven API](/docs/memory-views.html) provides checked view operations, implicit
array conversions, persistent Memory/ReadOnlyMemory and an immutable UTF-8 byte
view. Native execution covers array-backed byte views, including borrowed indexing,
writable updates, readonly conversion, length and integer/Range windows. Other
element types and UTF-8 views remain interpreter-only. Persistent byte memory
now survives managed suspension and collection. Socket receive/send and deadline
APIs use persistent byte-memory windows with native completion ownership; shared
stream and HTTP body API migration remains pending. Development array range
syntax returns `Slice<T>`; `ToArray` requests a copy. String slicing retains its grapheme semantics.

[Design and implementation status](https://github.com/marinasundstrom/neoCLR/blob/main/docs/native-memory-views.md)

## Introspection

[Introspection](../features/introspection/) describes types and members;
[reflection](../features/reflection/) provides bounded invocation and construction.
Broader dynamic loading and native-executable reflection need explicit identity,
resolution and code-lifetime rules.

[Introspection design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/introspection-design.md)

## Function types: delegates evolved

[Function types](../features/functions/) use structural signatures for callable values.
Possible nominal specializations and a common Callable interface remain separate
proposals, with conversion and identity contracts to resolve.

[Function proposals](https://github.com/marinasundstrom/neoCLR/blob/main/docs/proposals/delegates-evolved.md)
· [Callable interface](https://github.com/marinasundstrom/neoCLR/blob/main/docs/proposals/callable-interface.md)

## Extended CLI metadata and structural types

The [current metadata format](../metadata/) connects native libraries, tools and the
runtime. Further CLI-derived representation and structural types should preserve
identity across those boundaries. Familiar file containers alone cannot provide
compatibility with ordinary CLR execution or tooling.

[Metadata design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/design/extended-cli-metadata.md)

## Collections

[Collection interfaces](../features/collections/) distinguish reading, replacement
and growth. Grouping, additional reductions and immutable or frozen providers are
possible extensions. A read-only view and an immutable collection need different
guarantees when another reference can mutate the data.

[Collection design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/collection-contracts.md)

<a id="time"></a>

## Time and globalization

[Time](../features/time/) and [globalization](../features/globalization/) cover civil
dates, instants, zones, calendars and culture information. Broader locale data,
zone rules and localization require both data sources and clear conversion policies.

[Time design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/date-time-design.md)
· [Globalization design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/globalization-design.md)

<a id="io"></a>

## Storage and streams

[Storage and streams](../features/files/) provide host and memory workflows.
Richer enumeration and asynchronous I/O would extend them. Pending operations need
clear buffer ownership and cleanup on cancellation.

[Stream design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/stream-design.md)

<a id="async"></a>

## Completion and execution

[Tasks](../features/tasks/) separate completion, Result errors and cancellation.
Runtime-owned suspension, continuation affinity and broader concurrent execution
remain open. These affect scheduling and resource lifetimes as well as language syntax.

[Task design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/proposals/task-model.md)

<a id="runtime"></a>

## Runtime and language integration

[Native compilation](../features/native-compilation/) is developing alongside the
interpreter, with ARM64 as the primary native target. Broader application coverage,
portability and sustained server execution remain areas of work. JIT, trimming and
hot reload need additional contracts; hot reload is not limited in principle to
interpreted execution.

[Execution design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/execution-architecture.md)

## Further module capabilities

[Modules](../features/modules/) form namespaces and own declarations. Module-private
access, re-exports and module-centered runtime discovery are possible extensions.
Independent module loading would need separate lifetime and dependency rules.

[Module assessment](https://github.com/marinasundstrom/neoCLR/blob/main/docs/design/module-system-assessment.md)

<a id="feedback"></a>

## Discuss an idea

Use [GitHub issues](https://github.com/marinasundstrom/neoCLR/issues) to describe a
use case, suggest an alternative or discuss a limitation. Detailed proposals and
implementation evidence belong in the repository, where their assumptions and
history can be reviewed together.

## SQL data access

The [SQL development slice](../features/sql/) introduces a small session and row-mapping
API with a local PostgreSQL provider. Password authentication, TLS, remote connections,
streaming and more providers remain open work. The older SQLite-first proposal is
historical input, not the current implementation specification.

## Generic bounds and ranges

A design review considers generic included/excluded/unbounded endpoints and consistent
Raven range syntax across .NET and neoCLR, followed by overflow-safe integer iteration.
These are proposals; existing slicing behavior is unchanged.

[Range design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/design/generic-bounds.md)
