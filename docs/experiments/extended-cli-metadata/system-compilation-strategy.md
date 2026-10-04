# Strategy: compile the entire Raven-authored System library

Author-directed reprioritization, 2026-10-04. This supersedes the API-by-API
time-zone continuation in earlier roadmap entries. Individual APIs are acceptance
consumers of shared capabilities, not the unit of prioritization.

## Current reassessment after Number (2026-10-04)

This section supersedes the original priority table and intermediate checkpoints below.
The [post-Number audit](system-post-number-audit-2026-10-04.json) uses compiler
073718ac4 (implementation 41b2573fa), metadata/runtime repository ea7fdf77 and the
proven numeric ownership profile. The accepted 70-source baseline compiles again.
Reuse the existing Number execution evidence; this audit adds compilation evidence,
not another execution or full dual-target gate.

**Priority 0 completed:** Raven 459856a71 bypasses conversion caching while source
relationships are provisional. Empty-first and reversed 70-source builds pass; the
76-source combination with Tasks/Concurrency compiles. Numeric (99), generic Number
(42), broad application (exact output, 0) and task (42) consumers execute from native
artifacts. The main-based backport f0c3b75a0 also includes the prerequisite interface-list
cache guards; it passes 139 focused .NET tests independently and is fast-forwarded to
main. The integration branch passes the same 139 tests. See [evidence](source-order-conversions-2026-10-04.json).
Priority 1 below is now the next feature batch. No runtime/metadata representation changed.

**Priority 1 text ownership completed:** the explicit service catalog and source-owned
Char/String now execute together through separate native consumers, with both text
declarations excluded from the seed. The unchanged grapheme, comparison, UTF-8 slice
and sequence-construction samples pass. Literal/equality/pattern lowering, interface
dispatch and native String constructors are implemented rather than projected through
consumer stubs. See [text gate and evidence](source-text-2026-10-04.md).
Next use this foundation for larger stream/storage/JSON source groups, with missing
runtime services and cross-assembly inheritance addressed at their owning layers.


**Original reproduction: source-order-dependent generic interface binding.** Prepending
an empty file (`// No declarations.`) to the accepted source set produces two RAV1504
errors assigning ArrayList<byte> to List<byte> in MemoryStream. The same baseline
without that file succeeds; appending the Self probe succeeds. Sequential controls
reproduce the difference. This initially isolated a compiler binding/identity investigation; the subsequent
regression identified provisional conversion caching, including on ordinary .NET.
Do not fix this by sorting production inputs or rewriting MemoryStream. Reduce it,
fix the owning layer and validate/backport independently on main if applicable.

The full 166-source attempt stops in binding with 416 diagnostics, including 279
missing-member diagnostics and 141 distinct missing RuntimeServices member names.
These are cascades and incomplete bootstrap coverage, not 416 compiler defects or
141 missing runtime implementations. Inspect existing native implementations and
catalog bindings before adding any implementation. The earlier offset-profile audit
is unsuitable for the new numeric seed: it lacks numeric ownership and Self setup.
The audit tool now accepts an explicit ownership profile, derives the output identity
from it, and includes an accepted-baseline control.

| Priority | Capability batch | Reach and bounded acceptance gate |
|---|---|---|
| 0 | Stable canonical primitive/generic interface binding | Fix the empty-file/order reproduction first. Compile the cumulative library with reordered inputs and combine it with Tasks/Concurrency; execute artifact-only numeric, collection and task consumers. Preserve ordinary .NET controls. |
| 1 | Source-owned String/Char and coherent text-service bindings | SliceUtf8 is a direct missing dependency in encodings, StreamReader, FileSystem and JsonDocument. Build the real sources with explicit intrinsic ownership/storage, reuse existing native text services through the checked catalog, then execute separate text/stream consumers. Include nominal API reimport and UTF-8 behavior; do not turn bootstrap declarations into fake implementations. |
| 2 | Ordinary class inheritance across assemblies | Enable base construction, inherited state, overrides and virtual dispatch through existing CLI-compatible metadata and explicit adapters. Test base/derived/consumer assemblies on both targets, then revisit JSON value and introspection descriptor sources. Current combined probes are masked by priority 0; source declaration validity is a separate concern. |
| 3 | Metadata handles and introspection service contracts | Unblock the introspection cluster and JSON object mapping with explicit dependency resolution, canonical type identity and typeof/handle operations. Keep the C# metadata facade and Raven runtime APIs separate; use real services, not runtime-reflection objects in the importer/emitter boundary. |
| 4 | Remaining service families and demonstrated callback/storage gaps | Expand storage/network services by common signature category. Reduce mutable-capture, array-address or broader generic failures only when reached by these families; test mutation, retained callbacks and identity. Do not implement every possible generic constraint first. |

Priority 1 is the next large feature story after the stability fix. Priority 2 can be
isolated without waiting for all text services. JSON should first exercise its document/
value layer before making all object-mapping/introspection services a prerequisite.
Network/HTTP follows the common services and storage/callback contracts it actually
requires. Re-rank from newly exposed first failures after each batch.

Number, numeric widths, floating emission, method-parameter Number dispatch, basic
Self, enums, inhabited unit and the existing task gate are completed bounded gates;
masked probes do not reopen them. They do not establish full source-owned Object,
String, Char, handles, wider inheritance or all generic constraints. The native broad
application already has execution evidence; the entire rebuilt library and equivalent
broad .NET source-library execution remain unproved. Keep focused .NET controls and
paired acceptance checks throughout, without replacing its Reflection/Emit backend.

Reproduce with audit_system_compilation.py --compiler ... --core ... --seed ...
--ownership <numeric-gate>/ownership.json --output <fresh-directory>, after generating
the matching artifacts with bootstrap/verify_native_numbers.py. All identities, hashes,
commands, control source text and first diagnostics are in the linked audit. Failed
family attempts published no artifacts. Existing output paths must not be reused.

## Evidence and limits

The fresh [audit](system-compilation-audit-2026-10-04.json) covers all 166 Raven source
files under System, twelve source-family compilation attempts, and six minimal
capability probes. It uses Raven bc30fb0f2 and the explicit Clock core/seed.
The existing native execution gate covers 48 source files. This is not a claim
that 29% of the implementation work is complete: large, connected areas remain outside it.

- Full-source compilation stops in binding: 563 error diagnostics, including 367
  missing-member errors. Diagnostic counts are not independent defect counts.
- Excluding primitive declarations still stops in binding. The bootstrap gap is
  not confined to replacing core types.
- Lexical inventory finds 178 distinct RuntimeServices member names across 48 files.
  IsValue and UnpackValue each appear in 23 source files. These are potential reach
  counts, not promises that implementing two operations compiles 23 files.
- A host-service-free MemoryStream source group binds, then rejects the
  Result<unit, StreamError> Flush signature. A separate minimal Result<unit, string>
  probe reproduces it. Selecting the existing RuntimeUnitContract for the configured
  System.Void did not fix the rejection; do not assume this is only configuration.
- Minimal ordinary-driver probes reject enum declarations, double arithmetic,
  class inheritance, mutable captured locals and Self. These tests isolate failures
  hidden by larger binding cascades.
- OptionOperators/ResultOperators compile in the audit. This is emission evidence
  only, not a new execution gate.
- No async/await keyword use was found in the audited sources after comment removal.
  Tasks are implemented with explicit queues, callbacks and state. A general compiler
  async rewrite is not justified as the next prerequisite.

Family sets are diagnostic groupings over the accepted source baseline, not complete
dependency-closed production assemblies. Some include absent dependencies intentionally
to identify the frontier. Failures in binding mask later emission/metadata/runtime
failures; no rejected compilation was executed. The full source input is unchanged.
The audit does not make unsupported bootstrap references into implementations.

## Prioritized work

| Order | Shared work | Why it has leverage | Completion gate |
|---|---|---|---|
| 1 | Complete inhabited-unit and erased-value contracts | Unit occurs in 15 source files, including streams, encoders, JSON and HTTP. IsValue/UnpackValue span 23 files and underpin native outcome decoding. | Independently compiled MemoryStream/OutputStream with Result<unit, E>; typed native value outcomes round-trip success/error for multiple primitive kinds. |
| 2 | Replace incremental service wiring with a checked service ABI catalog | Missing runtime members dominate the full binding frontier and recur across 48 files. The existing bridge has useful signature and adapter knowledge that should be migrated, not rediscovered. | One catalog drives matching declarations/bindings/validation; at least text, I/O and task-service families compile through the ordinary driver and execute real host calls. Unsupported signatures reject explicitly. |
| 3 | Close reusable metadata/emission shapes | Enums block TaskState, EntryKind, StringComparison, BindingFlags and HttpStatusCode. Inheritance is required by JSON value classes and HTTP handlers. Wider numeric support blocks core/numeric and JSON work. | Separate library/consumer tests for enum storage/import, derived construction and virtual calls, and signed/unsigned/floating signatures and operations, with .NET controls. |
| 4 | Finish source-owned core identity and Self/static numeric contracts | Compiling applications against intrinsic primitives is not the same as compiling their member implementations. Number uses Self and static members; the full audit shows cascaded failures across ten numeric implementations. | A staged source-built core subset retains canonical primitive identity/storage, imports native Self relationships and implements Number; no seed/source duplicates or recursive ordinary fields for intrinsic storage. |
| 5 | Expand callback, storage and generic lowering only from cross-family failures | Task queues, cancellation, HTTP and JSON depend on callbacks, shared storage and generic contracts. Mutable captures and array-element receiver addresses have known gaps. | Observable retained callbacks with mutation/lifetime/identity; ref/out/array-address behavior; imported generic constraints/substitution needed by these source families. |
| 6 | Complete typeof/introspection/opaque-handle integration | Object/Introspection and JSON object mapping form a dependency cluster. A new runtime reflection facade is not needed: use the existing metadata/context model and explicit target contracts. | Source-built introspection contracts plus separate JSON consumer preserve identity and metadata resolution; no translated application/rebuilt-library references. |

Orders 1–2 form the first implementation batch. Within order 3, do enums first,
then rank numeric and inheritance work by the newly exposed blocked families.
Design the ownership model for order 4 during order 2; do not postpone discovering
core identity conflicts until every host service is wired. Do not treat this table
as permission for a wholesale backend rewrite.

### Batch 1: concrete next work

1. Reduce and fix the existing unit signature/representation mismatch through
   signatures, generic arguments, values, imports and no-result calls. Keep the
   language's inhabited value separate from CLI void returns. The MemoryStream
   source group is the first gate because it needs no new host service.
2. Specify System.Value as the existing native erased carrier at the compiler/metadata
   boundary; migrate the already implemented IsValue/UnpackValue intrinsic behavior.
   Test correct kinds, wrong-kind failures and identity without silently substituting
   CLR object boxing. Keep target policy explicit.
3. Inventory and classify existing service signatures: scalar, no-result/unit,
   erased outcome, vector, callback, opaque handle and source-owned nominal return.
   Reuse RuntimeServiceBindings and native runtime declarations as evidence. Separate
   real host services from bridge aliases that merely call source-owned factories.
4. Generate or validate matching primitive declarations and executable bindings
   from that catalog, with exact identities and failure-before-publication checks.
   Start with signature categories used by the most source families. Do not first
   hand-author 178 individual wrappers, and do not count declaration-only stubs as success.
5. Re-run the audit, group new first failures by common cause, and pick the next
   category using the rule below. Continue ordinary .NET regressions after shared changes.

## Ownership and architectural constraints

Maintain a single full-library declaration ownership plan. Each declaration has one
owner: intrinsic bootstrap, retained runtime implementation, or source-built library.
A migration must update signatures, binding, emitted references and linking together.
The current System.Runtime.rvnproj is a legacy per-slice bootstrap project; its default
source selection is not a full native System build. A production native build manifest
must represent the entire intended source set and its explicit prerequisites.

Readers -> introspection -> symbols and symbols -> emitter -> builders -> definitions
-> metadata -> PE remain separate boundaries. Do not reopen importer objects at emission.
Keep Reflection/Emit for ordinary .NET. General fixes should be isolated and validated
on main; fixes to portable contracts absent from main remain recorded integration work.
Existing Self infrastructure should be connected and completed, not replaced with a
new semantic design. Structural Function experimentation remains out of this strategy.

Use CLI-compatible signature/encoding and behavior where applicable: enums, inheritance,
numeric instructions and calling conventions. NeoCLR unit, erased Value, primitive
storage and Self differences need explicit mappings and cost/compatibility notes.
Re-use existing platform/bridge research for these contracts. No format fork, new
reflection replacement, new API design or performance claim is required for this batch.

## Selection rule and scorecard

Prioritize a fix by the number of independently blocked source families it advances,
its dependency reach and the confidence/cost of the change. File counts are supporting
evidence; do not invent exact unlock estimates from lexical counts or cascaded errors.
A small isolated API is worth doing when it is the shortest representative test of a
shared capability, not simply because it is easy to add.

For each family record separately:

1. Binding with explicit dependency identities.
2. Emission and metadata verification.
3. Reimport into a consumer with library sources absent.
4. Runtime execution and semantic checks.
5. Ordinary .NET regression/control status.

Report changes in those stages after each capability batch. Do not report a reduced
diagnostic count, successful declaration projection, larger instruction budget, or
increased accepted file count as equivalent to completion.

Keep one cumulative native gate, plus focused capability tests. Use the broad
application for integration and MemoryStream, text/encoding, task queue and JSON
consumers for complementary coverage. Re-run unchanged suites only when impacted.
The seven native consumers and metadata test baseline remain controls.

## Deferred work and maintenance

- Individual time-zone service wiring is no longer the next priority. It should
  fall out of the common scalar/vector service categories.
- Full async/exception features are not prerequisites without a source requirement.
- Full .NET execution of the rebuilt calendar/core subset remains unproved; preserve
  .NET controls rather than assuming native success establishes parity.
- The legacy translated snapshot refresh fails in SourceUnionReferences.Project
  with 'None' not in scope; the Instant digest is stale. Keep this visible as a
  separate reproduction/maintenance task. Do not substitute that bridge for native
  imports or make fixing all legacy tooling a prerequisite for native progress.
- No compiler/runtime feature was implemented in this assessment.

## Reproduce

Run audit_system_compilation.py with --compiler, --core, --seed and a fresh --output
directory. Use the matching Clock comparer core/seed documented in bootstrap/README.md.
The JSON records commands, source hashes, revisions, errors and output publication.
Minimal probes are embedded in the tool and materialized in the audit directory.
The checked-in evidence also records the additional explicit-unit-contract experiment.


## Batch 1 implementation evidence, 2026-10-04

Inhabited unit is now admitted through the portable signature/local planner using the
explicit existing RuntimeUnitContract. Generic unit returns retain their physical
value result; ordinary unit-returning calls remain no-result. Five unchanged stream
sources compile separately and their imported consumer executes byte mutation, shared
interface/cursor identity and Flush success/error. A separate generic-unit consumer
covers values, arrays and discarded returns. See bootstrap/verify_source_streams.py
and bootstrap/README.md. 38 focused C#/.NET controls pass.

The cumulative 53-source single PE now reaches the schema-2 1 MiB encoding limit after
binding/emission. It does not publish output. The supported separate-library workflow
passes; whole-library container sizing remains an additional shared blocker. Erased
Value and the common ABI catalog are still pending; the first batch is not complete.


The erased-value subgate now executes a separately imported generic wrapper around
real ParseInt64 outcomes: Int64 success, Byte invalid-format/overflow, kind tests and
wrong-kind unpack rejection. Metadata encoding/reimport retains the exact bound core
System.Value identity. The cumulative native application/callback gate passes after
fixing callback/discard result conventions exposed by inhabited-unit admission. This
proves two existing payload kinds, not universal erasure. The common service ABI catalog
is still pending; proceed there before adding individual service families.


## Checked service catalog and updated frontier (2026-10-04)

The existing RuntimeServiceBindings signature inventory now drives native core
declarations and generated seed wrappers for an explicit 20-service profile, plus the
two generic erased-value helpers. `Probe --native-service-catalog` emits classified
inventory and executable wrappers; checked generation rejects unknown, duplicate,
unsupported numeric/nominal or completion-callback selections. C# import validates all
22 declarations against the real seed. It does not claim every inventory entry is ready.

Nine unchanged System.IO/System.Text files compile as another native library. The
source-free consumer executes UTF-8 encode/decode/invalid input and file create/write/
flush/read/position, with an independently checked two-byte file. A separate contract
library executes a string worker callback and consumes its erased result. Full source
Tasks/Workers is **not** complete: source-owned TaskQueue services and the inhabited-Void
versus no-result completion callback boundary remain explicit gaps.

The [updated audit](system-compilation-catalog-audit-2026-10-04.json) keeps the same
166-source inventory and dependency-closed baseline. The unit probe now emits. The
single-PE memory-stream group reaches the 1 MiB transport limit instead of rejecting its
unit signature. Full-source binding still fails (573 diagnostics, 305 missing-member
errors); more visible signatures expose new cascades, so total counts are not a progress
score. Text still lacks core String.SliceUtf8; tasks first lack queue ownership/services.
Float/enum/inheritance/Self/mutable-capture probes retain their previously isolated gaps.

Next high-leverage work: admit the already specified bounded library transport profile
for API-produced PE assemblies, then connect source-owned queue identities and exact
completion-callback conventions. Enums remain the first reusable emission category;
TaskState and broader core/numeric ownership cannot be bypassed with substitute classes.
Keep no-result/inhabited-unit distinctions explicit in the catalog, shared codegen and
runtime. Do not resume isolated API wrappers or interpret this profile as full-System
or full dual-target completion.


The single-PE transport blocker is closed for the demonstrated subset: schema-3 library
PE writing/loading admits the 53-source combined library, and separate stream/broad
consumers execute. A >4 MiB API-produced library also imports into Raven and executes.
Schema-1/2 limits and declaration budgets remain unchanged. The 57-source combined
IO/UTF-8 experiment now exposes a source-owned Array<T> interface projection/binding
failure; these same sources work in separate libraries. Investigate declaration-readiness
caching before expanding more service APIs. See bootstrap/verify_combined_library.py.

## Combined array binding closure (2026-10-04)

The 57-source `verify_combined_library.py --services` gate now passes, building all
selected sources together and then importing only that library into separate
consumers. UTF-8/file mutation and unchanged broad application output execute.
[Recorded commands, revisions and hashes](combined-library-array-2026-10-04.json)
identify the tested bundle. Compiler fix `5dda5c5de` was independently reproduced
and validated on main, where it is integrated as `340fdb759`. Integration validation
passes 65 focused compiler tests and seven native consumers; the independent
main-based iteration suite passes 22 cases.

The blocker was premature caching of absent source-owned array shapes during
declarations, followed by empty interface projection. Raven now defers these cache
entries while declarations are provisional. CLI representation and native runtime
semantics are unchanged; arrays remain backed by nominal Array<T>. The shared
.NET regression uses imported array member access in an inferred static initializer
followed by an interface conversion in a method. Direct interface conversion in the
early initializer itself remains a separate declaration-binding gap.

Next capability batch remains queue/callback ABI ownership and reusable enums;
this result is not proof of full System compilation or dual-target class-library
execution. No guest public API changed, so the existing API snapshot maintenance
blocker is unchanged.

### Task scheduling ABI slice (2026-10-04)

ScheduleTask now accepts the existing inhabited-unit callback and an explicit
no-result callback. The primitive catalog selects the latter for CLI Action and
adds explicit entry draining. A helper library/consumer proves retained receiver
mutation with exit 42, with the 57-source combined gate still passing. This is a
callback ABI subgate, not completion of source Tasks/Workers. Next implement enum
metadata/emission for TaskState, then canonical source-owned queue service bindings.
Worker notifications remain unselected because their queue-post convention is still
legacy. Runtime signature rejection tests and guest-work execution cover both
existing scheduling behavior and the new no-result path.

### Reusable enum slice (2026-10-04)

Ordinary Int32 enum authoring/import/emission now passes the paired driver gate.
Unchanged TaskState is compiled with a helper library and consumed without its sources;
both targets return 42 after field mutation, enum arrays, comparisons and an unnamed
integer round trip. The C# metadata API also emits a CLI enum that executes under .NET
and a native library/consumer that executes in neoCLR. This retains standard CLI enum
metadata and the existing native nominal enum storage; no format fork is introduced.
See [recorded execution](enum-dual-2026-10-04.json) and the public metadata API reference.
This closes the reusable enum prerequisite, not source Tasks/Workers. Next resolve
source-owned TaskQueue service identities and runtime queue dispatch, then compile the
actual Tasks/Workers sources and execute their separate consumers.

Compiler slice: Raven `8407c122f`; metadata/runtime changes are committed with this
evidence. Validation: 132 metadata groups, 57 focused Raven tests, nine Rust enum
tests, the seven native consumer controls and paired driver execution pass.


## Tasks/Concurrency batch completed (2026-10-04)

[Separate-library execution](tasks-native-2026-10-04.json) now covers the six actual
Tasks/Concurrency files on top of the 57-source base. It closes source-owned queue
selection, constructed/interface callback binding, inhabited-unit callbacks and the
nullable/union patterns required by cancellation. Worker/task legacy controls remain
part of validation. No new native function-type format was required.

The [refreshed inventory](system-tasks-audit-2026-10-04.json) still stops the full
166-source build in binding. Its tasks-family case was rerun successfully after the
pattern fix. Prioritize the next shared blockers in this order:

1. Canonical primitive/Self ownership and numeric source contracts: separate bootstrap
   identity failures from genuine missing operators; numeric, text and namespace
   families share these failures. Add minimal source/import controls before expanding.
2. Extend the checked runtime service boundary by coherent source families (numeric
   parsing/text before storage/network); missing services dominate the full-source
   diagnostics. Reach counts do not prove these are the only blockers.
3. Broader inheritance and float emission where the first two batches expose them.
4. Metadata-only typeof/introspection facts and their service boundary; JSON depends on
   this layer, so individual JSON wrappers are premature.

Full async lowering, mutable captures and broad introspection remain explicit open
capabilities. Preserve the default .NET backend and validate shared fixes independently.


### Native Self conformance slice (2026-10-04)

The driver can select its existing semantic Self contract explicitly; native import
and emission preserve Self through the metadata facade. Actual Clonable now participates
in a three-assembly executable gate with concrete clone calls. Metadata implementation
matching substitutes Self with the declaring class/value owner, including generic owners.

The numeric-family binding audit with this configuration removes the reported unresolved
Self and Number implementation errors. It still fails on missing ParseBoolean/ParseByte/
ParseDouble/ParseInt16/ParseInt32 and related service contracts, plus resulting payload/type
errors. These are not all independent defects. Next connect a coherent numeric parsing
service family, then re-assess primitive ownership and static Number emission from the
remaining first failures. Do not treat this concrete-call gate as constrained generic
Self dispatch support. [Evidence](self-native-2026-10-04.json).


### Parsing service family and primitive selection (2026-10-04)

[The parsing gate](parsing-native-2026-10-04.json) connects all eleven existing parsers
through the checked catalog and executes the actual Boolean source library. This exposed
an unconditional System.Runtime primitive preference in Raven's shared CLI contract;
explicit metadata-core selection now takes precedence. The fix independently fails/passes
on .NET main with competing core references and is integrated as f749c1a75 (20 focused
tests). The seven native consumers remain passing.

The numeric-family audit drops from 43 to 33 missing-service diagnostics, 13 to 3
out-of-scope diagnostics, and 21 to 2 unsupported-operator diagnostics after this batch.
These counts describe cascading diagnostics, not independent defects or completion
percentages. Remaining first failures include character/string adapters, integer display,
IntPtr operations and other RuntimeServices members. Follow with the common primitive
signature/storage capabilities and static Number/Self contracts needed by the selected
numeric sources, using coherent service families rather than isolated API wrappers.
The core bool identity and native source System.Boolean declaration are still distinct;
full primitive-source ownership and instance dispatch remain explicit work.


## Floating-point writer subgate (2026-10-04)

The metadata API now supports Single/Double signatures, exact-bit literals, arithmetic
and unchecked numeric conversions. The same authored graph executes on .NET and
neoCLR with exit 42, checking NaN and signed zero. This reuses CLI signatures/opcodes
and the existing native runtime representation; high-bit Double literals select the
already-supported schema-3 UInt64 payload profile. See
[floating-point evidence](floating-metadata-2026-10-04.json).

This removes a writer prerequisite, not the Raven emission blocker. Next connect
portable primitive identities, literal/conversion lowering and both target adapters;
retain correct unordered comparison behavior. Then compile a separate floating
library/consumer and consume successful Single/Double parser payloads. Static Number
contracts and canonical source-owned primitive declarations remain separate gaps.


## Floating-point compiler integration gate (2026-10-04)

The follow-on Raven slice now passes separate-library import and execution on both
.NET and NeoCLR: Single/Double literals, casts, arithmetic, unary minus, generic
calls, fields, properties, arrays, signed zero and ordered/unordered comparisons.
A native-only consumer also unpacks successful ParseSingle/ParseDouble service
results and passes those values into the emitted library. Seven prior native
consumers remain green. Run `bootstrap/verify_floating.py` with the explicit core,
seed, base library and ownership manifest; it records commands, revisions and hashes.
See [execution evidence](floating-dual-2026-10-04.json).

This closes the audit's floating arithmetic emission capability. It does not compile
the actual Single/Double source declarations or implement static Number/Self dispatch.
Revisit those static contracts and canonical source primitive ownership next; missing
formatting/comparison services remain explicit catalog work. The primitive floating
unary binding correction is independently isolated on Raven's main-based fix branch.


## Number integration checkpoint (2026-10-04)

The author clarified the active story as the entire numeric family and Number interface.
The unchanged Number source now emits separately with static Zero/One/operators and
external ComparableTo<Self> inheritance. A separate Scalar struct and source-free
consumer execute direct arithmetic, identities and ordering. This fixture tests the
contract; it does not stand in for the ten primitive numeric sources. Metadata C# tests
also execute CLI constrained static dispatch and validate detached/builder parity and
static-vs-instance implementation rejection. See `bootstrap/verify_number_contracts.py`.

Actual Single.rvn advances past unsupported operator declarations to an ownership error:
`missing public interface implementation: get_Zero`. Binding already substitutes native
Self with the selected canonical primitive; metadata conformance currently substitutes
the ordinary nominal source owner. Do not resolve this by accepting arbitrary wrong
return types. Next establish explicit intrinsic primitive declarations/storage and
canonical import ownership, followed by the remaining numeric widths, checked runtime
service selection and generic Number callself emission. Verify the existing complete
numeric consumer across all ten types, parsing boundaries and NaN ordering before
claiming this story complete. Preserve the CLI primitive bootstrap explicitly while
source ownership transitions; duplicate seed implementations must be removed.


### Number prerequisite: integer width emission (2026-10-04)

All eight fixed-width integer signatures and unsigned instruction selection now
execute through separately compiled libraries and consumers on both targets.
[Evidence](integer-dual-2026-10-04.json). Standard CLI encodings and runtime operations
are reused, including sign/zero extension and unsigned comparisons; this introduces
no new numeric semantics. The primitive local API also admits floating locals.
The current blocking Number issue remains explicit canonical primitive ownership:
source Single's Self maps to float while its emitted declaring type is nominal.
Do not relax implementation signature checking to hide it. Generic Number dispatch
and intrinsic backing-field storage follow the same ownership decision.

Validation: 137 metadata C# groups, the paired integer and floating driver gates,
31 focused .NET controls, and all seven existing native consumers pass. API manual
updated; `build-api-docs.py --check` still reports the known stale guest snapshot.
No guest public API changed, and no snapshot was regenerated against a mismatched bridge.


### Number prerequisite: explicit scalar declarations (2026-10-04)

The C# metadata API now admits an explicit NativePrimitive designation for canonical
numeric definitions. Builder and manual-definition paths share validation; native
reading, introspection and imported methods preserve the scalar receiver. An independent
library/consumer executes receiver mutation and returns 42. Runtime representation and
instructions already existed; no runtime or format extension was necessary. Ordinary
System-named definitions do not implicitly acquire scalar ownership. Executable CLI
output rejects this native ownership, while reference-only projection remains possible.

Raven still needs an explicit primitive ownership catalog shared by special-type
resolution and emission. Its existing bootstrap ownership validator intentionally
rejects duplicate declarations: simply adding Single to the source list while retaining
a canonical CLI primitive declaration is not a solution. A checked source `m_value`
intrinsic mapping must emit receiver loads/stores, not record fields. The metadata API
does not silently discard fields. Generic Number-constrained method bounds and callself
emission remain subsequent work. These distinctions preserve the importer/emitter
boundary and avoid teaching ordinary .NET emission about native primitive ownership.

Validation for scalar declarations: 138 metadata groups pass. The saved independent
consumer verifies and runs with exit 42 and no stdout/stderr; see
[commands and hashes](primitive-declarations-2026-10-04.json). The existing stale API
snapshot check remains unchanged; the C# development API manual is updated. Integer
compiler support is Raven 17d7c50f5 with neoCLR fe2c263c.


### Source floating primitive ownership (2026-10-04)

Unchanged Single.rvn, Double.rvn and NumberParseError.rvn now compile into a native
library and execute through a separate artifact-only consumer. The explicit ownership
manifest selects nativePrimitives by canonical name and owning library. Raven retains
the declared CLI bootstrap while compiling the provider, checks its sole private
mutable m_value field, and emits scalar receiver operations rather than record storage.
Consumers select native declarations consistently for keyword, namespace and metadata
name lookup. Missing providers fail before publication. Emission authors references
from symbols and host dependency identities/digests, independent of reader objects.
The metadata writer preserves canonical scalar dependency names for authored references.

This matches the CLR distinction between primitive signatures/storage and ordinary
value types; allowing an explicitly selected external native provider is a NeoCLR host
configuration, not a .NET replacement rule. No runtime instruction or schema changed.
Generic Number-constrained calls, integer source providers and complete numeric-family
acceptance remain open. Run bootstrap/verify_native_floating.py with explicit compiler,
runtime, core, seed, base-library, number-contracts and ownership artifacts. It verifies
parsing payloads/errors, NaN ordering, scalar methods, identities and arrays (exit 42,
empty stdout), and failed missing-provider publication. See native-floating-2026-10-04.json.
38 focused .NET metadata-import/Self/operator/interface controls and 138 C# metadata
contract groups pass. The guest API snapshot remains the recorded stale artifact;
this change updates the development C# API manual and introduces no guest API.

Compiler implementation: Raven `da502f04a`; metadata changes are the matching native
floating ownership slice above neoCLR `a37fa6b9`. The paired floating .NET/native
control also passes with provider selection disabled. No independent main backport is
required: these changes are explicit native target integration, not general binder fixes.


### All numeric source implementations (2026-10-04)

The cumulative source-library subset now builds with unchanged Number, all ten numeric
structs, NumberParseError and IntegerDivisionError under one native owner. A separate
consumer imports that artifact without sources and executes parsing boundaries/errors,
identities, ordering, integer formatting, checked division and floating arrays/NaN cases
(exit 99, empty stdout). This goes beyond compiling declarations; runtime linking and
verification succeed without seed-owned Int32/Int64. Generic Number-constrained calls
remain open, so this is not the completion of the Number story.

Replacing seed primitive owners requires rebuilding their existing consumers. Loading
the old collections artifact alongside a new numeric provider correctly rejects its
undeclared dependency. The fixture rebuilds the actual cumulative source subset rather
than relaxing direct-reference rules. Provider emission maps exact bootstrap member
contracts to output-owned source methods. Object remains the bootstrap anchor.

Primitive members now retain canonical runtime names (System.Int32.ToString, etc.).
Native readers also accept the earlier encoded member names and preserve accessor
associations. Numeric interface matching can use the metadata declaration name when
an interface executable name is encoded; signature and accessibility checks remain.
This preserves the CLR-like distinction between declaration identity and executable
representation without introducing a new runtime instruction or metadata schema.
The checked service catalog adds existing Int32ToString/Int64ToString bindings, and
numeric-seed.neoil retains no duplicate numeric declarations.

See numeric-source-family-2026-10-04.json and bootstrap/verify_native_numbers.py for
commands, hashes and source ownership. The metadata C# suite covers canonical and
legacy names/accessors; focused runtime interface tests cover semantic name matching
and inaccessible implementation rejection. The stale guest API snapshot is unchanged;
no guest public signature changed.

Compiler implementation: Raven `d92e2bdc9` over `da502f04a`; matching runtime/metadata
slice is above neoCLR `6fe30a80`. Validation: 138 C# metadata groups, 43 focused runtime
interface tests, seven existing native consumers, and the source-family gate pass.
The earlier 38 focused .NET controls and paired floating control remain unchanged.


### Static constrained-call prerequisite (2026-10-04)

The reduced Raven repro `Sum<T>(left: T, right: T) -> T where T: Number => left + right`
binds successfully against the rebuilt numeric library, then rejects with NEOMETA001
at callable-declaration admission. Inspection found two independent gaps: the portable
callable filter rejects method constraints, and the metadata IL generator exposed only
concrete instance constrained calls. Do not relax the filter until bounds are preserved.

The latter gap is now closed. The existing CallConstrained operation also accepts owned
static interface contracts on owned nongeneric class/value implementations. Raw Emit
uses Call for static and Callvirt for instance methods. Both use the same validation
path. Self substitutes the implementing type in stack arguments/results. CLI uses its
standard constrained./call pair; native uses existing nonborrowed callself. No runtime
or schema change was necessary. The C# suite executes .NET dispatch; ordinary and
primitive-Self native artifacts both verify/run with exit 42 and empty stdout.

The next slice is method generic bounds across definitions/builders, CLI GenericParam
and GenericParamConstraint, native function constraints, reader snapshots and introspection.
Then admit those semantic bounds through the explicit NeoCLR capability and lower
static interface calls with a method-parameter implementing type. Imported bounds and
reference-only generic consumers must be tested before claiming generic Number support.
The importer/emitter boundary and ordinary .NET backend remain unchanged. Baselines:
138 metadata groups and 20 focused Raven Self/static-interface tests pass. See
static-constrained-2026-10-04.json for artifact hashes and executable evidence.

Compiler implementation remains Raven `d92e2bdc9`; matching compiler direction note
is `3787f15fb`. This metadata slice is based on neoCLR `907ccff7`. No compiler fix or
main backport is claimed. The guest API snapshot check still reports its known stale
state; the C# development API manual covers the changed generator contract.


### Method-bound preservation checkpoint (2026-10-04)

Owned nongeneric method interface bounds now survive builder/definition authoring,
standard CLI GenericParamConstraint rows, native function TypeBound records, snapshots
and introspection. Raven obtains canonical constraint symbols through the facade,
including inherited interfaces, and its existing binder rejects incompatible arguments.
The focused probe also confirms constrained-call emission fails before publishing bytes.
No importer objects are reused by the emitter and no Reflection/Emit behavior changes.

C# validation covers CLI execution and native/legacy projection round trips. The
API-generated native bounded method verifies and returns 42; seven existing native
consumers still execute. See method-interface-bounds-2026-10-04.json. The next bounded
work is external interface bound identity, then open constrained-call operands and
semantic authoring in Raven's emitter. Keep its constrained callable rejection until
these contracts are complete; this checkpoint does not claim generic Number execution.


### External method bounds checkpoint (2026-10-04)

Method constraints now retain external nongeneric interface identities through standard
CLI TypeRefs, existing native TypeBound records and reader snapshots. Introspection
resolves only through its explicit catalog and rejects missing/wrong dependencies and
noninterface definitions. The experimental constraint record now carries TypeReference;
local definition authoring remains supported. No runtime schema change was necessary.

Both targets execute the API-authored separate-contract bounded-method fixture with
exit 42. Raven's direct importer preserves canonical external bounds and its existing
binder rejects incompatible arguments. Seven native consumers remain passing. See
external-method-bounds-2026-10-04.json. The legacy reference-only projection explicitly
rejects external bounds; it is not a fallback. Primitive bootstrap and runtime seed
configuration are unchanged. Open constrained calls, imported bounded-method authoring
and Raven's constrained emission admission remain the next implementation work.


### Open constrained method calls (2026-10-04)

The metadata IL generator can call owned static interface declarations through bounded
method parameters. Typed and raw overloads validate the method scope and direct/inherited
local interface bound. Self is substituted with the method parameter in stack signatures.
Standard CLI constrained./call and native callself already encode the operation; no
runtime changes were needed. API-generated ordinary and native Self/Double fixtures
verify and execute with exit 42. C# metadata checks pass 140/140 groups.

See open-constrained-methods-2026-10-04.json. The next work is external interface target
operands, followed by Raven semantic constraint authoring and open-call lowering behind
an explicit capability. Its CallableSignature admission still rejects method constraints;
do not relax that guard until the emitter can retain bounds and dispatch correctly.
The .NET backend, explicit primitive bootstrap and runtime seed selection are unchanged.
The C# API manual is updated; the existing stale guest API snapshot remains unresolved.


### Generic Number end-to-end gate completed (2026-10-04)

The Number feature now passes through ordinary compiler commands with direct native
references. The gate rebuilds the cumulative class-library subset plus all ten unchanged
numeric sources as Numbers.dll, compiles native-number-algorithms.rvn as a separate
library, and compiles native-number-generic-consumer.rvn with only emitted references.
NeoCLR verifies and executes all ten instantiations, checking +, -, *, /, Zero, One,
inherited ComparableTo<Self>.CompareTo and constrained generic forwarding. Exit 42 and
empty stdout are required. The earlier parsing/boundary/array consumer still returns 99.
A string type argument fails binding with RAV0320 and publishes no output.

Method bounds are emitted from semantic symbols through an explicit native capability;
the importer remains independent. Shared linear calls carry compiler-owned method/type
operands. The native adapter uses metadata IL-generator calls, with managed addresses for
constructed instance dispatch. External numeric ownership and interface conversions are
explicit semantic facts. The default .NET Reflection/Emit implementation is unchanged.
CLI uses its existing constraint tables and constrained instructions; native uses existing
TypeBound/callself behavior. No runtime schema fork, new arithmetic semantics or performance
claim was needed. No independently useful binder fix was introduced for main backport.

Run bootstrap/verify_native_numbers.py with the same explicit primitive core, numeric
runtime seed and cumulative ownership manifest as the preceding source-family gate.
It records source/artifact hashes, revisions, commands and failures. See
number-generic-end-to-end-2026-10-04.json for this run's evidence. This completes the
Number numeric-family story; it does not claim full class-library compilation or broader
generic constraint categories. Generic owner-parameter forwarding, special method bounds,
structural Function experiments and replacing .NET Reflection/Emit remain separate work.

Compiler implementation: Raven `41b2573fa`, with matching metadata changes above `812417c7`.


### Explicit accessor follow-up (2026-10-04)

The source String Count-property emission rejection is repaired through actual interface
mappings. The cumulative source attempt now reaches String.Concat definite-assignment
validation. Resolve that shared lowering/emission boundary next, then establish primitive
String/Char ownership and execute artifact-only consumers. See
[the accessor slice](explicit-properties-2026-10-04.md) for limits and independent .NET fix.


### String source emission checkpoint (2026-10-04)

The next pattern-expression lowering fix (Raven aa2a7ec69) removes the String.Concat
assignment rejection without weakening metadata validation. The cumulative build including
unchanged String and Char now emits and passes runtime verification (654 IL functions).
An artifact-only consumer calls source String.CompareOrdinal and String.Concat and exits 42.
A separate paired three-assembly pattern test observes both null and non-null branches,
returning 42 on each target. [Evidence](pattern-expressions-native-2026-10-04.json).

Next establish canonical String/Char ownership and instance storage/dispatch; static source
methods do not establish those. The existing ownership manifest still describes the earlier
numeric subset. Native nullable reference annotations are also not preserved across imports:
a direct null argument to an imported Cell? parameter rejects RAV1503/RAV1509. This is a
separate metadata/import fact gap; the pattern test exercises null within the producer.
Public-plus-explicit same-name property binding remains a general compiler candidate.
The independent .NET setter fix is validated on main at 210d891e0 (seven focused tests).
Portable lowering does not exist on main yet, so aa2a7ec69 has no independent main backport.


### Runtime-owned String metadata prerequisite (2026-10-04)

The instance consumer confirms that Char still resolves to the bootstrap declaration
(FromString is absent). The existing native primitive provider configuration accepts only
numeric value types. Do not broaden Char to a numeric/CLI-char signature: its runtime
representation is a grapheme.

The metadata prerequisite now accepts an explicitly designated System.String reference
owner. Native local and external instance calls execute with a direct reference receiver;
142 C# metadata groups pass. A minimal empty System seed prevents duplicate ownership in
this API-level test. No Raven compiler selection or full class-library ownership claim is
made. Next connect String provider selection, map its intrinsic m_value read to the receiver,
and generate a retained seed excluding that canonical declaration before testing unchanged
String sources. Char needs a separate representation-aware contract. See the public
[host API reference](../../../api-docs/experimental-metadata.md).

[String metadata execution commands and hashes](string-reference-metadata-2026-10-04.json).


### Source String ownership completed for the bounded gate (2026-10-04)

The provider selection and retained-seed step now passes instance text operations,
explicit Count and grapheme iteration from a separate consumer. Char is deliberately
still bootstrap-owned. Next establish its grapheme representation without converting it
to a numeric CLI char. See [the integration gate](source-string-2026-10-04.md).


### Owned grapheme metadata prerequisite (2026-10-04)

The metadata API now authors canonical runtime-owned System.Char with an explicit
NativeGrapheme fact, preserving its nominal signature identity and value receiver. A
separate neoIL consumer round-trips combining text and a ZWJ emoji through the API-authored
method. 143 C# metadata groups pass, including the unchanged CLI Char tests. See
[execution evidence](grapheme-declaration-2026-10-04.json) and the host API reference.

This is a bounded prerequisite, not completed Raven Char integration. Next implement
storage-aware external references and map that metadata fact into Raven symbols/host
ownership selection. Then remove seed Char and compile unchanged source Char. Current
snapshot convenience imports reject the owned grapheme category rather than losing it.
The explicit nominal category avoids conflating neoCLR graphemes with CLR UTF-16 code units.
