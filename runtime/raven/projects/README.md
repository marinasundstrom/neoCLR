# Native class-library projects

These are native Raven projects. The older `../System.Runtime.rvnproj` remains the
legacy CLI bridge slice project; it is not this native build.

`System.Runtime/System.Runtime.rvnproj` selects the audited 175-source Runtime
foundation. Data, Networking and Web do not belong to this project. Platform helpers
remain in Runtime for now; extracting their service boundaries is a separate gate.
The colocated ownership manifest records source and semantic ownership explicitly.

Build with a Raven compiler containing the NeoCLR metadata adapter:

```sh
RavenNeoClrCoreReference=/absolute/path/Core.dll \
RavenNeoClrRuntimeSeed=/absolute/path/bootstrap/System.neox \
  dotnet /absolute/path/rvnc.dll neoclr \
  --project runtime/raven/projects/System.Runtime/System.Runtime.rvnproj
```

These environment variables are evaluated MSBuild properties. They must refer to
matching primitive-bootstrap and compile-time retained-seed artifacts; there is no
implicit download, host-framework fallback or application metadata projection.
`scripts/prepare-native-bootstrap.py` produces these inputs directly from checked-in
sources, without Numbers/Http or a previous Runtime artifact:

```sh
dotnet build docs/experiments/raven-target/Probe.csproj -p:RavenRoot="$RAVEN"
dotnet build tools/metadata/NeoCLR.Metadata.Translate
python3 scripts/prepare-native-bootstrap.py \
  --probe docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --runtime target/release/neoclr \
  --translator tools/metadata/NeoCLR.Metadata.Translate/bin/Debug/net10.0/NeoCLR.Metadata.Translate.dll \
  --output /absolute/path/fresh-bootstrap
```

Build the runtime with `cargo build --release` first. When producing a matching SDK,
finish SDK packaging before building the probe/translator against the same project
outputs. Use canonical absolute paths. See the
[clean-source reproduction](../../../docs/experiments/extended-cli-metadata/clean-bootstrap-reproduction-2026-10-07.md)
for the pinned revisions, toolchain prerequisites and validation limits. Select the matching Raven source
revision explicitly. Use the generated `Core.dll` and bootstrap directory in the staged
build command below. `bootstrap.json` is written last; when present, the library builder
validates all preparation hashes and rejects a different selected core. The old audit
workflow remains available for historical frontier experiments. They remain explicit dependencies, not a
claim of bootstrap-free compilation.

The output is `System.Runtime/bin/neoclr/System.Runtime.dll`. Before execution,
finalize the retained seed against that exact artifact:

```sh
dotnet /absolute/path/NeoCLR.Metadata.Translate.dll \
  /absolute/path/bootstrap/System.retained.json /absolute/path/System.runtime.neox \
  --reference runtime/raven/projects/System.Runtime/bin/neoclr/System.Runtime.dll
```

Use `System.runtime.neox` for consumers and runtime execution, with the emitted
Runtime artifact selected as Object owner. The compile-time seed and finalized seed
serve different purposes. A seed finalized against an older Runtime is not silently
reused. This prototype finalization remains explicit tooling; it does not translate
application/library references into .NET metadata.

`scripts/verify-native-runtime-project.py` reproduces the project build, finalization
and source-free execution of unchanged `application-order-collections`, with hashes
and exact stdout/exit checks. The higher-level projects and staged build below extend
this foundation; Platform extraction and full SDK/editor packaging remain open.

Compared with the existing .NET workflow, the native project uses the same evaluated
project/source model but requires explicit primitive and retained-seed inputs. This
makes ownership reviewable at the cost of an additional finalization stage. No .NET
compiler defaults or native metadata encodings change in this slice.


## Build the class-library bundle

The checked-in graph is `System.Runtime ← System.Data`,
`System.Runtime ← System.Networking`, and `System.Data + System.Networking ← System.Web`.
Networking currently owns its two native service adapters. Those adapters are not yet
separate Platform projects. Runtime does not reference the higher-level libraries.

```sh
python3 scripts/build-native-class-library.py \
  --compiler /absolute/path/rvnc.dll \
  --compiler-revision REVISION \
  --core /absolute/path/Core.dll \
  --bootstrap-directory /absolute/path/bootstrap \
  --translator /absolute/path/NeoCLR.Metadata.Translate.dll \
  --output /absolute/path/fresh-bundle
```

The bootstrap directory supplies `System.neox` and `System.retained.json` as described
above. The build uses ordinary compiler project commands in these stages:

1. Build Runtime with the compile-time seed (`RavenNeoClrBootstrapSeed`).
2. Finalize `System.runtime.neox` against that exact Runtime artifact.
3. Build Data, Networking and Web in order using `--no-build-references`, importing
   existing native project artifacts and the finalized seed (`RavenNeoClrRuntimeSeed`).
4. Confirm Runtime did not change; copy the four assemblies, primitive Core, finalized
   seed and ownership manifest into the fresh bundle directory.
5. Write `bundle.json` last with artifact hashes. A directory without this manifest is
   incomplete and must not be treated as a published bundle.

Prebuilt mode is explicit host orchestration, not freshness detection or a metadata
fallback. The project loader still checks dependencies and identities. The initial
approach of rebuilding Runtime through Web's dependency graph changed Runtime bytes
and was rejected; do not assume byte-deterministic compiler output or reuse a stale
finalized seed. The build tool fails rather than publishing such a bundle.

`build-evidence.json` records commands, input/source hashes and revisions. Generated XML and Markdown sidecars are bundled and hashed. No runtime
binary, Raven toolchain or VS Code extension is bundled here. These
are qualified class-library artifacts, not a complete installable SDK or release.
The same staged artifacts pass `verify-separate-web.py` and the ordinary-project
`verify-native-library-project.py` acceptance paths.


## Consume a relocated bundle

The bundle includes `NeoCLR.ClassLibrary.props`, using paths relative to that file.
An application can use this project (the example directory name is tested):

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <Import Project="SDK with spaces/NeoCLR.ClassLibrary.props" />
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <OutputType>Exe</OutputType>
    <AssemblyName>Headers</AssemblyName>
  </PropertyGroup>
</Project>
```

The imported configuration selects native metadata, all four native libraries and the
matching Core/seed/ownership artifacts. It selects Runtime for Object and async symbols,
and disables source-root and bootstrap-intrinsic authoring for applications. No library
sources, environment variables or absolute build-machine paths are needed for this
consumer. Compile with `rvnc neoclr --project Headers.rvnproj --run /path/to/neoclr`.
A native-enabled Raven language server uses the same project metadata provider.

`verify-native-bundle-project.py` checks relocated workspace symbols, missing/conflicting
inputs, manifest hashes and ordinary project execution. The manifest records integrity;
the project loader consumes the explicit reference catalog, not a package resolver.
Headless workspace validation does not replace installed VS Code acceptance.
