# Native async profiling — 2026-10-11

`profile_native.py` rebuilds a benchmark's native host with debug symbols, recompiles
its guest with a matching symbol inventory, and samples the repeated checked workload
with macOS `sample`.
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
the profile records their current hashes, including transitive local C dependencies.
The guest is recompiled from the recipe with the current backend; changed Raven
source/provider code still requires a new benchmark assembly/recipe. The original
benchmark remains unchanged.

`NEOCLR_GC_PROFILE` adds thread-local cumulative collector attempt/time counters.
They include validation, marking and sweeping, including failed attempts, but exclude
root publication and allocation. Timers compile out of normal builds. Sampled timing
is diagnostic: startup, sampling overhead, clock calls and OS scheduling can distort
it. Rerun `benchmark_modes.py` without instrumentation for performance conclusions.
The readable sample adds guest method names beside generated function numbers.
Native instruction/source mapping and suspended logical stacks remain open.

## Separate tools, shared diagnostics

The author suggests keeping debugger and profiler as different tools. Follow that
boundary: the profiler measures CPU, allocations, pauses and suspension behavior;
the debugger controls/inspects execution through breakpoints, stepping and values.
The sampling runner does not launch the interactive debugger. Shared source maps,
operation identities and diagnostic hooks belong below their separate commands and
interfaces. Sharing those facilities is a direction, not an implemented common
protocol. Profiling must not require debugger snapshots or per-instruction stepping.

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

## Indexed validation result

Collection reuses its already-built sparse address index for exact host-root,
dependent-owner and parked-owner checks. It does not remove descriptor checks,
change ownership or reduce collection frequency. [Sanitized validation](indexed-owner-validation.json)
includes index-bucket/free-span/adversarial-pointer coverage and 320 owned native
suspensions; [Raven generators](indexed-owner-generators.json) pass all three modes.

The [full matched rerun](indexed-owner-benchmark.json) showed noisy completed-100
JIT samples (20–110 ms). Keep that evidence, but do not claim a regression or speedup
from that cell. A [six-round native follow-up](indexed-owner-native-repeat.json),
alternating lowering/engine order, retains 18 samples per cell after process warmup.
It measures the same 100-operation inputs with instrumentation disabled:

| Await | Engine | State machine ms | Runtime async ms | Runtime / state machine |
| --- | --- | ---: | ---: | ---: |
| completed | JIT | 15.09 | 19.45 | 1.29× |
| completed | AOT | 14.28 | 18.28 | 1.28× |
| pending | JIT | 31.74 | 46.86 | 1.48× |
| pending | AOT | 30.38 | 44.34 | 1.46× |

Previous native ratios were about 1.55–1.56× completed and 1.81× pending.
Absolute runtime medians are about 19–27% lower than the previous matched run;
state-machine medians also vary, so prefer the within-run ratios. All invocation
allocation/reclamation counts balance and reserved stack bytes end at zero.
The full rerun retains interpreted controls (unchanged implementation); they remain
faster than state machines in this workload. These are local Apple M1 measurements,
not statistical guarantees, Windows qualification or HTTP throughput conclusions.

The native gap is reduced, not closed. Remaining profile costs include collection
marking, repeated root publication/validation, adapter allocations and carrier entry.
Collection pacing must be evaluated separately with stress-GC retained as a correctness
mode; reducing collection frequency alone is not proof of a better suspension design.
The benchmark now accepts `--engines jit aot --counts 100 --rounds 6` for a focused,
order-balanced follow-up without repeating unaffected interpreter work.

The [native HTTP/await showcase](indexed-owner-showcase.json) also passes JIT and
AOT language, cleanup, cancellation, HTTP client/server and terminal-Fault checks
after the index change. This checks behavior; it is not an HTTP throughput benchmark.

## Readable profiles and collector summaries — 2026-10-12

After the managed StackTrace slice, the profiler now writes:

- `selection.json` and `symbols.json`: the fresh native compilation inventory and
  an object/report-hash-bound map retaining exact identities alongside display names.
- `stacks.txt` and `stacks-readable.txt`: unchanged raw sampling and annotated guest
  names, including managed Task/Promise/provider methods. Unknown names stay intact.
- `samples.jsonl` and `profile.json`: every invocation and a summary excluding the
  first warmup, with median GC fraction, collection attempts, allocations and resumes.

The compiler's transitive local C dependencies are pinned via dependency output;
this includes implementation files included by the benchmark host. Input/object/report
changes during profiling fail validation. The command remains independent of the
debugger and does not claim file/line mappings for generated guest instructions.

The [qualified pending-100 run](readable-native-profile.json) completes 200 invocations
with balanced reclamation and zero retained native stacks. Readable names expose the
Task/Promise registration, provider wait and generated callback paths in the samples.
Collector timing still accounts for about two thirds of diagnostic invocation time:
26,664 attempts for 6,617 allocated blocks and 802 resumes per invocation. This keeps
collection work and its pacing as the next measured candidate. It does not establish
an additional speedup; use the uninstrumented matched results above for that claim.
Full raw and annotated outputs remain in the recorded local output directory, with
hashes in the checked-in report. Regenerate them using the recipe before comparing
changed implementations. Pause distributions, retention paths and native debugger
stepping remain open.
