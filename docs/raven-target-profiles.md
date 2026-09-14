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
