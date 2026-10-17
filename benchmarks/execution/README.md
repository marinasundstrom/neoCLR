# Preview 14 general execution baselines

Three explicitly registered cases use the existing `BenchmarkSuite` and external
process runner. They use the same compiled Raven application in interpreter/JIT
and its native AOT compilation, with the released macOS bundle's compiler/libraries.

- `cpu.sum`: add integers from 0 through 9,999 and verify 49,995,000. One operation
  is a complete 10,000-iteration loop, not one integer addition.
- `json.serialize`: serialize a prepared JSON node tree and compare the complete
  output with the canonical nested Unicode document.
- `json.deserialize`: parse that document and check root shape/count and name.
  Before READY, setup validates a full parse/serialize round trip.

The JSON document includes a nested object, arrays, Boolean/numeric/null values and
Unicode. These are JSON DOM cases; typed object serialization belongs to the HTTP
JSON workload and must not be inferred from node results.

```sh
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
python3 benchmarks/execution/prepare.py --bundle /path/to/neoclr-native-poc \
  --output /fresh/path/to/execution
python3 benchmarks/framework/runner.py \
  --manifest /fresh/path/to/execution/manifest.json \
  --output /fresh/path/to/execution/results.json \
  --rounds 5 --warmups 2 --operations 16 --timeout 300
```

The output must be fresh. Preparation uses the repository console native host profile; the timed workload performs no networking.
Native host/GC policies differ from JIT and are recorded with the build. It
verifies the bundle hashes and retains build/input/artifact hashes. The runner checks
inputs before and after measurement. No concurrent benchmark/build should run.

Each case/mode/round gets a fresh process. Startup includes registration and JSON
setup (also for the CPU case), metadata loading and JIT compilation; AOT build time
is excluded. Operation timing is external BEGIN-to-END batch wall time: it includes
callback dispatch, result checks, console/pipe markers and reader scheduling. Dividing
by the operation count does not remove those costs. This is not isolated serializer
CPU, a scalar instruction microbenchmark or an allocation benchmark.

Report median and min/max across the five rounds and retain all samples. Two warmups
per process and fixed batches are provisional; there is no adaptive calibration or
statistical confidence claim. Interactive host load remains uncontrolled. Peak RSS,
allocation rate, deployment size, Windows and external-runtime comparisons are not
measured here.

## Recorded runs — 2026-10-13

Apple M1/macOS 27.0, Preview 14 macOS ARM64 bundle. All 45 initial worker processes
and ten larger native CPU processes pass, with checksums and input hashes intact.
The host was interactive, so these are exploratory baselines rather than regression
acceptance evidence. [Build provenance](build-20261013.json), [all initial samples](results-20261013.json)
and [larger native CPU batches](cpu-native-results-20261013.json) are retained.

The initial 16-operation native CPU batches were below 1 ms, making transport and
scheduling overhead too influential. They remain as pilot data. The native CPU
baseline instead uses 10,000 operations and 20 warmups per process; batches last
about 0.24 s. Interpreter CPU and both JSON cases retain 16 operations/two warmups.
Different batch sizes are explicit; no calibration algorithm or statistical gate is
claimed. One CPU operation sums integers 0 through 9,999 and verifies the result.

| Case | Mode | Median operations/s | Round range | Median startup (s) |
| --- | --- | ---: | ---: | ---: |
| cpu.sum | interpreter | 7.31 | 7.17–7.55 | 2.888 |
| cpu.sum | jit | 41332.73 | 41163.65–41380.11 | 2.392 |
| cpu.sum | aot | 41119.82 | 40891.99–41318.89 | 0.010 |
| json.serialize | interpreter | 4.09 | 3.75–4.17 | 2.873 |
| json.serialize | jit | 246.97 | 243.72–251.55 | 2.425 |
| json.serialize | aot | 263.04 | 262.90–265.73 | 0.012 |
| json.deserialize | interpreter | 4.36 | 4.03–4.40 | 2.938 |
| json.deserialize | jit | 201.42 | 162.10–204.53 | 2.415 |
| json.deserialize | aot | 211.81 | 154.15–214.49 | 0.012 |

Repeat the larger CPU run with the same manifest:

```sh
python3 benchmarks/framework/runner.py --manifest /fresh/path/to/execution/manifest.json \
  --output /fresh/path/to/execution/cpu-native-longer.json --filter cpu.sum \
  --mode jit --mode aot --rounds 5 --warmups 20 --operations 10000 --timeout 300
```

## Candidate host preparation

Preparation selects `console` on macOS ARM64 and `windows-console` on Windows x64,
including the native executable suffix. Use a matching optimized bundle and run
Windows preparation from an MSVC x64 developer shell. The active Python interpreter
is used for child tooling. Host-selection tests are not Windows runtime qualification;
that requires preparing and running the candidate on Windows.
