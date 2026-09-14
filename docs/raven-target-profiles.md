# Raven target profiles, symbols and emission backends

Direction recorded 2026-09-14. This is an architectural proposal, not a delivered
profile framework or a decision to replace the existing emitter during stabilization.
The author wants Raven to support .NET versions, .NET NanoFramework and neoCLR without
accumulating framework-name exceptions throughout binding and code generation.

## Separate the decisions

| Layer | Responsibility | Examples |
| --- | --- | --- |
| Target profile | Describe the supplied framework and supported contracts | Reference identities; iteration/disposal/propagation roles; generic array shape; nominal Void support |
| Symbols | Represent the target's types and members accurately through common compiler interfaces | Ordinary CLI symbols; a projected generic array symbol where its behavior differs |
| Binding and lowering | Implement Raven language rules using resolved symbols and contracts | Bind foreach against Iterable/Iterator or IEnumerable/IEnumerator |
| Emission backend | Produce code and metadata from validated compiler representations | Existing Reflection.Emit-based path; a possible future direct metadata writer or another backend |

A .NET Target Profile and a neoCLR Target Profile are useful families. Different .NET
versions can supply different references and capabilities within a family. NanoFramework
may need a distinct profile or a constrained .NET profile; choose after comparing its
actual requirements. A new framework target does not automatically require a new
backend. Preserve CLI-compatible metadata and instructions for neoCLR where they meet
its requirements; a backend decision is separate from library naming and API differences.

## Profiles should resolve contracts, not rewrite arbitrary APIs

Build on the existing [runtime contract design](raven-target-contracts.md). Resolve a
profile into immutable, compilation-owned type/member contracts. The binder consumes
those contracts rather than checking target names. Missing or incompatible contracts
produce diagnostics; they must not silently fall back to host assemblies. The semantic
model and language server must see the same symbols as compilation and emission.

Most API calls still bind normally to supplied metadata. Iterable versus IEnumerable
is a configurable role because the compiler lowers iteration syntax. Ordinary library
methods such as ForEach do not each need a compiler mapping when normal member binding
and metadata emission suffice. The importer must still validate the runtime surface it
admits; that boundary is distinct from Raven's language semantics.

## Use distinct symbols where representation or behavior warrants it

The author proposed separate symbol implementations where they remove scattered
special cases. A configured array shape is a concrete candidate: T[] retains vector
storage/signatures while its members may be declared on Array<T>. A dedicated symbol
or composed projection can expose that model through ITypeSymbol/IArrayTypeSymbol
without asking every binder or emitter call site to rediscover the target policy.

Do not duplicate an entire symbol hierarchy per framework. Share normal CLI metadata
symbols; specialize only behavior that differs. Before extracting a projected symbol,
verify symbol equality, original definitions, substitution, metadata owners, conversions,
member lookup and reflection. Additional symbol implementations can otherwise move
inconsistency into those relationships rather than remove it.

The current array slice is an incremental step: it reads public members from the
configured metadata shape, with no hardcoded neoCLR type-name branch in member lookup.
It does not yet introduce a new target-profile abstraction or a dedicated array-symbol
implementation. Keep that limitation explicit while evaluating extraction.

## Evolve emission behind a boundary

The future goal is to remove unnecessary dependence on reflection runtime objects from
shared lowering and emission decisions. First inventory where Type, MethodInfo,
TypeBuilder and temporary proxies enter the current pipeline. Establish typed operands
and backend-neutral responsibilities before selecting another emitter. Reuse existing
IL-building abstractions where they suffice; do not introduce a second implementation
merely because the target profile differs.

Compared with the current Reflection.Emit-centered path, this could make metadata-only
targets easier to support and test. The cost is a substantial refactor with risks around
signature fidelity, generic substitution, debugging information and backend parity.
No new backend or output format is selected here. Validate the existing backend first,
then prototype one bounded path and compare metadata and execution before expanding.

## Work order and validation

1. Complete the current array API slice and keep the stabilization/release boundary.
2. Inventory existing target options and scattered policy checks; separate reusable
   contract resolution from experimental neoCLR semantics.
3. Prototype profile resolution and, if justified, an array-symbol projection at a
   compilation/factory boundary. Validate lookup, completion, conversions and emitted
   signatures against the current behavior.
4. Review general improvements independently for Raven main. Keep neoCLR profiles,
   policies and tests experimental until explicitly reviewed for integration.
5. Design the emission boundary and evaluate backend alternatives as a separate effort.

General correctness fixes still belong on Raven main even when found here. The void-
invocation stack fix is an example: it reproduces under ordinary .NET execution and
was integrated independently. Generic Void as a supported neoCLR type argument remains
part of the experiment. Do not treat modern .NET test success as validation on either
NanoFramework or neoCLR; each claimed target needs its own appropriate checks.


## Targetability and portability — long-term direction, 2026-09-14

The author named the overarching theme **targetability and portability**, clarifying
that microcontroller architectures are potential targets. This is broader than reducing
the class library: it concerns which architectures the toolchain can target and how
compiler/runtime implementations can move across environments. Preserve these inputs
to future design, recognizing that they are far away and depend on sustained effort:

- Raven might be bootstrapped, with the compiler itself rewritten in Raven. This is
  separate from writing neoCLR's System library in Raven, and from the older Neo
  concept-language bootstrap idea. No rewrite is underway or scheduled here.
- If neoCLR attracts sustained development, evaluate additional architectures and
  deployment environments, including AOT/Native AOT approaches and possibly
  microcontroller architectures as compilation/deployment targets. No architecture, hardware family or deployment mode is selected.

**Assistant's proposed distinction:** Treat runtime portability, ahead-of-time
compilation of application code, and a resource-constrained runtime/class-library
profile as separate investigations. Native AOT is not itself an architecture choice;
this note does not commit to using .NET's Native AOT implementation or promise that
it can directly compile neoCLR-targeted applications. Likewise, microcontroller
support does not imply that the full preview library or runtime fits every device.

These possibilities strengthen the reason to separate target contracts from emission
mechanisms without implementing hypothetical targets now. Before a bootstrap effort,
identify the language/library subset needed to compile the compiler, the seed compiler,
reproducibility and behavioral parity checks. Before a new native target, research ABI,
metadata retention, initialization, memory/GC requirements, interop, operating-system
services and toolchain constraints against representative hardware. Evaluate costs and
tradeoffs before choosing a constrained profile or AOT design.

These are open directions, not accepted release requirements. Preserve today's useful
.NET-like semantics and CLI boundary while documenting any later limits explicitly.

## MSBuild project support — future milestone, 2026-09-14

**Author's direction:** Provide proper MSBuild support for targeting neoCLR so Raven
project files can be used to compile projects. This belongs to targetability, beyond
merely evaluating project properties in the compiler or invoking a Python runner.

**Current boundary:** The [normal compiler path](raven-target-compilation.md) accepts
`.rvnproj` inputs and produces target PE metadata/IL; separate import and runtime
verification follow. The [.14 local tools](local-tools-20260914.md) validate this path.
They do not yet provide a complete neoCLR MSBuild SDK or establish `dotnet build`
and project-reference support for neoCLR applications.

**Assistant's proposed approach:** Reuse Raven's project/compiler integration, with
versioned neoCLR build assets supplying the target reference pack and contracts.
MSBuild should orchestrate reference resolution, Raven compilation, the current
import/verification stage and output layout. Keep importer execution behind its own
target so a later native loader can replace it without changing project semantics.
Design-time evaluation should supply the same references/contracts to completion
without executing the program or requiring runtime import to discover symbols.

This follows the existing .NET/MSBuild separation: project SDKs supply imported
props/targets, and target Inputs/Outputs enable incremental work. Sources: Microsoft's
[project SDK documentation](https://learn.microsoft.com/en-us/visualstudio/msbuild/how-to-use-project-sdk)
and [incremental-build documentation](https://learn.microsoft.com/en-us/visualstudio/msbuild/how-to-build-incrementally),
reviewed 2026-09-14. This is missing toolchain integration, not a proposed runtime or
IL divergence from .NET.

The author clarified that the initial goal is a useful MSBuild build system, not the
entire .NET build experience and not integration with `Microsoft.NET.Sdk`. Use familiar
Raven project files with a small set of neoCLR `.props`/`.targets` build assets. No
custom project SDK, NuGet target-framework integration or framework moniker is required
for this first slice. The host .NET installation needed to run Raven/MSBuild is separate
from the runtime/library targeted by the guest program.

Using MSBuild directly reuses project evaluation, task orchestration and diagnostics
without inheriting .NET deployment and runtime assumptions. The cost is defining and
maintaining the few build targets needed by neoCLR. A standalone build script remains
a useful fallback, but should not be the only way to compile Raven project files.

**Initial proposed acceptance:** Build one `.rvnproj` through MSBuild using the selected
neoCLR references, invoke Raven and the current import/verification stage as appropriate,
and produce documented outputs with useful diagnostics. Build does not execute the guest
program. A failed compile/import must fail the build; editor and build configuration must
agree on the target references. Validate this from the installed toolchain, including
rejection of unavailable host APIs. This does not require a new emission backend.

**Optional later slices, driven by need:** Incremental inputs/outputs and Clean/Rebuild;
project-reference build order and compatibility checks once library importing is ready;
versioned distribution/restore of build assets. These are separate follow-ups, not a
promise of full MSBuild/.NET SDK feature parity or prerequisites for the initial slice.

This is future work after the stabilization checkpoint, not a new requirement for the
already validated local build. Reusable Raven fixes remain candidates for main after
independent testing; neoCLR-specific build assets and tests remain experimental.


**Implementation checkpoint:** The [minimal standalone MSBuild slice](raven-msbuild.md)
now supplies props/targets and a single-application project template. It reuses the
installed .14 Raven compiler; all implementation is in neoCLR's build/distribution
assets. Later build-system capabilities listed above remain separate work.
