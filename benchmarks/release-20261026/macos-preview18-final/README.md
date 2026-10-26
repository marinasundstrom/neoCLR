# Preview 18 final macOS measurements — 2026-10-26

All five stages pass against the final PostgreSQL-inclusive archive: 55 CPU/JSON
processes and 1,152 HTTP responses. SDK/editor `0.1.18-neoclr.20261025.1`, Raven
`ed1d08def`, Apple M1/macOS 27.0.1, benchmark source `dce8207ca`.
Git was clean before and after. Builds, editor tests and the disposable SQL server
completed before timing; ordinary desktop/OS activity remained present.

Five measured rounds use separate startup, longer native CPU batches, and sequential
loopback HTTP/1.1 with connection close. Synchronous GC remains default. These are
not controlled speedup, peak concurrency, allocation or collector-comparison results.
The [earlier base-only candidate](../macos-preview18/README.md) is retained separately
and does not identify the released macOS package. See the [procedure](../../release/README.md).

| Workload | Mode | Median operations/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| cpu.sum | interpreter | 4.94 | 2.59–5.06 | 3.266 |
| cpu.sum | jit | 39392.19 | 38837.47–41515.34 | 3.576 |
| cpu.sum | aot | 40466.71 | 34296.61–41589.11 | 0.006 |
| json.serialize | interpreter | 12.94 | 11.85–13.37 | 3.366 |
| json.serialize | jit | 973.35 | 956.30–1021.27 | 3.360 |
| json.serialize | aot | 1035.81 | 959.68–1066.23 | 0.005 |
| json.deserialize | interpreter | 13.12 | 12.96–14.14 | 3.548 |
| json.deserialize | jit | 654.97 | 619.80–681.49 | 3.122 |
| json.deserialize | aot | 670.02 | 602.18–712.59 | 0.006 |

| Workload | Mode | Median requests/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| http-plain | interpreter | 3.53 | 3.43–3.67 | 3.314 |
| http-plain | jit | 185.20 | 164.70–188.13 | 6.950 |
| http-plain | aot | 192.97 | 178.38–193.74 | 0.005 |
| http-json | interpreter | 2.00 | 1.92–2.06 | 3.179 |
| http-json | jit | 103.27 | 90.92–106.06 | 13.283 |
| http-json | aot | 107.20 | 77.14–108.29 | 0.016 |

[Raw reports and identities](summary.json).
