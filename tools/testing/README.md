# neoCLR testing project (development)

The testing project supplies Raven assertions, test discovery and serial execution.
It is source-included repository tooling today, not a released test SDK. It has no
benchmark project dependency. Its API direction follows TUnit; features are added
when required, without promising full TUnit compatibility.

## Components and APIs

| Component | Owner and contract |
| --- | --- |
| Raven library | [`runtime/raven/tests/framework`](../../runtime/raven/tests/framework): assertions, result reports, suite and runner |
| Discovery | [`NeoCLR.TestDiscovery`](NeoCLR.TestDiscovery): native metadata inspection and typed registration generation |
| Repository harness | [`scripts/test-runtime-library.py`](../../scripts/test-runtime-library.py): build and validate named suites in selected modes |
| API reference | [Testing APIs](../../api-docs/testing.md), published on the site at `/docs/testing.html` |
| Examples and evidence | [Runtime tests](../../runtime/raven/tests/README.md), including data and category contracts |

`[Test]` marks a module function returning `Result<unit, TestFailure>`. Optional
`[Arguments(...)]` or `[MethodDataSource("Inputs")]` expands it into individual
cases. `[Category("fast")]` tags every row. The reference lists supported argument
constructors, provider shapes, identity rules and errors. Manual `TestSuite.Add`
and `Skip` remain available. Assertions return Results; return/propagate failures.

The framework is independent of compiler-provided reflection invocation: metadata
inspection generates typed Raven calls, keeping AOT reachability explicit. Runtime
sources run at registration; metadata inspection never executes tests or providers.

## Running repository tests

Use a matching native development bundle, runtime, JIT helper and AOT tool. Paths
below are inputs, not automatically downloaded dependencies:

```sh
python3 scripts/test-runtime-library.py \
  --bundle target/data-cases-bundle \
  --output target/my-test-run \
  --suite discovery-contract \
  --mode jit \
  --jit-host target/benchmark-data-v2/host.dylib \
  --runtime target/release/neoclr \
  --aot tools/aot-poc/target/release/neoclr-aot-poc
```

`--suite` and `--mode` are repeatable; modes are `jit`, `interpreter`, `native`.
The current harness defaults to native plus interpreter; pass modes explicitly.
`--execution-repetitions` repeats processes for evaluation. Use `--help` for the
maintained suite list. The harness checks expected output/exit codes, verifies final
metadata registration, checks unchanged inputs and saves `report.json` plus logs.
Intentional failure fixtures are successful harness checks only when they match.
Use a fresh output directory. Native builds need the platform's supported C toolchain.

For a new source-included test project, follow
[`discovery-contract/Tests.rvnproj`](../../runtime/raven/tests/discovery-contract/Tests.rvnproj):
include Assertions, TestSuite, TestAttribute and `$(NeoClrTestRegistry)`. Generate
that file with:

```sh
python3 scripts/discover-runtime-tests.py \
  --project path/to/Tests.rvnproj --bundle path/to/bundle \
  --output target/my-discovery
```

Set `NeoClrTestRegistry` to the resulting absolute `TestRegistry.rvn` path for the
final build with the same bundle. This helper is a discovery/build step, not a test
executor. The repository harness performs both compilation passes and verification.
A project's `Main(arguments: string[]) -> int` creates a suite, calls
`RegisterDiscoveredTests(suite)`, then returns `TestRunner.Run(suite, arguments)`.

## Guest runner selection and results

A built test executable accepts no arguments (all cases), `--filter <substring>`,
`--id <exact-id>`, or `--category <exact-category>`. Selectors are case-sensitive;
choose one selector per run. Runtime CLI launches use `--` before guest arguments;
standalone native executables receive them directly.

The console reporter prints PASS/FAIL/SKIP and totals. Exit codes are 0 for success,
1 for assertion failures, and 2 for invalid registration/selection. A runtime fault
is terminal; it is not converted to an assertion failure. Cases run serially and
continue after Result-based assertion failures. Runtime data-source faults are also
terminal. Reports carry individual case IDs, names and outcomes.

## Scope and next work

The current contract supports module functions, synchronous result-based tests,
inline scalars and typed array sources. Async tests, class fixtures, lifecycle hooks,
parallel execution and source generators remain gaps. See the
[parallelism proposal](../../runtime/raven/tests/README.md#parallel-execution-and-class-grouping--proposal-2026-10-11).
Categories select cases and never imply scheduling constraints. Migrate existing
program-style tests incrementally when that work benefits from the framework.
