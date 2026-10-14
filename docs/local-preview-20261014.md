# Local Preview 15 installation — 14 October 2026

The qualified macOS ARM64 bundle is installed for author testing:

- SDK: `~/.raven/sdk/0.1.15-neoclr.20261014.2`.
- Bundle and editable samples: `~/.neoclr/experiments/preview-20261014-15`.
- VS Code extension: `0.1.15-neoclr.20261014.3`, installed in the bundle's dedicated
  `editor-extensions` directory with `editor-state` as its profile.

The SDK matches the qualified bundle. The ordinary SDK default and global editor
extension are unchanged. [Installation record](validation/release-20261014/installation.json).

## Run

```sh
~/.neoclr/experiments/preview-20261014-15/open-vscode.command json
~/.neoclr/experiments/preview-20261014-15/open-vscode.command http-json-server
~/.neoclr/experiments/preview-20261014-15/open-vscode.command http-json-client
```

Choose **Terminal → Run Task → neoCLR: Run**. For HTTP, start the server with request
count `2`, then run the client with the server's printed `http://127.0.0.1:PORT/`.
The client receives a JSON report, posts it back and prints `{"accepted":true}`.

From a terminal:

```sh
cd ~/.neoclr/experiments/preview-20261014-15
python3 tools/run-native-sample.py json
```

[Installed acceptance](validation/release-20261014/installed-acceptance.json) verifies
all 1,552 catalogue files, builds all five sample projects and executes collections,
Tasks and JSON with default JIT. [HTTP checks](validation/release-20261014/installed-http.json)
cover successful requests, malformed JSON, missing routes and the Raven client/server
pair. Both pass. The matching packaged editor passes configuration recovery,
member hover through edits and inlay stability on macOS and Windows.
