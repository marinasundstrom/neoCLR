# Matched Go net/http baseline — 2026-10-18

The author reaffirmed that comparison with a web server/application in Go is the
benchmark objective. This fixture provides that reference alongside neoCLR's Serve
path. It does not replace the historical [32-request pilot](../../go/main.go), which
uses a different workload and is not a sustained-load comparison.

## Contract and differences

Both fixtures run on the same Apple M1/macOS host after the author stopped side
activities. They return ten `x` bytes, allocate the body per request, await a queued
operation, use direct handlers without routing middleware, close each HTTP/1.1
connection, and enforce a 16-connection budget. Go suppresses its automatic Date
and Content-Type headers to match neoCLR's two response headers. An independent
control listener signals shutdown only after the controller has drained and
validated responses. Completed connections exclude `/stop`; control sockets are
not counted. The shared validator checks status, framing and every body byte.

Go uses the installed **go1.26.3 darwin/arm64**, compiled with `CGO_ENABLED=0`, and
runs with **GOMAXPROCS=1, GOGC=100, no GOMEMLIMIT override**. This controls Go's
simultaneous Go-code execution budget; it is not OS CPU affinity or a claim that
the process has only one OS thread. neoCLR uses its owner-thread AOT Serve fixture
and explicitly experimental paced GC. Its default stress-GC control remains
separate. Neither framework's default production configuration is being ranked.

Go's [Server.Serve](https://pkg.go.dev/net/http#Server.Serve) owns connection service
goroutines. The fixture wraps its listener to bound accepted connections; a focused
test checks admission blocking, exactly-once close accounting and shutdown wakeup.
The [GOMAXPROCS contract](https://pkg.go.dev/runtime#GOMAXPROCS) defines the execution
budget. The installed toolchain's `src/runtime/debug.go` and `src/net/http/server.go`
were also inspected. Go's OS-default listen backlog and HTTP/parser implementations
differ from neoCLR's explicit backlog of 64; the shared client budget is 64. Header
limits/timeouts/error behavior are not claimed equivalent beyond this valid GET
workload. Go's goroutine/channel operation and Raven's Task.Run express analogous
queued work, not identical instructions or allocations.

The common controller offers arrivals independently of responses, keeps every miss
in the denominator, validates all admitted responses, separates timed completions
from drain, and reports offered latency, generator lag, server CPU and sampled RSS.
It alternates neoCLR→Go then Go→neoCLR and reverses rate order in round two. Both
rates use 64 client slots and identical provisional targets: 100% offered success,
p99 <=250 ms and generator-lag p99 <=5 ms. Build/profiling/test work finishes before
measurement. Shared-host activity and Python generator limits remain limitations.

## Reproduction

Build with `python3 benchmarks/http-server/serve/go/build.py --output NEW_BUILD`.
Run the limiter test with `go test benchmarks/http-server/serve/go/main.go
benchmarks/http-server/serve/go/main_test.go`.

`compare-go.py --neoclr-build NEO_BUILD/native --go-build GO_BUILD --output NEW_RESULTS`
runs the default two alternating rounds at 400/500 offered/sec for 30 seconds each.
The controller records exact commands, environment, ordering, build/source/executable
hashes and all outcomes. Individual Go runs use `load/run.py --runtime go` with the
Go build manifest; this changes artifact identity only, not request validation or
load semantics. Go's sustainable maximum is not inferred from equal offered rates.

## Alternating matched results

Two 30-second trials per runtime/rate, with reversed runtime/rate order in round two.
Every admitted response validates, server error counters are zero, and all runs
shut down cleanly. Missing responses below are client admission misses.

| Runtime / GC | Offered/sec | Timed completions/sec, 1 / 2 | Misses, 1 / 2 | Offered p99 ms, 1 / 2 | Server CPU ms/completion, 1 / 2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| neoCLR / paced | 400 | 399.97 / 399.93 | 0 / 0 | 85.99 / 78.02 | 2.440 / 2.435 |
| Go / default | 400 | 400 / 400 | 0 / 0 | 5.31 / 5.90 | 0.273 / 0.283 |
| neoCLR / paced | 500 | 408.40 / 409.53 | 2686 / 2652 | 159.86 / 166.30 | 2.447 / 2.440 |
| Go / default | 500 | 500 / 500 | 0 / 0 | 4.64 / 5.68 | 0.238 / 0.231 |

At matched 400/sec, neoCLR uses about **8.6–8.9× the sampled server CPU per
completion**. Both deliver the offered rate, so equal throughput here does not imply
equal headroom. Go's offered p99 is also much lower. At 500/sec, neoCLR loses
5,338 of 30,000 offered arrivals across the two trials while Go validates all 30,000.
This reproduces the [neoCLR CPU boundary](../capacity-20261018/README.md), rather than
an order-dependent win from one selected run.

Sampled peak RSS is 12,864–12,896 KiB for neoCLR and 15,072–15,504 KiB for Go in
these matched trials. This includes code/runtime/heap/process overhead and is not
bytes per request, allocation count, retained managed payload or a leak comparison.
CPU per request varies with offered rate, runtime scheduling and uncontrolled CPU
placement/frequency; the ratio above compares only matching rates. No CPU affinity
was applied, and no exact instruction-cost or universal runtime ratio is inferred.

## Higher-rate Go lower bounds

Two additional 30-second trials at 2,000 offered/sec validate **60,000/60,000 each**,
with offered p99 **6.10 / 7.16 ms** and generator-lag p99 **2.95 / 4.47 ms**, passing
the same provisional targets. These shorter checks establish a tested lower bound,
not Go's sustainable maximum. Generator lag is already near the 5 ms criterion in
one run while Go's sampled CPU remains low; a faster/separate load generator is
needed before assigning Go an upper capacity limit. Do not divide this offered rate
by neoCLR's 400/sec point and call it an exact maximum-throughput speedup.

Keep the lower-rate counterexample: both 1,000/sec trials validate all 30,000
responses, but offered p99 is 3.24 / 11.65 ms and lag p99 1.34 / **7.74 ms**. The
second fails the generator-lag criterion. The headroom sweep is therefore not
uniformly qualified; successful higher-rate samples do not prove monotonic or
production-stable capacity. All four trials and that failed target flag are retained.
No run was replaced to improve the result.

The next neoCLR improvement should target the measured native allocation/root
bookkeeping costs: whole-arena allocation validation, repeated root-plan string and
ancestor checks, and collection overhead. The Go reference now makes that gap
measurable. Broader comparisons still need keep-alive, representative application
work, JSON, TLS, multicore/default configurations and a separate generator host.

Evidence: [ordered commands and identities](results-20261018/comparison.json),
[neoCLR round 1](results-20261018/neoclr-1.json)/[round 2](results-20261018/neoclr-2.json),
[Go round 1](results-20261018/go-1.json)/[round 2](results-20261018/go-2.json),
[higher-rate plan](results-20261018/headroom-plan.json)/[results](results-20261018/headroom.json),
[Go build](results-20261018/build.json), [identical reproduced build and focused tests](results-20261018/validation.json),
and [matching wire fields/body](results-20261018/wire.json). Each load summary links
its complete lossless gzip report and SHA-256; per-second/latency/RSS analyses are
retained beside them. Both fixtures emit exactly the same two header fields and
body; the captured wire bytes also match exactly. Six Python measurement tests, the Go connection-budget
test and both executable wire/shutdown checks pass. The reproducible build helper
produces the same Go executable hash as the measured binary.
