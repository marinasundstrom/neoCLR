# Interface-to-Object dispatch — 2026-10-12

The native value profile now accepts managed interface references as receivers and
arguments of Object methods. This restores the existing .NET-style implicit upcast;
it introduces no new guest API or alternate equality semantics. Only the exact
Object base selected by the verified load set qualifies, never an unrelated class.

The native backend renames Object to a private class after verifying original
bodies. The general verifier no longer recognizes implicit interface-to-Object
conversions under that private name. A verification-only copy now makes those
already-admitted conversions explicit at calls, local/argument stores and returns.
It preserves evaluation order and relocates branches. Original-scope verification
and native shape analysis still run; the verification copy does not grant arbitrary
casts. Native code, GC plans and runtime instruction offsets use the original body,
so this adds no executable cast, spill or allocation.

Validation on macOS ARM64:

- `object_value_dispatch_preserves_identity_overrides_and_null_faults`: six cases
  compare native output/fault diagnostics with the interpreter, including interface
  identity, distinct objects, overrides, null receivers and an Object local.
- Seven focused `object_` integration tests pass, covering existing string/object,
  display, identity and rejected-override behavior.
- `interface_upcast_requires_the_explicit_object_base` passes; absent Object
  projection and unrelated concrete classes remain ineligible.
- [Full discovery AOT compilation](aot-compile.json) now succeeds. This is object
  compilation evidence, not linked execution or complete discovery support.
- [Full discovery JIT run](jit.json) now starts execution, then correctly faults
  because the assembly module catalog was not retained. It is still a failed
  discovery run, without interpreter fallback.

Next: configure explicit module-catalog/function retention and qualify the actual
ownerless function descriptors/attribute queries. General callable invocation,
generic signatures and registration remain separate parts of
[issue #27](https://github.com/marinasundstrom/neoCLR/issues/27).
Windows execution has not been requalified by this slice.
