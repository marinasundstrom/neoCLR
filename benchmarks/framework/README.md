# Development benchmark runner

This is the first executable foundation for the planned BenchmarkDotNet-like
framework. It is repository tooling, not a distributed runtime-library API.

`Worker.rvn` provides manual registration of module functions or callbacks:
`BenchmarkSuite.Add(name: string, operation: () -> int)` and
`BenchmarkSuite.Run(arguments: string[])`. An operation returns `1` after checking
its result; any other value or fault fails the process. Names must be unique and
nonempty. The exact case name, warmup count and measured operation count are the
three worker arguments. Counts are bounded: 0–10,000 warmups and 1–10,000 measured
operations. Unknown names, malformed counts and duplicate registration fail.

The external runner launches one isolated process per case/mode/round, rotates
mode order, and retains every sample. Each process performs its own warmup. The
runner requires all three command configurations and unchanged hashed inputs.
`--filter` selects an exact registered name; unknown filters fail. Existing result
files are never overwritten. Process failures, stderr, extra output, malformed
markers, unexpected checksums and timeouts fail the run and retain a failure report.

```sh
python3 benchmarks/framework/runner.py \
  --manifest target/benchmark-json-nodes-final/manifest.json \
  --output target/benchmark-json-nodes-final/results.json \
  --rounds 3 --warmups 2 --operations 8
python3 -m unittest discover -s benchmarks/framework -p 'test_*.py'
```

Version 1 manifests contain `commands` for `interpreter`, `jit` and `aot`, `cases`,
and `inputs` mapping absolute paths to SHA-256 hashes. The JSON preparation tool
fresh-builds those inputs. CLI jobs receive `--` before worker arguments; native
executables receive arguments directly. These private tool contracts may evolve.

## Measurement contract and limits

Worker stdout has four lines: `NEOBENCH/1 READY`, `NEOBENCH/1 BEGIN`,
`NEOBENCH/1 END`, and the checksum. Setup/registration finishes before READY.
Warmup runs between READY and BEGIN. The controller timestamps lines with Python's
monotonic performance clock and measures the BEGIN-to-END interval. It records
startup separately, including metadata loading and JIT compilation. AOT build time
is recorded by preparation and excluded from startup.

**This measures external batch latency**, including callback dispatch, correctness
checks, console/pipe transport and reader scheduling. It is not per-operation
internal timing; a divided batch duration does not remove these costs. Short
batches can be distorted by buffering/scheduling. Counts are fixed, with no pilot
calibration, baseline subtraction, confidence interval, allocation diagnostics or
statistical stability guarantee. Report median and range of raw batch samples.
The native and interpreter collectors also differ, so do not isolate serializer
or optimizer efficiency from these figures.

Async callbacks, setup/cleanup hooks and internal high-resolution
timing remain pending. Attribute discovery and generated registration are described below. Do not use
wall-clock `System.Time.Clock` as a substitute for a monotonic operation clock.
See the [framework direction](../../docs/benchmarking-framework.md).

The HTTP workbench shares `process.py` but retains its external network peer;
HTTP does not use the worker marker protocol. Its previously recorded source hashes
identify the historical script before this transport extraction.

## Attribute-driven function discovery

`BenchmarkAttribute` marks accessible nongeneric module functions.
Classes are not required. The current worker requires `int` with success token `1`;
other return types and async functions remain unsupported, rather than silently
omitted. For example, from the executable discovery sample:

```raven
[Benchmark("Sum a sequence of integers")]
func Sum() -> int {
    var total = 0
    for value in 0..<100 {
        total += value
    }
    if total != 4950 {
        System.Fail("Sum changed")
    }
    return 1
}
```

The marker derives from `System.Attribute` and uses
`AttributeUsage(AttributeTargets.Method)`. Constructors are `init()` and
`init(description: string)`; `var Description: string { get; set; }` also accepts
`[Benchmark(Description: "text")]`. Empty descriptions use the qualified function
name. Description is display metadata; exact filtering uses the qualified name.
Manual `suite.Add` remains available alongside discovered functions. Add
`--manual-case <selector>` when preparing a mixed manifest; it can be repeated,
but empty or duplicate selectors fail preparation.

Preparation with `--discover` compiles an inspection assembly, reads native custom
attributes through the metadata introspection API, writes a typed Raven registry,
recompiles, and verifies that the final assembly has the same discovered contract.
Neither benchmark bodies nor attribute constructors execute during discovery.
Typed function references preserve AOT reachability without runtime reflection
invocation or a separate reflection-retention file. This is build-time metadata
discovery with generated registration; in-process dynamic discovery is not shipped.

```sh
python3 benchmarks/json/prepare.py \
  --recipe target/jit-cli-http-final/validation.json \
  --compiler /absolute/path/to/native-enabled/rvnc.dll \
  --output target/benchmark-discovery \
  --source benchmarks/framework/Discovery.rvn --discover
python3 benchmarks/framework/runner.py \
  --manifest target/benchmark-discovery/manifest.json \
  --output target/benchmark-discovery/results.json \
  --filter Benchmarks.Discovery.Sum
python3 benchmarks/framework/validate_discovery.py \
  --build target/benchmark-discovery/build.json \
  --output target/benchmark-discovery-validation
```

Selectors are qualified names; the inventory additionally records a stable ID
composed of assembly, module and function identity, excluding assembly version.
Results are sorted by ordinal name. Invalid signatures, no discovered benchmarks,
or unsupported identifiers fail preparation. Failed metadata discovery removes
stale registry/inventory outputs. Generated adapters currently require plain ASCII
Raven identifiers. The attribute and worker remain repository tooling, not a new
public runtime-library assembly.

Compared with BenchmarkDotNet's method attributes, module functions avoid compulsory
fixture classes. This first contract has narrower return/setup/async support and
uses generated direct calls instead of runtime reflective invocation. Those are
current implementation limits, not permanent API choices.


Qualification: [inventory](discovery-evidence/benchmarks.json),
[generated registry](discovery-evidence/BenchmarkRegistry.rvn),
[nine execution checks](discovery-evidence/execution.json) and
[six discovery checks](discovery-evidence/validation.json). Execution covers two
discovered functions plus one manual case in interpreter, JIT and AOT on macOS
ARM64. These are functional checks, not benchmark performance measurements.

## Measurement configuration

Use `--config <file.json>` with `schemaVersion: 1`. Settings resolve in this order:
built-in defaults, manifest `benchmarkConfig`, the config file, and explicit CLI
flags. Reports use schema version 2 and retain the effective configuration, each
setting's origin, selected cases, the primary metric/unit and all raw metrics.

Supported measurements: `startup` (process launch through READY, including setup),
`batch-latency` (BEGIN through END), `batch-throughput` (operations per that interval),
and `first-invocation` (one invocation after setup, requiring zero warmups and one
operation). First invocation is not cold parsing or cold JIT: setup may exercise
the same code, and JIT compilation occurs before READY. All operation views include
validation, callbacks and external transport/scheduling costs.

Configurable fields are `measurement`, `modes`, `rounds`, `warmups`, `operations`,
and `timeout`. Modes are a nonempty unique subset of interpreter/JIT/AOT. At least
three rounds are required. Timeout is seconds per output/exit wait, not a total
job deadline. Unknown fields, invalid values, unavailable selected modes and
unsupported measurements fail before launching workers. Internal timing,
concurrency, allocation and GC diagnostics remain unsupported.

Examples are in [configurations](configurations):

```sh
python3 benchmarks/framework/runner.py \
  --manifest target/benchmark-json-discovered/manifest.json \
  --output target/configured.json \
  --config benchmarks/framework/configurations/warmed-batch.json \
  --measurement batch-latency --mode jit --mode aot \
  --filter Benchmarks.Json.DeserializeNodes
```

CLI flags override config values individually; repeated `--mode` replaces the
mode list. The existing count/filter flags remain available. Attribute defaults
and data parameters are subsequent layers; no unsupported settings are ignored.


## Argument rows (development)

The benchmark framework owns `NeoClr.Benchmarking.ArgumentsAttribute` and
`ArgumentsSourceAttribute`, following [BenchmarkDotNet's naming](https://benchmarkdotnet.org/articles/features/parameterization.html).
It has no dependency on the test framework, including its discovery implementation.

Use repeated `[Arguments(5)]` annotations with `[Benchmark]` to register a separate
case for each input row. The current constructor overloads accept one integer,
string or Boolean; two or three integers; two strings; or a string and integer.
Inline arguments must match parameter types and counts exactly. Native attribute
array/Type/null support and arbitrary argument combinations remain gaps.

`[ArgumentsSource("Inputs")]` names a public/internal parameterless nongeneric module
function returning `T[]` for a benchmark accepting one `T`. Names may be qualified;
unqualified names use the benchmark's module, within the same assembly. Sources run
once during registration, outside warmup/measured batches (but included in startup).
Each input is captured and reused for that case's invocations: mutable inputs must
account for that reuse. Async sources, tuple spreading and setup/reset hooks remain
unsupported. Empty sources or more than 10,000 rows fault registration.

Inline selectors append `(values)`; source selectors append `/Qualified.Inputs[index]`.
Every row can be filtered and reported independently. Source order must be deterministic:
row IDs are positional. Metadata inspection generates typed registrations without
running user code; its inventory marks source templates `dynamic: true`. Worker
`--list` emits `NEOBENCH/1 CASES`, all registered selectors, then `NEOBENCH/1 END`.
Preparation executes this outside measurement when runtime sources are present,
including manual registrations in the inventory. No reflection invocation is needed
in interpreter/JIT/AOT execution. See [the executable example](Discovery.rvn).

Invalid arity/types, duplicate inline rows/sources and missing/incompatible providers
fail discovery and remove stale output. This is a deliberately bounded subset;
benchmark features will grow independently as workloads require them.

Validation: [21 worker checks](data-evidence/execution.json) cover seven registered
cases in all three modes; [six final source checks](data-evidence/source-execution.json)
qualify the scoped adapter correction. [Eleven discovery checks](data-evidence/discovery.json)
cover invalid signatures/data and metadata-only inspection. These macOS ARM64 checks
establish behavior, not benchmark rankings or Windows qualification.
