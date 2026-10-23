# Grouped thread-local GC pacing state — 2026-10-24

## Why this experiment

After compiler-owned transient descriptors, a fresh source-matched four-owner
profile checks 47,421 plaintext responses. A byte-identical object rebuild verifies
method mapping. Among 10,324 worker self samples, TLS lookup accounts for 16.73%,
root publication 15.14%, collection 11.39% and strlen 4.47%. Of the TLS samples,
551 have `neoclr_gc_boundary_v1` as immediate caller, the largest group. These
wall-clock samples include blocked time and are not CPU shares or capacity results.
Coordinator samples are separated by the existing analyzer.

The prior retained-stack grouping did not establish a benefit and remains rejected.
This experiment instead groups the GC fields repeatedly resolved together on the
paced boundary path. It requires its own paired measurement rather than relying on
fewer assembly instructions as proof of an application improvement.

## Change and contract

Place GC statistics, pacing context/budgets and cleanup-active state in a single
private `_Thread_local` structure. Every field retains its existing lifetime and
updates. There is no whole-state reset, sharing across threads, new allocation,
changed object header, native ABI change, omitted check or collection-policy change.
The public statistics snapshot still returns the same value. Profiling reset does
not reset pacing or allocation/collection statistics. Distinct native contexts still
share their thread's cumulative epochs, with the same cross-context sweep behavior.

Optimized macOS ARM64 assembly reduces direct TLS-resolution sites in the boundary
function from four to one (three distinct TLS objects become one). The separate
root-head lookup remains. The grouped state has 72 bytes of payload versus 68 bytes
of separate fields: four bytes of trailing alignment padding; TLS linker overhead
is not included. Other thread-local heap, root and profiling storage is unchanged.

This uses the existing [CLR/Go/Java research](../../../../docs/design/allocation-metadata-runtime-comparison.md)
as implementation guidance for owner-local runtime state. It does not change .NET
API semantics or claim an equivalent collector or scheduling architecture. Native
TLS access cost is platform/compiler dependent; Windows performance is unmeasured.

## Validation

Extend the maintained pacing fixture with an optional pthread path: two fresh
threads start with zero allocation/collection counters, exercise both budgets and
cross-context collection epochs, then leave the main thread's counters unchanged.
Normal nonthreaded builds retain the original fixture. The focused runner exercises
pacing, invalid boundaries, cleanup/reentry and profile reset in optimized and
UBSan/bounds builds (six executable runs). Recompiled shared-worker graph/fault
checks and threaded heap differential checks pass in optimized/sanitized builds.
Forced-GC HTTP checks eight responses. These are local macOS ARM64 results.

No compiler source or generated-object change is required. The existing
compiler-owned-transient async-consumer evidence remains applicable; no new full
async suite is claimed. Two blank lines were removed after the first diagnostic
builds; capacity builds use the final source.

## Capacity methodology

Eight trials compare baseline `a6ef45c16` native-gc.c against the candidate with the
same owned-transient application object, host, owned heap, context reuse, GC budgets
and ten-byte response. One/four isolated owners run forward/reverse with 32
connections, two-second warmup and ten-second measured windows. CPU/response
includes startup, warmup and drain, subtracting client CPU. No builds, tests or
sampling overlap load. Short desktop runs do not establish sustained capacity,
JSON performance or Kestrel parity.

## Reproduction

Use the recorded Xcode SDK environment and new output paths:

```sh
python3 benchmarks/http-server/primitive-comparison/gc-state/validate.py --output NEW_DIRECTORY
python3 benchmarks/http-server/primitive-comparison/gc-state/measure.py \
  --build target/primitive-comparison-build/build.json \
  --object target/owned-transient/candidate.o \
  --client target/primitive-comparison-refresh/client --output NEW_DIRECTORY
```

The stack-profile capture/analyze runners reproduce the profile using the retained
build/compilation reports and their object identities. `assembly.json` records
compiler flags, assembly hashes and the complete boundary disassembly; the two
compressed assembly files preserve context. Raw validation and measurement reports
retain input identities and commands. Publication of website content remains separate.

## Result: retained

| Isolated owners | Baseline req/s | Candidate req/s | Change | Baseline CPU µs/response | Candidate CPU µs/response | Change |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 2,618.1 | 2,707.75 | +3.4% | 380.22 | 366.31 | −3.7% |
| 4 | 7,167.95 | 7,477.95 | +4.3% | 497.85 | 477.83 | −4.0% |

Means cover two samples per configuration. Both candidate samples beat both
respective baseline samples for throughput and CPU/response. One-owner p99 remains
mixed (baseline 13.79/18.53 ms, candidate 15.88/20.14 ms); four-owner candidate p99
is lower in these trials. No universal latency claim follows. Desktop conditions
are not a dedicated benchmark environment, and previous cohort numbers must not be
multiplied into a claimed cumulative speedup. The source-only change is retained.

The maintained native-GC validator now enables the threaded pacing fixture on
macOS; Windows retains the existing nonthreaded path. Python syntax checks pass.
No full platform matrix or unrelated website/API build was run.
