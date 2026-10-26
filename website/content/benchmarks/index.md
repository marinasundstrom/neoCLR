# Benchmarks

neoCLR benchmarks cover application request processing and general runtime execution.
Each report identifies the measured build, workload, machine and execution mode.

| Report | What it measures |
| --- | --- |
| [HTTP request processing](/benchmarks/http/) | Plain text and typed JSON across interpreter, JIT and AOT |
| [Garbage collection](/benchmarks/gc/) | Matched collector-path HTTP comparisons, allocation/reclamation diagnostics and correctness evidence |
| [General execution](/benchmarks/execution/) | CPU work, JSON serialization/deserialization and startup across interpreter, JIT and AOT |

The current Preview 18 macOS ARM64 and Windows x64 reports cover CPU/JSON execution
and plain/typed-JSON HTTP. Each platform validates 55 CPU/JSON processes and 1,152
HTTP responses. These short workstation/hosted-runner series retain variability and
limitations; they are not controlled-host regression gates or cross-platform rankings. Database and collector reports identify their own
builds separately. Sustained cross-runtime measurements remain open.

[Benchmark history](/benchmarks/history/) preserves dated results and their raw evidence
so future comparable runs can show progress, including regressions.

## Reading the results

Startup and steady-state work are reported separately. Request reports include
throughput, latency distributions and errors; execution reports state exactly what
one operation includes. AOT build time and deployment size are separate from run time.
Compare modes within the same recorded experiment, using equivalent inputs and
validated outputs. Different revisions, machines or workloads are separate series.

Graphs will accompany qualified measurements, with units, labeled execution modes,
visible variability and a numerical table. Each graph will link to raw samples,
reproduction commands, toolchain revisions and environment details. Missing results
will remain marked as pending, never plotted as zero. Comparisons with .NET, Go and
JavaScript require equivalent applications and recorded configurations.

## Measurement tools

The [Raven benchmark framework](https://github.com/marinasundstrom/neoCLR/tree/main/tools/benchmarking)
and [external runner](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/framework)
support selectable cases and separate process startup and operation batches.
Reports distinguish internal operation timing from external timings that include
validation, protocol and scheduling costs.
