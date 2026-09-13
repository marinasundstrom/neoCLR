# Raven application types

## First slice (2026-09-13)

The source import bridge now admits non-generic application classes and value types,
instance fields, constructors, property accessor bodies and ordinary instance methods.
Classes retain identity through calls and `ArrayList<T>` storage. Values copy through
assignment and collection insertion. The Raven spelling for the latter is `struct`;
this does not change the legacy Neo frontend.

Run [application-types.rvn](experiments/raven-target/samples/application-types.rvn)
with the source bridge and the experimental Raven branch. Its output is
`42`, `99`, `7`, `42`, `7`, on separate lines. Preview 4's published bridge/SDK do not
contain this slice; their assets remain unchanged.

```
python3 docs/experiments/raven-target/verify_application.py /path/to/Demo.rvnproj \
  --raven /path/to/Raven --runtime /path/to/neoclr
```

The project and declaration assembly can come from the Preview 4 demo. Build the
Raven compiler and then the bridge as described in the [integration guide](experiments/raven-target/README.md).
The check also verifies fresh saved-source compilation, emitted source-map lines and
rejection of type initializers and readonly fields before execution.

## CLR comparison and implementation layers

This follows the existing [type-category decision](roadmap.md#type-model-migration-directive-2026-09-12) and
[interface contract research](raven-interface-contract.md#netcli-comparison-and-placement):
ECMA-335 Partition I §8 defines value/object behavior, Partition II defines TypeDef,
Field and MethodDef metadata, and Partition III specifies `newobj`, `call`, `callvirt`
and field access. The compiler still emits these ordinary CLI constructs. This slice
fills a neoCLR importer gap, rather than claiming an improvement over CLR behavior.

The bridge derives application declarations from supplied metadata instead of adding
per-application entries to the library catalog. It uses token-derived internal type
and field names, with method-token/IL-offset mappings preserved after body relocation.
Property accessors execute; application PropertyInfo metadata projection is not yet
included. No new metadata encoding or instruction is required.

Nominal class constructors use the existing runtime class allocation path. Value
constructors currently lower to a helper that initializes a value local, calls the
constructor body through its managed address, and returns a copy. This follows CLI
value initialization while keeping the older runtime constructor-capability protocol
separate. Value instance no-result methods adapt to the existing unit-return receiver
convention internally; the input CLI still has a no-stack-result void return.

Unsupported metadata fails explicitly: generic application definitions, static
initialization/storage, enums, custom/explicit layout, nested definitions, readonly
fields, inheritance and interfaces are outside this first slice. Reachable bodies
retain the existing bounded opcode and exception-handler restrictions. This is not a
general-purpose PE loader or a full metadata verifier. No performance claim is made.

## Raven findings and target policy

Two compiler issues surfaced: constructing target-library generics with application
TypeBuilder arguments crossed MetadataLoadContext boundaries, and rewritten generic
member signatures incorrectly created an external self-assembly reference. The
experimental Raven branch (`cbd87efa8`) now preserves generic signatures without host type loading
and keeps application type references scoped to their defining module. Focused metadata
tests cover both class and value arguments. The compiler uses
[Type.MakeGenericSignatureType](https://learn.microsoft.com/en-us/dotnet/api/system.type.makegenericsignaturetype?view=net-10.0)
(documentation consulted 2026-09-13) for the mixed metadata/source signature.
Microsoft documents this as a reflection signature facility; compatibility with our
persisted emission path is demonstrated by the tests, not assumed from that description; the neoCLR execution check supplies the
end-to-end behavior evidence. Validation passed 12 focused Raven metadata tests,
all 49 existing saved-project cases, and the four application checks.

The author emphasized that this work should also stabilize Raven as a versatile
compiler, with nanoFramework targeting as relevant prior experience. Distinguish:

- Framework-neutral compiler bugs, candidates for separate review/upstreaming.
- Target-configurable contracts, such as Iterable/Iterator versus IEnumerable/IEnumerator.
- Deliberate neoCLR semantics, particularly generic Void and Result-based errors.

Making neoCLR work with Raven remains the primary objective. All Raven edits stay on
its experimental feature branch; ordinary .NET support must retain its behavior.
`Task<Void>` and its no-payload completion require a future async design, including
language return handling, emitted signatures, awaiter contracts and runtime suspension.
This records a requirement, not an async implementation or a chosen task API.
