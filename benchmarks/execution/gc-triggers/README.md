# Collection-request origins

Development evidence, 2026-10-24. `NEOCLR_GC_PROFILE` now enables a separate private
trigger snapshot/reset API. Requests and failed dispatches are counted on the
calling native thread; they are not necessarily completed collections. Worker
handoff can execute elsewhere, and direct conditional/participant entry bypasses
origin counting. Existing pause and lookup snapshots remain unchanged.

Origins are direct, explicit idle, pressure retry, managed GC.Collect, and four
paced causes in priority order: context change, collection epoch change, allocation
budget, publication budget. Reset between operations affects only these counters,
not pacing, heap state, pause counters or roots. Compile root/collector units with
consistent profiling flags. Disabled builds return zero without trigger TLS or
counter increments. Enabled storage costs 128 bytes per native thread.

## Findings

Same qualified plaintext/reflection-JSON programs and owned 1 MiB entry hosts, in
AOT and JIT. Each mode runs 0, 32 and 64 checked requests. Use the 32→64 difference
to exclude first-request initialization; zero-request runs expose startup/teardown.
These are instrumented, sequential-request diagnostics, not throughput trials.

| Request phase | Plaintext AOT/JIT | JSON AOT/JIT |
| --- | ---: | ---: |
| Publication-budget requests per HTTP request | 3 / 3 | 5 / 5 |
| Explicit-idle requests per HTTP request | 1 / 1 | 1.03125 / 1 |
| Allocation-budget / pressure requests | 0 / 0 | 0 / 0 |
| Managed allocations per HTTP request | 208 / 208 | 457 / 457 |
| Root publications per HTTP request | 3,676 / 3,676 | 5,535 / 5,535 |
| Inclusive measured GC µs per HTTP request | 57.0 / 66.6 | 115.8 / 115.2 |
| Peak live bytes at 64 requests | 45,490 / 45,490 | 52,602 / 52,602 |
| Peak arena extent at 64 requests | 73,408 / 73,408 | 79,552 / 79,552 |

All 12 server invocations pass response/completion checks. In these owned-host runs,
origin counts sum exactly to collector attempts, with zero failures. This empirical
relationship is not generalized to worker delegation. Idle counts vary with scheduling;
GC time is inclusive diagnostic timing, not reason-specific CPU cost. Zero-request
runs already have two allocation-budget and two publication-budget collections plus
one initial-context request and variable idle work.

The dominant request-phase trigger is the 1,024-publication budget, not the
256-allocation budget or heap pressure. About one explicit idle collection/request
comes through the native socket/scheduler polling adapter. A larger publication
budget is a concrete next experiment, preserving allocation/pressure/idle behavior
and checking peak retention, cleanup and latency. No budget change ships in this slice.

## Qualification

Eight optimized/sanitized enabled/disabled and worker-hook configurations pass;
each exercises the caller plus two fresh native threads. Tests cover all origins,
priority when causes overlap, impossible allocations, failed requests, direct
conditional bypass, delegated requests without local collections, and reset midway
through the publication budget. Four summary tests reject incomplete, duplicate,
failed, unclassified and truncated cohorts.

Optimized ARM64 assembly for all 38 pre-existing collector functions is unchanged
with profiling disabled, after normalizing comments/local labels and matching literal
pools by contents. Two zero-return/reset entry points are added; this does not claim
whole-binary identity or guarantee identical instruction placement. Initial comparison
attempts flagged compiler comment/constant-label numbering; raw assembly and the
failed comparisons are preserved alongside the corrected content-aware comparison.
No new Windows execution claim, compiler rebuild, managed API or attach-wire change.

## Reproduction

Runners reuse the qualified local bundle, objects and host recipes in the reports.
Use fresh output directories. Private counter documentation is in
[native profiling](../../../docs/experiments/runtime-async/native-profiling.md).

```sh
python3 benchmarks/execution/gc-triggers/validate.py --output target/triggers-new/tests
python3 benchmarks/execution/gc-triggers/verify_disabled.py --output target/triggers-new/disabled
python3 benchmarks/execution/gc-triggers/counts.py --sample plain --output target/triggers-new/plain
python3 benchmarks/execution/gc-triggers/counts.py --sample json --output target/triggers-new/json
python3 benchmarks/execution/gc-triggers/summarize.py \
  target/triggers-new/plain/results.json target/triggers-new/json/results.json target/triggers-new/summary.json
python3 -m unittest discover -s benchmarks/execution/gc-triggers -p 'test_*.py'
```

Existing published throughput claims remain unchanged; no website build is needed.
The interpreter's separate heap is not measured here. Reasons for cross-worker
requests are not transferred into the coordinator's pause records.
