# General execution benchmarks

These benchmarks support the [native compilation guide](/features/native-compilation/).
They compare AOT, JIT and interpreted execution outside an HTTP request workload.
Startup, operation cost, memory and deployment size answer different questions and
are reported separately.

| Workload | Available evidence | Scope |
| --- | --- | --- |
| CPU computation | [Three-mode probe](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/jit-cpu/README.md) | Synthetic checksum-validated calls, separate from startup |
| JSON serialization/deserialization | [Node worker](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/json/README.md) | External batches including validation and protocol costs; short framework qualification runs |
| Routing | [Historical measurements](/benchmarks/historical/) | Process wall time including startup/loading, with different GC policies |

These reports describe their own revisions and inputs. They do not form a single
current score or establish performance for arbitrary applications. The JSON report
distinguishes its original manual worker from subsequent attributed discovery runs.
Typed-object/collection comparisons require their own matched measurements.

Future charts will show one metric and workload per comparison, with variability
and numerical tables. Startup graphs will include JIT compilation and exclude AOT
build time explicitly; build costs and deployment size belong in separate results.
[HTTP request processing](/benchmarks/http/) has its own plain-text, JSON and database
matrix so network and database time are not confused with general execution speed.
