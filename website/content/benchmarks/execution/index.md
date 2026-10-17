# General execution benchmarks

CPU and JSON workloads run through the same Raven worker in interpreter, JIT and
AOT. These **18 October 2026, Apple M1/macOS 27.0** measurements use the Preview 16 package with Raven `c0854cdda`. All 45 CPU/JSON processes and ten longer
native CPU processes pass correctness and unchanged-input checks. Windows results follow below as a separate hosted-runner series.

## Workloads and results

One CPU operation sums integers 0 through 9,999 and checks 49,995,000. JSON operations
serialize a prepared node tree or parse a fixed nested document containing Unicode,
objects, arrays, numbers, Boolean and null. Serialization checks the complete output;
parsing checks root shape/count and name. Setup verifies a full round trip.

| Workload | Mode | Median operations/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| cpu.sum | interpreter | 6.26 | 6.11–6.43 | 3.316 |
| cpu.sum | jit | 39423.95 | 39416.56–39559.80 | 2.770 |
| cpu.sum | aot | 39560.05 | 38478.95–39694.88 | 0.008 |
| json.serialize | interpreter | 3.81 | 3.78–3.85 | 3.363 |
| json.serialize | jit | 438.32 | 423.51–441.45 | 2.723 |
| json.serialize | aot | 462.62 | 445.82–464.27 | 0.008 |
| json.deserialize | interpreter | 4.17 | 4.10–4.20 | 3.347 |
| json.deserialize | jit | 341.29 | 329.64–345.89 | 2.770 |
| json.deserialize | aot | 357.27 | 346.74–361.41 | 0.008 |

## Windows x64

The same package passes on Windows Server 2025 (AMD Family 26 Model 2), with
55 CPU/JSON processes and 1,152 HTTP responses validated across the full run.
Shared hosted-runner load is uncontrolled; these numbers are a separate series,
not a comparison with the macOS machine.

| Workload | Mode | Median operations/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| cpu.sum | interpreter | 5.34 | 5.25–5.48 | 3.857 |
| cpu.sum | jit | 71621.49 | 69438.18–72756.32 | 3.345 |
| cpu.sum | aot | 79799.16 | 75297.18–85303.27 | 0.010 |
| json.serialize | interpreter | 4.28 | 3.73–4.51 | 3.831 |
| json.serialize | jit | 379.43 | 346.25–386.29 | 3.220 |
| json.serialize | aot | 376.71 | 359.75–388.06 | 0.010 |
| json.deserialize | interpreter | 4.28 | 4.12–4.39 | 3.995 |
| json.deserialize | jit | 284.26 | 278.49–289.74 | 3.328 |
| json.deserialize | aot | 281.36 | 267.93–285.82 | 0.010 |

[Windows raw reports and package identity](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/release-20261018/windows)

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

[Raw samples, package identity and reproduction details](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/release-20261018/macos) ·
[Benchmark framework](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/framework) ·
[Earlier measurements](/benchmarks/history/)

[HTTP workloads](/benchmarks/http/) · [All benchmark reports](/benchmarks/)
