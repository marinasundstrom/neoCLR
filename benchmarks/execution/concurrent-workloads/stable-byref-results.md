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

Keep the option test-only. Before enabling it in the regular experimental pipeline,
add executable mutable/aliased-binding regressions beyond the policy test, verify
root retention under collection at intervening calls, qualify broader native
JIT/AOT/sanitized consumers, and compare JSON plus standalone CPU/allocation costs.
Then check multi-owner lifetimes/scaling and pause/retention behavior. This work does
not qualify shared heaps or a moving collector. The native host ABI is unchanged.

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
