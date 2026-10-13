# HTTP request-processing benchmarks

Three Raven server cases measure the complete request path: plain text, object JSON,
and a PostgreSQL query mapped to an object and serialized as JSON. Each case runs
in interpreter, JIT and AOT using the Preview 14 macOS ARM64 tools.

**Exploratory baseline — 13 October 2026, Apple M1, macOS 27.0.** The Python client
shares the machine with the server. Desktop/indexing activity was present. These
short, fixed-count runs are not controlled regression gates or production-capacity
claims. All **1,728 responses across 54 processes** passed validation, with zero errors.

<a id="plain-text-baseline"></a>
<a id="publication-status"></a>

## Workload matrix

Median successful requests per second across five rounds:

| Server workload | Interpreter | JIT | AOT |
| --- | ---: | ---: | ---: |
| Plain text | 1.71 | 60.96 | 63.43 |
| Object JSON | 0.93 | 33.36 | 34.76 |
| PostgreSQL + JSON | 0.55 | 15.65 | 16.06 |

The database's AOT executable links precompiled guest code to the released **shared
native host** for runtime-async services. It performs no guest JIT compilation, but
this is not a standalone SQL deployment qualification. Plain-text and object-JSON
AOT use the standalone HTTP host. Compare modes within their recorded workload;
do not subtract rows to infer an isolated database or serialization cost.

### What each request does

- **Plain text:** returns ten UTF-8 bytes, `Café 🌍`.
- **Object JSON:** creates a `Person` and serializes `id: 42`, `name: "Café team"`,
  and `active: true` for every request. This includes construction and reflection.
- **PostgreSQL + JSON:** executes a parameterized SELECT returning those three
  values, maps one buffered row to `Person`, then uses the same JSON response code.
  One database connection is opened before readiness and reused for 32 requests.

PostgreSQL **18.6** runs in an ARM64 Docker container with **2 CPUs and 512 MiB**,
using loopback-only trust authentication. The image digest is in the raw report.
The query uses no table or persistent dataset; this is a query/map/serialize baseline,
not disk-backed database scalability. The benchmark's container was stopped afterward.

## Variability and startup

Ranges include every measured round. Startup is process launch to the listening-port
message, including metadata loading, JIT compilation and database connection setup
where applicable. Native builds are excluded from startup and request timing.

| Workload | Mode | Requests/s range | Median startup (s) |
| --- | --- | ---: | ---: |
| Plain text | Interpreter | 1.61–1.83 | 2.510 |
| Plain text | JIT | 44.52–61.30 | 4.313 |
| Plain text | AOT | 57.77–64.11 | 0.018 |
| Object JSON | Interpreter | 0.92–0.94 | 2.432 |
| Object JSON | JIT | 30.77–33.77 | 6.095 |
| Object JSON | AOT | 17.71–35.32 | 0.013 |
| PostgreSQL + JSON | Interpreter | 0.55–0.56 | 3.255 |
| PostgreSQL + JSON | JIT | 13.21–15.82 | 8.241 |
| PostgreSQL + JSON | AOT (shared host) | 15.83–16.09 | 0.083 |

## Request latency

Latency includes connection setup, request/response transfer, server processing,
client validation and connection close. Each row pools 120 measured requests.
Percentiles use nearest rank; p99 is a coarse estimate at this sample size.

| Workload | Mode | p50 (ms) | p95 (ms) | p99 (ms) |
| --- | --- | ---: | ---: | ---: |
| Plain text | interpreter | 562.83 | 670.24 | 796.22 |
| Plain text | jit | 16.36 | 19.64 | 31.09 |
| Plain text | aot | 15.80 | 17.68 | 18.73 |
| Object JSON | interpreter | 1052.18 | 1228.85 | 1385.10 |
| Object JSON | jit | 30.03 | 32.40 | 35.22 |
| Object JSON | aot | 28.60 | 38.58 | 78.94 |
| PostgreSQL + JSON | interpreter | 1770.58 | 2068.41 | 2260.97 |
| PostgreSQL + JSON | jit | 63.56 | 68.12 | 111.48 |
| PostgreSQL + JSON | aot | 62.35 | 63.77 | 64.91 |

## How to reproduce

All requests use HTTP/1.1, no TLS, concurrency one and a new loopback connection.
One full 32-request process per mode warms the host. Five measured rounds rotate
mode order. Each process discards eight requests and measures the remaining 24.
Throughput is measured requests divided by their total wall time; tables report
medians and observed ranges rather than selecting the fastest round.

The harness verifies bundle hashes, builds one assembly per workload for all three
modes and validates every response. Text is byte-checked; JSON is parsed and its
complete three-field value checked. Native heap admission is 32 MiB. Interpreter
GC policy differs. No concurrent benchmark/build was launched during measurement.

[Reproduction instructions and source](https://github.com/marinasundstrom/neoCLR/blob/c713d0ce/benchmarks/http-server/README.md). Raw reports: [Plain text](https://github.com/marinasundstrom/neoCLR/blob/c713d0ce/benchmarks/http-server/results-20261013.json) · [Object JSON](https://github.com/marinasundstrom/neoCLR/blob/c713d0ce/benchmarks/http-server/json-results-20261013.json) · [PostgreSQL + JSON](https://github.com/marinasundstrom/neoCLR/blob/c713d0ce/benchmarks/http-server/database-results-20261013.json).

## History and next measurements

[Benchmark history](/benchmarks/history/) preserves dated runs and their workload
versions. The goal is stable or improving performance under comparable conditions;
controlled hosts, calibrated durations and noise-aware regression checks come next.
These exploratory baselines cannot yet enforce that goal.

Longer quiet-host runs, guest CPU/peak memory, Windows, HTTPS, connection reuse,
concurrency sweeps and sustained cross-runtime measurements remain open. Use a
separate load generator before making saturation claims.

Development now retains [direct-handler comparison projects and pilot results](/benchmarks/http-comparison/)
for Kestrel (.NET JIT/AOT), Go and Node.js alongside neoCLR. Routing is excluded.
Those short-run development comparisons do not replace the published baseline above.
They remain local, with publication on hold pending neoCLR HTTP improvements.

[HTTP server case](/cases/http-server/) · [SQL databases](/features/sql/) ·
[General execution benchmarks](/benchmarks/execution/)
