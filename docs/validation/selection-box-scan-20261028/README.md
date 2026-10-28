# Scan selected bodies once per dispatch pass — 2026-10-28

The preceding specialization profile attributed 15.8% self samples to selection. A [function-name candidate experiment](../call-target-lookup-20261026/README.md) did not establish a reliable benefit and was reverted. Inspection found a more direct repeated cost: the dispatch loop scanned all selected method bodies for `box Char` once per selected function, although the selected set was immutable throughout that loop.

## Contract and implementation

Compute the presence of boxed Char once per dispatch pass, after draining the pending work. New targets enter `pending`, and the next pass recomputes the predicate after processing them. This removes a repeated whole-body scan without storing a module-wide cache, changing reachability, lowering a body, or weakening validation. An unreachable body containing boxing does not retain an override. Boxing introduced by an interface implementation discovered on a later pass does retain it.

This is an internal closed-world selection optimization shared by eager/lazy JIT and AOT preparation, with no managed API or .NET boxing/Object-dispatch semantic change. Reuse the existing [runtime and metadata comparison](../../design/allocation-metadata-runtime-comparison.md); the benefit is less repeated preparation work, with one boolean per pass and no new heap allocation. Compiling lazily still blocks the managed caller on first invocation. Collector defaults and publication rules remain unchanged.

## Validation and cohort recovery

All 209 codegen unit tests pass (35 manual probes ignored), including a regression for boxing discovered through interface dispatch and for unused boxing. The earlier October 26 target directory, raw measurements and retained binaries were unavailable when work resumed. Preserve that limitation rather than reconstructing unsupported numbers.

A new cohort downloads the hash-verified published Preview 18 macOS ARM64 package, compiles the current HTTP fixture against its libraries, and rebuilds identical native hosts for both variants. This uses the standard sync budgets 256/4096 and concurrent budgets 1024/16384, 50% watermark, index-only stores and qualified stack policy. Baseline runtime source is `ad5a9f22b`; candidate changes only the selection scan and its test. Libraries differ from the October 26 development cohort: comparisons are within this fresh cohort only.

Run fresh processes with warm host caches, alternating variant order, JSON pipeline, four clients, depth 16, 16 warmup responses and 4,096 measured responses per process. Separate memory and ten-second startup CPU captures from timed runs. Task-owned compilation/profiling does not overlap throughput timing; ordinary desktop activity remains uncontrolled. This is not a production capacity or peak-memory test.

## Paired startup and HTTP results

All 24 timed invocations pass (98,688 checked responses including warmup). Eight preceding smoke invocations pass (384 responses). Every lazy run compiles 870 methods and leaves 811 uncompiled. Medians of three fresh-process pairs:

| Mode / GC | READY before / after (s) | Selection + verification before / after (s) | Warm req/s before / after |
| --- | ---: | ---: | ---: |
| Eager / sync | 11.488 / 10.619 | 7.262 / 6.323 | 1,997 / 2,004 |
| Lazy / sync | 8.522 / 7.726 | 7.315 / 6.507 | 2,218 / 2,190 |
| Eager / concurrent | 14.225 / 13.112 | 7.202 / 6.445 | 1,809 / 1,803 |
| Lazy / concurrent | 8.629 / 7.657 | 7.245 / 6.330 | 1,562 / 1,545 |

Eleven of twelve readiness pairs improve. The first lazy/sync pair regresses 2.37%; its other pairs improve 13.52% and 9.34%. Do not discard that variation. READY group medians improve 7.6%, 9.3%, 7.8% and 11.3%, respectively. This shared preparation gain precedes managed invocation and establishes no special concurrent-GC advantage.

Warm throughput group medians change +0.4%, -1.3%, -0.3% and -1.1%; no warm-throughput gain is claimed. The first lazy/concurrent pair is 8.29% lower, followed by +1.81% and -0.31%. These short local runs do not establish exact equivalence or production capacity.

| Mode / GC | First response before / after (s) | READY through first response before / after (s) |
| --- | ---: | ---: |
| Eager / sync | 0.002 / 0.002 | 11.490 / 10.622 |
| Lazy / sync | 1.681 / 1.670 | 10.204 / 9.397 |
| Eager / concurrent | 0.003 / 0.003 | 14.229 / 13.115 |
| Lazy / concurrent | 2.908 / 2.892 | 11.537 / 10.549 |

First-call lowering/compilation remains synchronous from the caller's perspective. Lower readiness does not eliminate cold-request latency. Raw ranges and pairwise differences remain in `summary.json`.

## Memory and CPU attribution

All eight separate memory invocations pass (32,896 checked responses). Live malloc block counts match at each recorded stage. Candidate live bytes are 16 bytes lower at every stage, including `beforePrepare`; this is not evidence of a new compiler memory saving. No new heap storage is introduced by the boolean hoist.

| Mode / GC | Allocator reservation after invocation before / after (MiB) |
| --- | ---: |
| Eager / sync | 944 / 964 |
| Lazy / sync | 872 / 956 |
| Eager / concurrent | 1,064 / 1,048 |
| Lazy / concurrent | 872 / 956 |

Allocator reservation is mixed (-16 to +84 MiB). These one-pair snapshots do not measure peak memory, and the larger lazy reservations remain a tradeoff to investigate under explicit memory pressure. They must not be presented as fewer live managed objects or a proven memory regression caused by this allocation-free scan change.

Separate ten-second lazy/concurrent CPU captures contain 15,480 baseline and 18,140 candidate samples. Selection's self share falls from 19.95% to 2.78%. These are shares of sampled cycles over separate captures, not exact instruction costs or equal-work phase durations. Remaining candidate samples include hashing (7.04% self), type equality (5.05%), dispatch-target discovery (4.80%), and object-upcast verification (9.86% inclusive). Specialization is 24.76% inclusive; overlapping shares must not be added.

Retain the per-pass scan: the reachability regression and codegen suite pass, eleven readiness pairs improve, all four readiness medians improve, and CPU attribution supports removing the repeated scan. This does not establish a warm-throughput benefit, a collector winner, or nonblocking lazy compilation.

## Reproduction and next investigation

`bootstrap.py` reconstructs the native hosts and fresh HTTP guest using the downloaded Preview 18 package from the release URL in `docs/validation/release-20261026/publication.json`; verify its SHA-256 from `environment.json` before extracting it into `target/selection-box-scan-20261028/release`. Build and retain the release codegen test runner at baseline `ad5a9f22b`, then build the archived candidate patch with the recorded Xcode SDK. The experiment expects `baseline-runner` and `indexed-runner` in that target directory. Run `measure.py` only after builds complete, then `summarize.py`. `finish.py` records the local export/archive procedure; its temporary script paths are provenance, not a portable installer.

The evidence index hashes reports, source snapshots, test/build logs, CPU-only trace exports, actual guest assemblies and consumed library metadata. Full traces remain local; no environment tables are exported. The previous name-index measurements are unavailable and excluded.

Next best-bet investigation: repeated type relationship work in dispatch-target discovery and object-upcast verification, using the remaining profile attribution to identify safe immutable reuse. Preserve exact verification errors and selected metadata; measure the shared startup benefit against allocator reservation. First-call suspension and warm GC/root bookkeeping remain separate, higher-complexity work for application continuity. Do not assume startup improvements fix traffic capacity.

Website validation passes all 26 contract tests and the 6,137-page local build (134 legacy pages retained). Existing API-description warnings remain. Tables are prepared for the next manual publication; no deployment or push is part of this slice.
