# Preview 18 candidate macOS measurements — 2026-10-26

**Superseded candidate:** this archive omitted the optional PostgreSQL provider.
Use the [final provider-inclusive measurements](../macos-preview18-final/README.md)
for Preview 18. The raw results below remain unchanged.

All five stages pass against the extracted Preview 18 candidate, SDK/editor
`0.1.18-neoclr.20261025.1`, Raven `ed1d08def`: 55 CPU/JSON processes and
1,152 validated HTTP responses. Apple M1, macOS 27.0.1; source `f2ca92260`.
The checkout is clean before and after measurement. Unrelated local builds and
editor checks completed before timing; ordinary desktop activity and read-only
hosted-status checks remained present.

Five measured rounds, separate startup, longer native CPU batches and sequential
loopback HTTP/1.1 with connection close follow the [release procedure](../../release/README.md).
Synchronous GC remains the package default. This is not a controlled before/after
experiment, peak-concurrency result, allocation measurement or GC comparison.
Windows qualification/measurements and publication remain pending.

| Workload | Mode | Median operations/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| cpu.sum | interpreter | 5.04 | 4.99–5.13 | 3.038 |
| cpu.sum | jit | 41600.54 | 41521.33–42140.65 | 2.823 |
| cpu.sum | aot | 41846.66 | 40502.36–42425.25 | 0.005 |
| json.serialize | interpreter | 14.56 | 12.81–14.79 | 2.925 |
| json.serialize | jit | 1014.35 | 933.26–1019.33 | 2.759 |
| json.serialize | aot | 1069.03 | 984.58–1075.47 | 0.005 |
| json.deserialize | interpreter | 14.45 | 13.37–14.63 | 2.921 |
| json.deserialize | jit | 665.86 | 626.83–681.79 | 2.784 |
| json.deserialize | aot | 718.65 | 661.65–719.91 | 0.005 |

| Workload | Mode | Median requests/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| http-plain | interpreter | 3.85 | 3.76–3.97 | 2.811 |
| http-plain | jit | 190.08 | 183.04–195.91 | 6.275 |
| http-plain | aot | 200.06 | 192.50–205.16 | 0.005 |
| http-json | interpreter | 2.06 | 2.00–2.24 | 2.796 |
| http-json | jit | 110.67 | 107.69–111.70 | 10.747 |
| http-json | aot | 114.99 | 113.46–116.16 | 0.005 |

[Raw reports and identities](summary.json). Earlier release results remain unchanged.
