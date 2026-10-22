# Preview 17 Windows measurements — retained 2026-10-23

Run 39859668223 passes all five stages against package run 39855375522:
55 CPU/JSON processes and 1,152 HTTP responses. Windows Server build 26100,
AMD64 hosted runner. SDK/editor `0.1.17-neoclr.20261022.1`.

The five-round methodology matches the [macOS candidate series](../../release-20261022/macos-preview17/README.md),
including separate startup, longer native CPU batches, sequential loopback HTTP/1.1
and connection close. Different hosts do not support platform rankings.
Initial/final Git status lists generated Python caches only; no tracked source
changes are reported. This resolves provenance for this run, not the missing dirty
paths in the earlier development report. Raw samples retain variance and input
hashes. These are not controlled speedup, allocation, GC or HTTPS measurements.

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

| Workload | Mode | Median requests/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| http-plain | interpreter | 3.27 | 3.23–3.29 | 5.778 |
| http-plain | jit | 62.51 | 52.97–65.90 | 9.468 |
| http-plain | aot | 59.81 | 52.88–66.52 | 0.008 |
| http-json | interpreter | 1.60 | 1.51–1.66 | 5.901 |
| http-json | jit | 42.73 | 39.52–46.61 | 15.525 |
| http-json | aot | 40.04 | 37.55–41.05 | 0.008 |

[Reports and identities](summary.json).
