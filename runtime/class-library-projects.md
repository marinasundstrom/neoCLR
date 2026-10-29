# Native class-library projects

These projects build the native class library from Raven source. The older
`legacy/raven/System.Runtime.rvnproj` and bootstrap preparation tools are legacy fixture and
documentation paths, not production inputs.

System.Runtime selects `RavenNeoClrSourceCore=true` and the explicit
`RavenNeoClrRuntimeContracts` manifest. It owns primitive declarations, core bases,
Self/method markers, managed array reservation and runtime-service declarations.
Data, Networking, Web and optional PostgreSql consume its native metadata through
`RavenNeoClrNativeCoreReference`. No Core.dll, CLI projection, translator or retained
service bodies are required. Ordinary Raven/.NET projects are unchanged.

## Build the class-library bundle

Use Raven `34f14c8ce` or a descendant containing native source-core contracts,
generic interface emission and nested native callback nullability, together with
a matching neoCLR runtime. Build the runtime and compiler first, then:

```sh
python3 scripts/build-native-class-library.py \
  --compiler /absolute/path/rvnc.dll \
  --compiler-revision EXACT_REVISION \
  --runtime /absolute/path/neoclr \
  --with-postgresql \
  --output /absolute/path/fresh-bundle
```

The builder constructs Runtime without references, creates an empty reserved System
module in NEOX format, then builds dependent projects with `--no-build-references`.
Data and Networking reference Runtime; Web references Data and Networking.
Each Neo abstraction project references Runtime. Implementations reference their
own abstractions; logging, localization and configuration reference DI abstractions.
Localization additionally references Data for JSON resources. The optional
Neo.Configuration.Json provider references Data and the configuration projects.
PostgreSql is optional. Runtime components are under runtime/<component>; Neo components are under
extensions/<component>. Both group source projects in src/<project> and optional
test projects in test/<project>. Each project contains its project file and
namespace folders. No solution file is required.
It verifies Runtime did not change during dependent builds and writes `bundle.json`
last. No manifest means an incomplete bundle. Native references still validate
exact dependency identities; this is not an implicit package resolver.

`build-evidence.json` retains commands, source/input hashes and revisions. XML and
Markdown sidecars are included. The class-library bundle does not include runtime,
compiler or VS Code executables. See [qualification](../docs/validation/native-core-removal/README.md)
for compilation, execution and editor evidence. The earlier bootstrap reproduction
records are historical and do not describe this build command.

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

The imported configuration selects the native System.Runtime core, dependent native
libraries, empty execution seed and explicit runtime contracts. Runtime owns Object
and async symbols. No library
sources, environment variables or absolute build-machine paths are needed for this
consumer. Compile with `rvnc neoclr --project Headers.rvnproj --run /path/to/neoclr`.
A native-enabled Raven language server uses the same project metadata provider.

The native project probe checks workspace symbols, runtime contracts and completion.
The legacy `verify-native-bundle-project.py` harness targets historical bootstrap bundles. The manifest records integrity;
the project loader consumes the explicit reference catalog, not a package resolver.
Headless workspace validation does not replace installed VS Code acceptance.

## Optional PostgreSQL development provider

Pass `--with-postgresql` to `build-native-class-library.py` to build and stage
System.Data.PostgreSql after the ordinary four libraries. It references Runtime,
Data and Networking; Data's common `System.Data.Sql` contracts do not depend on
Networking. The provider uses the existing explicit retained-async helper from
`runtime/compiler-services/RuntimeAwait.rvn`. Its project selects
`RavenNeoClrRuntimeAsyncHelper=RuntimeAwaitProvider.Bridge`; ordinary projects/.NET
semantics are unchanged. It currently supports local trust authentication only.
See [SQL data access](../docs/sql-data-access.md) for contracts and validation.

Rebuilt SDK imports now default consuming projects to runtime-await and include the
SDK-owned provider; bootstrap library projects keep their explicit configuration.
See [integration and fallback](../docs/runtime-product-integration.md#sdk-default-runtime-await-integration--2026-10-13).
