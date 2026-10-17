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
