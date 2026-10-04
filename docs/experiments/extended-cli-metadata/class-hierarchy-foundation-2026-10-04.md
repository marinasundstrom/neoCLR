# Class hierarchy integration foundation (2026-10-04)

The source JSON gate still rejects DocumentReader.Value's Result<JsonValue, JsonError>
signature. Investigation confirms multiple distinct integration boundaries, rather than
a capability switch to enable: SourceTypePlan admits root classes; the authoring graph
requires core bases; CLI writing hardcodes Object/ValueType/Enum; native writing omits
base relationships. The reader previously rejected base metadata as an unknown field.

This slice completes bounded reader materialization: local nongeneric native bases are
validated, retained in definitions, and resolved through MetadataLoadContext's canonical
views. Standalone NEOX assembly snapshots are now accepted alongside PE/#Neo. References
remain local and metadata-only; generic/external/abstract-base category expansion is
not implied. CLI projection rejects these snapshots rather than dropping inheritance.
The C# API reference documents every changed public surface and limitation.

The runtime already implements the relevant class semantics. The new
`three_binary_assemblies_preserve_base_construction_dispatch_and_identity` regression
assembles a base library, derived implementation and consumer, encodes each independently
to NEOX, decodes them, links explicit dependencies, verifies and runs. It checks base
constructor chaining, base-field mutation through a base view, override dispatch and
identity-preserving casts. Missing the base dependency rejects. This is an assembler/VM
control, **not Raven or Cecil-like-writer inheritance emission**. All eight class
inheritance tests pass. The C# metadata suite passes 144 groups, including the new reader
contract and negative cases.

.NET supplies the expected constructor/field/dispatch behavior; this work adds no new
semantics or metadata format version. No runtime implementation change was needed.
The reader retains the runtime's existing `base` relationship. No performance claim.

## Remaining implementation order

1. Definition/builder parity for base signatures, preserving ownership and exact scope.
   Encode CLI TypeDef.Extends and the existing native base relationship; do not encode
   nominal inheritance as an interface conversion. Preserve constructed arguments and
   external identity when those categories are admitted.
2. Explicit constructor chaining and declaration flags in the authoring contracts.
   Native constructors must not lose their base call; CLI constructors must not always
   target Object. Keep abstract, virtual/override, sealed leaf and closed-family facts
   distinct. Closed Raven families must not become non-inheritable CLI sealed bases.
3. Expand reader/facade resolution for external and constructed bases; feed those facts
   into Raven symbols. Target emission then consumes symbols and artifact identities,
   without reopening importer objects. Admit the target capability only when the full
   authoring/runtime case works.
4. Run equivalent separately compiled base/derived/consumer programs on .NET and NeoCLR,
   then resume the unchanged JSON source group. The broad JSON gate remains incomplete.

No Raven code changed in this slice and no main backport is required. The existing
.NET backend stays the default. The stale guest API snapshot remains recorded; the
host C# facade has a linked manual API reference.

[Source hashes and validation summary](class-hierarchy-foundation-2026-10-04.json).

## Authoring follow-up

The host library now accepts an attached local base through AddClass or a manual
TypeDefinition. CLI Extends and existing native `base` encode the same relationship.
Derived constructors explicitly initialize their direct base, with definite-once flow
checks and no duplicate injected Object constructor. Native field indices include base
storage, while CLI field tokens remain declaration-owned. The expanded C# fixture uses
both base and derived fields and executes on both runtimes. 145 metadata groups pass;
`--class-base-runtime` verifies and runs the generated native PE, returning 42.

The guest API snapshot check remains stale as recorded previously; host API documentation
is in the manual reference. No guest snapshot or unrelated website build was substituted.
Raven's equivalent driver test also uncovered an ordinary constructor binding defect
reproducing on main: initializer resolution was attempted before all source members
were available, and binder re-entry could publish a different constructor symbol.
That compiler correction is isolated independently of the metadata work.

## Raven driver gate completed

Raven `f38dbfb75` now emits ordinary local nongeneric class bases and bound direct
base-constructor calls through its explicit native capability. Both .NET 10 and
neoCLR compile the unchanged `bootstrap/class-base-consumer.rvn` and return 42.
The source puts Derived before Base, initializes separate base and derived fields,
upcasts to a base-typed alias and observes mutation through the original object.
[Commands, hashes and bootstrap catalog](class-base-driver-2026-10-04.json).
Run `bootstrap/verify_class_bases.py --help` for reproducible driver arguments.

The general constructor fix is isolated on Raven's main-based branch
`codex/fix-constructor-initializer-binding`, commit `2416a1646` (base `210d891e0`).
The defect reproduced on main before changes. Twelve constructor diagnostics/runtime
and canonical-symbol tests pass on that branch; 34 focused .NET tests pass on the
integration branch. Main itself has not been changed. The five-source text-stream
library and separate consumers also pass after the changes:
[regression evidence](source-text-streams-after-class-bases-2026-10-04.json).

JSON was retried against these artifacts. It still rejects
`DocumentReader.Value(depth): Result<JsonValue, JsonError>` before publishing output.
Its JsonValue closed family and protected constructor remain outside the admitted
source type/member categories. Next extend those declaration facts through metadata,
reader/facade and Raven, preserving closed-family versus CLI sealed-leaf semantics.
External/constructed bases and class virtual/abstract dispatch remain separate gates;
this local driver success does not establish those or complete JSON.
