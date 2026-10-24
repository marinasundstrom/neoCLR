# Exact-base cache capacity — 2026-10-24

The [post-pacing profile](../post-pacing-profile/README.md) identifies exact-base
validation as a residual cost. Compare 64 and 256 positive-only cache slots in the
owned native heap. Baseline is `cb6bccb27`; both use 4,096-publication pacing.
The candidate changes only the private cache capacity. Each extra native thread
uses 1,536 more TLS bytes on this 64-bit target; object headers and managed
allocations are unchanged. This is not a global address index or a metadata cache.

The existing ownership proof, exact-address equality, miss validation and reclamation
invalidation remain. Releasing ownership clears the cache. External/shared heaps
retain their checked path. Collisions cause misses, never admission of forged or
interior pointers. The expanded fixture uses 513 allocations to exceed both sizes,
checks survivors, reclaim/reuse, false payload headers and scope reset, and runs
optimized/sanitized with profiling enabled/disabled against both sources.

Four fixed-request cohorts compare 32→64 request deltas in JIT and AOT. Plain
calls remain 102/request: baseline visits 2,197 descriptors/request, candidate
915 AOT / 567 JIT (58–74% reduction). JSON calls remain 233/request: baseline
5,067 AOT / 5,049 JIT, candidate 3,410 / 2,907 (33–42% reduction). Address hashing
and process layout influence collisions; these are measured cohorts, not fixed
per-request constants or hardware cache-miss counts. HTTP bodies and completion
counts are checked. Profiling is excluded from capacity measurements.

`counts.py` uses the current binary-map objects and records actual collector source
identity; it does not modify the preexisting lookup-profile runner. `validate.py`
accepts an out-of-tree candidate source. `measure.py`/`jit.py` likewise accept
`--candidate-source`, use fixed baseline sources and header, and preserve matched
managed code. Reuse the publication-budget invocation arguments with that extra
option. Baseline/candidate trials run in forward/reverse order: two seconds warmup,
ten seconds measurement, 32 connections and GOMAXPROCS=2. AOT server-lifetime CPU
and JIT readiness-to-shutdown CPU are distinct windows. Do not pool their numbers.


## Retained change

Adopt 256 entries. The exact out-of-tree source measured here is preserved as
`candidate.c.gz` and is byte-identical to the integrated collector. Two paired
trials/configuration, no concurrent profiling or builds during load:

| Configuration | Requests/s, baseline → candidate | Throughput gain | CPU/response reduction |
|---|---|---|---|
| AOT one owner | 4,324 → 4,450 | 2.9% | 2.9% |
| AOT four owners | 11,060 → 11,400 | 3.1% | 3.1% |
| Owned JIT one owner | 2,279 → 2,352 | 3.2% | 3.0% |

Candidate throughput ranges exceed baseline ranges in all three configurations.
The baseline JIT trials vary appreciably; these are modest local gains, not
confidence intervals. p99 ranges overlap in all configurations, so no latency
improvement is established. JSON capacity is not measured. No interpreted GC,
metadata-loading, GC-policy or public API change is involved.

`summary.json` is generated from archived `capacity.json.gz`/`jit-capacity.json.gz`
by `../owned-root-chain/summarize.py`. Count/test reports and input hashes are
archived beside it. The integrated collector also passes the existing pacing,
cleanup-rejection, shared-worker graph/fault and async-consumer checks.

This follows the existing [.NET/Java/Go heap-index comparison](../../../docs/design/allocation-metadata-runtime-comparison.md):
a bounded owner-local positive cache amortizes validation but does not replace the
full checked walk on misses. Page/span indexes would address scaling more directly
at greater lifecycle and memory cost. Enlarging this cache indefinitely is not the
architectural plan; the measured 1.5 KiB/thread cost buys a small near-term gain.

The final validation runner extracts the fixed `cb6bccb27` baseline, so rerunning
after integration still compares both cache sizes. `tests-final.json.gz` passes
against the integrated candidate; the earlier out-of-tree run is retained as
`tests.json.gz`. For baseline diagnostic counts after integration, pass
`--gc-source` pointing to that extracted baseline source.
