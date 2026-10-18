# Sampled Serve GC phase costs

2026-10-18, macOS ARM64. Diagnostic only; production collector unchanged.
`capture.py` relinks the fixed callback-reservation guest with timing boundaries
in a generated copy of native-gc.c. One in 64 or 127 collections is sampled.
No logging or allocation occurs within collection hooks.

| Phase | CPU share, stride 64 | CPU share, stride 127 |
| --- | ---: | ---: |
| Index validation/build (including dense rebuild) | 15.97% | 15.70% |
| Validate roots/resources, fault-root discovery | 2.23% | 2.18% |
| Seed roots/reset marks | 7.37% | 7.14% |
| Trace graph and conditional-root fixed point | 59.59% | 60.39% |
| Expire dependent roots/resource cleanup | 1.60% | 1.57% |
| Sweep, poison and coalesce | 13.24% | 13.03% |

There were 308,493/308,498 collections and 4,820/2,429 sampled collections,
with no collector errors. Mean sampled CPU per collection was 11.95/12.14 µs.
The empty-hook control costs about 1.0 µs across six phases (about 0.17 µs
per boundary); these figures are **not subtracted**. Small phases are particularly
sensitive to instrumentation. Monotonic wall time and thread CPU use separate clock
reads; very short wall intervals may be less than CPU intervals due to resolution.

Totals include startup, warmup and shutdown, not just the request measurement.
Periodic sampling can alias workload cycles; agreement between two coprime strides
supports this ordering but is not a statistical confidence interval. No exact
unobserved total is inferred by multiplying samples by the stride.

Both 20-second 300/sec diagnostic runs completed all 6,000 arrivals. Offered p99
was 5.69/6.49 ms; generator-lag p99 was 0.81/0.83 ms. Every arrival and host condition
is retained in compressed request reports. This is not a new sustained-rate result,
an uninstrumented optimization comparison or a Go comparison.

## Decision

Graph tracing is the largest sampled collector cost. Reusing that walker on isolated
immutable data is worth investigating after worker infrastructure validation. Moving
only cleanup or root validation to a thread is unlikely to justify the lifecycle
complexity on this workload. This does not establish that copying the heap and paying
worker synchronization costs will improve Serve: the rejected snapshot prototype also
has a conditional-owner retention flaw. Keep cleanup and reclamation on the owner
until the reachability/publication protocol is validated.

This follows the existing [.NET/Go/other collector comparison](../../../../docs/design/concurrent-gc-evaluation.md).
It changes no managed semantics and selects no production collector design.

## Reproduce

Use the repository Xcode SDK environment, then run:

```
python3 benchmarks/http-server/serve/gc-phases-20261018/test_phases.py
python3 benchmarks/http-server/serve/gc-phases-20261018/capture.py --build BUILD --output NEW_DIRECTORY --stride 64
```

Repeat with stride 127. Build manifests retain exact source/object hashes and
commands. The tests check sampling counts, error accounting and fail-closed source
anchors; the two Serve runs exercise the instrumented collector with real requests.
This POSIX diagnostic is not enabled in shipped builds.
