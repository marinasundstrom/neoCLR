# neoCLR benchmarking project (development)

The benchmarking project supplies Raven benchmark workers, metadata discovery and
an external process runner. It has no testing-framework dependency. Its API reference
is BenchmarkDotNet; it implements a small experimental subset as workloads need it,
not API or measurement equivalence with BenchmarkDotNet.

## Components and APIs

| Component | Owner and contract |
| --- | --- |
| Raven worker | [`benchmarks/framework`](../../benchmarks/framework): BenchmarkSuite, callback protocol and attributes |
| Discovery | [`NeoCLR.BenchmarkDiscovery`](NeoCLR.BenchmarkDiscovery): metadata-only typed registrations |
| Host runner | [`runner.py`](../../benchmarks/framework/runner.py): isolated processes, configuration, validation and JSON reports |
| Preparation | [`benchmarks/json/prepare.py`](../../benchmarks/json/prepare.py): current macOS build-recipe adapter |
| API reference | [Benchmark APIs](../../api-docs/benchmarking.md), site route `/docs/benchmarking.html` |
| Workloads | [JSON](../../benchmarks/json/README.md), [HTTP](../../benchmarks/jit-http/README.md), [scheduler](../../benchmarks/scheduler/README.md) |

`[Benchmark]` marks a module function returning `1` after checking correctness.
Independent benchmark-owned `[Arguments]`/`[ArgumentsSource]` create separately
selectable rows. Manual `BenchmarkSuite.Add` supports captured setup data. The
[reference](../../api-docs/benchmarking.md) documents signatures and current limits.

## Prepare and run

The current preparation adapter requires a passing macOS HTTP recipe and matching
Raven compiler; it does not bootstrap the toolchain or qualify Windows automatically.
The following uses the same project-local inputs as the recorded validation:

```sh
python3 benchmarks/json/prepare.py \
  --recipe target/jit-cli-http-final/validation.json \
  --compiler /path/to/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --output target/my-benchmark-build \
  --source benchmarks/framework/Discovery.rvn --discover

python3 benchmarks/framework/runner.py \
  --manifest target/my-benchmark-build/manifest.json \
  --output target/my-benchmark-results.json \
  --mode jit --rounds 3 --warmups 2 --operations 8 \
  --filter 'Benchmarks.Discovery.Inline(5)'
```

Preparation compiles an inspection artifact, generates registration, recompiles and
verifies final metadata, then builds native artifacts and mode commands. For runtime
data sources it invokes worker `--list` outside measurement. Use fresh output paths.
The manifest captures selectors, commands and hashes. See the
[JSON instructions](../../benchmarks/json/README.md) for mixed manual/discovered setup.

`--mode` is repeatable (`interpreter`, `jit`, `aot`). `--filter` selects one exact
row; omit it to run all manifest cases. `--config` reads a version-1 JSON settings
file. Defaults are overridden by manifest settings, then config, then explicit CLI
flags. Supported `--measurement` values are `startup`, `batch-latency`,
`batch-throughput`, and `first-invocation`. The last requires zero warmups and one
operation. `--timeout` bounds worker waits. See [all limits and configuration
examples](../../benchmarks/framework/README.md#measurement-configuration).

## Interpretation and failures

Version-2 JSON reports retain effective configuration/origins, selected cases, raw
samples, summaries and input provenance. Startup includes setup and JIT work;
prebuilt AOT excludes build time. Batch timing includes correctness checks and
console/pipe transport. First invocation follows registration; it is not guaranteed
cold JIT/parsing. These metrics are not internal operation timing or allocation data.

A non-1 callback result, guest fault, process failure, stderr, bad protocol, timeout,
unknown selector or changed hashed input fails the run. The runner retains failure
information rather than turning it into a valid sample. Each case/mode/round gets
an isolated worker; mode order rotates. Benchmark jobs are not test scheduling jobs.

The current worker is synchronous. Typed-object/collection JSON workloads, internal
monotonic timing, async operations, lifecycle/reset hooks and diagnostics remain
incremental work. Migrate workloads as needed. Existing timings are exploratory;
functional framework checks do not establish performance rankings.

The [scheduler baseline](../../benchmarks/scheduler/README.md) uses a dedicated async
worker over the same external protocol. General async attribute discovery/lifecycle
support is still unimplemented; this worker does not change the synchronous API.

The scheduler preparation adapter also accepts `--registrations` for the
[direct versus captured continuation comparison](../../benchmarks/scheduler/registrations.md).
It uses two dedicated async cases in one assembly, without expanding attribute APIs.
