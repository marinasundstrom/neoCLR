# Server-managed Serve callbacks — 2026-10-18

The author requested `Serve(handler)` so applications register an asynchronous
request callback and the server owns dispatch, connections and completion. This
implements that API, preserves ServeOne's single-exchange behavior, and compares
both paths under the same offered load. Development after Preview 16; no release
or GC-policy change is implied.

## Contract

Serve starts immediately and returns a server-lifetime task. It owns 16 reusable
lanes, each observing an exchange and its handler task, with an explicit collection
rooting those lanes until shutdown completes. Suspended handlers do not serialize
other requests. At capacity the server waits; excess clients remain in the TCP
backlog. This is cooperative asynchronous execution, not an OS thread per handler.

An open idle server is required. Duplicate Serve and mixed manual acceptance are
rejected. Pre-cancellation leaves the listener open. Request errors close the
exchange, increment FailedRequests and update LastRequestError, then serving
continues. Successful responses increment CompletedRequests. Only one error is
retained; no unbounded diagnostic list or implicit retry/error page is introduced.
Listener errors end the lifetime with Error; runtime Faults remain terminal.

Close and active caller cancellation stop acceptance and close connections. Unlike
ServeOne's existing early cancellation acknowledgement, Serve joins all started
handler tasks and disposes their late responses, including streaming content.
Close produces successful lifetime completion; caller cancellation produces a
cancelled lifetime task. Handlers must settle cooperatively; arbitrary work cannot
be forcibly cancelled. This is not graceful draining of HTTP responses, and blocking
callback code still blocks the dispatcher. See the [API guide](../../../api-docs/http-server.md)
and [.NET comparison/design](../../../docs/http-server-design.md#server-owned-serve-lifecycle--2026-10-18).

## Validation

`Probe.rvn` and `validate.py` use independent clients and explicit control signals.
All nine scenarios pass: Close, caller cancellation and synchronous handler-initiated
Close, each in interpreter/JIT/AOT. Tests cover:

- Pre-cancellation, duplicate Serve and mixed Accept/ServeOne rejection.
- Handler Error, cancelled handler, invalid response and malformed request, followed
  by successful recovery and exact diagnostic counts.
- Sixteen suspended handlers; a seventeenth request waits until one slot is released,
  then completes while the other fifteen remain suspended.
- Closing all connections while handlers remain pending; the lifetime must remain
  incomplete until those handlers settle. Fifteen late owned streams are disposed.
- A handler that closes the server before returning its task; its late owned stream
  is still observed/disposed before lifetime completion.

The existing 30 stream-response cases also pass across interpreter/JIT/AOT. These
retain ServeOne cancellation/ownership behavior. Four native API snapshot tests and
the load harness contract tests accompany the matching public reference snapshots.
Runtime Fault recovery, TLS load, configurable concurrency and graceful HTTP draining
are not claimed by this slice.

## Matched fixtures and measurement

`Server.rvn` calls Serve once, keeps its lifetime task, and independently waits for a
shutdown signal. `ServeOneServer.rvn` uses 16 application-managed loops. Both use the
same async handler: await a queued `Task.Run` operation, build the same ten-byte body,
return the response. The Task.Run is a controlled suspension/continuation workload,
not a recommendation to wrap ordinary asynchronous I/O in Task.Run.

Both fixtures use identical control listeners. After draining and validating the
stop response, the controller signals that listener, closes the HTTP server and
verifies lifetime/worker completion and response counts. Control connections are
excluded from request throughput. Allocation profiling sees 34 socket closes
(32 requests, HTTP stop, one control); its 24-request window ends before stop/control.

The existing [offered-load contract](../load/README.md#measurement-contract) applies:
independent arrival schedule, misses/failures retained, byte-validated responses,
intended-start latency, completions counted only within the timed window, separate
drain, CPU/RSS samples and clean shutdown. Same Apple M1/macOS machine, AOT, loopback,
HTTP/1.1 connection close, 16 client slots. Two short rounds alternate fixture order.
Builds, profiles and lifecycle tests finish before timing. Background desktop work
and shared-host contention remain limitations. Passing rates are tested operating
points, not production maximum capacity.

## Allocation profiles

Same compiler, library, runtime and native adapters; vary only the application entry
path. Each policy is linked explicitly. Paced GC remains experimental. All four
profiles finish with zero GC failures and complete final reclamation.

| Measure | Application ServeOne loops | Server-managed Serve |
| --- | ---: | ---: |
| Objects/request | 405 | 385 |
| Allocated payload bytes/request | 19,594 | 18,409 |
| Peak live payload, stress | 64,461 B | 45,916 B |
| Peak arena extent, stress | 100,032 B | 68,128 B |
| Peak live payload, paced | 64,261 B | 45,716 B |
| Peak arena extent, paced | 100,096 B | 69,768 B |

Serve removes about 4.9% of allocated objects, 6.0% of allocated payload bytes and
28.8% of peak live payload in this fixture. These are managed payload/arena figures,
not process RSS. Instrumented timing is diagnostic only; some profiling overlapped
lifecycle checks, and it is not used as a throughput score. The fixtures intentionally
include their supervision overhead; this is not an isolated cost for a single method.

## Short offered-load comparison

Two 10-second rounds, baseline then Serve in round one and reversed in round two.
All arrivals and failures remain in the retained raw reports.

| GC / offered rate | Entry path | Timed completions/sec, rounds 1 / 2 | Admission misses, rounds 1 / 2 |
| --- | --- | ---: | ---: |
| Stress / 10/sec | ServeOne loops | 10.0 / 10.0 | 0 / 0 |
| Stress / 10/sec | Serve | 10.0 / 10.0 | 0 / 0 |
| Stress / 30/sec | ServeOne loops | 13.6 / 14.2 | 149 / 142 |
| Stress / 30/sec | Serve | 27.5 / 26.0 | 10 / 25 |
| Paced / 250/sec | ServeOne loops | 227.9 / 230.0 | 206 / 185 |
| Paced / 250/sec | Serve | 248.8 / 249.8 | 12 / 1 |
| Paced / 350/sec | ServeOne loops | 223.8 / 230.0 | 1,246 / 1,184 |
| Paced / 350/sec | Serve | 333.0 / 349.9 | 170 / 0 |

Observed mean overload throughput is about **1.92×** under stress at 30 offered/sec
and **1.50×** under paced collection at 350 offered/sec. These are fixture comparisons,
not maximum capacity or statistical confidence intervals. All these overload points
except the second Serve 350/sec trial have misses and fail the provisional 100%
success target. The passing short Serve 350/sec trial has p99 about 12.2 ms; its first
round has misses and p99 about 94.6 ms, so it does not establish sustainable 350/sec.

Keep the counterexamples: Serve at only 10 offered/sec in round two has successful
p99 about 402 ms and generator dispatch-lag p99 16.7 ms, failing the latency/lag target
despite no lost requests. Its 30/sec trial has dispatch-lag p99 about 10.2 ms. The
shared host is not a tightly controlled latency environment, and these samples are
not silently excluded or replaced. Other per-trial latency, CPU, RSS and drain data
are retained alongside throughput.

## Thirty-second checks and limitations

| Offered/sec | Entry path | Validated / offered | Timed completions/sec | Successful p99 | Generator lag p99 |
| ---: | --- | ---: | ---: | ---: | ---: |
| 200 | ServeOne loops | 5,998 / 6,000 | 199.93 | 55.8 ms | 1.9 ms |
| 200 | Serve | 5,964 / 6,000 | 198.80 | 79.3 ms | 9.9 ms |
| 300 | ServeOne loops | 6,660 / 9,000 | 221.53 | 130.0 ms | 39.6 ms |
| 300 | Serve | 8,815 / 9,000 | 293.80 | 92.4 ms | 19.7 ms |

Every missing success in these rows is a client admission miss, not an HTTP server
rejection. All admitted responses validate, all runs shut down cleanly, and server
error counters stay zero. Neither version passes the provisional 100% success target
in these long checks. Serve performs worse on losses and latency at 200/sec in this
pair. At 300/sec it delivers about **32.6% more timed completions**, but substantial
generator lag means these are not controlled production-capacity measurements.
No healthy sustained maximum has been established.

At 300/sec, sampled server CPU per timed completion is 4.23 ms for ServeOne loops
versus 2.77 ms for Serve. Sampled peak RSS is 29,536 versus 12,896 KiB. These describe
the whole compiled fixtures, including supervision and generated code; they do not
isolate the cause of the RSS difference or establish a leak bound. Longer runs,
a separate generator host and latency/GC scheduling attribution remain necessary.

Evidence: [Serve profile](results-20261018/serve-profiles.json),
[ServeOne profile](results-20261018/serveone-profiles.json),
[comparison commands/order](results-20261018/comparison-commands.json),
[Serve long runs](results-20261018/paced-serve-long.json),
[ServeOne long runs](results-20261018/paced-serveone-long.json),
[lifecycle checks](results-20261018/final-validation.json), and
[existing stream checks](results-20261018/streams-validation.json), and
[API rendering/snapshot evidence](results-20261018/api-validation.json).
All eight short-run reports (`stress|paced-serve|serveone-1|2.json`) are retained in
the same directory. Summary files link complete lossless gzip reports with SHA-256
checks. Matching builds, library source hashes, paced link commands and prepare
reports identify every input. No measured source changed during timing. Early
compile failures and the early smoke are retained separately; the final comparisons
use the explicitly rooted lane implementation.

## Reproduction

Build a matching System.Web bundle with `scripts/prepare-http-foundation-bundle.py`.
Compile both fixtures and Probe with `benchmark.py --prepare-only --server-source`.
Run `serve/validate.py --bundle BUNDLE --build PROBE/native --output REPORT`.
Run `load/profile-workers.py --build BUILD/native/build.json --output NEW_DIRECTORY
--workers 16 --control-stop` separately from timing. The profiling flags do not
belong in timed executables.

For load, pass `load/run.py --server-source FIXTURE --control-stop --workers 16
--concurrency 16 --rates ... --duration ... --rounds 1 --build BUILD/native --output
NEW_DIRECTORY`. Explicitly relink the same guest object with `-DNEOCLR_GC_PACED`
for the experimental paced control; retained commands identify each executable.

## Subsequent sustained-rate investigation

[Phase attribution, native profiles and longer repeats](capacity-20261018/README.md)
retain the earlier failures, distinguish the author's activity change, and qualify
400/sec locally under paced GC with two 120-second trials. The separate 500/sec
check exposes CPU saturation and admission misses. These are fixture/policy-specific
points, not an implementation speedup or a production maximum.

## Updated allocation, async I/O and execution-owner investigation — 2026-10-23

[Current staged counters, native profiles and single/two-process loads](investigation-20261023/README.md)
separate unnecessary work from allocation volume and establish the current native
owner-thread limitation. Includes explicit async I/O/threading follow-up criteria.
