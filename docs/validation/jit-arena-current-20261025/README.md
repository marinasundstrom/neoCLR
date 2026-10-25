# Current JIT placement qualification — 2026-10-25

The [split-validator follow-up](../split-storage-validation-20261025/README.md)
leaves a large JIT/AOT gap. Source and disassembly inspection finds equivalent
native root-wrapper code in the executable and dylib. Existing
[call-layout evidence](../../../benchmarks/execution/call-layout/README.md) already
isolated long-range lowering/dynamic linking without reproducing that gap. The
[earlier arena experiment](../../../benchmarks/execution/jit-layout/README.md)
therefore provides a stronger placement hypothesis than removing native guards.
This follows existing runtime/CLR research without changing public API semantics.

## Implementation and boundaries

A test-only `NEOCLR_TEST_JIT_ARENA=1` selects the pinned Cranelift 0.121.2
`ArenaMemoryProvider` with a 64 MiB reservation. System allocation remains the
production default. Both providers now go through the existing Apple ARM64
instruction-cache publication wrapper; the arena must not bypass the correction
introduced in `efb4d3f8b`. Long-range calls, compiler optimization, roots, GC policy,
managed heap size and native host remain unchanged within each comparison.

The reservation covers JIT code and static data together. It is virtual address
space on this macOS implementation, not a claim of 64 MiB resident memory or a
managed-heap allocation. Placement results do not isolate instruction-cache, TLB
or data-cache effects. The pinned provider's Windows reservation also commits
memory; Windows is not qualified by these results. Large-image policy, fallback
and production configuration remain open. The flag is unavailable in non-test
compiler builds and is not a public runtime setting.

The existing `CodeOwner` owns JIT code through invocation and frees it before the
native library unloads. The wrapper tracks successful executable allocations,
invalidates them before final permissions, clears pending ranges after success,
and delegates allocation/protection/free to the selected provider. No GC or
mutator synchronization rules change.

## Safety qualification

Both system and arena runs pass all 200 active codegen tests; 34 external-fixture
tests remain ignored. Production compilation passes. New Apple ARM64 tests cover
128 repeated reserve/publish/execute/free lifetimes with two finalization phases,
existing code remaining callable after later publication, read-only and writable
static data, allocation exhaustion preserving pending publication, and idempotent
free. Another 128 lifetimes exercise allocation failure and cleanup before any
finalization. Existing repeated generated stack-transfer tests run under both
providers, retaining the cache-publication regression coverage.

These tests do not qualify unlimited image size, Windows reservation behavior or
all possible native host lifetimes. No production placement default changes.

## Matched workloads

`prepare-jit-placement.py` validates a retained qualified HTTP build and pairs
system/arena recipes using the same current runner and exact native host. `http.py`
clears any ambient placement switch before applying the recipe. The standalone
JIT harness's `--arena-placement` mode likewise uses the same native library for
both variants and keeps typed stores and the qualified stack policy fixed.
Reports retain source, runner and host hashes. Compilation and invocation CPU are
separate; all timing runs are sequential and unprofiled.

## HTTP results

Three interleaved rounds validate 98,688 responses across plain/JSON, both GCs
and both placement policies, four clients and pipeline depth 16.

| Handler | GC | System requests/s | Arena requests/s | CPU change |
| --- | --- | ---: | ---: | ---: |
| plain | sync | 4344 | 7398 | -41.2% |
| plain | live | 3762 | 5672 | -33.2% |
| json | sync | 2454 | 3714 | -34.0% |
| json | live | 2219 | 3190 | -30.2% |

The repeated effect is substantially larger than the split-validator gain. It
affects both collectors and therefore identifies a shared runtime placement cost,
not an improvement in concurrent marking itself. Sync remains faster. These are
short loopback cohorts, not saturation, peer-framework parity, pause distributions
or confidence intervals. Managed memory and native/JIT resident memory are distinct;
this report retains managed arena counters but does not establish native RSS.

## Standalone results

Four interleaved rounds pass 48 concurrent JIT invocations across all six standard
workloads; the first round is warmup. Exact output, GC lifecycle and matched
allocation counts/bytes are checked, including equality with the prior AOT work.

| Sample | System CPU ms | Arena CPU ms | Change |
| --- | ---: | ---: | ---: |
| compute | 64.900 | 64.928 | +0.04% |
| reuse-buffer | 55.416 | 53.066 | -4.24% |
| steady | 256.651 | 167.139 | -34.88% |
| bursts | 134.782 | 88.879 | -34.06% |
| retained-window | 491.929 | 336.937 | -31.51% |
| mixed-phases | 256.140 | 168.059 | -34.39% |

Managed allocation work is unchanged; the four allocation-heavy patterns improve
about 32–35%, buffer reuse 4.24%, and compute is effectively unchanged (+0.04%).
The repeated, workload-dependent gain supports a shared code/static-data placement
bottleneck. It does not establish a particular cache mechanism or make concurrent
GC faster than sync. Invalid placement flag values are explicitly rejected.

**Next:** retain the tested experiment, qualify large-image exhaustion/recovery,
native memory use and provider lifetime under representative hosts, and choose an
explicit bounded/fallback policy before promoting it. Continue using sync as a
comparison. Concurrent default readiness, pause advantage and wider worker/platform
qualification remain open. Production system allocation is unchanged.

Reproduction (use fresh output directories):

```sh
python3 benchmarks/gc-suite/matrix/prepare-jit-placement.py \
  --build target/gc-split-frame-build --runner target/jit-arena-current/runner \
  --output target/jit-arena-current/http-build
python3 benchmarks/gc-suite/matrix/http.py \
  --build target/jit-arena-current/http-build --output target/jit-arena-current/http-cost \
  --modes sync-jit arena-sync-jit live-jit arena-live-jit \
  --protocols pipeline --requests 4096 --rounds 3
python3 benchmarks/gc-suite/matrix/compare-scalar-jit.py --arena-placement \
  --build target/gc-patterns-standard-build --aot target/gc-split-frame-standalone \
  --runner target/jit-arena-current/runner --output target/jit-arena-current/standalone-cost
```

Build the current release codegen test runner first and pin it at the recorded
runner path. The retained native/toolchain recipes are prerequisites, not a claim
of clean-checkout benchmark bootstrapping. Raw commands, identities and per-trial
CPU/memory fields are in the compressed reports.
