# Try the query-enabled Raven build locally

For the newer .9 build, use [the stabilization installation](raven-stabilization-local-build.md). The .8 record below is historical.

The local SDK and VS Code extension are `0.1.12-neoclr.8`. This build includes union
patterns, extension methods, deferred Where/Select, ToList and custom Raven
Iterable/Iterator implementations. It is a local experiment, not a published release.

Open the isolated environment:

```sh
code --new-window \
  --user-data-dir /Users/robert/.neoclr/vscode/queries-20260913 \
  --extensions-dir /Users/robert/.neoclr/vscode/queries-20260913/extensions \
  /Users/robert/.neoclr/experiments/queries-20260913/demo
```

Open `Main.rvn`. Type `numbers.` or `query.` with `import System.Linq.*` in scope to
see `Where`, `Select` and `ToList`. Choose **Terminal → Run Task → neoCLR: Run saved
project** to compile the saved file and execute it. **neoCLR: Build saved project**
compiles and verifies without running. Save edits before using these tasks.

The initial sample shows that callbacks do not run at query construction, repeated
Current reads reuse the selected value, and materializing the query enumerates it
again. Its output starts with two zeroes and ends with `5` and `3`, the predicate and
selector invocation counts. [The API contract](raven-query-api.md) explains the behavior
and remaining cleanup, mutation and inference limitations.

More examples are in the installation's `tools/samples` directory. Copy one into the
demo's `Main.rvn` to try it: `application-iterable.rvn` demonstrates custom interfaces;
`application-orders.rvn` demonstrates the workflow and union destructuring. Preserve
any edits you want to keep before replacing that file.

The SDK is installed at `/Users/robert/.raven/sdk/0.1.12-neoclr.8`. Project settings
select that SDK and the matching bundled language server. The earlier application
and pattern demos, their edits, the .7 installation, and the default Raven SDK
selection were preserved. If an already-open window has stale language service
state, use **Developer: Reload Window**.

## Build and verification record

The [machine-readable record](experiments/raven-target/query-toolchain.json) contains
source revisions, artifact hashes, installation paths and suite results. Compiler
fix: Raven `000ed511e` on `codex/neoclr-target-resolution`. Runtime/bridge source:
neoCLR `3ff57a3`. The build uses .NET SDK `11.0.100-rc.1.26425.128` on macOS arm64.

All six packaged suites passed: queries (14 cases), application behavior (12 cases),
order workflow, saved projects (50 cases), direct NeoIL samples and editor completion/
hover. The Raven regression selection passed 29 tests. All 685 pristine bundle payload
hashes were verified; the installed copy then received its local task/settings paths
and query demo. The installed compiler reports .8 and the isolated extension list
reports `raven.raven-vscode@0.1.12-neoclr.8`.

Artifacts are under `/Users/robert/.neoclr/builds/query-toolchain-20260913`: the SDK
archive, `raven-vscode.vsix`, neoCLR bundle archive, manifest and `validation` logs.
For future builds, follow [the experiment release procedure](experiments/raven-target/RELEASING.md).
