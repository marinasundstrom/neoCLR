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

## Remaining release blocker

The extracted package's JSON sample gets past the text binding gate but fails at
an async generated function's scalar-union instance test: `scalar union instance
tests require tagged conversion lowering`. Keep the packaged sample as the
acceptance workload; do not bypass its error-alternative checks to obtain a pass.
Windows validation and benchmarks remain pending. See [release readiness](../../release-readiness.md).
