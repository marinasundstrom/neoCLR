# Experimental .NET metadata libraries

`NeoCLR.Metadata.Experimental` is the first .NET-hosted reusable format library for
future Raven symbol-loader and code-generation adapters. `MetadataConformance` is a
separate executable consumer. Both target .NET 10, are non-packable and depend only
on the platform libraries; neither requires Raven or Mono.Cecil.

The current library covers NEOX 0.1 envelope framing, immutable section ownership
and structural signature syntax/context validation, reference-table codecs and
explicit-catalog structural identity and synthesized-member contracts. MetadataProfile
provides typed Read/Create/Write entry points with owned documents and section-profile
validation. Bounded PE32 recognition/extraction is implemented; actual
dependency binding and compiler adapters remain pending. A read-only Model namespace
now exposes real assembly/module/TypeDef declarations and definition-backed references. It is not the future neoCLR guest metadata/Introspection/Emit
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
is a foundation for a later Raven compiler adapter; complete assembly IO is pending.
A Raven port may underpin Metadata Introspection, with shared conformance contracts.

The artifact reader requires recognized extended input by default, with explicit
ordinary classification available. Fifty shared cases pass, including 44 rejections.
Support is bounded unsigned IL-only PE32, not a general loader or verifier.
The [Cecil-inspired object model direction](../../docs/design/extended-cli-metadata.md#cecil-inspired-object-model-direction-2026-09-30)
is provisional; the existing profile/codec APIs remain lower layers.

The model consumer reads a generated generic/nested Unicode fixture through owned
AssemblyDefinition/ModuleDefinition/TypeDefinition/TypeReference objects. It checks
snapshot-local resolution, absence of input/reader lifetime coupling, and malformed
CLI tables even with valid extension binding. Cross-module resolution and assembly
writing remain pending.
