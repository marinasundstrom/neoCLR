# Scoped runtime-service result contracts — 2026-10-07

The retained seed declares WriteLine(String) with an inhabited Void result; Raven's
source-owned declaration uses CLI no-result. The native registry already explicitly
admits both forms, and host-call resumption already honors each callee's no-result
flag. Cross-assembly duplicate validation nevertheless rejected the two declarations
before calls could use their scoped identities.

Validation now admits these distinct declarations only when they belong to different
modules, are both InternalCall, have identical parameter/result types, return intrinsic
Void, and the native registry accepts **both complete contracts**. Identical no-result
flags retain their existing path. Per-module duplicate rejection remains; ordinary
methods and unsupported result signatures still reject. No declaration is merged,
renamed or silently substituted. Local service references bind to their own module's
member identity before whole-program validation, and execution retains that contract.

This reuses the [source-unit contract](source-unit-2026-10-07.md) and existing no-result
service admission. Like CLI void, a no-result call produces no operand-stack value;
neoCLR's older unit-return declaration also remains usable. The explicit host adapter
handles that distinction. This is not general overload resolution by return type or
new metadata semantics, and it does not relax the registry for arbitrary callbacks.
No compiler, public metadata API or format changes are required.

## Validation and next gate

[Evidence and artifact hashes](scoped-service-results-2026-10-07.json):

- Two focused Rust tests pass. Separate native NEOX modules round-trip, link and
  execute value-return and no-result WriteLine declarations. The caller consumes a
  value only where declared; both outputs are exact (`value\ncontrol\n`), exit value
  is 42 and stderr is empty. The equal-contract control also passes.
- Ordinary-method conflicts, wrong results and duplicate declarations within one
  module reject at the intended validation layer. The existing native registry
  result-signature test passes, preserving its service allowlist.
- The already emitted 197-input System artifact advances past WriteLine. Runtime
  admission now rejects DnsLookup's callback result: it is the source-owned nominal
  System.Void, while the current completion-callback registry expects intrinsic Void
  or an explicitly admitted no-result shape. Full System execution remains unproven.

Next reconcile the selected unit representation inside function signatures at the
metadata/runtime boundary; do not admit arbitrary empty nominal structs as completion
callbacks or change .NET codegen. Keep the candidate Runtime/Data/Networking/Web
partition work behind executable ownership/ABI gates.

Runtime base is 96c55f89 plus this slice; compiler remains ca4aeccfb-equivalent with
the schema-4 metadata library. Formatting passes. No public guest API, website example
or documentation snapshot changes; no website build is needed for this runtime fix.
