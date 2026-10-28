# Worker execution tracer qualification — 2026-10-28

Implemented private POSIX worker recording and offline Chrome JSON export. This is the first execution-visibility slice, not a new scheduler or a production HTTP/JIT/GC trace. [Contract and reproduction](../../experiments/execution-trace/README.md).

Optimized and fail-fast UBSan trace fixtures pass at one and four workers: slot reuse, successful/failed callbacks, queued/running cancellation, joined teardown, disabled/live-read rejection, bounded overflow, truncated captures and invalid event ordering. All expected job outcomes are checked independently. Ordinary and compiled-coordinator native safety suites each pass their 11 groups in optimized/UBSan builds; the final combined `NEOCLR_WORKER_TRACE` + `NEOCLR_WORKER_GC` build also passes all 11 groups in both variants. UBSan does not establish race freedom.

Final complete captures observe 51 jobs (one worker, including two cancellations and a failure) and 49 jobs (four workers, including a failure), with no lost events. One-record-per-lane captures intentionally lose events and are marked incomplete. The exporter checks structure and chronology; interactive Perfetto import was not exercised.

## Recording overhead

Three alternating pairs per worker count, 8,192 tiny jobs per invocation. Times exclude pool setup/teardown/export; include first-touch and event recording during execution. Traced buffers reserve 65,536 records per lane. No records lost in timed traced runs. These are short diagnostic screens on an uncontrolled desktop, not production throughput evidence.

| Workers | Recording off/on wall median (ms) | Change | Off/on CPU median (ms) | Reserved event buffers |
| --- | ---: | ---: | ---: | ---: |
| 1 | 3.976 / 5.307 | +33.5% | 5.163 / 7.220 | 5 MiB |
| 2 | 6.585 / 12.188 | +85.1% | 11.903 / 24.253 | 7.5 MiB |
| 4 | 12.152 / 16.858 | +38.7% | 30.359 / 41.861 | 12.5 MiB |

Recording costs are material. Keep it opt-in and use untraced performance controls. The buffer reservation is a configured bound, not measured resident memory. No low-overhead or general application-overhead claim is made.

## Preserve the normal build

An initial always-instrumented implementation showed noisy short controls; a longer 524,288-job comparison still had roughly 10% higher two-worker elapsed median with recording disabled. The retained intermediate reports explain the decision, but their source revisions are superseded and those binaries are not the shipped experimental build. We added `NEOCLR_WORKER_TRACE` build-time opt-in. Without it, recorder storage, worker lane setup and hooks compile out. The ordinary constructor also disables recording within an instrumented build.

The final default-build control uses three alternating pairs of 524,288 jobs each against `ad6ebc989` worker source, compiled with the same compiler and a disabled-only probe adapter:

| Workers | Previous/default wall median (ms) | Change |
| --- | ---: | ---: |
| 1 | 232.417 / 233.897 | +0.6% |
| 2 | 320.530 / 341.716 | +6.6% |
| 4 | 726.286 / 728.814 | +0.3% |

A separate `-O2 -S` comparison of the final default worker source and baseline produces **byte-identical ARM64 assembly**, retained here. This supports removal of default-path instrumentation on this compiler/target; mixed timings are not an optimization gain or proof of exact performance equivalence on every platform.

## Evidence and limits

`qualification.json.gz` records final input hashes, compiler commands, all samples and diagnostic output. `baseline-workers.c.gz` retains the comparison source. The archive index hashes compressed evidence. The host is Apple M1/macOS, eight logical CPUs and 16 GiB; unrelated desktop/build activity was not isolated. Earlier screens had much larger elapsed variation and are retained, not silently replaced.

Raw event timestamps use CLOCK_MONOTONIC in nanoseconds; exported times are relative microseconds. Lane IDs are logical workers, not OS thread identities. Condition waits include mutex reacquisition; callback spans include blocking/descheduling. Neither is CPU utilization. No GC-phase, JIT-wait, async-context or HTTP scaling result is established. Next extend qualified context/operation correlation and collector/compiler hooks before using traces to explain sync/concurrent GC application differences. Website material is prepared for the next manual publication.

All 26 website contract tests pass for the updated proposal content. Local design/evidence links resolve; no website deployment was performed.
