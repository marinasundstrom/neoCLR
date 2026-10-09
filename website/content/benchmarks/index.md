# Native execution benchmarks

These measurements describe specific development builds on macOS ARM64. They help
evaluate the [native backend](../features/native-compilation/); they do not measure
all neoCLR applications or establish production capacity. Input artifacts, build
settings and raw results are linked with each comparison.

## Interpreter and native execution


The routing comparison runs the **same Raven artifact and entry point** through the
interpreter and native backend. The checked-in harness validates output before accepting
a timing, alternates execution order, warms up, and saves raw samples and tool hashes.
It uses a Release interpreter and optimized C adapters. The current metric includes
process startup, interpreter metadata loading/verification, routing and output; it is
not steady-state handler latency. The modes currently use different GC scheduling and
memory-limit policies.

Local macOS ARM64 results (2026-10-08), five measured pairs after one warm-up pair:

| Same routing workload, 1,024 operations | Median process wall time |
| --- | ---: |
| neoCLR interpreter, Release build | 29.447 s |
| neoCLR ARM64 native image, native GC | 0.392 s |

Both runs print `1024` after validating all routing outcomes. These numbers include
startup/loading and use different collector policies; they must not be read as an HTTP
throughput ratio. [Raw samples and build/input provenance](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/native-web/routing-validation.json)
allow the result to be checked and repeated as the implementation changes.

[Samples, reproduction commands and comparison contract](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/native-web)
are checked into the repository. The ASP.NET Core greeting sample is an initial comparison
candidate. No HTTP performance ranking against .NET or another platform is available yet.

The HTTP comparison uses one persistent process for 32 sequential
connection-close requests, excluding the first eight requests from request timing.
Three alternating pairs on Apple M1/macOS ARM64 (2026-10-08) produced:

| Same Raven HTTP artifact | Median requests/s | Median request latency |
| --- | ---: | ---: |
| Release interpreter | 3.08 | 324.40 ms |
| ARM64 native standalone | 20.09 | 49.65 ms |

Each response is validated. Latency is the median of per-run medians and includes
connection establishment and response reading by a client on the same machine.
The short runs use different GC/host scheduling policies; native adapters use -O2,
while native CIL currently uses Cranelift's default unoptimized setting. These are
POC baselines, not capacity or tail-latency claims. Startup is measured separately.
[Raw samples, tool hashes and full method](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/native-web/persistent-http-validation.json)
make the comparison reviewable.

Extend the short repeated-request comparison to longer matched loads, then ASP.NET
Core JIT and Native AOT with matching successful requests and responses. Start with
HTTP/1.1, identical UTF-8 payloads, connection-close behavior and concurrency one.
Measure startup separately from steady-state throughput, latency percentiles, errors,
peak memory and deployment size. Keep-alive, concurrency and JSON will be separate cases
as support grows. Record platform versions, build flags, hardware and load-generator
settings with every published result.

.NET already provides [ASP.NET Core Native AOT](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/native-aot?view=aspnetcore-10.0)
with its own compatibility constraints. neoCLR's smaller POC scope does not establish
an advantage over it; useful comparisons require equivalent behavior and measured costs.

## Native collector comparison

A separate three-pair comparison of the same native object measured **20.08 → 96.05
requests/s** after improving GC address lookup (49.74 → 10.43 ms median request latency).
Correctness and native cleanup checks pass. This short local adapter comparison does
not rerun the interpreter or establish a .NET comparison. Collection still runs at
every native boundary; production GC policy remains future work.

[GC lookup comparison and raw results](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/native-web/gc-index-validation.json).

The comparisons use different recorded builds. Do not combine their numbers into one
current performance claim. Repository benchmark reports retain exact revisions,
commands and validation details.
