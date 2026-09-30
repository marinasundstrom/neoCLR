# Extended CLI metadata for neoCLR

**Status: exploratory design, 2026-09-30.** Author-selected work on
`codex/extended-cli-metadata`, based on main `3ac2ae7c5c8434d743d1b866d87e6b21f48e4de0`.
This document starts format development. A standalone
[framing experiment](../experiments/extended-cli-metadata/README.md) now has a reader,
writer and inspector. A bounded [PE/CLI container probe](../experiments/extended-cli-metadata/README.md#pecli-container-probe-2026-09-30)
now transports #Neo and tests ordinary reader behavior. No production format, runtime
feature or Raven integration is implemented. Production numeric encodings remain unassigned.

## Purpose and inputs

Define a CLI-derived metadata format that can carry neoCLR semantics across compiler,
loader and introspection boundaries without reconstructing them from Raven-specific
carrier names. Later Raven integration is an explicit goal. Structural types are a
first-order requirement, reaffirmed by the author during this work.

Inputs are the [metadata proposal](../proposals/metadata-format.md),
[delegates evolved](../proposals/delegates-evolved.md),
[Callable proposal](../proposals/callable-interface.md),
[Self contracts](../self-types.md), and [current bridge](../raven-cli-bridge.md).
The uncommitted `docs/proposals/structural-types.md` draft in the author's
`codex/structural-types` checkout was also read on 2026-09-30. It remains there,
unmodified; this document captures the requirements used here without copying its
unfinished text or adopting all its APIs. Its arrays, tuples, synthesized members,
interface conformance and independent Object compatibility extend the earlier
metadata proposal's Function/union/intersection focus.

The proposals contain alternatives, including different Callable invocation APIs.
Neither dynamic Invoke nor Callable<F> is selected here. Metadata must permit a
later decision without baking those names into its binary grammar.

## Current baseline and comparison

Main has native Self and nominal delegates. `src/metadata.rs` is the native runtime
model, not a CLI binary reader. `docs/experiments/raven-target` translates CLI artifacts;
its `ApplicationSpecialization.cs` recognizes the configured Self marker. Structural
Function runtime behavior remains on a separate branch. Existing nominal tuple and
array contracts must not silently acquire structural identities when read.

Primary references checked 2026-09-30:

- [ECMA-335, sixth edition, June 2012](https://ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf),
  II.22.21, II.23.2 and II.24.2: CLI tables, signatures and streams are the physical
  baseline. GenericParamConstraint already admits TypeSpec references; the proposed
  change is richer expression semantics and validation, not merely another index kind.
  CLI signatures already construct arrays and function pointers. A managed Function
  value with captures is not thereby equivalent to FNPTR. Existing identifier strings
  use UTF-8; retaining those heaps does not settle neoCLR runtime String semantics.
- [C# nullable reference types](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/null-safety/nullable-reference-types):
  shipped compiler analysis distinguishes nullable references without creating a
  distinct CLR runtime type. Enforced nullable storage would be a deliberate neoCLR
  divergence, requiring runtime checks and initialization rules.
- [WebAssembly custom sections](https://webassembly.github.io/spec/core/binary/modules.html#custom-section),
  specification as retrieved: named, length-delimited custom data is ignored by core
  semantics. This is useful framing experience, but required neoCLR semantics cannot
  safely be treated as ignorable annotations.

A bounded [.NET reader and Cecil comparison](../experiments/extended-cli-metadata/pe-validation.json)
now validates fixture inspection and exposes Cecil stream stripping on rewrite.
Runtime execution and broader reader compatibility remain evidence gaps. Do not claim universal reader compatibility,
CLR execution compatibility, smaller files or faster loading from this design review.

## Provisional architecture

Retain the CLI container, ordinary declaration tables, heaps and module-local tokens
where they preserve meaning. Investigate a versioned `#Neo` stream for additional
relationships and feature requirements. Extend the signature grammar for type
expressions; avoid a second competing signature model in an overlay.

Separate three responsibilities:

1. Physical reader: bounds-checked rows, blobs, handles, version and capability data.
2. Semantic resolver: nominal identity, structural construction, substitution,
   conformance and supported operations, independent of Raven syntax.
3. Runtime and compiler adapters: validation, layout, GC, dispatch, lowering and
   introspection projection. Encoding a type does not implement its execution rules.

Compared with retaining carrier types and attributes, this preserves structural
identity across frontends and makes required semantics explicit. It costs new readers,
writers, verifier rules and tooling. Compared with an unrelated format, it reuses
ordinary metadata relationships at the cost of CLI-specific physical constraints.
Adding unknown tables to #~ is less attractive initially because row layout/index
calculations must remain interpretable. A sidecar avoids container changes but brings
pairing and distribution hazards. These remain alternatives if experiments reject #Neo.

## Structural types and identity

The logical grammar below is illustrative, not Raven syntax or assigned binary codes:

```text
Type = Primitive(kind)
     | Nominal(resolvedDefinition, arguments)
     | TypeParameter(owner, index) | MethodParameter(owner, index)
     | Self(contractContext)
     | Array(element, shape)
     | Tuple(elements)
     | Function(parameterModesAndTypes, result, callingSemantics)
     | Union(alternatives) | Intersection(requirements)
     | Nullable(element)
     | ByRef(element, accessMode)
```

| Form | Identity and unresolved semantics |
| --- | --- |
| Nominal | Assembly/module scope and declaration identity, plus arguments; never name alone. Named unions and nominal callable types stay distinct from structural expressions. |
| Array | Element and semantic shape; assess reuse of CLI array signatures. Fixed-size forms and storage strategies remain separate research. Do not infer Object inheritance or covariance from an array opcode. |
| Tuple | Ordered element types; names are proposed annotations. No required synthetic Tuple TypeDef. Existing nominal System.Tuple is not automatically the same type. |
| Function | Ordered parameters, passing/access modes, result and calling semantics. Target method, receiver and capture state belong to the value, not structural type identity. Variance requires separate assignability rules. |
| Union/intersection | Candidate normalization flattens like operators, removes duplicates and canonicalizes order. Subtyping reductions, empty forms, recursive forms and null interactions remain open. Reject unsupported forms instead of guessing. |
| Nullable | Explicit node; relationship to unions with null and nullable value representation remains open. Preserve distinctions until equivalence is specified. |
| Self | Context-bound type expression, preserving conformance ownership and substitution. Neither a globally named marker type nor an ordinary unconstrained parameter. |

Structural identity must survive different token numbers and independent emission in
two assemblies. Interning uses resolved nominal identities and normalized structure,
not blob bytes, TypeSpec row numbers or display strings. Generic parameter binders
must remain scoped during comparison and substitution. Hash collisions require full
identity comparison. Apply depth/size limits before normalization; reject unsupported
cycles. Initially avoid distributive union/intersection expansion and its size explosion.

A TypeSpec token is an optional physical handle for an expression, not its semantic
identity. The same structural expression may occur inline in many signatures. Nested
forms must work in parameters, results, fields, locals and generic arguments, with
position-specific validity checks. Any extension marker must work in each relevant
signature grammar, not only top-level TypeSpec blobs.

Object compatibility, boxing, layout, interface conformance and assignability are
separate properties. Structural does not mean reference type, universal Object subtype,
or arbitrary duck typing. Metadata may describe a form before the runtime supports it;
an executable consumer must reject capabilities it cannot implement.

## Synthesized members and conformance

Structural types may expose operations such as ArrayLength, TupleElement(index),
TupleDeconstruct and FunctionInvoke without MethodDef or Field rows. Use a semantic
member identity consisting of the constructed owner type, operation kind and applicable
indices/signature arguments. Display names are not intrinsic identifiers.

For persisted references, investigate a `StructuralMemberRef` extension record pointing
to an owner TypeSpec and a versioned operation identity. This is a candidate handle,
not a claim that all structural members require metadata tokens. Do not allocate fake
CLI method tokens or use unchecked strings as executable operation identities.

The type system supplies member discovery and nominal interface conformance; a backend
maps supported operations to neoIL or validated helpers. Conformance rules must identify
the library contract by resolved identity and version, not merely a familiar interface
name. User-defined conformance declarations, if supported, require explicit evidence and
coherence rules. Metadata must not allow an arbitrary file to assert an unsafe runtime
intrinsic or GC layout. Constraints such as “is a Function type” need a semantic predicate;
they are not necessarily a fake nominal base type.

The initial codec can round-trip such requirements without enabling invocation. Binding,
verification and interface dispatch require a later coherent consumer slice.

## Declaration extensions and wire-format questions

Candidate records are RequiredFeature, FreeFunction, NominalUnion, UnionCase and
StructuralMemberRef. Reuse MethodDef for a free function's physical body/parameters if
<Module> ownership remains lossless; namespace identity is semantic. Cross-assembly
references and overload identity must be tested before choosing this over FunctionDef.
Union cases need owner-scoped declaration identity and payload signatures; a structural
Union has alternatives, not nominal case declarations.

Prototype an envelope with magic, major/minor version, required capabilities and a
length-delimited directory. Each entry needs kind, schema version, flags, offset, length
and sufficient row framing to skip optional unknown data. Decide exact widths, alignment,
endianness, heap-index conventions and size limits in the codec experiment. Prefer
ordinary CLI token/heap references where appropriate; validate their kinds and bounds.
Extension handles occupy a separate namespace until an explicit IL operand scheme exists.
Do not promise row identifiers are stable across rebuilding or linking.

Investigate a single signature extension prefix with a versioned subtype and bounded
payload instead of claiming many unallocated CLI element codes. Reserved codes are not
vendor allocations. No numeric prefix is selected here. Unknown required signature
forms must fail semantic resolution even if framing lets an inspector skip their bytes.

## Compatibility and failure behavior

Distinguish container inspection, semantic reading, rewriting and execution. An old
reader ignoring #Neo is only inspection evidence. Required features must be checked
before a neoCLR consumer binds or executes affected code. Optional annotations may be
skipped; unknown required features, conflicting declarations, malformed lengths, bad
references and unsupported versions require deterministic errors.

A #Neo flag alone cannot force an unaware CLR loader to reject an otherwise runnable
image. The executable profile needs an independently tested recognition/rejection
strategy before shipping. Until then, artifacts are experimental neoCLR inputs only;
do not market them as safe dual-target binaries. Rewriters must preserve and remap
extension references or refuse semantic rewriting, never silently strip required data.

The [marked recognition experiment](../experiments/extended-cli-metadata/README.md#marked-artifact-recognition-2026-09-30)
now exercises a metadata-root discriminator and digest over exact stream contents.
Cecil preserves the discriminator while stripping #Neo, yielding deterministic refusal
by the aware inspector. Explicit expected-extended input is also required to reject an
image after both indicators disappear. This is a provisional consistency contract,
not authentication, method-body verification or an unaware CLR execution gate.

Keep the current bridge profile separate from the future native profile. Native output
must not fall back to nominal Func/Tuple carriers when a consumer lacks structural
support. Downgrade is an explicit, potentially lossy export with documented limits.
UTF-8 identifiers do not authorize reinterpreting CLI #US strings as neoCLR strings;
literal transport needs its own explicit contract.

## Development sequence and acceptance

| Step | Deliverable and exit evidence | Status |
| --- | --- | --- |
| 0 | Main-based branch, proposal inventory, structural requirements and compatibility questions | This document |
| 1 | Standalone experimental envelope/signature codec and inspector; precise experimental byte schema and golden fixtures | [Framing and structural payloads implemented](../experiments/extended-cli-metadata/README.md); bounded PE embedding/reader probe passes; real CLI reference resolution pending |
| 2 | Structural identity/resolution tests across independently emitted modules; synthesized-member and conformance references | [Reference/identity and structural-member fixtures pass](../experiments/extended-cli-metadata/README.md#structural-member-evidence); actual compiler modules and conformance pending |
| 3 | One native consumer slice using Self and structural Function signatures; verifier, invocation, GC and introspection evidence | Planned; depends on explicit runtime feature work |
| 4 | Raven reader/symbol adapter, then writer/backend integration through target capabilities | Later integration |

Step 1 should cover nested Function/Tuple/Array/Union/Intersection/Nullable and contextual
Self without pretending all are executable. Test encode/decode/encode determinism,
known optional versus required extensions, truncated/overflowing/overlapping sections,
wrong token kinds, out-of-range heaps, invalid generic binders, duplicate records,
unknown signature forms and resource limits. Include conventional CLI fixtures unchanged.
Use at least two differently numbered modules for identity tests in step 2; compare
actual behavior of pinned System.Reflection.Metadata and the bridge's reader separately.
Record tool versions and artifact hashes. The isolated codec now has
[30 passing focused tests](../experiments/extended-cli-metadata/README.md#structural-member-evidence);
Standalone fixtures now check consumer-local reference renumbering against a host catalog;
the bounded PE probe now covers two conventional readers. Actual CLI-reference
resolution and compiled cross-module checks remain planned.

Before production encoding, resolve normalization and position rules, capability/version
negotiation, unsafe downgrade prevention and cross-module reference identity. Later
runtime work additionally needs layout/GC, mutable-array variance, Function lifetime,
byref escape and constraint validation. No benchmark is required without a performance
claim or measured concern. Ordinary documentation changes do not require a website build.

## Raven integration boundary

neoCLR owns the versioned format and language-independent semantic contract. Raven owns
symbol projection, source diagnostics and emission under `RavenTargetPlatform=NeoCLR`.
The bridge owns temporary transport and conversion; native validation owns acceptance
regardless of compiler behavior. Runtime Contract configuration must eventually negotiate
metadata versions, features and canonical core identities explicitly. Property names for
new capabilities remain undecided; ordinary .NET targeting retains its existing behavior.

First import fixtures into Raven without emission, preserving structural equality,
synthesized members and nominal distinctions. Then emit and reload the same model before
connecting executable lowering. Compare bridge/native semantic results for supported
Self and Function examples. Keep native and bridge artifacts independently identifiable.
Publish paired Raven compiler documentation, neoCLR integration notes, changelogs and
matching tool revisions when compiler behavior changes. This planning slice changes no
Raven source/configuration and has no tested Raven bundle revision.

## Reader and writer support on .NET and neoCLR

**Author direction, 2026-09-30:** eventually provide metadata reader and writer support
for both .NET and neoCLR. This is a shared format/tooling requirement, not merely an
import path from Raven to the current runtime. Exact package names and implementation
languages remain provisional. The Python codec is a research harness, not the proposed
shipping library on either platform. The first [.NET library slice](../../tools/metadata/README.md)
now covers framing, owned sections, structural signature syntax/context validation,
reference tables, explicit-catalog structural identity and synthesized-member contracts. Physical CLI declaration
resolution, Raven adapters and neoCLR guest library support remain pending.
The catalog resolver reuses the provisional identity rules above: unlike a CLI token
local to one module, its resolved key carries host-assigned assembly/module scope and
declaration ownership. This permits reference renumbering without conflating binders,
at the cost of requiring authoritative, compatible host catalogs. The UUID scopes
are not a replacement for production CLI assembly identity.
[95 shared vectors](../experiments/extended-cli-metadata/dotnet-references-validation.json)
validate this bounded model; no execution, performance or production loader claim is made.

The .NET synthesized-member layer now reproduces the Python contracts in
[49 shared vectors](../experiments/extended-cli-metadata/dotnet-members-validation.json).
Unlike ordinary CLR reflection members backed by declaration metadata, these descriptors
derive from structural owners and carry no MethodDef or dispatch target. This keeps
reference renumbering out of identity and avoids invented carrier methods, at the cost
of a separate compiler/runtime lowering contract. Array length uses the already-tested
structural branch’s native unsigned/UIntPtr result, not .NET Array.Length’s Int32.
No runtime semantics change is introduced by porting this existing prototype.

The author clarifies the concrete library consumers:

| Library consumer | Reader integration | Writer integration |
| --- | --- | --- |
| Raven compiler on .NET | Symbol loader imports assembly metadata into Raven symbols, including structural types, synthesized members and owner contexts | Code generation supplies declarations, signatures and generated bodies to the metadata/image writer, which validates and fixes up references |
| neoCLR programs and tooling | Load assembly metadata into the shared Introspection model through a metadata-backed provider, resolving dependencies explicitly | Emit constructs assemblies through metadata/image builders and writes them for later inspection or loading |

The format libraries provide reusable reader/writer machinery. Raven-specific symbol
construction and code generation remain compiler adapters, not dependencies of the
neoCLR libraries. The neoCLR provider projects the same metadata into Introspection;
it must not expose Raven compiler symbols or require Raven to be installed. Metadata
writing owns encoding, handle assignment and fixups; Raven code generation owns source
lowering and method-body generation. An eventual Emit layer supplies generated bodies
and descriptions through the corresponding neoCLR writer.

“Load assemblies into Introspection” means opening their metadata and producing usable
assembly/module/type/member descriptions. It does not implicitly execute initializers
or realize executable types in RuntimeContext. The existing separation between metadata
inspection and explicit runtime loading remains applicable. Likewise, emitting an
assembly does not automatically load it. Structural descriptions must remain available
even where a particular runtime execution capability is unsupported, with unsupported
required metadata semantics diagnosed rather than silently discarded.

The native implementation and guest library may share machinery, but a Rust-only
internal loader does not satisfy the Introspection-loading and assembly-emission library
requirements. Conversely,
a .NET package called by the development bridge is not a library running on neoCLR.
Keep both distinctions visible in acceptance evidence.

### Shared contracts and platform adapters

Use one versioned wire specification and shared golden/malformed fixtures. Separate:

1. **Byte access and encoding:** bounded buffers/streams, integer encodings, offsets,
   tables, heaps, tokens, diagnostics and resource limits. No assembly loading.
2. **Raw metadata model:** declarations, signature expressions, required features and
   extension relationships. Physical handles stay distinct from resolved type/member
   identities. Preserve information even when a high-level API does not expose it.
3. **Resolution and semantic projection:** explicit dependency resolver, generic/Self
   owner contexts, structural identity and the Introspection adapter. No implicit
   host reflection or executable dependency loading to answer metadata questions.
4. **Construction:** builders with symbolic handles, validation and final token/heap
   assignment. Emission remaps every dependent extension reference. Loading the output
   and executing it remain separate operations.

The raw reader should work without native runtime layout/GC services. The semantic
adapter may use those services where actual runtime support is needed, but offline
inspection must not instantiate them. This follows the existing
[Introspection/Reflection/Emit separation](../introspection-design.md#emit-and-cross-origin-composition)
and [managed foundation boundary](../system-runtime-assembly.md).

On .NET, prefer evaluating System.Reflection.Metadata/PEReader and metadata builders
for conventional structures, with an explicit neoCLR extension codec and lossless
model above them. The existing probe is evidence for conventional inspection only.
It does not establish that standard signature decoders, emitters or Roslyn's importer
understand structural neoCLR types. Raven needs an adapter to the shared semantic
contract; it must not reverse-engineer nominal carriers to recover native semantics.

Cecil remains a useful independent comparison reader, but the observed removal of
#Neo on rewrite rules out an unmodified Cecil read/write pipeline as our writer.
Simply copying an opaque stream back after rewriting is also insufficient if tokens
or heaps moved. Unknown optional data may be preserved when copying an unchanged image;
semantic rewriting must refuse unknown reference-bearing schemas unless it can remap
or prove their independence. Required features must always be understood.

For neoCLR, compare a native codec with a Raven-authored portable managed codec using
a small byte-buffer API. A shared portable implementation would reduce duplicated logic
but depends on guest language/library maturity and the bootstrap. Independent .NET and
native implementations allow earlier compiler/runtime work and valuable cross-checks,
but cost more maintenance. Start with shared contracts and conformance tests rather than
committing now to a source-sharing mechanism, FFI boundary or final assembly split.
No extra mandatory metadata package belongs in System.Runtime merely for offline tools.

### Staged delivery and acceptance

- First close artifact recognition and actual PE-reference binding gaps using the
  current harness. Establish a minimal format profile, diagnostics and fixtures that
  both readers can implement without copying prototype accidents into public APIs.
- Then build the first .NET reader/writer library slice with a standalone executable
  consumer, using the same fixtures. Keep Raven integration as a separate adapter and
  use the .NET writer as one producer of runtime-independent test images.
- Add native neoCLR reader/writer support for that same bounded profile. Test both
  .NET-write → neoCLR-read and neoCLR-write → .NET-read, plus each local round trip.
  Verify semantic equivalence after legitimate row/heap renumbering; require byte
  equality only for a documented canonical output mode.
- Establish a guest-accessible library on neoCLR, either by porting a portable core or
  exposing validated native services through a managed API. Acceptance must include a
  neoCLR program loading an artifact into Introspection, inspecting its structural
  type/member descriptions, emitting an assembly and reopening it without invoking
  .NET. Pair public API
  documentation with that implementation, not with speculative names in this plan.
- Integrate Raven’s symbol loader and code generation as explicit consumers of the
  .NET libraries; integrate neoCLR Introspection loading and Emit through its libraries. Run cross-produced artifacts through both implementations, including
  malformed inputs, unknown required features, structural members and owner contexts.

Track ordinary CLI compatibility, extension understanding, rewriting preservation and
execution as four separate results. The present PE probe proves only the bounded first
category plus transport to the Python extension inspector. It does not complete either
platform's reader/writer library. This direction does not move structural runtime work
to main or change Raven's ordinary .NET target.


## .NET API direction and potential Raven port (2026-09-30)

**Author clarification:** establish a good enough .NET reader/writer API for later
Raven compiler integration. A pending Metadata Introspection API may be built on a
Raven port of that reader/writer API. The port is a potential implementation route,
not a completed decision or a claim that guest APIs exist.

The .NET reference-profile facade now gives callers Read, Create and Write entry
points and an owned typed document. It enforces section composition before exposing
syntax, bindings and member references, then leaves catalog resolution explicit.
This is a compiler-facing foundation; the bounded one-root experiment is not yet a
complete assembly reader/writer. [Conformance evidence](../experiments/extended-cli-metadata/dotnet-profiles-validation.json)
covers 36 profile vectors plus independent typed construction and ownership checks.

Compared with the .NET layers discussed in the existing research, raw metadata
reading/building belongs below compiler symbols and reflection-style objects. Our
facade adds neoCLR profile validation and explicit structural contracts at that lower
layer. Owned data avoids reader-lifetime coupling for the current small bounded
format, at the cost of copying payloads; no performance advantage is claimed. The
separation allows Raven symbols and Metadata Introspection to project the same decoded
model without making either model the wire format.

Before calling the .NET API ready for Raven integration, demonstrate:

1. Recognized artifact input and actual CLI declaration/dependency binding, with
   diagnostics that preserve useful failure context and no silent semantic fallback.
2. Assembly-level declaration/signature coverage required by a bounded compiler
   consumer; stable handles and explicit remapping/fixups for writer construction.
3. A tested Raven importer/emitter adapter using those contracts, with ordinary .NET
   targeting unchanged and structural experiments still isolated on feature branches.
4. Loss/unknown-data policy and deterministic output tests for the supported editing
   workflow. Byte-preserving round-trip alone is not semantics-preserving rewriting.

These are readiness criteria, not implemented features or a new public package promise.
For a potential Raven port, preserve the shared wire vectors, limits, validation
phases, ownership semantics and catalog contracts. Map .NET spans/collections/errors
to tested Raven equivalents and neoCLR UTF-8 contracts; do not mechanically import
host reflection types or assume .NET API spellings are available in the guest.
Metadata Introspection would sit above that port as a projection/provider, while
assembly emission would consume the writer. The exact public Raven surface, port
strategy and native boundary remain open. Next bounded .NET work is artifact
recognition/extraction before physical CLI binding.


## Cecil-inspired object model direction (2026-09-30)

**Author suggestion:** “We perhaps should model the metadata API:s on Cecil.”
**Assistant assessment:** use Cecil as the provisional ergonomic model for the
compiler-facing layer, retaining the tested byte codecs beneath it. This is an API
direction to prototype, not an author commitment to source compatibility, a Cecil
fork or a new dependency. No proposed types below are implemented in this slice.

### Primary-source comparison

Reviewed Cecil **0.11.6** (the version used by our probe), retrieved 2026-09-30:

- [AssemblyDefinition](https://github.com/jbevain/cecil/blob/0.11.6/Mono.Cecil/AssemblyDefinition.cs)
  exposes assembly identity, modules, entry point, creation, read and write operations.
  This is a useful discovery path for importer/emitter users.
- [ModuleDefinition](https://github.com/jbevain/cecil/blob/0.11.6/Mono.Cecil/ModuleDefinition.cs)
  separates reader/writer parameters, supports resolver configuration and imports type,
  field and method references into a destination module with a generic context.
- [TypeReference](https://github.com/jbevain/cecil/blob/0.11.6/Mono.Cecil/TypeReference.cs)
  carries scope/module information and delegates Resolve to its module, distinguishing
  a use-site reference from its definition.
- [.NET 10 MetadataReader](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.metadata.metadatareader?view=net-10.0)
  provides the lower-level CLI metadata reading baseline. Raw metadata access and a
  compiler-friendly editable object model solve different layers of the problem.

Our existing [Cecil rewrite evidence](../experiments/extended-cli-metadata/recognition-validation.json)
shows that stock 0.11.6 strips #Neo. Adopting its API organization does not establish
that its writer can preserve neoCLR semantics. The new .NET artifact adapter passes
[50 shared cases](../experiments/extended-cli-metadata/dotnet-artifacts-validation.json),
but still inspects only a bounded unsigned IL-only PE32 layout and local profile.

### Proposed layer and type mapping

Names are illustrative design vocabulary, not public API promises:

| Concern | Proposed approach | Existing implementation underneath |
| --- | --- | --- |
| Assembly and module model | AssemblyDefinition/ModuleDefinition with explicit ownership and read/create/write entry points | Artifact recognition plus profile codec; full declaration model absent |
| Nominal types and members | Separate references and definitions; scoped identity; generic owners retained | Explicit catalog keys and resolver |
| Reference import | Destination-module import with generic context and deliberate remapping | Currently caller-managed local indices |
| Structural types | First-class tuple, Function, union/intersection, Array/ArrayRef and Self type-reference forms | TypeExpression syntax and resolved structural keys |
| Synthesized operations | Structural member references/descriptors; no invented MethodDef | StructuralMembers codecs and derived contracts |
| Reading/writing configuration | Explicit resolver, input profile, limits and unknown-data policy | Bounded readers and required-schema rejection |
| Introspection | Projection/provider over metadata objects; runtime binding remains separate | Potential future Raven port; no guest implementation |

Do not force structural types to acquire fictional nominal declarations, or make every
structural Resolve return a TypeDefinition. Separate definition lookup from structural
identity/contract resolution. The string-based TypeExpression prototype should be
adapted behind typed object-model forms rather than copied directly into Raven symbols.
Likewise physical tokens are serialization addresses, not cross-module object identity.

### Tradeoffs and unresolved choices

Cecil-like navigation and reference import should reduce compiler adapter boilerplate;
this is a hypothesis to validate with an actual consumer. A mutable graph costs owner
invariant maintenance, cache invalidation and write-time fixups. A provisional split is
owned read views plus explicit editing/building that yields a validated snapshot; an
exact Cecil-style mutable graph remains an alternative to compare in the prototype.
No performance advantage is claimed, and deferred loading/lifetime behavior is not yet
selected. Existing eagerly owned bounded documents remain valid low-level APIs.

Alternatives: expose raw handles/codecs only (simpler core, more work for both Raven
and Introspection), or wrap/fork Cecil (reuse conventional CLI support but additional
extension-preservation, identity, licensing maintenance and port dependencies). Prefer
an independent Cecil-inspired surface provisionally, with an internal conventional
metadata adapter selected separately after real declaration tests. Do not change
package dependencies merely because the names/navigation are familiar.

For the potential Raven port, preserve semantic operations and shared fixtures while
adapting collections, errors, lifetimes and strings to tested Raven/neoCLR UTF-8 APIs.
Do not couple the portable object model to System.Reflection.Type or host assembly
loading. .NET ergonomic familiarity does not require identical implementation layers.

Next bounded prototype: assembly/module identity and nominal definition/reference
navigation backed by actual CLI metadata, with explicit dependency resolution and a
small importer consumer. Then validate destination-module reference import and one
supported writer path before claiming Raven integration readiness. Larger mutation,
resource ownership, diagnostics, multi-module and full assembly emit contracts remain
open and need deeper consumer evidence before settling the public API.


## Initial Cecil-inspired declaration model (2026-09-30)

Implemented in `NeoCLR.Metadata.Experimental.Model`: owned read-only AssemblyDefinition,
ModuleDefinition, TypeDefinition and definition-backed TypeReference. ReadAssembly
combines the recognized artifact/profile layer with System.Reflection.Metadata for
actual manifest/module and TypeDef rows. The existing Cecil/.NET comparison above
applies; no new dependency or change to runtime semantics is introduced.

This bounded experiment selects an owned read view first, not a mutable Cecil clone.
It preserves names, version, module MVID, generic arity and nesting; references resolve
to the exact definition within their module snapshot. Names/tokens/MVIDs are not
silently promoted into complete binding identities. Structural profiles remain attached
but their host-catalog UUIDs are not inferred from physical CLI metadata. The cost is
copying and separate future builder/import work; the benefit is a simple lifetime and
ownership contract for a compiler importer. No performance claim is made.

A deliberate initial difference from Cecil-style navigation is that Types is a flat
TypeDef view including nested declarations and the module pseudo-type. DeclaringType
provides nesting links. These four API types are documented in the
[host reference](../../api-docs/experimental-metadata.md#model-namespace).
[Consumer evidence](../experiments/extended-cli-metadata/dotnet-model-validation.json)
uses generated real CLI generic/nested Unicode declarations, compares ordinary reader
snapshots and confirms corrupt tables fail even with a consistent #Neo digest.

Scope: 4096 TypeDefs maximum, a cumulative 4 Mi UTF-16 code-unit declaration-name
budget (including repeated uses), and the artifact reader's existing unsigned IL-only PE32
limits. It reads one manifest module, not linked netmodules. TypeReference currently
means a reference to an existing owned definition, not physical TypeRef/TypeSpec
resolution. Members, signature decoding, constraints, mutation/emission and dependency
resolution remain open. Next bounded task: complete assembly-reference identity and
explicit dependency resolution for nominal references before import/remapping. The
potential Raven implementation and Metadata Introspection projection remain plans.


## Primary compiler abstraction and C# contracts (2026-09-30)

**Author decision:** the Cecil-like API will be the primary abstraction for manipulating
metadata, especially compiler PE assemblies; adapt it as neoCLR needs evolve. The author
also explicitly requires tests in C#. This promotes the earlier provisional API
orientation into the selected direction. It does not turn the currently read-only subset
into a completed manipulation/writer API or require Cecil source compatibility.

The assembly/module/reference/definition model is now the intended compiler-facing
surface. Codecs and profile documents remain lower-level infrastructure. Plan import,
mutation/builders, validation and PE emission through this primary model rather than
requiring Raven callers to assemble raw sections. The possible Raven port beneath
Metadata Introspection remains separate implementation work, with shared semantics.

This slice adds exact AssemblyIdentity, physical AssemblyReference rows and an explicit
IAssemblyResolver. Identity compares ordinal simple name/culture, four-part version,
normalized public-key token and retained flags. Full keys normalize to the conventional
token; key/token representation flags are excluded. Missing or mismatched candidates
fail. No implicit directory search, runtime assembly loading or resolver cache occurs.

Primary sources reviewed 2026-09-30: [.NET 10 AssemblyReference metadata fields](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.metadata.assemblyreference?view=net-10.0)
and [AssemblyName public-key tokens](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.assemblyname.getpublickeytoken?view=net-10.0),
alongside the pinned Cecil sources above. This adopts conventional CLI identity data
but deliberately does not reproduce CLR binding policy. Exact matching avoids silent
version/key substitution; its cost is rejecting candidates a more permissive binder
might select. Ordinal name/culture and retained flag equality are provisional policy
choices, not permanent claims about optimal CLR compatibility. Retargeting, redirects,
unification and cryptographic verification remain unsupported. Token hashing uses the
CLI identity convention, not an authentication algorithm chosen for this platform.

The reader bounds AssemblyRefs to 256 and cumulative key/token input to 4 MiB, in
addition to existing declaration-name and image limits. It preserves owned identity
values but does not yet retain full keys/hash blobs as an assembly-writing model.
The next bounded slice is physical nominal TypeRef resolution through explicit
assembly dependencies, before reference import and the first supported writer path.

A dedicated C# executable contract-test project generates real PE metadata in memory
and exercises all APIs without a Python driver or new test packages. Eleven tests cover
identity fields, ECMA key/token normalization, explicit resolver behavior and failures,
input ownership, malformed metadata and the 256/257 boundary. The existing generated
model consumer still passes. Tests return a failing process status on any failed case;
framework discovery/CI wiring can evolve separately. No performance claims are made.


## End-to-end compiler producer baseline (2026-09-30)

**Author completion criterion:** keep implementing until there is a more-or-less
complete usable baseline, and do not call it complete without loading an assembly
produced by this API into neoCLR. The author identifies this as the proof needed for
Raven integration. A .NET re-read alone does not meet that criterion.

The next implementation adds physical nominal TypeRef resolution (local, assembly and
nested scopes) with explicit dependency matching, and controlled AssemblyBuilder /
TypeBuilder / MethodBuilder graph construction. The writer emits real ordinary CLI
PE32 images, including MethodDef bodies, entry tokens, imported AssemblyRef/TypeRef/
MemberRef rows and correct call signatures. It validates its bounded Int32 stack model
before emission and supports body replacement and repeated writes. These builders are
part of the primary metadata model, below future Raven symbol/codegen adapters.

This is intentionally a supported compiler subset: static classes, static Int32
parameter/result or no-result methods, linear constants/argument loads/arithmetic/calls/
return. It does not rewrite arbitrary loaded images. That avoids pretending to preserve
unsupported fields, attributes, generics, IL or debug data. Full reference import,
mutable declaration rewriting, richer signatures/IL and native structural emission
remain work. Existing reference-profile codecs still describe structural semantics;
main's runtime does not yet consume #Neo directly.

The acceptance route matches the current Raven target: C# producer -> API-emitted PE
application plus PE library -> existing CLI import bridge -> serialized native assembly
-> neoCLR load/verification/execution. It deliberately tests an imported cross-assembly
call rather than only an empty image. The program computes `20 * 2 + 2` and exits 42.
The C# integration runner persists all outputs and tool/artifact hashes. Direct PE or
#Neo runtime loading is not claimed by this proof.

The manual run verified 4126 IL functions against the composed Raven System library,
then reported `=> Int32(42)` with process exit code 42. An automated C# runner performs
the complete production/import/serialization/load sequence. Use the matching checked-in
core reference and the collection-library composition, not a stale published bundle
or the minimal embedded System library: the bridge emits helpers with matching union
contracts. Native build on this host required SDKROOT pointing to MacOSX26.5.sdk because
the default 27.0 SDK was incompatible with the installed linker. This is environment
configuration, not a runtime source change. The bridge build had 11 existing nullable
warnings; the metadata library and C# tests build cleanly.

## Top-level functions and direct native emission (2026-09-30)

The author clarified that neoCLR metadata supports functions outside types. The author
then selected this sequence: establish a working metadata format; build integrations
in the refactored compiler; subsequently improve the format, including structural types.
Structural signatures remain a future requirement and existing codec evidence remains
useful, but structural runtime work is not the next prerequisite for compiler integration.
A top-level function declaration is separate from a structural Function signature.

The model now has `AssemblyBuilder.Functions` / `AddFunction`, using the same body and
signature builder as type-owned methods. Every callable has an Assembly; DeclaringType
is nullable. This single-module builder's ownership can later move below ModuleBuilder
without making a synthetic user class part of native semantics. PE emission represents
local global functions using the physical `<Module>` row, while direct native emission
has no synthetic type and supports cross-assembly top-level function calls.

**Comparison:** .NET already exposes type-independent static global methods through
[ModuleBuilder.DefineGlobalMethod](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.modulebuilder.defineglobalmethod?view=net-10.0)
and completes them with CreateGlobalFunctions (Microsoft documentation, retrieved
2026-09-30). This is a metadata/library capability, distinct from a language's syntax.
We retain a familiar shared callable builder but make nullable ownership explicit.
The benefit is that compiler clients do not invent carrier types for native functions;
the cost is target-specific PE import rules and more ownership cases for consumers.
Cross-assembly global calls are currently rejected by the PE writer, not declared
impossible in CLI. Their eventual PE mapping needs separate interop validation.

`WriteNativeAssembly()` emits the runtime's **existing format-5 JSON assembly**, without
invoking the bridge or reading a generated PE. This is a second target for the same
bounded graph, not a new NEOX payload hidden in PE. Reusing the installed native schema
provides direct loader/verifier/VM evidence now; the tradeoff is an explicit provisional
backend tied to that schema, with no native MVID field and no structural NEOX sections.
Runtime primitive operations supply the Int32 semantics. References use deterministic
identity-derived module names and exact revisions; no implicit probing is introduced.
Assembly/type/method origin metadata is preserved for the existing descriptive catalog.
This does not implement the future guest Metadata Introspection reader/writer library.

The C# direct acceptance test writes both native assemblies through the public API,
then asks neoCLR to verify and execute them. It exercises top-level and type-owned
calls, a cross-assembly function, no-result returns, parameters and arithmetic, producing
42. Missing dependency and wrong-revision cases fail with matching diagnostics.
[Evidence](../experiments/extended-cli-metadata/native-validation.json) records output
and binary hashes. Fifteen C# contract groups pass, including physical global-method
row ownership and entry tokens. Neither test requires hand-authored application JSON.

The acceptance baseline is now met for this subset. The next readiness work is a
compiler-driven inventory of required ordinary declarations, signatures, imports and
bodies, followed by reader/writer coverage and refactored-compiler adapters. Do not
claim general Raven readiness from an Int32 test. Structural extensions come after
that integration, following the author's clarified sequence.


## Callable declaration reader baseline (2026-09-30)

The next bounded ordinary-metadata slice closes a producer/reader gap: PE methods and
globals emitted by the API are now available as owned MethodDefinition objects. The
entry point, module token lookup and declaring-type collections share those objects.
Physical `<Module>` globals become model functions with no type owner, following the
already selected native-function contract. The physical pseudo-type stays visible in
the flat TypeDef collection; it does not duplicate global functions in its Methods.

This reuses the CLI/Cecil comparison above: MethodDef flags, row tokens and signature
blobs remain physical metadata, while the public ownership model follows native intent.
A narrow `TryGetStaticInt32Signature` recognizes the existing writer subset. The
alternative of rejecting all other signatures would break nominal inspection of ordinary
assemblies; silently mapping them to Int32 would corrupt symbol loading. Retaining owned
opaque bytes keeps that information but deliberately postpones general type resolution.
This reader is not an IL or general signature verifier. Dedicated C# cases cover opaque
unsupported/malformed encodings, canonical compressed counts, snapshot isolation and
row/decoded-blob limits. Eighteen C# contract groups pass. General signature decoding,
MemberRef resolution and body import remain the next compiler-readiness gaps; structural
extensions remain after compiler integration as directed.
