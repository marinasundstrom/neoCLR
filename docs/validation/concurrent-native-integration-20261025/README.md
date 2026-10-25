# Canonical concurrent native source integration — 2026-10-25

The collector components, mutation gate and admitted-provider generator now live
under [runtime/native/gc/concurrent](../../../runtime/native/gc/concurrent/README.md).
The standard HTTP builder uses those sources directly and records their hashes.
Historical benchmark entry points and C/header paths forward to the canonical
implementation; fault-injection tests read canonical source explicitly.

This is source integration, not concurrent-default promotion or a public GC option.
The synchronous collector remains the ordinary host default. Native-kit/package
exposure and Windows concurrent qualification remain separate release gates.
The broader candidate still has profiling requirements, collector-metadata allocation
failure limitations and owner/nonmoving restrictions documented in its README.

## Selected policy

`owned-nonmoving-v1` records the already measured configuration: 1024/16384
allocation/publication budgets, 50% completion watermark, sparse/exact allocation
indexes, inline base lookup, four store ranges and no store cache. It names the
qualified compiler stack policy. The standard builder validates this manifest
against its native command before linking. Missing settings, duplicate definitions,
conflicting values, undefinitions and shared-worker/other experiment overrides are
rejected. Independent research variants remain separate builds.

These settings consolidate existing evidence; they are not a new tuning result.
The host still owns its workload-specific arena/HTTP limits. The full HTTP harness
continues to require retained managed-toolchain/fixture inputs and recipes. Only
runtime source generation and compilation are claimed independent of benchmarks.

## Validation

- Every moved C/header implementation has the same bytes as its prior benchmark
  implementation. Both generated collector/provider sources are byte-identical
  before/after migration and in a copied-runtime-only tree.
- `validate-runtime-only.py` copies runtime C/header/Python/JSON sources to a fresh
  directory, generates both C units and compiles collector, provider and gate with
  strict warnings. It does not copy benchmark sources or retained executables.
- Live collector validation passes 38 commands, including optimized/sanitized
  tests and the deliberate missing-deletion-log failure control.
- Registered-mutator worker-epoch validation passes 25 commands and its existing
  negative controls. This preserves that separate experiment, not its admission
  into the selected owner-only policy.
- Seed validation passes six commands; scratch tests pass optimized/sanitized
  reuse/growth/cap/spill and rejection checks. These preserve research variants
  without enabling them in the selected configuration.
- Policy tests accept the selected settings and reject eight incompatible controls.
- Managed sync/live HTTP objects match the previous qualified objects byte-for-byte.
- HTTP smoke qualification passes eight plain/JSON, sync/concurrent, JIT/AOT
  cases: 640 validated responses including warmup. See `http-smoke.json.gz`.

Reports below retain exact commands, output and source hashes. Source migration and
object equivalence preserve earlier scoped performance evidence; these checks are
not fresh throughput measurements. Generator projection over the synchronous
scanner remains a structural maintenance constraint. Replacing text transforms
with explicit shared hooks is future work, not silently achieved by relocating files.

Historical archives retain their original source paths and commit identities.
For current qualification, use the canonical source hashes and selected-policy
manifest; older recipe input lists may name forwarding paths rather than every
transitive implementation dependency. This integration record and the current
standard HTTP report include the complete canonical input set.

The archived build and runtime-only reports precede the final policy-validator
check rejecting split `-D`/`-U` arguments. That validation-only change does not alter
generated C or compiled artifacts. `canonical-inputs.json` records the final sources;
`policy-test.txt` records all eight controls against the final validator. Original
report hashes are preserved rather than rewritten to match later source edits.
