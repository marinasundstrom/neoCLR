# Test Raven targeting neoCLR in VS Code

This setup provides project-backed completion against neoCLR's declaration assembly.
The Result demo executes through the separate neoCLR import probe. The normal Raven
**Build**, **Run** and **Debug** buttons do not yet implement that target pipeline.
Do not use their success as evidence that a program ran on neoCLR.

## Already prepared on this machine

A local extension build, `raven.raven-vscode@0.1.12-neoclr.1`, is installed. This is an
experiment label, not a published release. Its bundled compiler library and language
server include Raven commit `37ae9730409d52f876b6b6e47abfa950d8300064` from
`codex/neoclr-target-resolution`. A separate SDK bundle is installed alongside the
existing SDK at `/Users/robert/.raven/sdk/0.1.12-neoclr.1`; its `rvnc --version` reports
`0.1.12-neoclr.1`. The demo's settings select this SDK without changing global PATH
or the default SDK symlink.

Open the prepared folder:

```sh
code --new-window /Users/robert/Projects/neoclr/docs/experiments/raven-target/local/2026-09-12-editor/editor
```

1. If the window predates installation, run **Developer: Reload Window**.
2. Open `Main.rvn`. It contains the Result-based Math.Abs example.
3. Inside `Main`, temporarily add a line and type `System.Math.`. The member suggestions
   should include **Abs, Max, Min, Sign**. Delete/retype the dot if a pasted fragment
   does not trigger suggestions. An unfinished expression will produce diagnostics.
4. Try `System.Console.`: **WriteLine** should appear; **ReadLine** should not.
5. Try `System.`: **Option**, **Result** and **Void** should appear; **IO** should not.
6. Undo those test edits to restore the runnable example.

In **View → Output → Raven**, check the extension version and language server path.
The folder's `.vscode/settings.json` pins the installed experimental server, avoiding
an older SDK/server selected elsewhere. This affects this workspace only. The generated
`Demo.rvnproj` supplies only `NeoCLR.CoreProbe.dll`; that is a metadata declaration
artifact, not the runtime implementation. Host tools still run on .NET 11.

The local UI check on 2026-09-12 displayed all four Math methods. The client log recorded
`textDocument/completion` at `2:16`, `items=4`; server startup identified the installed
extension's server. The reproducible protocol check also covers Console and System.
See [editor-results.json](editor-results.json) for the bounded evidence.

## Rebuild and install

Prerequisites: the pinned .NET SDK from `global.json`, Node/npm, the VS Code `code`
command, and a Raven checkout with the commit above. From the Raven repository:

```sh
RAVEN_PACKAGE_OUTPUT="$PWD/artifacts/neoclr-local" scripts/package-vscode.sh 0.1.12-neoclr.1
code --install-extension "$PWD/artifacts/neoclr-local/raven-vscode.vsix" --force
code --list-extensions --show-versions
code --locate-extension raven.raven-vscode
```

This replaces the installed Raven extension with the local build and bundles the
updated language server. It does not publish anything. To return to a published build,
use VS Code Extensions → Raven → **Install Another Version**, selecting the desired
published version. Other projects retain their normal reference policy.

For a separate local SDK bundle, the repository also supports:

```sh
RAVEN_PACKAGE_OUTPUT="$PWD/artifacts/neoclr-local" scripts/package-sdk.sh osx-arm64 0.1.12-neoclr.1
```

To install that bundle alongside existing SDKs (the destination must not exist):

```sh
mkdir -p "$HOME/.raven/sdk"
cp -R artifacts/neoclr-local/raven-sdk-0.1.12-neoclr.1-osx-arm64 "$HOME/.raven/sdk/0.1.12-neoclr.1"
"$HOME/.raven/sdk/0.1.12-neoclr.1/bin/rvnc" --version
```

Set the demo's `raven.sdkPath` to that directory, or pass `--sdk /absolute/sdk-directory`
to `prepare_editor.py`. Do not replace an existing SDK directory while testing a new
build; use another experiment version or directory.

That SDK remains a Raven host toolchain; it does not turn the experimental neoCLR target
into a general SDK build target. The extension's bundled server is sufficient for this
completion test. No global SDK replacement or PATH change is necessary.

## Regenerate the editable project

First run the [emission and runtime probe](README.md#reproduce) into a **new** output
directory. Then run from the neoCLR repository root:

```sh
python3 docs/experiments/raven-target/prepare_editor.py /absolute/probe-output /absolute/path/to/Raven --server /absolute/installed-extension/server/Raven.LanguageServer.dll
python3 docs/experiments/raven-target/verify_editor.py /absolute/probe-output/editor
code --new-window /absolute/probe-output/editor
```

Use the extension directory printed by `code --locate-extension raven.raven-vscode`.
Omitting `--server` selects Raven's built Debug/net11.0 server instead. Preparation
refuses to overwrite an existing editor folder. It copies the sample and core reference,
so edits are local to that folder.

`verify_editor.py` launches the configured server over stdio, opens the actual project,
sends open/change/completion requests, and checks target member/namespace suggestions.
It writes `lsp-transcript.json` and `lsp-stderr.log` in that folder; the server additionally
writes `logs/raven-lsp.log`. This is a headless protocol test, independent of the manual
VS Code check above. It does not change the saved sample text.

## Run the runtime demonstration

From the neoCLR root:

```sh
cargo run -- run docs/experiments/raven-target/local/2026-09-12-editor/CoreUnion.neoil
```

Expected output is `42`, `Overflow`, and the CLI result display `=> Void`.
The import uses the real System.Math and Result library. This saved IL corresponds to
the sample at probe generation time. Editing `editor/Main.rvn` does **not** rebuild it.
For now, change the probe's `samples/library-result.rvn` and rerun the probe into a fresh
output directory to test changed code within the admitted subset. Connecting arbitrary
project edits to this importer is the next development-loop slice.
