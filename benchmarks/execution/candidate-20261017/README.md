# Local release-candidate runtime measurements

Date: 2026-10-17. These are development measurements, not a release-readiness
approval or published product performance guarantee. The editor-corrected macOS
candidate is `0.0.0-native-candidate.20261017.2`, Raven `7bfbce21f`, with the
previously qualified optimized runtime/codegen binaries unchanged.
[Package identity](../../../docs/validation/release-candidate-20261017/editor-fixed-package-identity.json)
and [comparison data](comparison.json) identify both cohorts. Preview 15's local
archive matches its [recorded published digest](preview15-identity.json).

All 45 candidate and 45 adapted Preview 15 processes pass workload checks and
before/after input-hash verification. Each case/mode has five fresh-process rounds.
CPU interpreter and JSON use two warmups and 16 operations; native CPU uses 20
warmups and 10,000 operations. One CPU operation sums integers 0 through 9,999,
requiring 49,995,000. The JSON cases are DOM serialization/deserialization of the
same nested Unicode document, not typed-object or HTTP measurements.

## Preview 15 syntax compatibility

The unmodified modern worker fails its CPU checksum on Preview 15 before any
sample is recorded; [retain that failure](preview15-unadapted-failure.json).
Raven changed `0..10000` from inclusive to half-open after that release.
The baseline changes exactly that loop to `0..<10000`, reproducing the historical
worker before commit `b1f3f492f`; the source is asserted equal to that Git version.
Iteration count, checksum, framework worker and measurement settings are unchanged.
[Compatibility record](preview15-compatibility.json) contains the build command.
This is an explicit syntax adaptation, not an unchanged-source comparison.

## Throughput

Values are median operations/second (minimum–maximum). Percentages compare
medians. Cohorts ran sequentially on the same interactive Apple M1 host; they were
not interleaved or thermally isolated. Small differences are not causal regression
conclusions. [Environment before](environment-before.json) and
[after](environment-after.json) retain power/load observations.

| Case | Mode | Candidate | Preview 15 | Median change |
| --- | --- | ---: | ---: | ---: |
| cpu.sum | interpreter | 4.21 (4.10–4.32) | 6.45 (6.19–6.54) | -34.8% |
| cpu.sum | jit | 39110.61 (37749.73–39221.56) | 40663.84 (40314.44–40744.61) | -3.8% |
| cpu.sum | aot | 39110.49 (37849.73–39292.09) | 40713.84 (38256.91–40800.02) | -3.9% |
| json.serialize | interpreter | 3.81 (3.36–3.83) | 3.76 (3.35–3.78) | +1.3% |
| json.serialize | jit | 433.81 (415.22–439.57) | 402.14 (385.53–405.02) | +7.9% |
| json.serialize | aot | 450.46 (440.27–457.89) | 421.70 (403.79–427.18) | +6.8% |
| json.deserialize | interpreter | 4.13 (4.04–4.23) | 4.15 (3.83–4.17) | -0.7% |
| json.deserialize | jit | 336.98 (312.47–341.14) | 340.35 (327.12–342.66) | -1.0% |
| json.deserialize | aot | 357.68 (327.56–359.13) | 360.73 (348.14–363.98) | -0.8% |

Compilation is excluded. Startup (including metadata loading, JIT preparation and
JSON setup) is recorded separately in every raw report and comparison entry.
Batch wall time includes callback dispatch, result validation, console/pipe
markers and reader scheduling. No allocation, peak-RSS or isolated-instruction
measurement is claimed.

## Release concern

CPU interpreter throughput is about **35% lower** than Preview 15; the observed
round ranges do not overlap. Candidate range lowering includes an additional
per-iteration overflow guard. The [candidate](candidate-cpu-diagnostic.neoil) and
[Preview 15](preview15-cpu-diagnostic.neoil) disassembly excerpts retain that
observation. This is a concrete investigation lead, not an isolated causal proof.
The general guard protects valid boundary/step cases and must not simply be removed.

Next bounded investigation: specialize provably safe constant unit-step half-open
iteration while preserving general overflow handling, then validate endpoint,
continue and break behavior and rerun the affected workloads. No optimization is
implemented or claimed by this record. A new compiler/library candidate will need
fresh affected qualification. Windows, HTTP benchmarks, source gates and remote
candidate pinning also remain open. Published Preview 14 measurements are unchanged.

## Raw results

- Candidate: [CPU interpreter](candidate-cpu-interpreter.json), [CPU JIT/AOT](candidate-cpu-native.json), [JSON serialize](candidate-json-serialize.json), [JSON deserialize](candidate-json-deserialize.json).
- Preview 15: [CPU interpreter](preview15-cpu-interpreter.json), [CPU JIT/AOT](preview15-cpu-native.json), [JSON serialize](preview15-json-serialize.json), [JSON deserialize](preview15-json-deserialize.json).

Each report preserves exact inputs, commands, configuration, all samples and
median/minimum/maximum summaries. The initial failed unadapted run contributes
no timing samples to the comparison.
