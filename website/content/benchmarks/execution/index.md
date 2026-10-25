# General execution benchmarks

[Allocation-pattern samples](/benchmarks/gc/#applications-with-different-allocation-patterns)
compare low-allocation work, object churn, bursts and retained windows under sync
and concurrent GC. The [standard procedure](/benchmarks/gc/procedure/) supports
separate benchmark and profiling runs.

## Preview 17 — 22–23 October 2026

The matching macOS ARM64 and Windows x64 packages each pass all five stages:
55 CPU/JSON processes and 1,152 HTTP responses. SDK/editor version
`0.1.17-neoclr.20261022.1`, Raven `301e668a5`. Five measured rounds retain their
ranges; startup is separate. HTTP uses sequential loopback requests with connection
close. Native CPU rows use longer batches than interpreter/JSON rows. Desktop and
hosted-runner conditions prevent controlled speedup or cross-platform claims.

### macOS ARM64

| Workload | Mode | Median operations/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| cpu.sum | interpreter | 5.05 | 4.98–5.08 | 4.426 |
| cpu.sum | jit | 41917.63 | 41337.75–42001.13 | 3.626 |
| cpu.sum | aot | 41831.46 | 41122.51–41852.21 | 0.007 |
| json.serialize | interpreter | 14.40 | 14.38–14.45 | 4.433 |
| json.serialize | jit | 725.64 | 721.61–729.68 | 3.574 |
| json.serialize | aot | 699.13 | 693.42–766.44 | 0.007 |
| json.deserialize | interpreter | 11.86 | 11.26–11.88 | 4.439 |
| json.deserialize | jit | 523.88 | 518.46–524.37 | 3.573 |
| json.deserialize | aot | 516.92 | 511.69–549.59 | 0.007 |

[Raw macOS reports and conditions](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/release-20261022/macos-preview17).

### Windows x64

| Workload | Mode | Median operations/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| cpu.sum | interpreter | 6.37 | 6.18–6.62 | 6.050 |
| cpu.sum | jit | 54422.27 | 51174.01–54535.53 | 5.165 |
| cpu.sum | aot | 54349.18 | 46513.73–54590.73 | 0.010 |
| json.serialize | interpreter | 12.34 | 12.08–12.53 | 5.943 |
| json.serialize | jit | 366.93 | 349.91–367.73 | 5.172 |
| json.serialize | aot | 367.67 | 350.12–369.09 | 0.010 |
| json.deserialize | interpreter | 9.85 | 8.44–10.08 | 5.964 |
| json.deserialize | jit | 269.52 | 254.45–271.13 | 5.176 |
| json.deserialize | aot | 270.52 | 250.29–272.16 | 0.010 |

[Raw Windows reports and conditions](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/release-20261023/windows-preview17).
Both runs record only generated Python caches in their initial Git status;
no tracked input changes occurred. Earlier series below retain their original
provenance and limitations. These results do not measure collector comparisons,
peak concurrent throughput or HTTPS performance.

## Earlier measurements

CPU and JSON workloads run through the same Raven worker in interpreter, JIT and
AOT. These **18 October 2026, Apple M1/macOS 27.0** measurements use the Preview 16 package with Raven `c0854cdda`. All 45 CPU/JSON processes and ten longer
native CPU processes pass correctness and unchanged-input checks. Windows results follow below as a separate hosted-runner series.

## Complete SDK development refresh — 22 October 2026

The complete macOS ARM64 development SDK passes the entire sequential benchmark
runner: 55 CPU/JSON processes and 1,152 HTTP responses, with unchanged-input checks.
Apple M1/macOS 27.0; Raven compiler `301e668a5`. This is a development candidate,
not a published release or a controlled comparison with the earlier snapshot.

| Workload | Mode | Median operations/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| cpu.sum | interpreter | 5.06 | 4.69–5.23 | 4.405 |
| cpu.sum | jit | 42206.91 | 42046.77–42822.33 | 3.565 |
| cpu.sum | aot | 42007.71 | 41375.75–42155.22 | 0.007 |
| json.serialize | interpreter | 14.63 | 14.33–14.67 | 4.391 |
| json.serialize | jit | 720.73 | 670.15–727.86 | 3.553 |
| json.serialize | aot | 707.77 | 421.02–776.19 | 0.007 |
| json.deserialize | interpreter | 11.95 | 11.50–11.99 | 4.377 |
| json.deserialize | jit | 523.30 | 519.97–529.01 | 3.562 |
| json.deserialize | aot | 514.11 | 511.07–519.37 | 0.007 |

Five measured rounds rotate modes. CPU/JSON uses 16 operations and two warmups;
native CPU rows use a separate 10,000-operation, 20-warmup run. Each HTTP process
validates 32 responses, discarding the first eight from timing; each mode also has
one whole-process warmup. Requests are sequential loopback HTTP/1.1 with connection
close, using a Python client on the same machine. Startup is separate and includes
JIT compilation. No unrelated local builds or full suites overlapped timed workers;
desktop and OS activity remained present. These results do not establish controlled
speedups, peak concurrency, allocation rates, GC comparisons or HTTPS throughput.

[Candidate identity and raw reports](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/release-20261022/macos-complete).
The [earlier development measurements](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/release-20261022/macos) are preserved. Windows qualification and the matching performance run have passed; results follow below.

## Windows development refresh — 22 October 2026

Windows x64 hosted runner, Windows Server build 26100, AMD64 Family 25 Model 1.
All five aggregate stages pass: 55 CPU/JSON processes and 1,152 HTTP responses.
This series uses the package qualified by run 39841298887 (original package run
39835531991), with benchmark source `69441f2fd`. It is separate from macOS and
published Preview 16 measurements; host differences prevent platform rankings.

| Workload | Mode | Median operations/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| cpu.sum | interpreter | 5.08 | 4.48–5.34 | 7.527 |
| cpu.sum | jit | 42245.41 | 41560.15–42727.94 | 6.219 |
| cpu.sum | aot | 43193.06 | 43100.68–43290.72 | 0.012 |
| json.serialize | interpreter | 9.60 | 8.22–9.98 | 7.634 |
| json.serialize | jit | 281.55 | 198.75–287.40 | 6.376 |
| json.serialize | aot | 251.35 | 195.55–285.68 | 0.013 |
| json.deserialize | interpreter | 7.47 | 6.74–7.77 | 7.685 |
| json.deserialize | jit | 205.44 | 136.87–213.24 | 6.765 |
| json.deserialize | aot | 199.89 | 184.79–216.20 | 0.014 |

Five rounds rotate modes. CPU/JSON uses 16 operations and two warmups; native CPU
uses a separate 10,000-operation, 20-warmup series. HTTP validates 32 responses per
process, discards the first eight from timing and uses one whole-process warmup
per mode. Requests are sequential loopback HTTP/1.1 with connection close. Startup
is separate from throughput and includes JIT compilation. These results do not
measure peak concurrency, allocation/GC rates, HTTPS throughput or production load.

The aggregate reports a dirty checkout. This is retained, not relabeled clean:
an audit of recorded source hashes matches 85 tracked inputs at the recorded
revision (exact bytes or Windows CRLF checkout). Two generated native archives
are hashed separately and have no tracked-source equivalent. The raw report does
not retain the dirty path list, so its cause is not established. Every measurement's
input-integrity and correctness checks pass; no controlled speedup is claimed.

[Candidate identity, source audit and raw reports](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/release-20261022/windows).

## Published Preview 16 series

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
