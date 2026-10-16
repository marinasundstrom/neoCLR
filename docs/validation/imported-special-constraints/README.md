# Imported generic class constraints — 2026-10-16 development

Qualifies the existing generic reference-class constraint profile across a separate
native provider and consumer. This extends the reference/value slice with `new()`
and nongeneric nominal class bounds; it does not resume structural types.

Run from the neoCLR repository:

```sh
python3 docs/validation/imported-special-constraints/run.py \
  --bundle target/variance-argument-sequence-20261016 \
  --output target/imported-all-constraints-20261016
```

The SDK fixture checks native generic-parameter special/nominal introspection,
explicit import registrations, identical/conflicting facts, invalid arguments,
abstract constructor rejection, symbolic special-base requirements, late
registration and CLR execution. The separate native consumer preserves reference
identity/storage and returns 42 in the interpreter and AOT; managed JIT reports 42.
AOT enables UBSan/bounds and native hosts verify cleanup. See [results](results.json).
The host SDK suite passes 182/182 checks. All 16 codegen specialization tests pass,
including nominal-bound rejection and unchanged source metadata.

[The Raven consumer](raven/README.md) additionally executes nominal subclass
arguments and local/imported constructor arguments across a real dependency.

Tested on macOS ARM64, neoCLR `main`, Raven `codex/neoclr`, using bundle
`target/variance-argument-sequence-20261016`. Reports record artifact and source
hashes. This is development qualification, not a deployment or a cross-platform
matrix. No Runtime Contract configuration or metadata/ABI change is needed.

Remaining profile limits: constrained generic methods, constructed/interface
nominal type bounds, constrained interface/value owner imports, nested constrained
owners, and Raven source declarations whose nominal bound is an external type.
`new T()` lowering is distinct from enforcing the constraint. Ordinary nominal
interface implementation remains explicit. Sliceable/Result variance stays
postponed. See the [integration contract](../../raven-cli-bridge.md#imported-generic-class-constraints-development-2026-10-16).
