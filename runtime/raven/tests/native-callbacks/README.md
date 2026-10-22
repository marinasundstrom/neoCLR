# Raven native callback qualification

Development, 2026-10-22: five built-in Raven tests execute real C callbacks through
NativeImport in the Unix interpreter. No neoIL is used to supply source behavior.
The fixture calls synchronously on the invoking thread and never retains a callback.

Coverage: module functions, repeated capturing callbacks with forced GC, bound
receivers, a unit-returning callback with forced GC, and a generic method return
closed over unit. The last test calls the same callback both directly and through C,
assigning its managed result between `unit` and `System.Void` in both directions.
`unit` is Raven syntax; the emitted runtime type is System.Void. Generic value-return
ABI and direct no-result ABI remain distinct execution conventions, not distinct types.

```sh
python3 scripts/test-runtime-library.py \
  --bundle /path/to/matching-bundle \
  --runtime /path/to/current/neoclr \
  --output /path/to/new-output-directory \
  --suite native-callbacks --mode interpreter
```

The harness builds fixture.c using CC, substitutes its absolute library path into
Imports.rvn, discovers the tests from compiled metadata, and verifies registration
and runtime output. Use a Raven native compiler built with the matching metadata SDK.
The compiler change is Raven codex/neoclr `18dd51f62`. Local qualification used
the same behavior based on 885925873 plus this slice (before removing an unused
private helper parameter),
neoCLR main based on 30c3fe1cc plus this slice, and a local compiler overlay on the
native-buffer development bundle. It is not a rebuilt or released SDK.
Report: `target/native-callback-unit-tests-20261022-d/report.json` (5/5 passed),
macOS ARM64 interpreter. Metadata SDK `--foreign-imports` roundtrip/rejection checks
and 24 website tests also pass.

The native compiler emits a source-facing managed wrapper only when a unit callback
needs projection. An internal foreign thunk accepts a true no-result Function; an
adapter discards the managed callback's logical Void result. This currently allocates
an adapter and Function binding for each adapted parameter on every call. Numeric
callbacks keep the existing path. No throughput or allocation improvement is claimed.

Compared with .NET delegate marshalling, this uses neoCLR's structural Function and
explicit metadata ABI. The same call-scoped lifetime rule applies; Raven's unit value
requires the compiler-owned return adaptation. Existing interop research and lifetime
tradeoffs are recorded in docs/design/native-import-export-contract.md.
Retained callback registrations, arbitrary native threads, JIT/AOT callbacks and
returned-function source projection are not qualified here. A generic module-function
method-group lookup and direct unit equality encountered during test expansion remain
separate compiler gaps; the generic ABI test uses a static class method and typed
assignments, so it does not claim either gap is fixed.
