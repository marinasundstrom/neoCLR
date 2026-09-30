# JSON versus native assembly benchmark

Release-mode experiment, 2026-09-30, on Apple M1 / macOS arm64, rustc 1.95.0,
using runtime `116be40e` and the directly assembled schema-3 artifacts. This measures
existing Raven collection System (417 types / 4,090 functions) and the FloatingMath
application, not synthetic instruction-only metadata. The author's benchmark request
follows the direct neoil assembly producer; no runtime implementation was changed to
improve these measurements.

[Raw samples, artifact/executable hashes and environment](json-vs-assembly-benchmark.json)
are retained. Source and complete metadata values match across paths; the sample
produces nineteen `0` lines and then `-1` in every measured CLI run.

## Results

Median times, including module destruction in decoder rows:

| Measurement | Current pretty JSON | Compact JSON, same loader | Native assembly |
| --- | ---: | ---: | ---: |
| System bytes | 15,017,185 | 7,104,628 | 5,542,303 |
| System decode | 65.97 ms | 62.78 ms | 52.80 ms |
| FloatingMath bytes | 535,539 | 301,495 | 238,988 |
| FloatingMath decode | 1.928 ms | 1.810 ms | 1.850 ms |
| CLI start/load/run, application + System | 3,454.8 ms | not measured | 3,440.3 ms |

System assembly is about **63% smaller than pretty JSON**, 22% smaller than compact
JSON, and has about **20% lower decode time than the current pretty-JSON path** (16% lower
than compact JSON). The smaller application is about 4% faster than pretty JSON and
about 2% slower than compact JSON in this run. There is no uniform decode-speed claim.

The full CLI difference is about 14 ms (0.4%) between medians, within substantial sample
variation: JSON spans 3,310–3,676 ms and assembly 3,303–3,635 ms. The median paired
JSON-minus-assembly difference actually favors JSON by about 25 ms. **This run does
not demonstrate a meaningful complete-startup/run improvement.** Common phases show
link/prepare at 1,334 ms, explicit verification at 699 ms and prepared execution at
2.15 ms. Linking/admission work is a much larger cost than decoding; these separately
measured phases must not be summed as a precise CLI breakdown.

The direct-typed JSON diagnostic takes **28.42 ms for System** and **0.766 ms for the
application**, faster than either current production decoding path. Removing the
intermediate JSON value tree is therefore a concrete optimization candidate. It does
not erase native assemblies' size benefit or establish that JSON should be the final
metadata format. The native decoder also performs a strict prevalidation pass; future
profiling should attribute its costs before replacing codecs or changing validation.
No production decoder or format policy changed for this benchmark.

The author accepts smaller files as the demonstrated improvement and defers runtime
optimization to future work. These measurements remain the baseline for that work;
metadata/compiler integration continues without requiring a startup-speed improvement.

## Method

The in-memory release example reads files before timing and compares complete decoded
modules. It rotates the first of four decoding paths across nine samples, with two
iterations per sample for System and ten for the application. Timings include allocation,
deserialization and destruction of decoded modules. Native assembly decoding includes
schema/envelope checks and the strict CBOR guard. Warm-up and equality checks precede
timing. Inputs remain resident; no file/process costs are in those decoder measurements.

The JSON baseline mirrors the current loader in `src/lib.rs::decode_module`: parse a
serde_json::Value, check format 5, then convert to Module. Compact JSON is generated
from the same typed module to control for whitespace. A direct-to-Module JSON decode is
also measured as an **unshipped diagnostic**, not described as current runtime behavior.
It shows how much of the gap could come from removing the intermediate JSON value tree.
Any future JSON-loader change still needs compatibility tests for malformed inputs,
duplicate fields, version checks and diagnostics; this benchmark does not implement it.

Common phases start from decoded modules: one link/prepare operation (including dropping the resulting snapshot), explicit typed
verification of a prepared program, and execution of a prepared program with captured
output. Each has a warm-up and nine single-iteration samples. Prepared execution creates
fresh execution state; it is not isolated arithmetic throughput.

Fresh-process runs alternate which format runs first over nine pairs, after one warm-up
per format. They run the actual CLI with application and matching System both JSON or
both native assemblies. Timings include launch, warm-cache file reads, System admission,
module-set validation/linking, preparation, execution, captured output and shutdown.
They do not include compilation or assembly. CLI load-set validation can perform more
linking work than the single common-phase measurement, so phase totals must not be
presented as an exact decomposition of CLI wall time.

This is one local run with nine samples, not a confidence interval, broad workload claim
or cold-filesystem benchmark. No peak memory measurement is included. Lower parse costs
do not imply the same percentage improvement in complete startup or execution.

## Reproduce

Use artifacts from the [direct assembler experiment](direct-assembly.md):

```sh
cargo build --release --bin neoclr --example native_assembly_benchmark
python3 scripts/benchmark-native-assemblies.py \\
  --runtime target/release/neoclr \\
  --phase-benchmark target/release/examples/native_assembly_benchmark \\
  --json target/extended-cli-metadata/raven-library-profile3 \\
  --assemblies target/extended-cli-metadata/direct-native-assembler \\
  --output target/extended-cli-metadata/json-vs-assembly-benchmark.json
```

The output must not exist. The recorded macOS build uses the command-local SDK 26.2
override documented in earlier experiments. No global SDK settings changed. The
benchmark checks correctness before timing; each process run must exit successfully,
produce the exact expected output and emit no stderr.
