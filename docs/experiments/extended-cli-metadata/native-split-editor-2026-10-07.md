# Split class-library editor acceptance — 2026-10-07

The installed VS Code acceptance now consumes the source-built Runtime, Data,
Networking and Web bundle through `NeoCLR.ClassLibrary.props`. Library sources are
absent from the consumer project. This is development-toolchain qualification on
macOS arm64, not a newly published SDK or completion of the bootstrap release gate.

## Reproduce

Build the native-enabled Raven compiler and language server with the matching
`NeoClrMetadataProject`, and compile the Raven VS Code extension. Use the bundle
from [the configuration gate](native-bundle-configuration-2026-10-07.md).
With absolute paths and fresh OUT, PROFILE and EXTENSIONS directories:

```sh
python3 scripts/prepare-native-editor.py --sdk "$SDK" --bundle "$BUNDLE" \
  --runtime "$RUNTIME" --output "$OUT"
code --new-window --user-data-dir "$PROFILE" --extensions-dir "$EXTENSIONS" \
  --disable-extensions --skip-welcome --skip-release-notes --disable-workspace-trust \
  --extensionDevelopmentPath="$RAVEN/src/Raven.VSCode" \
  --extensionTestsPath="$NEOCLR/scripts/native-vscode-acceptance.cjs" \
  "$OUT.code-workspace"
```

`--bundle` replaces the individual core/seed/ownership/reference options. The
fixture copies the bundle and imports its configuration for the application,
async application and separately compiled editor test libraries. It retains the
previous explicit-artifact mode for older fixtures.

## Results

All **25 checks passed** in VS Code 1.140.0 with Raven extension 0.1.13.
Fifteen warm Main.rvn hovers took 3–12 ms; three hovers immediately after unsaved
edits took 65–76 ms. An earlier run recorded up to 2.749 seconds for startup/reload
provider requests despite a maximum hover-handler duration of 86.2 ms. Waiting
outside the measured handler is a hypothesis to investigate, not a proven fix.
Raven revision: `e93fcfdc19dad0c9be46bd6a41b26b2b3796e649`.
NeoCLR source/tooling base: `9f0946c9` plus this acceptance slice; reused runtime and
bundle provenance are retained in the [machine-readable evidence](native-split-editor-2026-10-07.json).

## Acceptance and boundaries

Checks cover native hover, completion, declaration navigation, Markdown/XML help
and refresh, library replacement, missing-reference recovery, unsaved buffer
preservation, each optional library's native type navigation, and imported bundle
configuration failure/recovery. Normal build/run tasks execute unchanged orders
and Tasks/await samples with exact output and successful exit status. Failed builds
preserve the last good artifact. The companion .NET project retains ordinary hover
and diagnostics. Repeated native type/query hovers and hovers immediately after
unsaved edits are timed through VS Code's provider command.

The measured command interval includes client/server waiting; it excludes mouse
hover debounce and rendering. These are single-machine observations, not a
performance guarantee. Fast warm requests do not disprove the author's report of
persistent Loading messages in an earlier Main.rvn session. Keep that issue open
until its exact workload is reproduced or the cause is independently established.

This split bundle has no class-library documentation sidecars. Authored hover help
is tested on the separate EditorLibrary; full Runtime/Data/Networking/Web API-help
packaging remains pending. The local server build reuses an existing Raven.Macros
binary with BuildProjectReferences=false after the macro rebuild stalled. The
recorded artifact hash makes that input explicit; this is not a clean SDK build.
Primitive Core and the finalized retained seed remain explicit dependencies.

No compiler, native metadata or runtime semantics change in this slice. The
.NET editor path stays unchanged; NeoCLR uses native library imports and explicit
Runtime ownership. Platform service extraction, documentation bundling and clean
distribution qualification remain subsequent gates.

See the companion JSON for revisions, artifact hashes, source/configuration hashes,
raw VS Code results and performance observations.
