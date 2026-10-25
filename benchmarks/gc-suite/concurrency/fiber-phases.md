# Fiber root/resume and scheduler phases — 2026-10-25

Diagnostic extension of the [carrier transfer profile](fiber-results.md). The
`--phases` option instruments a copied native carrier; production code and defaults
are unchanged. Compiler objects and proof flags are retained. The sanitized
owned-body validation and missing-arrival failure control pass before relinking.

## Boundaries

- `resume_prepare`: successful resume admission, active/ready host roots, caller
  root parking, target root restoration and stack-bound installation.
- `resume_finish`: caller restoration, root resumption, completed-body cleanup
  and cache/unmap decisions. Failed admission before transfer is not timed here.
- `yield_roots`: active-root release and owned/unowned root parking before transfer.
- `scheduler_acquire`: the pump's acquisition call, including socket polling,
  delivery-root acquisition and any idle GC it invokes.
- `scheduler_wait`: the pump's readiness wait, excluding work in other phases.

Guest execution and carrier transfers are excluded from resume/yield phases.
Body completion before `yield`, queue-to-running delay, fiber allocation, readiness
scans, dispatch root handoff and unrelated host-root calls are not fully attributed.
Acquisition timing is inclusive of its internal GC work: it is not pure scheduling
CPU. These scopes are not a complete decomposition of execution time.

Every HTTP invocation checks five unique phase records and requires successful
prepare/finish/yield counts to match the independently balanced resume/transfer
counts. Clock overhead is included. Wall and thread-CPU timestamps have different
endpoints; neither their difference nor subtracting these results from baseline
CPU is a reliable estimate of overhead or descheduling. Small intervals approach
the clock's resolution and overhead, especially root operations.

## Reproduce

```sh
python3 benchmarks/gc-suite/concurrency/prepare-fiber-profile.py --phases \
  --build target/gc-matrix-http-build-v3 --output target/new-phase-build
python3 benchmarks/gc-suite/matrix/http.py \
  --build target/new-phase-build --output target/new-phase-profile \
  --modes sync-aot live-aot sync-jit live-jit --protocols close pipeline \
  --requests 1024 --rounds 1
```

Private macOS ARM64/ucontext diagnostic only. One run per cell is evidence for
attribution, not a new throughput baseline, saturation result or default decision.
The .NET comparison remains the [allocation/background GC research](../../../docs/design/generational-concurrent-gc-direction.md):
separate runtime coordination from tracing before selecting more collector threads
or generational policy. No public API or compiler bridge changes are involved.

## Results

All 16 cases pass, totaling 16,640 checked responses including warmup. Counts
match across collectors/modes: 6,248 resumes for close and 2,488 for pipeline.
All times below are owner-thread CPU milliseconds over the entire invocation.

| Payload | Protocol | Mode | Prepare + finish + yield | Acquire | Wait | Root/resume fraction of invocation CPU |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| plain | close | sync-aot | 4.11 | 14.88 | 0.18 | 1.9% |
| plain | close | live-aot | 4.37 | 15.97 | 0.19 | 1.5% |
| plain | close | sync-jit | 4.02 | 14.49 | 0.15 | 1.2% |
| plain | close | live-jit | 4.06 | 15.02 | 0.11 | 1.0% |
| plain | pipeline | sync-aot | 1.59 | 10.58 | 0.02 | 1.1% |
| plain | pipeline | live-aot | 1.62 | 8.88 | 0.02 | 0.9% |
| plain | pipeline | sync-jit | 1.62 | 10.84 | 0.02 | 0.7% |
| plain | pipeline | live-jit | 1.62 | 9.01 | 0.03 | 0.6% |
| json | close | sync-aot | 4.07 | 14.40 | 0.17 | 1.3% |
| json | close | live-aot | 4.08 | 14.82 | 0.11 | 1.0% |
| json | close | sync-jit | 4.19 | 19.14 | 0.26 | 0.8% |
| json | close | live-jit | 4.08 | 16.50 | 0.16 | 0.7% |
| json | pipeline | sync-aot | 1.61 | 11.18 | 0.02 | 0.6% |
| json | pipeline | live-aot | 1.64 | 9.03 | 0.01 | 0.5% |
| json | pipeline | sync-jit | 1.65 | 11.39 | 0.03 | 0.4% |
| json | pipeline | live-jit | 1.65 | 8.64 | 0.02 | 0.4% |

Root/resume intervals are small in these workloads, including clock overhead.
Scheduler acquisition costs more, but includes idle collection. Inspect
`neoclr_scheduler_acquire_v1` in `socket-listener.c`: it first calls
`neoclr_scheduler_step_v1`, then `neoclr_gc_idle_v1` for pending/idle results,
or acquires delivery roots for ready work. The next attribution should separate
polling from idle collection and delivery ownership, before changing roots or
carrier architecture. This profile does not identify root handling as the cause
of concurrent GC's HTTP CPU gap.

Invocation CPU includes background collector threads; phase CPU is owner-only.
Client load and idle waits vary, so wait wall time is not a GC pause metric.
Build commands, source hashes, validation outputs and raw HTTP profiles are retained
in `fiber-phases-build.json.gz` and `fiber-phases-http.json.gz`.
