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

Async callbacks, setup/cleanup hooks, attribute discovery, generated registration,
parameter sets and internal high-resolution timing remain pending. Do not use
wall-clock `System.Time.Clock` as a substitute for a monotonic operation clock.
See the [framework direction](../../docs/benchmarking-framework.md).

The HTTP workbench shares `process.py` but retains its external network peer;
HTTP does not use the worker marker protocol. Its previously recorded source hashes
identify the historical script before this transport extraction.
