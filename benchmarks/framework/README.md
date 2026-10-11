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

Async callbacks, setup/cleanup hooks, parameter sets and internal high-resolution
timing remain pending. Attribute discovery and generated registration are described below. Do not use
wall-clock `System.Time.Clock` as a substitute for a monotonic operation clock.
See the [framework direction](../../docs/benchmarking-framework.md).

The HTTP workbench shares `process.py` but retains its external network peer;
HTTP does not use the worker marker protocol. Its previously recorded source hashes
identify the historical script before this transport extraction.

## Attribute-driven function discovery

`BenchmarkAttribute` marks accessible, parameterless, nongeneric module functions.
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
Manual `suite.Add` remains available alongside discovered functions.

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
