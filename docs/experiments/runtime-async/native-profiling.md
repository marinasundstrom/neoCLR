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
The private collector counters are also qualified on Windows x64 as of 2026-10-13:
Windows uses `QueryPerformanceCounter`, while macOS uses `CLOCK_MONOTONIC`.
`python scripts/validate-native-gc.py --output NEW_DIRECTORY` runs the accounting,
root rejection and allocation-admission checks (on Windows, use an MSVC developer
prompt). [Retained Windows evidence](../../../benchmarks/http-server/windows-root-gc-validation-20261013.json)
checks both profiling-enabled and ordinary builds. This does not port the macOS
sampling driver or Unix-socket attachment transport to Windows.

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

## Fewer collector traversals — 2026-10-12

The first parked-stack-only experiment passed correctness but did not establish a
clear timing improvement. The completed slice also validates descriptors while
building the address index and resets marks while seeding intern roots. Every
validation check remains, collection frequency is unchanged, and each owned parked
registration is traced once after its owner is marked. See the
[collector design](../aot-console/native-gc.md#trace-each-owned-parked-stack-once--2026-10-12).

The [controlled AOT comparison](collector-walk-comparison.json) relinks identical
runtime-async guest objects against baseline collector `2ea1f796` and the changed
collector, with the same current host and headers. Six rounds alternate variant
order; each process runs 40 invocations, dropping its first from timing summaries.
Each cell retains 234 timings. All 960 invocations verify reclamation and stack release.

| 100-operation workload | Baseline median | Optimized median | Reduction |
| --- | ---: | ---: | ---: |
| completed awaits | 18.87 ms | 16.85 ms | 10.7% |
| pending awaits | 45.35 ms | 39.66 ms | 12.6% |

Reproduce from the repository root with a passing benchmark recipe:

```sh
python3 docs/experiments/runtime-async/compare_collectors.py \
  --benchmark target/collector-walk-benchmark/results.json \
  --baseline 2ea1f796 --output target/collector-comparison-new
```

The [six-round JIT/AOT rerun](collector-walk-benchmark.json) also retains compiler
state-machine controls. Runtime medians are 19.01/18.27 ms (JIT/AOT completed) and
47.26/42.58 ms (pending). It contains substantial timing noise, including a 146 ms
pending JIT outlier. Ratios to state machines range 1.15–1.58; do not treat these
noisy ratios as proof that the lowering gap has closed. The controlled same-object
AOT comparison supports the collector-specific improvement, not a Windows or .NET
performance claim. Other applications and interpreter performance are not inferred.

The [new instrumented profile](collector-walk-profile.json) runs 200 checked
invocations: collection accounts for about 64% of diagnostic time, with unchanged
26,664 attempts, 6,617 allocations and 802 resumes per invocation. Collection remains
a major cost. Investigate pacing separately rather than quietly reducing stress
coverage as part of this optimization.

[Sanitized validation](collector-walk-validation.json) covers descriptor rejection
without mutation, multiple forward-pointing intern roots, repeated GC, reverse owner
discovery, shared parked owners, changed slots on the next collection, failure/retry,
unrooted cycles, host handles and 320 owned native suspensions. Existing precise
boundary and forged-header tests remain active. No guest public API changes.

[Native Raven generators](collector-walk-generators.json) pass interpreter/JIT/AOT.
The [HTTP/await showcase](collector-walk-showcase.json) passes JIT and AOT language,
cleanup, cancellation, HTTP client/server and lifecycle checks with the new collector.
An initial replay selected an older interpreter without managed-body support; the
recorded successful run uses the qualified owned-runtime recipe. These are behavior
checks, not HTTP throughput measurements or refreshed Windows qualification.

## Payload and pause measurements — 2026-10-12

The [instrumented report](gc-pause-profile.json) adds requested allocation bytes,
post-collection live payload/block gauges, peak arena usage and a bounded 64-bucket
logarithmic histogram of timed collection attempts. All updates compile out without
`NEOCLR_GC_PROFILE`. The private C diagnostic struct changed: rebuild matching hosts
and collector objects together. No managed public API or collector policy changes.

The pending-100 runtime lowering runs 200 invocations and excludes the first from
summaries. Across 199 retained invocations, 5,306,136 collection attempts yield p50
at most 1,023 ns and p95/p99 at most 2,047 ns. These are histogram upper bounds,
weighted by individual attempts. Zero/sub-resolution timings occupy bucket zero.
The maximum timed attempt is 40.805 ms; sampling, scheduling and instrumentation
can contribute to this tail, so it is not an isolated collector latency measurement.

Median allocated payload per invocation is 194,373 bytes; observed peak live payload
**after collection** is 1,785 bytes and peak arena usage is 4,024 bytes. Payload
excludes headers, padding and free spans. Gauges describe the last successful
collection; peaks are maxima across contexts on one thread, not simultaneous totals.
Private harness resets happen between invocations and never alter heap state.

Median instrumented invocation time is 42.122 ms, about 63.1% in collection, with
26,664 attempts for 6,617 allocations and 802 resumes. This motivates investigating
pacing for small live heaps; it establishes no new speedup or production pause SLA.
All invocations balance allocation/reclamation and release native stacks.

[Sanitized validation](gc-pause-validation.json) covers enabled/disabled accounting,
reset without heap mutation, failed collection, rooted payload survival and release,
plus existing precise roots and 320 owned native suspensions. Three Python summary
tests cover weighted quantiles, bucket boundaries and rejection of inconsistent or
unreclaimed samples. Reproduce with:

```sh
python3 -m unittest discover -s docs/experiments/runtime-async -p test_profile_summary.py
python3 docs/experiments/runtime-async/validate_conditional_roots.py \
  --recipe docs/experiments/runtime-async/collector-walk-generators.json \
  --output target/gc-pause-validation --owned-bodies
python3 docs/experiments/runtime-async/profile_native.py \
  --benchmark target/collector-walk-benchmark/results.json \
  --case pending-100 --lowering runtime --output target/gc-pause-profile
```

Raw local outputs are identified by hashes in the report. These measurements remain
part of the build-and-sample harness, not a runtime attach protocol.

See [attachable runtime diagnostics](../../runtime-diagnostics.md) for the separate
external debugger/profiler architecture and the next read-only attachment gate.

The [provisional HTTP attachment proof](../runtime-diagnostics/README.md) now passes
on macOS ARM64 AOT/JIT: a separate client reads coherent owner-thread snapshots while
the server handles requests. This complements the build-and-sample workflow; CPU
stack sampling is not yet exposed through the attachment endpoint.

## Native object-lookup counters — 2026-10-24

`NEOCLR_GC_PROFILE` also enables the separate private
`neoclr_gc_lookup_profile_v1()` snapshot and
`neoclr_gc_lookup_profile_reset_v1()` operation. Existing pause/profile structures
and reset behavior are unchanged. Counters are cumulative per native thread and
include failed operations; reset between operations, never during collection.
Disabled builds return zero counters and compile out increments.

| Counter | Work counted |
| --- | --- |
| `base_calls` | Calls reaching exact-base validation |
| `base_cache_hits` | Owned-cache fast returns; null operands are already admitted |
| `base_walks`, `base_spans` | Slow validation attempts and descriptors visited, including trailing integrity checks |
| `index_build_spans` | Descriptors visited in the initial validated index-building pass; excludes the dense-table rebuild |
| `index_queries`, `index_outside` | Address-index queries and immediate out-of-arena rejection |
| `index_probes` | Block descriptors examined after an in-range bucket lookup |
| `mark_candidates` | Words submitted to marking, including roots and payloads |
| `payload_matches` | Allocated payload candidates, including already-marked/interior references |
| `newly_marked` | First marks of live allocations |
| `object_words` | Ordinary-object words scanned after the leading object word |

These are work counts, not timings or distinct-object counts. `index_queries` also
includes exact-base root validation, so it differs from `mark_candidates`.
The interpreter uses its own heap and is not covered by these native counters.
[Plain/JSON/ordinary-workload evidence and cache retention](../../../benchmarks/execution/lookup-profile/README.md)
keep diagnostic runs separate from uninstrumented capacity measurements. New tests
are registered in `validate-native-gc.py`; local qualification covers macOS ARM64,
not a new Windows execution claim.
