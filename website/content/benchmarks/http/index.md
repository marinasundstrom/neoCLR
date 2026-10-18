# HTTP request-processing benchmarks

Plain-text and typed-JSON Raven servers exercise the request path in interpreter,
JIT and AOT. These **18 October 2026, Apple M1/macOS 27.0** measurements use the
Preview 16 package with Raven `c0854cdda`. All **1,152 responses across
36 processes** pass validation, with zero errors. Windows results follow below as a separate hosted-runner series.

<a id="plain-text-baseline"></a>
<a id="publication-status"></a>

## Workloads and results

Plain text returns ten UTF-8 bytes, `Café 🌍`. Typed JSON creates a `Person` and
serializes `id: 42`, `name: "Café team"` and `active: true` for every request,
including object construction and reflection. Every response checks framing,
status and content before it contributes to the result.

| Workload | Mode | Median requests/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| http-plain | interpreter | 2.16 | 2.11–2.19 | 2.901 |
| http-plain | jit | 142.79 | 137.54–150.22 | 5.364 |
| http-plain | aot | 147.21 | 143.48–155.82 | 0.006 |
| http-json | interpreter | 1.17 | 1.16–1.18 | 2.782 |
| http-json | jit | 79.87 | 79.55–81.48 | 8.432 |
| http-json | aot | 83.18 | 78.17–83.39 | 0.005 |

## Windows x64

The same package passes on Windows Server 2025 (AMD Family 26 Model 2), with
55 CPU/JSON processes and 1,152 HTTP responses validated across the full run.
Shared hosted-runner load is uncontrolled; these numbers are a separate series,
not a comparison with the macOS machine.

| Workload | Mode | Median requests/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| http-plain | interpreter | 2.08 | 1.90–2.21 | 3.389 |
| http-plain | jit | 62.35 | 45.59–69.80 | 6.353 |
| http-plain | aot | 57.90 | 51.11–64.95 | 0.007 |
| http-json | interpreter | 1.21 | 1.16–1.24 | 3.434 |
| http-json | jit | 44.89 | 42.93–54.00 | 8.808 |
| http-json | aot | 45.67 | 36.61–50.93 | 0.007 |

[Windows raw reports and package identity](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/release-20261018/windows)

## Measurement boundaries

Each mode has one whole-process warmup and five measured fresh processes, with
rotating mode order. Each process validates 32 responses; the first eight are
excluded from timing. Startup measures process launch to listening-port readiness,
including metadata loading and JIT preparation. Builds complete before measurement.

The Python client shares the machine with the server. Requests are sequential,
use HTTP/1.1 loopback and close each connection. Host indexing was active. These
short, fixed-count workstation runs do not measure concurrent server capacity or
establish a controlled regression threshold. Request latency samples, host-load
observations and artifact/input hashes are retained in the raw reports.

The optional PostgreSQL workload has not been rerun for this release. Its
[historical measurements](/benchmarks/history/) do not qualify the current package.
HTTPS, keep-alive, RSS and managed allocation rate are outside these measurements.

[Raw plain/JSON samples and package identity](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/release-20261018/macos) ·
[Reproduction instructions](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server) ·
[Garbage-collection measurements](/benchmarks/gc/)

[General execution](/benchmarks/execution/) · [All benchmark reports](/benchmarks/)


## Development load investigation

After Preview 16, the [offered-load investigation](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/load)
separates offered arrivals, successful completions, client admission misses and
errors, with latency and CPU/RSS samples. It also compares worker-count and GC-policy
costs. These local development experiments do not replace the release measurements
above or establish production capacity; paced collection remains experimental.
