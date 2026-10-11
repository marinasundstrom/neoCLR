# Development iteration and the first JIT slice

2026-10-11. Local investigation, not a published benchmark or a JIT implementation.
The author requests evaluating JIT to improve the edit–test cycle while retaining
interpretation for platforms where it fits. See the
[execution comparison and .NET baseline](../../native-execution-investigation.md#jit-for-development-iteration-2026-10-11).

## What the harness measures

`scripts/test-runtime-library.py` now records discovery, project build and registration
verification durations. Discovery separates Raven source compilation from metadata
inspection, including the discovery tool's own build. The project build reports
Raven compilation, native selection/lowering/emission, native kernel build,
combined C host compilation/linking, dependency inspection and toolchain lookup.
Nested command timings are parts of their enclosing phase, not additional totals.

`--execution-repetitions N` runs each full suite N times in both modes. Every repetition
still checks the expected exit code and exact test output. Fault guards and runner
selectors run once; their timing is recorded separately. The default remains one
execution. Process timings include launch, loading and execution; they do not isolate
interpreter dispatch or guest execution. Total time includes checks and bookkeeping.
Failed commands/timeouts can leave partial timing evidence and must not be compared
as successful runs.

The HTTP lifetime suite covers handlers, disposal and cancellation. It is not a
socket throughput benchmark. Index/Range covers value contracts, array/string/list
access and the new extension indexers. Both are useful tests of repeated development
runs, but neither establishes production service latency.

## Reproduction

Use the matching development bundle qualified for Raven `163050da2` and the
Index/Range library slice. Keep the bundle, runtime, source and native host fixed.
The existing runtime test harness records source/artifact hashes. On this macOS host,
set `DEVELOPER_DIR` to the Xcode installation and `SDKROOT` to its macOS 26.2 SDK;
the default Command Line Tools SDK failed the libffi release build configuration.

```sh
cargo build --locked --release --manifest-path tools/aot-poc/Cargo.toml
python3 scripts/test-runtime-library.py \
  --bundle target/extension-indexer-final-bundle \
  --suite http-lifetime --suite index-range \
  --execution-repetitions 3 \
  --aot tools/aot-poc/target/release/neoclr-aot-poc \
  --output target/jit-iteration-release-baseline
```

Use a fresh output directory for each run. The comparison run supplies
`tools/aot-poc/target/debug/neoclr-aot-poc` instead. Release here describes the Rust
compiler tool build; it does not select a different optimization policy for guest
code. Building the compiler itself is a separate setup cost, excluded from the test
harness total. OS/file caches are not reset; first and later samples are retained,
not labeled as controlled cold/warm measurements. Three samples are exploratory.

## Local observations

[Retained evidence](validation.json): Apple M1/macOS ARM64, three process runs per
suite/mode and one build per tool profile. Both suites pass all repetitions, and all
20 Index/Range guard processes pass for each tool profile. The existing debug AOT
binary and freshly built release binary are identified separately by hash.

| Suite | Debug AOT compile | Release AOT compile | Native process median, debug/release tool | Interpreter process median, first/second run |
| --- | ---: | ---: | ---: | ---: |
| HTTP lifetime (10 tests) | 48.254s | 9.429s | 0.048s / 0.069s | 3.713s / 6.148s |
| Index/Range (20 tests) | 24.541s | 6.654s | 0.009s / 0.012s | 3.004s / 4.684s |

The compiler profile reduces the observed native compilation cost, but **whole-harness
time was 167.015s versus 161.937s**, including repeated execution and guard processes.
Source compilation and interpreter process time increased in the second run even
though the interpreter binary is unchanged. These sequential, uncontrolled samples
do not establish a repeatable end-to-end speedup or isolate the cause of that variance.
The first native launches were also slower than later launches; retain the individual
samples rather than presenting their median as startup latency.

This identifies work outside interpretation: each suite is compiled once for discovery
and again with generated registration; discovery also invokes a .NET build/run tool.
A JIT can remove object/executable publication and native host linking from its own
run path, but compilation, metadata work and runtime startup still need measurement.
Reusing lowering alone does not eliminate the observed native selection/codegen cost.
Use release compiler builds for the next experiment and time JIT compilation explicitly.
A controlled loopback/network workload and runtime loading/execution split are still
needed before predicting server throughput or the achievable edit–test improvement.

## Concrete JIT scope

The subsequent [scalar JIT slice](../jit-scalar/README.md) implements the first step
below. The subsequent [managed probe and benchmark](../jit-managed/README.md)
now exercise a real synchronous test assembly. The recommendations below record the
original experiment sequence.

Recommendation: begin a bounded JIT feasibility slice, while also using a release
compiler for realistic native iteration measurements. Reuse verified native lowering
rather than introduce another IL-to-machine-code implementation. The current scalar
and managed paths construct `ObjectModule` directly; managed helpers in `values.rs`,
`gc_probe.rs` and `fault_details.rs` also depend on it. Extract emission over the
Cranelift `Module` abstraction without changing AOT output/behavior first.

The first execution should be a verified Int32 function with arguments, arithmetic,
branches and a direct call. Compare its result with interpretation and AOT, reject
unsupported signatures/instructions explicitly, and measure compile/finalize time
separately from invocation. Own the generated code for the entire invocation lifetime.
This is a compiler feasibility test, not the developer-experience milestone.

The next meaningful milestone is executing a real test assembly through the same
metadata selection and runtime services, followed by loopback HTTP client/server
and JSON workloads. It needs explicit helper symbol binding, managed root reporting,
fault behavior, callbacks, code lifetime, permissions and diagnostics. Preserve an
AOT-only deployment path. Suspension/scheduling must share those ownership contracts;
no green-thread stack design is selected here. Tiering, dynamic PGO, hot reload and
on-stack replacement remain separate decisions.

The [Cranelift JIT module documentation](https://docs.wasmtime.dev/api/cranelift_jit/struct.JITModule.html)
provides the same Module interface and in-memory finalization, and requires generated
code pointers not to outlive released code storage. Retrieved 2026-10-11, this is
rolling 0.138.0-dev documentation; implementation must verify APIs against the existing
0.121.2 dependency pin. It does not supply neoCLR's runtime ownership contracts.
