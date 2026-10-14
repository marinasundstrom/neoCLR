# Structural types: local VS Code test kit

Open `StructuralTypes.code-workspace` from the prepared kit. Install its
`raven-structural.vsix` with **Extensions: Install from VSIX**, then reload the
window. Workspace settings select the matching compiler and language server;
.NET 10 and Python 3 must be available on PATH. The runtime is macOS arm64.

Use **Tasks: Run Task** and select **Run Unions**, **Run Intersections**, or
**Run ExternalBounds**. Each task builds your edited source and executes it with
the bundled interpreter. Successful runs print a confirmation and exit zero.
The same tasks are available from a terminal: `python3 run.py Unions` (or the
other sample name). Use these explicit tasks for this experimental kit.

- **Unions**: native `Task<Response | HttpError | JsonError>`, completed and queued
  await for all alternatives, plus structural introspection.
- **Intersections**: `Animal & Quacks` inputs/results, dispatch, generic arguments
  and `IntersectionTypeInfo`.
- **ExternalBounds**: `T: Reads<int> & Adds` using interfaces from **Contracts**.
  Its task rebuilds Contracts first. Remove `Adds` from `Answer` to try a negative
  case: having an `Extra` method alone does not satisfy the interface bound.

This is a development snapshot of the feature branches, not a published SDK.
Interfaces remain nominal; full open structural generic operands and JIT/AOT
qualification remain outside this POC. The kit uses copied library artifacts,
not whichever runtime or SDK is installed globally.

## Preparing another kit

Publish Raven.Compiler and Raven.LanguageServer with the explicit
`NeoClrMetadataProject` property into `tools/rvnc` and `tools/language-server` in
an output directory. Build neoCLR with `experimental-compositions`, then run
`prepare.py --output <directory> --library <matching-native-library-bundle>`.
Package the matching Raven VS Code extension as `raven-structural.vsix` in that
directory. The preparation script copies the existing tested source examples,
adds the external-bound example and writes the project/workspace/task files.

## Validation (2026-10-14)

All three kit tasks compiled and executed successfully with exit zero. The external
bound negative case produced RAV0320 after removing the explicit `Adds` declaration.
An isolated VS Code extension test host opened all three sources, returned function
hover information and reported no error diagnostics. See `editor-check.json` and
`build-evidence.json` for results and exact tool hashes. This checks editor loading,
hover and diagnostics; terminal execution was validated separately through the same
`run.py` commands used by workspace tasks.
