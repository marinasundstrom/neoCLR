# Captured continuation registration measurements

Development measurement, 2026-10-11. This compares two source-included pending-await
adapters on the same compiler-generated state-machine path. It is **not** a comparison
with runtime suspension, which is still unimplemented, or with .NET/Go/JavaScript.

`Registrations.rvn` contains two otherwise matched awaitables:

- `registration.direct` retains callbacks directly and calls SchedulerSubmit when
  the managed Promise completes.
- `registration.captured` retains a Registration object containing the callback and
  the invocation identity from SchedulerCapture, then calls SchedulerSubmitTo at
  completion. Foreign/expired destination validation belongs to this path.

Each operation creates 32 successive pending awaitables, each with a Promise<int>,
completes each through a runtime-submitted callback, awaits the result and checks
exactly-once/deferred delivery and an accumulated sum of 32. Concurrency is one pending
inner await. Both paths use the same unpumped producer queue (with a faulting sentinel),
result value, list replacement/traversal, loop count, outer Task builder, compiler and
runtime libraries. Setup creates the producer queue before READY. The captured variant
performs capture once **per registration**, including warmup; it does not pre-capture
one token for the entire process. The runtime's lazy identity allocation is warmed up.

The measured difference includes the extra registration object, its field access/method
calls, destination capture and checked submission, plus any specialization/codegen/GC
consequences. It cannot be attributed solely to the identity comparison. The two paths
also have different placement guarantees: the direct path cannot detect accidental
retargeting. This is a cost comparison for an ownership improvement, not equivalent
full semantics or a reason to discard destination validation.

The [.NET-based design comparison](../../docs/runtime-scheduling-design.md) separates
outcome storage from continuation placement. Neither adapter implements ExecutionContext
flow, custom Scheduler/SynchronizationContext affinity or cross-invocation completion.
The extra managed wrapper is a prototype implementation choice; the eventual runtime
registration/suspension representation need not retain that allocation.

## Reproduce

Use the qualified macOS ARM64 compiler/bundle and rebuilt runtime/backend binaries:

```sh
python3 benchmarks/scheduler/prepare.py \
  --recipe docs/experiments/scheduler-projection/validation.json \
  --registrations --output target/scheduler-registration-build
python3 benchmarks/framework/runner.py \
  --manifest target/scheduler-registration-build/manifest.json \
  --output target/scheduler-registration-results.json \
  --rounds 3 --warmups 2 --operations 32
```

Preparation emits one assembly containing both cases, one AOT executable and one
shared JIT host. Each case/mode/round runs in an isolated process. Mode order rotates;
case order is direct then captured within each round (not randomized). Startup is
launch to READY, including JIT compilation; BEGIN-to-END measures the external batch
after warmup, including correctness checks and pipe delivery. AOT compilation is excluded.
The runner verifies hashed inputs before and after measurement and fails on guest faults,
bad checksums, protocol errors or changed inputs.

Three rounds are exploratory, with background load, CPU/power state and pipe-reader
scheduling uncontrolled. Results are batch throughput, not individual await latency
percentiles. Allocation counts, allocated bytes and retained memory are **not measured**;
a source-level extra object is not a measured allocation count. Windows is unqualified.
These samples should guide the next ownership/API work, not establish a cross-runtime
ranking or predict production HTTP performance.

## Recorded observations

Apple M1, 16 GiB RAM, macOS ARM64. All eighteen worker runs pass correctness and
protocol checks: three rounds per case/mode, two warmup operations and 32 measured
operations per process (1,024 inner awaits). [Raw samples](registration-results.json)
and [build inputs](registration-build.json) retain exact commands and hashes.

| Mode | Direct median ops/s (range) | Captured median ops/s (range) |
| --- | ---: | ---: |
| interpreter | 4.16 (2.53–4.35) | 3.79 (2.52–4.34) |
| jit | 238.62 (117.08–245.08) | 226.67 (126.54–237.06) |
| aot | 241.06 (163.34–251.49) | 213.48 (151.71–237.54) |

The overhead estimate is **inconclusive**. Both adapters show substantial round-to-round
slowdown, with overlapping throughput ranges in every execution mode. These data do
not establish that captured registrations are faster, nor quantify a stable penalty.
`passed: true` in the report means valid workload/protocol execution, not statistical
performance qualification. A quieter, counterbalanced rerun is required before using
this comparison to choose a production representation. Do not divide these medians
into a claimed capture-overhead percentage.

The result does establish reproducible, matched consumers for both adapters in all
three execution modes. Keep destination ownership checks for correctness; investigate
registration representation and allocation costs with instrumentation rather than
removing guarantees based on noisy external timing.
