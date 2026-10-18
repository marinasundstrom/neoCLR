# Return to actual Serve loads before concurrent-GC integration

The author redirected the next step toward HTTP-server loads after the synthetic
CPU-contention evaluation. Further synthetic rate sweeps were set aside. This
checkpoint re-establishes the actual asynchronous Serve/Go comparison and identifies
where concurrent collection must integrate. It does not install the concurrent POC
in the HTTP server or attribute an HTTP speedup to it.

## Measurement contract and provenance

The retained, previously qualified Serve guest object is relinked against current
native runtime sources with the existing experimental paced **synchronous** collector.
Its Raven source matches the current `Server.rvn` byte for byte. The compiler/library
cohort is retained, not rebuilt with every newer language/runtime feature. The Go
fixture is freshly built. Source, object, archive and executable identities are in
[native build](evidence/native-build.json), [Go build](evidence/go-build.json),
[guest provenance](evidence/guest-provenance.json) and the lossless
[original guest build](evidence/original-guest-build.json.gz).

The existing `compare-go.py` runs 300 and 400 requests/sec for 30 seconds per point,
with two rounds reversing runtime and rate order. Each server starts fresh with
separate warm-up. The fixture uses real loopback TCP, HTTP/1.1 connection close,
16 Serve lanes, 64 client slots, a queued Task.Run in the asynchronous handler,
and a byte-validated ten-byte response. Go uses GOMAXPROCS=1 and GOGC=100. This is a
matched application contract, not identical internal work or a Go capacity search.

Arrival times are independent of completions. Client admission misses and failures
remain losses; latency includes the scheduled-arrival delay. Only in-window responses
count toward timed throughput; drain is separate. The existing provisional target
requires all offered requests to succeed, offered p99 <=250 ms and generator lag
p99 <=5 ms. It is not a product SLO. CPU, RSS and clean shutdown accounting are
retained. No synthetic competitors, compilation or profiling runs overlap timing.
Same-host generator/desktop scheduling remains a limitation.

## Actual HTTP results

| Runtime / round | Offered req/s | Timed completions/s | Missed arrivals | CPU ms/completion | Offered p99 ms | Generator lag p99 ms | Target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| neoCLR 1 | 300 | 299.97 | 0 | 2.231 | 104.04 | 0.87 | Pass |
| neoCLR 1 | 400 | 400.00 | 0 | 2.273 | 37.49 | 0.52 | Pass |
| neoCLR 2 | 400 | 390.77 | 238 | 2.336 | 255.32 | 1.11 | Fail |
| neoCLR 2 | 300 | 299.70 | 8 | 2.272 | 132.10 | 2.69 | Fail |
| Go 1 | 300 | 300.00 | 0 | 0.181 | 2.87 | 0.69 | Pass |
| Go 1 | 400 | 400.00 | 0 | 0.179 | 2.96 | 0.63 | Pass |
| Go 2 | 400 | 400.00 | 0 | 0.180 | 6.36 | 2.81 | Pass |
| Go 2 | 300 | 300.00 | 0 | 0.196 | 3.17 | 0.65 | Pass |

Both neoCLR first-round points pass, but neither repeats without admission loss.
The missed arrivals are `generator_inflight_limit`: the client could not admit
another request within its 64-slot cap. They are unsent load, not successful server
responses or server-generated errors. Every admitted response validates, every
server exits cleanly, server failure counts are zero and completion accounting
matches. The native host also checks its final heap is empty. Go validates all
42,000 offered requests. neoCLR validates 41,754 of 42,000 offered arrivals; some
responses complete during drain and are excluded from the timed rates.

CPU cost remains approximately 2.23–2.34 ms/completion for Serve versus 0.18–0.20 ms
for Go in this contract. No optimization was applied in this slice and no new
speedup, maximum rate, or repeatable 300–400/sec operating point is established.
Generator p99 alone cannot attribute the stalls to the server, GC or host scheduling;
next instrumentation must correlate those events rather than infer causation.

[Comparison commands/order](evidence/comparison.json) and complete lossless reports
are retained with compact summaries: [neoCLR 1](evidence/neoclr-1.json),
[neoCLR 2](evidence/neoclr-2.json), [Go 1](evidence/go-1.json),
[Go 2](evidence/go-2.json). The initial launch stopped before measurement because
the relink directory lacked the harness-required `app.dll`; the matching retained
DLL was copied and hash-checked before the successful comparison. That
[packaging failure](evidence/initial-packaging-failure.json) is not a timed trial.

Reproduce after reconstructing the native link command and matching guest artifacts
from the build evidence:

```sh
python3 benchmarks/http-server/serve/go/build.py --output target/http-go-new
python3 benchmarks/http-server/serve/compare-go.py \
  --neoclr-build target/http-native-new --go-build target/http-go-new \
  --output target/http-compare-new --rates 300 400 --duration 30 --rounds 2
```

## Concurrent integration work identified

The known-C and generated-callback experiments are insufficient for this Serve path:

- `neoclr_scheduler_submit_managed_v1` creates/releases roots while queuing work;
  `neoclr_scheduler_acquire_v1` transfers ownership into a delivery ticket, and
  `neoclr_scheduler_complete_v1` releases it. These are existing POC invalidation
  events, not read-only callbacks that can automatically preserve a snapshot.
- `neoclr_runtime_async_pump_v1` parks the entry stack, creates callback fibers,
  resumes/parks their roots and reaps expired owners. Resumption can expose a
  previously conditional root, so its invalidation cannot simply be removed.
- Idle/pending scheduler acquisition calls `neoclr_gc_idle_v1`, in addition to
  generated paced boundaries and rooted allocation-pressure fallback. Replacing
  only eager generated safepoints would leave this collection path untouched.
- HTTP host shutdown tears down async/socket/task/TLS ownership and verifies an
  empty final heap. A concurrent adapter must cancel/join any worker before context
  or heap destruction and preserve this error and reclamation behavior.

Next bounded implementation: an isolated HTTP adapter that observes capture/tracing
at these boundaries while synchronous reclamation remains authoritative. Measure
snapshot acceptance, invalidation, completion and capture-limit rejection on real
Serve requests; do not suppress root/lifetime transitions to improve the numbers.
Then qualify actual concurrent result application and synchronous pressure fallback
with byte-validated requests and joined shutdown before timing a candidate.

Required scenarios include normal requests, the existing 16-lane saturation case,
malformed input/handler errors, close/cancellation while handlers are suspended,
and memory pressure. Use the existing focused Serve lifecycle tests and native GC
oracles; do not infer coverage from HTTP throughput alone. Once qualified, compare
sync/concurrent/Go with the same source, transport contract, rates and offered-load
accounting, then examine CPU demand during real request bursts. These are next
steps, not implemented support or a new collector selection option.


## Follow-up: tracing now reaches the real scheduler

The [HTTP shadow adapter](../../../../docs/experiments/concurrent-gc-http/README.md)
implements the first tracing-only integration step above. Real requests, malformed
input recovery and concurrent peers pass with synchronous reclamation authoritative.
No background result application or timed concurrent-Serve comparison is enabled yet.

The later [whole-path audit](../../../../docs/experiments/concurrent-gc-http/REVIEW.md)
confirms this retained guest still contains the older TaskQueue batch implementation.
These native relink measurements do not include every current managed improvement;
refresh the full cohort before making a current-runtime capacity claim.
