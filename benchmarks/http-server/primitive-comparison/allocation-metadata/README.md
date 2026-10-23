# Allocation bookkeeping and execution metadata — 2026-10-23

The author asked to improve allocation bookkeeping/metadata processing, then
explicitly directed looking to .NET for metadata guidance. Two allocation experiments
were measured first: combining block-header checks without short-circuit branches,
and stopping first-fit selection after finding a hole while still validating the
tail. Neither established a benefit on this machine; both were reverted. The full
heap-integrity contract, first-fit placement and failure atomicity remain unchanged.
Rejected trials are retained separately and must not be reported as improvements.

## CLR guidance and the neoCLR boundary

CLR distinguishes loaded execution structures (MethodTable, EEClass, MethodDesc,
FieldDesc) from reflection metadata. Its design goals include fast executing-code
access and GC/stack-walker access without allocations or locks. Cached lookups and
specialized paths serve different consumers. This supports separating diagnostic
richness from the data needed by executing code; it does not justify caching arbitrary
mutable neoCLR descriptor pointers. [CLR type-system design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/type-system.md).

CLR's compiler supplies per-method metadata for stack walking and favors cheap
method execution over repeatedly doing stack-walk work. neoCLR currently publishes
explicit frames/slots, while retaining diagnostic JSON at each point. Its runtime
uses the slot/lane arrays for GC; JSON is checked for length and the spill marker,
not interpreted as a tracing program. [CLR stack-walking design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/stackwalking.md).

CLR's allocator uses thread allocation contexts and replenishes them when needed,
keeping the common allocation path local to a thread. neoCLR instead uses a bounded,
nonmoving arena and first-fit block walks with full corruption detection. Copying a
bump allocator into that API would change hole reuse and trailing-corruption checks.
A future faster allocator needs explicit ownership, maintained free-space metadata,
collector integration and a checked/diagnostic boundary; silently skipping the
existing checks is not this change. [CLR allocator design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/garbage-collection.md).

## Implemented metadata slice

For native GC emission only, the neoCLR backend emits a small point payload with
schema `neoclr-gc-point-v1` and the original ordered `requiredSpillLanes`. Full
`roots`, type-layout, consumed-value and stack-description JSON remains available
to diagnostic probes/inspection. Actual emitted spills, slot descriptors, stack-lane
counts, publication sites, worker exclusivity flags and native callbacks are unchanged.
The runtime still checks current plan bytes on every publication. No metadata cache,
new public API, trusted pointer lifetime or root-validation bypass is introduced.

This is an intermediate hot/cold metadata separation, not CLR-format GCInfo or a
binary root-map implementation. Native-GC frame inspection now sees the compact
schema; tools requiring complete plans must use inspection/diagnostic mode. Native
metadata should eventually own versioned compact maps and their image lifetime,
with diagnostics separate. Schema admission, unload/address reuse, mutable adapters,
cross-worker GC and malformed-map tests are prerequisites for that next step.

Raven's Runtime Contract configuration and CLI semantic emission remain unchanged.
The existing matching Raven bundle is 1128b98cf on codex/neoclr; the neoCLR backend
owns this lowering. See the paired bridge notes in neoCLR and Raven. No compiler
branch merge or public API snapshot change is involved.

## Measurement

All variants use the primary ten-byte plain response, connection close, 32 clients,
one/four isolated owners, current root validation and 256/1,024 paced collection.
Builds finish before serial forward/reverse trials (two seconds warmup, six seconds
measured). CPU per response covers startup/warmup/measurement/drain/shutdown. These
short closed-loop observations do not qualify a sustained production rate.

`compact.py` recompiles the same retained source assembly with the changed backend,
then links old/new objects against identical current native sources. It records
explicit input differences and artifact identities. Allocation comparisons reuse
the original application object and change only native-gc.c.

## Compact point results

| Variant | Owners | Requests/s (two trials) | CPU µs/response | p99 ms |
| --- | --- | --- | --- | --- |
| baseline | 1 | 683.2, 677.3 | 1,469.7, 1,477.9 | 49.27, 62.02 |
| baseline | 4 | 2,564.2, 2,498.3 | 1,552.8, 1,570.4 | 17.73, 25.54 |
| candidate | 1 | 750.7, 744.3 | 1,337.1, 1,340.5 | 45.60, 53.39 |
| candidate | 4 | 2,797.7, 2,745.2 | 1,412.7, 1,425.6 | 19.18, 23.97 |

Mean throughput rises about 10% and CPU per response falls about 9% with both
owner counts. p99 remains variable; these results do not establish a universal
latency improvement. Compare paired variants here, not absolute throughput from
another run on this shared desktop. The application object shrinks from 14,053,992
to 11,628,216 bytes (17.3%); this is object-file size, not runtime heap/RSS savings.

The spill-map contract test and all five native root-probe tests pass in optimized
Rust builds. A sanitized native server built from the new object also passes four
serial exact-response checks with forced collection at every publication and clean
shutdown. The normal benchmark uses paced GC and passes all eight trials. The
initial incremental debug compiler build stalled and was stopped; a nonincremental
optimized build succeeded. That interrupted build is not passing test evidence.

## Reproduction

Use the matching retained application build and bundle documented in the parent
README. Select the macOS 26.2 SDK using DEVELOPER_DIR/SDKROOT on this host. Build
and test before running serial measurements; the output directory must be fresh.

```sh
CARGO_INCREMENTAL=0 cargo build --release -p neoclr-codegen
CARGO_INCREMENTAL=0 cargo test --release -p neoclr-codegen --lib execution_metadata_tests
CARGO_INCREMENTAL=0 cargo test --release -p neoclr-codegen --test root_probes
python3 benchmarks/http-server/primitive-comparison/allocation-metadata/compact.py \
  --build target/primitive-comparison-build/build.json \
  --client target/primitive-comparison-eof/client --output target/plain-compact-metadata
python3 benchmarks/http-server/primitive-comparison/allocation-metadata/forced.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-compact-metadata/candidate.o --output target/plain-compact-forced
```

`rejected-allocation.patch` preserves the rejected native changes and
`rejected-allocation.json` their input hashes/trials. `measure.py` tests the combined
header/selection candidate plus an explicit intermediate source; `compare.py`
tests a current allocator against a pinned revision. Neither rejected candidate
is present in the runtime. Keep their results separate from compact metadata.

Shared-heap compiled workers also pass optimized and UBSan validation: 32 healthy
and 32 fault invocations per build, four concurrent graph participants, 24,544
coordinated graph collections and 448 fault collections. See worker-validation.json;
reproduce with docs/experiments/native-worker-pool/generated.py --parallel using
the matching bundle/libraries and invocation-context-native-validation.json recipe.

Integration identity: this backend change is on neoCLR main; the paired Raven
codex/neoclr documentation commit is 45ef69839. Raven source compiler behavior and
the tested 1128b98cf bundle are unchanged; no change was carried to Raven main.
