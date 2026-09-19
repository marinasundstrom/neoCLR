# Normal Raven compilation and independent neoCLR import

Source experiment, 2026-09-14. Requires Raven branch `neoclr`
and the matching neoCLR source tools. Installed .12 SDK/extension/bundle artifacts
are unchanged. The runtime library migration remains paused.

The compiler now owns selection of the emission core. The project selects matching
metadata and emission identities; CLI, workspace and editor use that configuration.
The normal Raven compiler emits the PE assembly, then neoCLR's separate importer
validates and translates it. The compiler does not select neoCLR APIs from a bridge
binding table.

## Project configuration

Generated editor projects and the next bundle template now include:

```xml
<RavenMetadataCoreAssemblyName>NeoCLR.CoreProbe</RavenMetadataCoreAssemblyName>
<RavenTargetCoreAssemblyName>NeoCLR.CoreProbe</RavenTargetCoreAssemblyName>
<RavenUnitAssemblyName>NeoCLR.CoreProbe</RavenUnitAssemblyName>
<RavenUnitType>System.Void</RavenUnitType>
```

Keep the existing explicit reference to `NeoCLR.CoreProbe.dll`, iteration and
propagation contracts, array-invariance policy, disabled implicit imports and
framework projections. These are still evaluated MSBuild settings; they may be
collected in a shared `.props` file. A complete versioned target-pack schema is not
implemented by this slice.

The emission setting is opt-in. Raven resolves the full identity from the supplied
metadata core, and normal `Compilation.Emit` uses it. Conflicting explicit EmitOptions
or inconsistent metadata/emission names produce RAVT003 before an assembly is written.
Old import-only projects and default .NET emission retain their prior behavior.

The normal driver also respects explicit-reference policy even when invoked with a
host/tooling framework argument. It no longer adds host framework assemblies or
automatically discovered Raven support assemblies back to those projects, and keeps
the project's core-shim/runtime-async defaults. Explicitly supplied references remain
inputs; compiler plugins are not sandboxed by this policy.

## Try the normal compiler path

Build the matching Raven compiler and neoCLR bridge first. Use a generated project
with both core settings above. In these commands, set paths to your source checkouts,
project and matching generated System library:

```sh
RAVEN=/path/to/Raven
NEOCLR=/path/to/neoCLR
PROJECT=/path/to/editor/Demo.rvnproj
CORE=/path/to/editor/NeoCLR.CoreProbe.dll
SYSTEM=/path/to/System.Collections.neoil
OUTPUT=/path/to/new-build-directory

dotnet "$RAVEN/src/Raven.Compiler/bin/Debug/net11.0/rvnc.dll" \
  "$PROJECT" --framework net11.0 --no-project-restore -o "$OUTPUT/compiled"

dotnet "$NEOCLR/docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll" \
  --import "$OUTPUT/compiled/Demo.dll" "$CORE" "$OUTPUT/imported"

"$NEOCLR/target/debug/neoclr" verify "$OUTPUT/imported/App.neoil" --system "$SYSTEM"
"$NEOCLR/target/debug/neoclr" run "$OUTPUT/imported/App.neoil" --system "$SYSTEM"
```

For a project input, Raven's `-o` is a directory; the assembly name comes from the
project (Demo in this example). The importer requires a fresh output directory.
Execute the imported program with neoCLR; ordinary .NET launch artifacts beside the
PE do not make it runnable on the .NET runtime. This does not yet change `dotnet run`
or provide a complete neoCLR SDK target.

The existing `run_project.py` convenience command remains available. It now calls
ordinary `Compilation.Emit`, with no runner-supplied EmitOptions, and uses the same
artifact-import implementation. Its input project must select both core identities.
A matching new compiler/bridge/project is required; no installed demo is rewritten.

## Checks and remaining boundaries

`verify_compiler_target.py` exercises rvnc, independent import, runtime verification
and execution:

```sh
python3 "$NEOCLR/docs/experiments/raven-target/verify_compiler_target.py" "$PROJECT" \
  --compiler "$RAVEN/src/Raven.Compiler/bin/Debug/net11.0/rvnc.dll" \
  --bridge "$NEOCLR/docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll" \
  --runtime "$NEOCLR/target/debug/neoclr" --system "$SYSTEM"
```

Five checks cover ordinary compilation, the string-filter example, combined
Result/Option/Void propagation, rejection of an unavailable host API and conflicting
core settings. Artifact import audits dependencies: foreign host references are
rejected rather than silently mapped away. The saved-project suite also passes with
the new configuration. Raven's normal .NET 10/11 target matrix passes; this does not
claim NanoFramework validation.

The importer remains bounded and C#/Cecil-based. Named Void value-storage projection,
per-API runtime adapters and generic library-body gaps
remain. The new command separates compilation from importing; it is not the runtime's
native PE loader or a general Raven library compiler. These are the next subjects of
[the target assessment](raven-target-evaluation.md), before resuming library authoring.

This reuses the .NET separation of reference contracts, compiler emission and runtime
loading described in [the binary profile](raven-binary-profile.md) and
[the assessment's primary-source comparison](raven-target-evaluation.md#comparison-and-tradeoffs).
The benefit is consistent compiler/editor target behavior and independently testable
artifacts. The cost is maintaining explicit configuration and a temporary importer
until native loading and general metadata resolution are ready.

Application symbols now use [assembly/signature identities](raven-import-identities.md);
private generated helpers and source locations retain module-local token information.

For independently compiled dependencies, see the bounded
[separate library import workflow](raven-library-import.md).


A later [minimal MSBuild slice](raven-msbuild.md) now orchestrates this compiler/importer
path using standalone `.rvnproj` imports. It supports one application and the supplied
core reference; it is not Microsoft.NET.Sdk integration or general library compilation.

## Runtime Contracts and documentation

Raven documents the reusable compiler mechanisms in
`docs/compiler/runtime-contracts.md` in its repository. The neoCLR project profile
selects those contracts; it also retains experimental policies such as nominal Void
generic arguments, generic arrays and result-based error flow. General compiler
mechanisms are integrated independently into Raven main. The neoCLR profile and
its specific tests remain on the experimental Raven branch.

The [Void contract](void-semantics.md#raven-runtime-contract--2026-09-14) distinguishes
no-result calls from value contexts and provides an end-to-end validation command.
Older installed SDKs do not implement the new unit properties; use a source-built
experimental compiler until the next tools refresh.

For every compiler-affecting integration change, update Raven’s compiler documentation
and changelog as well as neoCLR’s integration documentation and changelog. Record
configuration, semantic and emission consequences, limitations and validation evidence.
Keep the [evaluation record](raven-target-evaluation.md) current so that general fixes
and experimental policies can be reviewed separately.
