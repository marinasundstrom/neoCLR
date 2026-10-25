# Scalar-store follow-up from the execution matrix — 2026-10-25

Status: opt-in typed-store optimization passes native and expanded AOT/JIT
primitive-consumer validation and a matched six-sample AOT comparison. Keep it
experimental pending HTTP qualification and wider default acceptance. Defaults are unchanged.

## Measured question

The six-sample native matrix shows reuse-buffer invocation CPU of 18.04/111.19 ms
for sync/concurrent AOT and 19.20/111.78 ms for sync/concurrent JIT. The sample makes
only 14 managed allocations but performs 4,194,304 element updates. This points to
mutation/runtime work, not allocation volume, as a useful next target. It does not
by itself attribute the entire difference to one service.

## Current source evidence

- `runtime/codegen/src/selection.rs::scalar_array_element` identifies pointer-free
  single-lane elements. Byte uses a separate compact representation.
- `runtime/native/memory/text-arena.c::allocate_scalars` already allocates primitive,
  nontraced storage; numeric bits must not retain pointer-shaped objects. It uses
  a 24-byte shape, eight-byte element lanes and per-element initialization bytes.
  A new traced record category or new heap representation is unnecessary here.
- `runtime/codegen/src/values.rs`, the scalar `StoreArrayElement` branch, checks
  null and index bounds, writes the typed element, then writes its initialization
  marker. Reference-containing elements instead call the grouped record helper.
- `runtime/codegen/src/mutation_store.rs::lower_impl` recognizes compiler stack
  stores but otherwise lowers both scalar element and marker writes separately to
  `experiment_gc_store`. Element type information has been lost at that stage.
- `benchmarks/execution/concurrent-live/store.h::experiment_gc_store` validates
  owner/context, resolves the containing allocation and validates its extent for
  each call. Active snapshot writes can additionally enter the mutation gate.
- `benchmarks/execution/concurrent-live/scan.h::shadow_scan` reads payload only for
  intern, object, record and string allocations. Primitive allocation payload is
  not scanned. This is a prerequisite for any no-gate primitive-store path, not
  permission to omit ownership, liveness, extent or initialization checks.

## .NET comparison and proposed change

The pinned [.NET 10 JIT importer](https://github.com/dotnet/runtime/blob/v10.0.0/src/coreclr/jit/importer.cpp#L6734-L6834)
keeps typed array stores distinct: reference stores can require the array-store
helper, while primitive stores form typed indexed stores. The relevant lesson is
to preserve type information until selecting the required runtime work. This does
not establish that neoCLR can copy CoreCLR's exact barriers or ownership rules.

Start with an opt-in grouped scalar helper selected directly by typed array
lowering. Validate owned allocation identity, primitive kind, scalar layout,
index and lane width once, then update the element and initialization marker.
Do not add global address caches, allow arbitrary raw destinations, reinterpret
reference arrays, or infer reference-freedom solely from machine integer width.
Keep ordinary generic lowering for headers, unknown destinations, record arrays,
reference slots and foreign/provider mutation. Preserve value normalization,
narrow lanes, floating-point bits and reserved-array initialization semantics.

The expected benefit is one admission/lookup instead of two per scalar update,
with no marker synchronization for payload the collector never reads. Costs are
another internal trusted helper contract and compiler/runtime coupling; passing a
stack snapshot may add instructions. A direct compiler proof could eventually
remove more work, but has a larger proof surface and should follow measurement.

## Required evidence before adoption

1. Native positive/negative tests: every scalar width, floating bits, default and
   reserved arrays, bounds, malformed shape, reference-kind rejection, foreign
   ownership, reclaimed storage and pointer-shaped integers not retaining objects.
2. Mutations before/during/after real concurrent epochs; marker accounting stays
   valid and reference-array controls still require the gate. Include sanitized
   tests and deliberately broken controls where applicable.
3. Compiler symbol/lowering checks for enabled and disabled modes, plus executable
   primitive consumers in both JIT and AOT. Disabled output must remain unchanged.
4. Interleaved reuse-buffer and all six standalone samples, then plain/JSON HTTP
   under the now-qualified close/keep-alive/pipeline fixtures. Record CPU, checked
   work, allocations, memory and epoch effects; profile separately from timing.
5. Retain the candidate only if whole-workload evidence justifies it. A microkernel
   gain alone is not concurrent-default readiness or an HTTP throughput claim.

## Opt-in implementation

`NEOCLR_LIVE_SCALAR_ARRAY_STORES` selects a typed grouped helper in the compiler
and its native host. The helper receives one compiler-owned, normalized and
zero-padded eight-byte lane, validates the scalar allocation and updates that lane
plus its initialization byte. Header and byte-array paths are unchanged. Native
contract tests and AOT/JIT primitive-array qualification have selectable coverage;
initial validation has now run as recorded below. Do not enable this flag in
benchmark baselines or treat it as validated performance work.

## Initial qualification (2026-10-25)

- `target/gc-scalar-store-native-v1/results.json`: sparse/allocation index with
  range cache disabled; 38 commands pass, including optimized and UB/bounds
  sanitized 100-epoch scalar controls, existing 12,000-block liveness comparisons
  and a deletion-log negative control that aborts. This does not prove all thread
  interleavings or generated scalar-type normalization.
- `target/gc-scalar-store-primitive-v1/results.json`: 14 commands pass. Int32/Int64
  consumers and unchanged Byte controls pass paced/unpaced AOT (optimized and
  sanitized) plus JIT, 120 pressure/initialization/bounds cycles per consumer run.
- `target/gc-scalar-store-primitive-disabled-v1/results.json`: same checks pass
  without the feature. Disabled emitted object is byte-identical to the preserved
  baseline runner output (`cmp` exit 0).
- Baseline runner preserved before rebuild at
  `target/gc-scalar-store-compiler/baseline-runner`, SHA256
  `2ad0fc3cb812863274ffb1b9ef79052a8bb067bc2d46ce9d76ced28512713674`.
  The ordinary release runner now contains this draft compiler; prior benchmark
  recipes referring to its path need the preserved baseline for reproduction.

At this stage broader generated-type and performance evidence was still pending;
the following section records the subsequent qualification.

## Expanded qualification and measured benefit

`scalar-store-evidence/widths.json.gz` and `widths-disabled.json.gz` pass 14 commands
each, covering paced/unpaced JIT plus optimized/sanitized AOT. Generated stores now
cover SByte, Int16, UInt16, UInt32, UInt64, Boolean, Single (rounding and NaN), Double
(including NaN), IntPtr and UIntPtr, in addition to Int32/Int64 and unchanged Byte
controls. The fixture retains its width arrays and uses a larger test arena;
pressure/exhaustion modes explicitly fill that arena. Failed fixture construction
attempts are preserved in `fixture-development.json.gz`: unsupported branch/null/
reset spellings and native-width equality were corrected in the fixture without
adding unrelated compiler features. Native widths are compared after conversion
to 64-bit integers. Signed-zero bit preservation is tested at the native helper;
this is not a separate generated signed-zero observation test.

`scalar-store-evidence/native.json.gz` preserves the 38-command native suite and
negative deletion-log control. Every source-generated control object for the six
standard samples is byte-identical to the retained pre-change object; hashes are
in `disabled-objects.json.gz`. No Raven compiler, CLI bridge encoding or public
API semantics change; this is opt-in neoCLR native backend/helper coupling.

The compiler environment and host C definition both use
`NEOCLR_LIVE_SCALAR_ARRAY_STORES`. The helper is trusted private compiler plumbing,
not an arbitrary-pointer API. It does not collect. Generic store counters do not
include its updates; do not interpret their reduction as fewer application writes.
New scalar updates perform exact allocation/shape admission once, retaining the
existing primitive nontraced representation and owner-thread contract.

### Matched AOT comparison

Seven alternating control/candidate rounds across all six unchanged samples, first
round discarded as warmup: 84 invocations pass output and epoch checks, with equal
allocation counts/bytes. Both variants use the same rebuilt compiler/native host
sources, index-only store policy, 1,024 allocation/16,384 publication budgets and
50% finish watermark. Invocation CPU includes collector workers and cleanup;
compilation/startup is excluded. No overlapping build/profile workloads were run.

| Sample | Control CPU ms | Scalar CPU ms |
| --- | ---: | ---: |
| compute | 66.244 | 66.168 |
| reuse-buffer | 114.495 | 56.677 |
| steady | 158.605 | 158.377 |
| bursts | 84.242 | 84.212 |
| retained-window | 342.955 | 342.158 |
| mixed-phases | 160.583 | 160.214 |

Reuse-buffer falls from 114.495 to 56.677 ms CPU (50.5%) and from 114.482 to
56.667 ms elapsed. Its 14 allocations, 555 allocated bytes and 1,016-byte peak
arena are unchanged. Other median CPU changes are below 0.3%, not evidence of a
material improvement. Retained-window peak arena varies with concurrent progress.
This is a concurrent-against-itself gain; earlier sync reuse-buffer results were
still faster and were not rerun as a fresh cross-collector cohort here.

Reproduce with `compare-scalar-stores.py --build target/gc-patterns-standard-build
--runner <qualified-runner> --output <fresh-directory> --rounds 7`. The build report
and six-sample source revision must match the retained AOT recipes. Raw commands,
trial outputs, counters, hashes and medians are archived in `cost.json.gz`.

Next: execute the full JIT workload comparison and fresh plain/JSON HTTP controls
before considering wider enablement. This slice establishes neither HTTP throughput
nor pause improvements, and does not qualify concurrent GC as the default.

## Full JIT workload follow-up

All 48 invocations pass exact guest output, exit status, epoch invariants and
allocation-count/byte equality with the AOT cohort. Four alternating rounds,
first discarded as warmup, leave three observations per variant/sample. Rebuilt
JIT hosts reuse the qualified AOT native source/flags; JIT compilation, selection
and metadata loading are reported separately from invocation CPU. Both variants
use the same compiler and sample assemblies. Raw evidence: `scalar-store-evidence/jit.json.gz`.

| Sample | Control JIT CPU ms | Scalar JIT CPU ms |
| --- | ---: | ---: |
| compute | 64.544 | 66.104 |
| reuse-buffer | 114.116 | 53.908 |
| steady | 259.717 | 259.223 |
| bursts | 134.940 | 134.201 |
| retained-window | 514.079 | 509.128 |
| mixed-phases | 259.334 | 259.068 |

Reuse-buffer improves 52.8% (114.116 to 53.908 ms CPU), with unchanged allocation
work. Other effects are small and mixed: compute is 2.4% slower, while other
median changes are 0.1–1.0% faster. Do not label them general improvements or assume
noise has been established. This short cohort supports the large store-heavy gain,
not a universal workload win. No new sync-versus-concurrent comparison is claimed.

Reproduce with `compare-scalar-jit.py --build target/gc-patterns-standard-build
--aot target/gc-scalar-store-cost-v1 --runner <qualified-runner>
--output <fresh-directory>`. HTTP qualification and pause effects remain pending;
concurrent-default acceptance is still open.

## HTTP qualification follow-up

[Fresh paired HTTP hosts](scalar-store-http.md) pass 36 cells/2,880 responses across
plain/JSON, close/keep-alive/pipeline and six AOT/JIT variants, with 598 concurrent
epochs and equal control/candidate allocation work. Two additional JIT cases verify
ambient feature flags cannot leak into controls. Repeated HTTP performance and
pause measurements remain open; the qualification result is not a throughput gain.
