# neoCLR Preview 7 — project builds and compiler integration

Version: **0.1.0-preview.7** · Tag: **v0.1.0-preview.7** · Date: **2026-09-14**.

This preview makes standalone MSBuild the primary Raven project workflow, adds a
bounded application-plus-library example, and includes the reviewed compiler and
generic-array fixes since Preview 6. It retains neoCLR's own runtime library,
Result/Option error flow, generic Void, and familiar CLR value/reference categories.

## Install and try

Download the matching prerelease assets:

- `neoclr-0.1.0-preview.7-osx-arm64.tar.gz`
- `raven-sdk-0.1.12-neoclr.14-osx-arm64.tar.gz`
- `raven-vscode-0.1.12-neoclr.14.vsix`
- `raven-toolchain-notices.tar.gz`

Source archives, validation evidence, `release-manifest.json` and `SHA256SUMS`
accompany them. Prebuilt tools support **macOS arm64**. Raven/MSBuild require .NET SDK
**11.0.100-rc.1.26425.128**. Python 3.9+ is used for setup and validation, and VS Code
for editing. The Rust runtime does not require .NET. Source CI checks Linux, macOS
and Windows; the Raven binary distribution is not validated for those other hosts.

Extract the runtime bundle and SDK separately. Install the VSIX, then run from the
runtime bundle:

```sh
python3 configure.py --sdk /absolute/path/to/extracted/raven-sdk
code msbuild-demo
```

Use **Tasks: Run Build Task** to compile or **neoCLR: Run (MSBuild)** to build and run.
Reload existing VS Code windows after installing the extension. The program prints
`42`, `Saved`, `Completed`, `Overflow`, `Value found`, `42`, `Absent` on separate lines.
Build never executes the guest program. The normal Raven toolbar is not this target's
run/debug pipeline.

Terminal equivalent:

```sh
dotnet msbuild msbuild-demo/Demo.rvnproj -p:RavenSdkRoot=/absolute/path/to/extracted/raven-sdk
./bin/neoclr run msbuild-demo/bin/neoclr/Debug/App.neoil --system msbuild-demo/bin/neoclr/Debug/System.neoil
```

Open `project-reference-demo/App` for a separate library example. Its ProjectReference
builds the library before the application; the program prints `42` and `Library call`.
The language server resolves the library's public members before the first build.
See [MSBuild scope](raven-msbuild.md).

For more runtime APIs, copy a file from `tools/samples` over `msbuild-demo/Main.rvn`.
Try `application-order-collections.rvn`, `library-files.rvn`, `library-calendar.rvn`,
`library-reflection.rvn` and `library-array-foreach.rvn`. File/workflow samples may
create or replace their documented demo files in the working directory.

Direct neoIL remains a separate entry point:

```sh
./bin/neoclr run samples/neoil/type-categories.neoil --system lib/System.neoil
./bin/neoclr run samples/neoil/result-void.neoil --system lib/System.neoil
```

## Changes and migration

- Raven `.rvnproj` builds use standalone props/targets without Microsoft.NET.Sdk or
  a guest .NET TargetFramework. Build and editor share supplied references and target
  contracts. No Raven compiler change was needed for this MSBuild slice.
- One direct library ProjectReference is supported. Dependency DLLs use Raven's
  normal metadata references and are explicitly supplied to neoCLR import. Library
  and application must use identical core reference bytes. Dependency failures stop
  the build and invalidate prior runnable application output.
- `Array<T>.Empty` is a static property. `values.ForEach(action)` replaces the former
  static Array.ForEach helper. Empty currently allocates a zero-length array; shared
  identity is not guaranteed. Recompile against the matching core and runtime library.
- Indexers require `[index]`; `.Item` no longer binds as an element or offers its
  members in completion. `[index].` retains the normal element experience.
- The experimental Raven build incorporates independently tested general fixes for
  imported union patterns, metadata core identity, generic/interface signatures,
  void-call stack tracking and target-capability-based empty-array creation. General
  fixes are on Raven main; neoCLR-specific policies remain experimental.

## Limits and evidence

This is a bounded preview, not full .NET compatibility. The MSBuild path always
rebuilds and does not implement incremental builds, Clean/Rebuild, restore, package
references or multi-level project graphs. A library DLL is admitted for execution
when importing its consumer. Generic library bodies and the other
[importer limits](raven-library-import.md) remain unchanged.

The runtime library remains authored in neoIL; migration to Raven is still paused.
Reflection is introspection-only. Runtime async, full Raven debugging, new nullable
metadata and fault-unwind cleanup are not part of this release. Old Neo material
remains historical. API rough edges are intentional opportunities for feedback.

Publication requires all six exact-source CI jobs and extracted-package checks.
The attached manifest and validation archive record the actual results and revisions.
The experimental .14 Raven build is a separate distribution, not a normal Raven
release. Source evidence on modern .NET does not imply NanoFramework hardware testing.

`Type.IsValueType` exposes the runtime value/reference category through the Raven
reflection surface, without requiring a separate TypeInfo descriptor.
