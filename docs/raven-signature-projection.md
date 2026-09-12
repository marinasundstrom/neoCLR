# Shared Raven signature projection

Implemented 2026-09-13 as groundwork for the
[existing runtime API coverage plan](raven-runtime-api-coverage.md).

`RuntimeSignatures` now provides the common signature mechanics for the file and
collection catalogs, including collection constructors. Previously each catalog
substituted generic arguments and compared method references independently. The
shared code closes type parameters recursively through constructed types, vectors
and managed references, then compares the referenced method with its definition
under the same closed owner arguments. It does not mutate the input type graph.

Member matching checks receiver/calling convention, declaring type and owner arity,
then compares projected parameter and return types. The catalogs still choose which
concrete types, members and receiver adapters are admitted. Existing encoded-Void
validation, explicit assembly closure checks, output-initialization validation and
runtime verification remain separate requirements; signature matching replaces none
of them. The legacy union/static-call paths have not all migrated yet.

## CLR comparison and limits

This refactor retains the CLI type-parameter and constructed-signature model already
used by the [generic metadata foundation](generic-metadata.md) and the
[Raven reference artifact](raven-core-declarations.md). It adds no opcode or runtime
API contract. A CLI VOID return maps to `noresult`; named Void in a value position
remains a type, and a generic carrier containing it remains a carrier. See
[Void semantics](void-semantics.md) for the intentional difference from .NET.

The benefit is a single substitution/checking path for the next library catalogs,
reducing duplicated handling of nested generic signatures and Void. The cost is that
changes here affect both catalogs, so their executable regression probes remain
necessary. This is incremental bridge groundwork, not yet runtime-derived metadata
or arbitrary generic-program import.

Open method parameters, pointers, multidimensional arrays, custom modifiers and
nested managed references are rejected by this shared path. Pointers/modifiers need
explicit admission and semantics rather than silent erasure. A nesting limit bounds
recursive processing. These are current importer limits, not exclusions from the
runtime platform. Existing catalogs remain deliberately bounded until their library
shapes and execution tests are added.

## Verification

From the neoCLR root, with a built Raven checkout and a fresh output directory:

```sh
dotnet run --project docs/experiments/raven-target/Probe.csproj \
  -p:RavenRoot="$RAVEN_ROOT" -p:BuildProjectReferences=false -p:WarningLevel=0 \
  -- --signatures /tmp/neoclr-signature-checks
```

The probe checks nested substitution without mutation, closed residuals, signature
and receiver mismatches, bad arity/indexes, unsupported shapes, nesting limits and
Void contexts. It writes `signature-checks.json` only after all 20 checks pass (including two String-call rejection checks).

The existing `--interfaces` and `--files` probes, file execution/rejection checks
and saved-project regression suite exercise the migrated catalogs through real Raven
compilation and neoCLR execution. No Raven source or installed tool packages change
in this slice. The next work is to extend the shared catalog/type mapping and project
more existing APIs; the declaration inventory remains a coverage checklist.
