# Preserve immutable byref argument provenance

2026-10-25. An opt-in compiler prototype follows the
[remaining Result constructor calls](../concurrent-http-owners/forward-results.md).
Production/default behavior remains unchanged. This is a promising optimization,
not completed qualification for a new concurrent default.

## Cause and proposed lowering

In the exact isolated plaintext selection, `neoclr_value_855` has seven known
callers and an established 584-byte stack extent for parameter zero. Nevertheless,
its 82 stores initially have no eligible stack-argument removals. The pre-lowering
IR shows the constructor's byref parameter spilled to `ss0` and subsequently loaded;
that load hides its origin from the existing address proof. A snapshot write then
makes many scalar-store calls despite its destination residing on the caller stack.
The diagnostic rebuild produces the identical baseline object.

Test-only `NEOCLR_LIVE_SSA_BYREF_ARGUMENTS=1` preserves the incoming SSA value for
`Ty::Address` arguments only when the entire function contains neither `StoreArg`
nor `ArgumentAddress` for that argument. Other arguments, mutable bindings and
addressed bindings retain normal loads. Stores and descriptors for argument roots
are still emitted; no root slot, publication, allocation or error cleanup is removed.
This enables the existing complete-caller extent proof; it does not itself authorize
an unchecked heap write. The constructor now has 75 eligible removals out of 82.

This relies on the current nonmoving native collector. Before moving/compacting
collection, cached byrefs would need relocation-aware handling or this optimization
must be disabled. Pointee mutation is allowed; the pointer binding is immutable.
Whole-body exclusion includes unreachable operations and is independent of CFG
simplification. No handle migrates to another thread or outlives its invocation.
The compiler still publishes the original roots and retains runtime heap checks
for destinations whose extent cannot be proven.

## Evidence so far

The new binding-policy test and all 52 focused mutation/store-audit tests pass with
the switch enabled. The latter invocation explicitly ignores 31 native integration
tests requiring separately supplied hosts. Six additional managed-JIT standalone
runs exercise tasks, collections and arrays with stack elision off/on: expected
output, positive completed epochs, balanced shutdown and released index metadata
pass (1,520 completed epochs total). These are correctness runs, not JIT timing claims.
Disabling the new option reconstructs the baseline plaintext object byte-for-byte.
Final review makes optional lane selection lazy; repeated focused tests pass and
the final candidate object remains byte-identical to the measured candidate.

Three alternating one-owner AOT plaintext pairs validate **104,511 responses**:

| Live configuration | Median requests/s | CPU us/response |
| --- | ---: | ---: |
| Qualified direct/forwarded elision | 3,217.0 | 325.64 |
| Plus immutable-byref SSA prototype | 3,737.5 | 283.97 |

This cohort shows **16.2% more throughput** and **12.8% less CPU/response**. It uses
32 lanes, one-second warmup and four-second load windows; CPU includes startup,
warmup, drain and shutdown. Managed allocations remain about 208/response; small
fractional differences are amortization, not savings. Out-of-arena adapter calls
fall from about 4,057 per response to four calls across each whole candidate process.
All responses require EOF, completed epochs and index teardown; no failing pair is
excluded. No pause-distribution or memory improvement is established.

## Remaining qualification

Keep the option test-only. The root and native qualification below closes a
specific safety gap; it does not qualify moving collection, shared heaps or the
regular experimental pipeline. Next compare JSON and standalone CPU/allocation
costs, then multi-owner lifetimes/scaling and pause/retention behavior.

Raven source/semantic emission, Runtime Contract configuration and CLI/native
metadata representation are unchanged. This is neoCLR backend SSA preservation,
with no new bridge encoding; eventual native metadata replacement is unaffected.
The approach follows the existing compiler-proven stack/ownership design rather
than adding a special Result GC layout or trusting arbitrary pointer magnitudes.

[stable-byref-evidence](stable-byref-evidence/) retains original/candidate IR, audits,
tests, compiler/link recipes, executable input hashes, paired measurements, JIT
commands and the disabled-object check. The build and JIT drivers are archived
alongside reports. The flag must be combined with the existing complete-closure
stack-argument elision switches to obtain the measured guard removal. Fresh output
paths and matching retained bundles/native hosts are required.

## Root and native qualification follow-up (2026-10-25)

The executable mutable/aliased-binding fixtures initially failed during assembly:
the existing verifier rejects `starg` and `ldarga` on managed-reference parameters.
The new regression asserts that boundary explicitly for both fixtures. This is an
existing support limitation, not a new semantic restriction or a reason to enable
unsupported operations for this optimization. The defensive backend whole-body
policy test remains useful if an internal projection later admits such operations;
actual mutable-binding execution is not claimed.

The native consumer now begins an epoch with an empty caller root frame and then
calls generated `Write`, which nests two `Put` calls. The destinations belong to
**different objects**. The second destination must survive the first nested call
via its generated argument storage; the incoming value and its child must survive,
while an unrelated object is reclaimed. The epoch must complete inside generated
code, frame linkage must return to the empty caller, and a final empty-root
collection must release the heap. This additional test runs in unpaced completion
mode; the existing paced tests retain their caller roots and are reported separately.

A first draft used two fields in one object. Deliberately omitting storage-root
tracing did not fail that draft because transient call roots retained the shared
object. Splitting the objects made the control meaningful: both optimized control
and candidate now fail inside the new fixture when storage-root tracing is omitted.
Only the generated temporary test collector is modified for these negative runs.
The production/native tracing implementation is unchanged.

`concurrent-store/qualify-byref.py` clears ambient experiment flags and runs matched
speed-optimized controls with established direct/forwarded extent proofs enabled,
toggling only immutable-byref SSA preservation. Both configurations pass the three
byref-focused checks, native hook/host admission checks, optimized and UB/bounds AOT,
and managed-JIT tests. The new empty-caller case completes **600 cycles** across
the two optimized AOT, two sanitized AOT and two JIT executions. Two deliberate
missing-storage-root controls fail as required. No sanitizer-enabled JIT claim is
made; sanitizers cover native AOT hosts.

Six additional candidate families pass at speed optimization, with direct/forwarded
stack elision: reference arrays, record arrays, managed byte views, callbacks, async
pump and statics. Each runs optimized AOT, UB/bounds AOT and JIT in unpaced and paced
modes: **36 consumer executions, 3,960 fixture cycles**. They exercise publication,
pressure, faults, bounds, resume/abandon and cleanup through ordinary layout and
root contracts. They do not introduce a special record GC representation. These
are correctness runs, not timing or allocation comparisons. Prior six standalone
JIT samples and HTTP measurements above are reused, not rerun or enlarged here.

The `root-qualification-*` and `native-matrix-*` compressed evidence files retain
commands, selected environment, source/binary hashes, outputs and fault controls.
Reproduce root qualification with a fresh output path and the matching release
codegen test runner:

```sh
python3 benchmarks/execution/concurrent-store/qualify-byref.py \
  --runner target/release/deps/neoclr_codegen-12f33f26d2ff4df0 \
  --output target/fresh-byref-root-qualification
```

Use the recorded Xcode/SDK environment on this macOS ARM64 host. The separate
native matrix report contains all six driver commands and their common environment.
Website/API changes are unnecessary: public contracts and production defaults have
not changed. Broader performance, thread-sanitizer and default qualification remain
open; this follow-up does not establish another throughput/CPU/allocation gain.

The subsequent [workload cost comparison](stable-byref-cost-results.md) covers
90 standalone runs and 24,768 JSON responses; it reduces the performance
qualification gap but leaves JIT JSON, scaling, pause and default acceptance open.
