# Explicit runtime Object-root selection — 2026-10-05

This slice closes the runtime admission prerequisite exposed by the
[root investigation](object-root-ownership-2026-10-05.md). It does not claim source
Object compilation. Raven implementation is unchanged from `e86c436eb`; matching integration notes are `ee4ddba20`.
Runtime baseline: `4c7882e6`. [Source hashes and validation record](explicit-object-root-2026-10-05.json).

## Implemented contract

The host selects an exact TypeDefId through
`LoadedProgram::with_modules_and_object_root`. The mixed reader accepts the same
selection via `read_modules_with_object_root`. Only the private linked snapshot keeps
this context; no JSON/NEOX/PE field or version change is introduced. The selected root
must belong to System or an explicitly supplied dependency, not the application.
Its definition and all three ordinary Object slots are validated. Existing loaders
retain legacy admission and the existing application-lookalike rejection.

The explicit mode validates the System seed together with dependencies so the seed
can refer to a source root library. Its direct references and accessibility uses are
still checked. Wrong root identities/revisions, malformed seed formats, missing seed
references, inaccessible calls and duplicate ownership reject before execution.

The tests execute external-root primitive display/equality/hash and rootless overrides,
then round-trip NEOX artifacts and require host selection again. Original modules are
unchanged. They also prove revision matching and that JSON cannot inject the selection.
The runtime-only fixtures are contract tests, not substitute production Object code.

## Comparison and tradeoffs

.NET's runtime design identifies CoreLib as the provider of Object and other core
classes, with runtime-specific binding/validation rather than arbitrary type-name
matching. See [CoreLib design, dotnet/runtime v10.0.0](https://github.com/dotnet/runtime/blob/v10.0.0/docs/design/coreclr/botr/corelib.md),
“Dependencies” and “What makes System.Private.CoreLib special?”, retrieved 2026-10-05.
This is implementation-layer precedent, not a claim that ECMA mandates this hosting API.

neoCLR retains its existing abstract Object source shape and supported slot semantics.
Host configuration was chosen over an artifact-self-designation: it keeps the selection
in the same trust boundary as dependency loading and avoids extending the metadata
format prematurely. It costs an additional explicit host parameter and repeat selection
after deserialization. The host must validate artifact identities/digests independently;
a row/revision label is not a durable identity across arbitrary replacements.

The linked Module carries private transient context because verification, dispatch and
analysis already share that immutable snapshot. This avoids duplicating those paths,
but is a Rust source-compatibility change for callers constructing Module literals.
Use existing readers/deserialization instead; see the [API reference](../../../api-docs/runtime-hosting.md).
No runtime-reflection layer, new instruction, .NET backend change or speedup is claimed.

## Remaining end-to-end work

1. Add explicit source-root ownership to the bootstrap manifest/driver configuration.
2. Make keyword Object, named System.Object, implicit bases and override signatures
   use that identity, including a genuinely baseless source root.
3. Add definition/builder parity for root authoring and use the selected core for
   Object signatures, boxing and slots. Add C# contracts with those public APIs.
4. Connect the runtime host selection to ordinary CLI execution using the same artifact
   catalog. Then remove the competing seed root and run the unchanged cumulative
   source library and artifact-only JSON/Tasks/Object consumers.

No existing gate's seed has been replaced in this slice. Metadata APIs and Raven
contracts remain independent.

## Reproduction

```sh
cargo test --release --test object_root_identity --test object_display \
 --test object_equality --test loaded_program --test modules \
 --test library_references --test scoped_types --test revisions
```

On this machine SDKROOT points at MacOSX26.2.sdk. The guest API snapshot check still
reports the previously documented stale reference. Rust host APIs have a manual
on-site reference; no guest snapshot was rewritten and no website build was needed.

Results: all 86 focused tests pass, including 12 root-identity tests;
`cargo check --release --all-targets` passes. This is native runtime validation,
not new .NET or source-library compiler evidence.
