# Source Object binding — 2026-10-05

Raven `e748b089f` completes the producer-side semantic prerequisite for source Object
ownership. Runtime host selection is available at neoCLR `4e9e4045`. Metadata root
authoring, imported root selection, manifest/driver wiring and source-library execution
remain open. This is not proof that the production System.Object library emits or runs.

## Contract

The .NET-hosted compiler accepts
`MetadataImportOptions(core, providers, sourcePrimitives, useSourceObjectRoot: true)`
for an explicit NeoCLR producer compilation. After all source type declarations exist,
but before member signatures bind, it selects the source System.Object, removes its
provisional bootstrap base and gives implicit source bases that same identity.
Keyword/named Object signatures, array elements, generic-to-Object conversions and
source overrides agree across file order and cold/concurrent semantic queries.

Selection is compilation-owned. Changed options or removed source declarations do not
reuse an earlier snapshot's root. The root must be a public abstract nongeneric top-level
class with no base or instance fields. Invalid/missing roots diagnose with RAVT003.
The .NET target rejects the option and preserves its normal root. Unselected NeoCLR
compilations retain the bootstrap. No imported metadata object is mutated.

The Reflection.Emit backend rejects this configuration with RAVT003; the native backend
rejects with NEOMETA002, preserving output bytes and position. No carrier, CLI projection,
new metadata encoding or writer fallback is introduced. Both adapters remain independent
of the importer. The ordinary driver/ownership manifest does not expose the option yet.

The [Object-root design comparison](explicit-object-root-2026-10-05.md#comparison-and-tradeoffs)
remains applicable: explicit runtime/core identity rather than declaration-name inference.
Raven's special-type model supplies existing language conversions and override matching;
this is a target contract, not a new general language rule. No ordinary .NET compiler
fix was identified for backporting to main in this slice.

## Validation

The pre-change metadata/type-of baseline passes 36 cases. The final focused compiler
run passes 75 cases, including 15 new source-root cases and existing .NET virtual-member,
inheritance and constructor codegen coverage. The native adapter and probe build.
The probe with the existing NeoCLR bootstrap prints:

```text
PASS source Object root semantic identity and native emission boundary
```

The probe checks analysis plus rejection before publication, **not execution**.
The [evidence record](source-object-binding-2026-10-05.json) captures revisions, source
hashes, the bootstrap artifact hash and commands. No runtime or System source changed;
unaffected runtime evidence is reused rather than rerunning the platform suite.

## Next slice

Implement root definition/builder authoring in the independent metadata library. It
must represent a baseless owned Object, preserve CLI Object signature encoding, and
resolve boxing and virtual slots from the selected contract. Add C# definition/builder
parity and round-trip tests before removing the emission guards. Then wire producer and
consumer root identities through the manifest/driver and the runtime host selection,
remove the competing seed root, and execute the unchanged cumulative library consumers.
