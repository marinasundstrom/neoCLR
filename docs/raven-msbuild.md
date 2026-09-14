# Minimal MSBuild support for neoCLR

Available in Preview 7, 2026-09-14. A Raven `.rvnproj` can now build an application through
standalone MSBuild props/targets. It does not import `Microsoft.NET.Sdk`, declare a
.NET target framework, or require a new Raven compiler build. It uses the installed
experimental .14 compiler and the matching runtime bundle.

## Installed local example

A fresh local bundle is available at
`/Users/robert/.neoclr/experiments/msbuild-20260914`. Open its MSBuild project in VS Code:

```sh
code /Users/robert/.neoclr/experiments/msbuild-20260914/msbuild-demo
```

Use **Tasks: Run Build Task** to compile it. The matching `.14` SDK and language server
are already selected. Or build from a terminal:

```sh
dotnet msbuild /Users/robert/.neoclr/experiments/msbuild-20260914/msbuild-demo/Demo.rvnproj \
  -p:RavenSdkRoot=/Users/robert/.raven/sdk/0.1.12-neoclr.14
```

The packaged assets passed 15 build scenarios and 68 language-server checks on this
standalone project. The demo was also built and executed separately with its expected
Result/Option/Void output. All 758 manifest files were verified. See the
[local validation record](experiments/raven-target/msbuild-toolchain-20260914.json).
This is a local installation; published previews and prior demo directories are unchanged.

## Build a bundled project

New bundles contain `build/NeoCLR.Raven.props`, `build/NeoCLR.Raven.targets` and
`msbuild-demo/Demo.rvnproj`. From the bundle directory:

```sh
dotnet msbuild msbuild-demo/Demo.rvnproj \
  -p:RavenSdkRoot=/absolute/path/to/raven-sdk \
  -p:Configuration=Debug
```

The host requires the .NET SDK to run MSBuild and Raven; the guest program targets
neoCLR. No `TargetFramework` property is needed in this project.

Run the verified output separately:

```sh
./bin/neoclr run msbuild-demo/bin/neoclr/Debug/App.neoil \
  --system msbuild-demo/bin/neoclr/Debug/System.neoil
```

`configure.py --sdk /absolute/path/to/raven-sdk` also configures the MSBuild demo's
VS Code settings and a default **neoCLR: Build with MSBuild** task. Open `msbuild-demo`
and use **Tasks: Run Build Task**. Building does not execute the program.
The existing `demo` directory and Python runner remain available independently.

## Project file

The bundled project is deliberately small:

```xml
<Project DefaultTargets="Build">
  <PropertyGroup>
    <NeoCLRRoot>$(MSBuildThisFileDirectory)..</NeoCLRRoot>
  </PropertyGroup>
  <Import Project="$(NeoCLRRoot)/build/NeoCLR.Raven.props" />
  <ItemGroup>
    <Compile Include="Main.rvn" />
  </ItemGroup>
  <Import Project="$(NeoCLRRoot)/build/NeoCLR.Raven.targets" />
</Project>
```

For a project elsewhere, set `NeoCLRRoot` in the project to the installed bundle.
Include source files explicitly; regular MSBuild Compile globs are also available.
The props file supplies the core reference and Raven's iteration, propagation and
array contracts. Raven's normal project evaluator reads those same imports for
compilation and editor services. Design-time Build skips compilation entirely.

`RavenSdkRoot` selects the compiler host; `Configuration` selects Debug or Release
and is forwarded to Raven's project evaluation. Set other target/semantic properties
in the project or its imports. This slice does not forward arbitrary command-line
MSBuild global properties into Raven's separate evaluation process.

## A referenced library

Open `project-reference-demo/App` for the two-project example. The library sets
`OutputType` to `Library`; the application includes:

```xml
<ProjectReference Include="../Library/Library.rvnproj" />
```

MSBuild builds the library first and supplies its normal `TargetPath` DLL to Raven's
existing project-reference resolver. The importer receives that same DLL explicitly
alongside the consumer. Source-based language-server project references work before
the first build. No new Raven compiler policy or runtime opcode is required.

The application and library must use the exact same core reference bytes. The build
checks their SHA-256 hashes before compiling the dependency. This is intentionally
stricter than assembly identity compatibility: independently regenerated reference
packs may be rejected even if they are equivalent. Use the same versioned bundle.

A library build emits a DLL; executable runtime admission happens when importing its
consumer. Generic bodies and other importer limitations remain unchanged. One direct
library reference is supported. Libraries cannot reference further projects, and
cycles, executable dependencies, package references and extra assembly references
are rejected. This small graph is the tested starting point, not full solution support.

## Outputs and limits

Build performs validation, Raven compilation, import, runtime verification and copying
of verified output. Each attempt keeps its PE and imported artifacts under
`obj/neoclr/<Configuration>/<attempt-id>`. The successful runnable files are
`bin/neoclr/<Configuration>/App.neoil`, its `.map.json`, and `System.neoil`. The
compiled application/library DLL is copied to its declared `TargetPath` in that directory.
At the start of a normal build, these prior output files are invalidated.
Compiler/importer/verification failures stop the build before replacement output is
published. The initial targets always rebuild; they do not implement incremental
builds, Clean, Rebuild, Restore, Publish or Run targets. Old intermediate attempts
remain available for inspection and can be removed when no build is using them.

The current importer still determines the supported Raven/CLI subset; using MSBuild
does not expand runtime or library support. Concurrent builds of the same
project/configuration are not supported in this first implementation.

The design follows ordinary [MSBuild project imports and build phases](https://learn.microsoft.com/en-us/visualstudio/msbuild/build-process-overview).
Compared with the .NET project SDK, this preserves familiar project evaluation and
diagnostics while supplying only neoCLR's small build pipeline. The tradeoff is that
standard SDK conveniences are not automatically available. See the
[recorded scope and alternatives](raven-target-profiles.md#msbuild-project-support--future-milestone-2026-09-14).

## Validate

Use the installed toolchain without either source checkout:

```sh
python3 tools/verify_msbuild.py --bundle "$PWD" --sdk /absolute/path/to/raven-sdk
```

For development, the verifier also accepts `--assets /path/to/neoCLR/build`.
It checks SDK-free evaluation, design-time behavior, Debug/Release compilation,
Result/Option/Void, generic arrays, paths containing spaces, changed source, diagnostics,
unsupported inputs, stale-output invalidation and recovery, including dependency
changes/failures, Release configuration and mismatched reference packs. Successful artifacts are
executed separately to check behavior. This is not a full MSBuild or .NET SDK release gate.
