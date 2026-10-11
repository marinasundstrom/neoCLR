# JSON node benchmark — initial runner qualification

The first reusable Raven benchmark worker registers separate JSON node serialization
and deserialization cases. The document contains a nested object, arrays, null,
Boolean/numeric values and Unicode text. Setup validates a complete canonical round
trip; serialization checks the full output, and deserialization checks the root
shape and name value. Validation is included in each measured operation.

```sh
python3 benchmarks/json/prepare.py \
  --recipe target/jit-cli-http-final/validation.json \
  --compiler /absolute/path/to/native-enabled/rvnc.dll \
  --output target/benchmark-json-nodes-final \
  --discover --manual-case json.nodes.serialize
python3 benchmarks/framework/runner.py \
  --manifest target/benchmark-json-nodes-final/manifest.json \
  --output target/benchmark-json-nodes-final/results.json \
  --rounds 3 --warmups 2 --operations 8
```

Use fresh output paths. Preparation rebuilds the application and native host using
a qualified macOS ARM64 toolchain recipe. All three modes run the same assembly;
mode selection is explicit. The recorded run uses Raven integration `2535c8706`
and neoCLR `b81adfcf`, with runtime libraries identified by hashes in the reports.
The worker and preparation source hashes identify this development slice.

## Initial manual-registration evidence — 2026-10-11

[Raw samples](results.json), [build commands and hashes](build.json), and
[worker rejection checks](worker-failures.json). All 18 processes passed: two cases,
three modes and three rounds, each with two warmup and eight measured operations
(36 warmup operations and 144 measured operations). Input hashes stayed unchanged.
Five native-worker rejection checks cover unknown cases, malformed counts and
count bounds. Four controller tests cover protocol/failure handling, timeouts,
filtering, accounting and changed-input rejection. The shared HTTP transport also
passes 32 server and 32 client requests using existing AOT artifacts.

**These are external batch measurements**, including correctness checks, callback
invocation, protocol/pipe output and reader scheduling. They are a framework
qualification baseline, not serializer-only timings or an external-framework
comparison. Fixed short batches and three rounds do not establish statistical
stability. Startup includes setup and JIT compilation; AOT builds are excluded.

| Case | Mode | Median operations/s | Median startup (s) |
| --- | --- | ---: | ---: |
| json.nodes.serialize | interpreter | 4.70 | 2.044 |
| json.nodes.serialize | jit | 245.73 | 1.695 |
| json.nodes.serialize | aot | 244.96 | 0.012 |
| json.nodes.deserialize | interpreter | 4.78 | 2.070 |
| json.nodes.deserialize | jit | 191.54 | 1.700 |
| json.nodes.deserialize | aot | 199.75 | 0.011 |

See the [runner contract and limitations](../framework/README.md). Typed objects,
arrays/lists/sequences/maps as mapped object properties, asynchronous workers,
setup/cleanup hooks and internal monotonic operation timing
remain follow-up work. Arrays in this node document do not qualify typed collection
mapping benchmarks. No .NET/Go/JavaScript or Windows performance comparison is claimed.

The framework now supports [attribute-driven function discovery](../framework/README.md#attribute-driven-function-discovery); the recorded JSON workload above uses manual callbacks.

## Current discovery integration

`DeserializeNodes` is now an attributed module function, selected with
`Benchmarks.Json.DeserializeNodes`. This replaces the former manual selector
`json.nodes.deserialize`. `DocumentText()` supplies the immutable input; its call
is included in the measured function. Setup still verifies a complete round trip
before worker READY.

Serialization retains the manual `json.nodes.serialize` callback so it can capture
the node prepared before measurement. Native static storage remains unsupported;
moving parsing into the measured serializer function would change the workload.
A shared setup/fixture contract is still needed for that migration. This is a
current platform gap, not a permanent difference from .NET setup semantics.

Use `--discover --manual-case json.nodes.serialize` during preparation. Discovered
and explicit manual selectors are combined, with empty/duplicate names rejected.
Manual selectors must correspond to a registered case; the worker rejects missing
ones. `--manual-case` requires discovery mode. The default node source requires
`--discover`; its generated registry is necessary to compile the final application.
The earlier timing report above remains historical and is not attributed to the
new source. The discovery run retains separate evidence below.

The [mixed run](discovery-evidence/results.json) passes all 18 processes (144
measured operations plus 36 warmups) in interpreter, JIT and AOT on macOS ARM64.
[Discovery inventory](discovery-evidence/benchmarks.json) and
[generated registration](discovery-evidence/BenchmarkRegistry.rvn) identify the
function and description. All recorded input hashes remained unchanged. These are
external batch measurements with validation/transport costs, not internal timing.
