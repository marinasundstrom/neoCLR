# HTTP offered-load and worker-cost investigation — 2026-10-18

Development after Preview 16, continuing the author's request for honest performance
measurement and bottleneck discovery. This adds an executable load fixture and
controller, diagnoses worker/GC cost, and repairs a reproducible timeout race. It
does not promote a GC policy or claim production server capacity.

## Measurement contract

`Server.rvn` admits 1, 4 or 16 independent async accept/respond loops, selected by
its process argument. All serve the same bytes and use the same library. `/plain`
creates ten `x` bytes; `/bulk` creates 8 KiB. `/stop` responds before closing the
listener and joining cancelled accepts. Each trial starts a fresh process, warms
it, runs the fixed offered schedule, drains admitted client operations, and verifies
graceful shutdown and server/client completion accounting. Worker count is distinct
from client concurrency and does not mean one OS thread per worker.

`run.py` uses asyncio with scheduled arrival `start + index / rate`, independent
of response completion. It records every intended arrival. If the configured
client in-flight limit is full, that arrival is counted as
`generator_inflight_limit`, with no hidden queue and no replacement arrival. This
is a generator admission miss, **not an HTTP server rejection**. Network errors,
timeouts, malformed headers, wrong body bytes and unexpected status have separate
outcomes. Successful latency is recorded from intended arrival and actual start;
failures/misses remain in the offered denominator and are never successful samples.

Successful throughput counts only responses completed within the timed window,
divided by its full duration. Requests finishing during drain are counted separately
in the offered success fraction and latency distribution, not added to timed
throughput. p50/p95/p99 describe successful responses only; examine losses alongside
them. A low successful p99 with many misses is not a healthy server.

The provisional reporting target is **100% offered success, p99 <= 250 ms and
p99 generator dispatch lag <= 5 ms**. It is an investigation threshold, not a product
SLO. A passing point is a tested operating point, not the maximum sustainable rate.
Longer runs and stable retained memory are still required for capacity qualification.

The server's 16-exchange cap includes queued accepts. Thirty-two clients can wait
in TCP queues; this does not bypass that cap or necessarily generate a server
LimitExceeded error. Server limit/other-error counters are reported separately.

CPU is sampled from the server process's accumulated CPU time; RSS is sampled
approximately every half second. CPU/request divides window CPU by successful
window completions and includes unsuccessful server work. Sampling precision and
end-sample delay limit accuracy. Sampled peak RSS is neither exact peak RSS nor
managed live bytes. Resource sampling overhead stays enabled for all load points.
The generator runs on the same Apple M1/macOS host; dispatch lag is recorded but
does not prove absence of shared-host contention. Builds/profilers finish before
load measurement; desktop activity and host load remain recorded limitations.

## Offered-load results

AOT, ten-byte body, connection close, same Apple M1 host. These points use 16
client slots unless stated otherwise. Each row is one trial; the confirmation
report retains two separate 30-second trials at 200/sec. Full per-arrival reports
are retained losslessly as `.json.gz` beside browsable summaries, with SHA-256
identities. Failed/missed work is never removed from the denominator.

| GC policy / workers | Offered/sec | Seconds | Successful / offered | Completed/sec in window | Successful p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Stress / 16 | 5 | 30 | 150 / 150 | 5.00 | 118 ms |
| Stress / 16 | 10 | 30 | 286 / 300 | 9.03 | 1,890 ms |
| Stress / 1 | 100 | 15 | 1,500 / 1,500 | 100.00 | 10 ms |
| Stress / 1 | 150 | 15 | 2,039 / 2,250 | 134.93 | 124 ms |
| Paced / 16 | 100 | 30 | 3,000 / 3,000 | 100.00 | 8 ms |
| Paced / 16 | 150 | 30 | 4,500 / 4,500 | 150.00 | 8 ms |
| Paced / 16 | 200 | 15 | 3,000 / 3,000 | 199.93 | 36 ms |
| Paced / 16 | 250 | 15 | 3,242 / 3,750 | 215.13 | 79 ms |
| Paced / 16 | 300 | 15 | 3,150 / 4,500 | 208.93 | 86 ms |
| Paced / 16 | 400 | 15 | 3,141 / 6,000 | 208.33 | 86 ms |

Missing successes in these rows are client admission misses, not server rejections.
At higher rates the successful p99 remains modest precisely because the generator
has bounded admission: read both columns. The longer stress 10/sec trial needs
another 1.63 seconds to drain and misses 14 arrivals; its eventual 286 successes
must not be described as 9.53 successful completions/sec during the timed window.

The [stress concurrency sweep](results-20261018/stress-16-sweep.json) also includes
32 client slots at 20/sec: 49 successes, 81 client timeouts and 70 admission misses.
The server ultimately reports completion for those 81 timed-out clients, which does
not turn them into successful client responses. All final sweeps shut down cleanly,
with zero server limit/other errors. The fixture's server-side completion counter
is not a substitute for client-validated throughput.

Reports: [stress 30-second checks](results-20261018/stress-16-long.json),
[one-worker control](results-20261018/stress-1-control.json),
[paced sweep](results-20261018/paced-16-sweep.json),
[paced 30-second checks](results-20261018/paced-16-long.json),
[upper-rate sweep](results-20261018/paced-16-upper.json), and
[200/sec confirmation](results-20261018/paced-16-confirm.json).

Both 30-second 200/sec confirmation trials pass the provisional target: **6,000 of
6,000 responses each**, 199.97 completed/sec within each window, p99 **14.2 / 56.1 ms**,
and dispatch-lag p99 **0.99 / 1.09 ms**. The final request in each run completes during
drain. Sampled peak RSS is 27,776 KiB in both. This makes 200/sec the highest passing
offered rate tested here, not a production capacity qualification or a leak proof.
The observed variability reinforces keeping individual trials rather than only a best run.

These measurements locate a throughput plateau around 209–215/sec under paced GC
for this fixture and client cap; they do not prove a universal maximum. The 30-second
stress 5/sec run uses about 119 ms server CPU per completion, versus 4.6–4.9 ms for
paced 100/150/sec. Sampled RSS is about 27.1 MiB for 16 workers versus 12.0 MiB for
one worker. Different load levels, sampling and shared-host contention limit these
comparisons; retain the matched untimed profiles below for allocation attribution.

## Worker-count attribution

[Untimed profiles](results-20261018/worker-profiles.json) use the same compiled
application object and current library, 32 sequential requests plus stop. Each
policy is linked explicitly; `NEOCLR_GC_PACED` is experimental. Allocation windows
between accepted-connection closes 8 and 32 exclude the stop request. Whole-process
GC totals include startup and shutdown, and instrumentation affects timing.

| Policy | Workers | Collections attempted | Instrumented GC time | Peak live payload | Peak arena extent |
| --- | ---: | ---: | ---: | ---: | ---: |
| Stress | 1 | 101,282 | 231 ms | 9,304 B | 15,976 B |
| Stress | 4 | 102,763 | 601 ms | 25,497 B | 43,016 B |
| Stress | 16 | 107,863 | 2,864 ms | 87,766 B | 139,352 B |
| Paced | 1 | 1,690 | 9.7 ms | 9,223 B | 16,528 B |
| Paced | 4 | 1,680 | 14.5 ms | 25,444 B | 43,880 B |
| Paced | 16 | 1,751 | 57.1 ms | 87,702 B | 141,224 B |

All six allocation windows measure **338 objects and 16,880 payload bytes per request**,
despite their different worker counts and collection policies.

All six runs have zero GC failures and complete final reclamation. For stress GC,
1→16 workers increases collection attempts only about 6.5%, but GC time about
12.4×. This supports repeated scanning of the larger live worker graph as a major
cost. It is not an uninstrumented throughput ratio, and does not isolate every
collector subphase. Paced collection reduces frequency while retaining slightly
more arena extent; this evidence alone does not justify changing shipping defaults.

Reuse the [existing GC pacing design and ownership qualification](../../http-gc-pacing/README.md)
and [HTTP byte-ownership comparison](../../../docs/http-foundation.md#byte-ownership-and-lazy-materialization-direction--2026-10-14).
A .NET-style async handler should not require an OS thread per suspended exchange;
this investigation concerns neoCLR's implementation cost, not a proposed semantic
divergence from .NET. Keep collection policy and HTTP library improvements separate.

## Response timeout race

Native and interpreter network deadlines round a monotonic timestamp upward to
whole milliseconds. TimerStart instead adds its delay to an unrounded timestamp.
The stream producer's cancellation timer could fire before NetworkDeadline.IsExpired,
so OnContentRead classified a deadline-driven cancellation as a cancelled task.

The same 5 ms waiting-producer probe gave [0 cancellations in 500 stress-GC cases](results-20261018/deadline-before.json),
but [221 in 500 paced-GC cases](results-20261018/deadline-paced-before.json).
The faster path exposes the sub-millisecond ordering window. The fix schedules
producer cancellation with one additional millisecond, accounting for the network
deadline's upward rounding. Other deadline checks and caller cancellation precedence
remain unchanged. No clock, timer API or GC default is changed.

[After the fix](results-20261018/deadline-paced-after.json), all 500 paced cases
return TimedOut, with no cancelled tasks. [All 30 existing stream cases](results-20261018/streams-fixed.json)
pass in interpreter, JIT and AOT, including caller cancellation, source failures,
HEAD, ownership and short/full-block reads. This fixes a reproducible cause matching
the earlier symptom; the original failure did not retain a case index, so its exact
causal identity cannot be established retrospectively.

## Harness verification and earlier attempts

Three focused tests verify that slow service cannot shrink the offered denominator,
wrong body bytes count as failure despite valid status/length, and transport
failures stay in the counts. The first [harness smoke](results-20261018/smoke.json)
had a fixture bug: a binding pattern was used where a None case test was intended,
so accepted contexts were abandoned. Correcting the case test produced the
[second smoke](results-20261018/smoke-v2.json). Neither is a capacity qualification;
the failed run is retained. Source compilation attempts requiring syntax correction
are not performance measurements.

## Reproduction

Build Server.rvn with benchmark.py --prepare-only --server-source, using the
matching native bundle. `run.py --build BUILD/native --workers 16 --concurrency
1 4 16 32 --rates 5 10 20 --duration 10 --rounds 1 --output NEW_DIRECTORY` runs a
bounded exploratory sweep. Use `--executable app-paced` only after explicitly
relinking with NEOCLR_GC_PACED; retained link commands record that distinction.
Run profile-workers.py separately from timings. DeadlineProbe.rvn plus
deadline-probe.py provides the regression probe; the retained link/build reports
identify exact compiler, library, runtime and adapter inputs.


## Remaining performance candidates

The next bounded HTTP change to investigate is lazy queued-accept state:
`HttpServerExchange` currently constructs its decoder, 256-byte buffer and
cancellation state before it receives a socket. Sixteen suspended workers therefore
retain more objects for every stress collection to scan. Moving connection-only
state until acceptance could reduce that graph, but must preserve queued cancellation
and shutdown behavior. `AcceptCore` also allocates its rejection promise on the
successful path. These are observed code costs and proposed changes, not measured
improvements yet.

Native transport copies, response snapshots, parser representation and task/callback
allocation remain candidates for separate attribution. Keep the snapshot's mutation
isolation unless a deliberately different ownership API is designed and validated.
Further qualification needs longer runs, a separate load-generator host, bulk/JSON,
TLS, slow readers/writers and repeated memory-pressure/recovery cycles. This fixture
uses one request per connection; it does not establish keep-alive performance.


## Callback-path follow-up

The author clarified that `ServeOne` is the normal application entry point. The
results above use explicit `Accept`/`Complete` and remain mechanism measurements.
[The next comparison](../lazy-accept/README.md) uses `ServeOneServer.rvn`, including
an actual queued await in its handler, on both old and new libraries. Supply
`run.py --server-source benchmarks/http-server/load/ServeOneServer.rvn` to retain
the matching fixture identity. Several async serving loops provide bounded request
concurrency; one sequential awaited loop does not dispatch concurrent requests.

## Serve latency attribution

`--diagnostics` adds per-arrival connect, send, first-byte, EOF and client-close
timestamps. Every admitted or missed arrival records slot occupancy. These phases
are client observations: response wait includes server work, TCP queues and client
scheduling, not an isolated server processing timer. `analyze.py REPORT --output
SUMMARY` preserves per-second losses and the slowest requests alongside percentiles.
Instrumentation is opt-in so diagnostic runs remain separate from capacity runs.
See the [Serve investigation](../serve/capacity-20261018/README.md).
