# General execution benchmarks

These CPU and JSON node workloads run outside HTTP through the same Raven benchmark
worker in interpreter, JIT and AOT. **13 October 2026, Apple M1/macOS 27.0, Preview 14
macOS ARM64 tools.** All 45 initial processes and ten longer native CPU processes
passed checksums and unchanged-input validation.

This is an exploratory interactive-host baseline. Timings are external batches,
including callbacks, result checks, console/pipe markers and reader scheduling.
They are not isolated serializer CPU timings or a general language ranking.

## Operation throughput

One CPU operation sums integers 0 through 9,999 and checks 49,995,000. JSON operations
serialize a prepared node tree or parse a fixed nested document containing Unicode,
objects, arrays, numbers, Boolean and null. Serialization validates the complete
output; parsing checks root shape/count and name. Setup verifies a full round trip.

| Case | Mode | Median operations/s | Five-round range |
| --- | --- | ---: | ---: |
| Sum 10,000 integers | interpreter | 7.31 | 7.17–7.55 |
| Sum 10,000 integers | jit | 41332.73 | 41163.65–41380.11 |
| Sum 10,000 integers | aot | 41119.82 | 40891.99–41318.89 |
| Serialize JSON nodes | interpreter | 4.09 | 3.75–4.17 |
| Serialize JSON nodes | jit | 246.97 | 243.72–251.55 |
| Serialize JSON nodes | aot | 263.04 | 262.90–265.73 |
| Deserialize JSON nodes | interpreter | 4.36 | 4.03–4.40 |
| Deserialize JSON nodes | jit | 201.42 | 162.10–204.53 |
| Deserialize JSON nodes | aot | 211.81 | 154.15–214.49 |

## Batch size matters

The initial native CPU pilot used 16 operations and finished in less than 1 ms.
Transport/scheduling overhead was too influential, so it is retained as pilot data
rather than the primary native CPU baseline. The displayed native CPU results use
**10,000 operations and 20 warmups** per process, producing approximately 0.24-second
batches. Interpreter CPU and JSON use **16 operations and two warmups**. The operation
definition stays the same; these different batch settings are explicit.

Five fresh-process rounds rotate mode order. The runner is fixed-count and does not
yet calibrate automatically or supply a statistical regression gate. Native CPU's
larger batches improve measurement resolution; they do not establish repeatability
across days, operating systems or other hosts.

## Startup

Startup includes worker registration, JSON setup, metadata loading and JIT compilation;
JSON setup also happens for the CPU case. It excludes warmup batches and AOT builds.
These are worker startup measurements, not minimal runtime startup costs.

| Case | Interpreter (s) | JIT (s) | AOT (s) |
| --- | ---: | ---: | ---: |
| Sum 10,000 integers | 2.888 | 2.392 | 0.010 |
| Serialize JSON nodes | 2.873 | 2.425 | 0.012 |
| Deserialize JSON nodes | 2.938 | 2.415 | 0.012 |

## Reproduce and compare

The benchmark verifies every operation and retains raw samples, build commands and
compiler/runtime/library hashes. AOT uses the console host; JIT and interpreter
have different host/GC policies, so these numbers do not isolate optimizer or
collector efficiency. Typed object JSON is measured separately in the
[HTTP workload matrix](/benchmarks/http/).

[Instructions and workload source](https://github.com/marinasundstrom/neoCLR/blob/c713d0ce/benchmarks/execution/README.md) ·
[Initial raw samples, including the CPU pilot](https://github.com/marinasundstrom/neoCLR/blob/c713d0ce/benchmarks/execution/results-20261013.json) ·
[Larger native CPU samples](https://github.com/marinasundstrom/neoCLR/blob/c713d0ce/benchmarks/execution/cpu-native-results-20261013.json) ·
[Build provenance](https://github.com/marinasundstrom/neoCLR/blob/c713d0ce/benchmarks/execution/build-20261013.json).

[History](/benchmarks/history/) preserves these runs rather than overwriting them.
Controlled-host repeats and calibrated regression thresholds remain next steps.
Peak memory, allocation rate, deployment size, Windows and external-runtime
comparisons are not measured here. AOT build cost is separate from execution speed.

[Native compilation](/features/native-compilation/) · [All benchmark reports](/benchmarks/)
