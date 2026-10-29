# Descriptive test identifiers — 2026-10-29

The repository convention prefers backtick identifiers for behavior descriptions.
Traditional names remain valid when clear. Whenever prose is needed, put it in
the identifier with `[Test]`, rather than a Test attribute description.

## Migration

[The inventory](names.json) records 464 tests across 62 Raven files: 461 renames
and three retained traditional names (AnyWithoutCurrent, CasingNoNormalization,
CasingNotOrdinalFolding). Twelve direct async invocations were updated. Existing
backtick tests, helper/data-source names, and fixtures that explicitly test Test
attribute descriptions remain unchanged. Historical evidence keeps its old IDs.

The [source audit](name-audit.json) compares the migration against
`a8baba4f68ed8bbe0a0181b5dc561c15100a3cad`: applying only the recorded attribute,
identifier and invocation substitutions reproduces the changed Raven sources
exactly. All 464 assertion bodies and their helpers are unchanged.

Renames change name-based IDs, filters and ordinal discovery order. The harness
now expects qualified function names and the resulting ordinal order. Manual
async runners retain their explicit display strings and registration order.

## Focused validation

- All 464 exact identifiers compiled in a synthetic identifier probe, with display
  names checked against the inventory and identical final-artifact rediscovery.
  These synthetic bodies qualify names and generated references, not test behavior.
- Native collections: 23/23 passed, plus guard and filtering checks.
- Native configuration: 10/10 passed.
- Python harness syntax and `scripts/build-api-docs.py --check` passed.

See the [native execution report](execution.json). The bundle reused the verified
Panic-compatible compiler and libraries from `target/panic-rename-20261029`;
no runtime or compiler behavior changed in this slice. Full suites and a website
build were not repeated for this mechanical naming change.

Native execution command:

```sh
python3 scripts/test-runtime-library.py \
  --bundle target/descriptive-test-names-20261029/bundle/bundle \
  --runtime target/debug/neoclr --aot target/debug/neoclr-codegen \
  --output target/descriptive-test-names-20261029/execution \
  --suite collections --suite configuration --mode native
```
