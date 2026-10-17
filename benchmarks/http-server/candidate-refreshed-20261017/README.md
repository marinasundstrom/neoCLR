# Refreshed native HTTP candidate — 2026-10-17

The `.20261017.4` macOS ARM64 package passes both plain-text and typed-JSON
HTTP workloads in interpreter, JIT and AOT: 36 processes and 1,152 validated
responses, with zero errors. [Package identity](../../../docs/validation/release-candidate-20261017/inference-fixed-identity.json),
[plain samples](plain.json) and [JSON samples](json.json) retain exact provenance.

Each mode uses one whole-process warmup and five measured processes, rotating
mode order. Each process validates 32 responses and discards the first eight from
timing. Startup is reported separately. These are sequential loopback requests
with connection close and a Python client on the same Apple M1 host; they do not
measure concurrent service capacity. Builds completed before timing and no other
local build or benchmark ran concurrently. Host load and input hashes are in each
report. Active macOS indexing and this exploratory fixed-count harness preclude
controlled regression or cross-platform speed claims. Windows remains separate.

| Workload | Mode | Median requests/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| plain | interpreter | 2.22 | 2.18–2.27 | 2.717 |
| plain | jit | 150.52 | 143.73–150.67 | 5.056 |
| plain | aot | 156.91 | 148.10–157.58 | 0.005 |
| json | interpreter | 1.20 | 1.19–1.20 | 2.721 |
| json | jit | 81.42 | 78.25–82.13 | 8.388 |
| json | aot | 84.65 | 81.78–85.16 | 0.013 |
