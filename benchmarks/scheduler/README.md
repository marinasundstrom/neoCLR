# Scheduler and state-machine baseline

Development baseline, 2026-10-11. This is the current Raven compiler-generated
state-machine path over runtime scheduler submission, not runtime-managed suspension.
It provides a retained workload for later matched comparisons within each execution
mode. Like .NET's compiler-lowered async path, Task/Promise remain managed and the
scheduler delivers callbacks; this does not imply identical implementation or cost.

`StateMachine.rvn` is a dedicated async worker using the existing independent
benchmark runner's `NEOBENCH/1` protocol. It does not change the framework's synchronous
attribute/callback contract. One operation performs **32 sequential awaits**, checks
the sum, and returns through an additional await in async Main:

- `await.completed`: creates and awaits already-completed `Task<int>` values.
- `await.pending`: creates a Promise owner, submits its completion callback through
  the reserved internal SchedulerSubmit service, verifies deferred/exactly-once
  delivery, and awaits the result before admitting the next callback.

Concurrency is one operation with at most one submitted callback pending. Each inner
result is the integer 1. These are different workloads, not interchangeable timing
controls: pending work includes an owner, Promise, callback and dispatch. Neither
case isolates the bare cost of a context switch or proves a scheduler improvement.
The pending case uses the existing producer-queue state-machine continuation adapter.

## Reproduce

Build matching runtime/backend binaries and qualify the scheduler projection first.
The preparation adapter currently requires the recorded macOS ARM64 recipe and its
local compiler/bundle inputs; it does not bootstrap a toolchain or qualify Windows.
Use fresh output paths:

```sh
python3 benchmarks/scheduler/prepare.py \
  --recipe docs/experiments/scheduler-projection/validation.json \
  --output target/scheduler-baseline-build
python3 benchmarks/framework/runner.py \
  --manifest target/scheduler-baseline-build/manifest.json \
  --output target/scheduler-baseline-results.json \
  --rounds 3 --warmups 2 --operations 16
```

The preparation step builds a fresh assembly, shared JIT host and AOT executable.
Both native modes and the interpreter use the same runtime bundle. JIT runs through
`neoclr run --execution jit`; its development binding set now includes the reserved
scheduler service. Raven Runtime Contract configuration and lowering are unchanged.
This remains an internal service, not a new public managed Scheduler API.

## Measurement limits

Each case/mode/round runs in a fresh process; mode order rotates. Startup is launch
to READY (including JIT compilation); batch time is BEGIN to END after warmup, with
correctness checks and pipe transport included. AOT build time is excluded. The
worker, compiler dependencies, runtime libraries, runtime/backend executables and
native host inputs are hashed in the build/report. The baseline records the parent
source revision plus hashes of the changed worker and built executable inputs.

These are exploratory external batch samples, not per-await latency distributions.
Short native batches are especially sensitive to transport and scheduling noise.
Allocation counts/bytes, retained memory per pending operation and internal
suspend/resume timing are **not measured**. No allocation estimate is inferred from
RSS. CPU utilization, power and background system load are uncontrolled.

Next add instrumentation and representative fan-out/timer/I/O workloads as those
contracts develop. Retain these state-machine cases when a runtime-suspension path
exists, then compare both on the same scheduler, hardware, execution mode, workload,
bundle, warmup and optimization settings. External .NET/Go/JavaScript comparisons
remain separate. See the [measurement gates](../../docs/runtime-scheduling-design.md#scheduler-and-suspension-benchmark-requirement--2026-10-11).

## Recorded baseline

Apple M1, 16 GiB RAM, macOS ARM64; three isolated rounds per case/mode, two
warmup operations and sixteen measured operations per process. [Raw samples](results.json)
and [build provenance](build.json) retain commands, hashes and ranges.
All eighteen measured processes passed their checks. [Unknown-case rejection](rejections.json)
also passes in all three modes without producing benchmark protocol output.
Windows has not been qualified for this slice.

| Case | Mode | Median operations/s | Median startup (s) |
| --- | --- | ---: | ---: |
| await.completed | interpreter | 9.88 | 1.675 |
| await.completed | jit | 599.18 | 1.352 |
| await.completed | aot | 602.22 | 0.004 |
| await.pending | interpreter | 4.39 | 1.677 |
| await.pending | jit | 279.77 | 1.324 |
| await.pending | aot | 288.36 | 0.004 |

These numbers characterize this fixture and machine only. In particular, the first
completed-await AOT sample was slower than the other two; retain the raw range rather
than interpreting the small JIT/AOT median differences as a ranking.

## Interpreter readiness regression check

The subsequent private readiness correction keeps submitted callbacks from being
reported as Idle/Pending and moves source-turn arbitration inside the scheduler.
[Six fresh interpreter runs](readiness-results.json) retain the exact baseline
assembly, bundle and workload settings. [Input substitution](readiness-inputs.json)
records the replaced runtime binary hash and changed Rust source hashes; all other
baseline manifest inputs were checked unchanged before running. The full runner
checks their hashes again before and after measurement. To repeat from fresh
artifacts, use the preparation steps above and add `--mode interpreter` to the runner.

| Case | Baseline median operations/s | Readiness correction median operations/s |
| --- | ---: | ---: |
| await.completed | 9.88 | 9.82 |
| await.pending | 4.39 | 4.36 |

The small differences are not evidence of improvement or regression: these are
three-round, non-interleaved local measurements with uncontrolled background load.
This is a correctness change preserving existing dispatch behavior, with a retained
performance observation rather than a speedup claim. JIT/AOT code is unchanged and
was not remeasured for this interpreter-only slice.
