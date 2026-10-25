# Release compilation state before JIT invocation — 2026-10-25

Follow the [whole-process memory finding](../jit-fallback-cost-memory-20261025/README.md)
by separating managed JIT preparation from execution. This is a shared runtime
lifetime improvement; it does not change collector or placement defaults.

## Ownership change

`prepare_managed` returns a private `PreparedManaged` containing the finalized
JIT module owner, native library owner, entry/host pointers and scalar invocation
metadata. It borrows no source module, selection report or compiler options.
Generated static data is copied into module-owned allocations during emission;
imported native symbols remain protected by the retained library.

After preparation, `run_internal` drops compiler options, selected module/report,
runtime load context, dependencies and the original input module, then calls the
consuming `invoke` method. Code/static data and native library survive through
synchronous host cleanup; field order releases code before the explicit library
owner. Partial preparation and failed argument conversion still unwind through
the module owner. Arguments remain alive throughout the call, including Windows
UTF-16 backing storage. Existing host root/worker teardown requirements remain.

Production managed JIT uses early release. A test-only
`NEOCLR_TEST_KEEP_JIT_COMPILATION` control preserves the former lifetime for matched
comparison. JIT diagnostics report whether release occurred and its separate
startup duration; compilation and guest-invocation timings remain separate. No
public API, Raven/CLI metadata contract, GC trace semantics or call ABI changes.
This follows the existing runtime/CLR principle of keeping execution ownership
separate from compilation/loading machinery rather than retaining unused views.

## Qualification and memory

All 203 active codegen tests and production compilation pass; 34 external-fixture
tests remain ignored. Eight matched plain/JSON HTTP cases pass 8,320 responses,
using identical fallback placement and native hosts. The harness requires the
reported release policy to match each recipe. Both retained and released paths
exercise structured JSON and constant/static data after preparation.

| Handler | GC | Retained RSS MiB | Released RSS MiB | Release startup ms |
| --- | --- | ---: | ---: | ---: |
| Plain | Sync | 974.0 | 917.2 | 32.9 |
| Plain | Concurrent | 1097.1 | 1096.8 | 16.5 |
| JSON | Sync | 978.5 | 916.6 | 32.2 |
| JSON | Concurrent | 1090.9 | 1077.5 | 17.4 |

These are single whole-process snapshots after traffic, not allocator attribution,
peak memory or statistical bounds. The source containers are explicitly dropped,
but RSS does not uniformly fall: freed allocator pages can remain resident, and
other runtime state may remain. Do not attribute the remaining approximately
1 GiB to live source containers, or claim this change resolves that residency.
Timings in this memory cohort are diagnostic-only. Release moves existing cleanup
work before readiness; it does not make that work free.

The next diagnostic should distinguish allocated native bytes from resident or
reserved pages. The local macOS SDK's `malloc_zone_statistics` contract supports
summing all zones with a null zone; investigate that separately before claiming an
allocator/cache cause or invoking platform memory-pressure APIs.

## Invocation cost

Unprofiled concurrent HTTP passes 49,344 responses across twelve trials (three
rounds per handler/lifetime). Median results with identical hosts and fallback
placement:

| Handler | Retained RPS | Released RPS | Invocation CPU change |
| --- | ---: | ---: | ---: |
| Plain | 5,673 | 5,741 | −1.14% |
| JSON | 3,282 | 3,213 | +2.00% |

The six standalone patterns pass 48 invocations; medians exclude the first of four
rounds as warmup. CPU changes are compute +0.09%, reuse-buffer −0.10%, steady
−0.10%, bursts −0.34%, retained-window +1.26%, and mixed-phases −0.10%.
Managed allocation counts and allocated bytes match in both cohorts. Concurrent
peak arena sizes may vary with scheduling. These short comparisons are mixed and
do not establish a throughput improvement. Integrate the explicit lifetime cleanup
for bounded ownership and avoidance of unnecessary live compilation state, rather
than a claimed collector win. The observed JSON regression remains part of the
record; longer cohorts would be needed to distinguish noise from a persistent effect.

`http-cost.json.gz` and `standalone-cost.json.gz` retain raw trial measurements and
input identities. `http-memory.json.gz` is the separate diagnostic cohort;
`build.json.gz`, `tests.txt.gz` and `production-check.txt.gz` retain qualification.

## Reproduction

Build and pin the current release test runner, then use
`matrix/prepare-jit-lifetime.py --build target/gc-split-frame-build --runner
<current-runner> --output <fresh-build>`. Its `retained-sync-jit`,
`released-sync-jit`, `retained-live-jit` and `released-live-jit` recipes share the
same host/runner/fallback placement. Run `matrix/http.py` separately for memory
(`--process-memory`) and unprofiled timing. For the six standalone samples, use
`matrix/compare-scalar-jit.py --compilation-lifetime` with the standard build and
retained root-validation AOT recipes. All matrix paths are under
`benchmarks/gc-suite`. Raw reports retain source and artifact identities.
