# Default method inlining (development, 2026-10-28)

Author-directed work: provide inlining controls for JIT/AOT, then prioritize useful
default inlining instead of routine SDK annotations. Backend optimization level is
a separate follow-up; both should be enabled. Commit each validated slice separately.

## Contract and comparison

Use [.NET MethodImplOptions](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.compilerservices.methodimploptions):
`NoInlining = 8`, `AggressiveInlining = 256`, with no `Inlining` member. Unmarked
managed methods are eligible automatically. NoInlining overrides AggressiveInlining
when combined. AggressiveInlining is a hint, never a guarantee or a safety bypass.
The existing InternalCall = 4096 contract remains separate; mixing it with inlining
flags is unsupported. Unknown flags fail loading/emission. The interpreter accepts
these metadata hints and preserves ordinary execution and faults.

The shared Cranelift lowering inlines direct, static, closed Int32/Boolean leaf
expressions: arguments, literals, wrapping add/subtract/multiply and signed comparisons.
The default budget is eight native IL operations including return; the aggressive
budget is 64. These are bounded initial heuristics, not .NET's implementation thresholds.
No recursion, nested calls, branches, locals, allocations, byrefs, checked arithmetic,
virtual calls, or potentially faulting operations are inlined. These remain ordinary
calls, including when marked aggressive. Caller argument evaluation occurs once;
type-initializer checks remain before the original call site. No inline-frame or GC
root reconstruction is needed for this subset. Broader inlining remains a gap.

Scalar and managed lowering share the decision/emission code. Managed eager and lazy
JIT use that lowering; AOT generates the same leaf bodies for macOS ARM64 and Windows
x64. Original method bodies and metadata remain available for reflection/function
binding. Removing the call saves managed ABI/result/stack-frame overhead and exposes
operations to backend optimization. Costs include caller code growth and additional
lowering work; the budget bounds each expansion. No SDK overrides are added merely
because a method is small: such methods already qualify by default.

## Metadata and compiler ownership

Native `impl_flags` uses the established CLR bit values. The experimental metadata
API adds `MethodDefinition.SetImplementationFlags(MethodImplAttributes)` and its
builder facade, accepting zero or the inlining subset. Native reads, imports and CLI
reference projections preserve the bits. Raven's neoCLR backend consumes the exact
configured core MethodImplAttribute as implementation metadata, not a runtime custom
attribute. There is no new Runtime Contract switch. Source core ownership remains
System.Runtime; ordinary Raven/.NET emission is unchanged. Changes are on Raven
`codex/neoclr`, not a merge from/to Raven main.

The CLI reference is a temporary declaration projection; throwing reference bodies
are never evidence of executable behavior. Native metadata already carries these
flags, so its replacement need is limited to retaining them in future metadata/codegen
formats. Method inlining itself belongs to neoCLR's JIT/AOT, not Raven's CLI bridge.

## Validation

- Focused flag policy tests plus JIT execution and relocation checks for both AOT
  targets: default, no-inline, combined flags, and above-default-budget aggressive.
- Scalar and managed lowering each remove/retain calls as required.
- 17 interpreter/native/generic/internal-call tests pass; hint-bearing methods retain
  values and division faults.
- Metadata native/CLI/reference projection, entry flags and rejected options pass.
- Raven native source emission accepts 0/8/256/264, rejects 64/512/4096 on managed
  methods, and does not serialize an executable MethodImpl custom attribute.

The complete backend suite with default inlining and optimization `none` passes
211 tests (36 explicitly opt-in experiments ignored). The four SDK libraries rebuild;
the optional PostgreSQL documentation provider is retained unchanged from the qualified
Preview 18 package so the refreshed reference keeps its namespace coverage.
Raven integration commit: `256019f66`.

The [small scalar loop](default.neoil) executes 2,000,000 calls to a multiply-by-eight
leaf, paired with an otherwise identical [NoInlining control](noinline.neoil).
[Raw none-level timings](none-results.json) contain 11 invocations per compiled case,
checked against wrapping Int32 results. This isolates call overhead; it is not an HTTP,
SDK-wide, architecture-wide or allocation benchmark. Timings were collected on a busy
macOS ARM64 development host, so compilation times and small timing differences are
not stable performance claims. The backend remains at `none` in this slice; the separate
speed slice reuses the same fixtures. No SDK annotation is justified by this comparison:
default policy already selects the leaf.

Reproduce using the ignored `inlining::tests::qualification` test with
`NEOCLR_TEST_OPT_LEVEL=none` and `NEOCLR_INLINE_EVIDENCE` set to a fresh directory.
The output also includes ARM64 and x64 AOT objects for instruction inspection.


## Separate speed-optimization slice

The author explicitly requested enabling both independent layers. Production JIT
(scalar, eager managed and lazy managed) and AOT now set Cranelift `opt_level=speed`
through the shared settings factory. NoInlining only controls method expansion;
it does not disable optimizations inside a method. The private test-only
NEOCLR_TEST_OPT_LEVEL override remains available for isolated comparisons. There is
no new public command-line flag, tiering policy or NoOptimization attribute.

This follows the .NET separation between method-inlining hints and the runtime's
optimizer. Cranelift 0.121.2 already folds multiplication by zero/one, rewrites
powers of two, and selects architecture-dependent instructions. Keep wrapping versus
checked arithmetic distinctions in neoCLR lowering; do not replace arbitrary
multiplies with shift/add chains or assume fewer instructions means fewer cycles.
See the [Cranelift settings contract](https://docs.wasmtime.dev/api/src/cranelift_codegen_meta/shared/settings.rs.html)
and the pinned implementation `cranelift-codegen-0.121.2/src/opts/arithmetic.isle`,
`src/isa/aarch64/lower.isle`, and `src/isa/x64/lower.isle` in the Cargo dependency.
Speed optimization can increase compiler work and sometimes code size; this is an
explicit execution-throughput policy, not a claim that every function gets faster.

[ARM64 inspection](speed-arm64.asm) shows a shifted add for the inlined multiply-by-eight
loop, and [x64 inspection](speed-x64.asm) shows scaled LEA. Both standalone leaf bodies
use shifts. ARM64 execution returns the expected wrapping result; x64 is cross-compiled
and inspected here, not executed. No additional neoCLR multiplication rewrite is needed.

[Raw speed timings](speed-results.json), using the same checked two-million-iteration
loop, give these medians across ten post-first invocations on the busy ARM64 host:

| Backend | Default inlining | NoInlining control |
| --- | ---: | ---: |
| none | 1.171 ms | 5.470 ms |
| speed | 1.303 ms | 2.699 ms |

These short timings demonstrate call-elimination behavior and mixed optimization
results, not a universal speedup or a stable startup comparison. The speed case's
inlined AOT objects shrink from 488 to 480 bytes (ARM64) and 255 to 243 bytes (x64),
while the no-inline controls grow. Broader workload and compile-latency comparisons
remain release/performance follow-up; the correctness gate is the backend suite.

Validation with production speed defaults: [212 backend tests pass](codegen-speed.log),
including the settings assertion, default/override inlining, JIT values, both AOT
targets, checked arithmetic, stack/fault handling and managed lowering. The 36 ignored
tests require explicit external host/fixture inputs; this is not a fresh full
HTTP/collector/release matrix. The native API snapshot check remains unchanged.
