# Scheduling investigation baseline refresh - 2026-10-28

The new author proposal prioritizes Go-inspired lightweight contexts, work stealing and worker-independent suspension. Before a new scheduler comparison, rebuild the actual existing native worker-pool safety suite and copied-primitive CPU screen against current sources. This is a baseline refresh, not a new scheduling implementation.

Optimized and fail-fast UBSan builds each pass all 11 native test groups, including root handoff, cancellation races, result backpressure, reader/mutator exclusion and teardown. UBSan is not a race detector; earlier TSan qualification remains incomplete and is not claimed here.

The existing benchmark performs eight identical 20-million-iteration integer jobs, checks every result, and runs worker counts 1/2/4 then 4/2/1. Pool setup, expected-value calculation and teardown are excluded; admission and completion waits are included. Captures/results use the shared heap gate; the long compute loop uses copied primitives outside it. Two observations per worker count are a screening baseline, not statistically robust production evidence.

| Workers | Wall median (s) | Observed range (s) | CPU median (s) |
| --- | ---: | ---: | ---: |
| 1 | 0.318524 | 0.304930-0.332119 | 0.306720 |
| 2 | 0.156768 | 0.156552-0.156984 | 0.312810 |
| 4 | 0.082361 | 0.081799-0.082923 | 0.324273 |

These results show copied-primitive CPU parallelism in the existing pool. They do not measure work stealing, queue latency, saved-context switching, managed GC throughput, async I/O or HTTP capacity. The private compiled shared-heap experiment is a separate baseline and is not rerun by this screen. No production worker policy or default changes.

Commands:

```sh
SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk python3 docs/experiments/native-worker-pool/validate.py --output target/scheduling-direction-20261028/validation
python3 docs/experiments/native-worker-pool/measure.py --validation target/scheduling-direction-20261028/validation/validation.json --output target/scheduling-direction-20261028/copied-cpu
```

Reports contain exact source hashes, compiler flags and raw samples. The host is Apple M1, eight logical CPUs and 16 GiB RAM, macOS 27.0.1. Timing follows completed compilation/validation, with no other task-owned benchmark or profiler running; desktop load is uncontrolled. [Investigation design and next implementation](../../design/go-inspired-scheduling.md) define the missing comparisons and ownership gates.

The documentation/proposal update passes all 26 website contract tests. The design’s local file links resolve. The execution tracer remains a documented next slice; this baseline contains no new trace capture. Website publication remains deferred.
