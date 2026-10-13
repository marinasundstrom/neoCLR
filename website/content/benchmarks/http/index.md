# HTTP request-processing benchmarks

Measure the complete request path: receive a request, perform the application work,
and send a validated response. Our first recorded baseline runs the same Raven
plain-text server through interpreter, JIT and native AOT using the Preview 14
macOS ARM64 toolchain.

## Plain-text baseline

**Exploratory local measurement — 13 October 2026.** Apple M1, macOS 27.0.
The Python load generator shares the machine with the server. Desktop applications
and macOS indexing were active, so the figures include uncontrolled host contention.
They are not sustained-throughput, production-capacity or cross-runtime claims.

The server responds to `GET /greeting` with ten UTF-8 bytes (`Café 🌍`). Requests
use HTTP/1.1 over loopback, concurrency one, no TLS and a new connection each time.
All **576 requests across 18 processes** returned the expected response; no errors
were observed.

| Execution mode | Median requests/s | Range across five rounds | Median startup (s) |
| --- | ---: | ---: | ---: |
| Interpreter | 1.71 | 1.61–1.83 | 2.510 |
| JIT | 60.96 | 44.52–61.30 | 4.313 |
| AOT | 63.43 | 57.77–64.11 | 0.018 |

Startup is measured separately, from process launch to the listening-port message.
It includes JIT compilation and metadata loading; native build time is excluded.
The similar JIT/AOT medians in this run do not establish equivalent performance
across other workloads. The wider JIT range is one reason to retain every sample.

### Request latency

Latency includes connection setup, transfer, server processing, response validation
and connection close. It is not an isolated handler execution time.

| Execution mode | p50 (ms) | p95 (ms) | p99 (ms) |
| --- | ---: | ---: | ---: |
| Interpreter | 562.83 | 670.24 | 796.22 |
| JIT | 16.36 | 19.64 | 31.09 |
| AOT | 15.80 | 17.68 | 18.73 |

Percentiles use nearest rank across 120 measured requests per mode. With this small
sample, p99 is a coarse estimate. Longer runs on a quiet host are needed for stronger
claims; guest CPU and peak memory have not yet been measured.

### How to reproduce

One complete process per mode warms the host before five measured rounds with
rotating mode order. Each process handles 32 requests, discards the first eight and
measures the remaining 24. Throughput is measured requests divided by their total
wall time; the table reports the median and observed range across rounds.

The harness verifies the released bundle's file hashes, builds one Raven assembly
for all three modes and checks every response's status, bytes, length and connection
closure. Native heap admission is 32 MiB for JIT and AOT; interpreter GC policy differs.

[Run the benchmark](https://github.com/marinasundstrom/neoCLR/blob/fd84b143/benchmarks/http-server/README.md) ·
[Raw samples, build commands and provenance](https://github.com/marinasundstrom/neoCLR/blob/fd84b143/benchmarks/http-server/results-20261013.json) ·
[Server source](https://github.com/marinasundstrom/neoCLR/blob/fd84b143/benchmarks/jit-http/Server.rvn)

## Workload matrix

| Server workload | Interpreter | JIT | AOT |
| --- | --- | --- | --- |
| Plain text: return a fixed UTF-8 response | Baseline above | Baseline above | Baseline above |
| JSON: serialize a response model | Not measured | Not measured | Not measured |
| JSON + database: query, map rows, serialize response | Not measured | Not measured | Not measured |

These are separate workloads, not additive estimates. Standalone JSON serialization
or PostgreSQL timings cannot predict the composed HTTP result. The database case
will record engine/version, container settings, query, row count, connection reuse
and dataset. Client benchmarks will be reported separately from server throughput.

## Next measurements

Repeat with longer measurement windows on a quiet host, then add JSON serialization
and database access using matched payloads and connection policy. Measure CPU and
peak memory alongside successful requests/s, latency, errors and startup. HTTPS,
keep-alive and concurrency sweeps require separately labeled runs.

Windows and equivalent .NET, Go and JavaScript measurements remain unreported.
Use a separate load-generator host before making saturation claims. Comparison
charts will need matched, qualified runs with raw samples alongside the graphs.

[HTTP server case](/cases/http-server/) · [SQL databases](/features/sql/) ·
[General execution benchmarks](/benchmarks/execution/)
