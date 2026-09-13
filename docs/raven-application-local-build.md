# Try the application toolchain locally

For the latest local .8 query/pattern build, use [the isolated query demo](raven-query-local-build.md).
Earlier installation records below remain specific to their stated builds.

For the newer source-backed destructuring demo, see [the pattern walkthrough](raven-match-matrix.md#local-pattern-demo).
It uses a separate folder and an updated language server; the .7 package described
below remains unchanged and contains the earlier typed-case samples.

The 2026-09-13 application build uses experimental Raven SDK/extension
`0.1.12-neoclr.7`. This is a local macOS arm64 build, not a new published preview.
It includes application classes/value types, interfaces, abstract/virtual dispatch,
Func method groups and capturing lambdas. See [application types](raven-application-types.md),
[delegates](raven-delegates-lambdas.md) and the [order workflow](raven-order-workflow.md)
for supported contracts and limits.

## Prepared installation on this machine

Open the dedicated environment:

```sh
code --new-window \
  --user-data-dir "$HOME/.neoclr/vscode/application-poc-20260913" \
  --extensions-dir "$HOME/.neoclr/vscode/application-poc-20260913/extensions" \
  "$HOME/.neoclr/experiments/application-poc-20260913/demo"
```

The extension is installed only in that environment. The SDK lives alongside existing
SDKs at `$HOME/.raven/sdk/0.1.12-neoclr.7`; the global SDK selection is unchanged.
The project pins its supplied language server and neoCLR compiler bridge. Neither
source checkout is needed to edit and run it. .NET 11 and Python 3.9+ are required.

1. Open `Main.rvn`, prepared with the order workflow. If VS Code asks about trust,
   review and trust this local project to enable its language server and tasks.
2. Choose **Terminal → Run Task → neoCLR: Run saved project**. Use these dedicated
   tasks; the ordinary Raven Run/Debug buttons do not target this pipeline.
3. Expect the output below. The program writes `neoclr-orders.txt` in the working
   directory. A rejected second write leaves the first report intact.

```text
Saved
Order report exceeds limit
Order not found
Saved
Queued
Order: Coffee
```

Try completion after `System.Date.` or `System.Type.` inside a function, then remove
the incomplete expression and save before running. To explore another feature, copy
one of `../tools/samples/application-*.rvn` over `Main.rvn` and save. In particular,
`application-delegates.rvn` demonstrates method groups, shared mutable captures and
a callback returned from its creating function.

Terminal equivalent:

```sh
cd "$HOME/.neoclr/experiments/application-poc-20260913"
python3 tools/run_project.py demo/Demo.rvnproj \
  --bridge tools/bridge/Probe.dll --system lib/System.neoil --runtime bin/neoclr
```

## Constructor-completion hotfix

The prepared project now selects the patched language server at
`$HOME/.neoclr/builds/constructor-completion-hotfix/Raven.LanguageServer.dll`, built
from Raven `55c0f7ef5` on the experiment branch. Run **Developer: Reload Window**
if the open window still uses its previous server. Unsaved source edits are not
replaced by the installation.

The compiler now establishes constructor scopes for on-demand semantic queries;
`System.Int32.` offers Parse and Divide there, as it does in ordinary methods.
119 focused compiler tests and four target LSP contexts passed. The user's actual
client log recorded a completed request with zero items at `14:23`; the server startup
log confirmed the .7 server. The isolated replay reproduced both the empty constructor
result and the passing method result before the fix. This hotfix changes the installed
language-server selection, not the archived .7 SDK/VSIX or published Preview 4 assets.

## Build provenance and scope

[Application build evidence](experiments/raven-target/application-toolchain.json)
records source revisions, package checks and local artifacts. This build follows the
[experimental packaging procedure](experiments/raven-target/RELEASING.md); it is not
Raven's normal release. Published Preview 4 assets and its prepared demo remain intact.

Raven source debugging is future work. Generic application definitions, custom delegate
declarations and explicit/default application interface implementations remain outside
the bounded importer. The future injectable-clock API is a recorded design question;
this build still exposes the existing `Clock.GetLocalNow()` contract.
