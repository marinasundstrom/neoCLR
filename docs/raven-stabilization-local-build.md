# Try the stabilized Raven/neoCLR build

The local experimental SDK and VS Code extension are `0.1.12-neoclr.9`. They include
array iteration and query support, corrected numeric operations and conversions,
and neoCLR's invariant mutable-array contract. This is an installed local build,
not a newly published release.

## Open the prepared demo

```sh
code --new-window \
  --user-data-dir /Users/robert/.neoclr/vscode/stabilization-20260913 \
  --extensions-dir /Users/robert/.neoclr/vscode/stabilization-20260913/extensions \
  /Users/robert/.neoclr/experiments/stabilization-20260913/demo
```

Open `Main.rvn`, save edits, then choose **Terminal → Run Task → neoCLR: Run saved
project**. The configured task compiles the saved Raven source, imports and verifies
it, then executes it on neoCLR. Use **neoCLR: Build saved project** to omit execution.
The ordinary Raven toolbar Run/Debug commands are not the neoCLR task pipeline.

The [order workflow](raven-order-workflow.md) demonstrates shared class instances,
interface calls, Option/Result patterns, Void propagation, bounded file writes and
a deferred query. Its output is:

```text
Saved
Order report exceeds limit
Order not found
Saved
Queued
Order: Coffee
Pending orders
Tea
```

The report `neoclr-orders.txt` is written in the demo folder. The failed second write
leaves Tea queued. Change `Process(store, 8, 2)` to `Process(store, 8, 64)` to allow
both saves; the final pending list becomes empty.

For smaller experiments, examples are in `../tools/samples`, including
`library-array-queries.rvn`, `library-numeric-widening.rvn` and
`library-numeric-operators.rvn`. Save any work you want to keep before replacing
Main.rvn. Completion is provided by the bundled language server and neoCLR metadata.

The SDK is installed alongside existing versions at
`/Users/robert/.raven/sdk/0.1.12-neoclr.9`. The workspace selects it explicitly.
The default SDK, old isolated extensions and existing demos were preserved; hashes
of 315 existing Raven source files were unchanged after installation.

## Validation and boundaries

All six packaged suites passed: saved projects (57 checks), queries (28), application
behavior, order workflow, direct NeoIL samples and editor completion/hover. All 701
pristine bundle payload hashes were verified, including in the archive. The installed
SDK reports .9, VS Code lists `raven.raven-vscode@0.1.12-neoclr.9`, and the exact
configured Run task passed with the expected output and report content. Editor tests
exercise its LSP protocol; this is not a claim of full source-debugger support.

[Build evidence](experiments/raven-target/stabilization-toolchain.json) records source
revisions, artifact hashes, paths and results. Artifacts and logs are under
`/Users/robert/.neoclr/builds/stabilization-toolchain-20260913`. This build uses .NET
SDK `11.0.100-rc.1.26425.128` on macOS arm64 and requires the matching host runtime.

[Mutable arrays are invariant](array-variance.md). Raven's editor still follows CLR
array conversion rules, so some conversions accepted while editing are rejected by
the neoCLR importer. Target-aware diagnostics and covariant read-only projections
remain follow-ups. Query cleanup on early exit/fault, rectangular arrays and broader
runtime/compiler compatibility retain their documented limitations.
