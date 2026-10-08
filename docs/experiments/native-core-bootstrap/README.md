# Native core bootstrap frontier (2026-10-08)

The release requires a compiler target that consumes neoCLR metadata without a
.NET semantic bridge. Native output and a standalone executable are not sufficient
proof. This reduced experiment addresses the first producer boundary and records
the next compiler boundary; it is not a complete runtime library or executable demo.

## Writer correction

Previously `AssemblyBuilder(identity, identity)` could produce native payload data
but PE/#Neo wrapping failed with “external assembly identity collides with output
identity”: the reference projection unconditionally imported its core as external.
For native reference emission with an explicit authored Object root, core references
now use local TypeDef handles. Missing local declarations reject; executable CLI
emission and foreign imports colliding with the output identity still reject.
The reference-only CLI projection remains a container/tooling view, not a compiler
semantic input in this experiment. No format version or artifact naming change.

This follows the existing CLI metadata architecture: TypeDefOrRef permits local
TypeDef references for bases and signatures, and MemberRef parents may be TypeDefs.
See [ECMA-335 sixth edition](https://ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf),
Partition II §§22.25 and 22.37 (consulted 2026-10-08). It avoids manufacturing an
external self-dependency; it does not change IL semantics or claim a performance gain.
An alternative is retaining an external bootstrap core, which remains supported but
cannot satisfy the author's no-bridge target gate on its own.

## Reproducer and remaining work

`Program.cs` authors a small core directly through the native metadata API, imports
it exclusively as `NeoClrMetadataReference`, and tries to compile an integer consumer.
No C# reference assembly is generated or passed to Raven. The .NET process hosts the
compiler and metadata tools; compiler hosting is separate from target bootstrapping.
The core deliberately uses the currently required `NeoCLR.CoreProbe` identity so the
probe reaches semantic initialization instead of failing the earlier CLI profile
name guard. It includes Object, ValueType, attribute support and three primitives;
it is not a full primitive/core contract.

```sh
dotnet run --project docs/experiments/native-core-bootstrap/Probe.csproj \
  -p:RavenRoot=/absolute/path/to/Raven \
  -p:NeoClrMetadataProject=/absolute/path/to/neoclr/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj \
  -p:WarningLevel=0 -- /tmp/fresh-native-core-probe
```

At Raven `bc3c500e6`, the core artifact is written, then diagnostics report **RAVT004**:
the .NET MetadataLoadContext cannot find NeoCLR.CoreProbe. The probe exits 2 and does
not produce a consumer. `DotNetCompilationTarget.InitializeSemanticData` still
creates a .NET session from portable references before native semantic references
are initialized. Adding a CLI reference projection would hide the failure and would
not close this gate. Next evaluate a native-only semantic initialization path behind
an explicit target contract, then catalog/driver/project support and a complete
source-runtime core. Preserve ordinary .NET and temporary CLI profiles as controls.
No Raven compiler implementation change is included here.

## Validation

[Recorded evidence](validation.json) distinguishes the successful writer checks from
the expected compiler frontier. The dedicated `--native-core` test checks local bases,
reference marker ownership, no external/self references, native primitive roundtrip,
determinism and missing-declaration/root rejection. Existing Object-root controls
exercise external-core output. API snapshot checking is independent of target execution.

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --native-core
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --object-roots
```

Interpreter/AOT consumer execution, full source-library bootstrap, editor and Windows
qualification remain pending. The module-system proposal does not alter this work.
