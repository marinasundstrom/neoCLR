# Development candidate measurements — 2026-10-22

Apple M1/macOS 27.0, refreshed macOS ARM64 JIT bundle with current Raven-owned
libraries. This is a development candidate, not a published release. Bundle manifest
SHA-256: `d1449571bc51af327a14b9a7d0b90e229d6e0bffd03d60d9e9cf676fbe127c91`.
[Extracted package qualification](../../../docs/validation/release-stabilization-20261022/extracted-jit.json).

All 55 CPU/JSON processes and 1,152 HTTP responses pass correctness and unchanged
input checks. Five measured rounds rotate modes. CPU/JSON uses 16 operations and
two warmups; the reported native CPU rows use the separate 10,000-operation,
20-warmup run. Each HTTP process validates 32 responses, discarding the first eight
from timing; each mode also has one whole-process warmup. Requests are sequential,
loopback HTTP/1.1 with connection close, using a Python client on the same machine.

No local builds or full suites overlapped timed workers. The desktop and OS services
remained active; this is not an idle-host comparison, controlled regression threshold,
peak concurrency result or comparison between macOS and Windows. Startup includes
worker setup and compilation for JIT. Allocation/GC rates and HTTPS are not measured.

## CPU and JSON

| Workload | Mode | Median operations/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| cpu.sum | interpreter | 4.90 | 4.85–4.93 | 4.689 |
| cpu.sum | jit | 39849.60 | 39016.93–39904.81 | 3.783 |
| cpu.sum | aot | 39771.50 | 39631.77–39896.90 | 0.006 |
| json.serialize | interpreter | 13.60 | 13.59–13.70 | 4.682 |
| json.serialize | jit | 687.62 | 679.33–690.59 | 3.791 |
| json.serialize | aot | 721.98 | 720.69–723.66 | 0.015 |
| json.deserialize | interpreter | 11.19 | 11.17–11.25 | 4.666 |
| json.deserialize | jit | 494.10 | 491.10–501.68 | 3.807 |
| json.deserialize | aot | 518.33 | 514.60–522.75 | 0.015 |

## HTTP

| Workload | Mode | Median requests/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| http-plain | interpreter | 3.56 | 3.44–3.57 | 4.506 |
| http-plain | jit | 178.74 | 177.77–190.22 | 7.041 |
| http-plain | aot | 188.82 | 188.08–199.09 | 0.007 |
| http-json | interpreter | 1.74 | 1.73–1.74 | 4.535 |
| http-json | jit | 96.22 | 95.34–99.56 | 11.484 |
| http-json | aot | 99.62 | 99.36–104.01 | 0.013 |

## Provenance and fixture repairs

The aggregate runner started at `cac555e04`; its CPU/JSON phases passed but HTTP
compilation failed on removed `HttpContent.FromText`. The plain fixture now creates
`HttpStringContent` directly. JSON initially produced the correct body with the wrong
Content-Type because content metadata takes precedence over the header list; its
text content now explicitly selects `application/json; charset=utf-8`. It still
serializes the same Person into text before constructing content, preserving the
workload. Both HTTP reruns record source hashes, on `bf4c404bd` plus these fixture
edits. Library/compiler/runtime inputs were unchanged. The original aggregate report
remains failed; the combined results above use the four independently passing reports.

Raw reports are compressed with their uncompressed and compressed hashes in
[summary.json](summary.json). Failed compilation/warmup reports are retained and
excluded from passing counts. The prior [Preview 16 series](../../release-20261018/macos/README.md)
remains unchanged. CPU interpreter cost and startup increased while JSON and HTTP
rates increased in this observation; changed inputs and host load prevent assigning
causes or claiming controlled improvements. Windows refresh remains separate.
