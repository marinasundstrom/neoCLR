# Lazy queued accepts and async ServeOne measurement — 2026-10-18

Development after Preview 16, continuing the author's HTTP memory/performance
investigation. The author clarified that `ServeOne` with an async callback is the
usual application path. This comparison therefore uses the same callback fixture
on both library versions. Earlier [explicit-accept measurements](../load/README.md)
remain mechanism evidence, not interchangeable callback-path capacity figures.

## Change and preserved behavior

`HttpServerExchange` now allocates its request decoder and initial receive buffer
when the accepted connection can first read HTTP bytes, after TLS authentication
when applicable. Queued accepts retain their cancellation sources, registrations,
completion promises and queue links, but no parser or receive storage. The receive
buffer still starts at 256 bytes and grows to 4 KiB on incomplete parsing. Successful
`AcceptCore` calls no longer allocate a rejection promise they will never use.

No public API, snapshot ownership, admission bound, GC policy or shutdown/cancellation
contract changes. The existing .NET comparison in
[HTTP server design](../../../docs/http-server-design.md) remains applicable: this
changes internal allocation timing, not asynchronous handler semantics. The cost
is a nullable parser state and initialization check at receive time. Cancellation
state intentionally remains eager because a queued request must stay cancellable.

## Fixture and measurement

[ServeOneServer.rvn](../load/ServeOneServer.rvn) starts 1, 4 or 16 async serving loops.
Each loop awaits `ServeOne` with its own handler. The handler awaits `Task.Run` of
a trivial queued operation, then returns a ten-byte or 8 KiB body. This measures
actual suspension/continuation work without imposing an artificial timer or claiming
real database/network work. The measured load below uses ten-byte bodies, 16 loops
and 16 client slots. Each connection carries one request and closes.

An awaited loop is sequential; multiple loops provide bounded concurrency. `Accept`
can also be used asynchronously, but leaves context processing and dispatch to the
application. The callback fixture includes different machinery and object lifetimes
from the earlier explicit-accept fixture; do not derive the cost of ServeOne by
subtracting their unrelated throughput numbers.

`ServeOne` starts one exchange when called; its returned task represents completion
of acceptance, handling and sending. Awaiting that task suspends only its serving
loop, not the other loops. Discarding tasks in an unrestricted loop does not supply
concurrency supervision: it can fill the 16-exchange limit and keep generating
rejections without yielding. The author proposed independence from the main loop;
a higher-level `Run(handler)` owning admission, outcomes and shutdown would express
that naturally. Such an API is a proposal, not part of this optimization.

The author further clarified the intended normal application model: dispatch a
callback returning `Task` and continue accepting, with the server supervising
completion, errors and shutdown. The bounded serving loops here are benchmark
scaffolding, not a recommendation that every application implement its own dispatcher.
This comparison does not change the current full-exchange completion contract.

Use the existing [offered-load contract](../load/README.md#measurement-contract):
fixed arrivals independent of completions, all client misses/failures retained,
fully validated bodies, separate timed completions and drain, intended-arrival
latency, sampled CPU/RSS, clean shutdown and server/client accounting. This is
same-host Apple M1/macOS AOT loopback evidence, not a production capacity rating.
GC defaults remain unchanged; paced collection is an explicit experimental control.
Two matched rounds alternate baseline/candidate order. Builds, lifecycle checks and
allocation profiling finish before timed loads. Background desktop activity remains
a limitation. Ten-second points are exploratory; do not imply sustained qualification.

## Allocation attribution

Same fixture, compiler, runtime and host adapters; only System.Web differs. Untimed
profiles use 32 requests plus shutdown, with a 24-request steady-state allocation
window. All 12 callback profiles (1/4/16 loops × stress/paced × baseline/candidate)
finish with zero GC failures and complete final reclamation.

| Callback-path measure | Baseline | Candidate | Change |
| --- | ---: | ---: | ---: |
| Allocations/request, all profiles | 410 | 405 | −1.22% |
| Allocated payload bytes/request, all profiles | 20,299 | 19,562 | −3.63% |
| Peak live payload, stress / 16 loops | 73,988 B | 62,948 B | −14.92% |
| Peak arena extent, stress / 16 loops | 111,952 B | 97,960 B | −12.50% |
| Peak live payload, paced / 16 loops | 73,788 B | 62,748 B | −14.96% |
| Peak arena extent, paced / 16 loops | 112,552 B | 98,264 B | −12.69% |

These are managed payload/arena measurements, not process RSS. One-loop peak live
payload is unchanged; the benefit grows with queued loops. Whole-process counters
include startup/shutdown. Instrumented GC time is diagnostic and is not used as a
throughput score; lifecycle validation overlapped part of these untimed profiles.
Both allocation changes are measured together, so this does not isolate each one's
individual contribution. The explicit-accept candidate profile separately measures
333 objects and 16,143 bytes/request versus the earlier 338 / 16,880.

## Matched offered-load results

Two 10-second rounds, same callback fixture and 16 client slots. Ranges retain both
runs; successes include drained responses, while completed/sec counts only the timed
window. Misses are generator admission misses, not HTTP server rejections.

| Policy / offered rate | Version | Successful / offered, each round | Timed completions/sec | Successful p99 |
| --- | --- | --- | ---: | ---: |
| Stress / 8/sec | Baseline | 80/80; 80/80 | 8.0; 8.0 | 88.7; 72.8 ms |
| Stress / 8/sec | Candidate | 80/80; 80/80 | 8.0; 8.0 | 69.3; 72.1 ms |
| Stress / 20/sec | Baseline | 147/200; 152/200 | 13.1; 13.7 | 1,238; 1,224 ms |
| Stress / 20/sec | Candidate | 157/200; 163/200 | 14.3; 14.7 | 1,117; 1,090 ms |
| Paced / 150/sec | Baseline | 1,500/1,500; 1,500/1,500 | 150.0; 150.0 | 5.9; 6.7 ms |
| Paced / 150/sec | Candidate | 1,500/1,500; 1,500/1,500 | 149.6; 150.0 | 24.2; 6.0 ms |
| Paced / 250/sec | Baseline | 2,493/2,500; 2,483/2,500 | 249.2; 247.5 | 67.5; 81.1 ms |
| Paced / 250/sec | Candidate | 2,500/2,500; 2,500/2,500 | 250.0; 250.0 | 12.7; 27.8 ms |

Stress overload completion rate improves about **8.2%** using the mean of the two
rounds (13.4→14.5/sec). Both versions still overload at 20 offered/sec, so neither
rate is a healthy capacity claim. The candidate's first paced 150/sec trial has
worse latency and finishes four requests during drain: this is not an across-the-board
latency improvement. Both 150/sec versions pass the provisional reporting target.

A subsequent **30-second 250/sec check passes for both versions**: 7,500/7,500
validated responses each, 249.97 completions/sec within each timed window, p99
13.8 ms baseline / 16.2 ms candidate. Server CPU/completion is 3.62 / 3.41 ms, but
one local pair does not establish a stable CPU improvement. Sampled peak RSS is
29,312 / 29,360 KiB: the reduced managed live graph does **not** imply reduced process
RSS here. The short baseline misses therefore do not establish a capacity advantage
for the candidate. Neither version's maximum throughput is identified by these runs.

Evidence: [baseline profile](results-20261018/serve-baseline-profiles.json),
[candidate profile](results-20261018/serve-candidate-profiles.json),
[explicit-accept candidate profile](results-20261018/profiles.json),
[comparison order/commands](results-20261018/comparison-commands.json),
[baseline long check](results-20261018/paced-baseline-long.json),
[candidate long check](results-20261018/paced-candidate-long.json), and
[lifecycle validation](results-20261018/validation.json).
All eight short-run reports are retained as `stress|paced-baseline|candidate-1|2.json`
in that directory. Every summary links its complete losslessly compressed raw report
with raw/compressed SHA-256 identities. Build/prepare reports and explicit paced link
commands identify compiler, runtime, native sources, guest assemblies and libraries.

## Next work

Keep server-owned asynchronous callback dispatch as a separate design task, as the
author clarified. Its acceptance task and handler-completion tracking need explicit
contracts for admission, errors, cancellation and shutdown; silently shortening the
existing `ServeOne` completion task would change behavior.

For further performance attribution, the callback profile retains allocation-site
rows. Array storage helpers and collection iteration remain substantial sources:
`ArrayList.GetIterator` accounts for 28 exclusive objects/request in this fixture.
These are candidates for investigation, not evidence that all such allocations can
be removed safely. Separate framework/fixture overhead from application work, retain
matched callback-path measurements, and qualify longer/separate-host/TLS workloads
before making broader capacity claims.

## Validation and reproduction

`QueueProbe.rvn` checks the 16-exchange limit, cancellation of queued middle/tail/head
nodes, replacement admission, native pending-accept and promoted-successor
cancellation, pre-cancellation, listener recovery, shutdown of 16 pending accepts,
and rejection after close. An independent client validates the recovered response.
All checks pass in AOT, JIT and interpreter. The first compile omitted a namespace
import; that failed build is retained and is not a performance measurement.

`validate.py` also reuses the nine existing headers-first body cases in each mode:
fixed/chunked bodies, withheld body bytes, truncation, invalid framing, quota,
cancellation, close and prohibited bodies. All 27 pass. No broad suite or website
build is required for this internal allocation change. TLS-specific load and
handshake-failure qualification remain outside this measurement.

Build the library with `scripts/prepare-http-foundation-bundle.py`, using the prior
validated deadline-fixed bundle as baseline. Build both versions of
`ServeOneServer.rvn` with `benchmark.py --prepare-only --server-source`. Run
`profile-workers.py --build BUILD/native/build.json --output NEW_DIRECTORY`
separately from timing. Relink the same guest object with `-DNEOCLR_GC_PACED` only
for the explicitly labelled paced control; retained link commands identify inputs.
Run `load/run.py --server-source benchmarks/http-server/load/ServeOneServer.rvn`
with the selected build and rates. The retained comparison commands give exact
options and order.

For validation, build `QueueProbe.rvn` the same way. The body consumer combines the
imports and declarations from `docs/experiments/http-body-stream/Common.rvn` and
`server/Main.rvn` into one temporary source, then builds with the same helper.
`validate.py --bundle BUNDLE --queue QUEUE/native --body BODY/native --output REPORT`
runs both consumers across all three modes. Retained build reports hash the combined
source, and validation hashes the original source files.
