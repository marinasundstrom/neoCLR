# HTTP request-processing benchmarks

Measure the complete request path: receive a request, perform the application work,
and send a validated response. The [HTTP server case](/cases/http-server/) explains
the application APIs; this report organizes performance evidence.

## Workload matrix

| Server workload | Interpreter | JIT | AOT |
| --- | --- | --- | --- |
| Plain text: return a fixed UTF-8 response | Exploratory baseline | Exploratory baseline | Exploratory baseline |
| JSON: serialize a response model | Pending | Pending | Pending |
| JSON + database: parameterized query, map rows, serialize response | Pending | Pending | Pending |

These are separate workloads, not additive estimates. Standalone JSON serialization
or PostgreSQL timings cannot predict the composed HTTP result. The database case
will record the engine/version, container settings, query, row count, connection
reuse and dataset. Client benchmarks will be reported separately from server
throughput so each side's work is identifiable.

## Existing plain-text baseline

The recorded macOS ARM64 run on Apple M1 (2026-10-11) uses the same Raven server
assembly in all three modes: HTTP/1.1, a ten-byte Unicode response, sequential
loopback requests and connection close. Five rounds each measure 24 requests after
eight discarded requests. Every response is validated.

| Mode | Median requests/s | Median process startup (s) |
| --- | ---: | ---: |
| Interpreter | 1.38 | 3.120 |
| JIT | 53.49 | 3.874 |
| AOT | 54.31 | 0.010 |

**Exploratory historical result:** background compilation and indexing caused CPU
contention. These short concurrency-one runs need repetition on a quiet machine
before they support performance claims or a comparison chart. Startup includes JIT
compilation; AOT compilation happens before the run. This does not measure server
capacity or Windows performance.

[Raw samples and build provenance](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/jit-http/results.json) ·
[Workload and reproduction](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/jit-http/README.md)

## Next measurements

Run the matrix with matched payloads, connection policy, concurrency, warmup and
measurement duration. Report successful requests/s, p50/p95/p99 latency, errors,
startup and memory separately. State whether the load generator shares the machine.
Graphs will compare modes for one workload and configuration at a time, with tables
and raw samples alongside them. HTTPS, keep-alive and concurrency sweeps require
separate labeled runs. Equivalent .NET, Go and JavaScript comparisons remain pending.
