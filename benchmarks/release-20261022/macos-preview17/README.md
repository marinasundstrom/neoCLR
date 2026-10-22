# Preview 17 candidate macOS measurements — 2026-10-22

All five stages pass against the installed `0.1.17-neoclr.20261022.1` candidate:
55 CPU/JSON processes and 1,152 validated HTTP responses. This is not a published
release. Source revision `05d3cf2e6`, Apple M1/macOS 27.0.

The [methodology](../macos-complete/README.md) is unchanged: five measured rounds,
rotated execution modes, separate startup and longer native CPU batches, sequential
loopback HTTP/1.1 with connection close. No unrelated builds overlapped measurements;
desktop/OS activity and read-only qualification-report retrieval remained present.
The initial Git status records only a generated Python cache; it was removed before
the final clean status. No tracked source changes occurred during measurement.

These results do not establish a controlled speedup or regression, concurrent
throughput, allocation rates, collector comparisons or HTTPS performance.
The earlier development measurements remain separate.

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

| Workload | Mode | Median requests/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| http-plain | interpreter | 3.70 | 3.53–3.78 | 4.282 |
| http-plain | jit | 180.88 | 176.26–189.44 | 6.725 |
| http-plain | aot | 188.84 | 134.97–199.28 | 0.005 |
| http-json | interpreter | 1.71 | 1.69–1.82 | 4.537 |
| http-json | jit | 96.31 | 95.80–100.79 | 11.456 |
| http-json | aot | 99.59 | 99.56–100.45 | 0.014 |

[Raw reports and identities](summary.json) retain all results, including the short
CPU batches whose timer-scale variation is why the longer native rows are shown.
