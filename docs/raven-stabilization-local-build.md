# Try the stabilized Raven/neoCLR build

For the newer .11 SDK/extension, use [the generic-array installation](raven-generic-arrays-local-build.md). The .10 record below remains historical.

The local experimental SDK and VS Code extension are `0.1.12-neoclr.10`. They include
array iteration and query support, corrected numeric operations and conversions,
and neoCLR's invariant mutable-array contract, including target-aware editor
diagnostics. This is an installed local build, not a newly published release.

## Open the prepared demo

```sh
code --new-window \
  --user-data-dir /Users/robert/.neoclr/vscode/array-diagnostics-20260913 \
  --extensions-dir /Users/robert/.neoclr/vscode/array-diagnostics-20260913/extensions \
  /Users/robert/.neoclr/experiments/array-diagnostics-20260913/demo
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
`/Users/robert/.raven/sdk/0.1.12-neoclr.10`. The workspace selects it explicitly.
The default SDK, old isolated extensions and existing demos were preserved; hashes
of 899 existing Raven source files were unchanged after installation.

## Validation and boundaries

All six packaged suites passed: saved projects (57 checks), queries (28), application
behavior, order workflow, direct NeoIL samples and editor completion/hover plus
array diagnostic/recovery checks. All 702 pristine bundle payload hashes were
verified, including in the archive. The installed
SDK reports .10, VS Code lists `raven.raven-vscode@0.1.12-neoclr.10`, and the exact
configured Run task passed with the expected output and report content. Editor tests
exercise its LSP protocol; this is not a claim of full source-debugger support.

[Build evidence](experiments/raven-target/array-diagnostics-toolchain.json) records source
revisions, artifact hashes, paths and results. Artifacts and logs are under
`/Users/robert/.neoclr/builds/array-diagnostics-toolchain-20260913`. This build uses .NET
SDK `11.0.100-rc.1.26425.128` on macOS arm64 and requires the matching host runtime.

[Mutable arrays are invariant](array-variance.md). This build rejects differing
array element types while editing and compiling. The demo selects the policy with
`<RavenAllowArrayCovariance>false</RavenAllowArrayCovariance>`. Ordinary .NET projects
retain their existing array covariance behavior.

## Try the array diagnostics

After saving any work you want to keep, replace `Main.rvn` with:

```raven
import System.*
import System.Reflection.*
import System.Console.*

func Main() {
    let members: MethodInfo[] = typeof(int).GetMethods()
    for member in members {
        WriteLine(member.Name)
    }
}
```

Run the saved-project task to list methods. Change `MethodInfo[]` to `MemberInfo[]`:
VS Code should report that the array cannot be assigned. The explicit cast
`let members = (MemberInfo[])typeof(int).GetMethods()` should also report an error.
Restore `MethodInfo[]` to clear it. A prior diagnostic can briefly remain while the
language server rechecks an edit.

Covariant read-only projections remain a future feature. Query cleanup on early
exit/fault, rectangular arrays and broader runtime/compiler compatibility retain
their documented limitations.

The earlier `.9` installation remains in `stabilization-20260913` and retains its
[original build evidence](experiments/raven-target/stabilization-toolchain.json).
It does not gain these diagnostics automatically. Published Preview 5 assets are
unchanged.
