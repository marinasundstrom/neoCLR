# HTTP request-processing benchmarks

Plain-text and typed-JSON Raven servers exercise the request path in interpreter,
JIT and AOT. These **17 October 2026, Apple M1/macOS 27.0** measurements use the
refreshed native-core candidate with Raven `40fcae6d0`. All **1,152 responses across
36 processes** pass validation, with zero errors. Windows measurements remain pending.

<a id="plain-text-baseline"></a>
<a id="publication-status"></a>

## Workloads and results

Plain text returns ten UTF-8 bytes, `Café 🌍`. Typed JSON creates a `Person` and
serializes `id: 42`, `name: "Café team"` and `active: true` for every request,
including object construction and reflection. Every response checks framing,
status and content before it contributes to the result.

| Workload | Mode | Median requests/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| plain | interpreter | 2.22 | 2.18–2.27 | 2.717 |
| plain | jit | 150.52 | 143.73–150.67 | 5.056 |
| plain | aot | 156.91 | 148.10–157.58 | 0.005 |
| json | interpreter | 1.20 | 1.19–1.20 | 2.721 |
| json | jit | 81.42 | 78.25–82.13 | 8.388 |
| json | aot | 84.65 | 81.78–85.16 | 0.013 |

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

The optional PostgreSQL workload has not been rerun for this candidate. Its
[historical measurements](/benchmarks/history/) do not qualify the current package.
HTTPS, keep-alive, RSS and managed allocation rate are outside these measurements.

[Raw plain/JSON samples and package identity](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server/candidate-refreshed-20261017) ·
[Reproduction instructions](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/http-server) ·
[Garbage-collection measurements](/benchmarks/gc/)

[General execution](/benchmarks/execution/) · [All benchmark reports](/benchmarks/)
