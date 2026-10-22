# Complete development SDK measurements — 2026-10-22

The complete macOS ARM64 development SDK passes the entire sequential benchmark
runner: 55 CPU/JSON processes and 1,152 HTTP responses, with unchanged-input checks.
Apple M1/macOS 27.0; Raven compiler `301e668a5`. This is a development candidate,
not a published release or a controlled comparison with the earlier snapshot.

Manifest SHA-256: `15c3ff3f20364b999fc31091b47046251ed0bf24258bc36208d27de204727a47`.
Archive SHA-256: `5749e3aa28590aeafec67940134db49fcf3762398f5c5d2ddaffbb86ff6346a6`.
Benchmark source starts clean at `5ea8b4328`.
[Package/editor qualification](../../../docs/validation/release-stabilization-20261022/complete-macos-package.json).

Five measured rounds rotate modes. CPU/JSON uses 16 operations and two warmups;
native CPU rows use a separate 10,000-operation, 20-warmup run. Each HTTP process
validates 32 responses, discarding the first eight from timing; each mode also has
one whole-process warmup. Requests are sequential loopback HTTP/1.1 with connection
close, using a Python client on the same machine. Startup is separate and includes
JIT compilation. No unrelated local builds or full suites overlapped timed workers;
desktop and OS activity remained present. These results do not establish controlled
speedups, peak concurrency, allocation rates, GC comparisons or HTTPS throughput.

## CPU and JSON

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

## HTTP

| Workload | Mode | Median requests/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| http-plain | interpreter | 3.76 | 3.58–3.78 | 4.244 |
| http-plain | jit | 189.95 | 189.69–190.90 | 6.685 |
| http-plain | aot | 199.38 | 199.20–199.67 | 0.006 |
| http-json | interpreter | 1.73 | 1.69–1.77 | 4.492 |
| http-json | jit | 96.82 | 92.56–101.17 | 11.658 |
| http-json | aot | 100.61 | 99.65–100.71 | 0.013 |

## Evidence

The aggregate runner passes all five stages. Compressed raw reports and their
uncompressed/compressed hashes are listed in [summary.json](summary.json).
The [earlier development snapshot](../macos/README.md) and published Preview 16
series remain unchanged. Rebuilt library bytes differ from the earlier snapshot;
measurements are tied to this archive. Windows remains a separate hosted series.
