# Preview 18 Windows measurements — 2026-10-26

[Hosted run 40298846341](https://github.com/marinasundstrom/neoCLR/actions/runs/40298846341)
passes all five stages: 55 CPU/JSON processes and 1,152 validated HTTP responses.
It uses the unchanged archive from package run 40213940464, requalified by
40298792405, with SDK/editor `0.1.18-neoclr.20261025.1` and Raven `ed1d08def`.
Windows Server 2025, AMD64 Family 25 Model 1 Stepping 1; source `f2ca92260`.
Git records only generated Python caches before/after measurement, with no tracked
input changes. Hosted-runner conditions are not controlled between releases.

Five measured rounds follow the [standard release procedure](../../release/README.md).
Startup is separate, native CPU uses longer batches, and HTTP uses sequential
loopback requests with connection close. Synchronous GC is the package default.
These results do not establish peak concurrency, a GC comparison, allocation rates
or a controlled speedup. Preview 17 ran on AMD Family 25 Model 17, while this
run reports Model 1; the native CPU control is about 20% lower and HTTP medians
are also lower. The host difference prevents attributing that change solely to
the runtime, and does not rule out a runtime regression. The macOS provider-inclusive rebuild is qualified separately.

| Workload | Mode | Median operations/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| cpu.sum | interpreter | 5.42 | 4.99–5.56 | 5.607 |
| cpu.sum | jit | 43724.45 | 43270.52–43834.43 | 5.355 |
| cpu.sum | aot | 43544.85 | 42850.26–43785.18 | 0.011 |
| json.serialize | interpreter | 10.23 | 10.05–10.30 | 5.605 |
| json.serialize | jit | 347.94 | 342.47–349.52 | 5.387 |
| json.serialize | aot | 355.84 | 343.95–359.20 | 0.011 |
| json.deserialize | interpreter | 10.11 | 10.04–10.13 | 5.627 |
| json.deserialize | jit | 244.62 | 242.83–246.85 | 5.356 |
| json.deserialize | aot | 251.45 | 250.76–252.58 | 0.011 |

| Workload | Mode | Median requests/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| http-plain | interpreter | 2.71 | 2.65–2.76 | 5.408 |
| http-plain | jit | 42.10 | 37.55–42.15 | 11.416 |
| http-plain | aot | 37.59 | 36.93–39.86 | 0.009 |
| http-json | interpreter | 1.59 | 1.59–1.62 | 5.422 |
| http-json | jit | 27.30 | 26.55–28.14 | 18.664 |
| http-json | aot | 27.55 | 24.46–30.76 | 0.009 |

[Raw reports and identities](summary.json).
