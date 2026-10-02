# Extended CLI metadata for neoCLR

**Status: exploratory design, 2026-09-30.** Author-selected work on
`codex/extended-cli-metadata`, based on main `3ac2ae7c5c8434d743d1b866d87e6b21f48e4de0`.
This document starts format development. A standalone
[framing experiment](../experiments/extended-cli-metadata/README.md) now has a reader,
writer and inspector. A bounded [PE/CLI container probe](../experiments/extended-cli-metadata/README.md#pecli-container-probe-2026-09-30)
now transports #Neo and tests ordinary reader behavior. This opening section records the initial experiment; subsequent development now includes
a bounded Raven/runtime integration. See the [current assessment](../experiments/extended-cli-metadata/state-assessment-2026-10-01.md)
for implemented coverage. Production numeric encodings remain unassigned.

## Compatibility baseline reaffirmed — 2026-10-01

The author reaffirmed that the format should remain basically compatible with .NET
metadata except for intentional neoCLR extensions. Standard CLI tables, signatures
and IL are the baseline for ordinary constructs; new built-in coverage such as String
and argument assignment uses the existing CLI encodings, not new extensions.

The native PE/#Neo execution experiment described later in this document has not yet
met that full compatibility target: its conventional CLI declarations are a reference
projection with throwing bodies, while neoCLR executes the separate native payload.
The extension stream's transport compatibility is not executable compatibility or a
lossless extended-CLI assembly implementation. Preserve that distinction when claiming
progress. The bridge must be reconciled with the intended compatible representation;
it must not silently become a permanent parallel format for ordinary CLI constructs.
This clarification records the baseline, not a completed loader redesign or approval
of every exploratory proposal. Existing structural/native semantic extensions retain
their explicitly documented status and limits.

The author further clarified on 2026-10-01: neoCLR behavior is to align with .NET
unless an explicit alternative is chosen. The instruction set is the same for the
supported subset; unsupported facilities (for example exception handling in this
target) are coverage limits, not permission to redefine supported instructions.
Physical payload/loader differences must not be described as a new semantic baseline.
Existing intentional differences need their own documented decisions. New ordinary
compiler support should reuse standard CLI metadata and instruction semantics.

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


## Explicit callable reference resolution (2026-09-30)

The ordinary compiler baseline now reads owned MemberRef rows and resolves the nominal
static Int32/no-result method subset emitted by the writer. The source Module owns the
reference; resolution returns the dependency snapshot's MethodDefinition through the
existing exact-identity assembly resolver. Local TypeDef and TypeRef parents work too.
The matching contract includes the return result, while builder overload declarations
still distinguish name/parameter count. Multiple matches are rejected rather than
selected by row order. Runtime access checks are not replaced by metadata resolution.

This extends the existing CLI/Cecil reference comparison, retaining physical parent
and signature data while keeping host dependency policy explicit. Matching raw blobs
would incorrectly treat module-local nominal type tokens as global identity. The chosen
bounded decoder instead compares primitive contracts and rejects signatures it cannot
interpret. This supplies a usable import case without falsely claiming general generic,
instance, inherited or field binding. The cost is another explicit unsupported boundary;
general signature decoding must precede widening it. Opaque blobs remain readable.

All 21 C# contract groups pass, including emitted cross-assembly overload/no-result
calls, host-policy failures, absent/ambiguous/mismatched targets, unsupported parents,
physical row limits and a signature budget shared with MethodDefs. The existing PE
model consumer remains relevant for mixed conventional metadata. Structural extensions
remain later than ordinary compiler integration under the author's selected sequence.

## Refactored compiler as the next end-to-end consumer (2026-09-30)

The author selected activity `01a0f154-2448-7df3-8536-c837097b46c2`, **Refactor Raven
for platform targets**, for integration once this foundation is sufficient. The author
clarified the purpose: obtain an end-to-end case that drives testing and implementation
of the remaining metadata support. This refines the gate: do not wait for exhaustive
metadata coverage before attempting a bounded compiler adapter.

Read-only inspection of Raven's main-stability checkout at
`89ba7ecc9ddbe952cde21d6c2b425b499a2c47b3` found these actual boundaries:

- `Metadata/ISemanticDataLoader.LoadReference` returns symbols belonging to the current
  compilation. The metadata reader and indexes must remain private to the provider.
- `Metadata/IImportedAssemblySymbol` provides semantic type discovery; it must not leak
  this library's physical tokens or PE objects into shared binding.
- `Targets/ICompilationEmitter.Emit` owns backend output after shared semantic and
  target-contract validation. Streams remain caller-owned; generators are per call.
- `Targets/DotNetCompilationTarget` still composes the .NET emitter and reflection
  metadata services, including for the existing neoCLR CLI profile. It is not already
  a native metadata provider. A native adapter needs explicit composition and capability
  handling; changing a flag alone would not establish integration.

**Proposed first consumer:** compile a small Raven program with Int32 arguments,
arithmetic, static calls and an entry function; import an API-produced ordinary PE
library through the metadata provider, emit native format-5 output through the builder,
and load/verify/run the output in neoCLR. Include missing/wrong-identity references and
an unsupported-signature diagnostic. The test should exercise both symbol loading and
code generation, with the runtime result as its acceptance gate. A producer-only C# test
or copying a precompiled image would not establish those compiler boundaries.

This is the next integration experiment, not completed work. Keep modern .NET behavior
and its default provider unchanged, use an explicit experimental target path, and keep
unsupported constructs diagnostic. Do not require structural types for the first case.
Expand ordinary signatures/bodies when the consumer needs them. General runtime APIs,
full compiler coverage and the guest metadata-library port remain later. Compiler
changes will need paired Raven/neoCLR contract documentation, changelogs and focused
.NET regression checks. No Raven source was changed during this inspection.


## Independent metadata project, Raven target consumer (2026-09-30)

The author explicitly confirmed that the Cecil-style metadata API remains a separate
project and is consumed by Raven for the neoCLR target. Integration means adding a
consumer of the library, not moving its implementation into the compiler. The current
`tools/metadata/NeoCLR.Metadata.Experimental` project already has that independent
build boundary; its experimental naming and non-packable setting are unchanged. This
decision does not select a new repository, package name or distribution mechanism.

The dependency direction is Raven's neoCLR target adapters -> metadata library. The
library owns format reading/writing, the declaration/reference model, resolution
contracts and format validation. Raven owns conversion to compilation-local symbols,
lowering from compiler representations, target capability diagnostics and integration
with its semantic-loader/emitter services. Shared compiler code must not acquire a
dependency on the library's concrete metadata objects. The library must not depend on
Raven compiler symbols, syntax trees or compilation state.

Modern .NET retains its existing default loader/emitter. The first end-to-end case in
the selected refactoring activity should consume an explicitly pinned build of this
independent project through the neoCLR target. Keep the library's standalone C# format
and contract tests, and add compiler-side consumer tests for symbol import and emission
plus native runtime acceptance. Packaging/versioning and the eventual Raven port can
evolve separately; neither is implied complete by this boundary decision.

## First Raven compiler consumer outcome (2026-09-30)

The author explicitly expects several integration steps and authorizes Raven
refactoring when needed. A first compiler-owned consumer now exists in Raven at
`7e18edb66` on `codex/metadata-consumer`, based on shared main `89ba7ecc9`.
The separate metadata project is consumed via an explicit project reference. The
compiler library itself acquires no metadata-project dependency.

The probe uses Raven's public semantic operations to emit a small source application,
including a local top-level function and a call to an API-produced library. Native
output is produced directly through the metadata builder and executed by neoCLR with
result 42. The .NET input provider remains a documented bootstrap: a native symbol
loader and production target emitter are not implied by this result. This provides
a running source-to-runtime case now instead of waiting for all metadata capabilities.
[Evidence](../experiments/extended-cli-metadata/raven-compiler-validation.json).

The consumer exposed concrete shared compiler gaps: missing resolved binary operator
facts; Invocation.Instance projecting callee syntax instead of the receiver; and
MetadataLoadContext's synthetic null/default flag for a required signature parameter
without a Param row. Raven now exposes the bound operator facts, corrects receiver
projection, and checks actual row/default presence. General regressions and controls
pass (87 focused tests); these changes were fast-forwarded to local main independently
of the probe. They are not permanently experimental just because this consumer found
them. No remote push or release was performed.

Next: turn the adapter's explicit configuration and failure results into target-owned
compiler services, establish native metadata symbol loading, then widen ordinary
signatures and operations as source cases require. The current hand-constructed
fixture dependency is not an arbitrary PE-to-native importer. Structural improvements
remain after this ordinary compiler integration, as directed.

### Read-only call imports — 2026-09-30

The independent host library now provides `AssemblyBuilder.ImportReference` and
`MethodBuilder.Call(ImportedMethodReference)`. This follows the existing Cecil-style
import direction: a compiler consumes immutable definitions and imports a scoped
reference into its output. Compared with CLR reflection/Reflection.Emit, this needs
no loaded runtime assembly or executable method handle. Compared with full Cecil
imports it deliberately supports only unsigned, top-level-owner static Int32/void
signatures; no generic substitution, access checking or arbitrary IL translation.
The cost is an explicit host assertion of the dependency core contract and a bounded
format-5 naming agreement. Private reference-only nodes reuse both writers' existing
call encodings without retaining producer bodies. A future general signature model
should replace these bounded nodes as supported compiler cases require it.

Raven's feature-branch probe now receives only the read-only dependency snapshot;
it does not receive its builder graph. The primitive .NET Runtime Contract remains
the binding bootstrap, and ordinary .NET code generation is unchanged. Native
semantic-data loading and production emitter installation remain the next integration
stages; structural support stays later. C# contract tests pass (22 groups); both the
native global-function gate and Raven probe verify/run to 42. Missing dependency,
wrong revision, unsupported operation and binding diagnostics remain checked.

### Compiler-owned native adapter checkpoint — 2026-09-30

The working operations consumer is now the optional `Raven.CodeAnalysis.NeoClr`
project, with `NeoClrCompilationEmitter.Emit`, immutable output/core/dependency options
and a success/diagnostic result. The probe is a C# caller of this reusable adapter.
The metadata API remains a separate project with no Raven dependency. The new project
is opt-in through NeoClrMetadataProject; ordinary .NET behavior/default builds and
`Compilation.Emit` composition are unchanged.

Calls use explicit compiler-reference-to-snapshot bindings and resolved assembly-symbol
identity instead of a simple-name selection. Host snapshot consistency and the primitive
core assertion remain explicit responsibilities until a native provider owns them.
NEOMETA001 carries source locations; NEOMETA002 rejects incompatible configuration;
NEOMETA003 reports writer limits/invalid graphs. Original binding diagnostics survive.
All validation precedes output writes; host I/O failures propagate and can partially
write. Supported source/format-5 encoding remains the existing static Int32 subset.

C# consumer checks cover expression spans, unchanged rejected output, original error
identities, invalid/duplicate/unregistered/core bindings, writer limits, multi-tree
rejection, repeated output and stream ownership/failure. The emitted application still
verifies/runs in neoCLR with result 42. Hash evidence includes the new adapter binary.
Native symbol loading and production target registration remain pending. Structural
support remains later; this does not change the runtime bridge's platform capabilities.

### Multi-file native adapter checkpoint — 2026-09-30

The optional compiler-owned emitter now accepts multiple source trees. It collects
all supported top-level declarations before emitting bodies and retains each body's
own semantic model. Like the ordinary .NET compiler, valid cross-file calls bind
independently of file order; native metadata token/declaration order still follows
input order. No shared binding or .NET emitter change was required. Macro trees and
unsupported constructs remain excluded under the existing explicit bootstrap contract.

The new two-file regression first failed with the adapter's NEOMETA002 single-tree
restriction. After the refactor, Helper.rvn/Main.rvn and the reversed input order
both verify/run to 42 in neoCLR. A division expression in the later helper file
produces NEOMETA001 with that file's source location and leaves output unchanged.
The original one-file and adapter contract checks still pass. Validation evidence
includes hashes for all three applications. Native symbol loading and production
registration remain separate next steps; the metadata library stays independent.

### Native dependency input through a reference projection — 2026-09-30

The independent metadata project now reads its bounded native format-5 declaration
contract with `NativeAssemblyDefinition.ReadAssembly`. It checks manifest/name/origin
consistency, duplicate and unsupported declaration fields, owner/signature contracts
and resource bounds. Bodies remain opaque: successful metadata reading does not imply
successful native verification or execution. General native schemas, arbitrary types,
structural metadata and executable rewriting remain unsupported.

The snapshot can create a reference-only PE using an explicit core identity. The
projection preserves supported callable/type declarations and marks the assembly with
ReferenceAssemblyAttribute; placeholder bodies throw. It omits the native entry point
and implementation dependency references. These signatures need only primitive types.
This is the temporary native-input bridge, following the .NET separation of compilation
contracts and executable implementations, not a new executable CLI representation of
native code. The cost is an extra PE and the existing .NET symbol provider. Native
ISemanticDataLoader/symbol construction should ultimately consume native declarations
directly; the projection must then be retired, not made a permanent platform rule.
Primary comparison: [Microsoft reference assemblies](https://learn.microsoft.com/en-us/dotnet/standard/assembly/reference-assemblies).

The Raven probe now emits only the original native dependency from its producer,
reads that native artifact, and creates MetadataProbeLibrary.reference.dll from the
reader. It binds that reference with the existing .NET primitive Runtime Contract,
imports the read-only callable contract, and emits native applications. The original
native dependency (never the reference PE) is supplied to neoCLR. The one-file and
both two-file orders verify/run to 42; diagnostic/stream checks still pass. The report
records the reference projection hash alongside the native dependency/application hashes.

Ownership: the metadata project owns reading/projection; Raven owns compiler binding
and emission; the host supplies matching explicit core identities and native runtime
dependencies. Ordinary .NET defaults and existing CLI targets are unchanged. The
metadata project stays separate, and all work remains on the existing feature branches.
C# contracts pass (23 groups), including malformed inputs, projection ownership,
reference marking and rejection of execution loading by .NET. No production native
semantic loader or target registration is claimed.

### Raven library-to-application native case — 2026-09-30

Both sides of the integration case now originate in Raven source. The optional adapter
accepts library output without an entry point and public nongeneric static classes in
the global namespace containing public static Int32 methods. Ordinary Raven default
public method accessibility is accepted. Nonpublic members/types/library globals and
additional type contracts are rejected; the public-only metadata writer must not
silently widen a library's visibility. Console top-level functions remain native
functions outside types. Broader visibility/namespace/type support is still pending.

The producer declares MathLibrary.Twice overloads and a Multiply helper. Raven emits
the library as native format 5; the independent metadata API reads it and projects
reference-only declarations. A separate Raven application binds the one-argument
overload, emits a native external call, and neoCLR executes the original library's
local helper call. The one-file and both multi-file input orders return 42. No producer
builder graph or hand-authored native dependency body is used in the case.

The new case first failed with NEOMETA002 because the adapter accepted only console
output. Focused C# checks now confirm entry-less library output, overload/local-call
execution, source-located visibility/type rejection with unchanged output, and native
missing-dependency/wrong-revision errors. Existing diagnostic/stream and multi-file
checks pass. The .NET primitive Runtime Contract and reference-only input bridge remain
explicit; no default .NET behavior, general binder, metadata library API or runtime
format change was needed. The next replacement remains a native semantic provider and
production target composition, with further supported constructs driven by real cases.

### Transitive native runtime acceptance — 2026-09-30

The end-to-end chain now consists entirely of Raven-compiled native assemblies:
application -> MetadataProbeLibrary -> ArithmeticDependency. The outer library imports
the inner library's native declarations through the temporary reference projection.
The application references only the outer projection; Arithmetic is absent from its
symbol lookup and the outer reference PE's AssemblyRef rows, since all public
signatures use primitives. Like .NET reference assemblies, implementation dependencies
remain outside that compile-time signature surface. They are still required at runtime.

`NativeAssemblyDefinition.References` now exposes the exact direct native identities
in manifest order as an owned read-only list. It does not resolve dependencies or build
a transitive closure. The host explicitly supplies both native dependencies to neoCLR.
The existing runtime reference validator (`src/references.rs` in neoCLR), loader,
verifier and VM accept the emitted chain: all three application variants return 42,
and reversed runtime module order also returns 42. Missing direct/transitive modules
and wrong direct/transitive revisions fail verification with the expected diagnostics.
No runtime implementation change was necessary for this supported format-5 graph.

This is actual loading and execution of the emitted native format, not a claim based
on PE readability or reader roundtrips. Reference-only PEs are compiler input only.
The author reiterated runtime loading as a required acceptance gate. Direct PE/#Neo
loading and structural NEOX semantics are still unimplemented, and the .NET primitive
binding bootstrap remains temporary. Neither limitation is hidden by this test.

Validation: 24 C# metadata contract groups pass, including direct identity/list
ownership and rejection checks; Raven adapter/library/multi-file contracts pass; the
runtime checks above pass. Reports include both native dependencies and both reference
projection hashes. Compiler integration remains on codex/metadata-consumer and the
independent metadata/runtime checkout on codex/extended-cli-metadata.

## Direct runtime container checkpoint — 2026-09-30

The initial end-to-end gate now includes runtime implementation: Raven's opt-in
`EmitMetadataAssembly` produces a PE32 artifact through the independent Cecil-style
metadata project; neoCLR reads its required #Neo execution section, admits native
metadata, links explicitly supplied dependencies, verifies and runs it. The same
library PE files supply Raven's compiler references and neoCLR's runtime modules.
Top-level functions remain native assembly-owned declarations.

This is a **transitional execution profile**, not the final binary metadata layout.
NEOX 0.1 section **256, schema 1, required** contains UTF-8 native format-5 JSON,
including declarations and bodies. The existing recognition marker and SHA-256
binding cover every metadata stream. The native section is authoritative; CLI
metadata is a reference-only projection with `ReferenceAssemblyAttribute`, throwing
placeholder bodies and no CLI entry point. Runtime execution never consults CLI
method bodies. Unknown required schemas, missing markers, altered stream bindings,
malformed ranges and ordinary CLI PEs are rejected. Optional sections may be ignored.
The envelope is limited to 1 MiB and the PE to 4 MiB. The Rust host reads any native
format-5 module admitted by the existing runtime; the C# writer/reader intentionally
supports only its bounded public static Int32/void declaration subset. Parsing and
module-local validation are distinct from dependency admission and typed verification.

Compared with .NET reference assemblies ([Microsoft's reference-assembly contract](https://learn.microsoft.com/en-us/dotnet/standard/assembly/reference-assemblies),
reviewed 2026-09-30), the CLI surface serves the same compile-time purpose but the
neoCLR artifact additionally carries a separate implementation. This reuses the
already-tested reference projection and native runtime semantics. A full CLI table
and CIL loader would instead need token resolution, signature decoding and execution
translation; that is a larger compatibility commitment and does not supply native
structural semantics. Separate JSON and reference PE files were the previous bridge;
co-locating them removes that packaging mismatch from the tested producer path.
The cost is duplicate declaration storage, fresh projection MVIDs, bounded extra
copies, a hashing pass and continued JSON parsing. The digest detects modification,
not authenticity or equivalence between arbitrary CLI/native declarations. Hosts must
use the aware writer and consistent snapshots; arbitrary dual-view rewriting remains
unsupported. `AssemblyDefinition.ReadAssembly` still means the older reference
profile; `RuntimeAssemblyContainer.ReadCliProjection` is the explicit execution-profile
entry point and preserves the whole container in its snapshot.

The author highlighted parsing overhead as a reason to move metadata loading into
the runtime. This checkpoint makes **no startup or execution speed claim**: schema 1
still parses JSON after extracting the native section. Next evaluate a versioned
binary native schema with explicit table/heap indexing and compare load/parse, linking,
verification and execution timings separately against this baseline. Avoid treating
container packaging as a parsing optimization. Structural schemas need native runtime
contracts before becoming required sections; the earlier structural reference profile
is not accepted by this execution loader.

Validation: 25 C# metadata contract groups; 10 C#-driven runtime container checks
(including invalid native bodies and correctly re-bound unknown required schemas);
a Rust schema test and two Rust integration tests using a C#-produced fixture;
Raven's two-library chain plus single/multi-file applications returns 42, with reversed
module order, missing dependencies and revision mismatches checked. See
[runtime/compiler evidence](../experiments/extended-cli-metadata/raven-compiler-validation.json).
The feature branches remain `codex/extended-cli-metadata` and `codex/metadata-consumer`.
Production target registration, a native Raven symbol provider, guest Introspection
loading/emission APIs, broad signatures and structural execution remain open.

### Hello World, then an entry-point call

The author selects these as the initial acceptance sequence. Both are now compiled
from Raven and executed by neoCLR from PE/#Neo containers. Main returns Int32 zero;
in the second program it returns Greet(), which prints the line and returns zero.
The explicit compiler Console reference maps only the resolved static string-literal
WriteLine overload. Compared with .NET's Console overload surface, this is a deliberately
smaller bridge over neoCLR's existing UTF-8 String/System output contract, not an API
improvement or a general CLR import. The native writer emits ldstr/call/pop, preserving
the bundled System's current Void-valued return convention. The cost is a temporary
platform-specific builder convenience, `WriteConsoleLine`; a broader native callable
contract should eventually represent string parameters/results without special cases.
C# checks cover literal Unicode/bounds, native-only emission, explicit compiler
bindings, unchanged failed output and both exact stdout/exit-code runtime results.

## Binary native execution profile — 2026-09-30

Execution section 256 now admits **schema 2**, a bounded CBOR encoding of the current
format-5 object model. Schema 1 remains UTF-8 JSON and remains readable. The independent
host library adds `RuntimeAssemblyContainer.WriteBinary`; Raven's experimental
`EmitMetadataAssembly` now emits schema 2. Old schema-1-only runtimes reject it as an
unknown required schema rather than misreading it. No published target/format changes.
The CLI keeps PE bytes until native decoding: schema 2 goes directly through the binary
deserializer to `Module`, then existing validation/linking/verification. It does not
serialize CBOR to JSON or build a serde_json Value tree. Compiler-host emission and
reference projection still use the existing native JSON intermediate; this is a runtime
loading step, not completion of the future native object/reader/writer libraries.

The [CBOR standard, RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html) specifies
binary integer, text, array and map representations, with application choices for
key handling and accepted forms. This profile uses definite lengths, shortest integer/
length encodings, signed Int64 values, valid UTF-8 strings, arrays, text-keyed maps,
booleans and null. Duplicate keys, indefinite values, byte strings, tags, floats,
nonminimal arguments, trailing bytes and unsupported integers are rejected. The root
is semantic module format 5; existing declaration/instruction readers reject unsupported
shapes. Limits: 1 MiB envelope, depth 64 (root depth zero), 262,144 items including map
keys, and lengths/counts bounded by remaining bytes before allocation. Map order is
preserved by the writer but is not prescribed; this is not a canonical-CBOR claim.
The native format still has repeated names and descriptive identity strings; it is
not an indexed metadata table/heap format.

**Comparison and decision.** .NET CLI uses indexed metadata tables/heaps and method
bodies rather than a general binary object representation (see the ECMA-335 baseline
already linked above). Designing native tables now could remove repeated strings and
reduce allocation, but would couple this small compiler milestone to a broader stable
schema. Leaving JSON in the execution section preserves compatibility but retains
text parsing. Bounded CBOR reuses current fields and Serde semantics while removing
JSON parsing at the runtime boundary; its cost is repeated field names, allocated
runtime objects and an extra validation traversal. Rust uses pinned
[ciborium 0.2.2](https://docs.rs/ciborium/0.2.2/ciborium/) for direct Serde decoding,
preceded by the strict profile guard; the independent C# library implements only the
small specified subset and adds no package dependency. Primary sources reviewed
2026-09-30. This is provisional until load/size evidence and wider compiler cases
justify retaining it or replacing it with native indexed tables.

Hello World, Main calling Greet, both source-file orders and the two-library dependency
chain all load/run as binary PE/#Neo. The runtime continues to accept schema-1 PEs and
mixed inputs. A C#-produced binary fixture decodes to the same complete native model
as its JSON counterpart. Shared rejection vectors cover bounds, UTF-8, duplicate keys,
unsupported kinds and truncation. The C# host's Read reconstructs JSON for its existing
metadata reader; JSON whitespace and numeric lexical spelling are not preserved for
schema 2. `native_json` on the Rust host explicitly rejects binary containers;
`metadata_container::decode`/`load` are the format-independent APIs.

The [load comparison](../experiments/extended-cli-metadata/binary-loading.md) separates
in-memory binding/decoding from legacy load validation (which includes System linking),
warm-System preparation, typed verification and
prepared-program execution. No cold-start, general-runtime or execution-throughput
claim follows from a change in metadata decoding alone.

### Class-library JSON translation as a bootstrap

The author emphasizes compiling the runtime class library and importing its symbols
into Raven, and proposes translating existing JSON to neoCLR assemblies first. This
can reuse the current pipeline's metadata before direct native compiler emission
covers the whole class library. The first consumer should be a real bounded slice of
`runtime/raven/System.Runtime.rvnproj`, not another unrelated arithmetic fixture.

The existing C# WriteBinary input is restricted to the controlled writer's declaration
shape; it is not a general class-library translator. A translation tool needs to admit
real origins, complete declared signatures and member contracts, preserve bodies and
explicit dependencies, report unsupported shapes, and produce both native metadata
and an appropriately limited reference projection. CBOR schema 2 also currently
excludes floating-point values and larger images; those limits must be assessed on
real library artifacts rather than silently widening the profile.

Raven may initially bind the CLI projection because the present metadata models are
similar. That projection belongs behind the compiler's loader contract; native metadata
remains authoritative as semantics diverge. It must eventually be replaceable by
a native ISemanticDataLoader adapter without rewriting the runtime metadata library.
Class-library translation, direct compiler emission and the native symbol provider
are distinct milestones; success at one must not be presented as all three.


### Existing class-library translation checkpoint — 2026-09-30

The first translation test now covers the **entire current assembled System library**:
117 types, 743 functions (641 IL functions verified), 1,182,678 bytes of JSON to
466,240 bytes of binary NEOX. This assembles `runtime/System.neoil`, including existing
Raven-generated IL; it does not compile Raven source through the new metadata emitter.
The checked-in generated `.json` files are build manifests and cannot be translated
as executable modules.

The independent C# NativeModuleContainer transport preserves all native JSON values,
including fields, generics, origins, dependencies and bodies, without applying the
small Cecil-style writer's declaration restrictions. Runtime admission remains responsible
for semantic validity. The native envelope uses the existing execution schema 2 and
limits, no new semantic schema. The CLI admits it as root, dependency or explicit System.
The existing PE/#Neo APIs and reference projections retain their stricter admission.

Compared with the ECMA-335 PE/CLI and .NET reference-assembly baseline described above,
this standalone transport exercises the broader existing runtime model immediately,
without claiming a complete CLI projection. The cost is no .NET MetadataReference
compatibility, no PE stream-binding digest, and no new editable member model. Adding
a caller-supplied CLI view now could conceal declaration mismatches; expanding the
projection and compiler loader remains a separate next step. This is a provisional
bootstrap, not a format-performance claim or a replacement for the Cecil-style API.

C# checks compare complete JSON values before/after translation; native verification
covers System and both Hello examples. Both examples produce identical output with
JSON and binary System. A generic Box<T> library, a calling dependency and their
application all run as translated binaries with binary System and print 42. An unknown
body opcode is rejected by native loading. Rust additionally compares a C#-produced
fixture's entire decoded model, executes its consumer and rejects truncation, overlays,
optional execution and unsupported schemas. See the [reproduction instructions](../../tools/metadata/README.md#existing-native-json-bootstrap).

The [machine-readable result](../experiments/extended-cli-metadata/class-library-translation.json)
records complete-artifact hashes and consumer outcomes for reproducing this baseline.

The author clarifies the next milestone: compile the runtime class library from Raven
source, and use translated artifacts to check the compiler. Compare normalized
identities, declarations, signatures, dependencies and execution outcomes; raw byte
identity is inappropriate where token/MVID ordering differs. The baseline inherits
legacy compiler/importer assumptions and is not independent proof. Direct compilation,
Raven symbol import and a native semantic provider remain unimplemented by this slice.


### Library execution profile 3 — 2026-09-30

The sample experiment exposed two concrete schema-2 limits: the Raven collection
System (417 types / 4,090 functions) exceeds input/item bounds, and negative Double
literals use UInt64 bit operands above Int64.MaxValue. Required execution schema 3
addresses these in standalone NEOX assemblies. NativeModuleContainer.WriteLibraryBinary
emits it; Read and the native runtime admit it. Existing schema-1/2 admission and public
1 MiB envelope primitives retain their bounds. PE/#Neo remains schema 1/2.

Schema 3 uses the same definite-length CBOR map/array/text representation and strict
validation as schema 2. CBOR major 0 admits UInt64.MaxValue; major 1 still stops at
Int64.MinValue. No tags, byte strings, floating CBOR values, indefinite lengths,
duplicated keys, nonminimal integers or trailing bytes are admitted. Float instruction
bits are integers, preserving signed zero and NaN payloads without JSON floating
conversion. Reference names, tokens, signatures and instruction semantics do not change.

Budgets are 32 MiB input/reconstructed JSON, 8 MiB envelope, 2,097,152 items including
keys, depth 64. The observed library needs 15,017,185 JSON bytes, 5,542,303 binary bytes
and 846,454 items. This provides bounded headroom; it does not establish a production
maximum assembly size. Costs include greater possible allocation and validation work.
The runtime still deserializes the entire native model; no lazy/indexed loading or
speedup is claimed.

Compared with the ECMA-335 indexed-table baseline and the RFC 8949 analysis above,
this preserves existing native bit operands and reuses the codec with explicit limits.
Silently widening schema 2 would make its advertised compatibility misleading; wrapping
unsigned values as negative integers would change native operand meaning. Indexed tables
may reduce repetition but remain a larger design change. A new required schema makes
old readers fail explicitly, while keeping supported old images readable. The translator
CLI now writes schema 3; callers needing schema 2 can still use WriteBinary.

Validation covers .NET/Rust integer endpoints, exact payload/item limits and one-past
rejection, depth, malformed encodings, legacy rejection and C#-produced floating-bit
fixtures. Complete System JSON values roundtrip through the host API. A focused Raven
run verifies and executes FloatingMath, OptionPositional and ValueCopy with **binary
System and binary applications**. Their expected outputs match; six compiler-negative
cases remain rejected. The previous full matrix's successful JSON-System results remain
historical evidence, not a claim that all cases were rerun with schema 3.
[Consumer evidence](../experiments/extended-cli-metadata/raven-library-profile3.json).

The proper neoil binary producer, direct native Raven class-library compiler and broader
compiler symbol loading remain subsequent work. These are runtime assemblies without
a CLI projection, not inputs to the existing .NET semantic metadata provider.


### JSON/native assembly performance checkpoint — 2026-09-30

The author's requested [release benchmark](../experiments/extended-cli-metadata/json-vs-assembly-benchmark.md)
compares the real Raven collection System and FloatingMath artifacts, with complete
metadata equality and expected execution output. Native System is 63% smaller than
pretty JSON and current native decoding takes about 20% less time. The full CLI run does not
show a meaningful improvement; linking/admission remains dominant. Direct typed JSON
without the existing intermediate Value tree is substantially faster in the diagnostic.

This supports a concrete space benefit and a bounded current-decoder improvement,
not an inherent claim that binary encoding or this CBOR implementation is faster than
all JSON readers. Keep indexed metadata design and compiler interoperability decisions
separate from codec microbenchmarks. Profile linking and the binary validation/typed
deserialization passes before choosing optimizations; a JSON decoder change needs
malformed-input/version/diagnostic compatibility checks. No such change is included here.


## Instruction body editing direction — 2026-09-30

The author asks about raw opcode/operand emits alongside Call/LoadArgument helpers and
identifies Cecil ILProcessor-style insertion before/after an instruction as a possible
future direction. Current MethodBuilder has only bounded helpers, not raw Emit overloads
or an editable body. A proposed common instruction model would carry typed operands and
instruction-reference branch targets; helpers and raw Emit would construct the same
objects, and insertion/replacement/removal would act on the method body. Writers would
validate supported opcodes, stack contracts and reference/branch ownership before output.
This would also support Raven's future move away from Reflection.Emit for .NET. Relative
to today's builder it adds editing and backend reuse, with costs in ownership, exception
regions, branch repair, invalid intermediate states and serializer validation. API design,
branch/exception semantics and validation are open; no broad opcode or Cecil parity is
claimed. Defer body editing while advancing native assembly import/emission.


### First opcode surface — implemented 2026-09-30

A bounded OpCode enum and typed MethodBuilder.Emit overloads now construct the existing
linear instruction set, with helpers delegating to that path. Raven's native emitter
consumes it. This implements opcode-based construction from the preceding direction;
public instruction objects, body collections, branches and insertion/editing remain
planned. The internal operation model is not yet the proposed common editable body.

## Ownerless function namespaces — 2026-10-01

Real System.Math source exposes the need to retain namespace identity separately
from simple callable name without synthesizing native type ownership. ECMA-335 sixth
edition, II.10.8 and II.22.26, provides CLI globals on `<Module>` and a MethodDef name,
but no MethodDef namespace column ([primary specification](https://www.ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf),
consulted 2026-10-01). TypeDef namespace does not supply namespaces for ownerless
functions. This is an intentional native extension, not a change to CLI tables.

Decision: an optional native function `namespace` string, empty/absent for legacy
globals. The runtime retains and validates it; the producer incorporates it into the
executable name. Direct calls retain exact function references, not dynamic namespace
lookup. The metadata API keeps Namespace/Name/DeclaringType separate. Alternative
synthetic native types would change author-directed ownership; ambiguous dotted
name concatenation would lose boundaries. The temporary CLI projection uses a reserved,
reversible `<NeoFunction>` UTF-8 hex name for namespaced globals. Existing globals
retain their encoding. Reader/import adapters decode this projection; ordinary CLI
source loaders see physical names, not native namespace semantics.

Benefit: same-name functions in distinct namespaces and real class-library source
retain identity and ownerless access semantics. Costs: extra metadata text and name
encoding allocations, a reserved projection prefix, and matching reader/runtime
requirements. No performance improvement is claimed. Binary transport/schema remains
unchanged; the native declaration model gains an optional field. General native
metadata/semantic import and eventual extended-CLI namespace tables remain future
work; this bridge encoding is not that final table design.

Validation: 47 C# metadata groups, three focused Rust namespace tests, ordinary CLI
execution and direct binary native local/imported calls returning 42. Tests cover
same-name namespace isolation, invalid/duplicate declarations, import identity,
namespace/name consistency and legacy omission. Runtime owner validation is enforced
independently from the API. API snapshot check remains required.

## Root class and field producer foundation — 2026-10-01

The separate metadata API now creates nonstatic root classes with primitive mutable
instance fields. It writes ordinary CLI TypeDef/Field signatures and access flags,
and the matching existing native reference-type/field layout with field origin tokens.
The read-only snapshot exposes owned fields, signatures and type flags. This follows
[ECMA-335 sixth edition](https://www.ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf)
II.22.15 (Field), II.22.37 (TypeDef) and II.23.2.4 (FieldSig), reusing the existing
native field/access design rather than adding a new runtime storage model.

CLI classes inherit System.Object; the bounded native class is a root with no explicit
base. Static AddType behavior is unchanged. The API does not synthesize a constructor
or flatten properties into fields. Benefits are faithful primitive instance layout
and independently inspectable ownership/access; costs are additional field rows,
origin metadata and matching-reader requirements. No speedup is claimed. This remains
the PE/#Neo native-payload/reference-projection bridge, not CLI-body execution.
No Runtime Contract or runtime implementation changes; the tested binary is from
`e8611966`. Native reader/writer and raw field snapshots now have matching bounds.

Validation: 48 C# metadata groups; CLI reflection and owned field snapshots preserve
class flags, multiple-type field ranges and access; malformed origins and invalid
field declarations reject. API-produced binary class/field metadata loads/verifies
in neoCLR and its independent primitive entry returns 42. This is a metadata-load
gate, not yet an object allocation/mutation test. Instance calls and field bodies
are next, followed by property associations and Raven's real Order declaration.


## Owned nominal callable signatures — 2026-10-01

The root-object producer now carries exact owned class types in declared parameters
and results. This extends the existing ECMA-335/Cecil comparison above: ordinary CLI
method signatures encode CLASS plus TypeDef identity (Partition II, signature encodings),
while the native payload uses its existing Named type representation. No new extension
section or runtime instruction is required. Primitive signatures remain unchanged on disk.
The reader remaps nominal identity into each independent reference projection.

The selected alternative is a logical immutable signature model with backend-owned class
handles, rather than encoding every object as System.Object or naming it with a string.
This preserves overload distinction and exact stack validation at the cost of additional
signature mapping and an API migration from primitive-only signature properties. No
performance gain is claimed. Cross-assembly nominal imports, generic/subtyping/nullability
and nominal field/property declarations remain provisional follow-up contracts, not rules
of the platform. CLR-compatible TypeRef signatures are the later import boundary.

Validation uses C# producer execution and raw signature snapshots, wrong-class and foreign
builder rejection, native named-type reading and binary runtime execution. Raven's same
source tests factory returns, aliases, nominal constructor arguments and overload identity
in both source orders on .NET and neoCLR. The temporary CLI-reference/native-payload bridge
remains; this does not claim the runtime executes ordinary CLI method bodies.


### Owned nominal field storage — 2026-10-01

Mutable instance fields now use the same SignatureType as method signatures.
Compared with CLR fields, this preserves ordinary CLASS/TypeDef signatures and exact
class identity, including forward/self references. The native format uses its existing
Named field type; no format extension or JSON execution input is introduced. The
benefit is one output-owned identity model for compiler fields, parameters and results;
the cost is a development API change from primitive FieldType to SignatureType and
continued restriction to owned root classes. External imports, generics, nullable
contracts and readonly enforcement remain separate work. Default allocation remains
runtime behavior; this does not add a source-level non-null initialization guarantee.

C# tests exercise CLI execution, alias mutation through a stored object, native
reference projection, forward/self declarations and rejected foreign/wrong identities.
The API-produced PE binary verifies and executes with result 42 on neoCLR.


### Owned nominal property associations — 2026-10-01

PropertyBuilder now shares SignatureType with fields and methods. Compared with CLR,
property signatures and accessor associations retain ordinary Property/PropertyMap/
MethodSemantics rows and CLASS TypeDef identities. The native bridge uses existing Named
property types and setter parameter references; no binary schema extension is needed.
This removes a primitive-only producer restriction while preserving exact owned type
identity. The cost is a development API rebuild and continued limits on external,
nullable, indexed and generic contracts. A property adds associations, not storage or
runtime behavior: accessors remain independently declared methods with their own access.
Reference projection validates nominal getter/result and setter/parameter identities.
Static, read-only, write-only and private setters are covered by C# contract tests;
API-produced binaries verify/run with result 42 on the current neoCLR runtime.


### Readonly instance storage — 2026-10-01

The independent API adds optional AddField(isReadOnly: true) and FieldBuilder.IsReadOnly.
This preserves ordinary CLI InitOnly flags and extends exact writer validation to reject
stores outside the declaring constructor. The current native bridge carries the flag in
its existing field_readonly origin array, now consumed by execution as well as reflection.
A native semantic field flag should replace that provenance encoding when the extended-CLI
backend replaces the bridge; the source storage contract should survive that replacement.

Comparison with CLR: readonly is shallow storage protection, not object immutability.
Reuse the readonly research in docs/design-research.md and the existing managed-reference
contract. The runtime checks resolved declaring-type identity (including inherited field
ownership) for direct stores. Outside that constructor, managed field addresses retain
readonly capability, so reads work but stores through the address fault and fail verification.
Writable initialization references inside constructors remain possible. Raw unmanaged
pointers are outside these managed guarantees; this verifier is not a memory-safety sandbox.
The benefit is consistent metadata, writer and runtime treatment instead of reflection-only
flags; the cost is extra runtime ownership checks and a semantic compatibility requirement.
No performance claim is made. Missing/false flags retain the previous mutable behavior.

Consumers must rebuild for AddField's new optional parameter and use the updated runtime:
older binaries accept the metadata but do not enforce init-only stores. No format-number
or binary schema change is claimed. C# tests cover CLI flags, projection, owned/foreign
constructor writes and binaries whose direct or indirect illegal writes fail both verify
and run. Runtime Rust tests also cover readonly address reads. Raven consumes the flags
for private val storage and stored val properties; explicit readonly field syntax and
static readonly storage are separate compiler slices.


### Typed local object operations (2026-10-02)

Raven propagation exposes the need to address synthesized out locals. The first bounded
producer slice now supports exact typed reads and writes through owned local addresses,
using ordinary CLI `ldobj`/`stobj` (ECMA-335 sixth edition, Partition III, existing primary
baseline above) and the runtime's existing equivalent instructions. Builders append to
the existing definition body; writers encode those operations. There is no new encoding
or native opcode. Unlike the legacy native value-field store, native stobj already has
CLI's empty-result stack behavior, so the writer must not insert a pop.

The verifier tracks the addressed local identity: a store assigns that local, a load
requires assignment on every predecessor, and joins retain the existing exact-address
rule. This exposes the existing instruction contract instead of synthesizing defaults
to bypass definite assignment. It costs additional API/flow cases but avoids a new
memory mechanism. No performance improvement is claimed. General ref/out signatures,
callee out-assignment checks, imported value receivers and Raven admission remain later
work; this checkpoint does not change the unchanged collections diagnostic.

Validation: `LocalObjectChecks.cs` runs generic primitive/string/vector copies and
branch-merged local mutation; emitted CLI and native binary both return 42. Negative
checks cover uninitialized reads, value and address type mismatch, non-address operands,
Void and non-dominating writes. Run the C# contract executable, then its
`--local-object-integration <runtime> <fresh-output>` mode for actual native binary
loading, verification and execution. The runtime needs no rebuild for this slice.


### Writable ref parameters (2026-10-02)

Extend the preceding local-object slice with standard CLI BYREF parameters (ECMA-335
sixth edition, Partition II signatures; existing primary baseline above), mapped to
native ByRef without a new opcode or representation. This is a metadata producer gap,
not a .NET limitation. Definitions retain immutable typed signatures; builders append
ordinary argument, object and call operations. Readers, imports and reference projection
preserve the same shape. Generic substitution descends through the reference target.

Require initialized caller locals for this ref contract. Do not treat every byref call
as assigning its argument: CLI byref alone does not convey a C# out assignment guarantee.
Supporting out next needs explicit direction/assignment tracking, including callee
validation, and comparison with existing runtime verification. This costs an additional
contract rather than unsafe assignment inference. Readonly references, escaping returns,
byref locals, receiver addresses and argument rebinding remain rejected. No performance
claim or change to ordinary Raven/.NET emission is made.

`ByReferenceChecks.cs` verifies generic replacement, forwarded references, direct and
separate imported library/consumer execution (42), CLI snapshot and native projection
imports, and rejection of uninitialized/non-address/wrong-target calls and invalid type
positions. `--byref-integration <runtime> <fresh-output>` writes binary assemblies and
runs verify/run for both standalone and dependency-backed cases. The native runtime is
unchanged. The original collections application's out-local blocker remains open.

Validation recorded on 2026-10-02: 80/80 C# metadata checks passed; standalone and
separate-consumer native verify/run passed (42). Runtime binary SHA-256:
`3a254fac354a0878db897c17e46c66fdc19bddf1136d6659274033b25d063f0d`
on the `codex/extended-cli-metadata` integration worktree. API snapshot validation passed.


### Output parameter contracts (2026-10-02)

Reuse native Function.out_parameters and its existing loader and frame-return enforcement.
The metadata API now records explicit output indices in immutable method signatures,
retains them through generic substitution/import/projection, emits standard CLI Param
Out flags, and produces the equivalent native contract. This continues the ECMA-335
baseline above: BYREF is in the signature; Out is a parameter attribute, not a distinct
overload or a CLR verifier guarantee. C# definite assignment and the producer's proof
are separate from the CLI representation. An imported flag declares a contract; reading
metadata does not verify the external method body.

Track assignment for local slots and declared output parameters across branches. Store
through the exact parameter reference assigns it; read requires prior assignment;
normal return requires all outputs. Calls validate all ref inputs first, then publish
out assignment, preventing aliasing from masking an uninitialized input. Output
forwarding works without synthetic default initialization. Interface implementations
must match output contracts. This costs more flow state, but preserves the existing
runtime model and avoids either assuming every byref is out or initializing values only
to bypass the verifier. No performance benefit is claimed.

`OutParameterChecks.cs` covers generic assignment, forwarded outputs, Param Out
reflection and preservation via native projection/import, plus separate CLI/native
library/consumer execution (42). Rejections cover unassigned returns, early reads,
partial-path writes, non-address inputs, invalid indices, mismatched interface contracts
and aliased ref/out inputs. Conditional out_when_true and readonly contracts remain
future work. This slice does not yet change Raven's admission or the collections
application's observed blocker. Runtime binary is unchanged from the preceding checkpoint.


### Raven source ref/out integration (2026-10-02)

Raven `e6912a285` connects the preceding metadata contract to compiler-owned portable
signatures and instructions. `AllowsManagedReferences` is explicit adapter admission;
both .NET and neoCLR map local addresses and indirect operations, while native signatures
preserve output indices in the independent metadata API. Inline out declarations gain
uninitialized storage when addressed, and synthesized out locals no longer require
invented initializers. Binding retains language assignment rules; native producer flow
checks independently reject invalid bodies. Runtime Contract configuration is unchanged.

The temporary CLI snapshot remains symbol input; emitted binaries use native metadata
and actual runtime loading. Native metadata symbol loading, readonly references,
escaping byrefs, field/array addresses and imported value receivers remain pending.
This is an emission capability feature, not an independently discovered binder fix;
it remains on the integration branch. Existing isolated shared-lowering fixes are
unaffected. A separate shared-line capability port remains a review candidate.

[Five native controls](../experiments/extended-cli-metadata/raven-ref-out-validation-2026-10-02.json)
verify/run, with source out forwarding and ref mutation returning 42. The 64 focused
C# emission/parity/propagation/byref tests pass (24 baseline before edits); Debug and
Release assert shared admission and CLR behavior, not opcode sequences. The
[unchanged collections application](../experiments/extended-cli-metadata/collections-after-raven-ref-out-2026-10-02.json)
now reaches imported value-receiver TryGetOutput invocation admission. CLI control
still emits 7168 bytes; the whole native application remains incomplete. No runtime
instruction or metadata encoding change accompanies this Raven integration slice.


### Managed value receiver contracts (2026-10-02)

The next imported-member gap uses CLI's existing value instance `this` convention:
an initialized managed reference, direct call to a concrete method (ECMA-335 baseline
above). Preserve the owner value category through TypeSpec, native receiver_byref and
CLI projection. Reuse native slot references instead of boxing or copying the receiver;
mutation consequently affects the caller's storage. This extends existing producer and
reader APIs without introducing a new instruction. Value constructors and constrained
interface dispatch need different contracts and remain rejected.

The flow validator checks receiver initialization before committing out assignment,
including aliases between receiver and output. This costs receiver-aware validation but
avoids unsoundly treating an out alias as initialization of a call precondition. Fields
and indirect object operations share the typed reference contract. No performance
improvement is claimed. Imported final virtual value methods use direct concrete calls;
this is not permission to treat class virtual slots or interface dispatch as direct.

`ValueReceiverChecks.cs` verifies separate CLR/native library/consumer mutation and
generic value-owner TryGet(out T), with receiver and out contracts retained through
native projection. Negative checks cover value/uninitialized receivers, Callvirt,
value constructors and receiver/output aliasing. The native runtime is unchanged from
the prior checkpoints. Raven integration is the next step; this metadata slice alone
does not advance the unchanged collections application.


### Raven imported value receivers (2026-10-02)

Raven's metadata-consumer line (`bfd6578ea`) now opts into imported value-instance calls explicitly.
The shared plan takes an owned local address or forwards a ref/out receiver; concrete
public methods must be nongeneric and nonvirtual or final. Constructed value-owner
arguments are preserved. The metadata adapter matches both managed receiver and output
contracts before direct call emission, using the value-receiver metadata support in
`2276745b`. This follows CLI value-instance address semantics and preserves mutation.

Runtime Contract Propagation/Self/Unit configuration and binding are unchanged. The
ordinary .NET shared profile retains its existing fallback. The temporary CLI declaration
snapshot supplies symbols; this is not a native semantic importer. Source value
declarations, constructors, constrained dispatch and addresses of by-value parameters,
fields, arrays or temporary values remain unsupported in this bounded native path.
Shared emission capability changes remain a deferred shared-line port candidate; there
is no independent binder fix in this slice.

The separately authored native library and Raven consumer verify and run with result 42,
covering mutation, generic value setters and generic output calls. Missing dependency
registration rejects without modifying output. See the
[native evidence](../experiments/extended-cli-metadata/raven-value-receiver-validation-2026-10-02.json).
The [unchanged collections case](../experiments/extended-cli-metadata/collections-after-value-receivers-2026-10-02.json)
now rejects a lowered throw statement, with its 7168-byte CLI control unchanged.
`Lowerer.Propagate.CreatePropagateFailureBlock` synthesizes a throw when the carrier
provides neither output nor residual. The runtime does not currently have a producer
contract for that terminal failure. Do not remove the guard or silently return a default.
A subsequent slice must distinguish compiler-generated invalid-carrier failure from
general exception handling, preserve ordinary CLR behavior and validate both success
and deliberate invalid-carrier execution. Full application execution remains pending.

All 28 focused Raven external-signature/capability/shared-emission C# tests pass on .NET
11. An initial inline-output scope diagnostic in the ordinary .NET control is recorded
as an unclassified independent investigation in Raven's compiler documentation; it has
not been isolated against main or fixed in this slice. Native execution still tests
inline output locals.


### Compiler propagation failure guards (2026-10-02)

The metadata builder now has a literal terminal operation (`Fail`, also raw `Emit`).
Native emission reuses the existing fault instruction; runtime loading/verifying/execution
needs no new opcode or format version. CLI emission uses standard InvalidOperationException
construction and throw. Native faults end an invocation while CLR exceptions may be
caught: this target difference is explicit. It preserves a diagnostic without allocating
a guest exception or disguising failure as a normal return. General native exception
handling remains unsupported; the runtime `System.Fail` API and host `Fault` are unchanged.

Raven marks only its compiler-generated invalid propagation-carrier guards. Its shared
plan carries that marker through an explicit logical instruction supported selectively
by the adapters. The .NET adapter retains the previous ldnull/throw behavior (therefore
NullReferenceException); the native adapter requests a terminal diagnostic. Ordinary
source throws are still rejected by the bounded native plan. Binding, Runtime Contract
configuration and temporary CLI symbol projection are unchanged. This is an emission
capability feature, not a general binder fix or a native symbol importer.

Validation: 83/83 C# metadata groups pass, including success (42), Unicode terminal
diagnostics on CLR and native execution, malformed stack/unreachable body rejection and
terminal-only output parameters. The compiler regeneration/build succeeds; 14 focused
Raven propagation/external-signature/shared-emission tests pass, including a deliberately
invalid carrier that still raises NullReferenceException on .NET. The unchanged
collections source passes the guard and reaches imported carrier construction from None;
its CLI control remains 7168 bytes. No full native propagation consumer executes yet:
these tests establish the metadata runtime contract and Raven guard admission separately.


### Imported constructor checkpoint (2026-10-02)

Value constructors now use standard CLI instance metadata and the existing native
managed construction receiver. The producer tracks field assignment per branch and
rejects premature reads, incomplete normal returns and construction receiver escape.
Public top-level reference/value constructors import with exact signatures, and generic
owners substitute their arguments before Newobj emission. There is no constructor
chaining, byref constructor parameter support or implicit default-field workaround.

Raven opts into imported constructor admission through `AllowsExternalConstructors`.
The native reference resolver emits allocation for constructor handles consistently,
including defined source constructors. The ordinary .NET shared profile does not opt
into the new capability, retaining existing fallback emission. Binding and Runtime
Contract configuration are unchanged; CLI snapshots still supply symbols. This shared
emission feature remains an integration-line candidate for the shared compiler, not an
independent binder regression fix.

A separately authored native library and Raven consumer now construct ordinary and
generic values directly and execute mutation/output calls (42). Metadata C# consumers
also execute imported value/generic/reference constructors on CLR and neoCLR. All 84
metadata groups, 29 focused compiler tests and five native profile controls pass.
The unchanged collections sample still rejects Option<Order> construction from None:
its nested System.Option.None signature is outside the explicit top-level import model.
Next add native nested-type definitions/references and reader/writer identity support;
do not flatten declaring-type identity into a dotted name. Full collections execution
and native System library mapping remain pending.

Evidence: [constructor execution](../experiments/extended-cli-metadata/imported-constructors-2026-10-02.json)
and [unchanged collections assessment](../experiments/extended-cli-metadata/collections-after-constructors-2026-10-02.json),
with Raven `codex/metadata-consumer` revision `5bc0d567e` and this metadata slice.


### Nested declaration producer checkpoint (2026-10-02)

The .NET baseline is explicit NestedClass ownership and nested visibility, not a dotted
name convention. Following the existing ECMA-335/Cecil research above, the producer now
preserves that relationship in authored definitions, CLI rows and native declaring_type
identities. Parent-before-child construction makes identity validation deterministic;
existing top-level native names stay stable. The cost is an explicit current restriction
to nongeneric children/owners; captured generic parameters and external import are next.
Module.Types retains its established complete row inventory while NestedTypes exposes
immediate lexical children. This differs from Cecil's top-level collection convention and
is documented rather than silently changing existing iteration behavior.

A C# fixture emits a nested value constructor and executes to 42 on CLR and neoCLR through
binary PE/#Neo. It verifies loaded/reference-projection ownership, repeated short names
under different parents, two nesting levels, failed attachment and corrupt owner rejection.
No runtime opcode or format version changes were needed. Raven remains at 5bc0d567e;
the unchanged collections case remains blocked until nested imports and target mapping
are connected. The author explicitly directed continuing beyond intermediate slices until
the attempted samples compile and run successfully.


### Nested import/compiler checkpoint (2026-10-02)

ImportedTypeReference now preserves enclosing scopes in identity, signature substitution
and CLI TypeRef resolution. Native identities use explicit enclosing paths, while definitions
retain module-local ownership. Generic children under nongeneric owners are admitted;
captured outer parameters remain a deliberate bound. This follows CLI nested scope semantics
and reuses the existing native owner representation, without a format version change.

Raven selects the capability explicitly and matches physical union-case containers separately
from semantic carrier ownership. Runtime Contracts, binding and default .NET behavior are
unchanged. The temporary CLI symbol bridge retains these scopes until native symbol loading
replaces it. C# metadata tests (86 groups), focused compiler tests (30), CLR/native metadata
consumers (42), and a Raven dependency consumer (42) validate the slice. The unchanged
collections application now reaches Single callable signature admission; it has not run.
Extension/delegate signatures and the native System dependency mapping remain open.

Evidence: [Raven nested consumer](../experiments/extended-cli-metadata/nested-imports-2026-10-02.json)
and [unchanged collections assessment](../experiments/extended-cli-metadata/collections-after-nested-imports-2026-10-02.json),
Raven revision 942a0dce5 on codex/metadata-consumer.


### Function branch integration (2026-10-02)

At the author's direction, reuse structural Function runtime work from a081c6e3 rather
than add a new nominal delegate backend. Its existing design/research is documented in
[Function types](../function-types.md) and [the CLR comparison](../delegate-contract.md).
The benefit is reuse of checked signature binding, closure lifetime and migrated library
APIs; the cost is a feature-branch ABI change requiring matching generated references,
runtime and applications. Main and ordinary .NET delegate behavior are unchanged.

The runtime merge preserves binary PE loading and System.Fail. A synthetic Function.Invoke
contract now supplies the metadata branch's explicit empty namespace. Class-library snapshots
and API references are regenerated from the combined source. Existing Raven target settings
already select inhabited callback result transport; no new Runtime Contract option is needed.
CLI Func/Action remain temporary compiler carriers. The native metadata API will expose
structural signatures directly, preserving no-result and parameter/return contracts; shared
codegen must lower callback creation without Reflection.Emit. The unchanged collections
sample is still the direct native backend acceptance case, not the older CLI importer control.

The [combined-bundle validation record](../experiments/extended-cli-metadata/function-branch-integration-2026-10-02.json)
records 65 focused native tests, 86 metadata groups, five direct native controls and
Function CLI-import consumers. Full library source/bootstrap hashes and API snapshots match.


### Structural Function producer checkpoint (2026-10-02)

The .NET metadata API now carries ordered Function shapes separately from bound targets.
Exact signature equality, generic substitution and checked static binding reuse the native
runtime contracts integrated above. CLI transport is core-scoped Func/Action, with existing
ldftn/newobj/callvirt conventions; native output carries Function directly. This retains the
CLI baseline for readers while making the native shape explicit. The tradeoff is a bounded
bridge: byref/out callable shapes, receiver binding and inhabited Void callbacks are not yet
admitted. C# tests execute a callback through separately emitted library methods, a generic
higher-order call and a no-result callback on both CLR and neoCLR (42). Native PE reference
projection preserves the signature. Raven's direct shared-plan Function lowering is next.

## Direct native semantic import: implementation alignment (2026-10-02)

The author now prioritizes reading neoCLR metadata into Raven's semantic model over
expanding CLI translation, and explicitly reaffirms the existing reader/writer vision.
This changes the next integration task, not the architecture documented above or in the
roadmap's 2026-10-01 layering clarification: **builders → definitions → metadata → PE**,
with readers reversing those boundaries. Builders are optional facades over definitions;
Raven and eventual Introspection are separate consumers of that library.

### Audited gaps, not new architecture

- NativeAssemblyDefinition validates bounded writer-produced native declarations, but
  retains private rows and builder-backed signature references. Its public consumption
  path primarily materializes a CLI reference image through CreateReferenceAssembly.
  NativeLibraryDefinition separately inventories standalone/translated modules, with
  most signature information opaque. Neither is yet the complete definition reader
  required by the compiler. Avoid making either private row format a Raven contract.
- AssemblyDefinition/ModuleDefinition already serve the Cecil-like authoring and CLI
  snapshot paths. Loaded signatures still differ from AuthoredSignature, and references
  such as SignatureType can retain builder ownership. Finish that migration where
  native importing needs it: canonical definitions/references must be readable without
  authoring a placeholder body or serializing and rereading a CLI assembly.
- RuntimeAssemblyContainer already validates/extracts the authoritative native payload
  from PE/#Neo. Container reading, declaration decoding and dependency resolution remain
  separate responsibilities. The CLI projection must not supply missing native semantics.
- Raven's ISemanticDataLoader and IImportedAssemblySymbol are useful existing boundaries,
  but Compilation still owns DotNetCompilationTarget, whose setup unconditionally opens
  a .NET metadata session. PE symbols and ReflectionTypeLoader require reflection
  objects. This is not yet a selectable native importer despite the neutral interface.

### Next bounded implementation sequence

1. **Materialize native definitions.** Complete the read side of the existing definition
   model for API-produced PE/#Neo, initially exact assembly/module identity, references,
   namespace functions and primitive signatures. Preserve namespace ownership directly.
   Keep unsupported native declarations explicit rather than silently dropping or
   approximating them. Remove builder dependence from any signature/reference used by
   this read path; do not introduce a parallel compiler-only metadata object model.
   Native bodies can remain opaque initially, but must not become fabricated executable
   bodies. Distinguish a declaration-only read from a writable, lossless loaded graph.
2. **Import one native dependency into Raven.** Select a native semantic loader through
   the target boundary, using compilation-owned assembly/module/namespace/member symbols
   over those definitions. Test lookup, GetSymbolInfo/GetTypeInfo, accessibility and
   overload diagnostics for a small native function library. An explicitly identified
   CLI core may remain a bootstrap dependency for this first test; the imported native
   library itself must never be converted to CLI or loaded through reflection. This
   proves a bounded importer, not a complete native core.
3. **Bind and execute across assemblies.** Preserve definition identity into the native
   emitter so a call targets the original library. Compile a Raven consumer, load both
   assemblies in neoCLR and return 42. Reject a missing/mismatched dependency and
   unsupported required metadata before successful emission. Preserve .NET defaults.
4. **Expand the same model and importer.** Add nominal types, members, properties,
   interfaces, generic owners/substitution/constraints, arrays and structural Function
   signatures as required by tested library consumers. Import the real native System
   declarations and move core/iteration/propagation contract resolution onto native
   symbols. The standalone translated library's inventory limitations are reader work,
   not a reason to convert it back into a CLI semantic input.
5. **Return to source bootstrap.** Re-run source collections and the broad application
   with native dependencies, then compile more of System. The current mixed source/seed
   identity failure is not permission to equate distinct assemblies by type name.

Use immutable imported snapshots and compilation-local symbol caches, with explicit
reference resolution. Snapshot identity plus definition ownership must survive repeated
lookup and generic substitution; physical tokens remain module-local addresses. Unsupported
required semantics fail with diagnostics. Preserve unknown data only under a documented
round-trip policy; do not imply general loaded editing before it exists.

Acceptance for the first two slices includes C# reader/semantic tests, an assembly
produced by the API, exact scopes, repeated lookup identity, namespace functions,
missing references and malformed/unsupported input. Merely producing a reference PE
does not meet acceptance. Body decoding, broad editing and full System importing are
later increments, not prerequisites to the first honest native-reference test.

This follows the existing Cecil 0.11.6 and .NET MetadataReader comparison above: reuse
Cecil-like definition/reference navigation and keep low-level metadata below it. The
benefit is a single library contract for authoring, Raven and future Introspection; costs
are finishing ownership/signature migration and separating reflection-specific Raven
setup. No performance improvement is claimed. This section records the audit and next
work; it does not claim an implemented native semantic importer.

### Dependency resolution without a reflection API

The author explicitly rejects recreating .NET reflection while requiring dependency
resolution during symbol loading. Retain the existing IAssemblyResolver contract:
exact AssemblyIdentity to AssemblyDefinition, with AssemblyReference/TypeReference
rechecking scope. Native definition materialization must make this same resolver usable
for native assemblies. No System.Reflection.Assembly, Type, MemberInfo or execution-host
assembly loading is required by this contract.

The planned compiler import session supplies an explicit catalog/source policy and owns
resolution state. Establish an assembly identity and declaration index before recursively
resolving its referenced signatures, so ordinary cyclic assembly dependencies can reuse
one definition identity without publishing half-populated semantic results. Track loading,
ready and failed states; reject actual invalid declaration cycles separately. Keep caches
scoped to the supplied snapshot/configuration, including failed lookups, rather than
using global filename or simple-name caches.

Validate the requested full identity against the artifact. Missing dependencies, two
conflicting artifacts with one identity, unsupported required capabilities and malformed
metadata remain distinguishable failures that Raven maps to diagnostics. Version roll
forward, package search and filesystem probing are caller policy, not hidden resolver
behavior. The initial resolver can require exact registered inputs. A diamond dependency
must produce one shared definition and one symbol per compilation, while equal type
names in distinct assembly scopes remain distinct. Add missing/version-mismatch,
diamond, legal-cycle and conflicting-input C# tests as the native session is connected.

The existing resolver is implemented for current definitions; native materialization,
cycle-aware import sessions and these native semantic tests are planned follow-on work.

### Native definition materialization checkpoint (2026-10-02)

ReadNativeAssembly now materializes the first bounded native profile into existing
AssemblyDefinition/ModuleDefinition/MethodDefinition objects: primitive nongeneric
namespace functions, exact dependency references and entry identity. No CLI projection
is generated or read. Native method signatures are immutable logical signatures, with
no builder references, and loaded bodies remain opaque. Unsupported types/signatures
fail instead of disappearing. Original container bytes are retained for unchanged copy,
not editable re-emission. The native manifest has no MVID; consumers must use snapshot
ownership and assembly/definition identity rather than the empty MVID alone.

Both JSON and binary execution container profiles, input ownership, namespace overloads,
entry identity, read-only behavior and missing/mismatched dependency resolution pass;
96 C# metadata groups pass. This fulfills the first limited read-model slice, not the
whole materialization plan. The native validator still uses its existing private model
and builder-backed validation internally; broader typed reference migration remains
necessary before nominal signatures can enter this public read path. Next connect a
native dependency to Raven's semantic-loader boundary while preserving .NET loading
and emission. No reflection replacement or native semantic importer is claimed here.

### Native function semantic consumer checkpoint (2026-10-02)

The first Raven provider consumes ReadNativeAssembly definitions directly, retaining
namespace ownership and exact native assembly identity in compilation-owned symbols.
The existing definition reader/resolver design supplies this input; no new compiler-only
metadata model or reflection facade was added. The compiler's provider boundary now
composes native references with the existing CLI core loader for the bounded bootstrap.

C# semantic checks pass for overload/type binding, accessibility, stable declaration
identity, isolation, reference ordering and explicit dependency/version failures; 67
ordinary .NET target and symbol-equality tests pass. Native assembly versions cannot
collapse under simple-name symbol equality. Unsupported emission leaves output empty.
Next add direct native call references to the metadata library and Raven adapter, then
execute the imported call. Nominal/generic native importing and full core replacement
remain open. This advances the existing read → definitions → symbols direction.

### Direct native call execution checkpoint (2026-10-02)

The first native semantic provider now feeds actual cross-assembly emission. Raven
imports the exact MethodDefinition already owned by the bound native symbol through
AssemblyBuilder.ImportReference. The host supplies the registered native reference,
its exact Definition and matching explicit core in NeoClrMetadataDependency; mismatched
snapshots or translated implementation mappings are rejected without output.

Both an API-authored native overload library and a Raven-authored namespace-function
library are read directly, consumed by Raven and executed in neoCLR, returning
Int32(42). The primitive core remains the explicit CLI bootstrap. This follows the
shared definitions/reader/writer direction; neither a reflection facade nor a native
library-to-CLI symbol projection is involved. Compared with the CLI import path, native
snapshot consistency uses a cached SHA-256 of the owned image because native manifests
have no MVID. This conservatively rejects byte-different snapshots under one identity,
including equivalent containers encoded differently; it is not a persistent assembly ID.

C# metadata checks cover repeated identical-snapshot imports, namespace identity,
conflicting snapshots and incompatible core contracts: 96 metadata contract groups
and 22 Raven target-emission tests pass; the API snapshot check passes. See
[native call runtime evidence](../experiments/extended-cli-metadata/native-calls-2026-10-02.json).
Broader nominal/generic importing, native System symbols and source bootstrap remain
open. Existing CLI loading/emission continues through its current provider/backend.

### Direct native static-type consumer (2026-10-02)

The shared definition reader now materializes fieldless nongeneric top-level static
classes and their primitive methods, retaining native type origin tokens, namespace,
visibility and canonical method ownership. Raven supplies compilation-owned nominal
symbols, normal type lookup and primitive overload resolution. The emitter imports the
bound native method definition through the same route as namespace functions; there is
no synthetic CLI dependency or reflection representation.

The C# native-symbol runtime probe adds a Raven-produced static class library and a
consumer using both Boolean and Int32 overloads. The explicit CLI primitive core and
explicit matching native emission bindings remain required. Instance types, nominal
signatures, fields, properties, nested declarations and generics remain unsupported by
this direct-reader profile, even where existing writer/runtime paths support them.
This is the first nominal ownership slice, not full class-library metadata loading.

Validation: three native consumers return 42; all 96 metadata contract groups and
39 Raven accessibility tests pass. Evidence is recorded in
[native static-type runtime results](../experiments/extended-cli-metadata/native-static-types-2026-10-02.json).
A .NET-only regression reproduced an identifier-expression access-check omission;
Raven's general binder fix is isolated in e3afed13c for independent integration.
No merge to main is claimed.

### Direct native instance-class checkpoint (2026-10-02)

The direct reader now admits fieldless nongeneric top-level instance classes alongside
static types/functions. It preserves class flags, instance receivers and constructor
attributes in the existing definitions; Raven uses these for named-type/constructor
symbols and the existing imported allocation/call emit path. The Raven class consumer
constructs Calculator, stores an alias and invokes its primitive Add method, returning
42 in neoCLR. Private constructor and instance method calls require RAV0500 diagnostics.
Static signature helpers now reject loaded native instance methods; TryGetSignature
continues to report their primitive explicit parameter/result contract.

This matches the CLI ownership/constructor/receiver model for the admitted subset.
The explicit CLI primitive core remains. No new metadata encoding or runtime bridge is
introduced. Fields/properties, interfaces, value/nested/generic types and nominal
parameter/result signatures are pending in the direct reader. Added exploratory
reference-equality/inequality expressions hit the portable lowerer's BoundBinaryExpression
limit (NEOMETA001); that separate codegen gap remains open and is not a passing sample.

Validation: all 96 metadata contract groups and three runtime consumers pass; the API
snapshot check passes. [Instance-class evidence](../experiments/extended-cli-metadata/native-instance-types-2026-10-02.json).

### Direct native primitive storage checkpoint (2026-10-02)

The native read model now materializes primitive instance fields into the existing
FieldDefinition collection, preserving canonical owner/token identity, visibility and
readonly flags. The compiler adds native field symbols with lazy core primitive mapping.
No CLI blob or reflection field is fabricated: GetSignature rejects native fields and
TryGetPrimitiveType supplies the supported logical type. Authored FieldType remains a
separate existing contract until broader loaded signature materialization is implemented.

The Raven library's Calculator constructor writes its private Int32 storage; Add reads
it after the consumer constructs Calculator(20), stores a local alias and calls Add(22).
The native runtime returns 42. Public field lookup/type binding and private field
access diagnostics are also checked. Direct imported field emission still reports
NEOMETA001 without output because the external field operand adapter is not implemented.
This is distinct from the working cross-assembly method calls into stateful objects.

Compared with CLI field metadata, names, owner/origin tokens and FieldAttributes retain
the same model; only raw CLI signature access is unavailable for native input. Nominal
field/method signatures, properties and wider type categories remain pending. Native
format and runtime code are unchanged; the explicit CLI primitive core remains required.

Validation: all 96 metadata contract groups, three runtime consumers and the API snapshot
check pass. [Primitive-field evidence](../experiments/extended-cli-metadata/native-primitive-fields-2026-10-02.json).

### Direct native field access checkpoint (2026-10-02)

The imported field operand gap is closed for public primitive instance fields on public
nongeneric top-level reference classes. AssemblyBuilder imports an immutable typed
reference from the exact definition snapshot. The CLI writer emits a field MemberRef;
the native writer uses the validated native field ordinal. CLI snapshots cannot supply
native ordinals and reject native writing. No translated-layout guesses or new native
opcode encoding are introduced. Readonly stores, wrong receivers and foreign output
ownership reject before an assembly is returned.

Raven resolves the bound native field through its explicit dependency binding. Its
consumer writes Visible through an alias, reads through the original reference and
calls the stateful method (42). NativeFieldConsumer also performs a direct constructor/
field-load round trip (42). A private field preceding Visible tests ordinal preservation.
The explicit CLI primitive core remains; nominal signatures, generic/value owners,
translated field layouts and reference-comparison lowering remain pending.

Validation: 97 metadata contract groups (including actual CLR execution), four neoCLR
consumers and the API snapshot check pass. [Field-operand evidence](../experiments/extended-cli-metadata/native-field-operands-2026-10-02.json).

## Native local nominal signatures (2026-10-02)

The direct native reader now admits local nongeneric root class parameters/results in
functions, methods and constructors. Following the existing ECMA-335/Cecil comparison,
nominal identity is a reference into a definition graph, not a copied name or runtime
reflection Type. Internal parsing copies validated identities to module tokens before
materialization; public SignatureType.ReferencedType resolves to the immutable snapshot's
canonical TypeDefinition. No parser TypeBuilder escapes into a loaded signature.
ImportReference translates these references into interned output-owned ImportedTypeReference
operands. Primitive-only helpers return false for nominal signatures. Unsupported
categories still reject the entire read; the original native image remains opaque/copyable.
This changes the reader profile, not the PE/#Neo schema or instruction set.

Raven resolves signature types lazily after publishing the module, using a definition-keyed
symbol map and cached return/parameter types. The source-library consumer exercises a
factory, namespace/static/instance class identity functions, a constructor accepting a
class, alias writes and field reads; neoCLR returns 42. The 98 C# metadata groups include
both native container variants, same short names in different namespaces, canonical
resolution, output import interning, primitive-helper rejection and actual CLR execution
of equivalent imported signatures. See [hashed evidence](../experiments/extended-cli-metadata/native-nominal-signatures-2026-10-02.json).

The tradeoff versus full CLI loading remains deliberately bounded coverage: nominal
fields, external signature types, inheritance, generics, interfaces and value types are
not admitted by this native reader profile. The explicit CLI primitive core/bootstrap
contract is unchanged. No performance improvement or full System import is claimed.

## Native local nominal fields (2026-10-02)

The direct reader now uses the same snapshot-owned nominal signatures for fields and
methods. Type definitions are published before fields are materialized, admitting
forward/cyclic local references without retaining parser builders. Compared with the
.NET/Cecil nominal field model reviewed above, this preserves declared nominal type
identity and typed field access; it remains a bounded subset rather than a new storage
or object identity model. The benefit is direct native field inspection/import using
existing CLI MemberRef/native ordinal writers, at the cost of explicit current shape
limits: no external signature types, generic/value/interface/array fields or general
CLI nominal field decoding. No format or runtime instruction change is needed.

Raven lazily resolves a field's signature to its canonical module type, caching the
result after module publication. A native library consumer replaces a class-valued
field, mutates the replacement and checks the original object remains unchanged (42).
All four runtime consumers and 99 C# metadata contract groups pass; C# also executes
the equivalent native-snapshot import on .NET. See [hashed evidence](../experiments/extended-cli-metadata/native-nominal-fields-2026-10-02.json).
The CLI primitive core and translated System remain explicit bootstrap dependencies;
full native System import is not claimed.

## Explicit native signature dependencies (2026-10-02)

Native nominal field/method signatures now preserve exact external assembly identity
in immutable TypeReferences. This follows the assembly-scoped nominal identity model
from the existing ECMA-335/Cecil research rather than recreating System.Reflection.Type.
The reader remains independent of dependency loading; the existing IAssemblyResolver
is supplied at resolution/import boundaries. Native TypeRef identifiers are reader-local
logical row identifiers and do not pretend to be native origin/physical CLI rows.
Original-image Write behavior and the native wire format remain unchanged.

The benefit is direct symbol loading across real native library boundaries. The cost
is an explicit complete dependency set and the current nongeneric root-class profile;
there is no implicit file probing, version unification, type forwarding, or expanded CLI
signature decoding. Resolver imports check exact identities and snapshot fingerprints.
Raven validates dependency identities/type existence before publishing symbols, then
lazily resolves parameter/result/field types through compilation-owned modules. Emission
uses the already validated explicit metadata bindings.

The tested Raven pipeline builds PayloadLibrary, builds HolderLibrary against its direct
native reference, then compiles a consumer of both. Constructor arguments, a static
identity method and a class-valued field retain the PayloadLibrary type identity; the
consumer replaces the payload and runs in neoCLR (42). All five runtime consumers and
100 C# metadata groups pass, including .NET execution of equivalent scoped imports.
See [hashed evidence](../experiments/extended-cli-metadata/native-external-signatures-2026-10-02.json).
The explicit CLI primitive core/translated System bootstrap remains; full native System
import and generic/value/interface/array declarations are not claimed.

## Direct native vector signatures (2026-10-02)

The bounded native definition profile now retains one-dimensional vectors in field and
method/constructor signatures. The internal immutable signature row carries its element
shape; materialization reuses nominal definition/reference identities. A shared importer
maps primitives, nominal classes and vectors into output-owned signature operands.
External elements use the existing exact resolver contract, without reflection loading.

Compared with the CLI vector signatures in the existing research, this reuses ordinary
zero-based vector shape and alias behavior. It is a reader/import coverage extension,
not a new array format or storage model. Benefits are direct native array inspection and
cross-library compilation; costs remain the current exact-element, one-dimensional
profile. No new covariance, jagged/multidimensional or generic/value/interface element
support is claimed. Existing array opcodes and their validation are unchanged.

Raven now shares module-scoped signature mapping between fields and methods, caching
array symbols with their resolved element symbols after module publication. The native
payload/holder/consumer pipeline passes external-class arrays, stores one in a field,
replaces an element through an alias and passes an Int32 array across the same boundary
(42). All five runtime consumers and 101 metadata C# groups pass. C# additionally runs
native-array imports on .NET. See [hashed evidence](../experiments/extended-cli-metadata/native-array-signatures-2026-10-02.json).
The CLI primitive core and translated System remain bootstrap dependencies; full native
System importing remains pending.

## Direct native non-indexed properties (2026-10-02)

The reader materializes properties as associations over canonical getter/setter method
definitions, with immutable logical value signatures and exact nominal dependency
identity. This follows the CLI/Cecil property-versus-accessor separation documented
in the existing research: calls import methods, while inspection exposes properties.
There is no new property encoding or runtime instruction. The benefit is native
semantic property import without reflection or a CLI projection; the remaining cost
is a bounded non-indexed primitive/nominal/vector profile. Indexers and broader
owner categories remain explicit reader failures.

Raven associates each property with the already-created accessor symbols, preserving
MethodKind, visibility and canonical type identity. Property emission uses NeoCLR
signature capabilities, and shared static property lowering admits external class
owners only through the explicit external-reference capability. A separate general
binder fix checks private setter accessibility, independently reproduced against
.NET metadata. Default target capabilities and Runtime Contract configuration remain
unchanged. The CLI primitive core and translated System are still bootstrap inputs.

All five runtime consumers return 42, and 102 C# metadata contract groups pass. See
[hashed property evidence](../experiments/extended-cli-metadata/native-properties-2026-10-02.json).

## Direct native indexed properties (2026-10-02)

The existing property record already stores index parameters. The definition reader
now retains them as immutable ordered signature shapes and exposes them through an
explicit full-signature overload. This follows the CLI/Cecil separation of property
signatures from accessor call operands in the existing research. It adds inspection
and compiler coverage without changing the wire format, instruction set or runtime.
The non-indexed helper remains narrow to avoid silently dropping arguments.

Raven maps indexers to canonical accessor symbols and cached parameter lists, sharing
existing overload binding and call emission. Source indexer type admission uses the
NeoCLR target capabilities. Getter-only/set-only metadata remains representable;
source access to setter-only indexers still needs binder work. Generic/value/interface
owners and full native System loading remain outside this profile. The explicit CLI
primitive core and translated System remain bootstrap inputs.

All 102 C# metadata groups and five runtime consumers pass.
[Indexer evidence](../experiments/extended-cli-metadata/native-indexers-2026-10-02.json)
records tested bundles and the independently isolated general Raven corrections.
