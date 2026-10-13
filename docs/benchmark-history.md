# Benchmark history and regression policy

Author direction, 2026-10-13: retain historical benchmarks so maintainers and project
visitors can compare results. Benchmarks should be stable, and performance should
not degrade. This is a quality goal, not a claim that regression gates already exist.

## Immutable runs and versioned workloads

Keep dated raw reports, source/harness hashes, compiler/runtime/library revisions,
artifact hashes, commands, validation outcomes and environment data. Do not overwrite
an older result when repeating a run. Corrections get a new record explaining the
superseded claim; preserve the original evidence. The website should show current
baselines and provide a browsable history, an explicit exception to removing routine
historical prose from introductory feature pages.

Give each workload a stable identity and semantic version. Changing payloads, query,
row count, operation boundaries, validation, connection reuse or timing protocol
creates a new workload version or separately labeled series. A runtime optimization
changes the tested implementation revision, not the workload identity. Keep startup,
steady work, latency, memory and deployment size as distinct metrics.

Only compare matching workload versions, execution/host profiles, architecture,
hardware, OS/toolchain conditions and measurement settings. Record changes in GC or
resource limits. AOT with the shared native host is distinct from standalone AOT;
Docker resources and database image/version are part of SQL series identity. Never
splice unrelated old experiments into a trend or infer improvement from a new machine.

## Regression acceptance

The author requests stable or improving performance. The proposed operational rule
is **no statistically meaningful regression under comparable conditions**, with zero
correctness errors before performance is considered. A faster incorrect result fails.
An intentional performance tradeoff does not automatically waive this goal: document
it and obtain an explicit author decision before accepting a regressing baseline.

Before automated gating, establish repeatability on controlled hosts, calibrate run
lengths/warmups and quantify normal variance. Define per-metric tolerances and the
comparison/statistical method from that evidence; no percentage is chosen here.
Repeat suspected regressions before attribution and retain all attempts. Do not
cherry-pick a fast run, drop a slow round without a recorded validity reason, or hide
latency/memory degradation behind a throughput gain.

This follows the existing [framework investigation](benchmarking-framework.md):
BenchmarkDotNet-style isolated jobs, warmup and raw sample retention are useful, but
our fixed-count external loops do not yet supply its calibration or statistical
analysis. Host control and longer measurements cost time; that is necessary before
turning exploratory numbers into a release gate.

## Current status

The 2026-10-13 Preview 14 runs seed the historical record. They are short measurements
on an interactive Apple M1 with desktop/indexing activity. Retain their observed ranges
and provenance, but do not use them to enforce a regression threshold. Establish a
controlled reference series next. JSON/database HTTP cases and general CPU/JSON cases
have separate operation definitions; their numbers are not additive estimates.
