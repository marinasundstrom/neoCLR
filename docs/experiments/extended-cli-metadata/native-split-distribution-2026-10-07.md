# Split native distribution candidate — 2026-10-07

The native POC packager now accepts the source-built Runtime/Data/Networking/Web
bundle, keeping the legacy Numbers/Http input mode explicit. Split inputs carry
Core, the finalized seed, ownership, shared project configuration and generated
XML/Markdown sidecars. The packager checks every artifact against both the bundle
manifest and the library build evidence, and checks that its SDK compiler matches
the producer. Missing or mismatched inputs fail before staging.

Sample projects import the same relocatable NeoCLR.ClassLibrary.props used by
compiler/workspace acceptance. VS Code settings select the adjacent SDK using a
relative raven.sdkPath; Run tasks use the adjacent compiler/runtime. The extracted
verifier obtains seed/module/Object-root paths from the package manifest, with legacy defaults
for old archives. It checks all hashes before compiling and executing consumers.
This changes packaging, not native metadata, Runtime Contract or .NET semantics.

## Qualified results

A fresh macOS arm64 SDK staging build at Raven `ce51cd941` succeeds, including host
Core/Macros generation and the language server. The library producer is that SDK's
published compiler. The runtime is rebuilt with cargo build --release at NeoCLR
`f9c7bd3c` (only packaging/documentation changes in this slice).

The archive is extracted into a path containing spaces. All 1,028 manifest files
verify; collections, Tasks/await and JSON object mapping compile/run with exact
output and exit 0. Both HTTP assemblies compile and the HTTP/JSON requests and
paired client/server checks pass. Libraries are consumed solely from native artifacts.

The newly packaged VSIX installs in an isolated VS Code extension directory. All 26
extension-host checks pass using that installed package directory and the extracted
SDK, including API help, metadata navigation, configuration/dependency recovery,
orders/Tasks execution and ordinary .NET hover. The harness selects the installed
package as its extension development path; it does not use checkout extension code.
This is VS Code 1.140.0 and extension 0.1.13, with a development SDK version.
Warm Main.rvn hover commands took 2–9 ms; post-edit requests took 59–66 ms. These
observations do not resolve the prior persistent Loading report.

Negative checks reject mismatched producer binaries, altered XML and omitted sidecars
before staging. [Machine-readable evidence](native-split-distribution-2026-10-07.json).

## Reproduce

Build a native-enabled SDK with Raven's package-sdk.sh and the explicit
RAVEN_NEOCLR_METADATA_PROJECT selection. Build the four class libraries with that
SDK's tools/rvnc/rvnc.dll using build-native-class-library.py. Package the matching
VS Code extension, then run:

```sh
python3 scripts/package-native-poc.py --sdk "$SDK" --vsix "$VSIX" \
  --libraries "$LIBRARIES" --runtime "$RUNTIME" \
  --runtime-revision "$RUNTIME_REVISION" --output "$OUTPUT"
mkdir "$EXTRACTED"
tar -xzf "$OUTPUT/neoclr-native-poc.tar.gz" -C "$EXTRACTED"
python3 "$EXTRACTED/neoclr-native-poc/tools/verify-native-bundle.py" \
  "$EXTRACTED/neoclr-native-poc" --report "$REPORT"
```

All output directories must be fresh. Omit --core and --seed for split bundles;
those inputs are selected and hashed by the class-library build. Supplying a
second selection is an error. Compiler/runtime source revisions and artifact hashes
remain explicit provenance; no package resolver or implicit CLI projection is added.

This candidate is not release publication or a bootstrap-free compiler. Core and
the retained seed remain dependencies. Platform integration projects, complete API
help/unified RavenDoc generation and the persistent hover report remain open. A
fresh staging build from an existing worktree is not a clean-checkout reproduction.


## Failure found during qualification

The first extracted collections build succeeded, but direct execution failed with a
missing referenced System module because the old verifier omitted --object-root.
Ordinary rvnc --run already supplies this explicit binding. The package manifest and
both extracted runtime/HTTP verifiers now carry the same source-built Object selection.
This is a qualification-tool correction; no reference validation was relaxed and no
runtime fallback was added. Legacy aggregate archives retain their existing behavior.
