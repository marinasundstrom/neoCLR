# Unretained function-name candidate experiment — 2026-10-26

**Status recorded 2026-10-28:** This prototype was reverted before the boxed-Char scan candidate. Its raw timing/memory/profile artifacts and frozen sources under `target/` were no longer available when work resumed. Preliminary results were reported as mixed, but cannot now be audited and are not published as measurement evidence. Surviving test/build logs are archived here. The following describes the tested prototype, not a shipped API.

Baseline is `ad5a9f22b`, including immutable selection and specialization indexes. The [preceding startup profile](../specialization-lookup-20261026/README.md) still spends substantial sampled cycles in selection and comparison; inspecting call resolution reveals repeated scans of the entire function inventory despite an existing name index. This experiment targets those scans, without claiming that all comparison samples belong to this path.

## Contract and alternatives

`Module::functions_named` is a Rust compiler/runtime integration iterator, not a managed API or a complete method resolver. It preserves exact name matching and source row order. Within an immutable metadata scope it reuses existing candidate rows; outside a matching scope it scans without allocating a candidate list. The iterator borrows the module, so mutation cannot invalidate its rows; shared row storage may outlive the lookup scope until the iterator is dropped.

Native closed-world call resolution uses these candidates and retains every existing owner, instance, parameter and optional definition-identity check. Generic calls still fail before ordinary resolution, missing or mismatched calls retain their diagnostics, and multiple matching rows remain ambiguous. No resolved result, error or conformance decision is cached. This adds no new index: the function-name map already exists in the current scopes.

Reuse the [.NET/CLR metadata-resolution comparison](../../design/allocation-metadata-runtime-comparison.md#scoped-method-owner-candidates--2026-10-24): narrowing candidates must not substitute for signature/token/ownership checks. Direct token indexing would need uniqueness/canonical-row guarantees for all callers; caching complete resolution would need complete target/lifetime keys. The existing candidate map is a bounded first step with a linear differential oracle. Managed APIs, metadata, GC contracts and first-call scheduling remain unchanged.

## Validation

New tests compare scoped and unscoped overload/identity resolution, including wrong names, parameters, owner, instance flags, definition IDs, generic arguments and ambiguity after mutation. Another test covers nonadjacent row order, missing names, an iterator surviving scope exit, and rebuilding after mutation. Existing full selected-metadata/report differential tests remain in place.

All six lookup tests and 209 code-generation tests pass, with 35 manual probes ignored. The paired measurement run was interrupted before its evidence could be archived; no performance outcome is claimed. The measurement recipe preserves production-default codegen, identical guest/native hosts, eager/lazy JIT and both collectors. Timing, memory and CPU profiling run separately, with no task-owned build/profiler overlap during timing. Shared desktop activity remains uncontrolled.
