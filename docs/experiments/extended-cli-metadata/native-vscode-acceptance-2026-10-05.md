# Native VS Code POC acceptance — 2026-10-05

The bounded native editor gate passes in real VS Code 1.140.0 on macOS arm64.
This uses a development extension and adapter-enabled Raven compiler/language server,
not the published Preview 11 bundle. The companion JSON records revisions, exact
artifact/source/configuration hashes, output and test results.

## Observed acceptance

The extension-host test invokes VS Code's public hover/completion/definition APIs,
changes editor buffers, replaces actual files and executes real VS Code tasks.
It is not a mock client or solely a direct stdio test.

- Native imported method completion and hover.
- Read-only metadata declaration navigation, with the selected member's signature.
- Actual native library replacement refreshes symbols; unsaved consumer text survives.
- A deleted native reference produces a project diagnostic; restoring it clears the error.
- Unchanged `application-order-collections` imports the source-built `Numbers` and
  `Http` libraries with their sources absent from the application project.
- Native Option/Result type hovers retain union kind and constructed type arguments.
- The configured build task emits a native PE assembly using the evaluated project.
- A VS Code run task executes the native runtime and matches the entire checked-in
  collections output, exit 0 and empty stderr, including shared object identity.
- A failed build leaves the previous successful artifact unchanged.
- Ordinary .NET hover and diagnostics work in another workspace root.
- The unchanged `library-async` sample builds and executes through a VS Code task,
  printing `Suspended` and `42`, exit 0, empty stderr.

The replacement test library is itself compiled twice from Raven source in separate
projects outside the consumer workspace, then consumed solely as a native artifact.
The source-built class-library artifacts and retained seed reuse the previously
validated bootstrap ownership configuration; this does not claim a new full System
source bootstrap. Compiler/server dependencies run on .NET; the emitted applications
execute on NeoCLR.

## Reproduce

First build Raven's native compiler/server with the matching independent metadata project,
then compile the development extension:

```sh
dotnet build "$RAVEN/src/Raven.LanguageServer" -f net10.0 \
  -p:WarningLevel=0 -p:NeoClrMetadataProject="$METADATA"
npm --prefix "$RAVEN/src/Raven.VSCode" ci
npm --prefix "$RAVEN/src/Raven.VSCode" run compile
python3 scripts/prepare-native-editor.py --raven "$RAVEN" \
  --core "$CORE" --seed "$SEED" --ownership "$OWNERSHIP" \
  --reference "$NUMBERS" --reference "$HTTP" --runtime "$RUNTIME" --output "$OUT"
code --new-window --user-data-dir "$PROFILE" --extensions-dir "$EXTENSIONS" \
  --disable-extensions --skip-welcome --skip-release-notes --disable-workspace-trust \
  --extensionDevelopmentPath="$RAVEN/src/Raven.VSCode" \
  --extensionTestsPath="$NEOCLR/scripts/native-vscode-acceptance.cjs" \
  "$OUT.code-workspace"
```

Use absolute paths and fresh OUT/PROFILE/EXTENSIONS directories. METADATA is
`tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj`.
CORE, SEED, OWNERSHIP, NUMBERS and HTTP are explicit matching bootstrap artifacts;
the tested paths and hashes are in the JSON. The fixture selects the existing Numbers
async provider. The scripts do not reconstruct the full bootstrap or download artifacts.

The preparation script creates Native, DotNet and Async roots, normal `.rvnproj` files,
VS Code build/run tasks and two independently built versions of EditorLibrary.
The test's captured run task uses the same `rvnc neoclr --project ... --run ...` command
as the prepared Run task, redirecting output to assert exact stdout/stderr. The normal
Build task is executed directly. Check `vscode-acceptance.json` for `passed: true`;
script/process termination alone is not success. Tests use bounded waits and terminate
individual tasks that exceed their limit. Temporary workspace changes are isolated.

Client lifecycle/request logs are under PROFILE/logs in the Raven output channel log.
Server logs are in Raven's `logs/raven-lsp.log`; startup records identify the isolated
language-server copy actually used by VS Code. Relevant excerpts accompany the JSON.
An initial test failure came from JSON-serializing VS Code MarkdownString as `{}`;
reading its public `value` fixes the assertion without changing hover behavior.

## Scope and follow-up

`rvnc neoclr --project` uses the same explicit provider/catalog as the server. The
existing manifest parser is shared with the host project layer; bootstrap ownership,
async mappings and intrinsic authorization are explicit properties. Emission consumes
compiler symbols and host-supplied artifact identities, independently of importer objects.
No runtime instruction or metadata format change is needed.

Compared with ordinary .NET projects, native projects currently require artifact
HintPaths and explicit bootstrap/seed configuration. ProjectReference, PackageReference
and FrameworkReference builds remain unsupported in this bounded native path. Declaration
navigation shows metadata signatures, not recovered source or decompiled bodies. Initial
project-load errors retain existing host logging; watched-file failures are surfaced as
project diagnostics. External references outside workspace roots need watcher coverage
or a reload. IDE Run/Build **tasks** are qualified here; the extension's generic Raven
run/debug commands still use its existing .NET frontend, and native debugging is not claimed.

Next release work: a matched distributable toolchain and installation acceptance,
then updated downloads/setup instructions and manual publication. Other OSes, full
class-library source ownership, full decompilation and broader project orchestration
are not implied by this POC. The website's published bridge instructions remain intact.

Validated Raven revision: `16ec8e8156a94527d41bbeb5fbbf8fd61b1bcf1f`; NeoCLR runtime/metadata base
`aecb75f050b3ae677bac04637fb68c1ce552776f`. [Machine-readable evidence](native-vscode-acceptance-2026-10-05.json).
