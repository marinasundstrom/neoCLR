# Final macOS package measurements — 2026-10-18

Package `0.1.16-neoclr.20261017.1`, Raven `c0854cdda`, Apple M1/macOS 27.0.
[Identity and qualification](../../../docs/validation/release-20261018/README.md).
All 55 CPU/JSON processes and 1,152 HTTP responses pass correctness checks.
Complete raw reports are gzip-compressed to retain large build diagnostics without
inflating the source archive. [Summary and raw-content SHA-256 hashes](summary.json)
identify each report. Decompress with Python's gzip module or `gzip -dc`.

No local builds overlap timing. These are sequential interactive-host measurements,
with active desktop/indexing load; they do not establish controlled speedups,
production capacity or a cross-platform ranking. Startup is separate from throughput.

## CPU and JSON

Five rounds rotate modes. Interpreter CPU/JSON use 16 operations and two warmups;
native CPU uses 10,000 operations and 20 warmups. The smaller native CPU batches
remain pilot data in the full report, not the rows below.

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

[CPU/JSON raw samples](execution.json.gz) · [Long native CPU samples](cpu-native-longer.json.gz)

## HTTP

Each mode uses one whole-process warmup and five measured processes. Each validates
32 responses, discarding the first eight from timing. Plain text returns a ten-byte
UTF-8 body; JSON creates and serializes a Person per request. All framing, status
and body checks pass. The local Python client opens one connection per request.

| Workload | Mode | Median requests/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| http-plain | interpreter | 2.16 | 2.11–2.19 | 2.901 |
| http-plain | jit | 142.79 | 137.54–150.22 | 5.364 |
| http-plain | aot | 147.21 | 143.48–155.82 | 0.006 |
| http-json | interpreter | 1.17 | 1.16–1.18 | 2.782 |
| http-json | jit | 79.87 | 79.55–81.48 | 8.432 |
| http-json | aot | 83.18 | 78.17–83.39 | 0.005 |

[Plain samples](http-plain.json.gz) · [JSON samples](http-json.json.gz)

The retained [failed JSON warmup](http-json-stale-roots-failure.json.gz) used stale
reflection roots from a preliminary compilation. The passing run resolves roots
from its final assembly; see [diagnosis](../../../docs/validation/release-20261018/stale-http-reflection-roots.json).
The failed run is excluded from passing counts and summaries.

The interpreter CPU median remains near the range-guard recovery measurements.
[Preview 15 comparison](../../http-server/preview15-comparison-20261017/README.md)
uses the same host with an explicit historical range spelling. Different host-load
windows limit small-difference claims. Allocation/GC rates, sustained concurrency,
HTTPS and the optional PostgreSQL workload are not measured here.
