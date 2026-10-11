# Retained JIT code and sustained CPU work

2026-10-11, private macOS ARM64 experiment. The goal remains a functioning JIT;
this slice exercises compile-once/invoke-repeatedly ownership and distinguishes CPU
execution from the [managed test startup measurements](../jit-managed/README.md).
It does not expand the admitted managed library or asynchronous workload surface.

## Implemented step

The private scalar command accepts `--repetitions 1..1000` after its existing arguments.
It loads/verifies/compiles/finalizes once, keeps the same generated entry alive across
all synchronous calls, then releases its code owner. Each call has a fresh output slot;
success and divide-by-zero outcomes are tested across repeated invocations. Invalid
repetition counts fail before loading or executing the input. This is no public callable
handle, cross-process cache, concurrent execution or hot-reload API.

The result adds `invocations`, each with status, value and invocation duration. Existing
top-level status/value/invocation timing describe the first call. Compilation timing
occurs once. A fault is a per-call result; later calls are still attempted. Code pointers
never leave the owner scope. Managed and asynchronous retained-code lifetimes require
separate heap/root/callback ownership; this scalar experiment does not authorize them.

This follows the separation of compilation and execution in the existing
[.NET/JIT comparison](../../native-execution-investigation.md), using neoCLR's own
status/result ABI. Reusing code avoids compilation on every call, at the cost of
retaining executable storage until all calls finish. It is not tiered compilation,
lazy compilation or PGO. No .NET performance result is inferred.

## Workload and measurement

All three modes execute the same `workload.neoil`: an Int32 recurrence with wrapping
multiply/add, a direct helper call per iteration, and a counted loop. The input seed is
42. An independent Python implementation computes and checks every returned checksum.
Counts are 0, 10,000, 100,000 and 1,000,000. Zero is an invocation-overhead control;
increasing count increases computation without increasing metadata size.

For each count, three fresh-process rounds rotate JIT/AOT/interpreter order. Each
process executes four calls against retained compiled/resolved code. The first call is
retained as warm-up data but excluded from the nine-sample invocation summary. Every
call, including warm-ups, must match the checksum. Process timing includes all four
calls and startup, so it must not be compared directly with one invocation's duration.

- JIT uses existing scalar lowering with explicit long-range calls and one code owner.
- AOT uses the same scalar lowering and a prebuilt C host. Native code compilation and
  host linking are recorded separately, outside invocation/process timings.
- Interpreter loads/verifies once, resolves a function handle once, and invokes it with
  fresh guest state and a 30-million-instruction limit for each call. Invocation timing
  includes argument import, state setup and execution. Native loops have no instruction
  quota, so enforcement costs differ and are part of the current interpreter contract.

This is synthetic integer/call-heavy code, with no allocations in guest code, GC,
networking, JSON, scheduling or I/O. Call costs can dominate interpreter results. Both
native paths retain their existing lowering/optimization settings; the experiment does
not select a new compiler optimization policy. Native and interpreted hosts differ in
state setup and timer placement. OS caches and background host load are uncontrolled.
No broad backend ranking follows from one workload; metadata-size scaling remains open.

## Reproduction

Build release tools; on the measured host use the Xcode installation and macOS 26.2 SDK
as described in the [managed experiment](../jit-managed/README.md#reproduction).

```sh
cargo build --locked --release --manifest-path tools/aot-poc/Cargo.toml
cargo build --locked --release --example jit_cpu_interpreter
python3 docs/experiments/jit-cpu/benchmark.py --output target/jit-cpu-benchmark
```

Use a fresh output directory. The script records commands, artifact hashes, build times,
all per-call/process timings and exact checksums. Each subprocess has a wall-clock timeout.
This is a local development measurement, not a published cross-platform benchmark.

## Local result

Apple M1/macOS 27 ARM64, release-built tools, three process rounds and nine measured
invocations per count/mode. All **144 invocations** (including warm-ups and zero-work
controls) return the expected checksum. All 23 focused scalar compiler tests pass,
including repeated successful/faulting JIT calls and invalid repetition counts.
[Raw evidence, commands and hashes](validation.json).

| Loop iterations | JIT invocation median | AOT invocation median | Interpreter invocation median |
| --- | ---: | ---: | ---: |
| 10,000 | 0.0286 ms | 0.0290 ms | 32.368 ms |
| 100,000 | 0.2713 ms | 0.2730 ms | 305.511 ms |
| 1,000,000 | 2.7133 ms | 2.7430 ms | 3,056.177 ms |

At one million iterations, observed ranges are 2.712–2.743ms JIT, 2.732–2.948ms AOT,
and 3,026–3,233ms interpreted. Native invocation medians are close for this shared-lowering
workload. The roughly three-order-of-magnitude gap to interpretation is specific to this
synthetic scalar/direct-call loop, including current interpreter call/state/quota costs.
It is not an expected application-wide multiplier or a comparison with another platform.
The zero-work AOT calls fall below the host clock's observed resolution and report zero;
no speed ratio is derived from that control or the smallest timings.

At the largest count, median whole-process times are 75.0ms JIT, 16.0ms prebuilt AOT and
12.290s interpreter, each including four invocations. These remain different from the
per-invocation figures above. JIT process time includes its single compilation; AOT
build costs and all tool builds are excluded. All original phase data remain in the report.

The next functional milestone is **managed async entry/callback lifetime**, followed by
HTTP qualification. A functioning JIT needs those runtime services and production tooling,
not just a fast scalar kernel. Interpreter call/dispatch profiling is justified by this
workload but should not displace that JIT milestone. Metadata-size scaling, allocations,
GC and networking need separate evidence before generalizing these numbers.
