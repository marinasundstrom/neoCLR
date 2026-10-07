# Native class-library documentation packaging — 2026-10-07

The native bundle builder now stages generated XML and Markdown sidecars alongside
Runtime, Data, Networking and Web. It validates the XML assembly name and requires
the generated Markdown manifest before publishing bundle.json. The bundle manifest
associates documentation with assembly names and hashes nested relative paths.
Relocation preserves those directories, verifies all hashes and tests native symbol
help plus ordinary HTTP project execution.

This follows the familiar .NET adjacent-sidecar convention, while Raven also supports
Markdown. No reflection fallback, metadata encoding or runtime change is introduced.
Raven's existing native project documentation writer remains responsible for generation.
The four-project build still finalizes the seed against the exact Runtime artifact;
Core/seed ownership and Object/async selections are unchanged.

## Results

All 26 installed VS Code checks passed on macOS arm64 (VS Code 1.140.0, development
extension 0.1.13), including the actual IPAddress summary from the shipped sidecar.
Orders and Tasks/await retain exact output and exit 0. The relocated native workspace
probe and HTTP executable also pass, with all 329 bundle files hash-verified.
Raven's probe-only revision is `ce51cd941`; the compiler/server remain at e93fcfdc1.
[Machine-readable evidence](native-bundle-documentation-2026-10-07.json).

## Validation

Use the existing build and relocation commands from
[native bundle configuration](native-bundle-configuration-2026-10-07.md), selecting a
fresh output directory. The extended editor command is:

```sh
python3 scripts/prepare-native-editor.py --sdk "$SDK" --bundle "$BUNDLE" \
  --runtime "$RUNTIME" --output "$OUT"
code --new-window --user-data-dir "$PROFILE" --extensions-dir "$EXTENSIONS" \
  --disable-extensions --skip-welcome --skip-release-notes --disable-workspace-trust \
  --extensionDevelopmentPath="$RAVEN/src/Raven.VSCode" \
  --extensionTestsPath="$NEOCLR/scripts/native-vscode-acceptance.cjs" \
  "$OUT.code-workspace"
```

The native bundle C# probe checks IPAddress.GetDocumentationComment after relocation;
the real VS Code test checks the same summary through its hover provider. Existing
editor behavior, replacement/configuration recovery, orders and Tasks execution remain
part of that run. The matching compiler/server are based on Raven e93fcfdc1; only the
C# acceptance probe changes in Raven. The server reuses the previously qualified local
build (including its explicit reused macro dependency), not a clean release SDK.

## Remaining coverage

The generated XML contains 136 Runtime, 0 Data, 41 Networking and 136 Web entries.
These include synthesized members and are not a public-API coverage measure. Generated
sidecars represent existing source comments; independently authored website help is
not silently copied onto declarations it may not match. Data help, missing union type
summaries, canonical attribution of external documentation and a unified RavenDoc
model remain open. No website snapshot refresh is needed for unchanged API signatures;
the website development status and API maintenance backlog are updated.

The companion JSON records exact bundle hashes, build/relocation output and editor
results. No runtime/compiler performance improvement is claimed.
