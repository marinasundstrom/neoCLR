# Build and run a Raven project

Start with a saved .rvnproj project. The same project describes your source files and runtime references for the editor and build tools.

**Published Preview 9.** These instructions use the matching macOS arm64 packages. The feature pages distinguish this preview’s API from later development changes and future proposals.

[Set up the preview ↓](#install)

<a id="install"></a>

## Install the matching tools
The prebuilt preview supports **macOS on Apple silicon**. For the development tools, install .NET SDK `11.0.100-rc.1.26425.128`, Python 3.9 or later, and VS Code.

**neoCLR itself and programs running on neoCLR do not depend on .NET.** The .NET requirement belongs to the surrounding tools: the Raven compiler, MSBuild, the import bridge and Raven Language Server. The VS Code extension uses that language server for editor features. Once a program is built and imported, running it with the neoCLR runtime and its matching runtime library does not require .NET.

Download these four assets from [Preview 9’s downloads](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.9):

- `neoclr-0.1.0-preview.9-osx-arm64.tar.gz` — runtime, project and samples.
- `raven-sdk-0.1.12-neoclr.async.20260923-osx-arm64.tar.gz` — matching compiler tools.
- `raven-vscode-0.1.12-neoclr.async.20260923.vsix` — matching editor extension.
- `raven-toolchain-notices.tar.gz` — companion notices.

Extract the runtime and SDK archives into separate folders. In VS Code, open Extensions, use the ⋯ menu, choose **Install from VSIX…** and select the downloaded extension. Reload any open VS Code windows afterward.

Open a terminal in the extracted runtime folder. Replace the SDK path below with the actual extracted SDK directory:

```text
python3 configure.py --sdk /absolute/path/to/extracted/raven-sdk
code msbuild-demo
```

If `code` is unavailable, use VS Code’s **File → Open Folder** and select `msbuild-demo`. Keep the bundle directories together; rerun configure.py if you move them.

<a id="run"></a>

## Build, then run
Open `Main.rvn`. Use **Tasks: Run Build Task** to compile and verify it. Use **Tasks: Run Task → neoCLR: Run (MSBuild)** to build and execute the program. Save edits before building.

The supplied program prints:

```text
42
Saved
Completed
Overflow
Value found
42
Absent
```

From a terminal in the runtime bundle, the equivalent steps are:

```text
dotnet msbuild msbuild-demo/Demo.rvnproj -p:RavenSdkRoot=/absolute/path/to/extracted/raven-sdk
./bin/neoclr run msbuild-demo/bin/neoclr/Debug/App.neoil --system msbuild-demo/bin/neoclr/Debug/System.neoil
```

Build checks the program and writes verified output; it does not execute it. The dedicated neoCLR task runs that output. The ordinary Raven toolbar is not this target’s run/debug pipeline.

<a id="project"></a>

## Project configuration

The bundled `Demo.rvnproj` is small:

```text
{{PROJECT_SAMPLE}}
```

`NeoCLRRoot` locates the extracted runtime bundle. The imports supply runtime references and build steps. `Compile` lists your Raven files; add an entry for each additional source file. For a project outside the bundle, set NeoCLRRoot to its installed location.

Unlike a .NET SDK project, this project does not need `Microsoft.NET.Sdk` or a .NET `TargetFramework`. `RavenSdkRoot` selects the host compiler. The editor reads the project’s imported references and target contracts too.

To explore a library, open the bundle’s `project-reference-demo/App` folder. Its application references a separate Raven library and prints `42` and `Library call`. The preview supports one direct library ProjectReference; use the same bundle for both projects.

<a id="explore"></a>

## Application source and examples

Make a copy of `msbuild-demo/Main.rvn`, then replace it with a sample from the bundle’s `tools/samples` folder. Build and run the same Demo.rvnproj. Start with `library-async-default-queue.rvn` for a worker and await, `library-task-composition.rvn` for completion composition, or `library-introspection-tour.rvn` for type discovery, `library-grapheme-strings.rvn` for text, or `library-calendar.rvn` for dates. `library-files.rvn` creates or replaces its demo file in the working directory.

Meet the syntax on the [Raven language page](../raven/), then explore [Option and Result](../features/outcomes/) or [collection capabilities](../features/collections/). Site examples describe their current implementation status; use the samples bundled with Preview 9 when running Preview 9.

<a id="limits"></a>

## If something does not work
- **Missing SDK:** run `dotnet --list-sdks` and check the exact version above is installed.
- **Missing references or completion:** rerun configure.py with the matching SDK and reopen the project folder. Keep the compiler, extension and runtime bundle versions together.
- **Build or importer error:** read the first diagnostic. The importer accepts a bounded subset of Raven/CLI programs; unsupported input stops the build. A failed build invalidates earlier runnable output.
- **No debugging:** Raven source debugging on neoCLR is not implemented. Use the dedicated build/run tasks and printed output.

This preview always rebuilds. It has no package restore, incremental build, Clean/Rebuild targets or multi-level project graphs. Prebuilt Raven tools are validated for macOS arm64; source checks on other hosts do not establish equivalent binary support.

<a id="development"></a>

## Version compatibility and migration

Preview 9 adds Task/Promise, async/await and isolated workers. Rebuild applications and libraries with its complete matching toolchain. From Preview 8: queries use Filter/Map instead of Where/Select; File and Path move to System.Storage; the System.Error wrapper is removed in favor of strings or domain error types. Older Introspection and text migrations still apply. The future System.Concurrency namespace and Task.Run-style API are not included.

The [feature pages](../#feature-pages) show tested examples and current limits. The [proposal overview](../proposals/) explains the open questions and possible future additions. Tell us what works for your programs and where these contracts should improve.
