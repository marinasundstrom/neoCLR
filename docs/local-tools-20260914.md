# Local Raven/neoCLR tools — 2026-09-14

The experimental SDK and VS Code extension `0.1.12-neoclr.14` are installed locally
on macOS arm64. This is a validated local candidate, not a published preview.
Published Preview 6 artifacts remain unchanged.

## Try it in VS Code

Open the fresh demo directory:

```sh
code /Users/robert/.neoclr/experiments/arrays-20260914/demo
```

If VS Code was already open, run **Developer: Reload Window**. The project settings
select the installed experimental SDK and extension language server. Existing demo
files and the normal SDK selection were preserved.

Edit `Main.rvn`, save, then run **Tasks: Run Task** and select the neoCLR run task.
The matching runtime, core reference metadata and library are already configured.
The initial program demonstrates Result/Option propagation and Void results.

The bundle's `tools/samples` directory contains additional examples, including
`library-array-foreach.rvn`, `library-array-callbacks.rvn` and
`library-managed-array-metadata.rvn`. Copy a sample into the demo's `Main.rvn` to try it.
Use `values[index]` for indexers, `Array<int>.Empty` for an empty array, and
`values.ForEach(action)` for callbacks. Indexers no longer appear as named `Item`
properties in dot completion; `[index].` offers the element's members.

## Installed components and provenance

- SDK: `/Users/robert/.raven/sdk/0.1.12-neoclr.14`
- Extension: `/Users/robert/.vscode/extensions/raven.raven-vscode-0.1.12-neoclr.14`
- Runtime bundle: `/Users/robert/.neoclr/experiments/arrays-20260914`
- Packages and validation logs: `/Users/robert/.neoclr/builds/arrays-20260914`
- neoCLR source: `a71e642829e4cbf00f3670fd8360644c810335e7`
- Raven experiment: `ee7b2e5af5412ff7af72658fe6de289e7d41ab6b`
- Host SDK: `.NET 11.0.100-rc.1.26425.128`

The [machine-readable record](experiments/raven-target/local-toolchain-20260914.json)
includes package SHA-256 hashes. All 751 files listed by the bundle manifest were
verified after configuration and testing.

## Validation

The rebuilt packages passed:

| Check | Outcomes checked |
| --- | ---: |
| Saved projects, including rejection cases | 63 |
| Application/GC/dispatch scenarios | 15 |
| Queries and expected faults/rejections | 30 |
| Installed compiler target integration | 5 |
| Bundled language server | 68 |
| Installed extension language server | 68 |
| Importer signatures | 121 |
| Generic-array compile/import/execute programs | 4 |
| Direct neoIL programs | 4 |

Both language-server runs cover completion, hover, diagnostics and signature help,
including Array<T>.Empty, instance ForEach, invalid `.Item.` and valid `[index].`
completion. This exercises the stdio server used by VS Code; it is not a claim of a
manual UI walkthrough. Dependency checks covered 26 NuGet dependencies and verified
all 34 packaged notice sets.

Before packaging, Raven main's empty-array fix passed 93 focused checks, including
.NET 10/.NET 11 reference metadata with and without Array.Empty. The experimental
synchronization passed 120 focused checks. General fixes are on Raven main through
`f70ba5026`; experimental target contracts remain on
`codex/neoclr-namespace-metadata`.

Follow the [experimental release procedure](experiments/raven-target/RELEASING.md)
for a later public build. This local validation does not replace a new preview's
release gate or claim complete CLR compatibility. Runtime-library migration remains
paused while the target/importer stabilization work is evaluated.


## Published Preview 7 installation

Installed the published Preview 7 runtime bundle at
`/Users/robert/.neoclr/experiments/preview7-20260914`, retaining the existing Raven
SDK/VSIX `0.1.12-neoclr.14` and normal SDK selection. All 765 runtime manifest entries
matched; the installed MSBuild demo built and produced the expected propagation output.
Existing experiment folders and edited demos were preserved.

Open the primary project:

```sh
code /Users/robert/.neoclr/experiments/preview7-20260914/msbuild-demo
```

Use **neoCLR: Build with MSBuild** or **neoCLR: Run (MSBuild)** from Tasks.
The separate-library example is in `project-reference-demo/App`. The matching metadata
includes `Type.IsValueType`. See the [published release](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.7)
for portable installation instructions and validation evidence.
