# Managed JIT and three-mode test benchmark

2026-10-11 development experiment. A private **macOS ARM64 synchronous JIT** now
executes the existing Raven Index/Range test assembly: all 20 tests pass, including
classes, generics, arrays, strings, list interfaces, extension indexers and iterator
disposal. No-match filtering and a bounds fault agree with AOT and interpretation.
Registration is discovered/generated at build time using TestAttribute; this does
not claim JIT-time reflection discovery. There are no Raven bridge or public runtime
API changes. Normal runtime execution defaults are unchanged.

## Contract and implementation

`neoclr-aot-poc --jit-managed --closed-world INPUT ROOT HOST_DYLIB ...bindings...`
uses the existing selection, verification and managed lowering over Cranelift's
`Module` interface. The third positional argument is a trusted native helper library,
instead of the object output used by AOT. Required bindings include native GC,
reference arena, stack roots and stack budget. Repeat `--jit-entry-argument TEXT` to
pass guest arguments; argv[0] is supplied by the probe. Use the verifier below to
reproduce the exact bindings from a successful native build report.

All selected functions are compiled eagerly into memory; there is no per-method lazy
compilation, tiering or cache. The probe does not invoke a system linker or emit a
guest object. The helper library is built separately, reusing existing AOT GC, text,
console, fault, entry-argument and stack helpers. Cranelift remains pinned to 0.121.2.
Long-range calls are explicit for independently allocated JIT functions and native
helpers; colocated ARM64 branches failed during initial qualification. AOT retains
its linker-managed relocation policy, with byte-identical object output verified for
this assembly. The implementation follows the pinned JIT backend's non-PIC and
non-colocated-libcall settings, rather than assuming all functions share an allocation.

Every declared helper import must resolve before finalization/invocation. An incomplete
host fails before its entry can execute. The host and generated code remain owned for
the synchronous invocation. `host.c` uses a 1 MiB heap, renders faults before cleanup,
checks that no stack roots/strong handles remain, collects, then frees the heap before
code reclamation. Broken cleanup invariants abort. This private trusted adapter is not
a general embedding ABI or an isolation boundary; native loops have no instruction budget.

Async entries, timers, task queues, sockets and host bootstrap are rejected. Windows
JIT, suspension, hot reload, concurrent invocations and retained callbacks remain
unqualified. Future asynchronous code needs a longer-lived code owner and explicit
quiescence before unload. AOT-only deployment remains independent of the JIT library.

Guest output is unchanged. A final stderr line `JIT_RESULT {json}` supplies
`guestExitCode`, `compileAndFinalizeSeconds` and `invocationSeconds`. Process exit 0
means the probe completed, even if the guest returned nonzero; consumers must inspect
`guestExitCode`. Binding/compilation failures exit 1 without a result line.

## Reproduction

Use the existing successful Index/Range build from the
[iteration experiment](../jit-iteration/README.md), its matching library bundle and
Raven compiler `163050da2a79444878169f1d011abff644d37b2b`. Build the native tool in release
mode. On this host, set `DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer` and
`SDKROOT` to that installation's `MacOSX26.2.sdk` before building.

```sh
cargo build --locked --release --manifest-path tools/aot-poc/Cargo.toml
python3 docs/experiments/jit-managed/verify.py \
  --build target/jit-iteration-release-baseline/index-range-build \
  --output target/jit-managed-validation-final
python3 docs/experiments/jit-managed/benchmark.py \
  --qualification target/jit-managed-validation-final/validation.json \
  --build target/jit-iteration-release-baseline/index-range-build \
  --output target/jit-managed-benchmark/benchmark.json
```

Choose fresh output paths. The verifier compiles the helper library and checks three
cases in all three modes, missing-helper rejection, and identical AOT object bytes.
[Qualification evidence](validation.json) retains hashes and commands. Focused native
regressions pass: 22 scalar, 56 value, three Windows object-emission tests and one
Windows frame-boundary check; four
Windows execution tests are ignored on macOS. Existing stale limit/negation rejection
expectations were corrected to match already implemented behavior.

## Benchmark result

Apple M1, macOS 27 ARM64, release-built Rust interpreter and compiler, same compiled
Raven assembly and libraries. One unmeasured warm-up per mode, then nine fresh-process
runs per mode, sequentially rotating order AOT/interpreter/JIT. Every run must have
identical successful output for all 20 tests. Samples and artifact hashes are retained
in [benchmark.json](benchmark.json); no concurrent build/test workloads were launched.
OS caches and background system load are not controlled. These are local observations,
not cross-platform or statistically general performance claims.

| Measurement | Median | Min–max |
| --- | ---: | ---: |
| Prebuilt AOT process | 9.00 ms | 8.38–15.01 ms |
| Interpreter process | 2,961.54 ms | 2,924.98–3,373.99 ms |
| JIT process, including compilation | 2,345.66 ms | 2,311.21–2,558.26 ms |
| JIT host loading + lowering/finalization | 430.69 ms | 421.27–488.14 ms |
| JIT invocation, including host setup/cleanup | 5.64 ms | 5.53–6.43 ms |

JIT's median process time is **20.8% lower (1.26× faster)** than interpretation for
this test run. Prebuilt AOT has a much shorter launch-to-exit time. Its compilation and
linking are excluded, while JIT compilation is paid in each process. Source compilation,
test discovery and building the tools/helper library are setup costs excluded for all
three modes; this is not an end-to-end edit/build/test benchmark.

JIT invocation timing excludes CLI metadata loading, resolution/selection and native
compilation. Compilation timing starts at helper-library loading, after metadata work.
The remaining process time therefore includes metadata work and other startup overhead;
it has not been independently decomposed. Do not compare the 5.64 ms invocation directly
with the 9.00 ms AOT process as a code-speed ratio, or infer interpreter dispatch cost
from its whole-process duration. This workload proves useful test execution and a local
process-time improvement. It does not establish steady-state HTTP/JSON throughput,
long-running JIT/AOT parity, or a performance advantage over .NET or another framework.

The next execution milestone is async/HTTP support with explicit runtime/code lifetime;
next performance work should isolate metadata startup and measure a sustained workload.
The author's [execution direction](../../platform-direction.md#execution-direction-clarified-2026-10-11)
selects eventual JIT defaults, preferred AOT deployment and full interpreter support.
