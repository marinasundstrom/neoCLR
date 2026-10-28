# Shared async/GC tracing qualification — 2026-10-28

Implemented bounded native sessions and the first async/GC correlation bridge. [Ownership, event meaning and reproduction](../../experiments/execution-trace/session-README.md). JIT/compiler and worker-pool registration remain unimplemented. No production performance/default change is claimed.

## Validation

Optimized and fail-fast UBSan builds pass the concurrent-producer lifecycle test and the existing async/GC fixture with shared recording. Producer tests cover simultaneous independent writers, failed nested attachment, creator-only close, rejection of live reads/destruction, detach before close, rejection of lane reuse, post-close attachment, exact loss accounting and joined teardown. They exercise generic pthread producers, not the JIT compiler or worker pool. UBSan does not establish race freedom.

Both synchronous and active concurrent-GC fixtures pass complete and overflow captures. The complete eight-round sync capture has 41 paired collection requests. The concurrent capture has 42 requests and 41 terminal epoch spans: 40 finish and one is explicitly cancelled on heap release. Nested async progress, rooted suspension/resumption, fault and forced-shutdown paths remain checked by the reused fixture. Export tests remove an interior event and a whole lane: loss produces raw points with no inferred durations, and missing lanes are rejected.

An epoch span is the lifetime from publication through disposal/join; it is not marker CPU time or a stop-the-world pause. The request span includes owner dispatch work, including epoch admission/finish where applicable. Heap identities, finer phases and direct conditional-collection paths that bypass profiled dispatch are not recorded.

## Incremental recording cost

Three alternating pairs per collector, 512 rounds each, with the existing private async recording active in both controls. The enabled variant adds shared async mirroring and GC events; it is **not** a comparison of all tracing against tracing disabled. Setup/export and the final cancellation control are excluded from the measured loop. Full raw samples are retained.

| Collector | Existing-only / shared elapsed median (ms) | Change | Existing-only / shared CPU median (ms) |
| --- | ---: | ---: | ---: |
| sync | 6.877 / 7.260 | +5.6% | 6.856 / 7.258 |
| concurrent | 22.761 / 23.069 | +1.4% | 23.145 / 23.688 |

The single shared lane reserves 3.5 MiB at 65,536 records, in addition to the existing 2.5 MiB async buffer. There are no lost events in the timed full captures. These are short desktop diagnostic screens with uncontrolled load, not HTTP capacity evidence or a collector recommendation. An earlier screen showed roughly +20%/+4% elapsed cost and is retained; variability precludes a universal overhead claim.

## Normal builds and evidence

Compared against `10d4c88a1`, the normal ARM64 assembly is byte-identical for the profiled synchronous collector, configured concurrent collector, ordinary async carrier and legacy async-only instrumentation. The shared flag is absent in these comparisons. A shared-enabled GC compilation without GC profiling also succeeds, with request hooks absent as documented. Commands, baseline/current assembly and current source hashes are retained.

The host is Apple M1/macOS, eight logical CPUs and 16 GiB. `qualification.json.gz` contains source hashes, compiler commands, every sample and sanitizer/epoch output. Generated collector C, raw complete/overflow captures and combined Chrome JSON exports are archived. The initial test wrapper failed compilation because renaming C main removes its implicit return; adding an explicit return repaired the fixture. No runtime workaround was needed. That failed report is retained. Interactive Perfetto import was not exercised.

The author subsequently prioritized scheduling/work-queue optimization and clarified JIT for development, Native AOT for publish/distribution, with shared GC/scheduling and concurrent GC preferred. Next compare queue policies using these diagnostics. Compiler/worker registration, marker phases and real I/O correlation remain open; application qualification must include Native AOT and JIT with both collectors. Publication of website updates remains manual.

All 26 website contract tests pass; the subsequent execution-role prose update reuses those unaffected structural checks. Local documentation links resolve. No deployment was performed.
