# Final Windows package measurements — 2026-10-18

Package `0.1.16-neoclr.20261017.1`, Raven `c0854cdda`; hosted Windows x64,
Windows Server 2025 (10.0.26100), AMD Family 26 Model 2.
[Benchmark run](https://github.com/marinasundstrom/neoCLR/actions/runs/39075237776)
uses the exact archive from successful package run `39073227105`;
[archive hash](archive-hash.json) and [package run](package-run.json) retain identity.
All 55 CPU/JSON processes and 1,152 HTTP responses pass correctness checks.

Complete raw reports are gzip-compressed losslessly. [Summary](summary.json)
includes raw-content and compressed SHA-256 hashes. Builds precede measurement;
no build runs overlap timed processes on this runner. Shared hosted-runner load
is uncontrolled. These results are not a cross-platform ranking, controlled
regression threshold or production capacity measurement. Startup is separate.

## CPU and JSON

Five rounds rotate modes; interpreter CPU and JSON use 16 operations and two
warmups. Native CPU rows use 10,000 operations and 20 warmups. The smaller native
CPU pilot remains in the raw report but is excluded from these rows.

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

[CPU/JSON raw samples](execution.json.gz) · [Long native CPU samples](cpu-native-longer.json.gz)

## HTTP

Each mode uses one whole-process warmup and five measured fresh processes.
Each validates 32 responses, excluding the first eight from timing. The Python
client opens one loopback connection per request; plain text returns ten UTF-8
bytes and typed JSON constructs/serializes a Person per request. Reflection roots
come from the final compiled assembly and are shared by JIT/AOT preparation.

| Workload | Mode | Median requests/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| http-plain | interpreter | 2.08 | 1.90–2.21 | 3.389 |
| http-plain | jit | 62.35 | 45.59–69.80 | 6.353 |
| http-plain | aot | 57.90 | 51.11–64.95 | 0.007 |
| http-json | interpreter | 1.21 | 1.16–1.24 | 3.434 |
| http-json | jit | 44.89 | 42.93–54.00 | 8.808 |
| http-json | aot | 45.67 | 36.61–50.93 | 0.007 |

[Plain samples](http-plain.json.gz) · [JSON samples](http-json.json.gz)

The same-package Windows measurements have no matched Preview 15 baseline.
macOS measurements are a separate host series. Allocation/GC rates, sustained
concurrency, HTTPS throughput and optional PostgreSQL are not measured here.
