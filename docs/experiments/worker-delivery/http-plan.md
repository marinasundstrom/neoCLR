# HTTP integration and fresh measurement gate

## Shared-service and profiling clarification (2026-10-28)

The author requires shared scheduling mechanisms for Task.Run and other scheduled
work, avoiding HTTP/I/O-specific policy unless evidence justifies it. The purpose
is predictable scheduler/GC/service interactions and attributable tuning. Keep
scheduler configuration and resources constant when comparing collectors. Extend
HTTP coverage to many endpoints mixing file reads/writes, downstream HTTP/socket
I/O and complex handlers, including interference with short requests.

Before completing this goal, profile execution-context memory and suspension/
resumption costs as well as queue behavior. The [profiling contract](../shared-scheduling/README.md)
separates native records, managed state/capture retention, thread resources,
cooperative handoffs, OS-thread switches, queue/lock/wakeup costs and GC/JIT
interference. Retain actual measurements and instrumentation overhead; a sizeof
inventory or throughput result alone does not close this gate.

Author direction, 28 October 2026: prove the candidate applies to HTTP before
eventual integration, and run a new benchmark after integration. Current native
delivery results establish neither HTTP improvements nor a production default.

## Prerequisite integration

Trace a real generated HTTP handler through submission, capture retention, worker
completion, owner delivery and response completion. Preserve shared object identity;
do not copy owner-heap closures into a separate heap to make the test pass. Audit
the compiled worker coordinator's admitted services and parked-root coverage before
allowing those handlers. An initially narrow handler contract must be explicit.
Qualify completion before/after parking, success/fault/cancellation, admission limits,
consumer-root pressure, disconnected clients, concurrent collection and shutdown.
No worker invokes owner-bound socket/async services or touches unaudited foreign
heap references. Keep cancellation settlement distinct from forced interruption.

## Audited boundary and next implementation gate (2026-10-28)

The current compiled callback bridge temporarily transfers the pool-owned arena
and static storage to a fresh invocation context under its heap gate. It rejects
parked/TLS roots and does not admit async/socket services. The separately compiled
HTTP worker experiment creates independent whole applications/heaps; it cannot
satisfy shared-capture Task.Run semantics. Copying owner closures to the pool would
also change those semantics.

Generated closure delivery is now [qualified](../../validation/worker-delivery-generated-20261028/README.md)
within that restricted bridge under both collector configurations. Worker
callbacks use synchronous fallback in the concurrent build: the borrowed pool
arena is not registered as the worker's owned heap. The owner does run concurrent
epochs. Actual concurrent marking of worker-accessed objects remains unqualified.
The next design slice must coordinate an existing HTTP owner heap with workers: register both active and
parked roots, retain closure/static identity, serialize or coordinate allocation
and collection, and keep socket/async operations owner-affine. Admission must
reject unsupported services before execution. Decide whether to extend the
existing compiler participant protocol or introduce an exclusive heap lease that
can safely coexist with parked owner roots; measure owner blocking as part of the
choice. A lease that blocks the I/O owner for the whole handler could erase the
benefit. No heap attachment or managed scheduling default changes are qualified by
this prerequisite.

The [exclusive native lease prerequisite](../../validation/heap-adoption-20261028/README.md)
now validates sequential transfer and marker join with both collectors. It remains
outside the generated worker bridge: host-bootstrap callbacks are not admitted by
the live-store-barrier compiler profile. Next qualify that restricted compiler path
before enabling any worker marker. Preserve static initialization, captured graph
identity, fault roots and native-service restrictions. Ownership alone does not
establish barrier coverage; pressure collections can also suppress epochs.

## Fresh benchmark after integration

Retain a build of the previous HTTP implementation as control and build the
candidate from matching compiler/SDK inputs. Use matched request logic, responses,
worker/admission limits, heap/GC policy and CPU/memory resources. Record hashes and
effective configuration. Native AOT is the distribution target; include JIT development
controls and both sync/concurrent collectors. Separate cold JIT/readiness effects
from warmed handler scheduling.

Compare I/O-only, CPU-heavy, allocation/shared-state and mixed downstream-I/O
handlers, including independent requests while a slow handler runs. Exercise
single/multiple connections and applications sharing resources, light load,
paced arrivals through saturation, cancellation and overload/backpressure. Avoid
using closed-loop throughput alone: retain offered/completed/error rates and
latencies at a common offered load, then sustainable capacity within predeclared
tail/error budgets. Include idle CPU and small-work controls to expose wake overhead.

Predeclare host budget, warmup, repetitions, trial lengths and acceptance/tail/error
limits before interpreting the cohort. Interleave baseline/candidate runs and retain
regressions. Measure readiness, throughput, p50/p95/p99 and maxima, CPU/request,
memory/high-water marks, queue residence, completion-to-resume and GC events.
Profile separately; traced runs are not headline timings. Inspect whether gains
come from fewer waits, different admission, altered work or GC policy. In particular,
attribute owner idle-check/epoch frequency: the native candidate increased concurrent
epoch starts and CPU in its accepted cohort; do not hide that cost behind lower
completion latency.

Update development HTTP measurement tables from that cohort; leave native fixture
results labeled separately and preserve published-release evidence. Adoption
requires correctness and a useful application tradeoff, not just a large synthetic
wake-latency gain. Website publication remains a separate manual operation.

The [restricted owned callback profile](../../validation/worker-owned-callback-20261028/README.md)
now supplies the compiler barrier/image-contract prerequisite and completes actual
worker marking epochs. Owner-affine services remain excluded from those callbacks.
The next HTTP step should reuse this qualified boundary or improve owner scheduling
without pretending the existing isolated heaps share captures. Evaluate the smallest
useful path first; a general simultaneous-mutator redesign is not an acceptance
requirement unless the chosen HTTP integration needs it.

## Bounded acceptance target (author clarification, 2026-10-28)

The author wants a useful integration that is "good enough for now", with further
optimization left for later. This supersedes treating every possible scheduler or
heap optimization as a prerequisite. Preserve shared captures, safe roots/barriers,
bounded admission, cancellation and shutdown; reject unsupported services explicitly.
Use the smallest real HTTP path that meets those contracts and measure both collectors.

Before declaring this goal complete, retain matching before/after HTTP runs for
small I/O handlers and heavier allocation/CPU handlers, including mixed traffic
where short requests compete with long work. Verify responses and errors, compare
throughput and p95/p99 at matched offered load, and record CPU/memory. Keep supported
.NET/other-runtime controls where comparable; do not claim a runtime ranking from
unmatched implementations. Investigate a substantial regression or clearly poor
performance in a representative case. Defer small improvements, advanced work
stealing, broad migration and ownership-metadata tuning unless they block safety
or an acceptable measured result. Record the achieved boundary and remaining work
rather than promising universally best performance.
