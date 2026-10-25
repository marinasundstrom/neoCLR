# Scheduler acquisition attribution — 2026-10-25

This extends the [fiber phase profile](fiber-phases.md) using a copied socket host.
`prepare-fiber-profile.py --scheduler` implies `--phases` and adds three nested
intervals inside the pump's scheduler acquisition:

- `poll`: `neoclr_scheduler_step_v1`, including socket/timer/runnable polling and
  the pending-operation scan. It performs native I/O as well as scheduling work.
- `idle_gc`: the `neoclr_gc_idle_v1` call for idle/pending acquisition results.
- `delivery_roots`: successful ready-delivery ownership lookup/transfer/creation.

These phases are inside the previous `scheduler_acquire` measurement, so do not
sum them with it. Timestamps add overhead and have different wall/thread-CPU
endpoints. They are attribution evidence, not a new throughput benchmark. Owner
CPU excludes background marker CPU; invocation CPU includes it. Idle collection
here does not measure collections invoked elsewhere in generated execution.

For each successful HTTP invocation, the driver checks exactly three named phase
records, one poll per acquisition and poll count equal to idle plus ready-delivery
count. Error exits are outside these successful-path equalities. The copied carrier
still passes sanitized owned-body validation and its missing-arrival failure
control; that fixture stubs the scheduler, so the new socket instrumentation is
covered by the real HTTP matrix rather than that sanitized fixture.

The next optimization should follow the full cost picture rather than removing
root/idle semantics based on small timings. Existing native ownership and deferred
resource cleanup contracts remain intact. This diagnostic changes no production
runtime, compiler bridge, public API or collector defaults.

## Reproduce

```sh
python3 benchmarks/gc-suite/concurrency/prepare-fiber-profile.py --scheduler \
  --build target/gc-matrix-http-build-v3 --output target/new-scheduler-build
python3 benchmarks/gc-suite/matrix/http.py \
  --build target/new-scheduler-build --output target/new-scheduler-profile \
  --modes sync-aot live-aot sync-jit live-jit --protocols close pipeline \
  --requests 1024 --rounds 1
```

Retained generated objects and proof flags, current copied native sources, private
macOS ARM64/ucontext hosts. Commands, hashes and validation output are retained in
`scheduler-build.json.gz`; checked HTTP rows and raw diagnostics are retained in
`scheduler-http.json.gz`. Only direct command/source inputs are hashed, not every
transitive header. One diagnostic run per cell cannot establish stable capacity.

## Results

All 16 cases pass 16,640 responses, including warmup. The nested count checks
pass for every invocation. Owner CPU milliseconds:

| Payload | Protocol | Mode | Poll | Idle GC | Delivery roots | Idle calls |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| plain | close | sync-aot | 13.77 | 0.19 | 2.21 | 24 |
| plain | close | live-aot | 13.00 | 1.19 | 0.77 | 20 |
| plain | close | sync-jit | 13.28 | 0.16 | 1.93 | 21 |
| plain | close | live-jit | 14.05 | 0.68 | 0.74 | 21 |
| plain | pipeline | sync-aot | 7.53 | 0.05 | 2.38 | 5 |
| plain | pipeline | live-aot | 7.87 | 0.12 | 0.30 | 4 |
| plain | pipeline | sync-jit | 7.94 | 0.04 | 2.40 | 6 |
| plain | pipeline | live-jit | 8.15 | 0.26 | 0.31 | 5 |
| json | close | sync-aot | 14.16 | 0.16 | 2.18 | 19 |
| json | close | live-aot | 14.41 | 0.99 | 0.77 | 22 |
| json | close | sync-jit | 14.30 | 0.18 | 2.09 | 21 |
| json | close | live-jit | 15.30 | 0.75 | 0.74 | 20 |
| json | pipeline | sync-aot | 8.44 | 0.03 | 3.08 | 5 |
| json | pipeline | live-aot | 8.59 | 0.59 | 0.30 | 10 |
| json | pipeline | sync-jit | 8.88 | 0.03 | 3.02 | 5 |
| json | pipeline | live-jit | 8.97 | 0.15 | 0.30 | 5 |

Socket polling dominates acquisition CPU in this cohort; idle GC and delivery
ownership do not explain the bulk of the concurrent invocation CPU penalty.
This does not prove every socket configuration has low idle-GC cost. Generated
allocation/publication-triggered collection and store admission are outside these
intervals and remain stronger targets in the existing standalone evidence.

The next bounded optimization returns to the drafted scalar-array store path:
retain type information to validate one primitive element/initialization update
once instead of lowering two generic stores. The standalone reuse-buffer sample
performed 4,194,304 updates with only 14 allocations and roughly sixfold concurrent
CPU versus sync. That is a measured lead, not proof of the proposed helper's gain.
Qualify native ownership/liveness, JIT/AOT semantics and same-collector performance
before retaining the change. Broader pause, worker scaling and default qualification
remain open. Scheduler/GC ownership semantics are preserved.
