# Refreshed native candidate — 2026-10-17

Local candidate `.20261017.4`, Raven `40fcae6d0`, matching rebuilt native libraries,
and the previously qualified optimized runtime/backend. The [package identity](../../../docs/validation/release-candidate-20261017/inference-fixed-identity.json)
records exact artifact hashes. All 45 CPU/JSON processes and ten longer native CPU
processes pass checksums and input-hash checks. [Raw samples](results.json),
[longer native CPU samples](cpu-native-longer.json), [manifest](manifest.json) and
[host load](host-load.json) are retained.

Five rounds per mode; interpreter and JSON use two warmups and 16 operations per
process. Native CPU rows below use 20 warmups and 10,000 operations, avoiding the
sub-millisecond initial native CPU batches (retained as pilot data). Startup is
separate from externally timed batches. No builds ran concurrently. macOS indexing
was active; these are exploratory local measurements, not a controlled regression
threshold or a cross-platform comparison. Windows and HTTP results are separate.

| Workload | Mode | Median ops/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| cpu.sum | interpreter | 6.39 | 6.24–6.54 | 3.197 |
| cpu.sum | jit | 40931.74 | 39674.56–41490.56 | 2.635 |
| cpu.sum | aot | 41435.87 | 39618.40–41564.93 | 0.007 |
| json.serialize | interpreter | 4.00 | 3.92–4.07 | 3.209 |
| json.serialize | jit | 444.98 | 429.72–448.37 | 2.625 |
| json.serialize | aot | 464.31 | 451.37–471.34 | 0.009 |
| json.deserialize | interpreter | 4.21 | 4.07–4.24 | 3.176 |
| json.deserialize | jit | 346.38 | 345.70–348.03 | 2.654 |
| json.deserialize | aot | 361.11 | 359.39–367.03 | 0.016 |

The full-package interpreter CPU median is 6.39 operations/s, compared with the
earlier isolated compiler result of 6.31 and the earlier Preview 15 result of 6.45.
This is consistent with recovery of the redundant-range-guard regression; differing
host conditions preclude treating the small difference as a speedup claim.
