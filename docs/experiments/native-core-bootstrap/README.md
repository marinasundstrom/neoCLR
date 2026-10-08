# Native core bootstrap frontier (2026-10-08)

The release requires a compiler target that consumes neoCLR metadata without a
.NET semantic bridge. Native output and a standalone executable are not sufficient
proof. This reduced experiment addresses native core production and compiler initialization,
then executes a small consumer; it is not a complete runtime library or release demo.

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
it and a separate native library through `NeoClrReferenceCatalog.ReadNative`, and
compiles an integer consumer using the catalog’s references and emission dependencies.
No C# reference assembly is generated or passed to Raven. The .NET process hosts the
compiler and metadata tools; compiler hosting is separate from target bootstrapping.
The current core uses the explicit `NativeCore` identity and remains deliberately
incomplete. It is not a full primitive/core contract.

```sh
dotnet run --project docs/experiments/native-core-bootstrap/Probe.csproj \
  -p:RavenRoot=/absolute/path/to/Raven \
  -p:NeoClrMetadataProject=/absolute/path/to/neoclr/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj \
  -p:WarningLevel=0 -- /tmp/fresh-native-core-probe
```

### Initial failure and native-only follow-through

At Raven `bc3c500e6`, the original three-primitive probe wrote its core artifact but
reported **RAVT004** during .NET metadata-session initialization. The
[initial evidence](validation.json) remains historical. It exposed a compiler
composition requirement rather than a failure to decode native metadata.

Raven commit `10dce0c3b` adds the explicit compiler API mode
`MetadataImportOptions.WithNativeMetadata()`, which now
bypasses that session. The current fixture uses its own `NativeCore` identity,
explicit unit/Object contracts and native semantic references only. It includes
Int32, Int64, Boolean, Void and String plus minimal Object slots and attribute support.
These are test fixtures; constant hash/display/Equals bodies are not production Object
behavior. `consumer.rvn` exercises a static call into a separate native library,
integer arithmetic and return. No fixture
Object method is selected by the AOT closure.

The probe checks symbols, emission, missing-core rejection, exact core-version
mismatch and default CLI emission refusal. A separate executable verifier assembles
an empty native System seed to avoid a competing default Object, then runs the consumer
in the interpreter and ARM64 native code. Both produce integer 42; the CLI interpreter
returns process exit 42. The C harness checks the native result/status and exits zero.
The executable links only macOS libSystem.

```sh
python3 docs/experiments/native-core-bootstrap/verify.py \
  --raven /absolute/path/to/Raven \
  --runtime target/release/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output /tmp/fresh-native-core-execution
```

[Execution evidence](execution-validation.json) records commands and artifact hashes.
The [catalog execution evidence](catalog-execution-validation.json) records the
follow-through using `ReadNative` and a separate library. The earlier execution report
retains the direct-reference arithmetic-only baseline. Focused Raven catalog checks
cover immutable snapshots, replacement, optional XML, dependency emission, missing
dependencies, duplicate identities/paths and CLI/malformed input rejection; the old
CLI-bootstrap catalog remains supported and its regression probe passes.

The driver now exposes this path with `--native-core-reference Core.dll`, repeated
`--reference Library.dll`, and the usual output/source arguments. Pass optional
`--driver /absolute/path/to/rvnc.dll` to verify.py to compile through that command and
run the result in both modes. [Driver evidence](driver-execution-validation.json)
records parity and rejection of mixed bridge flags without output publication.

This remains bounded: project/editor configuration and complete
source-runtime bootstrapping still use the CLI core path. Next wire project/editor
selection and test native core declaration completeness before changing the full bundle. RavenDoc now has an explicit native input provider reusing the same catalog. Build it
with the metadata project property and use `--native-core-reference` plus explicit
`--reference` dependencies, or `apiInputs`/`nativeCoreReference` in site configuration.
The [documentation qualification](ravendoc-validation.json) uses two native libraries
with actual declaring identities, existing member rendering, sidecars, navigation/search
and input rejection. This closes the loader issue, not the complete production bundle
migration. The website keeps its full existing API reference until native inputs pass
its public-API coverage checks.

## Validation

[Recorded evidence](validation.json) distinguishes the successful writer checks from
the original compiler frontier; the new execution report records its bounded resolution. The dedicated `--native-core` test checks local bases,
reference marker ownership, no external/self references, native primitive roundtrip,
determinism and missing-declaration/root rejection. Existing Object-root controls
exercise external-core output. API snapshot checking is independent of target execution.

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --native-core
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --object-roots
```

Full source-library bootstrap, editor and Windows qualification remain pending. The module-system proposal does not alter this work.
