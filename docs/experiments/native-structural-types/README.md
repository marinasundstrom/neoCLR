# Native structural type qualification

Development branch, 2026-10-15. This is integration evidence, not a release claim.

## Tuple AOT cost comparison

Runtime implementation: `4fe78429b`. Tool hashes and machine details are in the
raw result files. On Apple M1, the two-integer structural tuple and nominal record
both emit the **same 800-byte object**; sixteen-integer versions both emit the
**same 1,352-byte object**. Full object hashes match in each pair, not merely sizes.
Neither has undefined symbols, so these closed workloads cannot call a heap
allocator. Tuple construction uses inline slots with the same ABI as nominal records.

| Elements | Structural median | Nominal median | Structural / nominal |
| --- | ---: | ---: | ---: |
| 2 | 3.9608 ns/call | 3.9624 ns/call | 0.9996 |
| 16 | 13.124 ns/call | 13.122 ns/call | 1.0002 |

These differences are noise, not evidence of a speedup. Identical object code is
stronger evidence of equivalent cost for these workloads. The increased cost with
size is shared aggregate copy/call work and an optimization candidate for both
representations. Do not introduce a tuple-only optimization without measuring the
shared codegen path first.

Each workload constructs a value, passes it through Echo, copies it into a local
and reads the first and last elements. Inputs vary and the native host verifies
the accumulated result. Nine interleaved samples follow warmup, with five million
calls per pair sample and one million per sixteen-element sample. Timing uses the
monotonic clock around the loop, excluding compilation, linking and process launch.
The backend uses its normal Cranelift settings; the C timing host uses `-O2`.
Emission time is separately retained and is not a compilation benchmark.

Raw evidence: [pair](tuple-pair-results.json), [sixteen](tuple-16-results.json).
Exact generated inputs are retained next to these files. Reproduce with a matching
macOS ARM64 SDK (this host required Xcode's MacOSX26.2 SDK):

```sh
python3 docs/experiments/native-structural-types/benchmark-tuples.py \
  --codegen target/debug/neoclr-codegen --output target/tuple-cost-pair
python3 docs/experiments/native-structural-types/benchmark-tuples.py \
  --codegen target/debug/neoclr-codegen --elements 16 --iterations 1000000 \
  --output target/tuple-cost-16
```

The script preserves previous output directories. A flat nominal record currently
admits at most sixteen fields; larger comparisons need equally nested layouts.
There is no packed-layout, cache-pressure, managed-reference/GC, boxing, union,
interpreter or JIT qualification in this result. These remain explicit follow-ups.
