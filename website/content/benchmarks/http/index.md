# HTTP request-processing benchmarks

Measure the complete request path: receive a request, perform the application work,
and send a validated response. The [HTTP server case](/cases/http-server/) explains
the application APIs; this report organizes performance evidence.

## Workload matrix

| Server workload | Interpreter | JIT | AOT |
| --- | --- | --- | --- |
| Plain text: return a fixed UTF-8 response | Pending qualification | Pending qualification | Pending qualification |
| JSON: serialize a response model | Pending | Pending | Pending |
| JSON + database: parameterized query, map rows, serialize response | Pending | Pending | Pending |

These are separate workloads, not additive estimates. Standalone JSON serialization
or PostgreSQL timings cannot predict the composed HTTP result. The database case
will record the engine/version, container settings, query, row count, connection
reuse and dataset. Client benchmarks will be reported separately from server
throughput so each side's work is identifiable.

## Publication status

The current website does not publish a qualified performance score. A matched
matrix with reproducible results is required before publishing comparison charts.

## Next measurements

Run the matrix with matched payloads, connection policy, concurrency, warmup and
measurement duration. Report successful requests/s, p50/p95/p99 latency, errors,
startup and memory separately. State whether the load generator shares the machine.
Graphs will compare modes for one workload and configuration at a time, with tables
and raw samples alongside them. HTTPS, keep-alive and concurrency sweeps require
separate labeled runs. Equivalent .NET, Go and JavaScript comparisons remain pending.
