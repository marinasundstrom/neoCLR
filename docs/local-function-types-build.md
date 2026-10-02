# Local Function types build — 30 September 2026

The matching Raven SDK and VS Code extension are
`0.1.12-neoclr.20260929.functions1`. Packaging started on 29 September and the
local installation was validated on 30 September. This is a development build,
not a published release.

## Open and run

```sh
~/.neoclr/experiments/development-20260929-functions/open-vscode.sh
```

The launcher opens an isolated VS Code workspace. Start with
`playground/Main.rvn`, save your changes, then choose **Terminal → Run Task →
neoCLR: Run** for the playground folder. The task builds, imports, verifies and
runs saved source. Use these tasks rather than the ordinary Raven toolbar Run/Debug
commands, which do not select the neoCLR pipeline.

The playground demonstrates signature-based callbacks, value equality, target
MethodInfo, ToString, IsFunctionType and an Object round trip. Its output is:

```text
42
42
True
False
AddOne
[Contracts]AddOne(System.Int32) -> System.Int32
True
42
```

The workspace also includes editable `callbacks`, `introspection` and `oftype`
contract projects, each with Build and Run tasks. Their successful output reports
that their contracts passed. Existing SDK selections, profiles and editable demos
were preserved.

## Installed files and evidence

- SDK: `~/.raven/sdk/0.1.12-neoclr.20260929.functions1`.
- Runtime, reference, library, bridge and workspace:
  `~/.neoclr/experiments/development-20260929-functions`.
- Extension package: `~/.neoclr/builds/functions-20260929/raven-vscode.vsix`.
- Isolated VS Code profile: `~/.neoclr/vscode/functions-20260929`.
- Workspace settings explicitly select the installed SDK and its language server.

[Validation record](experiments/function-types/local-toolchain-validation.json)
records source revisions and artifact hashes. All 440 installed SDK files and 90
extension payload files match their packages; extension manifest version matches.
The bundle's recorded payload hashes match. All four projects build and execute
using the installed standalone MSBuild pipeline. The installed language server
passes completion and hover checks for Function objects, FunctionTypeInfo, TypeInfo
and NominalTypeInfo over stdio LSP. This is protocol validation, not a claim that
every editor feature was visually tested.

Build logs and the local validation record are in the bundle's `validation/` folder
and `local-validation.json`. No release publication, platform matrix or website
rebuild was needed for this local installation.
