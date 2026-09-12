# Test Raven targeting neoCLR in VS Code

This setup provides project-backed completion against neoCLR's declaration assembly.
Dedicated neoCLR tasks now compile the saved project, import it, verify it and run it. The normal Raven
**Build**, **Run** and **Debug** buttons do not yet implement that target pipeline.
Do not use their success as evidence that a program ran on neoCLR.

## Already prepared on this machine

The current local experiment uses `raven.raven-vscode@0.1.12-neoclr.3` and the
side-by-side SDK `/Users/robert/.raven/sdk/0.1.12-neoclr.3`, built from Raven commit
`1e3f7ff07d8a9785ed54105b74d1fcda96c8795f` on `codex/neoclr-target-resolution`.
These are local experiment labels, not published releases. The default SDK launcher
remains unchanged; the new workspace selects the experimental SDK and installed server.

Open the prepared folder:

```sh
code --new-window /Users/robert/Projects/neoclr/docs/experiments/raven-target/local/2026-09-12-fundamentals-3/editor
```

1. Run **Developer: Reload Window** if VS Code was open during the update.
2. Open `Main.rvn`, containing the combined product workflow.
3. Choose **Terminal → Run Task → neoCLR: Run saved project**.
4. Expect `42`, `Completed`, `Price overflow`, `Skipped`, `Product not found`,
   `Skipped`.
5. Change `products.Add(99)` to `products.Add(7)`, save, and run the task again.
   The final missing-product path should become `42`, `Completed`.
6. Try completion after `System.Math.` or `products.`; undo any unfinished test
   expressions before running. Hover over the loop variable declaration to see `int`.

The workspace settings pin the installed extension's language server. The project uses
only the neoCLR declaration assembly, while executable calls use the adapted runtime
library. Host tooling still requires .NET 11. The SDK is a Raven host toolchain;
neoCLR execution uses the dedicated tasks, not Raven's normal Build/Run/Debug buttons.

The installed server passed the stdio completion/hover checks, and the saved workflow
compiled, verified and ran successfully on 2026-09-12. The wider pre-merge runtime
suite found generic Clonable and calculator regressions, subsequently fixed and
validated across all integration suites; this local demo is not a claim that the branch is release-ready. Propagation and match syntax release coverage
are still pending.

## Edit, build and run the saved project

Requires Raven commit `1e3f7ff07d8a9785ed54105b74d1fcda96c8795f` or the corresponding
source build. The runner uses that checkout's compiler API; the independently installed
SDK is not the target build backend yet.

1. Edit `Main.rvn`, for example changing `products.Add(99)` to `products.Add(7)`, and **save** it.
2. Choose **Terminal → Run Task → neoCLR: Run saved project**.
3. The task compiles the project's saved Compile items against its declared core, applies
   the target Void projection, imports the bounded IL, verifies it and executes it.
   The edited workflow handles the final product successfully rather than printing Product not found.
4. **neoCLR: Build saved project** performs the same steps except execution.

The tasks are already configured in the prepared folder. They are separate from Raven's
standard toolbar commands. Each attempt writes a fresh `.neoclr-build/build-*/output`
directory and reports its verified IL path. Compile, import and verifier failures stop
execution; a failed attempt never runs an older artifact. Generated directories can be
removed when no task is using them. Unsaved buffers are not build inputs.

From a terminal, use the same runner:

```sh
python3 docs/experiments/raven-target/run_project.py /absolute/editor/Demo.rvnproj --raven /absolute/path/to/Raven --runtime /absolute/path/to/neoclr
```

Build neoCLR with `cargo build --locked` first, or point `--runtime` to an existing build.
The runner uses the pinned SDK in the bridge folder and the existing built Raven compiler;
rebuild Raven after changing its checkout. To add/update tasks in an existing folder:

```sh
python3 docs/experiments/raven-target/configure_tasks.py /absolute/editor/Demo.rvnproj --raven /absolute/path/to/Raven --runtime /absolute/path/to/neoclr
```

Preparation also creates these tasks; pass `--runtime` to choose the executable (default:
neoCLR's `target/debug/neoclr`). Unrelated task entries are preserved. The current project
profile accepts exactly the supplied core reference and no project references. It reuses
the Result/Option/Void importer: this is not general Raven IL support. Some declarations
visible in completion are still outside that execution profile, beyond the documented catalog.
Unsupported instructions/APIs fail admission instead of falling back to host execution.
The edit/build/run milestone is closed. The bounded interface/iteration contract now works; union propagation remains deferred.

`verify_project.py` exercises the three union demos, a saved edit, a compiler failure,
and an unsupported-instruction failure without overwriting the user's project.

## Rebuild and install

Prerequisites: the pinned .NET SDK from `global.json`, Node/npm, the VS Code `code`
command, and a Raven checkout with the commit above. From the Raven repository:

```sh
RAVEN_PACKAGE_OUTPUT="$PWD/artifacts/neoclr-local" scripts/package-vscode.sh 0.1.12-neoclr.3
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
RAVEN_PACKAGE_OUTPUT="$PWD/artifacts/neoclr-local" scripts/package-sdk.sh osx-arm64 0.1.12-neoclr.3
```

To install that bundle alongside existing SDKs (the destination must not exist):

```sh
mkdir -p "$HOME/.raven/sdk"
cp -R artifacts/neoclr-local/raven-sdk-0.1.12-neoclr.3-osx-arm64 "$HOME/.raven/sdk/0.1.12-neoclr.3"
"$HOME/.raven/sdk/0.1.12-neoclr.3/bin/rvnc" --version
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

## Collection project follow-up

This follow-up needs Raven commit `1e3f7ff07d8a9785ed54105b74d1fcda96c8795f`
on `codex/neoclr-target-resolution` (or a compatible descendant). The installed experimental extension above includes project iteration configuration.
For a source-only setup instead, build the current server and let generated workspace
settings select it. This does not change global .NET settings.

From the Raven repository:

```sh
dotnet build src/Raven.CodeAnalysis/Raven.CodeAnalysis.csproj -p:WarningLevel=0
dotnet build src/Raven.LanguageServer/Raven.LanguageServer.csproj -f net11.0 -p:BuildProjectReferences=false -p:WarningLevel=0
```

From the neoCLR repository, choose a fresh output directory and substitute your Raven path:

```sh
cargo build --locked
dotnet run --project docs/experiments/raven-target/Probe.csproj -p:RavenRoot=/absolute/path/to/Raven -p:BuildProjectReferences=false -p:WarningLevel=0 -- --interfaces /tmp/raven-collections-editor
python3 docs/experiments/raven-target/prepare_editor.py /tmp/raven-collections-editor /absolute/path/to/Raven --collections --runtime "$PWD/target/debug/neoclr"
code --new-window /tmp/raven-collections-editor/editor
```

If using CARGO_TARGET_DIR, supply its executable path to `--runtime` instead.
Open Main.rvn and run the generated neoCLR build/run task. Expected output is
`42`, `Completed`, `Price overflow`, `Skipped`, `Product not found`, `Skipped`,
with no runner return-value suffix. The sample looks up products with Option, validates a price with
Result, and returns an Option<Void> completion marker. The project selects Iterable/Iterator through
RavenIteration* properties; the task generates and uses the matching adapted runtime
library for each build. On a List<int> parameter, completion should offer Add, Count,
GetIterator and Item. Hover over the loop variable declaration to see `int`.
The normal Raven build/run buttons still do not invoke this import pipeline.

Repeat the automated saved-project and stdio language-server checks:

```sh
python3 docs/experiments/raven-target/verify_project.py /tmp/raven-collections-editor/editor/Demo.rvnproj --collections --raven /absolute/path/to/Raven --runtime "$PWD/target/debug/neoclr"
python3 docs/experiments/raven-target/verify_editor.py /tmp/raven-collections-editor/editor --collections
```

The first checks executable integer Math.Min/Max/Sign, the combined workflow,
Result/Option/Void, value copying, array and
class aliasing, loops, a saved edit and failed-build stale-output protection.
The second checks target completion and the inferred loop binding through the actual
language server. It changes the in-memory document and retains a local LSP transcript;
it does not edit the saved sample. The collection declaration profile now includes Result/Option/Void, so all of these
examples use the same declaration assembly and adapted library. Existing non-collection
projects retain their smaller profile; regenerate older collection demo folders to
obtain the added declarations. Automatic disposal and defer remain future work, with
NeoCLR-specific behavior isolated from existing .NET/CLR support.

Normal runs print only guest output. For runtime diagnostics, `neoclr run <input> --show-result` writes the return value to stderr.
