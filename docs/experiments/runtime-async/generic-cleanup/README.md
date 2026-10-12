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

## Producer, constructor and project follow-through

The compiler rebuilt Runtime/Data/Networking/Web from clean neoCLR sources at
`4778c5ac`: [build evidence](library-build.json), [bundle preparation](bundle-preparation.json).
All 32 checks pass again with those freshly produced libraries:
[consumer evidence](mac-fresh-libraries.json). The compiler/library files in that
bundle remain pinned to the original async fix, rather than being silently replaced.

The fixture exposed a general Raven bug in explicit `self.value` initialization of
generic readonly storage. Main commit `3bfdf9b92` fixes it with 35 passing property
binding/execution tests. Native commit `201d1c090` carries that focused fix without
merging compiler branches; [native evidence](mac-readonly-integration.json) uses
an explicit initializer with both async lowerings and all three execution modes.

The first expanded Windows [run 38184041599](https://github.com/marinasundstrom/neoCLR/actions/runs/38184041599)
passed HTTP/framework checks but rejected project-based async `use` with RAVT006:
[gate](windows-first-gate.json), [project failure](windows-first-project.json).
Its report stays failed. Project loading still restricted cleanup to runtime bodies,
while the direct CLI path had already enabled it for the selected async library.

Native compiler `c1a938ccdc473dda6a7dfb2dbde4710855d5b5db` aligns `RavenNeoClrAsyncLibrary` project configuration
with `--async-library`. [Project evidence](mac-project-validation.json) passes
32 checks across both lowerings, interpreter/JIT/AOT and repeated native invocations.
Use `--project` with the runner to exercise this path, and `--case generic-use`
for only the generic fixture. The Windows gate now pins this corrected compiler.

The second Windows [run 38185267503](https://github.com/marinasundstrom/neoCLR/actions/runs/38185267503)
compiles the project successfully, then exposes explicit `GC.Collect` being bound
only with macOS runtime suspension: [gate](windows-second-gate.json),
[project](windows-second-project.json). Explicit collection now belongs to the
shared native GC adapter and is enabled by `--native-gc`, independently of async
lowering. It uses the same published guest/host roots and does not add Windows
runtime suspension. Windows execution qualification now passes (below).

The shared collector rerun passes [32 macOS project checks](mac-shared-gc-validation.json),
including both lowerings and repeated same-heap AOT execution. State-machine AOT
now explicitly omits `--bind-runtime-async`. Both fixtures also pass
[Windows x64 object emission](windows-cross-gc-validation.json) on macOS; this
cross-emission check is not a substitute for Windows execution.

## Completed Windows qualification

[Run 38186468730](https://github.com/marinasundstrom/neoCLR/actions/runs/38186468730)
passes at neoCLR `ce90e82b9bd79c93bdd24db03a87a930fcef8181` with Raven
`c1a938ccdc473dda6a7dfb2dbde4710855d5b5db`: [gate report](windows-final-gate.json),
[cleanup project](windows-final-use-project.json),
[generic project](windows-final-generic-project.json). Both fixtures execute
successfully under interpreter, installed default JIT and native AOT on Windows x64.
The same gate passes scalar/CLI, HTTP and framework checks.

[Provenance](windows-final-provenance.json) records all six exact execution outputs
and artifact hashes; JSON reports normalize CRLF to LF only.
[Preparation](windows-final-preparation.json) and [manifest](windows-final-manifest.json)
pin the rebuilt helper, compiler and libraries. The macOS shared-GC report retains
execution output and input hashes, replacing large native compilation inventories
with their SHA-256 hashes; its original report hash is recorded.

This closes the bounded generic async/synchronous async-`use` compiler slice.
Windows runtime suspension, Windows ARM64 JIT and awaited state-machine disposal
remain separate capabilities. Fault remains non-recoverable and does not promise
managed cleanup.
