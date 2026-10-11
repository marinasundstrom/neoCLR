# Native async profiling — 2026-10-11

`profile_native.py` rebuilds a benchmark's native host with debug symbols, reuses
its guest object, and samples the repeated checked workload with macOS `sample`.
It records commands, hashes, stacks and per-invocation resource counts. This is
currently a private macOS ARM64 diagnostic tool, not a portable profiler or debugger.

```sh
python3 docs/experiments/runtime-async/profile_native.py \
  --benchmark target/task-owned-benchmark-final/results.json \
  --case pending-100 --lowering runtime --output target/async-profile
```

Use a fresh output directory. `--seconds` controls sampling duration and
`--repetitions` controls checked invocations. Too few repetitions can let the process
exit before sampling finishes. A successful report requires the expected invocation
count, zero guest output/failures, full heap reclamation and zero reserved stack bytes
at teardown. Source files can intentionally differ from the benchmark baseline;
the profile records their current hashes. Guest code must be regenerated separately
when changing the compiler/provider. The original benchmark remains unchanged.

`NEOCLR_GC_PROFILE` adds thread-local cumulative collector attempt/time counters.
They include validation, marking and sweeping, including failed attempts, but exclude
root publication and allocation. Timers compile out of normal builds. Sampled timing
is diagnostic: startup, sampling overhead, clock calls and OS scheduling can distort
it. Rerun `benchmark_modes.py` without instrumentation for performance conclusions.
Native guest symbols currently identify generated functions by number, not Raven
source; native debug/source mapping and suspended logical stacks remain open.

## Tooling priorities

The author explicitly requests tools that help understand and optimize the runtime,
and asks that debugger and GC tooling receive attention. The existing
[interactive debugger](../../debugger.md) provides interpreter frames, heap and GC
inspection. This sampling/collector tool is the first bounded native counterpart;
it does not implement native stepping, heap snapshots or an async-aware debugger.

Next diagnostic needs are allocation/live-heap and pause distributions, symbol-to-IL
mapping, then suspended-operation identities and logical stacks usable by both
profiling and debugging. Keep them attached to measurable runtime-async/HTTP issues.
Retain collector stress validation while evaluating collection pacing separately.
This complements the .NET distinction between execution profiling, GC diagnostics and
interactive debugging; it does not promise CLR diagnostic protocol compatibility.

## First profile

The [pending-100 baseline](native-profile-baseline.json) completes 200 checked
invocations. Each performs 26,664 collections for 6,617 allocations and 802 resumes.
Collector timers account for roughly 70% of invocation wall time (diagnostic,
clock-instrumented). Sampling independently identifies collector/marking code as
most of the workload's sampled execution. This justifies investigating collector
work before changing the carrier or adding a new scheduler abstraction. Sanitized
collector, host-root and 320 owned-body suspension tests pass with timers disabled
(`target/profile-collector-validation/validation.json`).
