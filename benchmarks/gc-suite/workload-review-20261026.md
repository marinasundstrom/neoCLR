# Workload coverage and rerun review — 2026-10-26

Author-directed review during the [lazy-JIT optimization investigation](../../docs/validation/lazy-jit-optimization-20261026/README.md).
The long-term choice should serve many workloads and retain optimization potential.
The intended lazy-JIT/concurrent-GC direction is conditional on useful gains, not on
winning one tiny response benchmark. This is a coverage review and measurement
policy; unimplemented cases below are explicitly future work.

## Current evidence and decisions

| Existing case | What it answers | Decision |
| --- | --- | --- |
| Plaintext/JSON fixed-work execution matrix | Runtime/collector regression and protocol costs; cold readiness versus warm execution | Retain. Rerun current lazy placement and both collectors with production codegen defaults; the fresh three-round HTTP cohort passes 148,032 responses. Keep close, sequential reuse and pipeline results separate. |
| Compute and allocation standalone samples | Costs outside HTTP; whether changes help more than one application shape | Retain. Fresh production-default eager/lazy pairs pass 24 invocations. Add varied object lifetimes/graphs when changing collector policy. |
| Retained objects plus fragmented input | Larger live set and overlapping pending reads | Retain as a controlled diagnostic. A fixed 20 ms header gap is an imposed delay, not a typical network distribution. Earlier three-round results use the explicit `speed` override. |
| New complex-handler offered load | Nested JSON, temporary records, checked computation, a rolling live set and overload behavior | Added and measured under production-default codegen, both collectors and eager/lazy with matched placement; keep all arrival misses and per-app tails. |
| New awaited dependency case | Handler suspension followed by processing; overlap of I/O and collection | Added and qualified with checked dependency accounting. The delayed one-byte socket exchange is not a database, HTTP client, pool or storage benchmark. |
| Cold and uneven two-app traffic | First-use interruption and process-level resource interference | Measured in the completed sequential queue. Two processes share a host; this does not enforce CPU/memory quotas or qualify shared-heap tenants. |
| Existing HTTP + PostgreSQL case | One query, materialization and JSON on a reused connection | Historical API baseline. Its generated row and one connection do not represent database load. Refresh only with matching libraries, provider admission and real table/pool pressure before using it to choose runtime defaults. |
| Historical Kestrel/Go/Node primitive comparisons | Comparable basic API layers under a matched old protocol/configuration | Preserve as historical. Rebuild peers and neoCLR together after the workload contract is settled; do not infer current relative performance from old scores. |

The current load sweep has 10-second warm windows and two rounds, not a long soak.
It brackets CPU/RSS around traffic and drain and records client dispatch lag.
Repeated rate windows in one process expose some sustained-state behavior, but do
not prove memory stability, capacity, or absence of thermal/desktop interference.
The generator shares the host; high-lag results cannot establish server saturation.
The review also adds and measures an ordinary-request control with no deliberate header
fragmentation, under both collectors in lazy mode. Keep its results separate from
the pending-read stress case; retain the downstream delay only for the service
variant. This changes one workload dimension rather than silently replacing the
original pressure measurements.

## TechEmpower review

Reviewed the official [test definitions](https://github.com/TechEmpower/FrameworkBenchmarks/wiki/Project-Information-Framework-Tests-Overview)
on 2026-10-26. They cover plaintext, per-request JSON serialization, database reads,
multiple queries, updates, Fortunes rendering and caching. Plaintext assumes
pipelining; JSON uses a fresh message object and real serialization. Routing,
framing and Server/Date headers matter. Database and rendering cases add work that
our tiny response fixtures omit. The [benchmark setup](https://github.com/TechEmpower/FrameworkBenchmarks/wiki/Benchmarking-Getting-Started)
separates application, database and load-generator roles, while allowing one-machine
development checks. The [repository](https://github.com/TechEmpower/FrameworkBenchmarks)
is marked archived on March 24, 2026. Its preserved specification remains useful;
we do not assume an active submission service or extract a new ranking from the
JavaScript-only results page.

Our current JSON payload is `Person`, not the specified message. The fixture does
not provide Server/Date headers. Its connection and handler limits, loopback client,
codegen controls and machine differ. Internal `ServePersistent` now supplies a
qualified persistence path, but public `Serve` retains its existing close behavior.
The old note's statement that no persistence implementation exists is historical.
These cases are not TechEmpower-compatible submissions or cross-machine rankings.

## Next benchmark updates, ordered by the optimization question

1. **Mixed warm and cold routes.** Use an already warm lightweight route alongside
   a newly reached complex route. Record each route's offered arrivals and response
   timeline during first compilation. This is the acceptance workload for safe
   scheduler suspension; compiling every route beforehand would conceal the defect.
2. **Lifetime and heap-pressure variation.** Vary retained live data, allocation per
   request and mutation rate independently. Include short-lived records, reusable
   buffers and larger object graphs. Report live bytes, collection phases, process
   CPU and useful completions, so lower allocation or fewer collections cannot
   silently substitute for equivalent work.
3. **Real database pressure.** Reuse the existing PostgreSQL provider, but add a
   populated table, randomized reads, multi-query requests and checked updates.
   Explicitly measure connection admission/pooling and database CPU/latency. Treat
   missing provider/concurrent-GC support as a runtime gap; do not disable auditing
   or replace I/O with a delay and call it a database result. Use a disposable DB.
4. **Sustained and burst demand.** After locating useful rates, confirm them with
   longer runs and overload/recovery cycles. Vary arrival shape and payload sizes,
   add slow peers and cancellation, and retain failures. A separate load host and
   enforced resource budgets are prerequisites for a strong capacity comparison.
5. **Matched external primitives and specification cases.** Qualify exact response
   contracts and public deployment configuration, then rebuild Kestrel/Go/Node
   counterparts under the same host budgets and traffic. A TechEmpower-shaped
   plaintext/JSON case is a separate versioned fixture, not a rename of current
   measurements. Fortunes/caching require their real operations and API support.

These updates should select the next bounded optimization rather than delay all
runtime progress until a complete web framework exists. Current profiles already
justify examining metadata lookup, first-call waits, warm frame/TLS work and
conservative stores. Validate each change against the relevant realistic case and
at least one contrasting workload, then broaden only when the impact warrants it.

## Rerun and publication rules

- Record runtime/compiler/library revisions, production versus test overrides,
  placement, collector settings, payload, connection policy and resource limits.
- Never combine `speed` and production-default codegen into one median. The newly
  added production preparer removes that override; the HTTP harness clears ambient
  optimization flags before applying its recorded recipe.
- Preserve old release results. New implementation or workload changes get fresh
  cohorts and identities; label previous scores historical rather than rewriting them.
- Keep instrumented profiles separate from timing. Count all offered arrivals;
  successful-response p99 is insufficient when many arrivals are missed.
- Report breadth of benefit and regressions. A mode with more removable overhead
  has headroom, not a proven future speedup. Sync remains a useful control and option.

See the [matrix tooling](matrix/README.md), [existing PostgreSQL benchmark](../postgresql/README.md),
[HTTP query baseline](../http-server/README.md#json-and-database-workloads), and
[TechEmpower comparison history](techempower-comparison.md).
