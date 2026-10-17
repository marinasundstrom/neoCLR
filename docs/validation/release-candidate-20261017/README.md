# Native text arrays: release candidate regression

Qualified locally on macOS ARM64, 2026-10-17. This is a focused correctness check,
not package acceptance or a performance measurement.

`TextVectors.rvn` encodes non-ASCII text through the native class library, reads
`Sequence<byte>.Count`, and iterates six grapheme characters. All three execution
modes return 42; commands, output and artifact hashes are in
[text-vectors.json](text-vectors.json). The JIT command deliberately overrides the
old extracted bundle's backend with the newly built optimized backend. AOT uses
that same backend. The original archive is not relabeled as passing.

The previous code rejected native `arrayref<Byte>`/`arrayref<Char>` results from
`Utf8Encode`/`StringGraphemes`. Admitting those exact declarations then exposed the
legacy UTF-8 immutable snapshot header and missing returned-byte-array dispatch
candidate. The fix gives native UTF-8 results the ordinary byte-array header and
retains their nominal `Array<byte>` interface implementations. Legacy copied-array
results retain their snapshot behavior. No public API or compiler emission changes.

This follows the existing nominal array contract: like .NET, an array result can
be consumed through the interfaces implemented by its array backing. The private
native header is an implementation detail, not a new language rule.

## Reproduce

Build `neoclr` and `neoclr-codegen` with `cargo build --locked --release -p neoclr
-p neoclr-codegen`. Use `scripts/native_source_project.py:create_project` to create
a project for `TextVectors.rvn`, importing the extracted native library bundle's
`NeoCLR.ClassLibrary.props`. Compile/link with `scripts/build-native-project.py
--profile console --project <project> --bundle <bundle> --aot
target/release/neoclr-codegen --output <new-output-directory>`.

Run the generated assembly with `neoclr run` in interpreter and JIT modes and run
the linked executable. Each must exit 42 without diagnostics. The retained JSON
records the exact local paths and module arguments used for this qualification.

Additional checks:

- `cargo test --locked -p neoclr-codegen --lib source_owned_text_vectors_keep_exact_elements_and_internal_contracts`: one test passes, covering both legacy/native results and invalid signatures/implementations.
- `cargo test --locked -p neoclr-codegen --test console utf8_text_`: five tests pass with Xcode clang and its macOS 26.2 SDK. The first run using `/usr/bin/clang` and Command Line Tools macOS 27 SDK failed to link (`unknown architecture arm64e.x1`); no product failure is inferred from that toolchain mismatch.

## JSON identity-pattern regression

The packaged JSON sample exposed an `isinst`/`unbox.any` pair targeting the same
value-only structural union already held by an `Error(value)` payload. Native
lowering now checks the initialized tag and preserves the payload without an Object
allocation. A default/null union does not match the type test; identity unboxing
preserves that default/null value, matching the interpreter. Concrete alternative
extraction retains its existing null and wrong-type faults.

Admission requires the same union shape, value-type alternatives, and an immediate
null test after `isinst`. Different union targets, reference-alternative union tests
and escaping test results remain outside this slice. This repairs an existing
packaged workload; it does not resume structural expansion. .NET has no corresponding
structural-union metadata signature; ordinary nominal boxing and .NET Raven are unchanged.

The unchanged JSON sample covers async stream/JSON/mapping error alternatives,
reflection construction/property access, UTF-8 streams and serializer round trips.
[json-patterns.json](json-patterns.json) records interpreter/JIT/AOT commands,
output, matching reflection roots and final artifact hashes. The final AOT compile
and link reuse the frozen assembly from the source build, avoiding stale reflection
row identities after recompilation. Generate reflection roots again whenever the
assembly is rebuilt.

Focused regression: `cargo test --locked -p neoclr-codegen --test values union_`
with Xcode clang/SDK as above: 13 tests pass. The new differential identity test covers initialized
and default scalar/record unions, identity unboxing and rejection of different
shapes or escaped test results. Existing exact-extraction, null and wrong-type
checks remain in the same suite.

## Package status

A new archive embeds the corrected backend. Its extracted-package checks, without
an override, pass collections, tasks and JSON. [Acceptance](package-acceptance.json)
records the archive hash and the next failure: Raven rejects the HTTP JSON server's
`HttpError | JsonError` match in `AdaptClientResult<T>` as non-exhaustive (RAV2100).
The sample is unchanged; no catch-all bypass was added.

[CPU/JSON preparation](benchmark-preparation.json) now compiles and links the worker
against that extracted bundle. This establishes build readiness only, not workload
correctness or performance. No measurements have been taken. Windows validation,
full package acceptance and final SDK/editor qualification remain pending.
See [release readiness](../../release-readiness.md).

## Native union coverage in Raven

Raven `codex/neoclr` commit `743e229a6` adds the missing structural constituent
coverage path to the existing match evaluator. Nominal interface conformance and
ordinary .NET union behavior are unchanged. This target-specific symbol is absent
from Raven main, so no main change or branch merge is needed for this fix.

[Validation](union-exhaustiveness.json) records an isolated copy of the installed
compiler with its Release/net10.0 `Raven.CodeAnalysis.dll` replaced. Ten existing
baseline checks pass; the new regressions fail in three expected cases before the
fix and all 21 selected native/.NET checks pass afterward. Public semantic-model
coverage answers are checked alongside diagnostics, not a full LSP/editor session.

The unchanged packaged HTTP server compiles. The client no longer reports missing
union coverage, but native metadata emission fails with `NEOMETA003: unsupported
authored method signature`. No sample catch-all or contract workaround was added.
The isolated composition is qualification evidence, not a replacement release SDK.
The installed SDK and previous archive are unchanged.

## Authored method union signatures

The metadata library now recursively admits native unions in authored method
signatures. The failing import was `HttpClientJsonExtensions.GetFromJson<T>`
returning `Task<Result<T, HttpError | JsonError>>`: Raven supplied the correct
signature, but the authored-method validator lacked its union branch. This changes
validation, not the native format or ABI. Intersections remain outside this repair.

`--authored-compositions` passes native composition checks plus nested generic-union
method interning, substitution, binary roundtrip and scope/ownership rejection.
`--authored-method-bounds` passes. The unchanged HTTP client now compiles using the
isolated Raven 743e229a6 compiler with the rebuilt metadata library; the installed
SDK and archived compiler have not been replaced.

[Interpreter](http-authored-union-interpreter.json) passes all HTTP requests and
the client/server pair. [JIT](http-authored-union-default.json) passes server requests,
but the client fails with `generic argument type metadata was not retained: Union`.
That backend retention failure remains a release blocker. The waiting server's
orphan backend was terminated after the client failed so the harness could finish
recording its report; the recorded killed-server status is cleanup, not the cause.
AOT and fresh coherent package/editor qualification remain pending.

## Union generic-argument metadata retention

The backend maps native union tokens recursively between source identities and
specialized constituent identities, just as it does for function signatures.
Union display names and shape queries retain structural/value-type identity; no
nominal declaration or ABI change is introduced. Missing constituent metadata and
open arguments still reject. Both interpreter and backend visibility now require
all union constituents to be public.

[Focused evidence](union-metadata.json) records seven backend metadata tests and
one interpreter visibility test, optimized backend/compiler hashes, runner adapters
and AOT build commands. The unchanged HTTP server request checks and Raven
client/server pair pass in [JIT](http-union-metadata-jit.json) and
[AOT](http-union-metadata-aot.json). Interpreter execution passed in the preceding
slice; its executable behavior is unchanged by this metadata-retention fix.
AOT outputs are built from fresh assemblies with matching reflection roots.

The previous failure reports remain available. The corrected backend is explicitly
overridden for JIT; the archived package and installed SDK are not modified.
Review of the introspection website found no new API or signature to document;
this repairs the existing generic-argument/retention contract. Full coherent
package/editor acceptance, Windows validation and runtime measurements remain open.

## Coherent macOS SDK and installed editor candidate

The local candidate `0.0.0-native-candidate.20261017` is a qualification identifier,
not a selected published release. NeoCLR `a9b01a0e1` and Raven `e5d6afe82` produce
an optimized runtime/backend, freshly packaged SDK (compiler, project tool and
language server), matching VSIX and all five native libraries. The normal Raven
packager uses net11.0 tooling, with the installed .NET 11 RC SDK; native sample
projects still target net10.0. This host requirement must remain explicit when
selecting the eventual release toolchain.

[Build commands](coherent-package-build.json) and [identity checks](coherent-package-identity.json)
record exact inputs and archive identity. The VSIX embeds the same server, compiler
analysis, native adapter and metadata-library binaries as the SDK server directory.
The archive is `target/release-coherent-20261017/neoclr-native-candidate-osx-arm64.tar.gz`.
[Extracted-package acceptance](coherent-package-acceptance.json) passes collections,
tasks, JSON and HTTP client/server checks using the bundled compiler and default
JIT, without binary overrides. Prior focused AOT and interpreter evidence remains
separate; this run does not claim full source or Windows qualification.

The packaged VSIX was installed in an isolated VS Code profile. The
[main editor suite](coherent-editor-acceptance.json) passes 26 checks including
native hover/completion/documentation/navigation, dependency replacement and
recovery, configuration recovery, build/run, failed-build artifact preservation,
ordinary .NET editor behavior and Tasks/await execution. [Inlay checks](coherent-editor-inlays.json)
pass after edits and concurrent range queries. Existing user SDK/default profiles
are unchanged.

The [initial editor run](coherent-editor-legacy-harness-failure.json) exposed an
obsolete harness assumption: it tried to invalidate the removed ObjectLibrary
property. Both configuration harnesses now mutate `RavenNeoClrNativeCoreReference`
to a missing DLL; syntax checks pass and the main suite confirms invalidation and
recovery. The [repeated configuration test](coherent-editor-configuration.json)
recovers twice, then times out waiting for the third missing-core diagnostic.
The harness restores the valid configuration in `finally`. This remains an editor
release blocker; no cause or runtime failure is inferred from that timeout.

No publication, global SDK replacement, Windows run or benchmark measurement is
claimed. Next: investigate repeated configuration diagnostic publication/reload,
then qualify the repaired editor and complete Windows/source/benchmark gates.

## Reload recovery investigation

The unchanged packaged server passed a [traced three-cycle run](coherent-editor-configuration-three-traced.json),
but the [ten-cycle run](coherent-editor-configuration-ten-traced.json) again missed
the third invalidation diagnostic. The harness now records edit phases and all
project-load diagnostic transitions. This is intermittent, not qualified by the
short passing run. Server logs for the failing cycle show Async rejecting the
missing core without a corresponding App load failure, although they share the
imported props file; the cause remains unproven.

A separate deterministic regression exposed loss of open-document membership
when restoring a failed reload snapshot. Raven main `24898a8d5` preserves that
membership; `codex/neoclr` `e7ce2f8ee` carries it deliberately. Two focused reload
tests pass on each line, including unsaved text preservation through failure and
recovery. A ten-edit imported-property evaluation test passes on main. No Runtime
Contract, signature, lowering or ABI change is involved. The archived candidate
has not been rebuilt with this fix; its identity and previous evidence remain
unchanged.

The [development-server ten-cycle run](development-editor-configuration-recovery.json)
uses the same isolated extension with an explicit server override to Raven
`e7ce2f8ee` Debug/net10.0, including the native metadata adapter. It recovers four
times, then times out waiting for the fifth recovery: App retains the missing-core
error after the props file is restored, while Async has cleared. The harness
restores valid configuration in `finally`. This is diagnostic evidence, not a
qualification of the archived candidate or a claim that the original intermittent
failure is fixed. Next: trace watched-file delivery and evaluation/publication
ordering across both roots before selecting the next fix.

## Watched-file fixes and repeated recovery

Raven main `860fe15fd` fixes output-directory exclusions to be relative to the
opened workspace; native carry `b6590c2d2` passes the same six focused tests.
Previously an explicitly opened workspace under `target` or `artifacts` ignored
its own ordinary configuration events. Native metadata discovery happened to
bypass that exclusion, while reevaluating projects during file-event filtering.

Native `7bfbce21f` snapshots metadata dependencies during project loading instead.
Unrelated file events cannot populate the evaluation cache during filtering;
failed reloads retain prior and newly discovered dependencies for recovery.
Five focused tests pass, including a regression that failed before the change.
These fixes remove observed filtering/evaluation hazards; the precise scheduling
of every earlier intermittent failure is not claimed to have been reproduced.

The [fixed development-server check](editor-fixed-development-both-projects.json)
passes ten invalidation/recovery cycles, requiring diagnostics from **both** native
projects and recovery of **both** before continuing. This uses Raven `7bfbce21f`
Debug/net10.0 with the explicit server override. Fresh SDK/VSIX packaging and a
no-override installed-server run are the next qualification step. The older
archive remains unchanged. Runtime contracts, emission and ABI are unaffected;
structural expansion remains parked.

## Fixed packaged editor qualification

Local candidate `0.0.0-native-candidate.20261017.2` contains Raven `7bfbce21f`,
fresh SDK/server/VSIX and five libraries built by that exact compiler.
The optimized runtime and codegen binaries are unchanged from the prior qualified
candidate. [Identity](editor-fixed-package-identity.json) checks SDK/VSIX server,
analysis and metadata-adapter hashes, runtime reuse and all four installed-server
selections. [Build commands](editor-fixed-package-build.json) and
[extracted acceptance](editor-fixed-package-acceptance.json) identify the archive
and passing collections/tasks/JSON/HTTP execution without overrides.

The [main installed-editor suite](editor-fixed-main.json) passes all 26 checks,
and [inlays](editor-fixed-inlays.json) pass. Two independent runs of the stronger
configuration test pass [ten cycles](editor-fixed-configuration-1.json) and
[ten more](editor-fixed-configuration-2.json): both native projects must report
the missing core and both must clear their errors after restoration in every
cycle. No server or SDK override is configured. This closes the repeated stale
project-diagnostic blocker for this macOS candidate.

Archive: `target/release-editor-fixed-20261017/neoclr-native-candidate-osx-arm64.tar.gz`.
This is local qualification, not a published release or an update to the user's
normal SDK/profile. Windows qualification, canonical source gates, remote candidate
pins and runtime measurements remain release requirements. CPU/JSON preparation
against this extracted archive passes; timed results are recorded separately.
