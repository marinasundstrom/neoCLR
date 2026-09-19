# Development conversation record

This is a record of conversations in which the project author directs neoCLR's
development with an AI coding assistant. Through this concrete project, readers should
gain insight into how the author works with AI and approaches software development as
a whole. The exchanges supply that insight through actual questions and actions, rather
than an invented account of motivations. It serves a purpose similar to meeting
minutes: preserving questions, proposals, decisions, corrections, actions and outcomes,
with attribution to the person or assistant that raised them. It is not a personality
profile, a polished manifesto or a verbatim transcript.

**Initial record compiled: 2026-09-09.** The project began in the author-provided
shared chat [Define neoCLR proof of concept](https://chatgpt.com/s/cx_6aa11941d6d881918b897324f3a94730),
read on that date. This single record follows selected exchanges from that founding
conversation through Preview 1 and subsequent development.

Individual message dates are unavailable: sequence is retained without invented dates.
The founding account combines both participants' visible shared-chat messages with
additional exchange and validation detail retained in this task. Historical test,
commit and release reports are not fresh validation. Technical comparisons record
what was said then, rather than asserting what a .NET release currently implements.
The post-Preview-1 retrospective explicitly marks unavailable assistant replies;
missing responses and outcomes are not inferred from later code. Blockquotes reproduce
available original wording; other text is a summary.

See the [changelog](../CHANGELOG.md) and linked design documents for technical status.

## 2026-09-19 — Implement the first feature guide

**Author direction.** Continue after recording the feature-page plan.

**Assistant action.** Built the first Introspection guide with one runnable Raven
walkthrough, generated excerpts, shared expected output, downloads, design tradeoffs,
preview limits and feedback questions. Updated the homepage's superseded Type/Info
and class-based descriptions, and added nested-page/anchor validation to the site
builder. The guide is a local development artifact; no push or publication occurred.

**Follow-up question.** The author asked whether TypeInfo already extends MemberInfo.
The assistant clarified that it remains separate: the sealed MemberInfo cases are
FieldInfo, MethodInfo and PropertyInfo. Nested-type membership is a future option
requiring a declaring-type contract that also handles top-level types. The website
work does not change this API boundary.

## 2026-09-19 — In-depth feature pages for the next release

**Author proposal.** Create dedicated feature pages for the next release, going into
particular APIs in depth and showing more substantial samples.

**Assistant response and action.** Proposed a page structure covering the problem,
runnable walkthrough, API behavior/tradeoffs, preview limits and feedback questions.
Suggested Introspection first, drawing code from tested samples, with strings after
its later API/runtime story. Recorded a [content and delivery plan](design/feature-pages.md)
and linked it from the roadmap/design index. No feature page or website publication
is claimed; this documentation planning slice is separate from the closed API work.

## 2026-09-19 — Close the Introspection story

**Author decision.** With discovery, metadata tokens and Sequence collection contracts
in place, close this story for now. The String API and runtime string behavior are
a later story.

**Assistant response.** Accepted that boundary. Finish validation, keep discovery
and Sequence migration as separate commits, and refresh the runnable VS Code snapshot.
Do not begin string implementation or dynamic loading as part of closing this story.

## 2026-09-19 — Finish the Introspection collection migration

**Author direction.** “Migrate the remaining Introspection contracts from returning
array to sequence.”

**Assistant action.** Committed the preceding discovery/token slice separately as
92bd616, then changed the remaining TypeInfo/member collection contracts to
Sequence<T>, including parameter and enum-name results. Updated reference metadata,
Raven implementations, samples and checks together. Preserved native array storage
internally, independent snapshots, indexing, enumeration and query extensions.
The migration replaces Length with Count and requires consumer rebuilds; it does
not introduce covariance or promise immutable concrete storage. Dynamic loading
remains future context work, and strings follow the minimal Introspection boundary.

## 2026-09-19 — Dynamic loading remains context-owned future work

**Author direction.** “We might need to solve dynamic assembly loading later. It does
belong to RuntimeContext”.

**Assistant response.** Agreed that future loading/resolution belongs to RuntimeContext,
while AssemblyInfo remains descriptive. Kept this preview's queries over the already
loaded catalog; no loader, search policy, unloading or API signature was implemented
or selected in response. Unavailable references currently produce an explicit fault.

## 2026-09-19 — Finish minimal introspection before strings

**Author direction.** Implement ExecutingAssembly, ReferencedAssemblies and
MetadataToken before moving to strings; finish the minimal Introspection slices.
The author reiterated considering interface collection returns rather than arrays.

**Assistant response and action.** Kept these as the remaining preview boundary and
selected Sequence<T> for the new assembly/module collections because it supports
iteration, Count and indexing without mutation methods. Internal snapshot arrays
remain permitted. Implemented the supporting source assembly/module/token metadata
and validated import/execution, artifact round trips and scope checks. Public
assembly discovery and token interface properties remain in progress, so strings
have not started. The completed TypeInfo acquisition slice is f8104d1; a fresh,
verified VS Code workspace is available at ~/.neoclr/experiments/typeinfo-20260919/demo.

**Subsequent implementation.** Added the two sealed assembly/module interfaces and
Raven providers, ExecutingAssembly, direct ReferencedAssemblies and MetadataToken on
Info interfaces. The running sample reports Demo referencing System.Runtime and its
module's Widget type; editor checks expose all eight interfaces. New collection
returns use Sequence<T>. The assistant selected an explicitly bounded loaded-catalog
reference model: an unavailable reference faults, rather than loading or disappearing.
Discovery covers retained definitions; old array returns remain for later review.
Source-caller, token, generic-definition, resource and admission tests accompany the
change. The [implemented contract](introspection-design.md#minimal-discovery-and-token-apis-implemented--2026-09-19)
records these assistant implementation choices and their compatibility costs.

## 2026-09-19 — Assembly references and Info metadata tokens

**Author additions.** Requested AssemblyInfo.ReferencedAssemblies, if that is the
right owner, and specified that the executing assembly must show System.Runtime as
a reference. Then requested MetadataToken on the Info classes and clarified: “MetadataToken
should be on the interfaces.”

**Assistant response and action.** Agreed that references belong to assembly metadata,
with RuntimeContext owning runtime discovery. Committed to preserving direct references
and the bootstrap-to-System.Runtime identity mapping without implicit loading.
Explained that MetadataToken is module-scoped and must not be a renamed
DefinitionIndex. Recorded primary .NET comparisons and the bootstrap complication:
separately compiled library slices can reuse source token rows when merged. These
requirements are part of the next discovery/metadata slice, not implemented claims.
See [the contract and validation needs](introspection-design.md#assembly-references-and-metadata-tokens--2026-09-19).

## 2026-09-19 — Unified type acquisition implemented

**Author corrections.** Remove the remaining Type class, implement Object.GetType
(the selected spelling replaces GetTypeInfo), and expose ExecutingAssembly on
RuntimeContext. Keep introspection the common foundation for later reflection and emit.

**Assistant action and outcome.** Replaced the Raven public/runtime Type class with
TypeInfo and migrated structural signatures and saved samples. Added Object.GetType
and the production RuntimeContext.Current handle resolver. Validated allocation
identity through base/interface, boxed, string and array references; fixed native
snapshot materialization to preserve the public interface view. All 73 slices
regenerate reproducibly, 62 focused runtime tests and 33 admission cases pass.
The saved-project suite passed 43 cases before exposing the native-result defect;
after correction, all 31 remaining cases passed. Editor checks hide public Type,
its old Info hop and runtime providers, while identifying Info interfaces correctly.

**Remaining work.** ExecutingAssembly and assembly/module discovery are not part of
this completed acquisition slice. The compiler reference still needs an internal
empty System.Type shell for CLI closed-hierarchy attribute tokens; it is not a
public API or executable runtime type. The historical Neo profile remains separate.

## 2026-09-19 — A coherent API foundation for preview feedback

**Author clarification.** The unified introspection API should later expand into
reflection and emit. The author reaffirmed string APIs as the next step after this
foundation: the next preview should let evaluators understand the API direction
and give feedback about what could work better. In the preceding corrections,
the author requested removal of System.Type, selected `Object.GetType()` (replacing
the earlier GetTypeInfo spelling), and requested `RuntimeContext.ExecutingAssembly`.

**Assistant response and action.** Kept type acquisition and assembly discovery in
the current implementation scope, with invocation and emit deferred. Recorded
introspection followed by a minimal string surface as the preview sequence, with
runnable examples and explicit provisional contracts as the basis for evaluation.
Updated the current design spelling while preserving earlier decisions in this
record. The acquisition migration is in progress; this entry does not report its
completion, a finished string API, or a published preview.

## 2026-09-19 — String handling after basic introspection

**Author direction.** After the basic Introspection API, focus on neoCLR string
handling and implement a minimal UTF-8-related surface. The author left encoding
undecided and explicitly said not to add specialized Utf8String classes yet.

**Assistant response and action.** Preserved introspection as the current priority,
recorded string handling as the next step, and narrowed the implementation scope of
the larger string proposal without deleting its earlier ideas. Encoding and Char
representation are not selected by this message. No string implementation change
was made in response to this sequencing instruction. See [the roadmap](roadmap.md#immediate-sequence-clarified-2026-09-19).

## 2026-09-19 — TypeInfo as a member

**Author proposal.** “We could technically make TypeInfo part of the MemberInfo
hierarchy so it can represent a nested type.”

**Assistant response.** Agreed that this would allow nested types in member queries
and add TypeInfo as a fourth public match case. Identified the required ownership
change: top-level types have no declaring type, unlike nested types. Recommended
optional ownership in the unified TypeInfo model, compared .NET's corresponding
inheritance and nullable DeclaringType, and recorded a separate wrapper as an
alternative. The existing three-case implementation is not evidence that this
extension or the unified acquisition API is complete. See [the proposal](introspection-design.md#proposed-typeinfo-member-case--2026-09-19).

## 2026-09-19 — Exhaustive matching over the public member cases

**Author clarification.** “So if you have a MemberInfo and you do a match, then you
must cover all inheriting interface types.”

**Assistant action and evidence.** Removed the shared storage base from MemberInfo's
direct implementations so its permitted cases are exactly FieldInfo, MethodInfo
and PropertyInfo. Compiled a separate consumer with three public interface arms
and ran it against real descriptors. Removing each arm produces RAV2100. Added
reference type-test execution to neoCLR because the importer previously rejected
Raven's emitted isinst instruction. Recorded the remaining generic Raven diagnostic
issue: errors currently name hidden leaves rather than the public covering interfaces.
See [the contract and limitation](introspection-design.md#exhaustiveness-over-memberinfo).

## 2026-09-19 — Seal the introspection model

**Author decision.** “At the moment, we will model the introspection model as a
sealed hierarchy, which Raven supports also for interfaces.”

**Assistant response and action.** Changed the in-progress public Info declarations
to sealed interfaces and internal providers to closed families/leaves. Replaced the
proposed external-provider sample with a rejection case, and began preserving and
checking Raven's permitted-type metadata in the target reference. Recorded the
extensibility tradeoff and the distinction between Raven/importer enforcement and
raw neoIL runtime enforcement in the [design](introspection-design.md). This decision
does not settle the separate collection-return question or complete RuntimeContext.

## 2026-09-19 — Collection interfaces versus array results

**Author question.** During the Info-interface migration, the author asked whether
APIs should ever return arrays, or choose an interface conveying the collection.

**Assistant response.** Recommended capability-based result contracts, with existing
Sequence<T> (iteration, Count, indexed reads) as a candidate for materialized
introspection results. Distinguished read-only access from immutable storage and
snapshot semantics, and retained buffers/caller-owned storage as possible array
uses. Compared .NET's array/enumerable reflection surfaces and documented costs and
validation needs in the [design](introspection-design.md). This is a proposal; the
author has not yet selected a return type or a library-wide array policy. The
ongoing provider migration retains existing array results pending that API slice.

## 2026-09-19 — Close the source-port execution gate

**Author direction.** Continue implementation, retaining separately committed slices
and the requirement that programs targeting neoCLR run.

**Assistant action and outcome.** The complete source-port Rust baseline exposed a
lost declaring-type relationship on imported Option/Result cases. Preserved their
lexical nesting in target output (`d21c6de`), verified exact owner identity for all
four cases, regenerated 73 source slices and passed 12 union-admission and 34 focused
runtime tests. Raven already emitted correct nested CLI metadata; only its feature
branch documentation changed (`819e73ac6`). The final Rust batches now total 1246
passing tests across 178 integration binaries plus unit tests, with no unresolved
failure. All 65 saved-project cases also pass against the corrected library.
See [the validation record](raven-library-port-validation.md) for the
scope of reruns and the remaining API boundary.

**Still open.** RuntimeContext/AssemblyInfo discovery, the public Info-interface
migration, canonical Object.GetTypeInfo acquisition and the refreshed VS Code API
are separate work. Source-port execution success does not establish those APIs.

## 2026-09-19 — Optional services associated with RuntimeContext

**Author suggestion.** “Other things might later be tied to the runtime context.
Perhaps an optional GC, instead of a GC class.” This is a future possibility, not a
request to remove the existing collector or implement a GC-free execution mode now.

**Assistant response and action.** Recorded an optional context-associated collector
capability in the [maintained design](introspection-design.md), comparing it with
.NET's static System.GC API. Proposed separating capability availability from the
underlying memory-management policy. Heap scope, shared collectors, rooting and
lifetime rules remain open, as do API names and representation of absence. This
documentation update does not expand the initial assembly-discovery implementation.

## 2026-09-19 — RuntimeContext owns assembly discovery

**Author direction.** RuntimeContext will expose the executing assembly as
AssemblyInfo, whose modules and types can be queried. The author then clarified:
“In essence, RuntimeContext will hold a lot of the static methods that Assembly
hold in .NET. Making RuntimeContext the true context.”

**Assistant response.** Accepted context-owned discovery, with AssemblyInfo and
ModuleInfo describing their entities. The assistant proposed executing-assembly
traversal as the first preview slice and left assembly loading outside that initial
scope; the author has not selected exact member spellings or a complete list of
transferred .NET operations.

**Action and status.** Updated the [maintained design](introspection-design.md)
to replace the tentative all-assemblies entry point with the selected minimum and
remove the obsolete value.Type acquisition wording. Recorded the .NET comparison,
frame-selection validation need, benefits and compatibility costs. Production
interface/provider migration and the refreshed VS Code API remain open; the
System.Runtime project identity slice alone does not deliver this API.

## 2026-09-19 — Documentation first, then the remaining Raven library port

**Author direction.** “Let's port the rest of the System.Runtime library from neoIL
to Raven code.” The author reserves API alignment with the current proposals for
the next step, supplies the original proposals, and requests documentation
organization first, committed as its own slice.

**Assistant proposal and action.** Added categorized documentation indexes and an
original-proposal index linking the supplied texts to maintained design notes.
Preserved original proposal bodies, existing document paths and published notes.
The current project remains `System.rvnproj`; the System.Runtime assembly identity
is still planned. Source migration will preserve the existing API in this step.

**Author follow-up.** After porting, review Raven changes so bug fixes and general
improvements are on Raven main while the neoCLR target remains separate.

**Assistant response and current action.** Accepted that review as part of this
work. The collection-contract port exposed a generic-interface base-scope error;
three reduced cases reproduce it on an isolated branch from Raven main using
ordinary .NET references. Integrated the independently validated fix on local Raven
main (`2e3856a6b`, 87 focused checks) and cherry-picked it to the experimental branch.
The branch audit also reproduced a delegate bridge InvalidProgramException with
ordinary .NET metadata and integrated its fix on main (`5a37cd56c`, 19 focused checks).
The neoCLR branch remains separate. Deferred general candidates and policy boundaries
are recorded in the [branch audit](raven-library-port-validation.md#raven-branch-audit).

**Author acceptance criterion.** “The criteria for the port to be done is that
neoCLR is in a functioning state. Programs targeting neoCLR run.” The assistant
accepted compile/import/verify/execute checks against the rebuilt library as the
acceptance gate, rather than source conversion alone.

**Assistant-reported outcome.** The documentation organization is committed separately
as `f957b84`. The library grows from 28 to 47 Raven source slices, covering fundamental
and collection contracts, clocks/local time, native integers, Int32, Console,
Environment and file reads. Saved-program validation found and fixed lost parameter
names in imported library metadata; existing introspection output now passes.
All 64 saved-project cases pass against the rebuilt library, along with controlled
process/file checks and clean snapshot regeneration. This demonstrates functioning
programs, not completion of every source migration. String, unions/error carriers,
remaining descriptors and runtime adapters still contain handwritten neoIL.

**Follow-up direction and response.** The author asked for the next step. The
assistant proposed String/Error, followed by unions/error carriers, descriptors and
array/delegate adapters. The author said “Continue” and subsequently directed:
“After this work. Continue with the remaining slices and don't stop until finished.
Commit separately”. The assistant accepted separate validated commits and continued
without starting proposal API alignment.

**Descriptor follow-up.** The assistant preserved runtime snapshot field order and
names while moving the inherited MemberInfo family, parameter copying and accessor
selection into Raven. The target importer admits a bootstrap-only vector read view;
consumer metadata is unchanged. Nine admission cases, 23 reflection tests and the
freshly compiled reflection program pass. This is the 66th source slice; array,
delegate and remaining intrinsic helpers are still open. The separate ordinary .NET
constructor-arity correction is integrated on Raven main (`d7292b935`), rather than
merging the neoCLR branch.

**Trying the finished port.** The author asked to try the port in VS Code once it is
finished. The assistant committed to completing migration and execution checks first,
then preparing a workspace with the updated Raven compiler and neoCLR library.
VS Code preparation is pending; no successful editor run is claimed yet.

**BindingFlags correction.** The author said “BindingFlags should become a normal
Enum”. The assistant discarded an uncommitted struct-shaped authoring approach and
changed the source to a normal `[System.Flags] public enum`, retaining the six
existing Int32 values. Intrinsic enum operations are compiler/runtime lowering, not
source-authored struct methods. Import exposed a missing CLI backing-field flag;
an ordinary .NET regression reproduced it independently. Raven main now contains
`266b457f5` (13 focused checks), cherry-picked to the neoCLR branch as `6219343b9`.
The source admission checks pass; the slice's execution results are recorded in
[port validation](raven-library-port-validation.md).

**Branch cleanup direction.** After the port, the author asked to clean up side
branches created in Raven while retaining its long-lived neoCLR branch alongside
ordinary branches. Asked about its name, the author explicitly chose “Rename it to
neoclr”. The assistant will remove temporary integrated fix branches and rename the
integration branch after the final review; cleanup is pending at this point.

**Final helper decision.** The source-body audit found TypeOf<T>.Of as the last
handwritten managed helper after the array port. The assistant proposed migrating
it; the author instead directed removal because typeof is now supported. The
assistant removed its runtime/reference declaration and importer mapping, updated
active samples/tests to typeof or type tokens, and preserved published preview
notes as historical records. Existing compiled helper calls require rebuilding.
The ownership check now finds only generated declarations/bodies and explicit
runtime services in the selected Raven profile; final execution validation remains
open until the final gate below is recorded.

**Object.GetType follow-up.** The author suggested that Object may need GetType,
explicitly placing it in API alignment and the smallest feature set for the preview
release. The assistant distinguished declared-type typeof(T) from runtime-object
type discovery and recorded GetType as a candidate for that next stage. No Object
GetType implementation or final API decision is claimed by this port; removing the
old declared-type TypeOf<T>.Of helper remains the current action.

**Compatibility principle.** The author clarified that .NET compatibility should
provide the minimum familiar ergonomics and developer experience, while leaving
neoCLR free to choose its own design. The project is still in development and may
break compatibility. The assistant accepted .NET as the comparison baseline,
committed to documenting benefits/costs and migration consequences, and carried
that principle into preview API alignment, including the Object.GetType candidate.
This does not claim that the proposed preview API has already been implemented.

**Final compiler review and branch cleanup.** The assistant independently reproduced
editor host-reference injection with ordinary .NET references and integrated its
fix on Raven main (`6c2ccb13e`, 65 workspace checks). A known-value deconstruction
change was initially described as an optimization; independent testing instead
showed a real ref-struct InvalidProgramException. Main `f7f3f0c6d` fixes it, with 31
checks covering value-copy mutation, references and null/type checks. Both fixes
were applied to the neoCLR branch without merging target policies into main.
The integration branch is now `neoclr`; the eight temporary integrated fix branches
were deleted. Existing ordinary branches were preserved.

**Editor preparation outcome.** The assistant created an isolated local development
snapshot and VS Code profile, tested the exact saved-project command, and opened
Main.rvn. The initial ArrayList/Option/Result program executed with expected output.
Headless editor checks passed across the current target APIs; actual VS Code logs
confirm the selected server started and opened the demo. This is a local snapshot,
not a release. Instructions and reproducible tooling are in the
[local build guide](raven-port-local-build.md). The broad final runtime run remains
open until its result is recorded below.

**Next implementation direction.** After the local port workspace was prepared,
the author directed establishing the System.Runtime project (currently System),
then completing the basic System.Runtime.RuntimeContext with model implementations.
The author explicitly retires System.Type in favor of System.Introspection.TypeInfo
and selects Object.GetTypeInfo() as the canonical instance method. This replaces
the earlier tentative Object.GetType name; the earlier discussion remains recorded.
The assistant accepted that sequence, began checking the proposal and executable
context prototype, and kept the outstanding port validation running. No completed
production context migration is claimed at this point.

**Project boundary slice.** The assistant renamed the shared source project to
System.Runtime.rvnproj and changed its implementation assembly identity to
System.Runtime. All 73 slices compile/import with unchanged executable bodies;
snapshot and ownership checks pass. The private compiler reference remains an
explicit mapped contract in this slice. The author observed that the open VS Code
sample still showed class-based Info types and no RuntimeContext; the assistant
confirmed that snapshot was the completed source-port baseline and that production
API migration had not yet landed, then committed to updating it after migration.
The broad port run subsequently exposed missing declaring-type metadata for
flattened generated union cases; that correction remains open at this point.

**String/Error slice.** Raven sources now own both method surfaces, with explicit
importer checks for intrinsic storage, mixed String receivers and opaque Error
receivers. Native ownership and the existing parameter metadata remain unchanged.
Eleven admission checks pass, including rejected storage mutation and fabricated
opaque construction. All 64 saved-program cases also pass against the committed String/Error library
(`8744717`) and a fresh consumer core. The empty-declaration slice then exposed
source shadowing of an explicitly configured unit assembly. The assistant reduced
it independently with .NET ValueTuple, integrated the general fix on Raven main
(`c17cb8397`, 19 passing focused checks), and cherry-picked it to the neoCLR feature
branch (`b6f12353f`, 24 passing feature checks). Five empty errors and Void bring
the port to 55 source slices; nine admission cases, 14 Rust checks and four saved
programs pass. Unions/error carriers, descriptors and adapters remain pending.

**Open.** Complete and validate the remaining Raven source port, then align API
contracts in the subsequent step. Documentation organization does not itself
implement proposal APIs or rename assemblies. See the [documentation index](README.md)
and [original proposals](proposals/README.md).

**Typed error carrier slice.** The assistant reports seven more Raven source owners
(62 total), with checked erased storage and constructor projection. Fourteen admission
checks, 25 focused Rust tests and all 64 saved-program cases pass. Generic unions,
descriptors and adapters remain open; continuation and separate commits remain the
author's instruction.

**Generic-union work in progress.** The propagation declaration is committed separately
(`49e7307`), with five admission checks and 20 focused runtime checks. Authoring the
remaining carrier bodies exposed a general Raven out-forwarding diagnostic defect.
An independent ordinary .NET regression failed before the fix; the assistant reports
41 checks passing and integrates `5f6e17347` on Raven main, with feature cherry-pick
`2d2a1d586`. Carrier source migration remains in progress at this entry.

**Generic-union outcome.** Raven source now owns Option/Result carriers and cases
(65 slices total). The assistant reports 12 admission cases, 41 focused Rust tests
and all 64 saved-program cases passing. Earlier 14 typed-carrier and 17 foundation
admission checks still pass. Readonly adapters and literal Boolean return emission
were corrected during verification. The separate unqualified generic constructor
lookup investigation reproduces incorrect ordinary .NET execution (0 instead of 42);
a namespace-arity candidate fix now passes its initial focused checks, with broader
validation and integration still pending at this entry.

## 2026-09-17 — Plan the minimal System.Runtime assembly

**Author direction.** Plan a System.Runtime project/assembly supplying the minimal
types a managed application needs, substantially those being implemented in System.
The author keeps the next release focused on API shape and a running demonstration.

**Assistant proposal and action.** Recorded a staged assembly plan: inventory actual
dependencies, establish one foundation identity, migrate reference generation and
runtime contracts together, then execute the POC against it. Namespaces retain their
separate responsibilities; Reflection and Emit remain optional capabilities.
Updated library documentation and website with explicitly planned status.

**Open.** Exact minimal membership and reference/implementation identity mapping
require the inventory. No project or assembly has been renamed in this documentation
slice. See [the plan and .NET comparison](system-runtime-assembly.md).

## 2026-09-17 — RuntimeContext owns typeof acquisition

**Author direction.** Runtime*Info implementations must remain hidden; handle-based
type acquisition and RuntimeContext should return the same or value-equivalent
TypeInfo. The author selected `RuntimeContext.Current.GetTypeInfoFromHandle`,
contrasting it with C#/Raven's System.Type factory, and explicitly requested the
neoCLR Raven compiler change. The release goal is a running API-boundary POC,
not complete implementations.

**Assistant action.** Added an opt-in Raven compiler contract on the existing neoCLR
feature branch and a minimal runtime-backed context in the isolated Raven probe.
The sample uses actual typeof(Date), returning the descriptive interface and
querying fields without reflection invocation. The check executes the source entry
unchanged; separate test instrumentation compares underlying runtime identities.
Updated the proposal, integration documentation and website excerpt.

**Boundary and remaining work.** The default .NET compiler path remains System.Type.
The POC uses a lightweight context facade and hidden adapters over existing runtime
types, not a completed loader or replacement core. Production identity migration,
context lifetime and full cross-origin equivalence remain open.
See [the executable probe](experiments/raven-target/introspection-v1/README.md).

### Runtime-owned descriptor acquisition — 2026-09-17

**Author clarification:** Runtime*Info implementations should not be visible, but
the runtime must know which implementation to produce from a handle or type-of
instruction result. RuntimeTypeInfo is returned as TypeInfo. RuntimeContext should
obtain the same object or a value-equivalent description.

**Assistant response:** Recorded hidden public implementation types, runtime-owned
resolution and shared identity/equivalence across type-of and RuntimeContext as
requirements. Identical allocation is not mandated. The current prototype's direct
internal adapter construction is bootstrap-only; it does not implement the final
resolver, context discovery or equivalence contract. Exact lowering and lifetime
remain open, with validation requirements in the [proposal](introspection-design.md).

### Refined interface-based introspection proposal — 2026-09-17

**Author direction:** Supplied a simplified proposal and asked to update the docs
before continuing work. `*Info` types are interfaces without I-prefixes. The public
Type/TypeInfo, Assembly/AssemblyInfo and Module/ModuleInfo pairs disappear. V1 is
runtime-backed through RuntimeContext; full MetadataContext and typed introspection
are deferred. Reflection binds/executes against a context; Emit generates without
implicitly loading. Prefer collection queries over BindingFlags.

**Assistant action:** Recorded the [new proposal](introspection-design.md), preserved
earlier positions as superseded history, and updated the migration docs and website.
Set aside the uncommitted MemberInfo abstract-class source-port experiment because
it pursued the superseded public shape. Existing committed runtime APIs remain
working migration baselines. Proposed a minimal executable interface/provider probe
before the coordinated runtime identity migration; it is not the full v1 API.

**Author follow-up:** "Keep the usage of binding flags for now".
**Assistant response:** Retain BindingFlags and existing filtering semantics in the
current interface migration and samples. Collection-style querying is a future
option, not a requirement to replace flags in this slice.

**Implementation outcome:** Added an isolated Raven TypeInfo/MemberInfo interface
probe backed by current runtime types and field snapshots. Verified interface
emission, DeclaringType returning TypeInfo, execution and rejection of Invoke/Info
on the minimal interface. Existing three introspection consumers and eleven focused
runtime tests pass. No production descriptor identity or RuntimeContext implementation
is claimed. The website describes the direction and prototype status without the
BindingFlags detail, as requested by the author.

### Shared introspection model clarification — 2026-09-16

**Author correction:** Limit the API to necessary concepts. Introspection need not
be independent of the runtime. System.Runtime.Reflection and System.Runtime.Emit
are additional capabilities that may attach operations and supply constructs within
the same descriptive model, with clear boundaries.

**Assistant response:** Acknowledged having overextended the separation into runtime
independence. The next port retains existing runtime services and adds no offline
provider or alternative descriptor hierarchy. The shared model remains descriptive;
invocation and builders are additional capabilities.

**Implementation follow-up:** Ported the existing TypeInfo queries and internal
handle factory to Raven. Kept runtime metadata services and the instance Info
property; adapted query results to managed arrays. Library import now checks
internal factory contracts and bootstrap generation preserves their visibility.
Flag parameters use the descriptive name `flags`. No runtime Reflection/Emit APIs
were added. The member hierarchy remains the next source-port work.
The author also requested sample updates. Updated the executable Raven examples
to show the shared TypeInfo shape and named `flags` arguments, with readable blocks.

**Further author requirement:** Future Emit must avoid the problems of combining
types from runtime Reflection, MetadataLoadContext and Emit implementations.
**Assistant response:** Recorded cross-context identity, provenance and binding as
requirements of the shared model, with mixed-origin signatures and unfinished-builder
tests. Common API shapes alone do not establish interoperability. Exact binding APIs
remain future design; the current TypeInfo port does not implement Emit.

### ParameterInfo source migration — 2026-09-16

**Author direction:** Continue the API restructuring and Raven migration in
separately committed slices.

**Assistant action:** Selected ParameterInfo as a bounded first descriptor port.
Its six getters and private constructor are now Raven-authored, with an importer
check protecting runtime snapshot layout. Existing factories and public contracts
are preserved. Added positive/negative authoring checks and reused executable
introspection consumers. TypeInfo query adapters and member inheritance remain
subsequent work. Documentation and website status identify the completed subset.

**Validation:** 21 focused runtime tests, positive/negative ParameterInfo authoring
checks and three executable Raven consumers passed. Bootstrap hashes and the
website build also passed; this is local development validation.

### First namespace migration slice — 2026-09-16

**Author direction:** Start implementing the new structure, keep documentation and
the website current, and commit each feature slice separately.

**Assistant action:** Moved existing descriptors and BindingFlags into
System.Introspection in the runtime, reference metadata, Raven Type source and
consumers. Regenerated bootstrap artifacts and coverage inventories. Preserved
query behavior and documented the breaking namespace migration. Info remains an
instance property and descriptor bodies remain neoIL for this bounded slice;
their Raven port and extension boundary are still open. Website status reflects
this distinction. No Raven compiler changes were required.

**Validation:** 49 focused runtime tests passed, along with Type authoring checks,
signature checks, and three Raven consumers executing against the migrated
namespace. The old descriptor namespace is rejected. Bootstrap hash and coverage
checks and the website build validate generated records and presentation.

### Proposal-driven Raven library migration — 2026-09-16

**Author direction:** Commit the proposal records and prioritize moving existing
APIs into their proposed namespaces and structure, with the goal of porting the
runtime class library from neoIL to Raven.

**Assistant recommendation:** Start with the existing introspection descriptors,
following the completed Type port, then migrate the existing time family. Update
reference metadata, importer mappings, generated artifacts and consumers with each
slice. New globalization, stream and async implementations remain separate work.
Recorded the sequence in the [Raven library plan](raven-system-library.md).
This exchange records direction and a proposed sequence, not completed API moves.

**Subsequent author clarification:** The next release should have the API shapes in
place even without complete implementations, serving as a demo or POC.
**Assistant response and action:** Updated the library-preview objective and Raven
migration plan to prioritize namespaces, signatures and capability relationships
across all seven proposal families, alongside working examples for implemented
paths and explicit declaration-only status for unfinished areas. Full backends are
not required before exposing the proposed structure. No API implementation changed
as part of recording this clarification.

**Website follow-up:** The author asked to show these improvements on the website.
Updated What's next with the demo/POC objective, migration priorities and all seven
proposal families, explaining benefits and distinguishing current implementations
from proposed API shapes. Website publication is separate from this source update.

### File-system capability proposal — 2026-09-16

**Author proposal:** Treat the file system as an explicit capability rather than a
collection of globally ambient static gateways. Keep `Path` as a pure value, use a
`FileSystem` context with a sensible default and injectable implementations, and
obtain `File`, `Directory` and `FileSystemEntry` handles from it. Keep streams as the
byte-I/O abstraction and avoid making the filesystem interface responsible for text,
encoding or buffering.

**Assistant action and outcome:** Added the [file-system capability proposal](filesystem-design.md)
with the current static File API as baseline, alternatives including an injectable
facade and runtime intrinsics, a System.IO.Abstractions comparison, capability/error/
ownership boundaries and validation requirements. No implementation or existing API
changes were made; handle lifetime, path semantics, enumeration, sandboxing,
cancellation and async contracts remain provisional.

### Globalization, introspection and streams proposals — 2026-09-16

**Author proposals:** Define globalization as immutable culture data resolved from
`CultureId` and supplied through a contextual provider; separate core type identity,
descriptive introspection and optional runtime Reflection/Emit capabilities; and model
streams as composable synchronous/asynchronous byte capabilities rather than one large
`.NET Stream` class. The Stream proposal keeps partial I/O, explicit buffering,
text-over-bytes readers, and FileSystem as the source of stream capabilities.

**Assistant action and outcome:** Added [Globalization API](globalization-design.md),
[Stream API](stream-design.md), and the new Introspection/Reflection section in the
[reflection review](reflection-model-review.md). Compared the proposals with current
.NET conventions and relevant independent abstractions, and recorded tradeoffs,
validation requirements and open contracts. No runtime or public API implementation
changed; the existing static file, bounded text, reflection and console surfaces remain
the implementation baseline.

### Ongoing Type/Reflection separation — 2026-09-15

**Subsequent corrections and scope.** The author placed TypeInfo in System.Reflection,
kept MemberInfo as the member hierarchy's base, and noted that TypeInfo can also be
a member for nested types. Closed hierarchies remain a desired direction, not an
implemented property of this port. The author then explicitly prioritized porting
Type alone while retaining reflection implementations in NeoIL. Type must retain
only basic identity/shape information. The author clarified that TypeInfo can use
the same descriptor while exposing the full metadata model. Finally, descriptive
parameter names across the Raven-authored APIs were requested as the next slice.

**Current action.** Implemented the Raven Type source and checked import path,
with TypeInfo retaining an opaque handle and performing queries on demand. Migrated
Raven reference metadata and examples to `.Info`. Earlier statements that a failed
export check prevented the port were premature: the causes included parameter names,
private constructor/static factory admission and self types inside arrays. Earlier
documentation conflating member descriptors with TypeInfo subclasses is superseded
by these corrections. See [the migration](raven-reflection-api.md).

**Author's direction:** Keep `Type` focused on the necessary identity and shape
information. Reflection and runtime member lookup should be separate; a future
`Type.Info` may expose that lookup explicitly, acknowledging that it can be costly.
TypeInfo and its member descriptors should form a closed algebraic hierarchy.

**Assistant's response:** Accepted this as the next API boundary. Existing member
queries remain available during the transition; the reflection hierarchy and native
descriptor construction must be formalized before moving the implementation from
NeoIL to Raven. The proposal and its compatibility status are recorded in
[reflection-hierarchy-plan](reflection-hierarchy-plan.md) and
[type inspection](type-inspection.md).

## Founding conversation — from the initial brief through Preview 1

### O1. The author sets the experiment's boundaries

**Author's brief:** Build an independent, cross-platform, .NET-derived runtime, starting
with a small interpreter proof of concept. Metadata and instructions should remain
familiar, with deliberate breaking changes where useful. Types should not inherently
choose value or reference semantics; values begin with stack allocation unless requested
otherwise. Void should be usable in generics, functions need no class container, and
Option/Result should model optionality and recoverable errors. Terminal errors are
Faults. The library should remain recognizable without the interface I-prefix.

**Follow-up direction:** Include an assembler, HelloWorld and feature demonstrations,
and document the work. Assembly spelling may differ while metadata and IL stay familiar.

**Assistant's reported action:** Built the interpreter and assembler with 35 instruction
forms, samples, README and design notes; reported 19 passing tests. On the author's
request, committed the initial proof of concept as `112fb7d`. Binary emission was
explicitly unfinished; the artifact was temporary JSON.

### O2. Familiarity becomes an API and metadata requirement

**Author's direction:** Calls need signatures for overloading. Implement the runtime
library in the platform's own IL, with explicit native implementation mappings akin to
CLR InternalCall. Keep familiar consumer APIs even where implementation differs.

**Assistant's reported action:** Added signature-based calls and overloads, a platform
System library and validated native bindings; committed `1899cd2` after the author
requested a commit. The author then prioritized the type system before native interop.

**Assistant's proposal:** Separate type identity and behavior from storage and access;
use canonical primitives and type-owned methods, then address mutation and interfaces.
The proposed `Ref<T>` and future ownership rules were suggestions, not settled policy.

### O3. Memory policy is explored, then deliberately deferred

**Author's questions:** How should stack and heap construction be encoded? Could an
allocation environment select an allocator, with optional GC or counted ownership?
The author rejected assuming Rust's memory model and explored an explicit counted
`Ref<T>` wrapper, while leaving language conveniences to compilers.

**Assistant's response:** Proposed separate allocation, construction and lifetime
services, and pointed out that copying a record containing a counted owner would need
corresponding retain/release behavior. Moving collectors would need references or
handles that can follow relocation, rather than unrestricted native addresses.

**Author's decision:** Defer memory management; implement heap allocation and pointers
first, but preserve the ideas.

**Outcome reported:** Canonical types, type-owned methods and the memory direction were
committed as `44ee213`. Counted ownership and GC remained deferred. This is an early
position, subsequently revised toward managed GC in the continuation; it must not be
retold as though that later decision had already been made.

**Additional author examples retained in this task:** An inline `int[10]` might
imply stack storage; a heap allocation might produce a pointer or later be held by
Ref<int[]>. System.Array<T> could provide a familiar view over separately chosen
storage and ownership. These examples did not settle escaping lifetimes or GC.

### O4. The author revisits assembly syntax and insists on native pointers

**Author's request:** Preserve optional parameter/local names even though execution uses
indices. The author initially left declaration syntax to the assistant, then explored
name-first versus type-first notation.

**Assistant's response and action:** Chose consistent `name: Type` syntax and argued that
low-level execution did not require types before names. Later, the author explicitly
requested `Type optionalName`; the assistant reported migrating declarations, samples
and documentation in `0ba69e3`. When the author clarified that identifiers are only
contextual mappings, the assistant documented numeric normalization and added field
aliases in `848dcf6`.

**Pointer exchange:** The assistant initially recommended tracked allocation-plus-offset
pointers for diagnostics. The author specified native pointers similar to .NET.
The assistant reported native addresses, explicit allocation/free, field access and
loads/stores; the pointer and named-slot work was committed as `519df99`, followed by
native integers in `2d6bfbe`. A later C-ABI P/Invoke sample modified guest memory through
native code (`588b60d`). External boundaries and lifetime guarantees remained limited.

### O5. Fundamentals precede larger frameworks

**Author's direction:** Get fundamentals working and commit each slice. The author also
asked whether UTF-8 should be standard.

**Assistant's response:** Recommended valid UTF-8 with scalar iteration and explicit
UTF-16 compatibility, leaving length/indexing contracts to a separate decision.
Reported successive integer, bitwise, floating-point, checked conversion, memory-copy,
frame-allocation and branch slices. These continuations authorized progress; they are
not evidence that the author independently endorsed every low-level implementation detail.

**Author's next priority:** Establish the type capabilities needed for Option/Result
before reflection. **Assistant's action:** Proposed unions for tagged payloads and enums
for integer constants, then implemented generic metadata (`66cd94a`).

### O6. The author stops a union implementation to question its layer

**Author's intervention:** A union is a carrier accepting variant types. After referencing
Raven, the author asked whether unions should have IL support at all or just be a type
convention, and asked the assistant to stop.

**Assistant's response:** Recommended ordinary carrier/case types, recognizable metadata
and methods, with storage treated separately. It acknowledged that the union implementation
was still uncommitted. Following the author's suggestion of a .NET 11-like convention,
it proposed constructors for alternatives and typed extraction without forcing boxing.

**Action reported:** Removed the uncommitted union-specific machinery and documented the
convention in `378295f`. Existing bootstrap Option/Result behavior remained, so this did
not yet remove every special union operation. Generic record construction, methods and
native storage followed (`7ca411d`, `d813e20`, `c57910e`).

### O7. Pause feature accumulation and examine architectural prerequisites

**Author's direction:** Stop after the current slice, reconsider implementation order,
and identify opportunities relative to CLR. Keep a future high-level language and
library implementation in view.

**Assistant's response:** Paused after marker attributes (`d87c8a6`) and proposed a shared
metadata/IL contract for assembler, compiler and execution backends. It identified copied
receivers, construction, uninitialized storage, resource-owning copies and interface
views as unresolved contracts. It distinguished value semantics from physical placement.

**Outcome reported:** Documented construction and addressed-access proposals (`e43ae0a`,
`21d0897`), explicitly without implementing them yet. After further continuation, added
control-flow and typed verification (`e6988c1`, `2ef3f52`). Verification remained opt-in,
with value-dependent validity checks left to execution.

### O8. Embedding and native compilation influence the whole architecture

**Author's direction:** Provide hosting like CLR, Mono or a script engine, and consider
interpretation, JIT and NativeAOT as architectural targets.

**Assistant's proposal:** Separate runtime services, execution contexts and invocations;
share semantics across backends and make fallback explicit. **Author's correction:**
The execution-mode requirement concerned the entire platform, not specifically hosting.

**Action reported:** Corrected the scope and documented execution architecture in
`6f9aeae`. Subsequent implementation included function/type identities, reusable loaded
programs, module sets, dependency references and revisions, typed invocation and
cancellation. Bounded call-graph analysis (`131a3b5`) prepared for compiled execution;
it did not implement native code generation.

### O9. Useful small programs guide scope

**Author's direction:** Prioritize simple but functional programs, Fault stack traces,
primitive wrappers, strings and arrays. A modified C# or Raven subset might later
produce the same IL; first explain how programs can be built and run.

**Assistant's actions reported:** Documented the progression in `0094515`, implemented
String methods (`3d2d2fb`) and added requested build/assemble/verify/run instructions
(`735576c`). Neither frontend had yet been selected.

**Nullability exploration:** The author suggested explicit declaration-level nullability,
then considered strict compiler/tooling enforcement without Nullable<T> wrapping.
The assistant noted that inline values still require an absence representation and
recorded the candidates (`9ec6206`, `c9c001f`), with no runtime change. Later selection of
type-encoded nullability belongs to the continuation, not this earlier exchange.

**Scope correction:** When networking was mentioned as a future demonstration, the author
reasserted console I/O and fundamentals, cautioning against premature Stream-style APIs.
The assistant recorded a workflow of starting with a runnable program, finding the gap
in existing IL, and distinguishing library logic from host operations (`1286b34`).
Console I/O followed in `220c2c8`.

### O10. Ordinary unions drive ordinary object capabilities

**Author's direction:** Implement constructors, properties/accessors and accessibility
so unions can be expressed as ordinary types; remove special union instructions.
Use familiar .NET access rules for now, leaving redesign for later.

**Assistant's reported actions:** Added property metadata (`e1f2987`), method and field
access enforcement (`a32a8dc`, `c62d5e7`), then constructors (`7a029f4`). Ordinary
Option/Result carriers in platform IL followed in `5606e20`, with boundary migration
still required.

**Author's publication decision:** Define the MVP as a runnable source preview, include
basic reflection, use Raven-like pseudocode to explain IL, and call it Preview 1.
**Outcome:** The assistant recorded six illustrative program mappings (`3b441ee`);
a working language compiler was not a Preview 1 requirement.

### O11. Nested cases, storage concerns and incomplete attempts remain visible

**Author's clarification:** Cases are ordinary nested types, not an inheritance scheme.
For generic unions, Raven uses a non-generic companion owning independently generic
cases; non-generic unions can nest their cases directly.

**Assistant's actions reported:** Recorded that model (`7fe18e2`), implemented nested
ownership (`270fc1f`) and refined the direct/companion rule (`f1a35eb`). The migration
included reverted attempts and changing diagnoses of arity/scope problems. Those reports
are not one verified root cause. The assistant eventually reported companion declarations
in `82a37e3`, still with old wrappers during transition.

**Author's concern:** System.Value might become hidden boxing; explicit low-level
storage was preferable to prematurely fixing that abstraction. **Assistant's response:**
Distinguished typed erasure from pointer-backed storage and documented its limits.
Later, the author directed its eventual retirement once ordinary storage/references
could replace it; that migration requirement was recorded in `322b1bb`.

### O12. Publication requires explaining limits, not only listing features

**Author's questions:** What remains before a preview, and what will reviewers need
answered? The author selected MIT and authorized the proposed preparation slices.

**Assistant's response:** Prioritized finishing union migration, specifying storage
costs, minimal reflection, reproducible demonstrations and platform validation. It
warned against confusing familiar metadata with binary compatibility or interpreter
checks with a sandbox. MIT licensing was reported in `1b7b950`.

**Author's further exploration:** Could Void* carry arbitrary union payloads?
**Assistant's response:** Yes, with type/lifetime responsibility and a clear copying
contract; an owning carrier copy could otherwise leave dangling aliases.
**Action:** A pointer-carrier sample demonstrated stack/heap borrowing, aliasing and
explicit release (`511a312`), without new instructions. Later copy and borrowed-pointer
TryGet demonstrations (`6972853`) retained those lifetime obligations.

### O13. Collections expose the need for checked reference receivers

**Author's requests:** Add ArrayList<T> directly under System.Collections; explore a
small interface slice for the preview. Be explicit that an interface is a reference
view from the high-level language's perspective.

**Assistant's actions reported:** Added ArrayList (`a9a58e9`) and borrowed interface
dispatch (`76f90e5`). It disclosed that receivers were copied: pointer-backed mutation
worked, but inline fields did not write back.

### O14. Correct TryGet to return a case through an output reference

**Author's direction:** Ordinary library APIs should pass safe references unless a
particular API specifically needs native pointers. Reference parameters and receivers
were requested for caller-provided output storage and interface access. The author
asked for documentation showing `&` in Raven-like pseudocode.

**Specific API correction:**

> The general convention for TryGet in unions is Option<T>.TryGet(out Some<T>& value) etc

**Assistant's response:** Adopted overloads selected by the case-reference type, with
familiar `out` in illustrative source and an explicit assignment-on-success contract
in neoIL metadata. At that stage the library used `out(true)` and callers branched on
the Boolean before reading a newly initialized output. Extraction copied a case value;
it did not return a borrowed address into carrier payload storage.

**Author's follow-up:**

> Remove the trygetpointer methods

**Action and observed outcome:** Removed TryGetOkPointer/TryGetErrorPointer from the
pointer-carrier sample and its callers, tests and active documentation. The aliasing
demonstration instead read the carrier after mutation. The combined output-reference
and overloaded-union slice was committed as
[a2de4c4](https://github.com/marinasundstrom/neoCLR/commit/a2de4c4).
The assistant reported the relevant tests, sample verification and Clippy passing.
Historical work-log descriptions of the superseded API were retained as history.

### O15. Follow the API decision into examples and contract checks

**Author's question:** What should happen next?

**Assistant's proposed order:** Integrate case-based TryGet into console/file programs,
review reference contracts, refresh the preview plan, and complete release validation.
It identified System.Value retention as a remaining preview boundary rather than
claiming that Void* alone solved payload storage. The author's subsequent continuations
requested more work; they are not recorded as explicit approval of every detail.

**Actions:** Migrated four console/file samples to TryGet and Value properties
([cdaf3f5](https://github.com/marinasundstrom/neoCLR/commit/cdaf3f5)); then added reference
regressions for conditional forwarding, aliased output slots, receiver mutation and
interface contract matching ([f167512](https://github.com/marinasundstrom/neoCLR/commit/f167512)).

**Assistant's results:** Reported 28 relevant sample tests passing for the migration,
then 38 reference/interface tests and strict Clippy for the review. No runtime change
was needed in that review. The documentation distinguished caller-side static
initialization proofs from per-invocation assignment checks and their conservative limits.
These counts describe the reported runs then, not the current test-suite size.

### O16. Add a familiar equality contract without changing primitive equality rules

**Author's request:** Add a preferably generic equatable interface, based on the .NET
interface, so its type argument specifies what is being compared.

**Assistant's proposal and action:** Added System.Equatable<T> with Equals(T other),
then implementations for Int32, String and System.Type and a Point example using
explicit interface views. It reported this as ordinary library/interface work requiring
no new opcode, automatic boxing, hashing or implicit collection comparer selection.

**Validation issue retained:** A full regression run exposed a test assuming the first
library method was a native declaration. The new interface changed that ordering.
The assistant corrected the test to locate the intended declaration by name, reran it
and the remaining suites, and reported combined coverage of 521 passing tests plus
Clippy. Committed as [50335a5](https://github.com/marinasundstrom/neoCLR/commit/50335a5).

**Still separate:** Broader primitive coverage, hashing, ordering and collection comparer
policy were not implied by this one-method interface. The author's request was not
recorded as a request for all of those facilities at once.

### O17. Articulate the platform direction, while leaving later lifetimes open

**Author's observation:** The platform's direction was becoming clearer: values by
default, explicit reference passing and optional capabilities, with runtime tracking
that need not use a Rust-style borrow checker.

**Assistant's response:** Distinguished checking a live, correctly typed and initialized
slot from requiring exclusive borrowing. Multiple writable aliases were allowed in
the implemented call-scoped subset; a language could impose stronger rules and a future
backend could remove checks it proved unnecessary. Stored, escaping or cross-thread
references still required explicit contracts.

**Action:** Recorded that distinction and refreshed preview gates in
[2d2d7ac](https://github.com/marinasundstrom/neoCLR/commit/2d2d7ac).
This was not a report that all longer-lived references or concurrency had been solved.

### O18. Turn release readiness into evidence, and correct a Windows-only failure

**Assistant's proposed work after the author's continuations:** Verify the minimum
Rust toolchain and clean-source instructions, then audit the source package and draft
release notes. Those were proposed implementation steps, not new author-specified
features.

**Performed checks:** Installed Rust 1.85.0, built a clean tracked-source snapshot and
reported one uninterrupted 521-test run and 13 documented workflow steps passing.
Declared the minimum in Cargo and added minimum/stable CI coverage
([195c68c](https://github.com/marinasundstrom/neoCLR/commit/195c68c)).
Then inventoried 20 locked dependencies, preserved 42 notice texts and checked a
368-file source archive ([a133046](https://github.com/marinasundstrom/neoCLR/commit/a133046)).
The assistant explicitly bounded provenance and platform claims rather than treating
local success as completed cross-platform release validation.

**Author's report:** The recently pushed changes failed Actions.
**Assistant's investigation:** Both Windows jobs failed a test whose LF-only multiline
replacement did not match a CRLF included sample. The intended mutation was absent,
so output remained 42 rather than 52. Linux and macOS had passed. The fix normalized
the source, required exactly one edit marker and exercised LF and CRLF input, with a
separate Rust 1.85 run using a physically CRLF sample
([4fb1f9e](https://github.com/marinasundstrom/neoCLR/commit/4fb1f9e)).

**Author's confirmation:** “It succeeded”. The assistant initially acknowledged this
as the author's report; it later inspected
[run 34145375938](https://github.com/marinasundstrom/neoCLR/actions/runs/34145375938)
and confirmed all six jobs passed. Reported success and independently checked success
are separate steps in the record.

### O19. Request finalization, then publish only after candidate checks

**Author's later comparison:** Some aspects felt closer to WebAssembly than CLR.
**Assistant's response:** Compared separation of execution mechanisms from language
policy, while locating neoCLR's CLR influence in typed metadata, generic types and
members. It explicitly distinguished typed/native-pointer facilities from adopting
WebAssembly linear memory. This was an architectural comparison, not a decision to
implement a WebAssembly backend or import its memory model.

**Author's request:** First asked for a GitHub tagline. The assistant supplied the
experimental, .NET-inspired description emphasizing values and explicit memory and
references. The author then directed:

> Proceed. We need to finalize the release and add badges to README.

**Assistant's concrete plan:** Selected v0.1.0-preview.1 for a source-only GitHub
prerelease, added CI/version/license/minimum-Rust badges, and stated that publication
would follow successful CI on the exact release commit. The version choice was the
assistant's implementation decision under the release request, not a separately
quoted author selection.

**Actions and verified outcome:** Committed and pushed
[f11ec01](https://github.com/marinasundstrom/neoCLR/commit/f11ec01), verified its source
archive membership and notice hashes, and built an extraction on Rust 1.85.0.
[Run 34146500779](https://github.com/marinasundstrom/neoCLR/actions/runs/34146500779)
passed all six jobs before the assistant created the
[Preview 1 prerelease](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.1).
Tool results confirmed the release was published rather than draft, marked prerelease,
and targeted that commit. Uploaded asset hashes and the fetched tag were checked
against the validated local archive and commit.

**Scope at publication:** Source and checksums were published; no prebuilt runtime,
crates.io package, high-level compiler or JIT/AOT backend was claimed. The conversation
then continued into new development. Later GC, Neo and reference work below must not
be read as features retrospectively shipped in Preview 1.

## Post-Preview-1 continuation — author-side retrospective

Recorded 2026-09-09. **Assistant response unavailable for these earlier exchanges.**
Each entry summarizes the author's questions or directions; references to a change
of position are based on subsequent author messages. The entries do not imply that
the assistant originated, endorsed or implemented a particular answer.


### 1. Start with a runtime experiment, and examine its foundations

The opening request in this continuation was to inspect the prototype against its design goals, identify
what should be fixed before proceeding, and expose the choices that future features
would create. Cleanup and separate commits were part of that process.

The author then articulated the central direction:

> We are working towards a runtime platform where value semantics are default for type, passing by reference when explicitly wanting to, lifetimes are deterministic. Pointers are a low-level construct available for native interop.

Destructors, scope exit, the last active reference, Disposable/Closable and Clonable
were explored as related but distinct responsibilities. These were early questions,
not a final declaration that all managed heap cleanup would be deterministic.

### 2. Explicit references should still be managed

The author repeatedly distinguished choosing reference semantics from manually
managing a reference's validity. A reference should behave transparently once obtained;
its allocation location should not dictate how the caller accesses its members.

The conversation explored returning a reference to a local counter. Later direction
made the escape rule explicit: a function must not return an address into its own
frame. Returning a reference into an argument owned by an outer frame is different,
as illustrated by returning &counter.Age from a Counter& parameter. The runtime
should validate this and fault on an invalid escape.

This is a meaningful evolution to retain: the attractive source example prompted a
question about allocation and lifetime, rather than settling that question by syntax
alone. Later discussion also left copying a value onto the heap as a possible explicit
future operation.

### 3. Reuse managed references and familiar instructions

Ref<T> was explicitly described as a proposal, not the chosen abstraction. The author
preferred reusing CLR-style managed references to express reference intent.

> We should keep close to .NET CLR instruction set and semantics when we can, unless we deviate to improve. We are still in a preview and you are allowed to make breaking changes

The discussion of newobj, a possible newval instruction and the existing initobj
asked whether the model could use consistent, familiar patterns before introducing
new machinery. Preview compatibility was not a reason to preserve an unsound choice.

See [managed-reference semantics](managed-reference-semantics.md).

### 4. Retain garbage collection and managed productivity

The author subsequently directed the project to implement garbage collection for
managed heap memory. The platform was still intended to offer the productivity of a
managed runtime, alongside explicit storage and reference choices.

This qualified the earlier lifetime discussion: frame-owned values and managed heap
objects need different lifetime mechanisms. Native pointers remained a separate
interop capability. Pinning, calls into native code and native consumption of managed
data were raised as future considerations, while the immediate scope stayed focused
on a sound memory-management starting point. GC monitoring should grow when needed.

### 5. Treat reference use as an addressing choice, not a different kind of object

An Object hierarchy was considered useful for library familiarity, but the author
did not want every runtime type forced to inherit from Object. Equality and hashing
were discussed in terms of value content, with reference identity a separate question.

> There is no inherent value-typeness or reference-typeness in our system. It's just an addressing mode.

The sequencing mattered too: reach a memory-management milestone, then explore the
object model and adapt memory management as inheritance required it.

### 6. Build Neo to test and explain the platform

A small high-level language with a Raven-like syntax was proposed as an end-to-end
exercise and named Neo. Documentation, runnable examples and a grammar were requested.
Control flow and later patterns/unions would make the examples more expressive.

The compiler should stay updated as neoCLR evolved, but it was not intended to become
a complex, full-fledged compiler. Rewriting parts of the runtime library in Neo, and
perhaps bootstrapping the compiler, were possibilities for the future rather than
immediate commitments.

The language therefore became both a demonstration and a way to expose gaps in the
runtime's contracts.

### 7. Make reference access transparent, including collections and views

The author objected to samples explicitly dereferencing managed references: accessing
an int& or Foo& should be handled automatically by the compiler. Pointers were a
separate case.

Arrays raised the same storage question: an owned int[3] value and an int[]& referring
to a managed heap array should fit one consistent model. Interfaces then supplied a
practical test of reference views and virtual dispatch. The usual I-prefix naming
convention was not wanted.

Later, ArrayList<Foo&> made the consequence concrete: the list stores references,
Add accepts a reference, and accessing an element remains transparent. Its internal
buffer should be a managed array reference. These requests connected the abstract
model to ordinary library use.

### 8. Make the running system inspectable and its contracts reviewable

A live debugger was requested to show call frames, stack memory and heap memory.
The author chose an interactive terminal interface and immediately connected it to
the need for Neo-to-IL source mapping. Stepping into and over calls, and instructions
for using the debugger, were part of that concern.

Managed references also prompted a review of library API parameters, an API design
document, and reflection/introspection. MethodInfo, FieldInfo and PropertyInfo were
preferred for Type; FunctionInfo was only relevant if module-level functions were
being queried. The hierarchy could wait until the platform supported it properly.

### 9. Make progress understandable outside the conversation

The author requested a changelog reconstructed from previous work and maintained
with every commit, keeping published sections unchanged. Release preparation included
asking what still needed fixing, rather than equating a growing feature list with
readiness.

Small sample details mattered as well: uninitialized array allocation should not
require empty braces, and examples should use indexer syntax rather than accessor
method spellings. Familiarity should be visible in actual code.

### 10. Put characteristics at the layer that can justify them

The roadmap grew to include inheritance, nullability, delegates, generic constraints,
async, dynamic dispatch, enums and a useful base library. The author clarified:

> The familiarity is mostly in APIs and behaviors.

The project should research .NET/CLR alternatives, while remaining free to avoid
complexity imposed by compatibility constraints elsewhere. That freedom was not an
instruction to put everything into the runtime.

Immutable bindings were explicitly kept at the language level for now, because a
universal runtime slot mechanism lacked a concrete justification. Nullability was
examined separately: slot property or type property? The eventual direction was an
explicit type characteristic for both values and references, with null a special
state rather than simply zeroed data. Option remained the preferred expression of
many forms of optionality.

### 11. Let useful library scenarios determine the next building blocks

The next preview should demonstrate a recognizable runtime library and the benefits
of Option and Result. That raised dependencies on inheritance, interface inheritance,
abstract classes and constructor chaining. Base-class use should be a reference view
of the concrete object, not a sliced value copy.

Delegates were retained as a useful platform abstraction on which languages could
build, including future closures. Inference and automatic function-to-delegate
conversion could reduce language friction. Comparable, Iterable, Iterator, character
helpers, math and separate date/time types followed as practical library needs.
Globalization and sophisticated formatting could wait; retrieving the system's local
date and time was already a useful bounded scenario. LINQ was also deferred.

### 12. Question whether the experiment is worth its friction

The author did not treat explicit references as a proven usability improvement:

> Is the friction caused by this really worth it? Right now we don't know what the real experience will be using this language.

The discussion contrasted design-testing samples with code resembling real applications.
Long experience with C# was acknowledged as part of the perspective, while recognizing
that unfamiliarity alone does not establish that a model is bad. The response was to
try practical scenarios and investigate whether problems came from runtime behavior,
library implementations or the language projection.

The cleanup discussion was revisited in that context. A value facade around heap
storage might suggest destructor-based cleanup, but the author later acknowledged:

> You are right. Since they are heap allocated resources, we don't need manual cleanup

This does not settle cleanup for every external resource. It records a correction
about managed heap storage. See [the reference-experience experiments](experiments/reference-experience/README.md).

### 13. Use real workflows to refine unions and type-design guidance

Raven's Result.Ok/Result.Error projection led to questions about generic inference.
The author explained independent case types: Ok<T> need not know E, yet can be accepted
by a Result<T,E> carrier. Each carrier constructor defines an accepted variant type.
Source union declarations, if let and let … else were requested to make this usable
in the order-workflow example.

Guidance for type authors was also requested: small immutable data often suits copying;
shared mutable state often suits references. But size alone does not define the contract:

> You need to look at a type as the sum of its parts and what the contract guarantees and don't. Passing reference into something has an implication.

A type's contained references, copying behavior and promises matter alongside its
surface syntax. See [type-design guidelines](type-design.md) and
[the case-construction discussion](result-construction.md).

### 14. Explore reduced syntax, then retain explicit borrowing

Removing the explicit & when passing a value to a reference parameter was considered
as a possible ergonomic improvement. The author then settled the immediate direction:

> Explicitness with & is the right way to go here

A separate clarification concerned assignment. Foo& assigned to Foo& should copy the
reference and retarget the binding, not copy the contents into the previous target.
The example let foo2 = foo after let foo = new Foo() was intended to preserve the
same target. Member access such as foo2.Bar() follows that reference automatically.

The rejected shorthand and the assignment correction should both remain in the
record: explicit borrowing and transparent use of an existing reference are different
choices, not contradictory goals.

### 15. Separate platform capability from language usability

Generic constraints and calls through generic receivers brought the boxing comparison
back into focus. The author observed that neoCLR's architecture need not use boxing
to the same degree as .NET, but also distinguished that capability from evidence about
how pleasant a high-level language is to use.

> One interesting thing to see is how few changes we need to make to the instruction set to accommodate our model.

The direction was to fix essential Neo behavior for demonstrating both high- and
low-level patterns, then return to unfinished unions. This observation about the
instruction set was an architectural interest, not a benchmark result.

### 16. Add tools and features when a concrete need makes them useful

The author requested readable compiler IL for debugging, noting that binary instruction
encoding could remain a future step. Enums with flags had a specific first use in
BindingFlags. That immediately led to another connection:

> This ties into constant support

The possibility of revisiting the feature later was explicitly retained. These are
examples of identifying an adjacent abstraction without requiring the current slice
to solve its entire future design.

## Recent exchanges — both sides available, recorded 2026-09-09

These entries follow the available conversation order. The recording date does not
supply a missing timestamp for an individual message.

### 17. Finish essential receiver behavior, then return to unions

**Author's direction:** Fix the essentials in Neo because it tests and demonstrates
both high- and low-level patterns, then finish unions. The author also observed how
few instruction-set changes the model appeared to require.

**Assistant's response:** Proposed finishing and committing generic receiver adaptation
before moving to generic source carriers. Described the receiver work as establishing
behavior, with future JIT optimization still unproven. In response to the instruction-set
observation, pointed to changes in the type system, verifier and lifetime rules, while
retaining much of the familiar instruction vocabulary.

**Subsequent direction:** The author said to continue.

**Reported outcome:** The assistant committed receiver adaptation and generic carriers
separately (`1081870`, `32de966`). Inline generic cases remained unfinished. These are
assistant-reported implementation outcomes, not additional statements by the author.

### 18. Request readable IL for debugging

**Author's direction:** After the next slice, consider a disassemble command, or at
least a way to show the IL resulting from Neo code. Binary instruction encoding was
acknowledged as future work.

**Assistant's proposal:** First expose the existing compiler lowering as readable
source-to-IL output. Treat arbitrary-artifact disassembly as a separate writer rather
than require a binary format before providing useful inspection.

**Status at that exchange:** Recorded as upcoming tooling work; no claim that a full
disassembler had been requested and approved in detail.

### 19. Use BindingFlags to motivate enums, and identify the connection to constants

**Author's direction:** Implement enums with flags support, citing a concrete need in
BindingFlags. A later continuation moved work into that slice.

**Assistant's proposal:** Begin with Int32-backed enums, nominal identity, named values,
zero defaults and unnamed integer values. Preserve BindingFlags bit values and filter
behavior. Reuse integer instructions and typed methods; leave other widths and formatting
for later. During implementation the assistant reported keeping existing factories
compatible and adding enum reflection queries.

**Author's observation:**

> This ties into constant support

**Assistant's response:** Distinguished named enum literals from writable storage and
proposed using literal metadata as a starting point for general constants. Placed
constant-expression evaluation in Neo and encoded type/range validation in the runtime;
kept ordinary immutable bindings separate.

**Author's qualification:**

> We might revisit this feature later

**Assistant's response:** Agreed to keep enum/literal support provisional and finish
validation of the bounded slice, leaving general constants for a later revisit.

**Reported outcome:** Enum/BindingFlags work was committed as `78f418b`. The record
does not treat the author's qualification as approval of every implementation detail.

### 20. Implement the first IL inspection command

**Author's direction:** Continue.

**Assistant's action and explanation:** Implemented emit-il for one Neo source file,
with stdout or new-file output, original source mappings, verification without guest
execution, and no-overwrite behavior. Documented that it is source emission rather
than JSON artifact disassembly.

**Assistant's validation report:** Existing CLI regressions passed. A new test initially
failed because parallel tests selected the same temporary directory; the assistant
changed the test helper and reran it. Reported all ten selected CLI tests and clippy
passing, then committed `6888ffd`.

**Still open:** Artifact disassembly and binary encoding. Neither was represented as
completed by this command.

### 21. Decide what this record should preserve

**Author's request:** Keep a record of the thoughts and direction given during the
project, so others can understand how the author works creatively as a developer
using AI.

**Assistant's first proposal:** Write a development reflection, separating stated views
from editorial synthesis and preserving changes of mind.

**Author's clarification:**

> Yes, keep a timeline of directions and questions.

**Assistant's response:** Proposed an ordered timeline without fabricated historical
dates, plus repository instructions to maintain it.

**Author's further clarification:** The conversation itself is important. The author
compared the purpose to meeting minutes: record what the author said, what the assistant
might have proposed in return, the questions and decisions, so people can see the
collaboration. The author also reiterated: “keep a record”.

**Assistant's correction:** Changed this document from a reflective account toward an
attributed conversation record. Agreed to record direction/question, response/proposal,
subsequent decision and open issues, and to identify unavailable earlier replies instead
of reconstructing them.

**Recording convention now requested:** Preserve the exchanges and the author's role
in directing them. Do not substitute an account of what the assistant thinks the process
says about the author's personality or development philosophy.

**Author's definition of outcome:** “Actual direction” means the outcome: what action
was taken. The author then identified the original shared chat as the project's start.

**Assistant's initial action:** Read that source and added a separate, linked record of its
founding exchanges through Preview 1. Updated this record and repository instructions
to include actions and outcomes, distinguishing reported work from proposals and
unavailable evidence. The original conversation reveals earlier positions on memory
management and nullability that must remain visible alongside later changes.

**Author's purpose clarified:** The core purpose is to give others insight into how the
author works with AI and approaches software development as a whole, using neoCLR as
the concrete case. This purpose guides which exchanges to retain, not just which
technical milestones to list.

**Author's next request:** Merge the origins and timeline documents.

**Assistant's action:** Consolidated them here in conversation order, combining the
founding account with the retained detailed exchanges and removing the separate origin
file. Kept the distinctions between historical reports, observed validation and missing
replies. The README and workflow now lead to this single record.

### 22. Ask where performance-oriented complexity should be visible

Recorded 2026-09-09.

**Author's observation:** Developers in many high-level environments expect an object
variable to contain a reference, automatically followed on member access. C# makes a
stronger distinction through structs and classes, while Java historically separates
primitives from objects. The author sees .NET's struct, reference and readonly features
as ways to unlock performance beyond familiar object behavior, often inside library
implementations rather than directly in ordinary consumer code. Java's work on value
classes approaches the problem differently.

**Assistant's response:** Distinguished copying an object reference from passing the
variable's storage by reference. Framed neoCLR's question as whether choosing value or
reference use independently of the type is worth the added decisions. A language could
supply defaults without removing that runtime capability; no such change was selected.

**Comparison checked:** Microsoft's [performance guidance](https://learn.microsoft.com/en-us/dotnet/csharp/advanced-topics/performance/)
explicitly connects structs, ref access and readonly access with reducing allocation
and copying. Structs are also ordinary modeling tools, and public APIs such as Span<T>
expose these contracts: they are not exclusively hidden implementation optimizations.
OpenJDK's [Valhalla project overview](https://github.com/openjdk/valhalla-docs/blob/main/site/_index.md)
and [background design notes](https://github.com/openjdk/valhalla-docs/blob/main/site/design-notes/state-of-valhalla/01-background.md)
describe value objects without identity, enabling optimized representations. This records
the project's design direction, not a claim that all Valhalla features have shipped.
Removing identity is different from neoCLR's choice to reference a mutable value.

**Assistant's interpretation and open question:** The useful comparison is where each
platform places complexity and how much reaches application code. neoCLR may unify
runtime capabilities yet expose more decisions in Neo. Library and language defaults
need evaluation with real programs; a more uniform runtime alone does not establish
better usability or performance.

**Action:** Recorded these two exchanges and checked the comparison against primary
sources. Runtime semantics, syntax and roadmap priorities remain unchanged.

### 23. Reconsider universal value defaults in favor of ordinary class references

Recorded 2026-09-09 at the author's explicit request.

**Author's exploration:** Languages should not have to live with the platform's universal
value-default decision. Ordinary class use should match the expectation that a variable
holds a reference, with automatic member access. Value behavior should be an opt-in;
explicit reference passing is more appropriate when accessing those values without
copying. Requiring ampersands throughout ordinary application code was identified as
the wrong experience. The author connected this reconsideration to understanding the
.NET class/value distinction and its integration of primitive wrappers into the hierarchy.

**Assistant's response:** Identified this as a revision of the original universal
value-default premise. Distinguished passing an object reference by value from passing
the caller's slot by reference. Suggested ordinary class references, opt-in value
classes and separate output/rebinding contracts, while retaining managed references
for interior access, value borrowing, readonly access and lifetime checks.

**Assistant's proposal, not an author decision:** Express a type's intended usage default
in metadata for consistency across languages. Keep that contract separate from physical
allocation. The author has not yet selected the metadata encoding, syntax, hierarchy
rules or migration strategy; the assistant's preference is not recorded as an approved
implementation design.

**Author's subsequent clarification:** “I don't say we should go down this route just yet.”
The class-reference/value-opt-in model is an alternative to investigate, not a selected
platform direction or authorization to implement a migration. The assistant had framed
the discussion too strongly as a change in direction and corrected this record.

**Outcome and next action:** The author requested this timeline entry and a search for
existing design/architecture evaluation documents. The assistant found the
[reference-experience evaluation](experiments/reference-experience/README.md),
[type/API design discussion](type-design.md),
[runtime groundwork review](runtime-groundwork-review.md),
[execution architecture](execution-architecture.md) and
[design research process](design-research.md). The experiment is the closest existing
assessment of reference friction; the groundwork review is an earlier milestone-specific
architecture assessment, not a current audit. Existing implementation documentation still
describes the value-default model. This entry records the alternative and the author's clarification. The current
value-default model remains in place; no replacement has been selected.

**Author's further reasoning:** Reference behavior is the familiar expectation for
ordinary class use, with heap management largely transparent to application developers.
The author sees explicit value behavior as a type-design intention that should surface
when it matters, and finds Java's modifier-based evolution of existing wrapper classes
potentially less intrusive than C#'s class/struct split. These observations continue
the exploration; they do not supersede the clarification that no route has been chosen.

**Assistant's qualification:** This expectation is common, not universal across modern
languages. Value semantics concern copying and identity, not necessarily a short
lifetime: values can live inline in long-lived heap objects. Valhalla's opt-in also
restricts identity and mutation; it is not just a storage hint or C# mutable-struct
behavior under another spelling. Its wrapper migration therefore needs separate
consideration from neoCLR's ability to reference mutable values.

**Evidence and action:** Added this reasoning to the existing entry, with no code or
architecture migration. The [C# struct specification](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/structs)
and [Java team's value-class demonstration](https://inside.java/2025/10/27/try-jep-401-value-classes/)
support the distinctions. The Java demonstration includes Integer and LocalDate in
preview mode; it does not establish universal deployment or allocation elimination.

**Author's clarification about lightweight values:** The earlier lifetime observation
was general: a value can live within a long-lived heap object, while still being
lightweight in representation. **Assistant's response:** Acknowledged that its lifetime
qualification had been narrower than the author's point required.

**Author's preference for the explored model:** The author now understands why C# uses
the term struct and recognizes that .NET and Java's emerging value behavior differ,
with architecture and backward compatibility informing those designs. This was not a
request to adopt Java's behavior. The preferred concept is a value class that can
preferably be inlined, rather than a separate struct equivalent.

**Assistant's interpretation:** Described an ordinary class with an explicit value
contract, with copying, mutation, identity and reference access still requiring separate
decisions. Inline storage would be enabled by the contract, not promised in every use.
This interpretation does not establish those unresolved semantics as agreed decisions.

**Action and scope:** At the author's request to record, appended these clarifications
to the ongoing exploration. The preference concerns the shape of a possible value-class
model; the broader decision to replace current defaults remains open. No runtime,
metadata or language change was implemented.

### 24. Separate storage, mutability, identity and equality

Recorded 2026-09-09.

**Author's observation:** A possible value class resembles a record locked against
modification. The author wants to distinguish the individual characteristics, especially
storage and reference/value access, and points to C# record classes as objects that
participate in value equality through Equals/GetHashCode while retaining reference use.
Record structs and record classes combine these properties differently.

**Assistant's response:** Distinguished equal contents from being the same object.
A record class can preserve observable reference identity while providing generated
value equality; assigning it still copies the reference. A record struct copies its
value. Neither record syntax nor value equality alone guarantees deep immutability.
Ordinary classes can opt into value equality, but do not all do so automatically.
The [C# record reference](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/record)
provides the comparison, including equal records with distinct reference identities.

**Open design question:** Which characteristics should a possible value-class declaration
combine, and which should remain independent? Accessing storage through a reference
need not, by itself, settle the language's equality or object-identity contract.

**Action:** Recorded the distinction and checked the C# comparison. No particular
combination of semantics or change to neoCLR's current defaults was selected.

### 25. Explore runtime contracts, then return to the intended object experience

Recorded 2026-09-09 at the author's request.

**Author's question:** Should these characteristics be runtime functionality or
conventions built on existing interfaces and other facilities?

**Assistant's provisional answer:** Equality, hashing and record conveniences could
use ordinary methods and compiler generation. Storage, copying and reference validity
need shared execution contracts. Immutability or absence of identity would need runtime
enforcement if optimizations relied on them. Suggested identifying the missing guarantee
before adding a single category that bundles several characteristics together.

**Author's alternative:** Under a possible reference-default model, neoCLR could mark
some types as inline, implementing struct-like behavior directly without a ValueType
base-class split. The author proposed an illustrative IL declaration:

```text
.type inline value Int32
```

The author considered primitives with object-like wrappers but no separately declared
instance data, and explicitly left open whether the inline flag should trigger value
semantics. This was exploratory spelling, not an implemented or selected syntax.

**Assistant's response:** Distinguished an inline representation hint from a contract
that makes assignment copy contents. Suggested that `value` and `inline` could express
separate concerns, while noting that primitive payload representation is another matter:
no declared fields alone does not establish an integer's runtime representation.
Whether flags belong to a type or each use remained open. These were assistant proposals,
not decisions attributed to the author.

**Author's clarification of the priority:** Preserve object-like qualities on primitive
types and allow user-defined types to have that behavior. The low-level representation
need not mirror the high-level abstraction. The author compared the exploration to
Java's effort to bridge primitive and wrapper types through runtime optimization with
semantic consequences, without requesting adoption of Java's model. The aim is familiar
.NET usage while improving runtime architecture and design, rather than reproducing
its historical decisions. The experiment remains useful for learning about alternatives.

**Assistant's correction and suggested evaluation:** Acknowledged moving too quickly
toward modifiers and categories. Reframed the goal as methods, interfaces, generics and
coherent APIs for primitive and user-defined data, with efficient representation and
precise copying, aliasing, mutation and identity contracts. Proposed testing one small
API with both a built-in number and a user-defined value, including generic/interface
use. That test was suggested, not performed or selected as the next implementation task.

**Action and remaining scope:** Recorded this exchange. No new metadata flag, value-class
category, convention or default behavior was implemented. The broader model remains
under evaluation; this entry does not override the author's earlier clarification that
changing routes has not been decided. Prior .NET/Valhalla comparisons in entries 22–24
provide context; the Java analogy here is attributed reasoning, not a new release or
performance claim.

### 26. Evaluate improvements by continuity of developer experience

Recorded 2026-09-12.

**Author's evaluation criteria:** An improved .NET should not fundamentally change how
developers work. The author considers it probably wise to associate reference/value
semantics with types, while leaving their representation open. The platform should
avoid introducing manual management and should simplify inconsistencies in the familiar
experience. Runtime APIs should probably favor Result and Option unions; expected,
recoverable failures should prefer Result, with exceptions reserved for irrecoverable
Faults. Keep the .NET instruction set as the basis, make the platform easy for languages
such as Raven and C# to target, and support nullable values even if APIs prefer to avoid
them. Nullability could later have an updated or unified model.

**Author's architectural emphasis:** Distinguish language responsibilities from runtime
responsibilities. Runtime implementation details can remain hidden from developers while
still providing meaningful benefits. Familiar usage does not require identical internals.

**Assistant's assessment:** These criteria shift the evaluation toward whether runtime
changes preserve productive, familiar source-level behavior. The existing experiment's
managed references and storage machinery can be assessed as implementation capabilities,
without requiring all their distinctions to appear in everyday source syntax. Type-level
defaults remain a proposed direction, not an implemented replacement for current semantics.

**Tradeoff identified by the assistant:** Moving expected failures from exceptions to
Result changes API and control-flow contracts. Familiar targeting should therefore be
tested with concrete language mappings, including how existing exception-based code
would migrate; familiar instruction spelling alone does not establish compatibility.
Fault containment and cleanup, nullable representation, type defaults and their metadata
encoding remain contracts to evaluate, not details settled by this discussion.

**Action and scope:** Recorded the criteria and assessment. No runtime or compiler change
was made, no syntax or metadata encoding selected, and no architecture migration started.
Existing [reference-experience evidence](experiments/reference-experience/README.md),
[type-design discussion](type-design.md) and [design research process](design-research.md)
remain starting points for a subsequent evaluation rather than proof of the proposed model.

### 27. Evaluate three projections before changing implementation

Recorded 2026-09-12.

**Author's question and instruction:** Asked what to do next, then directed the assistant
to continue with its proposed bounded architecture evaluation of the order workflow.

**Assistant's proposal:** Compare current explicit modes, shared type-declared defaults
and language-only defaults; inspect construction, assignment, parameters, collections,
interfaces and Result/Option use. Produce concrete code comparisons and a recommendation
before changing runtime behavior.

**Action and findings:** Wrote the [evaluation](reference-defaults-evaluation.md), inspected
metadata, lowering, slot and return checks, emitted the current workflow's IL, and ran
22 focused tests plus the SDK 10.0.100 C# baseline. All passed. Existing sharing/snapshot
behavior works; cleaner spelling can plausibly use current signatures. Reference-slot
outputs remain a real compatibility gap, while shared declaration intent matters for
importers defining new uses of a type. More automatic heap construction can add facade
allocations; no performance improvement was inferred.

**Assistant's recommendation, not a selected migration:** Test shared usage defaults that
lower to explicit signatures, with a bounded opt-in compiler/import experiment before
considering an enforced nominal category. No experimental compiler was needed to establish
the existing lowering evidence in this slice. Illustrative alternatives were not executed.
The author has not yet selected the recommended implementation or changed current defaults.

### 28. Keep runtime improvement primary and Neo as its test frontend

Recorded 2026-09-12.

**Author's clarification:** Runtime behavior and Neo language behavior are separate
levels. Both can improve, but the main focus is the runtime. Neo exists primarily to
demonstrate and test patterns by compiling to IL and running on neoCLR. Seek some
compatibility with the existing CLR, avoid excessive divergence, retain the good parts
and their foundations, and improve what can usefully be improved now.

**Assistant's response:** Acknowledged that improving Neo's projection is not a substitute
for evaluating runtime contracts. The earlier compiler/import experiment remains a
possible validation tool, not the project's primary objective. Proposed first assessing
reference/slot distinctions, construction/copying, initialization/nullability and generic
interface use against CLR behavior. The exact compatibility level is still open; no
binary compatibility commitment was inferred from the author's statement.

**Action:** Updated the [evaluation](reference-defaults-evaluation.md) to put that runtime
assessment before the optional compiler experiment. Preserved the previous findings and
recorded why the priority changed. No runtime or language implementation changed.

### 29. Make an existing Raven compiler target the runtime and its library

Recorded 2026-09-12.

**Author's goal:** C# and Raven should target neoCLR with limited adaptation because
its runtime interface remains similar to CLR. The author clarified that exact exception
compatibility is not required: this is a new platform, and migrated code should adapt
expected failures to Result instead of centering its design on exceptions.

**Assistant's response:** Distinguished source targeting from unchanged binary execution
and separated ABI concerns from metadata, IL, library and behavioral contracts. Identified
cleanup/Fault boundaries and exception-centered API migration as explicit design work.

**Author's proposed experiment:** Address the lack of binary metadata support and add a
neoCLR target to the existing Raven compiler. It should compile against neoCLR's runtime
surface and load neoCLR's own library. The author then requested documentation and a
new branch for this experiment.

**Assistant's proposed order:** Inspect Raven's backend, define the artifact/library
contract from actual dependencies, then compile and run a Console HelloWorld before
expanding to user-defined types and Result. Raven's backend details are not yet verified.

**Action:** Created `codex/raven-neoclr-target` from `64024d6` and documented the
[experiment](raven-target-experiment.md), including the first acceptance milestone,
binary-format choices and scope limits. Linked it from compatibility and format guides.
No Raven/compiler or binary-loader implementation was performed in this documentation
slice; no new reference/value default was selected.

### 30. Inspect the existing Raven backend before inventing a target

Recorded 2026-09-12.

**Author's instruction:** Continue with the planned experiment slices, beginning with
Raven inspection. **Assistant's action:** Located Raven, read its repository instructions
and compiler-investigation skill, and pinned its clean checkout at d92b02812740ae052f277c23151e9cc208f7672d.

**Findings:** Raven imports PE symbols with MetadataLoadContext and emits through
PersistedAssemblyBuilder/ManagedPEBuilder plus Cecil normalization. It already has a
target-core-library option and retargeting tests. Those hooks do not provide a full
neoCLR target: .NET framework discovery, host core identities, Unit/void mapping and
exception-based framework projections still need explicit target contracts.

**Outcome:** Produced the [integration map](raven-backend-integration-map.md), with pinned
source links, adaptation points and a proposed minimal Console probe. Source and test
bodies were inspected; no Raven build/test or target execution was claimed. Raven was
not modified. The next slice should define the minimal library/call contract and run
an emission/reference-facade probe before selecting a binary format.

## 31. Minimal Raven target probe — recorded 2026-09-12

**Author direction:** Continued the experiment after its backend investigation. The
standing direction is to test existing compilers against neoCLR's own library and
binary surface, with the runtime as the primary focus.

**Assistant action:** Built the pinned Raven compiler and added an emission-only
Console declaration probe plus a normal .NET control. Inspected the output references,
helper definitions, methods and entry point; tested nonexistent members and omission
of the Console library. Raven source was not modified.

**Outcome:** Target Console binding and core-reference rewriting worked. The omitted
library still resolved through the host environment, so the isolation negative failed.
The declaration fixture remains incomplete and output retains mscorlib. These findings
are recorded in the [minimal contract](raven-minimal-target.md) and reproducible report;
no neoCLR execution or complete compiler target is claimed.

**Assistant proposal, still open:** Evaluate signature-driven CLI-to-runtime translation
for no-result calls alongside the binary container decision. This may preserve the
existing writer and inhabited Void model, but requires explicit validation and debugging
mapping. The author has not selected that implementation or changed class/value defaults.

## 32. Binary profile and dependency audit — recorded 2026-09-12

**Author direction:** Continued the Raven target experiment after the emission probe.

**Assistant response/action:** Investigated the explicit host-assembly seeding in Raven's
metadata setup. Added an experiment resolver that only accepts supplied assemblies and
checks metadata reference resolution, with positive and negative fixtures. Documented
CLI PE reuse and signature-driven translation as the bounded experiment direction.

**Outcome:** The audit rejects the incomplete target and host-fallback artifacts. It
accepts self-contained metadata and an explicitly supplied dependency, and detects
missing or mismatched identities/types/members. This is a tested tooling boundary, not
a fix to Raven's binder or an implemented runtime loader. See the
[binary-profile decision](raven-binary-profile.md) for evidence and limits.

**Still open:** The compiler needs isolated target resolution and complete core
declarations. Generated-helper policy, input stack validation and reader implementation
remain before execution. The assistant's format choice is scoped to this experiment;
the author's continuation is not recorded as explicit approval of every design detail.

## 33. Isolated compiler metadata import — recorded 2026-09-12

**Author direction:** Continued after the binary-profile and dependency-audit slice.

**Assistant action:** Implemented opt-in explicit-only metadata imports in Raven on
`codex/neoclr-target-resolution`, selecting the metadata core independently of the
compiler host. Preserved default .NET import behavior and blocked incremental state
reuse across import policies. Updated the neoCLR probe to test target Console binding
and missing-library diagnostics through the new API.

**Outcome:** Raven commit `1d7341fa64a66b514e5e68031b6d072d8140ea3a` passed 15 focused
framework/import tests and the .NET 10/11 target matrix. The updated probe binds the supplied Console fixture and rejects omitted
Console instead of finding it on the host. The core reference artifact remains
incomplete; this is compiler integration progress, not runtime execution. Exact
revisions and validation are linked from the [probe instructions](experiments/raven-target/README.md).

**Still open:** Complete neoCLR core declarations, generated-helper handling, profile
verification and binary loading. No change to neoCLR's class/value defaults is implied.

## 34. Minimal core declarations — recorded 2026-09-12

**Author direction:** Continued the experiment and specified that any Raven repository
work must remain isolated on a feature branch. The assistant confirmed Raven was already
on `codex/neoclr-target-resolution`; this slice required no additional Raven changes.

**Assistant action:** Built a small metadata-only core reference assembly and compiled
the static-call corpus with that single reference. Added negative Console/signature
checks and corrected Cecil inspection order so synthesized in-memory references are
not reported as artifact dependencies.

**Outcome:** The Console, empty/nested-call and Int32-return examples bind and emit with
closed metadata dependencies. This is a reference surface for testing; neoCLR has not
executed these binaries. Earlier application mscorlib inventory entries included a
synthetic Cecil reference; the old incomplete fixture's real dependency remains.

**Still open:** Bind the reference surface to real System implementations, decide
helper admission, and implement/validate the binary reader and call translation. The
[core declaration contract](raven-core-declarations.md) records the evidence and limits.

## 35. Compatibility work belongs primarily in neoCLR — recorded 2026-09-12

**Author direction:** Make as few changes to Raven as possible. Generally useful metadata
improvements can belong there, but most work should improve neoCLR's CLR compatibility.

**Assistant proposal/action:** Reconsidered the earlier blanket Void adaptation and added
an explicit no-result return convention to neoCLR for static IL methods. Calls and returns
use the existing instructions, with verifier/runtime enforcement. Existing inhabited Void
methods retain their behavior. No Raven changes were needed.

**Author clarification:** Void should remain a real type usable as a generic argument,
without necessarily changing ordinary IL return behavior. The author pointed to runtime
async as a comparison and emphasized preserving the metadata/instruction interface.

**Assistant response:** Kept type/storage semantics separate from the return convention,
added generic Void regression tests, and recorded the runtime-async design comparison as
future groundwork rather than expanding this slice into async implementation.

**Further author direction:** Align with .NET value-type/reference-type semantics later.
Avoid working around that model unless necessary to demonstrate the project's direction.
The assistant treats this as the experiment's intended direction, not an implemented
migration or a requirement to redesign the type model in this slice.

**Outcome:** Focused runtime tests exercise empty/nested calls and the existing Console
library boundary. Binary reading and execution of Raven artifacts remain separate work.
The [return-convention document](no-result-methods.md) records the restricted first subset,
host-envelope behavior and the still-needed library binding policy.

## 36. Void type identity versus stack representation — recorded 2026-09-12

**Author clarification:** “In the neoCLR, void will be treated as a real type and the
compiler targeting it has to adapt to it”, followed by “Even if void doesn't really
put anything on the stack.”

**Assistant response:** Distinguished real type participation from evaluation-stack
representation. The previous no-result slice does not make inhabited generic Void's
current internal representation a permanent contract. Targeting compilers must understand
neoCLR's Void semantics; binary generic encoding and uniform zero-stack handling remain
open implementation work.

**Action/outcome:** Updated the [return-convention document](no-result-methods.md).
Continued the independent [standard CIL decoder](cil-decoder.md) groundwork with byte
offsets and token preservation. This does not implement the broader Void representation
change or load Raven binaries.

## 37. Familiar binary contracts and a useful Raven milestone — recorded 2026-09-12

**Author direction:** New runtime semantics should not automatically change metadata or
the instruction set. Preserve .NET's major behavioral expectations, retain intentional
Void and library improvements, and make adaptation easy for existing compilers.

**Further clarification:** The mission is a minimal, useful Raven subset targeting neoCLR.
Aligning value/reference type semantics is central; the original explicit-reference model
should not drive compatibility workarounds.

**Assistant response:** Agreed to finish the bounded PE/body reader already in progress,
then prioritize runtime type classification using a class-alias/value-copy Raven scenario.
Binary parsing is supporting work, not the primary acceptance milestone.

**Action/outcome:** Added standard PE32 container and tiny/fat method-body inspection with
.NET-emitted fixture comparisons. Updated the [experiment order](raven-target-experiment.md#next-milestone-useful-raven-subset).
The type migration and actual execution of Raven binaries remain unimplemented; Raven
itself was unchanged by this slice.

## 38. Raven must consume the runtime library — recorded 2026-09-12

**Author clarification:** The goal is staged support for Raven compiling to neoCLR-compatible
IL and consuming neoCLR's runtime class library to build simple programs.

**Assistant response:** Made the end-to-end path the next acceptance test: compile Raven,
import a bounded emitted program, and execute its Console call against the real neoCLR
System implementation. Proposed a neoCLR-owned experiment bridge, with no further Raven
changes, while the native reader's metadata binding remains incomplete.

**Action so far:** Completed nominal class reference defaults. The subsequent bridge now imports the four static Raven programs and all four verify/run
on neoCLR, with HelloWorld using the real System.Console. The [workflow and recorded
results](experiments/raven-target/README.md#first-runtime-library-execution-milestone)
distinguish this tooling path from direct native PE loading.

## 39. VS Code as a target development environment — recorded 2026-09-12

**Author directive:** Have the development experience for targeting neoCLR in VS Code.

**Assistant response:** Proposed staged editing/build/run support for Raven targeting
neoCLR, then diagnostics and source-mapped debugging. Kept host .NET debugging distinct
from integration with neoCLR's own execution/debugger.

**Author correction:** Full debugging is not the MVP. The minimum is code completion in
Raven files/projects targeting neoCLR; focus on target support first.

**Assistant response/action:** Corrected the [experiment plan](raven-target-experiment.md#vs-code-development-experience)
to require completion against the actual target library, with compiler/project and language
server settings agreeing. Debugging is optional later work. The command-line Raven/library
path works; target-aware completion remains planned.

## 40. Unions belong in the MVP — recorded 2026-09-12

**Author directive:** Demonstrate union usage in the MVP as well.

**Assistant proposal:** Use a Raven program consuming a real neoCLR Result/Option API,
with pattern matching for both outcomes. Proposed Math.Abs(Int32) to show successful
absolute value and overflow, preserving the runtime's Result-based contract.

**Action/outcome:** Added this to the [MVP acceptance criteria](raven-target-experiment.md#mvp-acceptance-criteria-and-union-directive-2026-09-12).
The current slice expands real Math/Console consumption and demonstrates completion through
Raven's compiler API using the same target declarations. Generic union import and the
union demonstration remain the next support work; neither is claimed complete.

### Follow-up evidence — recorded 2026-09-12

**Assistant investigation and action.** Following the author's continuation, added a
Raven sample consuming Math.Abs as Result and matching its success/error case types.
Binding against isolated declarations succeeds, but emission fails while resolving
Result as a host runtime type. No union program has executed. The assistant proposes
fixing target metadata emission before expanding the neoCLR importer; this is a
technical proposal, not an additional author decision. Raven remains unchanged in
this slice. The [probe and next steps](raven-target-experiment.md#union-metadata-probe-and-emission-blocker-2026-09-12)
record the evidence and distinguish the remaining compiler, library, and runtime work.

### Emission follow-up — recorded 2026-09-12

**Assistant action and corrected finding.** On the next continuation, the assistant
changed Raven on its existing feature branch to preserve target metadata types during
retargeted emission and correct generic/byref member references. The sample emits and
passes the dependency audit. Inspection also showed that the earlier declaration
lacked the Value property needed for Raven to recognize a union: its binding success
alone had not established union matching. The probe now includes that recognition
contract and checks for both case extractors. No runtime union execution is claimed.
The [follow-up](raven-target-experiment.md#target-metadata-emission-follow-up-2026-09-12)
records this correction and the remaining API-projection and importer work. No new
author design decision is inferred from continuation.

## 41. A concrete POC demonstration — recorded 2026-09-12

**Author direction.** The author wants a demo of a .NET-like runtime with its own
runtime class library, Result and Option rather than exception/null-centered flows,
Void as a valid type parameter, and Raven targeting with basic VS Code completion.
The author identifies this combination as sufficient to distinguish and justify the
experience; this does not request broad compatibility or a full debugger.

**Assistant response and action.** The assistant acknowledged these as the POC
priorities. During the current Result slice, the first Raven Result sample executed
on neoCLR against the real Math and Result implementations, printing 42 and Overflow.
The assistant added control-flow/initialization validation, malformed-input probes,
run instructions and the executable regression artifact. Option, generic Void from
Raven, and actual VS Code integration remain unfinished; compiler completion alone
is not reported as editor completion. The [POC priorities](raven-target-experiment.md#poc-demonstration-priorities-2026-09-12)
distinguish the author's criteria from the assistant's proposed next-step order.

## 42. A locally testable VS Code integration — recorded 2026-09-12

**Author direction.** During the editor slice, the author requested instructions for
actually testing the integration in VS Code, observing that no available build yet
supported it, and suggested a local updated SDK and VS Code extension build. The author
then clarified the term “VS Code extension.”

**Assistant response and action.** The assistant added project-level explicit metadata
configuration in Raven on its isolated branch, prepared an editable Result demo, and
built and installed a local VSIX containing the updated compiler library/language server.
Headless LSP checks covered Math, Console and System names; a manual VS Code interaction
visibly returned Abs, Max, Min and Sign. The temporary UI test edits were discarded. A separate SDK bundle was built and
installed alongside the existing SDK, with its version command checked.
The assistant documented the setup and the distinction between editor support and the
separate runtime import pipeline in the [walkthrough](experiments/raven-target/VSCODE.md).

**Outcome and remaining work.** Target-aware completion now works in the installed
extension. This does not make the regular Raven Build/Run buttons neoCLR commands.
Connecting saved project edits to compilation/import/execution remains the next slice.
The local build is experimental and is not a published release.

## 43. Finish the development loop, then propagation — recorded 2026-09-12

**Author direction.** While the assistant connected saved projects to runtime execution,
the author requested union propagation, then suggested it could be the next milestone.
The author subsequently directed completion of this milestone, leaving interfaces and
related capabilities for later.

**Assistant response and action.** The assistant kept the current milestone focused on
saved-source build/run tasks and recorded propagation as next. Compilation exposed an
inferred .NET framework attribute; a small Raven project-system fix removed that inference
for explicit-core targets. The new bridge reuses the editor project model and existing
runtime admission, with isolated outputs for each build.

**Outcome.** Result, Option and generic Void execute from project sources. A saved edit
changes the output, while compiler/import failures stop without running stale artifacts.
Propagation and interfaces remain planned, not reported as implemented. The dedicated
VS Code tasks still differ from Raven's normal toolbar pipeline.

## 44. Close the POC and establish interfaces first — recorded 2026-09-12

**Author direction.** The author stated that union propagation can wait until the
interface contract exists, asked what should come next, and proposed closing and tagging
this milestone. This revises the ordering recorded in entry 43; it does not erase it.

**Assistant response and action.** The assistant assessed the bounded POC as complete,
recorded its evidence and remaining limits, and chose a local annotated experiment tag,
`milestone/raven-poc-2026-09-12`, pinning the Raven dependency. This does not publish a
release or push either repository. The next proposed milestone is a documented interface
contract followed by a concrete class/interface dispatch example from Raven on neoCLR.
Union propagation is explicitly deferred; the detailed interface scope remains to be
validated against CLI metadata, runtime behavior and the end-to-end example.

## 45. The existing runtime library is the desired demo — recorded 2026-09-12

**Author direction.** After closing the bounded POC, the author said the eventual goal
is to run the same, adapted runtime library surface previously used for Neo, then
clarified: that is the desired demo.

**Assistant response.** The assistant distinguished the completed integration checkpoint
from this broader goal. The existing neoCLR library remains the implementation foundation;
Raven's declaration assembly is its compiler-facing view. The assistant proposed using
an existing collection/iteration contract for the interface milestone so it advances
that demonstration rather than only exercising a disconnected interface example.

**Action and open scope.** Recorded the shared-library direction in the runtime library,
experiment and roadmap documents. The collection slice is a proposal; its exact API
adaptations and runtime/import requirements remain to be assessed. No new library or
interface implementation is claimed by this documentation change. The prior POC tag
remains unchanged, and propagation remains deferred until the interface contract exists.

## 46. Type categories, including arrays, replace the old default — recorded 2026-09-12

**Author correction.** “We intend to go away from the value by default and the explicit
references, so we align with the .NET semantics around the categories of types. And
that also applies to arrays.” This clarifies the earlier migration direction after the
assistant repeatedly preserved Neo's original array/value model in runtime slices.

**Assistant response.** Acknowledged that .NET-style type categories are the intended
ordinary model. The old value-by-default and explicit-reference projection is transitional,
not a second public model to preserve. Normal class/interface/array references remain
distinct from managed byrefs to slots. The recent arrayref/array.new split is migration
machinery rather than the desired final user experience.

**Action and outcome.** Updated the roadmap, class-semantics and managed-array documents
to distinguish the target from current implementation. The implementation has not been
migrated by this documentation change. Upcoming slices need to migrate Neo/library
classifications and array spelling/lowering alongside the Raven collection work. Earlier
positions and implementation descriptions remain available; separate stack-buffer
facilities are not implied to be ordinary arrays.

## 47. .NET is the baseline; faults are not exception classes — recorded 2026-09-12

**Author direction.** The author said to stay with .NET semantics unless an improvement
is found. They accepted Result-based class-library error flow, while clarifying that
exception-like terminal failures may mean faults: “I don't however think we should model
Exceptions as classes.” The corrected message uses exception semantics, not Result
semantics, for that fault distinction.

**Assistant response.** Proposed expected recoverable failures as Result data and
terminal runtime faults reported at the host boundary with diagnostics. No guest
Exception hierarchy is required. The assistant emphasized completing the type-model
migration rather than extending the old defaults.

**Action and outcome.** Inspected `src/lib.rs`: Fault is already a host-side Rust record
with diagnostic fields, documented as uncatchable by guest code. Updated the API policy,
error policy, roadmap and experiment documents. Corrected stale text that still called
the selected type model an undecided investigation, and an old roadmap instruction to
insert dummy Void values for CLI calls. This commit changes documentation only. Cleanup,
containment and future async fault boundaries remain open; no catchable guest exception
mechanism or runtime migration is claimed by this action.

## 48. Keep Neo outside the CLR/Raven experiment — recorded 2026-09-12

**Author correction.** After the assistant began changing Neo array lowering, the
author said “leave neo out of this” and clarified: “We work with neoCLR and Raven
as we adapt neoCLR to get those CLR semantics that are familiar to developers.
Neo doesn't need to be updated here”. This narrows the scope of the migration in
entries 46–47; it does not reverse the runtime type-category direction.

**Assistant response and action.** Acknowledged the correction and removed the
uncommitted Neo frontend changes and the runtime array-construction helper instructions
introduced solely for them. Updated the roadmap and active design documents to exclude
Neo migration from this experiment. Existing committed neoCLR ordinary array/class
reference support and Raven array import remain intact. No Raven files were changed.

**Next work.** Adapt the actual collection/iterator runtime library and admit its
class/interface calls through the Raven bridge, with runtime and Raven acceptance tests.
That implementation remains pending; this scope correction does not complete it.

## 49. Configure Raven's library contracts by target — recorded 2026-09-12

**Author direction.** The author asked for a way to specify classes and contracts whose
names differ in neoCLR, including interfaces without the I prefix and iteration APIs.
Raven must keep targeting .NET while also supporting the experimental neoCLR target.

**Assistant response.** Proposed a target contract description for compiler-recognized
roles, including types, members and signatures. Distinguished Iterable (source) from
Iterator (cursor), and ordinary API name binding from compiler-driven iteration lowering.
The assistant proposed validating actual target symbols rather than global name replacement.

**Action and outcome.** Recorded the [target-contract proposal](raven-target-contracts.md)
and inspected Raven's existing core metadata options and explicit .NET iteration names.
No Raven changes were made. The concurrently implemented runtime collection profile
adapts existing algorithms and passes runtime tests; it does not yet enable Raven's
collection import or implicit iteration. Compiler configuration, target diagnostics and
.NET/neoCLR editor regression tests remain planned on Raven's feature branch.

## 50. Target iteration lowering and the cleanup gap — recorded 2026-09-12

**Author direction.** The author clarified that when Raven targets neoCLR, foreach-style
lowering should use Iterable and that target's contract. They asked to note the missing
Dispose call and consider fixing it. They subsequently proposed a finally-like alternative,
possibly defer in Raven, given neoCLR's absence of recoverable exception handling.
They concluded: “That should be noted for now”.

**Assistant response.** Agreed to select actual target interfaces and method signatures
rather than only rename emitted references. Reported that Raven's existing iterator-loop
emitter does not automatically dispose on its default .NET path either. Proposed treating
that as a separate cleanup issue. Distinguished a possible defer syntax from runtime
support for cleanup, and suggested evaluating finally-only CLI regions without guest
Exception classes. These cleanup choices are proposals, not author-approved implementations.

**Action and outcome.** Implemented configurable synchronous iteration in Raven's feature
branch (`c28657859dda6b473d182445d91e5d44469a6974`), with 41 focused tests passing.
The neoCLR probe selects Iterable/Iterator and executes a for-loop sample covering normal
completion, break and return. Recorded [cleanup/defer discussion](raven-target-contracts.md#recorded-cleanup-discussion-no-implementation-yet)
for future work as requested; neither automatic disposal nor defer was implemented.
Project configuration and VS Code integration of this option remain pending.

## 51. Scoped cleanup and the target boundary — recorded 2026-09-12

**Author direction.** The author proposed deterministic destructors using Disposable,
Closable or an iteration of those contracts, then clarified that the intended feature
is deterministic resource cleanup, separate from deconstruction. They explicitly directed
that possible NeoCLR-specific behavior must leave existing CLR support working the same.
Future NeoCLR behavior may differ; that is not a direction to change the .NET target.

**Assistant proposal.** Distinguish cleanup responsibility owned by a scope from an
object's memory lifetime: multiple reference aliases must not each trigger disposal
when leaving scope. Disposable/Closable are candidate hooks. Cleanup failure policy,
including Result-returning close operations, remains to be designed. These are proposals,
not implemented destruction or ownership semantics.

**Action and outcome.** Recorded the boundary in both target-contract documents. Kept
cleanup/defer out of the current slice, following the earlier direction to note it only.
The separate project/editor integration now selects iteration through evaluated project
properties, runs the adapted collection library and verifies completion and inferred
loop-binding types through the language server. Default project settings remain unchanged.
See [target integration and cleanup boundary](raven-target-contracts.md#project-and-editor-integration-2026-09-12).

## 52. Fundamental demonstration, with Raven isolated — recorded 2026-09-12

**Author direction.** Full integration is not necessary. Demonstrate fundamental things
already shown with Neo: .NET-aligned value versus reference categories and Raven adapting
to neoCLR's runtime library and type names. Evaluate changes to Raven separately as the
experiment advances and differences become clearer. Keep Raven on a “Separate branch,
as today”.

**Assistant proposal and action.** Proposed combining the existing collection and union
profiles in one small program, leaving propagation and cleanup deferred. Kept Raven on
`codex/neoclr-target-resolution`; this slice changes no Raven files. Added a product
workflow using List/ArrayList, Option, Result and Option<Void>, and paired checks for
class/array aliases and value-carrier copying. Recorded the bounded scope and separate
future evaluation of Raven changes; no merge or general Raven adoption is implied.

**Outcome.** The combined probe compiles, imports and executes the success, overflow
and missing-product paths on neoCLR. The project/editor verification evidence is linked
from the [fundamental demo](experiments/raven-target/README.md#fundamental-raven-demonstration).
These examples establish a useful subset, not full migration or arbitrary type import.

## 53. Trying the tools locally before a public bundle — recorded 2026-09-12

**Author direction.** Asked to update the SDK and VS Code extension so they could try
the experiment, and asked whether more work was needed first. Added that, when ready,
the project should produce a build other people can try themselves.

**Assistant response.** Proposed testing the current bounded demo now, using a local
experimental SDK/extension and a prepared workspace. Proposed a later downloadable bundle
with pinned neoCLR/Raven tools, the extension, demo and a clear build/run path. Local
feedback should expose usability problems before distributing it more broadly.

**Subsequent direction.** The author proposed merging neoCLR work into neoCLR main,
then updating documentation and producing a public build/release. They also suggested
publishing matching Raven tools with NeoCLR. The assistant agreed that the tools should
be pinned experimental SDK/extension artifacts, without merging Raven's branch or
presenting them as a general Raven release.

**Propagation follow-up.** The author proposed adding propagation before the public
preview for a complete NeoCLR union story, and specified that its interface belongs in
the NeoCLR runtime library. The assistant proposed completing the local installation
first, then validating Raven against that runtime-owned contract while preserving its
.NET behavior. This is the next feature slice, not functionality in the current build.

**Release verification.** The author also requested checking match expression and
statement syntax and documenting what works and does not work for the release. The
assistant agreed to require compilation and execution evidence, not just parsing or
completion. The pre-merge full neoCLR suite subsequently found a generic Clonable
assembly-resolution failure; merge readiness remains pending that regression.

**Scope.** Public distribution remains a future goal, not a release announced or
published by this exchange. Raven remains isolated on its experiment branch. See the
[local installation instructions](experiments/raven-target/VSCODE.md) for installation
status and the existing bounded-target limitations.

## 54. Normal output and the release API boundary — recorded 2026-09-12

**Author direction.** Asked whether `=> Void` was test output and agreed that it should
not appear during normal execution. Requested propagation and APIs for reading/writing
text files for a complete release; reflection APIs would also be nice to have.

**Assistant action and scope.** Removed the default runner return-value suffix and
introduced opt-in `--show-result` diagnostics on stderr. Updated CLI/demo expectations.
Recorded propagation and text-file APIs as required Raven-target release coverage and
reflection as desirable. Existing Neo/runtime implementations are not evidence that
Raven can already consume all those APIs; target admission and end-to-end checks remain
necessary. This output slice does not implement those later library features.

## 55. Demonstrate the foundations already built — recorded 2026-09-12

**Author clarification.** The goal is to demonstrate the fundamentals of the new
platform, its differences from .NET and a strong foundation. A complete foundation is
not required; the building blocks already present are important for showing what the
platform is about.

**Assistant response.** Kept the preview tied to the bounded demo: familiar type/IL
behavior, the target library, Result/Option with propagation, basic file APIs and Raven
tooling. Regression repair supports reliability of existing capabilities; it is not a
reason to expand toward a complete platform or runtime library before the preview.

**Date/time addition to the demo.** The author asked to include the date/time libraries
as relatable evidence of the platform, even if incomplete. The assistant proposed
exposing the existing bounded date/time and local-clock surface through Raven, with
formatting/globalization still outside scope. This is release-demo scope, not a claim
that Raven import already supports those APIs.

**Acceptance emphasis.** The author emphasized that the fundamentals, error handling,
and basic VS Code completion through Raven integration are key. The assistant agreed
to treat their combined end-to-end demonstration as acceptance criteria rather than
feature count or completeness of the entire platform.

## 56. First propagation library building block — recorded 2026-09-12

**Prior author direction.** Propagation should be part of the public preview, and its
interface belongs to the NeoCLR runtime library. Raven remains an isolated experiment.

**Assistant proposal.** Inspection found that Raven's protocol includes two extraction
methods and a static residual factory, while neoCLR supports only instance interface
members. Proposed a prefix-free Propagatable extraction interface and concrete carrier
factories, explicitly documenting that this does not yet enforce a static abstract
interface contract. This is a provisional implementation choice, not an author decision
to permanently omit static interface support.

**Action and outcome.** Added the interface and Result/Option methods, including Void
absence and conditional initialization. Four new tests and related runtime suites pass;
the existing Raven demo still executes. The [contract and limitations](propagation-contract.md)
record the .NET comparison and next Raven projection/import work. No `?` support or
Raven changes are claimed by this first slice.

**Subsequent implementation outcome (same date).** The assistant added a target-selected
Raven protocol and a bounded Result<Int32,OverflowError> `?` program. Execution verifies
success continuation and early residual return. Inspection found implicit exception
capture in the default Raven lowering; the opt-in target uses result-only lowering,
while existing .NET propagation tests still pass. Import checks reject malformed
contracts and unsupported thrown values. Option/Void projection and refreshed tools
remain open; continuation was not treated as approval of a permanent protocol design.

## 57. Familiar ArrayList construction — recorded 2026-09-12

**Author direction.** Put desired initial capacity in the constructor and provide a
parameterless constructor, resembling .NET List. The assistant proposed retaining
Allocate for compatibility; the author corrected this: “No need to keep compat”.

**Action.** Replaced Allocate with constructors and migrated active examples/tests.
Default construction starts empty with zero capacity; explicit capacity reserves storage.
The Raven projection uses ordinary class construction. Historical published examples
retain their recorded API. Validation includes capacity, growth and negative arguments.

## 58. Void as a type versus a return marker — recorded 2026-09-12

**Author question and correction.** Asked whether Void should be neoCLR's unit type,
then rejected Option<Void> as the motivating example: None already handles absence,
whereas Result<Void, MyError> expresses completion or failure. Void can remain a
technically valid type argument. Asked whether methods should always return a unit
value or preserve CLR void behavior.

**Assistant proposal and author decision.** Proposed unit semantics for Void in the
type system, preserving the CLR no-result stack convention for void-returning calls,
with compiler adaptation where a value is needed. The author replied: “Agree then.”

**Action and outcome.** Documented the [chosen split](void-semantics.md). The bounded
Raven bridge now executes Option<int> absence propagation and Result<Void,OverflowError>
completion/error propagation. The compiler fix prevents an unused propagated Void
value remaining on the stack. Refreshing the local SDK/extension was explicitly
requested. Installed local SDK/extension 0.1.12-neoclr.4 and verified target completion,
Option output inference and the saved combined workflow. The [setup instructions](experiments/raven-target/VSCODE.md)
identify the fresh demo and remaining preview limits. No public release was made.

## 2026-09-12 — Existing runtime APIs define the Raven POC scope

**Author's direction.** “We want all libraries that we implemented when experimenting
with Neo to be available in the POC release,” clarified as “all runtime APIs.” The
author then described Raven targeting neoCLR as a good starting point even without
the CLR's full feature support.

**Assistant response and action.** The assistant distinguished runtime-library API
coverage from complete CLR/compiler feature parity, and proposed finishing the file
projection, then auditing the existing runtime against Raven's declarations and
importer. The preview acceptance criteria now require that inventory and closure of
its API gaps. The [file projection](raven-file-api.md) subsequently passed compilation,
import, runtime execution and rejection checks (`e2d3e94`); full library coverage is
not yet achieved. Added a [regenerable source inventory and coverage plan](raven-runtime-api-coverage.md)
for the remaining API work. Neo frontend changes are not part of this migration, and Raven remains an experiment
on its separate branch.

**Open work.** Make each existing public API usable through the Raven target and test
representative behavior, while documenting language/runtime feature limitations and
any contract adaptation. This direction expands the release gate; it does not claim
that today's bounded importer already exposes the library.

## 2026-09-12 — A separate, lightweight Raven distribution

**Author's direction.** The special Raven build shipped with neoCLR should not go
through Raven's full release cycle. Test that it works with neoCLR, package it
separately, and document a repeatable procedure in this repository for later releases.

**Assistant response and action.** The assistant agreed to pin the compiler revision,
package SDK/VSIX assets separately, and validate the neoCLR integration. Added an
[experimental release procedure](experiments/raven-target/RELEASING.md) covering
focused tests, packaged-build smoke checks, provenance and repeat releases. It records
the remaining source-bridge distribution gap explicitly. No package or public release
has been produced as part of documenting this procedure.

## 2026-09-13 — Visible rough edges are part of the POC

**Author's observation.** People evaluating the runtime library will see rough edges:
some APIs resemble .NET, some differ, and some are unfinished or still to be decided.
The author explicitly described that visibility as positive: “That is a good thing.”

**Assistant response.** The assistant agreed that the POC should make the distinction
between familiar behavior, deliberate differences and provisional/missing work visible
so people can evaluate the experiment. It proposed keeping implementation and test
evidence clear while recording this framing.

**Author's clarification.** The author hopes evaluators will give feedback on the
APIs. neoCLR can design its own contracts while retaining familiarity and some code
compatibility. Result-based error handling is an expected compatibility break; the
underlying platform still follows similar principles to .NET.

**Assistant response.** The assistant agreed to distinguish familiar platform
principles from exact API compatibility, and to present provisional contracts as
candidates for feedback rather than settled designs.

**Action and outcome.** Added this framing to the [preview introduction](raven-preview-acceptance.md)
and [API design policy](api-design.md). The runtime-library coverage plan continues
to identify actual gaps, including error-case APIs. The current String slicing work
has passed runtime/propagation checks; this framing does not describe unsettled APIs
as complete or stable.

## 2026-09-13 — Complete the existing runtime API pass before stopping

**Author direction.** “I mean don't stop work until Runtime API is done.” This
clarified the completion boundary: the APIs already developed for neoCLR must be
usable in the Raven proof of concept, following the earlier direction to leave the
Neo frontend out and keep Raven changes on its experimental branch.

**Assistant actions and outcome.** Completed the consecutive API projection and
validation slices, including reflection, native buffers, interfaces, arrays and
nested collection/union payloads. The assistant used CLR-style boxing for copied
value-interface views, documenting its allocation and identity costs; direct value
operations remain available. Internal reserved array capacity tracks unreadable
slots instead of inventing default union cases. These implementation choices are
assistant decisions, not additional author quotations or inferred approvals.

Raven regressions uncovered by the integration included generic metadata fields
and pointers, unsigned array reads, member resolution and value-property setters.
Fixes and focused tests remain on `codex/neoclr-target-resolution`. The
[source-by-source API audit](raven-runtime-api-coverage.md) records the projection
and remaining importer limits. Nine validation suites passed using the packaged
bridge outside both development checkouts. SDK/VSIX `0.1.12-neoclr.6` were installed
locally; the installed server passed protocol checks and the saved demo ran.
[Build provenance](experiments/raven-target/local-toolchain.json) records exact
source revisions, hashes and scope; [VS Code instructions](experiments/raven-target/VSCODE.md)
provide the runnable entry point. No public release was made. Full compiler parity,
general application-type admission and fault-unwind cleanup remain outside this
completed existing-library pass.

## 2026-09-13 — Present the runtime and Raven as the release

**Author direction.** The author specified a runtime preview with neoIL samples,
Raven samples and the special Raven distribution that can target neoCLR, accompanied
by release documentation. The old Neo language material should be separated from the
main experiment. The purpose is to show “that we're actually using a real CLR language
and adapting it for our new CLR-like runtime” and demonstrate the platform changes
working in Raven.

**Assistant response and action.** Organized the release guide around two entry
points: direct runtime execution using neoIL, and Raven compiled against the adapted
runtime library. Added executable neoIL examples for class aliasing/value copying
and Result<Void, Error>, with a repeatable verifier/output check and bundle inclusion.
Marked the Neo guide and plan as historical, retaining their implementation and prior
published notes. The primary README now links the runtime/Raven walkthrough.

**Outcome and remaining work.** Both new neoIL samples verified and ran against the
validated local adapted library. The source packaging recipe now includes them; this
does not retroactively change the existing .6 archive. A fresh candidate, its complete
asset attribution and package validation, and publication remain release work. No
claim of general Raven/CLR binary compatibility accompanies these demonstrations.

## 2026-09-13 — Keep ordinary void-return spelling familiar

**Author question.** “Why does neoil use ‘noresult’?” The author asked whether `void`
was expected. **Assistant response.** The assistant explained that `noresult` exposed
the prototype distinction between an empty return stack and the first-class Void unit
value, and proposed retaining that distinction internally instead of requiring a new
public return keyword.

**Action.** Added lowercase `void` in neoIL return position for the existing no-result
calling convention and updated the new preview samples. `System.Void` explicitly
names the unit type; uppercase `Void` and `noresult` retain legacy behavior. Tests
cover stack preservation, rejection of a stray unit return and generic unit storage.
The binary CLI boundary still uses its existing VOID marker; no new opcode or binary
signature encoding is introduced. See [Void semantics](void-semantics.md).

## 2026-09-13 — A small website for the project

**Author direction.** Create a small website presenting the project clearly, host it
with GitHub Pages, and trigger build/publication in a separate GitHub Action.

**Assistant action.** Built a static landing page centered on the current runtime and
Raven integration, with code excerpts sourced from executable samples, library scope,
preview boundaries and links to documentation. Earlier Neo work is linked as history.
The page distinguishes published previews from the locally validated Raven candidate.
Desktop/mobile layout and navigation were inspected in a browser. A separate Pages
workflow builds on relevant changes and deploys only from main; pull requests do not
receive deployment permissions. GitHub Pages was enabled with GitHub Actions as the publishing source. The
[build and deployment run](https://github.com/marinasundstrom/neoCLR/actions/runs/34745892835)
succeeded for `edae382`; the [live site](https://marinasundstrom.github.io/neoCLR/)
was opened and its HTML/CSS/icon bytes verified against the build. Website publication
does not publish or tag the runtime/Raven release candidate.

## 2026-09-13 — Publish the runtime/Raven preview

**Author direction.** Continue with the release, following the earlier request to
ship the runtime and neoIL samples together with Raven examples, the special Raven
distribution and documentation.

**Assistant actions.** Selected Preview 4 and prepared its versioned source commit
`c135659`. CI exposed a stale release-script assumption about the old stdout return
suffix; the check was corrected to request diagnostic results on stderr, retaining
its environment/clock/file assertions. Rebuilt the runtime distribution, tested all
ten package suites, and waited for the six exact-source CI jobs to pass. Published
Raven's experimental branch so the compiler revision is accessible, without changing
Raven main or making a normal Raven/Marketplace release.

**Outcome.** [Preview 4](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.4)
was published with the runtime, SDK, VSIX, source, notices, checksums and validation
evidence. The website now links the release; the local demo uses the released build.
The [validation record](preview-4-validation.md) distinguishes source platform checks
from macOS arm64 binary support and preserves the POC's limitations.

## 2026-09-13 — Rust dependency and alternative implementation languages

**Author questions.** Asked whether C++ or another implementation language would
be better for neoCLR and remove its Rust dependency, then specifically asked what
would be lost without Rust's memory-management model.

**Assistant assessment.** Distinguished the implementation's memory safety from
the guest heap and GC: changing language does not inherently change neoCLR's
managed type or memory contracts. Rust ownership checks protect implementation
lifetimes; C++ RAII and smart pointers provide resource management but do not supply
the same general borrow checking. C++ is a viable runtime implementation choice,
as CoreCLR's native VM illustrates, with more responsibility for lifetime correctness.
Rust does not prove guest GC rooting or unsafe/FFI code correct. A rewrite has no
demonstrated performance benefit here. Provisionally favor keeping Rust unless a
concrete tooling, integration or maintenance need justifies migration; this is an
assistant recommendation, not an author decision.

**Evidence and action.** Inspected Cargo.toml, the execution architecture and native
memory/interop call sites. `otool -L target/release/neoclr` listed only macOS
libSystem, demonstrating no separate dynamically linked Rust library for that local
binary, not all distribution targets. Build-time Rust remains required. Consulted
the [Rust ownership documentation](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html),
[unsafe documentation](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html),
[linkage reference](https://doc.rust-lang.org/reference/linkage.html), and
[CoreCLR VM build sources](https://github.com/dotnet/runtime/blob/main/src/coreclr/vm/CMakeLists.txt)
on 2026-09-13. These are general comparisons against live documentation/main,
not a pinned implementation study or benchmark. The existing
[architecture](execution-architecture.md) already separates semantics from Rust
representations and leaves a native hosting ABI pending. Only this discussion and
its changelog entry were recorded; no implementation migration was performed.
The desired degree of build-tool independence and any migration decision remain open.

## Maintaining the conversation record

Append significant exchanges with the date on which they are recorded. Capture the
author’s question/direction, the assistant’s response or proposal, any subsequent
author decision/correction, the action taken and outcome, and remaining open questions.
Link evidence where available; explicitly say when an outcome is only reported or
unavailable. A plan is not a performed action. Do not require every exchange to contain
all parts; record what actually happened. Distinguish a recorded date from the date of an earlier
remark. Preserve earlier positions when thinking changes, and link a later entry back
to them. Correct factual transcription mistakes transparently rather than inventing
a cleaner history.

Attribute the author's statements separately from assistant proposals, implementation
outcomes and editorial interpretation. Use direct quotations only when the original
wording is available. Do not infer approval from silence, turn a possibility into a
commitment, or log every routine continuation as a new design decision.

The changelog records changes to the project; this document records the conversations
that direct it. An implementation result is not a substitute for the preceding question,
nor evidence of author approval beyond what was actually said. Only conversation actually available to the
maintainer can be recorded—this is not an automatic capture of conversations elsewhere.


## 2026-09-13 — Application programs and Raven targeting findings

**Author direction.** After Preview 4, the author asked what should follow and
approved three proposed slices: application-defined types, interface dispatch, and
an order-workflow application using the runtime library. The author then emphasized
that integration exposes Raven bugs and should help stabilize the compiler for other
frameworks, citing the existing nanoFramework adaptation as prior experience. The
primary purpose remains making neoCLR work with Raven.

**Assistant response and action.** The assistant distinguished compiler correctness
from configurable target contracts (Iterable/Iterator) and deliberate neoCLR differences
(generic Void, Result). The first application class/value sample now executes through
the bridge, including ArrayList storage. Raven metadata emission exposed a mixed
MetadataLoadContext/TypeBuilder generic signature failure and an incorrect external
self-assembly reference; fixes and focused tests remain on the experimental branch.
See [application types and findings](raven-application-types.md).

**Author clarification.** Generic Void also matters for a future Task<Void> and
async/await story. The assistant recorded that as a design requirement, without
claiming async implementation or choosing a task contract. Interface and workflow
slices remain in progress at this entry; the published Preview 4 artifacts are unchanged.


**Further direction in the same exchange.** The author added delegates and lambdas
to the support goals, and required debugging support for a future release. The
assistant placed delegate/lambda work after the class/interface foundation and
recorded a future Raven debugging milestone in the [roadmap](roadmap.md). The
interface/inheritance sample now executes, revealing another Raven compiler bug:
implicit interface implementation overrode declared abstract/virtual method flags.
The fix remains on the experimental branch. These findings reinforce the author's
distinction between compiler stabilization, target-specific contract mappings and
intentional runtime changes; they do not turn this into a general Raven release.


**Outcome of the initial three slices.** Application types and interface/inheritance
support are committed separately. The [order workflow](raven-order-workflow.md) now
executes with OrderStore dispatch, ArrayList<Order>, Option<Order>, Result<Void,
FileWriteError> propagation and file I/O. Isolated checks confirm that failed writes
preserve both queued order state and the previous report. Delegate/lambda work remains
in progress; debugging remains a future release requirement.


**Delegate/lambda outcome.** The requested follow-up now executes instance method
groups and Raven-generated closure objects, including mutable shared captures and a
callback returned from its creating function. A further compiler issue was found:
interface/abstract method groups used ldftn, while explicit base groups could use
virtual binding. The experimental Raven correction preserves ordinary CLR behavior;
its tests include actual invocation and binding-time null failure. The neoCLR bridge
uses the existing Func family and documents its adapters and limits in
[delegate/lambda support](raven-delegates-lambdas.md). No async or debugger implementation
is implied by this result, and published Preview 4 tools are unchanged.


### 2026-09-13 — Modern library APIs and a replaceable clock

**Author direction.** While the application toolchain was being packaged, the author
asked that future runtime-library design feel modern, including base types and
namespace-level functions, without carrying legacy APIs forward merely for familiarity.
Date/time might instead center on a clock implementation that can be mocked.

**Assistant response.** Proposed separating date/time values from time acquisition,
with an injected system or controlled test clock. Compared the idea with the shipped
.NET TimeProvider/FakeTimeProvider baseline, rather than claiming .NET lacks it.

**Action and outcome.** Recorded the policy in [API design](api-design.md), with
alternatives, placement and future validation in [clock design](local-clock.md#future-clock-contract-review-2026-09-13)
and a backlog entry. No clock redesign was implemented; the current toolchain refresh
continues. Exact contracts and namespace-function projection remain open.


### 2026-09-13 — Pattern ergonomics, extension methods and prototype LINQ

**Author direction.** During the Int32 completion investigation, the author asked for
readable union patterns using case imports and target-member binding. The example
matched File.ReadAllText with `Ok(let text)` and `Error(let error)`, and the author
clarified: “That means supporting destructuring.” Preliminary pattern support was
identified as important for promoting neoCLR.

**Assistant proposal and action.** Prioritized the existing constructor-completion
bug first, then investigated target union metadata, payload deconstruction and generic
case emission. The completion fix was committed on Raven at 55c0f7ef5 and selected in
the local demo. Pattern work remains in progress at this entry; do not infer delivered
support from the proposed syntax alone.

**Further direction.** The author observed that Raven features such as extension
methods can work through existing runtime constructs and library contracts, then
requested prototype LINQ for the next release. The assistant proposed validating
extension methods after patterns and starting with Where/Select on Iterable/Iterator.
The roadmap/backlog record that proposal and unresolved laziness/cleanup semantics;
no LINQ implementation is claimed here.


**Pattern slice outcome (2026-09-13).** The source experiment now uses the existing
member-union and Deconstruct contracts, rather than adding Raven named-case attributes
to the target library. The initial attribute approach was discarded after exposing
incompatible case lookup behavior. The assistant corrected generic case inference,
value deconstruction emission, primitive CLI signatures and scope retention in Raven;
these corrections are committed as `04c953d67` on its experimental branch. The neoCLR bridge provides
payload deconstruction adapters and preserves definite assignment across nested
pattern branches. The order workflow now demonstrates the author's imported-case
syntax, including String and application-object payloads. The executable
[pattern matrix](raven-match-matrix.md) records the tested boundary; LINQ and extension
method validation remain upcoming work. Existing archived builds are unchanged.

The assistant also prepared a separate local source-backed pattern demo with an
updated language server. The previous edited demo was preserved. Validation passed
299 Raven tests, 50 saved-project checks, the match matrix, workflow state checks and
payload completion/hover checks; this did not publish or replace the archived SDK.

**Extension slice outcome (2026-09-13).** Following the author's request to explore
extension methods before prototype LINQ, the assistant reproduced missing generated
marker metadata in the target declaration image. Supplying those metadata-only
dependencies lets nongeneric Raven extensions execute through existing static calls;
no compiler change or new opcode was needed. The sample exercises integer receivers,
shared ArrayList references, Iterable receivers and captured callbacks. Completion
also exposes the extension. Generic application extensions remain rejected by the
bridge. The assistant retained deferred Where/Select as the intended next contract
and identified generic query bindings and iterator representation as prerequisites,
not completed query support. See [the implemented boundary](raven-extension-methods.md).
Archived packages and the author's edited demo were left unchanged.


**Query slice outcome (2026-09-13).** Continuing the author's prototype LINQ direction,
the assistant chose generic runtime-library operators with exact bridge bindings,
rather than expanding generic application-body importing. Where/Select remain deferred;
ToList performs explicit materialization. Their implementation uses ordinary classes,
interfaces and delegates, without new opcodes. Tests exercise callback timing, caching,
repeat enumeration, value/reference payloads and GC-held captures. The Void exercise
found a missing method-argument projection in the bridge and that was corrected;
explicit Func<T, System.Void> works while unconstrained Unit inference remains separate.
A custom Raven Iterable/Cursor probe failed during compiler emission and remains open;
the low-level custom iterator fixture verifies acquisition and disposal directly.
[The API contract](raven-query-api.md) records these outcomes and the retained cleanup
gap. Raven and legacy Neo sources and archived builds were not changed in this slice.


**Custom iterator follow-up (2026-09-13).** The assistant reduced the failure to
serialization of generic interface MethodImpl signatures by the persisted metadata
writer. Raven now normalizes those declarations through its existing target-method
reference path; both ordinary CLR and target emission execute the regression test.
A second bridge issue required closing definition-relative interface signatures
before matching application methods. The former failing repro is now a passing
[custom Iterable sample](experiments/raven-target/samples/application-iterable.rvn).
This resolves the previously recorded compiler gap without new runtime behavior.


## 2026-09-13 — array queries need an array contract

The author reported a reflection-array `for` loop rejected by the importer (`bge`)
and then `GetMethods().ToList()` missing from editor lookup. They directed that the
fixes belong in the next release. The assistant first fixed branch admission and
proposed array-specific query overloads backed by a sequence adapter.

The author clarified the intended .NET-like Iterable/Iterator pattern, correcting
an initial reference to ArrayList with: “ArrayList already does that. I mean Array”.
They added: “Anyway, we should make sure that the same pattern exists in neoCLR.”

The assistant changed the implementation to a runtime array-to-Iterable view with
independent library iterators and opt-in Raven target-symbol support, removing the
query-only overload approach. The development query tests cover arrays in interface
locals, parameters and returns, independent positions, mutation visibility and GC;
the runtime tests check reference identity and invalid/null dispatch. See the
[array/query contract](raven-query-api.md). Preview 5 remains frozen; this work is for
a subsequent release, not a claim that the reported programs work in its artifacts.


The author then reiterated the compatibility goal: Raven should need only minimal
changes, such as selecting different target interfaces, while the runtime supplies
the behavior required by ordinary emitted IL. The assistant agreed to keep contract
selection target-neutral and to investigate missing runtime support before adding
NeoCLR-specific compiler lowering. The array implementation follows that split:
Raven projects the configured interface; neoCLR owns reference identity and dispatch.


## 2026-09-13 — Stabilizing ordinary Raven programs

**Author:** “Let's stabilize neoCLR.” The preceding direction was to keep Raven
changes minimal and let neoCLR implement the contracts ordinary CLR features need.

**Assistant proposal and action:** Focus on correctness rather than new features.
A basic array query with `value > 1` reproduced an importer rejection of `cgt`,
although the runtime already implemented it. Admitted the existing numeric comparison
and branch families with operand/control-flow validation. Boundary tests then exposed
Raven's signed emission for unsigned comparisons and unordered floating comparison
errors; corrected those in the existing Raven experiment branch, without adding a
neoCLR-specific compiler workaround.

**Outcome and limits:** The updated query regression suite passes, including integer
boundaries and NaN. The .NET execution test also passes. A wider Raven test run found
an independent mixed `ulong`/`long` binding failure, reproduced with unchanged compiler
code; it remains open. These are next-build fixes, not a new published release or a
claim that the entire platform is stable. See the [comparison contract](raven-primitive-api.md#numeric-comparisons-after-preview-5).

## 2026-09-13 — Mutable array invariance and read-only views

**Author:** Asked to resolve array co-/contravariance in neoCLR. The assistant initially
proposed CLR-style array covariance with checked writes. The author corrected that
proposal: a typed mutable array, including Foo[], should not become an array of a
supertype such as Object[]. The author then identified a possible exception for a
read-only array or a read-only interface, while leaving its mechanism open.

**Author's reasoning:** Described array covariance as a historically useful feature
from before generics, but unsuitable for the intended modern type system. This is the
author's evaluation, not a newly established history of the CLR design.

**Assistant response and action:** Accepted invariant mutable arrays as the direction.
Distinguished a read-only element interface from an immutable binding or readonly
reference to the array slot. Added early cast rejection and regression coverage on
top of existing exact array storage rules. Documented both the compatibility cost
and the future read-only projection in the [array variance contract](array-variance.md).

**Open:** Covariant read-only projection API, generic variance metadata and target-aware
Raven editor diagnostics. No new read-only array feature or generic variance behavior
was implemented in this slice.

## 2026-09-13 — Preserve CLR ergonomics while selecting improvements

**Author's direction:** Define what neoCLR can improve when starting without legacy
constraints, while preserving useful .NET foundations. The value/reference type split
still serves the project and should not be the current redesign focus. Spans and
managed references may warrant more consistent runtime support.

**Author's additions:** Reaffirmed UTF-8 strings and requested modern string, character
and encoding APIs. Asked to revisit nullability and its metadata, while acknowledging
that another solution might not be better. Requested an API scope that can be implemented
incrementally as consumers need it. Emphasized preserving C# and other languages'
ergonomics on the platform.

**Further questions:** Asked whether delegates might become function types, without
choosing that change. Wants runtime support for async/await; its model remains open,
and tasks still seem relevant. Identified the implemented array-covariance change as
something to observe for type-system consistency without losing ergonomics. Reaffirmed
building date/time toward instantiable, mockable environmental APIs.

**Assistant proposals:** Keep familiar type categories and compiler boundaries.
Separate public API shape from runtime representation: language function types need
not replace delegates, and tasks need not dictate suspension internals. Compare .NET's
existing span, Unicode, nullability, delegate, Task and TimeProvider contracts before
claiming an improvement. Develop APIs by concrete scenarios; distinguish representation,
metadata enforcement, compiler diagnostics and library policy. Treat all unselected
representations as research questions.

**Actions and outcome:** Added the [platform direction](platform-direction.md) and
[runtime API plan](runtime-api-plan.md); updated active policy summaries that still
prescribed value-by-default. Extended the text, nullability, delegate and date/time
research notes and linked the roadmap. This slice changes documentation only. No new
Span, Rune/codec, nullable signature, callable representation, Task or injectable-clock
implementation is claimed. Proposed slice ordering remains an assistant recommendation;
no reply selecting those detailed designs is available in this exchange.

**Follow-up on research scope:** The author requested reviews of other platforms,
comments on .NET runtime APIs, and independent projects that seek to improve the
.NET experience. The assistant proposed retaining .NET as the compatibility baseline
while examining concrete feedback, alternative designs and counterevidence. Updated
[design research](design-research.md#broader-api-review-scope-2026-09-13) to require
relevant comparisons for substantive API design, with attributed opinions, source
status and transfer costs. Added Noda Time, Rust text documentation and .NET API-review
process links as initial entry points. No full comparative review or adoption of
those projects is claimed; this establishes the workflow for subsequent API slices.

**Author's qualification:** Not everything warrants change. Larger platform decisions
need input and grounding, while some ideas can be prototyped immediately. The assistant
updated the research policy to scale evidence to consequences, keep prototypes
provisional and include retaining current behavior as an explicit alternative. This
is a decision-quality requirement, not a new permission or external-feedback gate.


## 2026-09-13 — A bounded view before a collection hierarchy

**Author:** Asked to continue the proposed read-only view/variance investigation,
then cautioned that .NET's accumulated collection interfaces should be reviewed later.
Questioned the need for readonly, immutable and frozen contracts. Suggested Iterable,
counted Collection, indexed/ordered Sequence, Map and Set with separate Mutable*
contracts as an illustration, explicitly not a selected design.

**Assistant response and action:** Kept the prototype application-local. Audited the
existing interfaces and found variance metadata/dispatch prerequisites and a legacy
Iterable byref-return mismatch with the Raven projection. Added an explicit Dog-array
adapter returning Animal through a getter-only interface, a C# comparison and direct
IL checks. Reviewed .NET variance and collection discussions, Java backed views and
alternative immutable .NET collections as different contracts.

**Outcome and limits:** The prototype demonstrates live aliases, shared element
identity, private backing storage and GC retention without copying the array. It
allocates an adapter and does not implement implicit covariance. No new standard
collection interface, general variance or readonly/immutable/frozen type was selected.
See the [experiment and validation](experiments/readonly-views/README.md). The future
collection hierarchy and variance representation remain open.

**Subsequent supplied proposal:** The author shared a ChatGPT hierarchy with covariant
Iterable/Collection/List/Set, Map<K,out V>, invariant Mutable* extensions, and concrete
Array/ArrayList/HashSet/HashMap plus Immutable* and Frozen* families; an earlier list
also included LinkedList. Recorded both proposals in the
[collection-contract review](collection-contracts.md). The assistant identified Set
membership inputs, Map's Option/byref lookup results, fixed-size array mutation and
linked-list indexing costs as questions to resolve before selecting that hierarchy.
No author decision adopting these particular interfaces is inferred.

**Explicit array decision:** The author then stated, “System.Array should be generic
in NeoCLR. System.Array<T>”. Recorded this as the selected generic array direction,
while keeping the rest of the hierarchy provisional. The assistant proposed preserving
ordinary T[] syntax and CLI array signatures/instructions, with no wrapper allocation,
and identified type identity, metadata, reflection, member lookup and fixed-size mutation
as mapping work still to perform. This slice does not implement the generic Array API.

**Variance follow-up:** The author observed that generic System.Array allows variance
to be declared on the type. The assistant clarified the proposed rule as invariant T
for mutable arrays, with covariance potentially on read interfaces. A generic declaration
still needs to agree with runtime array signatures, casts and element access; it does
not itself make mutable covariance safe or complete the missing generic-variance work.

**Shape clarification:** The author confirmed unchanged language treatment and
emphasized “The shape is known”. The assistant recorded the purpose as a known generic
array definition for the runtime and tools, linking element type, members, interfaces
and invariance to existing array signatures/instructions. This refines the intended
mapping; it does not claim that mapping has been implemented.

**API priority direction:** The author placed the generic array API, collection
interfaces and basic implementations on the immediate runtime-API priority list.
A complete library is unnecessary, but these fundamental building blocks should at
least have usable prototypes. The assistant updated the API plan and roadmap to put
generic array mapping first, then minimal contracts and alignment of arrays/ArrayList,
followed by bounded keyed collections. The text-first suggestion was superseded.
The detailed slice boundaries are the assistant's proposed execution plan, not an
adoption of every previously supplied collection type. No new API was implemented
in this documentation update.

## 2026-09-13 — Normalize array and native-memory roles with .NET

**Author:** Asked to continue with Array<T>. On learning that the current generic
System.Array is a native descriptor rather than the managed array object, observed
that this question has changed since returning to .NET-style type semantics and
said, “We should normalize with .NET”.

**Assistant finding and response:** Audited the native descriptor, managed ArrayRef
representation, reflection and Raven bridge. Initially proposed moving the native
API to an explicit interop name, then clarified that renaming alone would leave
the descriptor's mixed ownership and borrowing responsibilities intact. Proposed
the .NET separation of managed arrays, explicit native allocation/release, borrowed
memory views and distinct ownership when needed. Recorded the comparison and
migration order in [collection contracts](collection-contracts.md#managed-arrays-versus-native-buffers).

**Outcome and open work:** The ownership direction is recorded; the Array<T> mapping
and native API migration are not yet implemented. The prior generic-array identity
and invariance decisions are retained, rather than interpreting the latest direction
as an unspoken reversal. Exact native replacement APIs, safe borrowed views and
their lifetime enforcement remain implementation/design work. No new standard
NativeBuffer type was adopted.

## 2026-09-13 — Implement the array shape without a native compatibility alias

**Author:** Asked to continue, then clarified that compatibility with old neoCLR
iterations is unnecessary. Asked whether Array<T> should implement interfaces.
Explained that direct array loops should remain indexed, while interface support
allows arrays to be passed as iterables and used with extension methods.

**Assistant response and actions:** Implemented a generic managed array shape over
the existing array identity in the Raven profile. Initially staged a temporary
LegacyNativeArray rename; after the author's correction, removed that path and
replaced the native descriptor API with a bounded NativeMemory allocation/free
API. No legacy alias was committed. Added an explicit Iterable<T> declaration,
kept the current growable List contract off fixed-size arrays, and added a sample
covering indexed loops, an Iterable parameter and the ToList extension.

**Compiler finding:** Native method emission exposed a missing pointer case in
Raven's metadata-reference rewriter. Fixed it with a metadata regression on the
existing Raven experiment branch. Native-sized numeric and explicit pointer
conversions remain separate compiler gaps. The target adapter also accounts for
Raven's empty Unit pointer locals when importing native void-pointer calls.

**Outcome:** See [generic managed arrays](generic-managed-arrays.md) for implemented
behavior, migration and validation. The source profile changes do not refresh the
installed SDK/extension. The broader collection hierarchy, read interfaces, Span
and native owner contracts remain open. Historical Neo work remains outside this
Raven-profile migration, as previously directed.

**Consistency observation:** The author described the generic array signature as
a more consistent solution for neoCLR while retaining .NET familiarity, then
corrected a spoken project-name transcription to neoCLR. The assistant agreed
about a shared generic definition for members/interfaces/reflection, while noting
that ordinary CLI array signatures already encode their element type. This is a
regularization of the array contract, not a claim that CLR arrays were untyped.


### 2026-09-13 — Reading array contracts from target metadata

**Author direction:** Continued the Array<T> work after emphasizing that arrays
must participate in Iterable and extension methods while direct loops remain indexed.

**Assistant proposal and action:** Replaced the experiment's Boolean array-interface
projection with an optional generic shape mapping in Raven. The core reference now
declares System.Array<T> implementing Iterable<T>; Raven substitutes the element type
and reads the declared interface closure. The runtime support came from the preceding
slice. This keeps future interface additions primarily in target metadata rather than
adding a compiler rule for each contract. The normal .NET target remains unchanged.

**Boundary:** This does not adopt a broader collection hierarchy or make explicit
System.Array<T> source annotations interchangeable with T[]. Installed tools are not
refreshed by this source slice.

**Validation outcome:** The metadata projection passes 25 focused Raven tests,
58 saved-project cases and 28 query checks, including array extensions. The interface
probe checks the reference declaration. These are source-level integration checks;
a packaged SDK/editor refresh remains separate.


### 2026-09-13 — Unifying the array spellings and refreshing the tools

**Author direction:** Accepted the proposed consecutive slices: make Array<T> and
T[] interchangeable in Raven, verify the full array experience, then refresh the
local SDK and VS Code extension.

**Assistant action:** Implemented target-specific canonicalization in source type
binding and imported signatures. Both spellings use ordinary CLI vector signatures;
interface member calls use normal interface dispatch. The new sample demonstrates
alias mutation, nested arrays, indexed loops, Iterable parameters, GetIterator,
query extensions and reflection. Compiler regressions cover opt-in behavior and
invariant arrays. Allocation remains an array expression rather than a new generic
class constructor. Broader collection taxonomy and arbitrary class-member projection
remain separate. Local packaging/installation is the following slice.

**Source validation:** The new sample passed on neoCLR; the saved-project suite
passed 59 cases, and the new SDK server passed 52 completion/hover/diagnostic checks.
Raven's compiler slice is committed as `6072dcf4b` on its experiment branch.

**Local tool outcome:** Built and installed SDK/VSIX `0.1.12-neoclr.11` in a new
isolated profile with the unified-array demo. Seven packaged suites passed; the
installed VSIX server and exact configured Run task also passed. Preserved 1,074
existing Raven source files. [Local instructions](raven-generic-arrays-local-build.md)
and the linked manifest record artifact hashes and evidence. This completes the
authorized local refresh without publishing a release or replacing prior demos.


### 2026-09-13 — Collection contracts advertise supported operations

**Author direction and question:** Asked to continue the collection review/prototype,
then asked what the assistant would do. The assistant proposed Iterable, Collection,
Sequence, MutableSequence and the existing growable List as a minimal capability chain,
with invariant type parameters and provisional naming.

**Author response:** Observed that this is preferable to a mutable default followed
by a view that rejects Add. The assistant clarified that arrays should expose element
replacement without claiming growth; read interfaces restrict access but do not make
shared storage immutable or prevent mutation through other aliases.

**Action underway:** Recorded the [capability prototype](collection-contracts.md#2026-09-13-capability-prototype)
and compared it with .NET's array/read-wrapper contracts and the proposed read/mutable
interface inheritance change. Implementation exposed a Raven inherited-indexer lookup
bug, which is being addressed on the experiment branch rather than worked around in
samples. No implementation completion or tool refresh is claimed by this planning entry.


**Subsequent direction:** The author asked to focus next on Map and a default
implementation corresponding to .NET Dictionary. The assistant queued Map<K,V>
plus a dictionary-style implementation, proposed evaluating HashMap as the concrete
name, and identified equality/hashing, comparer ownership, duplicate-key behavior,
Option/Result lookup and read/mutation separation as decisions for that slice.


**Capability outcome:** Implemented the experimental hierarchy in the Raven profile
and used normal array member dispatch for Count/indexers. ArrayList inherits the
same read/replacement contracts through List. The generic array shape no longer
hardcodes a single interface, but still forbids incompatible record storage; ordinary
conformance validation rejects unimplemented operations. Validation passed 39 runtime
tests, 22 Raven tests, 60 saved projects, 28 query checks, six capability rejections
and 56 editor checks. The installed .11 tools remain unchanged; Map is the next
planned slice.


**Separate follow-up:** The author asked to consider Option or Result for LINQ
terminal methods such as First and explicitly said to keep the work in separate
slices. The assistant recorded collection capabilities, Map/dictionary work and
terminal-operator outcome design as separate steps. First returning Option is a
proposal; Single cardinality semantics and other operator outcomes remain to decide.
No terminal operators were added to the current capability implementation.

### 2026-09-13 — A first Map implementation, with comparer policy still open

**Author direction.** After separating collection read/replacement/growth capabilities,
the author asked to focus on a Map interface and a default implementation corresponding
to .NET Dictionary. The author also raised Option/Result outcomes for LINQ terminals
and explicitly said, “Keep this as separate slices.” The subsequent “Continue” starts
the Map work; it does not settle every proposed collection name or comparer policy.

**Assistant proposal.** Implement invariant Map<K,V>, MutableMap<K,V> and HashMap<K,V>
using existing managed storage and IL. Because the current library has typed equality
but no general hash-code/default-comparer contract, start with explicit equality and
hash callbacks. Use Option for lookup, Boolean TryAdd for duplicate rejection, Set
for insert/replace, and a snapshot of keys. The assistant identified the convenience
cost of mandatory callbacks and left default comparers, null-key policy, removal and
pair enumeration open. These are assistant-selected prototype boundaries, not a
reported author endorsement of the final API.

**Action and outcome.** Added the [IL implementation](../runtime/raven/Map.neoil),
metadata/importer adapters, a [Raven order-lookup sample](experiments/raven-target/samples/library-maps.rvn)
and [contract/research notes](map-contracts.md). The table stores collision chains in
managed arrays/ArrayLists and grows using saved hashes. The sample demonstrates
Option destructuring, reference keys and shared reference values. No Raven compiler
or runtime opcode change was needed. Direct IL checks cover collection pressure,
reference retention, collisions, snapshots, invalid interface operations and callback
reentrancy. The existing test fixture was also corrected to use the migrated indexer
owner and the target library when assembling.

Validation reported by the assistant: 13 collection/runtime tests, 61 saved-project
cases, 81 signature checks, 10 capability rejection cases and 59 editor checks passed.
The .NET comparison confirms the shared duplicate/replacement behavior and illustrates
its live key view and null-key rejection. neoCLR's key snapshot and current
callback-defined key domain are documented differences, not asserted improvements.
The installed .11 SDK/extension was not changed. LINQ terminal implementation remains
a separate slice, and this bounded Map is not described as a complete Dictionary API.

### 2026-09-13 — LINQ terminal outcomes and concrete collection guidance

**Context and assistant proposal.** Following the author's request to keep Map and
LINQ terminal outcomes in separate slices, the assistant proposed First/Last returning
Option<T>, and Single returning Result<T,SingleError> with Empty and Multiple cases.
Filtered selection would compose through Where; all normal outcomes would dispose
the acquired iterator. Runtime faults would remain faults, without implying fault
unwinding or introducing exception classes. These concrete contracts are an
assistant-selected prototype interpretation of the author's Option/Result direction.

**Author's next slice.** During implementation, the author asked for built-in filter
functions on ArrayList with appropriate Option/Result usage in an upcoming slice.
The assistant recorded a review of existing Find, FindIndex and Exists alongside
additional useful operations. The plan distinguishes absence, empty filtered results,
Boolean questions and actual recoverable errors rather than wrapping every return.
No ArrayList API or implementation was changed in this terminal slice.

**Guideline and correction.** The author said to prefer suitable built-in functions
when the concrete collection is available and use LINQ when querying through an
interface. The assistant initially qualified a blanket speed claim. The author
clarified: “It's not that they are faster necessarily,” explaining fewer allocations
from direct data-structure access instead of a chain of objects. The author then
explicitly allowed LINQ specialization and reiterated the general preference for
built-in operations. The recorded guideline now reflects allocation efficiency,
permits specialization, and does not claim universal faster execution.

**Action and evidence.** Added the terminal library methods, SingleError value union,
metadata adapters and a [Raven sample](experiments/raven-target/samples/library-query-terminals.rvn)
covering pattern matching and propagation. Direct IL tests check cardinality,
short-circuiting, disposal and fault boundaries. A measured equivalent lookup used
five managed allocations for ArrayList.Find versus nine for Where(...).First(),
including common setup. The assistant identified that the existing Find still creates
an ArrayIterator: direct-storage scanning is upcoming work, not a completed change.
Those counts measure this interpreter implementation, not host bytes or elapsed speed.
See the [query API](raven-query-api.md) and [upcoming API plan](runtime-api-plan.md).
The installed SDK/extension remains unchanged; no Raven compiler or opcode change
was needed for the terminal implementation.

**Validation outcome.** The assistant subsequently verified 62 saved-project cases,
29 query checks, 90 signature checks, 59 editor checks and 18 focused runtime tests,
plus Clippy, formatting and the API audit. A .NET 10 target comparison confirms the
existing default-sentinel ambiguity and exception-based cardinality behavior. The
ArrayList work remains planned separately; LINQ specialization remains permitted.

## 2026-09-13 — Concrete ArrayList filtering after the LINQ terminals

**Author's direction.** The author requested built-in ArrayList filtering as a
separate upcoming slice, with Option and Result used appropriately, then directed
continuation. The preceding discussion clarified the motivation: avoid unnecessary
query-chain allocations when a suitable concrete operation exists, while retaining
LINQ composition and possible specialization. It was not a universal speed claim.

**Assistant's selected contract.** Implement the familiar predicate method family:
Find/FindLast return Option<T>, FindIndex/FindLastIndex return Option<Int32>,
Exists/TrueForAll return Boolean, and FindAll returns a new shallow list. Changing
FindIndex from -1 absence to Option is the assistant's concrete interpretation of
the requested outcome review, not a separately quoted author decision. Empty
filtered lists and Boolean questions need no Result wrapper; callback faults remain
runtime faults. Range/removal and fallible-predicate overloads remain open.

**Action and evidence.** Added direct IL scans over the initial backing buffer and
extent, retaining the existing search policy under callback mutation. Updated Raven
metadata, samples, completion expectations and migration documentation. The
[filtering contract](arraylist-filtering.md) compares .NET's method family and index
sentinel with the selected Option contract. Scalar searches no longer create an
iterator. The equivalent Find lookup now measures four managed allocations versus
nine for Where(...).First(), including setup, compared with the earlier five versus
nine observation. This measures interpreter heap objects, not speed or host memory.

**Validation outcome.** Four direct filtering tests passed, including callbacks that
grow the source during GC. Source validation passed 63 saved-project cases, 12
application cases, 104 signature checks and 59 editor checks, plus Clippy and
formatting. A Raven sample initially used a qualified None pattern that did not
satisfy exhaustiveness checking; importing Option cases and destructuring Some
fixed the sample without a compiler change. The .NET comparison ran targeting
net10.0. No new SDK/extension was installed and no release was published in this
slice. Historical Neo behavior remains separate and unchanged.

## 2026-09-13 — Testing the collection slices together

**Context and proposal.** After the ArrayList filtering commit, the author directed
continuation. The assistant selected an integration slice: combine the new Map,
filtering and LINQ terminal APIs in an order application before expanding the API.
This follows the author's earlier request to evaluate the platform through realistic
programs and allocation-aware concrete collection use; the particular scenario is
an assistant proposal, not a newly specified author requirement.

**Action.** Added an Order class, duplicate-preserving registration, Option lookup
and propagation, direct FindAll filtering, array/interface query composition and
Result-based Single handling. The sample demonstrates that filtered membership is
fixed at creation while class instances remain shared with the map and source list.
No runtime or Raven compiler change was required. The documented current limits
include explicit hash/equality callbacks, no transactional registration and no
concurrency guarantee.

**Outcome and evidence.** The [collection workflow](raven-order-workflow.md#collection-integration-scenario-2026-09-13-source-slice)
runs unchanged under colliding hashes and with additional garbage-collection pressure.
Validation passed 15 application checks and 61 editor checks, including inferred
Order members through map lookup and filtered-list access. The assistant initially
wrote a None construction without the required constructor invocation; correcting
the sample to None() resolved the import failure. This does not establish improved
compiler diagnostics for that invalid expression. The source sample and release
procedure are ready for inclusion by the existing bundle builder; no package,
installation or publication occurred in this slice.

## 2026-09-13 — Predicate terminals and the next library-authoring direction

**Author's request and action.** The author asked for First and related overloads
that accept a function predicate. The assistant added First/Last/Single overloads,
retaining Option/Result outcomes, normal-outcome iterator disposal and terminal
fault boundaries. Direct loops avoid a Where wrapper. Verification required control
flow that establishes a first match before reading the saved generic value; the
implementation was corrected to express that without a default T or verifier change.

**Validation outcome.** The source changes passed 63 saved-project cases, 29 query
checks, 15 application checks, 116 signature checks and 64 editor checks, including
predicate signature help. Five runtime tests cover both overload families, matching
order, cardinality, cleanup, faults and allocation comparisons. Clippy passed.
The .NET comparison shows that its List Last(predicate) can traverse backwards,
whereas this prototype keeps forward query order. Installed .12 tools remain the
previous validated build, without these latest source additions.

**Author's proposal and decision.** While this work was underway, the author proposed
writing the runtime class library in Raven, with neoIL still preferred where useful,
to help develop neoCLR itself. The assistant supported a high-level implementation
path and proposed checking bootstrap and generic-import gaps first. The author then
clarified that neoIL remains important but that, now a high-level language targets
neoCLR, the class library should use it. The author requested a release or at least
a tag before proceeding, and emphasized that migration should start now because
library growth would make the translation larger and harder.

**Assistant's response and next action.** Use Raven as the default authoring language
for ordinary library code; retain neoIL for justified low-level, bootstrap and test
uses. Finish the predicate slice and take a source milestone tag rather than claim
that the older .12 binaries contain it. The [library direction](runtime-library.md#raven-as-the-library-source-language-2026-09-13)
records stable library identity, bootstrap metadata and generic-body importing as
migration groundwork. The current bridge admits known generic API calls but rejects
generic application bodies; the migration is not yet implemented by this checkpoint.

## 2026-09-14 — Pause library migration and evaluate Raven target support

- **Author:** Asked whether the Raven bridge should be evaluated before authoring
  System classes in Raven, then directed a pause. Prioritize reviewing Raven changes
  and integrating suitable bug fixes, followed by consistent target mapping rather
  than an attached bridge that compensates for compiler behavior.
- **Author:** Clarified that stable importing is crucial, but Raven itself should
  handle .NET and neoCLR targets. Earlier comments distinguished namespace functions
  from their CLI container and rejected NotImplementedException as a neoCLR API.
- **Assistant:** Proposed separating general compiler fixes, reusable target
  configuration and temporary neoCLR adapters. Retain shared CIL emission where it
  fits; the importer should validate and resolve the emitted contract.
- **Actions:** Paused and locally stashed the scalar Math migration, inventoried the
  37 Raven experiment commits and inspected metadata, iteration, propagation and
  application-import boundaries. Recorded [the assessment](raven-target-evaluation.md)
  with review batches, target-layer responsibilities and migration exit criteria.
- **Finding:** Reproduced the author's string-filter screenshot in a disposable
  project. Adding missing String equality operator metadata made the query run
  without Raven changes; collection-interface conversion was not the failing step.
- **Outcome/open work:** Migration is paused. No Raven fixes have been merged by this
  review; integration requires dependency review and tests on proposed branches.
  Namespace-function identity projection and removing the metadata-only exception
  placeholder remain explicit follow-up work. Installed demo files were preserved.

- **Follow-through after the author's “Continue through those issues”:** Extracted
  general Raven fixes into two reviewed batches on `codex/compiler-fixes-integration`,
  with before/after regressions and a passing 5,489-test baseline. Pushed the review
  branch; Raven main was not merged. Kept target-contract changes on a separate
  experimental branch.
- **Assistant's implementation and finding:** Made emission-core selection a project
  compiler option and separated compilation from neoCLR artifact import. Testing the
  ordinary driver exposed host-reference injection and host-derived defaults; these
  were corrected for explicitly configured metadata targets. The .NET 10/11 matrix,
  targeted compiler tests, five independent compile/import checks and 63 saved-project
  checks passed. [The workflow](raven-target-compilation.md) records how to reproduce it.
- **Remaining direction:** This is an incremental target-support improvement, not a
  completed target pack or general library importer. Migration stays paused while
  stable identities, generic bodies and temporary adapters are addressed. No installed
  SDK, extension or user demo was changed by this follow-through.


## 2026-09-14 — Integrate general Raven fixes into main

- **Author's directive:** “Keep the tracks separate”: experimental neoCLR support
  belongs on its own Raven branch, while general fixes should be integrated into
  Raven main. The author reinforced that anything benefiting Raven generally belongs
  on main.
- **Assistant's response:** Extract fixes by behavior and dependencies, validate with
  ordinary .NET inputs, and advance main without merging the experiment wholesale.
  A fix is not permanently experimental merely because neoCLR exposed it.
- **Action:** Reused the isolated main-based numeric/binding integration branch and
  extracted the namespace metadata/completion fixes. Reworked the namespace regression
  to use default compilation options and normal framework references. It reproduced
  the completion failure without neoCLR settings, then passed after the fix.
- **Workflow:** Recorded the separation in AGENTS.md. Further general cross-target
  emission candidates remain explicitly queued for dependency review; intentional
  Void/array/protocol differences remain on the experimental track. The System-library
  migration stays paused during this integration work.
- **Outcome:** The 47 focused namespace tests and full 5,490-test baseline passed
  without failures/skips. Fast-forwarded and pushed Raven main to `8fa59a967`, containing
  the reviewed general numeric, binding/dispatch and namespace fixes. Experimental
  target policies were not merged into main. The remaining emission candidates stay
  queued for independent review. No installed SDK or extension was refreshed.
- **Track synchronization:** Merged Raven main into `codex/neoclr-namespace-metadata`
  at `5d1022ced`, preserving the experiment's Void regression and main's independent
  namespace test. Compiler source was unchanged by this merge; 50 focused tests passed.
  The experimental branch now contains main as an ancestor. This merge direction does
  not bring experimental policy onto main.

## Release stabilization before library migration — 2026-09-14

**Author direction:** “Our goal is to fix these things before we make the next
release. After that we can worry about migrating code from neoil to Raven.”
This follows the directive to integrate general compiler fixes into Raven main
while retaining experimental neoCLR target behavior on a separate branch.

**Assistant response and action:** Continued independent review of closed-generic
metadata emission, using ordinary .NET reference assemblies and Raven's existing
EmitOptions contract. The adapted regression reproduces an emission failure on
main. The fix was subsequently integrated into Raven main at `031b9aaaa`: 18 focused
checks and the .NET 10/.NET 11 build/run matrix passed. Remaining fixes and release
validation are still pending; this entry does not report a release.

**Outcome/direction:** Stabilizing the compiler/importer integration is a prerequisite
for the next release. Authoring the runtime library in Raven is deferred until after
that release; the current neoIL library remains the implementation during this work.

## General Raven fixes across frameworks — 2026-09-14

**Author clarification:** “Everything that's benefiting Raven, even when targeting
the .NET Framework or the Nano Framework, should be on main. The experimental
integration with neoCLR will stay in its experimental branch.”

**Assistant response/action:** Confirmed that target-independent compiler and
metadata fixes belong on Raven main, updated repository instructions, and continued
independent review of generic-field emission. The boundary is general utility versus
neoCLR-specific policy, not whether a defect was discovered through this experiment.

**Validation boundary:** A .NET 10/.NET 11 matrix result is not evidence of executing
on .NET Framework or NanoFramework. Those targets remain relevant to design/review;
actual target validation must be reported separately when performed.

**Implementation outcome:** Integrated the generic-field fix into Raven main at
`ea6f3383b`; 22 focused checks and the modern .NET build/run matrix passed. The
completed feature branch was removed, preserving the active neoCLR experiment.

## A configurable target model, not scattered special cases — 2026-09-14

**Author direction:** Integrating neoCLR as a Raven target should require very little
special-case compiler handling. The author cited selecting `Iterable<T>` instead of
`IEnumerable<T>` as an example of a manageable framework-contract mapping.

**Assistant proposal:** Put reusable target configuration, compiler correctness and
metadata handling on Raven main. Keep neoCLR's experimental configuration and
unresolved semantic differences on its experimental branch. Classify each mismatch
as a compiler bug, a configurable framework contract or a genuine runtime difference.

**Action/outcome:** Continued independent metadata-emission fixes and recorded this
as the review criterion. This is not a claim that all existing neoCLR special cases
have been removed or that a finalized target-configuration API has been integrated.

**Subsequent author clarification:** Nothing specific to neoCLR should enter Raven
main yet, including tests; those can be considered when the integration is ready.
General fixes can enter now when useful across CLI-metadata-compatible frameworks,
even when their library API surfaces differ. The author raised a possible future
emission/backend rethink, explicitly as an open question beyond this work.

**Assistant response:** Keep current integration evidence independent of neoCLR,
record the narrower boundary in repository instructions, and treat backend changes
as future evaluation rather than part of stabilization. Current imported regression
fixtures use ordinary .NET references and no neoCLR-specific options or names.

**Further author clarification:** NeoCLR should currently remain sufficiently close
to the .NET Framework and other CLI derivatives that few special differences are
needed. More significant differences may arise later, with runtime nullability as
an example. The author clarified that such differences could affect Raven's
**semantic model**, not merely metadata mapping.

**Assistant response:** Distinguish configurable API names from differences in type
identity, conversions, flow analysis, diagnostics and emitted metadata. Continue
compatibility fixes against the shared CLI model now; keep runtime nullability and
its semantic-model consequences as a separate, unresolved design question.

**Immediate author priority:** Stabilize and commence a release so there is a stable
new preview to demonstrate. The assistant proposed finishing the remaining
compatibility fixes, validating demos and packaged tools, and preparing the release;
library migration and new runtime semantics remain outside that release work.
This records the intended sequence, not a completed release gate.

## Raven main stability audit — 2026-09-14

**Author concern:** Raven main should remain stable and integrated changes should not
break its samples; the work should demonstrably improve the compiler.

**Assistant action:** Set aside the pending delegate fix and audited exact main
`ea6f3383b`. The baseline passed 5,493 tests and the standalone corpus built/ran
173/172 samples successfully. The project build exposed four NanoFramework failures
in custom-attribute emission plus a separate MacCatalyst/Xcode prerequisite failure.
The earlier narrow checks had missed that compiler regression.

**Correction/outcome:** Added primitive and enum attribute regressions, both failing
before correction, and separated runtime-type attribute serialization from target
metadata signature resolution. Integrated the correction into Raven main at
`5a67d5d4c`: 39 focused tests, all four NanoFramework builds and the modern .NET
build/run matrix passed. All 38 eligible project executables also passed; the 13
build-only and 20 non-executable classifications stayed explicit. No sample source
or exclusion was changed. MacCatalyst still needs Xcode 26.6; this host has 26.2,
so the entire project-build gate is not claimed green. NanoFramework build success
is not hardware execution. The pending delegate fix was then restored for combined
validation. See Raven's `docs/compiler/main-stability-audit.md` for the audit scope.

**Follow-through:** The restored general delegate fix passed 55 combined focused
checks and was integrated into Raven main at `35a9df494`. Temporary audit/integration
branches were removed. Broader baseline and sample results retain their original
commit scope; release validation remains outstanding.

## Publishing the stabilized preview — 2026-09-14

**Author direction:** “Then its perhaps time for a new preview release of neoCLR”.
This followed the requirement to keep Raven main stable, integrate general compiler
fixes there, and retain neoCLR-specific integration on the experimental branch.

**Assistant action:** Synchronized the experimental Raven branch with the reviewed
main fixes, built a fresh SDK/VSIX and prepared Preview 6. Release validation found
Rust syntax newer than the declared minimum, preview samples being checked against
the historical library, and additional extension dependency notices. Corrected those
issues and reran validation against the selected source and packaged artifacts.

**Outcome:** Published [Preview 6](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.6)
from neoCLR `5c52b4bce2da2ef31f279d0f5f3fa924b82baf61`, with experimental Raven
`0.1.12-neoclr.13` from `246d697bf6c69ff8cc56ca4859879edd7c081e7d` on
`codex/neoclr-namespace-metadata`. All six Linux/macOS/Windows stable/minimum-Rust
source CI jobs passed, along with 15 extracted-package suites, the match/signature
checks, both packaged language servers and isolated VSIX installation. All eight
uploaded asset digests were verified. The release includes source, the macOS arm64
runtime bundle, separate Raven SDK/VSIX, notices and validation evidence.

**Still separate:** This is the neoCLR experiment, not a full Raven release gate.
The earlier main audit's MacCatalyst/Xcode limitation remains documented. Runtime
library migration to Raven and broader semantic changes remain deferred.


## Website: features, influences and open discussion — 2026-09-14

**Author direction:** Improve the website to better market neoCLR and its implemented
improvements, list the project's influences and features, and demonstrate them with
samples. The author then requested areas being explored for improvement and a general
invitation for feedback and discussion.

**Assistant proposal:** Organize the page around the CLR foundation, design reference
points, current feature contracts and source-backed examples. Distinguish implemented
preview behavior from research and explain benefits alongside .NET compatibility costs.

**Action:** Expanded [the homepage](../website/index.html) with feature summaries and
six generated excerpts covering Result, Option, generic Void, collection capabilities,
UTF-8 slicing and query cardinality. Added research questions on nullability, memory
views, callables, async, text, collections and injectable clocks. Corrected outdated
application-type/capture limitations against the current API coverage record. Linked
general feedback to GitHub Issues after the repository API reported Discussions disabled;
no repository settings were changed. Noda Time is described as a research reference,
not an adopted API; Rust Result is a design precedent, not an imported ownership model.

**Evidence and status:** Reused [platform direction](platform-direction.md),
[API coverage](raven-runtime-api-coverage.md), [collection contracts](collection-contracts.md),
[query contracts](raven-query-api.md), [text model](text-model.md), and
[array variance](array-variance.md). Checked the linked Rust Result, .NET array/Unicode,
and Noda Time primary references on 2026-09-14. The website build checks embedded
excerpts and local links. These are website edits, not new runtime capabilities or a
new release; deployment is separate from the local changes.

**Author clarification:** The focus is demonstrating the platform's style through
Raven code, with some neoIL as a supporting view. “Discussion” means an invitation
to exchange ideas, not a request to use GitHub Discussions.

**Assistant adjustment:** Put a source-backed Raven example in the hero, moved
features and examples before the influences section, and placed the neoIL example
after the five Raven examples. Kept the invitation broad, with GitHub Issues as an
available contact route rather than defining discussion as a GitHub feature.

**Publication direction and validation:** The author requested committing this work
on main and publishing the site. The local build and repository-link checks passed;
browser checks covered desktop and 390px mobile, including example navigation and
absence of page-wide horizontal overflow. The assistant is committing the website
changes with the changelog and using the existing GitHub Pages workflow. Deployment
completion must be verified from that run rather than inferred from a successful push.


**Further author direction:** Organize the message around familiar value/reference
semantics and metadata/assemblies; Void as a real generic argument and its implications
for Func APIs; Option, Result, modernized collections and date/time; evolving APIs and
feedback; and Raven as the first adaptable compiler target with .NET-hosted tooling
and a VS Code extension. Describe the next work as moving the class library from
neoIL to Raven and implementing missing library features.

**Assistant action:** Revised the website around that outline and added excerpts
from the existing Void callback and calendar samples. Distinguished .NET-hosted Raven
tooling from neoCLR execution, and a migration direction from unchanged-binary
compatibility. Listed library migration and API expansion as planned work; this website
request does not itself resume the paused library implementation or override its
compiler-stabilization prerequisites. Reused the Void, delegate, calendar, format and
runtime API design records. Continued the authorized main/Pages publication workflow.

**Runtime additions requested by the author:** Mention the interpreter, garbage
collection and initial debugger support. Present a possible future JIT and garbage
collector improvements as areas under consideration.

**Assistant action:** Added current runtime capabilities with links to execution,
GC and debugger documentation, distinguishing the terminal debugger from future
Raven source debugging in VS Code. Added separate JIT and collector research cards;
no JIT selection, implementation or performance improvement is claimed.

**Positioning clarification:** The author wants neoCLR marketed as a software
platform improving on .NET and evolving through feedback, rather than centering the
compiler integration alone. The assistant revised the hero and platform/library
introductions to connect type system, execution engine, class library and tools,
with Raven as the demonstration language. The invitation asks visitors to help shape
the platform; implemented differences and future investigations remain distinct.


**Further structure and audience direction:** The author requested a concise overview
organized as Familiar, Runtime, Type system, Runtime class library (Option, Result,
unions, modernized APIs, collections, date/time and LINQ), Migration, Tooling and next
implementation work. It should be relatable to .NET developers but stand on its own,
explaining features, familiar behavior and differences rather than abstract sales
copy. UTF-8 remains prominent. APIs are evolving and feedback is welcome.

**Examples and highlighting:** The author requested Raven union sugar and a note
about its multiple construction forms, plus syntax highlighting following the adjacent
Raven repository or MyServiceBus's website. The supplied MyServiceBus path did not
exist; the assistant found `/Users/robert/Projects/MyServiceBus` and inspected its
TextMate/Oniguruma integration. Reused Raven's grammar with its MIT license and
adapted tokenization to build-time HTML spans rather than shipping Monaco or a
browser tokenizer. Added a test for token scopes, multiline comments, escaping and
exact source-text preservation. Source examples now use qualified Result factories
and imported Some construction; detailed union patterns and research are optional.

**Validation scope:** The Result propagation and UTF-8 samples compiled, verified
and ran using the locally available stabilization toolchain. A focused imported
Some construction program also compiled and ran with result 42. The full current
query sample could not run on that older package because its metadata predates the
query-terminal APIs; no full query-suite pass is claimed. The original `.Some` trial
failed and was replaced with the validated imported `Some(value)` form. No Raven or
MyServiceBus source was edited. Continue the already authorized main/Pages workflow.

**Invitation clarified:** The author reiterated that visitors should be invited to
join the discussion and give feedback, specifically about the runtime and APIs.
The closing section now names that focus directly.

**Website checks:** HTML nesting, anchors and repository links passed. Desktop and
390px mobile review confirmed the six-card library grid, highlighted code and union
disclosure behavior without horizontal page overflow.


## Indexer ergonomics in Raven — recorded 2026-09-14

**Author's report and direction:** Supplied a VS Code screenshot showing
`typeof(int).GetProperties().Item.` offering `PropertyInfo` members and said,
“This should be aligned with the experience in C#.”

**Assistant's response:** Explained that Item is an indexer metadata name and that
element-member completion belongs after indexed access, such as
`GetProperties()[0].`. Proposed treating this as a general Raven fix, separate from
the imported-union emission work already in progress.

**Action and outcome:** Reproduced the behavior using ordinary .NET collection
classes/interfaces and a Raven-declared indexer. Fixed named lookup, symbol
referenceability and semantic-model fallback, with positive coverage for indexed
access and ordinary properties named Item. Integrated `ac4901f6b` into Raven main;
440 focused checks passed. The preceding union-emission fix was separately integrated
as `43f288b05`, with 291 focused checks and the .NET 10/.NET 11 matrix passing.

**Still open:** The experimental Raven branch and installed SDK/extension have not
been synchronized or rebuilt in these slices. The source fixes do not yet change
the author's installed editor. See the [target evaluation](raven-target-evaluation.md)
for validation scope and remaining stabilization reviews.


## Empty arrays on the generic array type — recorded 2026-09-14

**Author's direction:** Pointed out that neoCLR has `Array<T>` and proposed
`Array<T>.Empty`, with Empty probably exposed as a property.

**Assistant's response:** Agreed that the closed array type already supplies the
element type. Distinguished this proposed neoCLR API from .NET's
`System.Array.Empty<T>()` method, and from Raven's general ability to emit an empty
array when a target lacks that factory.

**Action and status:** Added this distinction to the ongoing array-factory review.
The property and its compiler mapping are proposals, not implemented in this entry.
Any neoCLR-specific mapping remains on Raven's experimental branch; general
metadata-capability handling belongs on Raven main after independent validation.


**Follow-up direction:** The author pointed out that ForEach no longer needs to be
static with `Array<T>`. The assistant proposed an instance `ForEach(action)` method,
with the receiver supplying both the array and its element type, retaining the
existing `Func<T, Void>` callback contract. This is recorded for the array API slice;
the static implementation has not yet been replaced.


**Implementation follow-through:** Added the static Empty getter and instance ForEach
to the Raven runtime profile, compiler reference declarations and importer catalog.
Raven's experimental array projection now exposes configured shape members. Empty
uses a zero-length allocation for now; no shared-identity guarantee is made. Four
Raven samples compiled, imported, verified and executed, and 121 signature checks
passed. Expression-bodied callbacks exposed a general void-call stack defect;
`8dbd96fb6` was independently reproduced/tested and integrated into Raven main, then
cherry-picked as `5c32d1d06` into the experiment. Nominal generic Void emission remains
experimental. Installed tools have not yet been refreshed.


## Target profiles, symbol implementations and future reach — recorded 2026-09-14

**Author's direction:** Suggested avoiding accumulated framework-specific exceptions
in Raven's semantic model and code generation, using separate symbol implementations
where helpful and configurable contracts such as Iterable versus IEnumerable. Proposed
.NET and neoCLR Target Profiles encompassing .NET versions, NanoFramework and neoCLR,
and eventually refactoring emission to support backends beyond Reflection.Emit.

**Assistant's response:** Proposed separating a framework profile from an emission
backend: several profiles could share one backend. Suggested dedicated/composed symbol
representations where behavior really differs, while keeping common CLI symbols and
language rules shared. The generic array shape is a candidate for evaluating that
boundary. Proposed preserving the current stabilization scope rather than starting a
backend rewrite inside the array slice.

**Author's additional future directions:** Asked to record that Raven might eventually
be bootstrapped, with its compiler written in Raven. If neoCLR becomes a project with
sustained investment, consider different architectures, Native AOT and potentially
microcontrollers. Explicitly described these as far-future considerations.

**Assistant's response and action:** Recorded the
[profile/symbol/backend proposal](raven-target-profiles.md), linked it from the contract
assessment and roadmap, and separated compiler bootstrapping, runtime portability,
ahead-of-time application compilation and constrained runtime profiles. These are
possible directions, not promises or selected implementations. No bootstrapping, new
backend, architecture port or microcontroller implementation was started.

**Concrete outcome of the ongoing slice:** The generic array API is committed in
neoCLR as `ff617c4` and its compiler projection in experimental Raven as `8823261b6`.
The general void-invocation fix was independently integrated on Raven main as
`8dbd96fb6`. The array implementation remains an incremental metadata-driven projection;
a full target-profile abstraction and specialized array-symbol design remain open.


**Author's clarification:** “I mean we might support microcontroller architectures as
a target.” Named the broader theme “targetability and portability.” The assistant
updated the architecture proposal and roadmap accordingly: architecture targeting and
implementation portability are the direction; a smaller runtime profile is only one
possible future consideration, not the meaning of the request.


## MSBuild support for Raven projects — recorded 2026-09-14

**Author's direction:** “We should have some proper MSBuild support targeting neoCLR
eventually so that we can use the Raven project files.” Explained that this is useful
for compiling projects.

**Assistant's response:** Proposed treating this as a future targetability milestone:
reuse Raven project compilation and supply neoCLR reference/build assets, with MSBuild
orchestrating compilation and the temporary import/verification stage. Suggested
single-project support first, followed by incremental builds, project references and
consistent editor resolution. Package/SDK names and a framework moniker remain open.

**Action and outcome:** Added the proposed scope, alternatives and validation criteria
to the [target-profile design](raven-target-profiles.md#msbuild-project-support--future-milestone-2026-09-14)
and roadmap. This records future work; no MSBuild integration was implemented in this
exchange, and the existing local .14 installation is unchanged.

**Author's clarification:** “I don't think we should have it all” — MSBuild is a useful
initial build system to support. Added: “It won't be a .NET SDK integration, but the
experience using the project files will be similar.”

**Assistant's response and action:** Narrowed the recorded initial scope to a small
MSBuild path for `.rvnproj` compilation using neoCLR references and build targets,
independent of Microsoft.NET.Sdk. Kept incremental builds, project references and
asset restore as optional later slices rather than requirements for initial support.
The familiar project-file experience is the goal; no SDK integration was selected.


**Implementation follow-through:** After the author asked to continue, the assistant
implemented standalone MSBuild props/targets and a `.rvnproj` template in neoCLR.
The existing installed Raven compiler compiles the project, followed by independent
import/verification; Build does not run the program. No Raven edits or .NET SDK
integration were needed. Added distribution/editor build-task configuration and
[documented the bounded build path](raven-msbuild.md), with larger build-system
features explicitly deferred.


**Author's observation:** A project targeting neoCLR should also improve the language
server experience by giving it context.

**Assistant's response:** Agreed that source items, references and target contracts
should come from the same project evaluation for build and editor. Raven already
reads `.rvnproj` files for language services; the standalone imports centralize that
configuration. Proposed checking completion against the actual new MSBuild project,
rather than assuming successful compilation proves editor behavior.


**Verified outcome:** The installed standalone MSBuild project passed all 68 editor
checks against its configured language server, confirming shared project/reference
context without a .NET SDK import. The fresh local bundle also passed 15 build
scenarios; its propagation demo built and ran separately. Instructions and exact
revisions are in the [MSBuild guide](raven-msbuild.md). This is a local installation,
not a new public release.


### 2026-09-14 — Plan reflection around needs before expanding descriptors

**Author's direction:** Requested `Type.IsValueType` and restrained additions to the
existing model. Then asked for a roadmap review of reflection versus introspection,
using .NET's `Type`/`TypeInfo` split as a concern: identify the needs before choosing
how to implement the functionality.

**Assistant's response:** Proposed examining metadata inspection, runtime execution
capabilities, portability and AOT separately, without preselecting separate public
classes. Kept the bounded `IsValueType` addition in the active preview scope.

**Action and outcome:** Added a [roadmap investigation](reflection-model-review.md)
with a .NET baseline, alternatives and validation criteria. This records planned
architecture work; it does not claim a new reflection implementation or a completed
preview release.


**Author's follow-up:** API planning can identify the core of the platform, its
standard contract, and features or APIs that differ by target.

**Assistant's response and action:** Added a common-core, optional-capability and
host-service classification to the runtime API plan and roadmap. Required availability
and conformance questions to be resolved before choosing profiles or capability
metadata. The classification is a planning framework, not a finalized standard or
an implemented target-capability system.


### 2026-09-14 — Combine the MSBuild workflow and library demonstration

**Author's direction:** Asked to continue with the three proposed slices, then said
“Or together.” The assistant proposed a combined preview: make standalone MSBuild
the primary workflow, add a bounded application-plus-library scenario, then validate
and publish the complete package. The author subsequently requested IsValueType and
future reflection/core-contract planning.

**Implementation so far:** Added one-level ProjectReference orchestration using
Raven's existing project evaluation, with exact reference-pack compatibility checks,
changed-library rebuilds and failed-output invalidation. The source MSBuild suite
passes 22 scenarios; the library project's editor scenario passes 69 checks before
building. Added separate Build/Run tasks and a readable two-project example. No Raven
compiler changes were needed. Packaged release validation and publication remain
pending; these source-check outcomes are not a release claim.


### 2026-09-14 — Task-based async with runtime support

**Author's direction:** Said that neoCLR need not retain all backward compatibility
and can choose the latest async model: “we should stick with task-based, the
task-based abstractions.” Intended execution support belongs in the runtime.
Compiler-generated async state machines are not necessarily required, but may be
used transitionally because Raven already supports emitting them (author-reported
capability, not validated for neoCLR in this exchange).

**Assistant's response:** Distinguished the selected public task abstraction from
runtime suspension and resumption. Clarified that removing compiler-generated state
machines does not remove the need to preserve execution state. Proposed treating
Raven lowering as an optional transition and retaining modern .NET as a comparison,
without assuming that newer machinery is inherently faster or copying legacy contracts.

**Action and outcome:** Updated the [platform direction](platform-direction.md#async-and-time-preserve-the-application-model),
backlog, roadmap and Raven application notes to reflect this decision. Consulted the
.NET TAP documentation and runtime-async draft; no runtime or compiler implementation
was changed. Earlier entries describing the model as open remain historical positions.

**Open:** Concrete task types, no-payload completion, Result/error and cancellation
semantics, scheduling, references across suspension, GC roots, cleanup, debugger
integration and metadata. A pinned implementation comparison and a bounded execution
probe remain needed; this exchange does not choose opcodes or promise a release.


**Author's clarification:** The runtime will eventually contain suspension logic,
while Task still represents the operation. Explicitly identified Task<void> as possible in neoCLR, using its ability to accept
Void as a generic argument.

**Assistant's response and action:** Recorded Task as the operation representation
and Task<Void> as completion without a result payload in the platform direction.
This sharpens the intended contract; task members, suspension machinery and error
semantics still require design and implementation.


**Author's library-design clarification:** Application projects may still use
callbacks. The runtime class library and System namespaces should be designed for
modern consumers and modern programming languages.

**Assistant's response and action:** Recorded this distinction in the
[API policy](api-policy.md#design-for-modern-language-consumers) and linked it from
the async direction. Applied task-based completion to public asynchronous operations;
explained that callbacks can still serve roles such as predicates, event handlers
and internal adapters. These examples are the assistant's interpretation of the
principle, not additional author-selected API contracts. No APIs were implemented
or removed; detailed contracts and adaptation needs remain open.


**Author's correction:** Clarified that some runtime APIs may themselves be better
expressed as callbacks. Tasks are the normal way to represent asynchronous execution,
not an exclusive requirement for all runtime APIs.

**Assistant's response and action:** Updated the API policy and platform direction
to state this explicitly, including public runtime APIs. The earlier callback
examples were illustrative, not an exhaustive set of exceptions. No implementation
changed.


**Author's feasibility question:** Asked whether neoCLR already has everything needed
for compiler-generated async/await, given Raven's existing state-machine emission,
and whether it could be made to work before runtime suspension.

**Assistant's assessment and action:** Inspected the local Raven lowering and neoCLR
contracts and recorded a [source-backed feasibility assessment](async-state-machine-assessment.md).
The approach appears feasible without suspended VM frames, but current task/builders,
error lowering, heap state identity and continuation/execution ownership need work.
Proposed a guest queue and an initially incomplete await as a bounded probe. No async
implementation or execution test was performed; no Raven source was modified.


**Author's error-model clarification:** Directed that errors should continue to
propagate through Result, giving Task<Result<T>> as the asynchronous shape.

**Assistant's response and action:** Updated the async direction, API policy and
feasibility assessment: Task carries completion and Result carries success or a
recoverable error. Await yields Result; an Err completes the task as a value rather
than entering a task-exception channel. Recorded the current explicit-error spelling
Task<Result<T,E>> without claiming a one-parameter Result alias is implemented.
Cancellation and terminal Fault handling remain separate open contracts. No code
was changed or async execution validated.


**Author's API-design questions:** Asked whether asynchronous APIs should also have
synchronous versions, whether methods need an async marker, and whether neoCLR
should use different conventions or behavior from .NET.

**Assistant's proposals and action:** Recorded an [API design discussion](async-api-design.md)
comparing .NET guidance, Rust activation and .NET ecosystem naming policy. Proposed
purposeful sync/async surfaces, explicit Task/await contracts, familiar names as a
starting point and further decisions on cancellation, ownership and scheduling.
Distinguished optional frontend async syntax from implicit suspension and required
runtime metadata. These recommendations have not been adopted by the author or
implemented; broader comparison and executable validation remain open.


**Author's naming clarification and decision:** Clarified that the question concerned
Async method-name suffixes. After the assistant recommended ordinary async names,
the author agreed: make async the ordinary operation, identified by return type,
with blocking/sync alternatives as the exception. Stated that existing synchronous
APIs were a major reason for Microsoft's .NET naming convention. Requested that the
decision be documented and committed. The author also explicitly corrected the error
shape to Task<Result<T,E>> earlier in this exchange.

**Assistant's revised proposal and action:** Initially proposed retaining Async in
the broader review, then recommended omitting the mandatory suffix after the naming
clarification. Updated the API policy, async comparison and platform direction to
record the selected convention and distinguish it from still-open behavior proposals.
Recorded the historical explanation as the author's rationale, supported by TAP's
coexistence guidance rather than claiming a complete history of Microsoft's decision.
Prepared this documentation and the preceding async discussion for commit; no runtime
or compiler implementation or existing API names changed.

**Release outcome (2026-09-14):** Published Preview 7 from `5da27a7` after six
stable/minimum-Rust Linux/macOS/Windows CI jobs and all 18 packaged validation suites
passed. The package includes MSBuild Build/Run tasks, the one-library demo and
Type.IsValueType; the SDK/VSIX remain the reviewed experimental Raven .14 build.
The core-contract and introspection/reflection reviews remain plans. Evidence and
checksums accompany the [release](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.7).


### 2026-09-14 — Namespaces rather than utility classes

**Author's direction:** Asked to resume library migration, then clarified that
System.Math should be a namespace containing functions. Directed that, at least
for now, this should follow Raven's existing .NET metadata contract. Explained that
utility classes usually substitute for namespaces of functions, and that neoCLR
need not copy that public structure. A real namespace should also accommodate
related types and other declarations, not only functions and constants.

**Assistant's response and action:** Agreed to separate the public namespace API
from its CLI representation. Recorded the [API guideline](api-policy.md#namespaces-for-utility-functions-2026-09-14)
and updated the Math migration plan. Inspected the existing namespace-member probe
and reran it successfully (eight checks): target-owned TopLevelAttribute metadata,
separate consumer imports and overloads, completion, accessibility and rejection of
unmarked lookalike containers. No new binary metadata format or Raven compiler
changes were introduced for this decision.

**Outcome and remaining work:** The guideline is recorded. The earlier uncommitted
static-class Math pilot needs to be adapted; the namespace Math reference surface,
qualified calls, coexistence with related types, and implementation import still
need validation together. The probe result establishes the existing namespace
contract, not completion of the Math migration. Cross-language consumption and
reflection of generated containers remain explicit design costs.


### 2026-09-14 — Shared System project and namespace lookup correction

**Author's follow-up:** Directed continuation of the namespace-function pilot, then
asked for a shared project for foundational runtime APIs, comparing it with the
historical mscorlib grouping. Subsequently explicitly asked to fix the Raven
resolution issue too. The assembly's historical name was context for the grouping
question, not a selected neoCLR assembly identity.

**Assistant's implementation:** Consolidated the pilot into
[System.rvnproj (now System.Runtime.rvnproj)](../runtime/raven/System.Runtime.rvnproj), with Math as its first source.
The source exposes namespace functions through Raven's existing CLI container and
TopLevelAttribute contract. Added fresh bootstrap reference generation, ordinary
Raven project compilation, signature-checked importing and checked-in executable
fragments. Retained the existing internal Math method owner for direct IL and the
archived Neo frontend, while exposing a namespace in Raven. Generated bodies replace
the five handwritten scalar implementations; native Double services remain.

**Problem found and compiler action:** A consumer adding a type to System.Math could
use wildcard-imported functions but could not call them with qualified names.
Reduced this to ordinary .NET reference metadata: source-only namespace lookup
omitted referenced functions and constants. A focused regression failed before the
fix; all 284 namespace/import tests passed after qualified lookup used the merged
namespace. Integrated only that general correction into Raven main as `3ec32c96e`,
then cherry-picked it onto the experiment as `008cb3245`. Removed the temporary,
fully merged correction branch. No neoCLR policy was added to Raven main.

**Validation and remaining scope:** The separate Raven consumer checks 69 outcomes,
including named arguments, wildcard/qualified calls, a related namespace type,
Int32 boundary values, Result payloads/errors and a Double function. Six invalid
library contracts are rejected without executable output. Fresh regeneration matches
the checked-in bodies, and existing direct runtime Math tests pass. The
[authoring document](raven-system-library.md) distinguishes the bootstrap reference,
implementation input and executable library. General generic-body importing and
migration of core type definitions remain later gates. The installed .14 SDK is
unchanged and predates the compiler correction; no new release is claimed here.

**Author's source-style and workflow follow-up:** Suggested importing
`System.Result.*` and `System.Option.*` to use `Ok`, `Error`, `Some` and `None`
directly, then expressed the hope of porting the API to Raven slice by slice without
interruptions. The assistant identified a repeatable build/validation path as the
objective, with generic bodies and shared type identities still requiring proof.
Updated Math and its consumer matches to imported Result cases. The qualified
carrier annotation remains explicit after the signature guard caught an unresolved
return-annotation issue; recorded that as a deferred general Raven candidate, not a
completed compiler fix. This slice establishes the first migration path, not proof
that every remaining API can already be translated without integration work.

### 2026-09-14 — Runtime Contracts and one unit value type per target

**Author's direction:** Asked for Raven targeting neoCLR to use Void wherever its
.NET target uses Unit, while keeping ordinary void calls free of a stack result.
Clarified that Unit is not another neoCLR platform type. Compared this selection
with Iterable/IEnumerable selection, endorsed the name “Runtime Contract”, and
explicitly directed integrating reusable target support into Raven. Also asked for
compiler changes and the neoCLR integration to be documented in both projects.

**Assistant's response and work:** Separated the unit value representation from the
no-result call convention and added an opt-in unit contract to compilation/project
options. The selected target type represents `()` in value contexts; intermediate
emitter Unit machinery is removed from target output. Built an end-to-end probe for
Void values, calls and Result<Void, E> propagation. Independently extracted metadata,
iteration, propagation and core-selection mechanisms onto a main-based Raven branch,
retaining default .NET exception behavior and excluding the experimental generic-array
and no-exception policies. General unit-contract tests use System.ValueTuple on .NET.

**Related compiler correction:** The earlier unresolved-return candidate was reduced
to an ordinary .NET diagnostic defect and fixed on Raven main as `0c66fbaa7`, with
`e20534894` in the experiment. Missing imports now prevent emission. Case imports and
carrier-type imports remain distinct. Generic Unit storage testing also exposed an
invalid generic void emission in the default target; the correction preserves a real
unit value type in generic arguments.

**Scope:** Runtime Contracts describe compiler-facing requirements; a target profile
selects their values and supplies references. They do not implicitly select exception,
array variance or cleanup policies. Generic implementation importing was deferred in
this turn to prioritize the author's unit-contract direction. No SDK installation or
release is implied by these source changes; final validation/integration is recorded
with the implementation commits and changelog.

**Completed outcome:** Raven main now includes the reusable metadata, iteration,
propagation, core-selection and unit contracts through `2d17199a1`; the temporary
integration branch was fast-forwarded and removed. The experimental branch adopts
unit projection as `fc4592663`, retaining its separate target policies. Raven's
compiler documentation and both repositories' workflow instructions now require
compiler-affecting changes to be recorded on both sides. The combined contract/project
suite passes 87 tests; Raven's CI gate passes 311 compiler, 73 core and 249 language-server
checks with three existing skips. The experimental unit suite passes 13 checks.
The neoCLR probe emits no System.Unit and executes six expected output lines,
including both propagation paths. All 69 Math consumer results and six rejected
contracts pass; regenerated Math bodies remain identical. These are local commits;
no new tools installation or publication was performed.


## 2026-09-14 — Larger library migration slices and an Operators container

**Author direction:** Asked for larger slices and to continue porting feasible runtime
library code, fixing Raven/compiler or neoCLR runtime problems encountered along the
way. The previous rule remains: general Raven fixes go to main independently;
neoCLR-specific target policies stay experimental.

**Assistant action:** Extended the bounded importer from direct generic parameters to
constructed collection, delegate and union signatures. Ported ToList, First, Last and
Single with predicate overloads into the shared Raven System project. Preserved
Option/Result outcomes and iterator cleanup, then checked the generated bodies with
existing query/runtime tests. General imported-signature, generic-member emission and
same-namespace type-lookup defects were reproduced with ordinary C# reference
contracts and corrected independently in Raven. Bare empty member-union returns also
exposed missing case construction; the independent .NET regression and correction
cover return expressions and explicitly typed local initializers.

**Author refinement:** “We should probably not call that type Enumerable but Operators
perhaps.” **Assistant response and action:** Adopted `System.Linq.Operators` for the
extension-method container, updating reference metadata, direct IL calls and docs;
receiver syntax remains `items.First()`/`items.ToList()`. Consumers need rebuilding.

**Outcome and remaining work:** The seven terminal overloads execute from Raven-generated
bootstrap bodies. Instance type definitions and deferred iterator classes still require
shared implementation/reference identity support before they can migrate faithfully.
The [authoring document](raven-system-library.md) records this boundary and validation;
compiler fixes and commit references are tracked in the
[integration evaluation](raven-target-evaluation.md). No release or local tools refresh
is implied by this source migration.

## 2026-09-15 — Injectable clocks and factories on collection interfaces

**Author proposal:** Suggested a `Clock` interface with a `SystemClock` implementation
for dependency injection and mocking. Considered `Clock.Instance`, but explicitly
said this might be bad design and emphasized the abstract time-provider purpose.
Separately proposed `List<string>.Create()` to request an ArrayList through its List
contract, observing that List is currently mutable.

**Assistant response:** Recommended selecting SystemClock at application setup and
passing Clock to time-dependent code. A possible SystemClock.Instance convenience
would keep that choice visible; a static default does not inherently rule out
injection. Considered the collection factory a reasonable convenience for fresh
mutable storage, with concrete construction retained when implementation-specific
behavior matters. Highlighted the dependency from a factory on the interface to its
default implementation and the need to validate static interface-member binding.

**Actions and outcome:** Checked the implemented List/MutableSequence/Sequence
hierarchy, confirming List supports Add and indexed replacement. Recorded the
[clock proposal and comparisons](date-time-design.md#clock-contract-and-default-implementation-proposal--2026-09-15)
and [collection factory tradeoffs](collection-contracts.md#interface-owned-factory-proposal--2026-09-15).
Neither factory/default accessor nor a new Clock interface was implemented. These
remain proposals; clock method shape and final factory placement are open.

**Ongoing migration:** Completed the separate nongeneric instance import gate in
`d073ee0`, with constructor/private-state/property execution and contract rejection
checks. This does not yet port generic collection implementations or establish
static members on interface implementations.

## 2026-09-15 — A namespace function for terminal faults

**User proposal:** While continuing the runtime-library migration, proposed a
`System.Fault` function “for causing the system to fault and terminate”, then clarified
“with a message”.

**Assistant response and action:** Selected the existing terminal guest-fault boundary,
with a computed String diagnostic, and implemented a namespace function using Raven's
existing CLI namespace-container contract. Distinguished guest termination from forcibly
aborting an embedding host and made no cleanup guarantee. The compiler's recognition
of non-returning calls is a separate open issue; no name-specific Raven policy was added.
See [the contract and validation](system-fault.md). This provides a library fault API;
checked generic storage and private query implementation dependencies remain migration work.


## 2026-09-15 — Confirming Raven fixes against .NET

**User question:** “You do verify bugs in Raven with its .NET target, do you?”

**Assistant response and action:** Confirmed that the imported-generic-method repro
uses an independent C# library and executes on .NET 11 under default and explicit
System.Runtime metadata options. Two explicit-metadata cases failed before the fix.
A separate generic-array repro subsequently crashed its isolated .NET test host;
typed element access repaired that general emission defect.

**Outcome:** Integrated the fixes separately into Raven main as `730adc7b0` and
`8dfb64a2b` after focused tests and the bounded integration gates passed, then retained
the experimental cherry-picks separately. Removed the temporary general branches.
No .NET Framework/NanoFramework execution was claimed. Checked storage and private
library-helper admission remain specific to the neoCLR experiment. See
[the evaluation and validation](raven-target-evaluation.md).

## 2026-09-15 — NeoCLR Time API v1 proposal

**Author proposal:** Define a layered time architecture with `Instant` as the real
timeline point, `Duration` for elapsed time, separate civil values (`Date`, `Time`,
`LocalDateTime`), explicit offset/zone projections, independent calendars, and
`Period` for calendar arithmetic. Keep `Clock` as a narrow injectable provider of
`Instant`, with system, fixed and manual implementations. Model timezone gaps and
overlaps as explicit union outcomes, and use Result for parsing and lookup failures.
Retain recognizable .NET naming and ergonomics without creating another combined
`DateTime` type. The supplied Raven-shaped examples are illustrative; their Raven
validity was not asserted.

**Assistant action and outcome:** Updated the [date/time design proposal](date-time-design.md#neoclr-time-api-v1-proposal--2026-09-15)
with the type responsibilities, .NET/Noda Time comparison boundary, tradeoffs and
validation requirements. No new time types, timezone implementation, calendar system,
parser, formatter or runtime contract were added by this documentation update. The
existing Date/Time core and static local-clock API remain implemented; the v1 layering,
method signatures, defaults and DST policies remain provisional.

## 2026-09-15 — Minimal Unicode-centred text model proposal

**Author proposal:** Refine the text direction to a small Unicode-centred model:
Unicode supplies the semantics, String uses canonical UTF-8 storage, UTF-8 and UTF-16
are explicit representation views, and ASCII is a constrained subset rather than a
separate text system. The proposal makes Char a Unicode scalar value, suggests
`AsciiChar`/`AsciiString` constrained representations and explicit Result-based ASCII
narrowing, and leaves other encodings as `Encoding` codecs. The supplied Raven-shaped
examples are illustrative; their validity in Raven was not asserted.

**Assistant action and outcome:** Recorded this as a substantive proposal in the
[text-model review](text-model.md#minimal-unicode-centred-model-proposal-2026-09-15),
including the conflict with the currently implemented UTF-16 code-unit Char contract,
the .NET-compatible alternative, migration costs and validation needs. No compiler,
runtime, metadata or String API behavior was changed. The choice between the existing
Char/Rune split, a scalar-oriented String with a compatibility Char view, and the
minimal scalar model remains open.

## 2026-09-15 — Consolidated String and encoding proposal

**Author clarification:** The intended follow-up was to ensure that the String proposal
captures the supplied detailed model: immutable Unicode String as a sequence of scalar
Char values, canonical UTF-8 storage, explicit `Utf8String`/`Utf16String`/`AsciiString`
representation types, a separate `Encoding` transformation abstraction, and the
existing Option/Result/Fault conventions. The examples are illustrative and are not
asserted to compile as Raven.

**Assistant action and outcome:** Expanded the [text-model proposal](text-model.md#consolidated-string-proposal-semantic-text-and-representation-views)
to make that model the preferred direction for further review. The current UTF-16
code-unit Char behavior, existing String slicing contract and runtime implementation
were not changed; migration, view lifetime, metadata and interop validation remain open.


### 2026-09-15 — Finish the port before revisiting API design

**User direction.** The author asked to “Finish the ongoing work” and clarified:
“We won't delve to much into the design of the API. We have design proposals, but
that is the focus after the port.”

**Assistant response and action.** The assistant scoped completion to the ongoing
Time port, preserving its existing public behavior and deferring Date and API
redesign. The implementation moves Time into Raven and extends the checked library
importer for its existing interfaces, static factories and readonly receivers.
A parser issue reproduced with Raven's .NET target is recorded as a deferred general
compiler candidate; no Raven compiler changes are included in this slice.

**Outcome.** Source and regression coverage are recorded in
[the library migration notes](raven-system-library.md#time-value-implementation--2026-09-15).
Design proposals remain proposals and will be considered separately after the port.


### 2026-09-15 — Broader implementation batches after Date

**User direction.** After Date, the author requested a “bigger and safer slice” covering
as many APIs/areas as possible, with separate commits.

**Assistant response.** Finish and commit Date first, then select existing APIs whose
contracts fit the checked importer. Preserve behavior and separate commits by area;
API redesign remains outside this migration work.


### 2026-09-15 — Namespace folders for library sources

**User direction.** The author specified that each runtime-library namespace should
have its own folder.

**Action.** Raven sources move under `src/System`, with subfolders matching declared
namespaces. Type files retain type names; namespace-function files use Functions.rvn.
Project inputs, generation source maps and documentation follow the new paths.
This is source organization only, without changing APIs or assembly partitioning.


### 2026-09-15 — Current folders do not settle future type ownership

**User observation.** The author noted that structure may change later and that
System.Char functions should probably belong to a ported System.Char struct.

**Assistant response.** Agreed to preserve that as a later structural change. Current
folders follow current declarations; the ongoing port does not redesign the public
Char type or make the function namespace permanent.


### 2026-09-15 — Port the fundamental structs as a family

**User direction.** The author requested as many fundamental types as possible,
noting that several have few or no members and should share the already ported logic.

**Assistant action.** Established a checked intrinsic-storage rule with Int64 and
extended it to ten numeric/Boolean structs. Kept existing APIs and comparison
behavior. Native-sized integer types retain IL while their operator/conversion
reference contracts are incomplete; the corresponding comparison compiles against
.NET with the same Raven compiler. No Raven compiler changes are included.

## 2026-09-15 — Minimal clock layer and public API preview

**Author direction.** Introduce Clock.Now returning Instant, a SystemClock backed by
system time, and Duration. Keep the initial layer small. The author then asked for
system-local rendering into LocalDateTime for the demo, and a website preview of
both this API and the Type/introspection versus TypeInfo/reflection boundary.
The author corrected the assistant's attempt to update an archived `.neo` sample:
current examples must use Raven.

**Proposal and refinement.** The author supplied the broader Time API v1 proposal
(Instant/Duration, civil time, separate offsets, zones, calendars and Period) and
asked that the implementation comply with its minimal subset. The assistant chose
signed 100 ns ticks, a narrow injectable clock and a provisional system-zone
conversion. To match the proposed civil-time boundary, LocalDateTime now contains
only Date and Time; the old static Clock.GetLocalNow and offset property are removed.
FixedClock is demonstrated as an application-defined test double, not a new library
class. The broader proposal remains future work.

**Actions.** Instant and Duration are authored in Raven; system-clock and local
conversion services use typed runtime calls. Raven examples demonstrate clock
substitution. Website excerpts come from executable Raven samples and distinguish
current development APIs from the published preview. See [the minimal contract](instant-clock.md)
and [the broader proposal](date-time-design.md). No Raven compiler or .NET target
changes are part of this slice.

### 2026-09-19 — Put TypeInfo in the member hierarchy, then address strings

- **Author:** explicitly selected “Make TypeInfo extend MemberInfo” after asking
  whether that relationship had already been implemented. This reopens the earlier
  closed Introspection story for that bounded change.
- **Assistant:** implemented the fourth sealed case and optional declaring-type
  ownership, preserving nested source ownership through metadata rather than names.
  Co-located the declarations to satisfy Raven's existing same-file rule.
- **Author:** directed the following order: “strings API - minimal API adjustments
  and behavior implementation”, then the feature page.
- **Assistant:** accepted separate commits in that order. String work and its page
  remain subsequent slices; this entry does not report them as completed.

- **Author, subsequent correction:** asked that APIs and samples favor union
  patterns/destructuring over GetSomeCase-style extraction and idiomatic case
  construction over explicit carrier wrapping. Requested a separate cleanup after
  this hierarchy slice and before strings.
- **Assistant:** updated this slice's owner access and accessor filtering to patterns,
  and used imported Some/None case constructors. Recorded the ongoing style rule in
  AGENTS.md; the wider cleanup remains the next commit.

### 2026-09-19 — Document idiomatic Raven and clean up before strings

- **Author:** requested a cleanup after the TypeInfo commit and before String work,
  then showed `val DeclaringType: Option<TypeInfo> => Some(StoredDeclaringType)` as
  the preferred property form. Asked to document idiomatic Raven conventions and
  look at the Raven repository for guidance.
- **Assistant:** read Raven's style guide, feature-meaning guide, property spec and
  pattern/Option/Result feature docs. Added a linked neoCLR conventions guide,
  distinguished target bootstrap ABI from ordinary user code, and applied the
  conventions to runtime properties and sample union usage.
- **Assistant:** asked whether the String slice should begin with strict UTF-8
  conversion or the larger proposed scalar-based Char redesign.
- **Author:** “Do what you believe benefits us right now”.
- **Assistant decision:** begin with strict UTF-8 conversion while preserving the
  current String/Char semantics; scalar Char remains deferred. This is a plan for
  the next slice, not a report of its implementation.

- **Validation finding:** the wider application checks exposed a pre-existing
  source-token mismatch for value-type constructors lowered to free functions.
  The assistant fixed the synthetic receiver token slot separately in `346678f`;
  the class-identity/value-copy sample now executes and the application suite passes.

- **Author:** clarified that `()` maps to void and that neoCLR-targeting Raven should
  use the `void` keyword rather than `unit`, at least for now.
- **Assistant:** verified that the runtime-unit mapping already selects System.Void,
  but the parser does not yet recognize lowercase void. Recorded the desired
  spelling and the implementation gap separately in the conventions guide.

- **Author, revised decision:** “Or we keep unit keyword as mapping to System.Void.
  For now.”
- **Assistant:** retained Raven's existing keyword and target configuration; no
  keyword compiler change was made. Updated the conventions while preserving the
  earlier suggestion and correction in this record. String work remains next.

## 2026-09-19 — Minimal String boundary API

The author asked to continue with strings after the introspection and idiomatic
Raven cleanup, leaving the feature page for the following slice. When asked whether
to start with strict UTF-8 conversion while retaining String/Char semantics, the
author replied, “Do what you believe benefits us right now”. The assistant chose
System.Text.Utf8 with Sequence<byte> and a typed invalid-input Result, preserving
Char's code-unit behavior and deferring Encoding and specialized Utf8String types.
The implementation and tradeoffs are recorded in [the String integration notes](raven-string-api.md).
The author's last source-keyword correction remains unit mapped to System.Void;
this slice makes no keyword change. A String feature page follows separately.

### Feature pages as implementation notes

The author emphasized that the API should stay minimal, demonstrate working
behavior and remain open to input. A feature page need not show everything and
should describe the current implementation, which may change. The assistant
shortened the String page to one executable conversion example, linked detailed
tests and documented provisional status. The author then proposed a separate
summary page for proposals, with feature-specific future-direction sections.
That overview is the following documentation slice, separate from shipped behavior.

The proposal overview is now implemented at `website/proposals/index.html`, with
links from the homepage and feature pages. It summarizes existing design records,
labels current foundations separately and records costs and unresolved questions.
String and Introspection pages now have short future-direction sections. This
changes documentation only and does not promote proposals into implemented APIs.

### Website upkeep and publication

The author asked to document the structure for later updates, keep the site aligned
with feature work and the product at release, and suggested manual publication
instead of publishing with pushes. The assistant documented the workflow in
[feature-page maintenance](design/feature-pages.md), linked it from contributor
instructions and AGENTS.md, and changed Pages deployment to manual dispatch on main.
Push and PR validation remain automatic. This is a local workflow change; no site
publication has been performed, and the remote policy changes only after integration.

### Native UTF-8 direction supersedes the compatibility carryover

The author clarified: “We don't want to use UTF-16 in NeoCLR”, directing work toward
native UTF-8 and following the String proposal. This supersedes the assistant's
earlier suggestion to preserve UTF-16 semantics. String storage was already UTF-8;
the assistant changed CompareOrdinal to native byte/scalar ordering and updated
samples and direction notes. Scalar Char is now the selected migration target,
not merely an optional proposal; its existing 16-bit representation remains an
explicit implementation gap requiring coordinated runtime, metadata and compiler
changes. This slice does not claim that migration has already happened.

### Further pages and the preview review

The author directed the next sequence: finish the String feature, then create the
other website feature pages, then review the whole preview to decide what it needs
to communicate the direction. The assistant added short notes for the existing
Option/Result, collection/query, date/clock and bounded-file APIs using tested source
excerpts. The release review distinguishes implemented paths, the remaining scalar
Char migration and packaging/validation gates from optional proposals. No new
release version, date or publication has been selected.

The assistant's [readiness review](preview-readiness-2026-09-19.md) identifies scalar
Char as the clearest remaining implementation gap before claiming the selected
native text model. It recommends keeping broad proposal families deferred and
prioritizing a pinned, validated evaluator bundle. A fresh local VS Code UTF-8
workspace runs and completes correctly. These are local findings, not a selected
release scope, version/date, cross-platform gate or publication.

### Scalar Char implementation begins

After the author asked to continue, the assistant began the outstanding scalar-Char
migration. The runtime now stores and validates 32-bit scalar values, uses four-byte
native storage and classifies supplementary characters from the pinned Unicode data.
The Neo frontend accepts supplementary literals. Raven's targeted lexer/literal
baseline passes (70 tests); the target-specific compiler and bridge integration is
next. This does not yet claim a complete Raven scalar-Char experience.

### Feature links and Raven language introduction

During the scalar-Char integration, the author asked for feature boxes to link to
feature pages instead of repository samples and requested a Raven language page.
The assistant updated homepage links, added the language introduction with existing
executable snippets, and recorded the navigation rule. Raven’s own README and style
guide supplied the language context; target limitations remain explicit.

The author then clarified that readers should find user content, including try-it
instructions, on the website and favored the .rvnproj approach. The assistant added
a self-contained Preview 7 installation and project guide, moved complete feature
sample downloads onto the site, and made repository references optional. Newer APIs
remain labeled as development work without a published matching evaluator bundle.

The author emphasized that neoCLR and its guest programs have no .NET dependency;
the dependency belongs to surrounding tooling, including Raven Language Server
used by VS Code. The assistant made this explicit on the homepage, Raven page and
setup guide, and recorded the distinction in the website maintenance rules.

### Raven scalar Char integration — 2026-09-19

On continuation, the assistant completed the target-specific compiler/bridge slice:
scalar literals, patterns, array storage and classification work in Raven programs.
Execution testing exposed 16-bit array admission and boxed supplementary patterns;
both were corrected. Invalid Int32 scalar conversions fault at runtime, and the
surrogate predicates were retired. Wide numeric casts still narrow before scalar
validation; scalar String access remains separate. General lexer candidates remain
unextracted pending independent main-based validation. This is local development
validation, not a published preview or a complete text proposal implementation.

### Characters as text units — 2026-09-19

While the assistant was implementing scalar String access, the author questioned
whether the abstraction had gone too far toward encoding and said a character
should mean what a developer sees as a character. The assistant distinguished
scalars from grapheme clusters and paused the scalar public API. Asked whether
ordinary length, indexing and iteration should use graphemes, the author answered
“Yes—graphemes by default”.

The author then emphasized encoding-independent String and character abstractions,
with UTF-8 as the canonical encoding. The assistant recommended String as Unicode
text, Char as one extended grapheme cluster, explicit scalar access and encoding at
byte boundaries. It revised its interim suggestion of returning String from indexing:
that is useful implementation scaffolding, not the intended Char abstraction.
Char representation, normalization/equality and scalar naming remain open.

A temporary grapheme implementation passed nine Rust String tests but was preserved
outside the checkout instead of integrating its provisional public API. Earlier
scalar implementation history is retained. The independent expression-bodied
indexer emission bug exposed during this work was reproduced on Raven main's
ordinary .NET target, fixed with class/struct execution tests (31 focused tests
passed), integrated into main and cherry-picked to neoclr. The temporary Raven fix
branch was removed. See [the revised text abstraction](design/text-abstraction.md).

### Implementing the revised character model — 2026-09-19

The author asked for a recommendation based on other languages and environments.
The assistant recommended Swift-inspired grapheme characters with explicit scalar
and encoding access, and refined its earlier indexing recommendation: provide
iteration first and defer integer indexing. The author directed “Make it so” and
then emphasized an improved .NET-like experience fitting modern computing.

The assistant implemented owned grapheme Char storage, String Length/iteration,
explicit uint scalar traversal and UnicodeScalar classification. A Raven program
now executes combining sequences, emoji, literal patterns, arrays and interface
iteration on neoCLR. Runtime testing exposed the need to count Char payload bytes
in array limits; end-to-end testing exposed String interface receiver adaptation.
Both were addressed in this work. The implementation keeps ordinal equality,
Unicode 16 segmentation and snapshot iteration explicit; normalization, cursors,
a scalar value type and host/target literal Unicode-version alignment remain open.
See [the contract and evidence](design/text-abstraction.md). This records local
implementation, not publication of a new preview or website deployment.

The author then explicitly affirmed that a character is not a number format and
that numeric casts should not exist merely because of its backing storage. The
assistant confirmed that the implementation rejects those casts and keeps Unicode
numeric access explicit through GetScalars. This is an API principle, not just an
implementation limitation or a temporary missing conversion.

The author subsequently suggested future UTF-8 and ASCII string types for
encoding-specific functionality, while preserving String as the default neutral
text container and Char as an encoding-independent character. The assistant
recorded Utf8String/AsciiString as possible future specialized types, not additions
to this minimal implementation. UTF-8 remains the canonical runtime storage choice.

### Next preview release request — 2026-09-19

After the grapheme implementation, the author directed that a release should follow.
The assistant selected Preview 8 and a fresh experimental Raven .15 toolchain,
continuing the previous release's source matrix and macOS arm64 binary scope.
The compiler, runtime and website text slices were committed separately. Release
preparation uses an isolated clean checkout so existing local edits are preserved.
At this point, publication was pending exact-candidate CI and extracted-package validation.

### Task contracts before dependent APIs — 2026-09-19

During Preview 8 preparation, the author selected Task and async state-machine
contracts as the first priority after release, with runtime suspension as a future
step. The author asked for a modern .NET-like developer experience without legacy
constraints, identifying ConfigureAwait as one area to reconsider. The author then
clarified: “The rationale is that we need the Task contract for upcoming APIs”.

The assistant proposed treating completion, continuation scheduling and logical
context flow as explicit contracts, aiming to avoid routine per-await boilerplate.
The assistant recorded the priority and rationale in the async design, assessment
and website proposals. This selects the order of work, not a final scheduling policy
or a shipped Task implementation. Runtime suspension and the behavior of cancellation,
UI affinity, cleanup and context propagation still require design and validation.

### Conventional query terms before async — 2026-09-19

The author subsequently directed that LINQ-style operators use conventional terms,
rather than inheriting .NET method names, and placed this work before the async model.
The assistant recorded that revised order and recommended Map/Filter for the current
Select/Where operations. FlatMap and Fold were examples for future semantic review,
not claims of newly implemented operators. The selected direction is recorded in
[API policy](api-policy.md#query-operator-naming-direction-2026-09-19); Preview 8 retains
its existing operator names until the separate migration slice is implemented and tested.

The author confirmed initial capitals and explicitly requested method renaming after
release. The author then supplied a ChatGPT formulation: “Prefer terminology that
has converged across modern languages for fundamental iterable operations; retain
.NET terminology where it is already broadly conventional or materially clearer.”
The author highlighted Where → Filter, Select → Map and SelectMany → FlatMap as
strong candidates, while rejecting an automatic preference for Fold over Reduce or
Drop over Skip. The assistant adopted that principle: the immediate implementation
scope is the existing Where/Select pair; other operators require separate semantics.

### Preview 8 publication — 2026-09-19

The assistant published [Preview 8](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.8)
at neoCLR `07ecdbe` with Raven `ddf10eca` (.15). All six source CI jobs passed
1,280 tests each, and all validated archives matched the published source files.
The extracted package, editor and notice checks passed; GitHub's asset digests
matched the eight local uploads. The isolated VS Code grapheme workspace is available
locally and its language client started successfully. See the
[validation record](preview-8-validation.md) for scope and limitations.

Release content and installation instructions were updated on the website for a
separate manual publication. The subsequent Filter/Map migration is a separate
post-release slice; it does not modify the published Preview 8 artifacts.

The Preview 8 website was published through the separate manual
[Pages run](https://github.com/marinasundstrom/neoCLR/actions/runs/35456186059),
and the live installation page was checked for the .8/.15 asset set.

### Query operator rename after publication — 2026-09-19

After Preview 8 was published, the assistant renamed Where to Filter and Select to
Map in the Raven-authored API, reference declarations and importer bindings.
The implementation retains System.Linq and the existing iterator bodies. First,
Last, Single and ToList keep their names; FlatMap is a future direction, not a new
operator in this slice. There are no legacy aliases, so callers must rebuild with
the matching development references and runtime library.

Samples and the website now distinguish the new development names from Preview 8.
The new on-site example is compiled by the query suite and prints 10 and 30.
Execution, stale-name rejection, signature admission and completion are covered;
see [the query validation record](raven-query-api.md#validation-after-the-naming-migration).
No Raven compiler change was needed. Task/async contracts remain the next foundation.

### A basic iterable set before Task work — 2026-09-19

The author asked, “Can we add additional operators before doing the task work?” and
clarified, “So we have the basic set”. The assistant proposed Any, All, Count, Take,
Skip, Concat and FlatMap, plus explicitly seeded accumulation. Asked to choose its
name, the author replied, “Fold for seeded accumulation”. This selects a bounded
library slice before Task/async contracts, not complete LINQ compatibility.

The assistant implemented the proposed operators in Raven, extended the neoCLR
bridge for the new signatures and nested open collection arguments, and added
execution/lifetime tests and a runnable sample. The [query contract](raven-query-api.md#basic-operators-before-task-contracts--2026-09-19)
records .NET/Rust comparisons, normal disposal guarantees and deferred work.
Validation results are recorded there when completed. No Raven compiler change or
new release artifact is implied by this development slice.

While this slice was being validated, the author requested a subsequent port of
Raven.Core's Option and Result operator methods, with documentation and website
updates, before Task work. The author also requested a .NET-to-neoCLR LINQ mapping
table and an operator list or demonstrations for Option/Result. The assistant added
the query mapping table and queued the outcome-operator port as the next separate
slice; this note does not claim that port is complete.
