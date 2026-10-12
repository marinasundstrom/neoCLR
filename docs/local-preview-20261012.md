# Local default-JIT preview — 12 October 2026

Installed for author testing after the extracted bundle gates passed. This is a
macOS ARM64 development installation, not a published release or fresh Windows
qualification.

- SDK and VS Code extension: **0.1.13-neoclr.20261012.2**.
- SDK: `~/.raven/sdk/0.1.13-neoclr.20261012.2`.
- Runtime, compiler, libraries, editable samples and launch script:
  `~/.neoclr/experiments/preview-20261012-2`.
- VS Code uses dedicated `editor-state` and `editor-extensions` directories there.
  The ordinary VS Code installation still lists Raven 0.1.14; no global SDK default
  or shell PATH was changed.
- Raven native compiler: `6b5e052ce`; general compiler fix on main: `0ccb9a265`.
  Native object-budget fix: neoCLR `b1b43b35a`. Component and archive hashes are in
  the [installation record](experiments/native-type-budget/installed-preview/installation.json).

## Open and run

```sh
~/.neoclr/experiments/preview-20261012-2/open-vscode.command json
~/.neoclr/experiments/preview-20261012-2/open-vscode.command http-json-server
~/.neoclr/experiments/preview-20261012-2/open-vscode.command http-json-client
```

Each opens the selected sample in the dedicated editor setup. Choose **Tasks: Run
Task → neoCLR: Run**. Start the server with request count **2**, then start the
client with `http://127.0.0.1:PORT/`, using the server's printed port. The client
GETs a typed JSON report, POSTs it back and prints `{"accepted":true}`. The server
prints `Reports served` and exits. Run both tasks again to repeat. Ordinary Raven
.NET Run/Debug commands are a different target; use the neoCLR task.

The same operations work from a terminal:

```sh
cd ~/.neoclr/experiments/preview-20261012-2
python3 tools/run-native-sample.py json
python3 tools/run-native-sample.py http-json-server -- 2
# In another terminal, using the printed port:
python3 tools/run-native-sample.py http-json-client -- http://127.0.0.1:PORT/
```

The compiler can print informational propagation diagnostics and existing
unreachable-code warnings in these samples; successful execution is checked by exit
status and exact JSON output. Reflection preparation uses a temporary disassembly
and does not overwrite an existing caller-owned App.neoil.

## Qualification

The final archive was hash-checked and extracted outside the checkout to a path
with spaces. [All five packaged projects compile](experiments/native-type-budget/installed-preview/acceptance-default.json).
Collections, Tasks and JSON execute through default JIT. The [HTTP gate](experiments/native-type-budget/installed-preview/acceptance-default-http.json)
checks GET, POST, malformed JSON, missing routes and the Raven client/server pair.

Both [first](experiments/native-type-budget/installed-preview/installed-http-repeat-1.json)
and [second](experiments/native-type-budget/installed-preview/installed-http-repeat-2.json)
consecutive runs through the **installed sample runners** pass, including rebuilding
both projects. Installed catalogue files and the side-by-side SDK match the qualified
hashes. The VSIX installation/listing confirms the exact version. The installed SDK's
language server passes [Task and Promise completion probes](experiments/native-type-budget/installed-preview/editor-completions.json)
over stdio against installed library references. VS Code's CLI reports the opened
`neoCLR Preview — json` window; interactive extension activation was not independently
verified through that window.

The earlier interpreter control remains valid for unchanged compiler/runtime/library
inputs. Native layout boundary execution and sanitized GC checks are linked from the
[object-budget record](experiments/native-type-budget/README.md). This slice does
not establish fresh Windows, HTTPS or AOT HTTP qualification. Native heap objects
remain bounded at 256 payload lanes; inline values remain bounded at 128. Compiler
field-liveness optimization remains separate work.
