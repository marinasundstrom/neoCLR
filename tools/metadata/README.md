# Experimental .NET metadata libraries

`NeoCLR.Metadata.Experimental` is the first .NET-hosted reusable format library for
future Raven symbol-loader and code-generation adapters. `MetadataConformance` is a
separate executable consumer. Both target .NET 10, are non-packable and depend only
on the platform libraries; neither requires Raven or Mono.Cecil.

The Cecil-style API remains an independent library project. Raven's neoCLR target
will consume it through compiler-owned semantic-loader and emitter adapters. Format
code and metadata objects stay here; compiler symbols, lowering and target diagnostics
stay in Raven. The dependency is one-way, from that target to this library. The current
.NET compiler provider remains the default. Package distribution and a possible
repository split have not been selected.

The current library covers NEOX 0.1 envelope framing, immutable section ownership
and structural signature syntax/context validation, reference-table codecs and
explicit-catalog structural identity and synthesized-member contracts. MetadataProfile
provides typed Read/Create/Write entry points with owned documents and section-profile
validation. Bounded PE32 recognition/extraction, explicit AssemblyRef/TypeRef binding
and the emitted static-method MemberRef subset are implemented. The Model namespace
exposes assembly, type and callable declarations, alongside controlled PE/native writers.
Compiler adapters remain pending. It is not the future neoCLR guest metadata/Introspection/Emit
library. Names and contracts are experimental.

- [Complete API contract](../../api-docs/experimental-metadata.md)
- [Format design](../../docs/design/extended-cli-metadata.md)
- [Wire schemas and probe history](../../docs/experiments/extended-cli-metadata/README.md)

From the repository root:

```sh
python3 docs/experiments/extended-cli-metadata/verify_dotnet.py
python3 docs/experiments/extended-cli-metadata/verify_dotnet_signatures.py
python3 docs/experiments/extended-cli-metadata/verify_dotnet_references.py
python3 docs/experiments/extended-cli-metadata/verify_dotnet_members.py
python3 docs/experiments/extended-cli-metadata/verify_dotnet_profiles.py
python3 docs/experiments/extended-cli-metadata/verify_dotnet_artifacts.py
python3 docs/experiments/extended-cli-metadata/verify_dotnet_model.py
```

The verifier builds the library/consumer, runs ownership and writer checks, cross-reads
four existing fixtures and independent .NET emission, and compares malformed-input
rejection with Python. Output binaries go under `target/extended-cli-metadata`; a
versioned evidence record goes beside the existing experiment records. No package is
published or installed. Prefer focused conformance over unrelated runtime/website builds.

The signature consumer compares 14 positive payloads and 103 rejection vectors against
Python, including nominal syntax (without resolution), nested Self/generic contexts,
Function modes and no-result, and resource limits. TypeExpression preserves wire syntax;
it is not the eventual Raven symbol model or resolved neoCLR type identity.

The reference consumer compares 95 shared table/identity vectors, including 69
rejections. Catalog scopes are host-assigned, not physical CLI assembly identities.
Reference numbering is erased from resolved equality; declaring owners, Function
contracts and array storage remain significant. General PE support and actual CLI
declaration loading remain separate work.

The member consumer compares 49 shared vectors: 14 derived contracts, five identity
comparisons and 30 rejections. Array length returns native unsigned, tuple projection/
deconstruction preserve element contracts, and Function invocation preserves modes
and no-result. Descriptors have no runtime invocation or dispatch capability.

The profile consumer covers 36 shared cases (29 rejections), independent typed
emission, optional preservation, ownership and explicit catalog failures. The facade
is a foundation for a later Raven compiler adapter; general assembly IO is pending.
A Raven port may underpin Metadata Introspection, with shared conformance contracts.

The artifact reader requires recognized extended input by default, with explicit
ordinary classification available. Fifty shared cases pass, including 44 rejections.
Support is bounded unsigned IL-only PE32, not a general loader or verifier.
The [Cecil-inspired object model direction](../../docs/design/extended-cli-metadata.md#cecil-inspired-object-model-direction-2026-09-30)
is now the selected primary compiler abstraction; the existing profile/codec APIs remain lower layers.

The model consumer reads a generated generic/nested Unicode fixture through owned
AssemblyDefinition/ModuleDefinition/TypeDefinition/TypeReference objects. It checks
snapshot-local resolution, absence of input/reader lifetime coupling, and malformed
CLI tables even with valid extension binding. Nominal cross-module resolution and controlled new-assembly writing now exist; general
rewriting remains pending.


The Cecil-like model is the selected primary compiler abstraction for metadata and PE
manipulation. Reading, explicit nominal resolution and a bounded editable PE producer
exist; general import, declaration mutation and arbitrary rewriting remain pending. Adaptations to neoCLR are intentional, not a
promise of Cecil source compatibility.

Run the dedicated C# contract tests independently of the Python conformance drivers:

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests/NeoCLR.Metadata.Experimental.Tests.csproj --no-launch-profile
```

This is an executable test runner (nonzero exit on failure), not a dotnet-test discovery
project. Its 21 contract groups generate PE metadata in C#, exercise exact identities and resolver
contracts, and cover invalid inputs, limits and ownership. It uses only .NET platform
libraries and the project under test.


## Compiler producer baseline

Physical TypeRef resolution supports local, assembly and nested scopes. Controlled
AssemblyBuilder/TypeBuilder/MethodBuilder objects emit new ordinary CLI PE assemblies
with static Int32 methods, validated linear bodies and imported cross-assembly calls.
Fourteen C# contract groups cover the model and writer. The runtime integration runner
emits an application/library pair and requires neoCLR verification plus exit code 42:

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --runtime-integration /path/to/neoclr /path/to/Probe.dll \
  /path/to/NeoCLR.CoreProbe.dll /path/to/System.neoil /fresh/output
```

Use matching bridge/core and composed Raven System inputs. The test serializes the
bridge output to native JSON and loads it in neoCLR; no hand-authored application IL
or JSON is substituted. This establishes the existing Raven bridge route, not direct
PE/#Neo runtime loading. The writer's deliberately bounded subset and every public
member are described in the API reference. General rewriting and wider codegen remain
pending rather than silently dropping unsupported metadata.

## Direct native producer and top-level functions

`AssemblyBuilder.AddFunction` creates a function with no declaring type.
`WriteNativeAssembly()` emits the current native format-5 assembly directly, including
cross-assembly top-level calls. `Write()` still emits PE; local globals map to CLI
`<Module>` methods, while cross-assembly global calls are explicitly unsupported there.

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --native-integration target/debug/neoclr target/metadata-native-proof
```

Use a fresh output directory. This C# test directly loads the API-produced native
application and library, verifies them and executes to 42; it also rejects missing and
wrong-revision dependencies. No bridge is used. See the [API reference](../../api-docs/experimental-metadata.md#controlled-pe-and-native-assembly-construction)
for ownership, limits and transport details. The 21 contract groups additionally check
CLI global-method rows and native ownership. This is a compiler/backend baseline for
the bounded Int32 subset, not complete NEOX structural execution or Raven integration.

The PE reader exposes `ModuleDefinition.Methods` (all rows), `Functions` (top-level
functions), `TypeDefinition.Methods`, `GetMethodDefinition` and `AssemblyDefinition.EntryPoint`.
`MethodDefinition` retains flags and owned CLI signature bytes, with explicit recognition
of the emitted static Int32/no-result subset. General signatures remain opaque; bodies
and general signature/member import remain pending. Physical MemberRefs now resolve
the writer's static Int32/no-result method subset through explicit dependencies. See the manual API reference for
ownership and reader resource bounds.


## Raven consumer, first stage

Raven's `tools/NeoClrMetadataProbe` at commit `7e18edb66` on
`codex/metadata-consumer` consumes this project through an explicit
`NeoClrMetadataProject` project-reference property. Its source program binds against
an API-produced PE dependency through the existing .NET provider, then its operations
adapter emits a native application through this API. neoCLR verifies/runs the resulting
application and dependency with result 42. [Recorded evidence](../../docs/experiments/extended-cli-metadata/raven-compiler-validation.json).

This is an independent compiler-side consumer, not Raven code moved into this library.
It is not yet an installed native loader/emitter target. Production target composition,
native metadata symbol loading and wider signatures remain staged follow-ups. The
consumer also checks unsupported operators and compiler binding failures.

Read-only callable imports now let compiler consumers emit PE/native calls without
the dependency builder graph. See the [host API reference](../../api-docs/experimental-metadata.md#importing-a-read-only-callable-development).

The native declaration reader now projects compiler-only PE references from the
writer's format-5 output. Bodies stay opaque; execute only the original native artifact.
See the host API reference for strict bounds and the temporary input bridge contract.

## Direct native metadata containers

`RuntimeAssemblyContainer.Write(nativeBytes, core)` embeds the authoritative native
format-5 payload in a required PE/#Neo execution section. `Read` retrieves owned native
bytes; `ReadCliProjection` supplies a Cecil-style snapshot of the reference declarations.
The same PE file can be a Raven compiler reference and a neoCLR runtime input.
Schema 1 still contains JSON; binary encoding and faster parsing remain future work.

```sh
cargo build
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --container-integration target/debug/neoclr target/extended-cli-metadata/container
cargo test --test metadata_container
```

The compiler-side `EmitMetadataAssembly` and the two-library acceptance case live on
Raven's `codex/metadata-consumer`. See the [public API contract](../../api-docs/experimental-metadata.md#runtimeassemblycontainer).
