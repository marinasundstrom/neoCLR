# Benchmarks

neoCLR benchmarks cover application request processing and general runtime execution.
Each report identifies the measured build, workload, machine and execution mode.

| Report | What it measures |
| --- | --- |
| [HTTP request processing](/benchmarks/http/) | Plain text, JSON, and JSON with database access, each across interpreter, JIT and AOT |
| [General execution](/benchmarks/execution/) | CPU work, serialization, startup and memory, with AOT compared against JIT and interpreter |

The HTTP matrix is being assembled. Existing measurements are development baselines;
there is no completed HTTP + JSON + database comparison yet. Correctness checks for
a feature do not establish its performance.

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

<a id="interpreter-and-native-execution"></a>
<a id="native-collector-comparison"></a>
<a id="development-measurement-tooling"></a>

[Historical routing, HTTP and collector measurements](/benchmarks/historical/)
retain their original dates, methods and limitations.
