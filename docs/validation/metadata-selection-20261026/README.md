# Immutable metadata lookup during native selection — 2026-10-26

This follow-up targets the startup hotspot from the [lazy-JIT CPU investigation](../lazy-jit-optimization-20261026/README.md): type-definition lookup accounted for about 20% of captured startup CPU, and selection appeared in 49% of inclusive stacks. Those overlapping samples identify a target, not a predicted speedup.

## Contract and alternatives

Native method selection now runs within the existing thread-local immutable module lookup scope. `Module::with_metadata_lookups` is a Rust compiler/runtime integration entry point; it passes an immutable module reference to its closure. Name candidates narrow type lookup while existing arity, identity, ordering, signature, access and dispatch checks remain responsible for validity. Selected output is a separate mutable module and does not share the source index. Source verification still occurs before projection.

Same-module nested calls reuse the index, another module temporarily replaces it, and return or unwinding restores the previous scope. Mutation between scopes rebuilds indexes. Indexes are not transferred to other threads or retained with lazy compiler state after selection. This adds temporary name-map construction/storage during selection; allocator retention can outlive the logical index and is measured separately from managed heap usage.

Reuse the [.NET/CLR comparison and scoped-owner research](../../design/allocation-metadata-runtime-comparison.md#scoped-method-owner-candidates--2026-10-24). The implementation extends bounded shared metadata resolution without changing managed APIs, metadata formats, native code generation or GC contracts. A permanent module cache would need mutation/invalidation rules; explicit indexing of every caller would duplicate plumbing. This slice uses the existing scoped mechanism and retains the linear implementation for differential validation. It applies to shared JIT/AOT selection, not interpreter execution, and does not claim improved warm native request throughput.

## Validation and measurements

Five lookup lifetime/differential tests and all 208 code-generation unit tests pass; 35 manual probes remain intentionally ignored. The selection test compares complete serialized selected metadata and the report against the unindexed path, then verifies the selected program. All 24 timed HTTP invocations pass response, guest-exit and clean-shutdown checks (98,688 responses including warmup). Every lazy run compiles 870 of 1,599 selected methods and leaves 729 uncompiled. The retained baseline runner predates only the prior slice's test-only host gate; its exact source is archived in the prior evidence. Production implementation matches `15beb09c1`.

Measurements use production-default Cranelift optimization, unchanged HTTP guest/native hosts, eager/lazy JIT and sync/concurrent GC on the same Apple M1 development host. Timed runs are sequential; profiler and memory diagnostics are separate. Desktop background activity remains a limitation. A small closed-loop HTTP run checks execution and shutdown, not server capacity.

The first test build encountered an inherited macOS 27 CommandLineTools SDK with an older selected Xcode linker. Validation and the candidate release build use the matching SDK under `/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk`. This is a build-environment correction, not a runtime change. Both retained runners are loaded once with `--list` before the paired cohort; the measurements describe repeated fresh processes with warm host caches, not reboot/file-cache-cold startup.

## Paired HTTP results

Three alternating pairs per mode, 4,096 measured JSON responses after 16 warmups, four clients, pipeline depth 16. These are process-to-READY and post-warmup measurements, not offered-load capacity. Medians:

| Mode / GC | Ready before → after (s) | Selection + verification before → after (s) | Warm req/s before → after |
| --- | ---: | ---: | ---: |
| Eager / sync | 13.390 → 12.531 | 9.390 → 8.396 | 1,764 → 1,876 |
| Lazy / sync | 9.906 → 9.682 | 8.724 → 8.480 | 1,996 → 2,088 |
| Eager / concurrent | 15.556 → 15.817 | 9.089 → 9.132 | 1,731 → 1,679 |
| Lazy / concurrent | 11.296 → 9.781 | 9.904 → 8.451 | 1,361 → 1,401 |

Median readiness improves in three modes, but eager/concurrent's group median is 1.7% slower. Pairwise readiness changes are −4.88% (eager/sync, range −9.49 to −3.43%), −2.26% (lazy/sync, −2.85 to −2.18%), −2.28% (eager/concurrent, −9.10 to +8.27%), and −12.10% (lazy/concurrent, −13.41 to −10.41%). The differing group/paired result for eager/concurrent illustrates variability; there is no universal speedup claim.

Selection is shared work before managed invocation, so different observed gains do not establish a special concurrent-GC interaction. Warm throughput is mixed: eager/concurrent falls, and one lazy/concurrent baseline trial falls to 656 req/s; that trial remains in the evidence. The index does not change generated request-processing semantics. These three local pairs are insufficient to attribute warm throughput changes or establish broad capacity. Full metric ranges and all trials are retained in `summary.json` and the compressed reports.

## Memory diagnostics

One separate baseline/candidate pair per mode passes all 32,896 checked responses and shutdown checks. At every measured malloc stage (`beforePrepare`, `beforeRelease`, `afterRelease`, `afterInvoke`), live block counts and bytes match exactly within each mode. These snapshots occur after selection, consistent with releasing the temporary index rather than retaining it with lazy compiler state. They do not measure peak index memory.

| Mode | RSS at READY before → after (MiB) | Allocator reservation after invocation before → after (MiB) |
| --- | ---: | ---: |
| eager-sync-jit | 866.4 → 710.6 | 912.0 → 916.0 |
| lazy-sync-jit | 917.5 → 946.9 | 908.0 → 940.0 |
| eager-live-jit | 1084.5 → 925.7 | 1052.0 → 1072.0 |
| lazy-live-jit | 812.3 → 778.7 | 924.0 → 936.0 |

Candidate allocator reservation is 4–32 MiB larger after invocation. Logical index release does not imply returning pages to the OS. Whole-process RSS varies substantially (including drops during traffic) and does not establish a lower footprint. Retain this resource cost alongside the modest startup benefit; neither counter is managed-heap size or a peak.

## CPU attribution and decision

A separate ten-second CPU Profiler capture for baseline and indexed lazy/concurrent startup contains 24,228 and 25,317 samples. Selection appears in 50.20% versus 41.31% of sampled cycles (inclusive); `Module::type_definition` accounts for 18.31% versus 15.45% (self). Linking remains about 85% inclusive. These overlapping shares describe two bounded captures, not exact phase speedups or recoverable CPU percentages; profiler timings are excluded from medians.

Retain the bounded change: lookup/selection semantics pass differential and full code-generation checks, all three lazy/sync and lazy/concurrent readiness pairs improve, and the profile shows less selection work. Keep eager/concurrent's slower group median and allocator-reservation cost visible. This does not change GC/JIT defaults, first-call blocking, or the workload-specific conclusions from the earlier load study.

Remaining type-lookup callers include generic conformance (`specialization::expand_with_conformance`), interface closure and variance/assignability checks. Reviewing immutable scopes in those paths is the next profile-backed candidate; caching a conformance result would require stronger instantiated-identity and lifetime guarantees. It is not implemented by this slice.

## Reproduction and retained evidence

`prepare-pairs.py`, `measure.py` and `summarize.py` retain the exact local preparation and sequential measurement commands. The existing `http.py` checks response payloads, guest exit and native concurrent invariants. The paired manifest pins both runners, unchanged guest/native hosts and separate source snapshots. Timing uses three alternating pairs; memory uses one additional pair per mode. Profiles run afterwards. The initial 12-process baseline qualification is retained separately and is not pooled into the paired medians.

`evidence-index.json` hashes each retained report/source/log. Only CPU-profile tables are exported from the traces; full trace bundles remain local because their other tables include environment metadata. No website deployment or push was performed. This Rust integration API does not add a managed API or require a Raven reference-assembly snapshot.

Website validation passes all 26 contract tests and the full 6,137-page build. The build uses the independently updated working-tree RavenDoc revision recorded in `website-toolchain.json`; that tool/archive change is not part of this slice. Existing API-description warnings remain. Updated tables are staged for the next manual publication, not deployed.
