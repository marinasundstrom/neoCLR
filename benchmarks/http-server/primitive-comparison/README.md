# Plain-text server primitive baseline — 2026-10-23

This is the primary development baseline requested by the author: process as many
plain HTTP requests as possible through worker execution while using as little
CPU as possible. Allocations are a secondary diagnostic. JSON, artificial handler
work, queued Task.Run hops and complex graph workloads are secondary until this
baseline improves. These priorities do not redefine ordinary Task/HTTP semantics
or turn neoCLR into a web application framework.

## Contract and scope

GET / returns status 200 and exactly ten ASCII/UTF-8 bytes, `xxxxxxxxxx`.
HTTP/1.1, loopback TCP, no TLS, no compression, Connection: close, 32 concurrent
clients. Handlers directly return/write the text; no extra scheduled work is
inserted to simulate an application. Every response checks status, Content-Length,
body bytes and the connection-close header; the compiled client reads through EOF.
All server processes must exit successfully after a separate control connection.

The same host runs each server sequentially, two seconds warmup then six measured
seconds. Seven configurations run in forward and reverse order. Builds and other
benchmarks finish first. Only responses completed inside the window contribute
to throughput/latency; drain responses remain recorded. Latency includes connection
establishment through server EOF. Closed-loop percentiles are not an offered-load
SLA: requests that might have arrived while all clients wait are not represented.

The reference APIs are Go net/http HandlerFunc, Node http.createServer, and a
Kestrel terminal request delegate with no routing/MVC middleware. They are small
idiomatic API consumers, not identical internal implementations. Headers such as
Date/Server differ; no pooling/GC/thread policy is rewritten to mimic neoCLR.

neoCLR uses its AOT source HTTP host with one/four isolated owners, one shared
kernel listener, 1 MiB heap and 16 Serve lanes per owner, explicitly paced GC.
This is not shared-heap HTTP dispatch. Go uses GOMAXPROCS=1/4 and GOGC=100; Kestrel
targets .NET 10 with DOTNET_PROCESSOR_COUNT=1/4 and server GC; Node uses one JS
event loop. These settings are not CPU affinity or equal hard thread budgets.
Reference servers retain ordinary connection admission defaults. The client uses
GOMAXPROCS=2 in every final trial. Extra CPUs can contend on this desktop; this is
not a production capacity measurement or a ranking of languages.

Connection-close is deliberately the common supported baseline: neoCLR's
[response encoder](../../../runtime/raven/src/System/Web/Http/HttpServer.rvn)
currently emits Connection: close. Persistent connections are a separate support
gap and essential future comparison, not a reason to compare unlike connection
policies now. No keep-alive throughput claim follows.

## Results

See [comparison.json](comparison.json) for all measured response counts, latency,
process CPU and client utilization. The checked-in [build summary](build-summary.json)
pins the compiled application, native source inputs and toolchain.

| Configuration | Requests/sec, two trials | Estimated server CPU µs/response | p99 ms, two trials |
| --- | --- | --- | --- |
| neoclr-1 | 491.2, 485.2 | 2,048.1, 2,066.6 | 67.00, 70.59 |
| go-1 | 25,373.2, 22,411.5 | 30.8, 31.5 | 1.58, 2.53 |
| node-1 | 24,338.8, 21,348.2 | 33.5, 34.2 | 1.60, 2.87 |
| kestrel-1 | 19,358.0, 21,445.2 | 72.8, 66.9 | 3.44, 3.01 |
| neoclr-4 | 1,530.8, 1,692.5 | 2,422.9, 2,279.0 | 29.34, 31.23 |
| go-4 | 18,574.0, 18,698.8 | 75.9, 70.9 | 2.97, 3.18 |
| kestrel-4 | 15,912.7, 15,636.3 | 142.6, 150.3 | 3.40, 3.37 |

Four neoCLR owners deliver roughly 3.1–3.5 times the single-owner throughput,
but CPU cost per response increases from about 2.05–2.07 ms to 2.28–2.42 ms.
This demonstrates useful worker throughput scaling and remaining coordination/
runtime cost, not CPU efficiency parity. The reference single-setting servers
process roughly 19,000–25,000 responses/s at 31–73 CPU µs/response on this workload.
The gap warrants investigating neoCLR's per-request execution/GC/root/scheduling
work alongside worker utilization. These timings alone do not attribute the gap
to one subsystem, a language implementation, or allocations.

The four-processor reference settings are slower here than their single settings;
this tiny loopback workload and shared desktop do not establish general multicore
scalability. Do not select a production worker count or rank frameworks from it.


Server CPU per response below is an estimate: whole server-process CPU, including
startup/warmup/drain/shutdown, divided by all validated warmup/measurement/drain
responses. CPU used by the separate compiled client processes is subtracted from
child-process accounting. It is not handler-only CPU or a measured-window profiler.
One CPU microsecond/response is equivalent to one CPU second per million responses.
The final client consumes less than its two-processor budget in these trials;
this does not prove an unbounded load generator or maximum server capacity.

## Measurement corrections and provenance

The initial Python client reached approximately one CPU at only 5,000–6,000
requests/s against the reference servers. A Go HTTP client raised that ceiling
but early local close could exhaust ephemeral ports (EADDRNOTAVAIL) at higher
connection-close rates. That failed run is not a score. The final compiled TCP
client waits for server EOF before closing, validates the fixed HTTP contract,
and uses two Go execution processors. This changes only the measurement driver,
not any server or system TCP settings.

The [calibration record](calibration.json) preserves the earlier driver outcomes
and identities. Final results must not be mixed with those driver-limited runs.
Go 1.26.3, Node 22.21.0 and .NET/ASP.NET Core 10.0.0 are installed test versions,
not claims about the newest releases. The matching neoCLR Raven bundle is
1128b98cf; source codegen/native inputs are pinned in build and comparison records.
Large compiler stdout is replaced by a digest in the retained build summary;
full local reports remain at their recorded paths.

## API/runtime research and implications

Primary sources reviewed 2026-10-23; these are design comparisons, separate from
the measurements above. Hyper/Tokio is researched here but not benchmarked.

| Runtime/API | How requests and I/O execute | Implication for neoCLR |
| --- | --- | --- |
| .NET/Kestrel | Async request work uses normal pool threads; guidance explicitly discourages redundant Task.Run and blocking waits. Response bodies expose Stream and PipeWriter contracts. | Preserve the .NET semantic direction: async waiting releases execution capacity, worker scheduling serves concurrent requests, and the byte-writing path need not force unnecessary wrapper/copy work. |
| Go net/http | Serve creates a service goroutine per accepted connection; handlers receive a Request and ResponseWriter. GOMAXPROCS controls simultaneous execution of Go code rather than creating one OS thread per connection. | Lightweight logical request execution can be separate from OS worker count. A goroutine is not equivalent to an allocated native thread. |
| Node http | Request callbacks execute on an event loop; built-in async I/O handles socket waiting. Worker threads primarily help CPU-intensive JavaScript. | One execution thread can handle high plain-response throughput. Parallelism helps only after accounting for per-request work and scheduling costs. |
| Rust Hyper/Tokio | Hyper models an async request-to-response service; Tokio separates I/O driver, scheduler and timers, with current-thread and multithread runtime options. Spawned migratable tasks require Send. | Separate I/O progress from managed execution and make cross-thread lifetime/ownership explicit. Green tasks do not remove the need for GC and root coordination. |

Sources:
[ASP.NET performance guidance](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/best-practices?view=aspnetcore-10.0),
[request/response body APIs](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/middleware/request-response?view=aspnetcore-10.0),
[Go net/http](https://pkg.go.dev/net/http#Serve),
[Go execution controls](https://pkg.go.dev/runtime#GOMAXPROCS),
[Node HTTP](https://nodejs.org/api/http.html#httpcreateserveroptions-requestlistener),
[Node event loop](https://nodejs.org/en/learn/asynchronous-work/dont-block-the-event-loop),
[Node workers](https://nodejs.org/api/worker_threads.html),
[Hyper service](https://hyper.rs/guides/1/server/hello-world/),
[Tokio runtime](https://docs.rs/tokio/latest/tokio/runtime/index.html),
[Tokio task ownership](https://tokio.rs/tokio/tutorial/spawning).

neoCLR already has nonblocking native socket I/O and owner-local cooperative
fibers. The qualified HTTP multicore path currently duplicates application owners;
shared-heap asynchronous Tasks/I/O remain unfinished. The new synchronous shared
coordinator's graph speedup is not an HTTP speedup and must not be combined with
these results.

Use this fixture for subsequent worker scheduling and runtime hot-path changes.
Report requests/second, CPU per completed request, tail latency, worker count and
response errors together. Increase useful work per wakeup and reduce root/GC or
queue overhead when profiling identifies it; retain shared-root correctness tests.
Investigate allocations when they explain CPU/GC costs, without making zero
allocations the prerequisite for throughput progress. Keep JSON and other handler
work in separate later results. No public API change is proposed by this baseline.

## Reproduce

Use a matching compiler bundle and rebuilt libraries, with the macOS SDK selected
explicitly on this host:

```sh
NeoClrLibraryRoot=/absolute/libraries python3 scripts/build-native-project.py \
  --project benchmarks/http-server/primitive-comparison/Server.rvnproj \
  --profile http --http-workers 4 --bundle /absolute/bundle \
  --aot /absolute/neoclr-codegen --output target/plain-build
python3 benchmarks/http-server/primitive-comparison/compare.py \
  --neoclr-build target/plain-build/build.json --output target/plain-comparison
```

Fresh output directories are required. Go, Node and the .NET 10 runtime/SDK must
be available. The controller builds all reference executables and relinks the
same neoCLR object for one/four owners before timing. All source hashes and
commands are retained. No public API snapshot or website build is required.

## Follow-up: balanced paced collection

The [GC pacing experiment](pacing/README.md) compares three bounded budgets with
the same plain response and selects 256 allocations/1,024 publications for opt-in
paced mode. Final paired trials improve mean one/four-owner throughput about
26%/23% and reduce CPU per response about 21%/20%. The original baseline above
remains historical evidence; forced-GC behavior and the HTTP API are unchanged.
