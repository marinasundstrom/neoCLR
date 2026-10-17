# General execution benchmarks

CPU and JSON workloads run through the same Raven worker in interpreter, JIT and
AOT. These **17 October 2026, Apple M1/macOS 27.0** measurements use the refreshed
native-core candidate with Raven `40fcae6d0`. All 45 CPU/JSON processes and ten longer
native CPU processes pass correctness and unchanged-input checks. Windows results
remain pending.

## Workloads and results

One CPU operation sums integers 0 through 9,999 and checks 49,995,000. JSON operations
serialize a prepared node tree or parse a fixed nested document containing Unicode,
objects, arrays, numbers, Boolean and null. Serialization checks the complete output;
parsing checks root shape/count and name. Setup verifies a full round trip.

| Workload | Mode | Median ops/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| cpu.sum | interpreter | 6.39 | 6.24–6.54 | 3.197 |
| cpu.sum | jit | 40931.74 | 39674.56–41490.56 | 2.635 |
| cpu.sum | aot | 41435.87 | 39618.40–41564.93 | 0.007 |
| json.serialize | interpreter | 4.00 | 3.92–4.07 | 3.209 |
| json.serialize | jit | 444.98 | 429.72–448.37 | 2.625 |
| json.serialize | aot | 464.31 | 451.37–471.34 | 0.009 |
| json.deserialize | interpreter | 4.21 | 4.07–4.24 | 3.176 |
| json.deserialize | jit | 346.38 | 345.70–348.03 | 2.654 |
| json.deserialize | aot | 361.11 | 359.39–367.03 | 0.016 |

## Measurement boundaries

Five fresh-process rounds rotate execution modes. Interpreter CPU and JSON use
16 operations and two warmups per process. Native CPU uses 10,000 operations and
20 warmups, avoiding the sub-millisecond batches of the retained 16-operation pilot.
Startup includes worker registration, JSON setup, metadata loading and JIT
compilation, even for the CPU case; it excludes compilation and warmup batches.

External batch timings include callbacks, correctness checks, console/pipe markers
and reader scheduling. They do not isolate serializer CPU time. No builds ran
concurrently, but macOS indexing was active. These exploratory workstation results
are not a controlled regression threshold, language ranking or platform comparison.
Peak memory, allocation rate and deployment size are not measured here.

[Raw samples, package identity and reproduction details](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/execution/candidate-refreshed-20261017) ·
[Benchmark framework](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/framework) ·
[Earlier measurements](/benchmarks/history/)

[HTTP workloads](/benchmarks/http/) · [All benchmark reports](/benchmarks/)
