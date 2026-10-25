# Sustained ordinary workloads for concurrent GC

This extends the HTTP measurements with existing standalone sample bodies. It is
an internal experiment on the qualified macOS ARM64 owned-heap host; production
GC remains synchronous. No managed API or compiler admission is changed.

`build.py` renames each sample's Main to RunSample and adds an async batching
entry. Original bodies and console output remain intact. Existing explicit
OnCompleted adapters retain their documented purpose. Every selection uses the
same retained compiler bundle and dependency/binding recipe as the HTTP cohort.
Both synchronous and live selections are attempted; rejection is a coverage gap,
not a zero-cost result. These scripts require the retained artifacts named in
their recipes; they are not standalone distribution tools.

```
python3 benchmarks/execution/concurrent-workloads/build.py \
  --runner target/concurrent-store/test-runner-http-live-jit \
  --output target/concurrent-workloads/batch4096 --iterations 4096 \
  --only tasks collections arrays
python3 benchmarks/execution/concurrent-workloads/measure.py \
  --build target/concurrent-workloads/batch4096 \
  --output target/concurrent-workloads/measure4096 --rounds 5
```

The measurement harness compiles every AOT host before execution, alternates
collector order, checks complete output and exit status, and reports diagnostics
on the owning thread after invocation cleanup. Invocation CPU uses process
getrusage deltas and includes collector threads; wall time uses a monotonic clock.
Both exclude compilation and process startup, include console writes and cleanup,
and differ from the earlier whole-process HTTP CPU measurement. Counters include
allocations, bytes, heap peaks, collection count, base lookups and live stores.
Allocation count/byte equality is checked across controls. The collector's peak
payload and arena occupancy are not process RSS. Concurrent epoch counters check
completion/cancellation balance and quiescence. Completed epochs must be examined
before interpreting a batch as sustained concurrent collection.

.NET's allocation/GC/execution distinction remains the comparison model described
in the [existing investigation](../concurrent-gc-investigation/README.md). This
harness changes measurement scope, not GC semantics. Core collector timings are
not a complete mutator pause measurement; outer-boundary pause instrumentation,
JIT sustained execution, guarded synchronous controls and worker scaling remain
follow-up work. Background OS activity limits absolute performance claims.

## Initial coverage (2026-10-25)

All eight source builds succeed. At 128 iterations:

- Tasks, order collections and array tour compile and execute under both collectors,
  with exact output, clean shutdown and identical allocation counts/bytes. Live
  runs complete 4, 15 and 5 epochs respectively. These short measurements are pilot
  diagnostics, not reliable capacity numbers.
- Cancellation compiles for both, but fails during repeated execution with
  NativeMemoryLimitExceeded in CompletionSignal construction. The wrapper queues
  pending cancellations without returning to the outer pump each iteration;
  native-runtime.c has a 128-operation limit. This batching adaptation needs a
  bounded/drained workload before performance interpretation. The failure is
  preserved, not counted as successful throughput or attributed to the collector.
- Grapheme strings fail shared native compilation at `neoCLR.Runtime.CharCategory`.
- Reflection fails shared native compilation at `neoCLR.Runtime.TypeEnumNames`.
- Task.Run async compiles synchronously but live admission rejects the unqualified
  `neoCLR.Runtime.ScheduleTask` helper. Qualify its roots, callback and native output
  boundary before enabling it; no broad allowlist exception was added.
- Standalone JSON mapping also fails shared native compilation at
  `neoCLR.Runtime.ReflectionConstructionCheck`; reflection construction roots
  must also be checked before treating it as a qualified workload.

Raw evidence is archived alongside this file; the initial unsuccessful matrix is
retained separately from the longer successful cohort. No claim of comprehensive
sample coverage or readiness to change the default follows from these runs.

## Sustained AOT results

Five alternating rounds per collector, 4,096 complete sample iterations each:
30/30 runs pass output, status and lifecycle checks. All controls have identical
allocation counts and payload bytes. See [summary](summary.json) and
`sustained-measure.json.gz` for every measured run and compiled-input hashes.

| Sample | Sync CPU seconds | Live CPU seconds | Live/sync CPU | Allocations per invocation | Allocated bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Tasks | 0.017138 | 0.103722 | 6.05× | 65,548 | 2,261,361 |
| Collections | 0.099004 | 0.559068 | 5.65× | 237,580 | 7,864,689 |
| Arrays | 0.052827 | 0.140913 | 2.67× | 86,028 | 1,892,721 |

CPU values are medians; console work is retained. Background mediaanalysisd,
photolibraryd and dasd were active during preparation; no project build or other
agent experiment ran during this timed cohort. These are instrumented short-batch
execution comparisons, not saturated HTTP capacity estimates or quiet-machine
absolute baselines. Live completes 128/464/168 epochs in the first respective runs,
with one shutdown cancellation each. Peak managed payload is larger under live
pacing: 9,329/10,113/6,239 bytes versus 497/1,694/593 synchronously. Different pacing
means the ratios do not isolate the cost of concurrency alone.

The first collections run records 2,555,956 out-of-arena stores, 943,694 inactive/new
stores and 10,704 guarded old stores; no store rejection or store-triggered
cancellation. Its range cache hits 458,072 times but still performs 436,662 checked
walks. Tasks and arrays also make 372,788 and 225,332 out-of-arena calls. These
counts support investigating proven unnecessary store checks across ordinary and
HTTP workloads. They do **not** establish a CPU percentage or authorize skipping
validation for unknown destinations. Ordinary exact-base lookup misses are much
rarer (125 collections-case walks), so optimizing that counter alone would miss
the adapter's separate range-search work.

Next: compare the same batches with a guarded synchronous control, attribute
range-walk and publication cost, then qualify a narrowly proven reduction with
mutation/lifetime negative controls and matched HTTP plus standalone measurements.
Repair/qualify the missing helper and cancellation workload boundaries separately;
retain JIT, outer pauses and worker scaling as explicit coverage gaps. The current
live collector remains slower than synchronous GC despite the prior cache gains.

## Exact-base follow-up

[Guarded controls and exact-base lookup results](exact-base-results.md) now
separate more of the adapter cost from marking. The measurement tool accepts
`--guarded-control` and `--disable-exact-store`; the HTTP tool accepts
`--compare-exact-store` for interleaved previous/new lookup controls. The new
shortcut reduces standalone range walks and CPU, while HTTP gains remain small.

## Recent-range follow-up

[Four recent ranges](ranges4-results.md) now reduce interior-field lookup cost.
The tools default to `--store-ranges 4`; use `--store-ranges 1` to reproduce the
preceding exact-base-only configuration. HTTP `--compare-store-ranges` interleaves
one-range and configured-range controls. Production GC is unchanged.

## Call-site attribution and stack transfers

[Store attribution and transfer proofs](transfer-results.md) connect runtime counts
to generated functions and preserve stack provenance across single incoming edges.
Both measurement tools accept `--profile-store-sites` for diagnostic-only cohorts;
`summarize-sites.py` requires exact symbol/selection reconciliation. CPU comparisons
use ordinary providers, with HTTP `--previous-live-object` interleaving old/new code.

The [generated-code optimization qualification](optimizer-results.md) compares
Cranelift settings across both collectors, including native correctness fixtures
and retained standalone/HTTP selections. Production compilation keeps its defaults.

[Fixed-width native stores](direct-store-results.md) compare scalar copy strategies
without changing generated managed objects or weakening mutation/ownership checks.

[Private TLS and CPU sampling](packed-tls-results.md) retain ownership isolation and
compare identical generated code. Sampled cohorts are diagnostic-only; their wall
stack observations, including waits, are not CPU-time percentages.

[Header-validation inlining](inline-header-results.md) is retained as an opt-in
experiment: safety passes, but mixed/small performance effects leave the default
outlined. The report separates initial and final control labels explicitly.

[Cached header constraints](header-proof-results.md) test reuse under current owned
allocation admission. Safety passes, but mixed sub-1% live effects leave full
validation as the default; the extra native TLS is opt-in only.

[Within-store snapshot reuse](snapshot-reuse-results.md) removes a duplicate search
without new cache state. Native assembly verifies the elimination; matched runs
show modest CPU/throughput gains with equal allocations.

[Concurrent client matrix](client-concurrency-results.md) compares one, four and
sixteen connection-close clients. It separates client/server CPU, response latency,
allocation variation and diagnostic profiles; queued load exposes a larger guarded
store cost despite fewer concurrent epochs. This does not admit shared-worker GC.

[Checked walk anchors](walk-anchor-results.md) use current allocation proofs to
skip redundant heap prefixes. Optional header-visit diagnostics explain the larger
queued-load gains; full-walk controls preserve the comparison and worst-case gaps.

[Private store-state grouping](store-tls-results.md) follows the post-anchor CPU
profile. It reduces verified TLS resolution while retaining separate heap ownership,
all hint validation and a split-layout control.

[Owned allocation index](allocation-index-results.md) compares a maintained start
bitmap with anchored walks. The opt-in includes independent lifecycle/model tests,
metadata-allocation/cleanup accounting and explicit .NET/Go design comparisons.

See [exact-base index reuse](exact-index-results.md) for direct and hybrid
lookup controls, HTTP gains, ordinary CPU regressions and unchanged metadata cost.

[Balanced interleaving and exact-base inlining](inline-base-results.md) isolate
the ordinary CPU regression and retain HTTP gains, with code-size and allocation
tradeoffs. The summary tool checks balanced evidence before reporting paired ratios.

[Post-inlining profiles and worker-handle control](worker-tls-results.md) attribute
TLS work to callers, reject an unsupported layout performance claim and refresh
exact outside-store sites for the next compiler-provenance investigation.

[Bounded stack-argument audit](stack-argument-audit-results.md) follows those sites
into the checked compiler closure. Read-only reports preserve byte-identical
objects and distinguish candidate static stores from qualified guard elimination.

[Bounded stack-store experiment](stack-elision-results.md) adds test-only guard
removal, mixed-pointer exclusion tests and matched previous-code controls across
ordinary and HTTP workloads. The native collector settings remain identical.

[Managed-JIT follow-up](stack-jit-results.md) qualifies transformed HTTP and
standalone execution, retains a rejected zero-epoch control and identifies the
remaining owner-pause measurement boundaries.

[Complete owner-operation pause diagnostics](owner-pauses-results.md) close the
phase-timer coverage gap for selected operations, separate overlapping phases
from outer GC stalls, and retain synchronous controls and overhead limitations.

[Finish CPU/wall attribution](finish-phases-results.md) separates waiting from
active collection and retains components of the same slowest invocation. The
observed dominant cost is final collection, guiding index/root/seed/sweep work.

Follow-up: [final-collection attribution and opt-in linear seed validation](collection-seed-results.md) retain disjoint CPU accounting, native/JIT/AOT safety, and modest uninstrumented HTTP gains with unchanged defaults.

[Seed validation/application split](seed-parts-results.md) narrows the remaining seed work with nested CPU/wall accounting and retained diagnostics.

[Validated seed header reuse](direct-seed-results.md) retains canonical mark semantics and opt-in indexed/direct comparisons; local CPU gains have small mixed application effects.
