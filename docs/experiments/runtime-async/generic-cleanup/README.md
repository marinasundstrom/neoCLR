# Generic async and synchronous use cleanup

Development checkpoint, 2026-10-12. Native Raven now supports generic Task-returning
async methods, including methods on generic owners, with runtime bodies or generated
state machines. Selecting the native async library also enables synchronous `use`
cleanup across suspension for the state-machine path. The runtime-body provider
remains an explicit compiler option; this change does not make it the compiler default.

The old runtime-async guard rejected generic methods before lowering. The generated
path additionally rejected generic frames and failed to preserve their lexical
generic scopes. Native nested classes already support explicit generic parameters;
the compiler now maps enclosing and method parameters consistently through frame
constructors, fields, locals and callbacks. No metadata-format extension was needed.
A separate runtime commit removes the redundant sixteen-field admission check for
reference records; their existing 32-slot/264-byte flattened bound remains enforced.

This matches the relevant .NET behavior: async generic methods retain their type
identity, and scoped resources survive suspension and dispose before completion.
neoCLR deliberately keeps Fault invocation-terminal rather than providing CLR
exception unwinding. Fault does not promise managed Dispose calls. Awaited disposal
and state-machine async-iteration limitations are separate from synchronous `use`.

## Qualification

[`RuntimeGenericUse.rvn`](../RuntimeGenericUse.rvn) tests value/reference/union
results, generic methods and owners, a captured generic value, private receiver
state, GC while pending and reverse-order disposal.
[`RuntimeUse.rvn`](../RuntimeUse.rvn) checks ordinary completion, cancellation,
implicit unit completion and no early disposal.

Raven native compiler commit `1f3cdda12c06a448caaf1110eb4b858ba8124089` is qualified against the pinned
`default-jit-complete` library bundle; [macOS evidence](mac-validation.json) records
32 successful compile/run/link checks, including the captured-lambda case.

The [runner](../validate_generic_cleanup.py) compiles both fixtures with both
lowerings, runs interpreter/JIT/AOT and repeats each AOT program four times on the
same heap. Reports pin the compiler binaries, backend, bundle, source and host inputs.
The .NET compiler regression suites also pass 37 scope-cleanup/generic-owner/capture
cases. Compiler main and the native branch remain separate: these changes are in
the native adapter and its portable admission path.

```sh
python3 docs/experiments/runtime-async/validate_generic_cleanup.py \
  --bundle target/default-jit-complete/bundle \
  --compiler /path/to/Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --output target/generic-cleanup-validation
```

Windows qualification uses the state-machine path. Windows runtime suspension and
Windows ARM64 JIT remain unsupported. The installed default-JIT gate before this
compiler slice has [separate evidence](../../jit-cli/default/README.md).
