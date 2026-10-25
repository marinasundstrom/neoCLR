# Typed scalar-store integration — 2026-10-25

Typed primitive-array stores are now the normal lowering within the private
concurrent candidate. Ordinary hosting still passes `live_stores=false` and uses
synchronous GC. This is integration of a qualified runtime optimization, not
promotion of concurrent GC or a new public API.

The compiler retains pointer-free element information through lowering and emits
one checked scalar helper call for the value and initialization marker. Reference,
record, byte-array and unknown stores keep their existing paths. The host always
provides the typed helper; no matching preprocessor flag is required. Its existing
ownership, shape, bounds and nontraced-storage checks are unchanged. See the
[original design and measurements](scalar-store-follow-up.md) for the reasoning,
tradeoffs and scoped workload benefit.

Qualification builds can select the former generic lowering with
`NEOCLR_LIVE_GENERIC_SCALAR_ARRAY_STORES=1`; production binaries do not read this
variable. Primitive validation and HTTP preparation expose
`--generic-scalar-stores`. Their former `--scalar-array-stores` option remains an
alias for the normal typed path. Matched scalar comparison tools select this
explicit generic control and clear ambient selection. Old archived experiments
retain their original flags and hashes; reproduce them with their pinned runner,
not by silently applying the changed selection to current sources.

## Validation

- Release codegen test runner builds successfully.
- Native validation passes 38 commands, including optimized and UB/bounds-sanitized
  scalar lifetime/bit-pattern/ownership checks, differential marking and deliberate
  missing-deletion-log failure controls.
- Expanded primitive-array consumers pass 14 commands per path: default typed and
  explicit generic, paced/unpaced, optimized/sanitized AOT and JIT execution.
- Both emitted primitive objects are byte-identical to their earlier qualified
  typed/generic objects. Fresh HTTP sync and typed-live objects are also
  byte-identical to the preceding qualified versions. This selection integration
  does not establish an additional performance improvement; retain the prior
  standalone gains and small/mixed HTTP results.

Evidence is archived as `scalar-store-evidence/integration-*.json.gz`, including
commands, compiler/source hashes and object equivalence. Fresh HTTP qualification
passes 24 cells and 1,920 responses: plain/JSON, close/keep-alive/pipeline,
sync/default-concurrent and AOT/JIT. These are correctness checks, not new throughput
measurements. No package or additional-host qualification is implied.

## Existing stack-transfer test issue

A supplementary `mutation_store::tests::` run aborts in
`unambiguous_stack_transfers_elide_only_proved_stores` on an unexpected JIT return
value. The isolated test is intermittent in both binaries: three repetitions
produce two passes and one abort in the preserved pre-integration opt-in runner,
and two passes and one abort in the current runner. This is not a clean full-suite
pass. The retained comparison includes stdout/stderr; cause and correction remain
open and must be resolved or explicitly assessed before release qualification.
The focused scalar/native/HTTP evidence does not certify this separate stack path.

### Follow-up

The [code-publication investigation](../../../docs/validation/jit-code-publication-20261025/README.md)
traces the failure to missing instruction-cache maintenance on macOS ARM64 and
corrects it in the shared JIT memory provider. The original failure evidence above
is retained; the later record contains the passing tests and negative controls.
