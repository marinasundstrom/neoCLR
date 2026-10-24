# Consolidated runtime investigation — 2026-10-24

Read the [investigation and ordered workstreams](../../../docs/design/runtime-bottleneck-investigation.md)
for findings, interpretation, design comparison and remaining gaps. This directory
preserves a current baseline and reconciles earlier evidence; it introduces no
production runtime optimization.

## Current evidence

- `startup/`: 18 checked task/collection interpreter/JIT/AOT executions, three per
  case/mode, plus two separate startup stack captures. Release binaries built from
  `db0290fb5`; retained qualified Raven assemblies are hashed. Native hosts are
  rebuilt from current source. External 1 MiB paced heaps match the ordinary-mode
  fixture, not the owned HTTP configuration.
- `peers/comparison.json.gz`: 14 checked trials, same ten-byte connection-close
  response, 32 client connections, two-second warmup and ten-second measurement.
  Forward/reverse order, one/four runtime settings, current owned native runtime.
  All builds finish before measurement. Go 1.26.3, Node 22.21.0 and installed .NET
  versions/commands/hashes are in the report. CPU includes server lifecycle and
  subtracts the separate client; these are not process-affinity comparisons.
- `worker-profile/`, `worker-analysis/`, `worker-build.json.gz`: separate checked
  four-owner profile of the same peer binary. Every worker self sample is accounted
  for. The analyzer recompiles the native object byte-identically before assigning
  managed names. Coordinator waiting is separate. These are wall samples, not CPU
  percentages; profile throughput is excluded from capacity results.
- `gc-timing/profile.json.gz`: separate one-owner plaintext diagnostic run, opt-in
  GC timers, live-payload/arena peaks and pause buckets. Collection elapsed time
  cannot be subtracted from the uninstrumented capacity cohort's CPU time.
- `plain-counts/`, `json-counts/`: eight checked server invocations, four/twelve
  requests in JIT/AOT for each handler (64 responses). Current owned host and native
  collector. Differences divided by eight remove fixed invocation work; collection
  schedules can still differ. `json-verification.json` confirms the retained JSON
  object also recompiles byte-identically with the current compiler.
- `summary.json`: generated startup medians/ranges, peer rates/CPU, disjoint worker
  leaf groups, GC timing and per-additional-response lookup counts. Each input is
  hashed. `provenance.json` identifies the machine, source revision and archives.
- `interpreter-*`: attempted refresh of the older CPU/JSON worker with the current
  interpreter and unchanged qualified package inputs. It fails before READY with
  `runtime representation is reserved for canonical primitive types`; zero timing
  samples are used. The first broad manifest also detects intentionally changed
  repository native sources; preparation then scopes inputs to the unchanged
  package/application files actually retained for interpreter use. Both preparation
  history and the guest admission failure are explicit. This requires a coherent
  current worker/package rebuild, not loosening type validation.
- `prior-tracing-failure-*.json.gz`: the preceding image-specific precise-object
  experiment fails during AOT startup. It is not included in any successful cohort
  here, and neither its prototype nor its throughput is promoted by this report.

Five new aggregation tests cover disjoint sample accounting, invalid sample counts,
failed/incomplete startup cohorts, the CPU response denominator and lookup increments.
Ten existing JIT symbolization/summary tests and seven worker-stack analysis tests
also pass. A full runtime suite and website build are not relevant to this
measurement/documentation-only change; no managed API or bridge behavior changes.

## Reproduce

These scripts reuse qualified application/library inputs and native recipes from
prior fixtures. They are not a clean-checkout installer. Reconstruct those fixtures
from their linked reports before running; do not silently substitute another bundle.
Use fresh output paths and run sequentially without concurrent builds or benchmarks.
The exact recorded commands and hashes take precedence over examples below.

```sh
export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
export SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.2.sdk
cargo build --release -p neoclr --bin neoclr -p neoclr-codegen --bin neoclr-codegen
python3 benchmarks/execution/bottleneck-review/startup.py --output target/review-new/startup
python3 benchmarks/http-server/primitive-comparison/compare.py \
  --neoclr-build target/primitive-comparison-build/build.json \
  --neoclr-object target/root-map/candidate.o --refresh-native-runtime --owned-heap \
  --seconds 10 --output target/review-new/peers
python3 benchmarks/execution/bottleneck-review/diagnostics.py --directory target/review-new
python3 benchmarks/execution/bottleneck-review/summarize.py \
  --directory target/review-new --output target/review-new/summary.json
python3 -m unittest discover -s benchmarks/execution/bottleneck-review -p 'test_*.py'
```

`interpreter.py --directory target/review-new` reproduces the separately retained
admission failure with the old package; it is not part of the successful baseline.
The JSON byte-identity replay command is retained in `json-verification.json`.
To reanalyze archived data, decompress the `.gz` files into a fresh directory while
preserving their relative paths, then use summarize.py. Input-location strings differ
from the original report; numeric results must remain identical. Prior JIT placement,
owner-index, GC policy and interpreter batch results remain separate historical
cohorts, never pooled with these current observations.
