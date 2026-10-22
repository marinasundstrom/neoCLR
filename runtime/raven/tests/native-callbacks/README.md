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

## Returned functions (2026-10-22)

The follow-through adds two tests, bringing this suite to seven. C exports return
`int (*)(int)` and `void (*)(int)`; Raven NativeImport signatures return `(int) -> int`
and `(int) -> unit`. Tests force collection before invocation and pass each result
back through the original callback imports. The unit case assigns the direct result
to System.Void/unit and checks native side effects after both calling directions.

The SDK preserves `function_return`; Raven inherits the import's convention. Numeric
returns use the interpreter's native Function binding directly. Unit returns use an
output-owned adapter that invokes a no-result Function then materializes System.Void.
The adapter/binding allocate once per import result. Loaded library owners remain
retained, but foreign context ownership is not transferred. Null and data pointers
remain rejected by the existing runtime contract. Retained callback registrations,
foreign threads and JIT/AOT execution remain separate gaps.

Evidence: `target/native-return-tests-20261022/report.json` (7/7 interpreter tests),
metadata SDK `--foreign-imports` (numeric/no-result roundtrip and descriptor rejection),
and the website's focused tests. The local bundle compiler overlay was refreshed from
Raven codex/neoclr `5966b5f84` (built before comment-only cleanup); no SDK release is claimed.
The comparison remains .NET native function/delegate binding, using structural Function
instead; explicit ABI metadata is retained rather than inferring a callable from an
untyped pointer.
