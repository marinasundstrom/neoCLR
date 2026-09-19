# Changelog

Notable changes to neoCLR, grouped by release and development date. Every commit
updates the Unreleased section; related changes on the same date share an entry.
Published sections are frozen. See the [maintenance workflow](docs/changelog.md).

## Unreleased

### 2026-09-19

- Replace the Raven profile's public System.Type/Type.Info split with sealed
  TypeInfo identity, shape and query contracts. Both typeof(T) and Object.GetType()
  return TypeInfo; RuntimeContext.Current supplies the configured handle resolver.
  Preserve concrete types through base/interface, string, array and boxed views.
  Native descriptor results retain their public interface view. Rebuild consumers;
  only a hidden CLI attribute-token Type shell remains in the compiler reference,
  and the historical Neo profile retains its old contract. All 73 slices regenerate
  reproducibly; 62 focused runtime tests, 33 importer admission cases and 74 saved
  project checks pass (43 before a native-result fix, 31 resumed). Editor checks
  expose Info interfaces and RuntimeContext and hide Type/Info/private providers.
  Assembly/module discovery and ExecutingAssembly remain the next slice.

- Record string handling as the next focus after basic introspection. Limit the
  planned UTF-8 work to a minimal surface, leave encoding undecided, and defer
  specialized string classes such as Utf8String. Clarify that introspection followed
  by strings should provide a coherent API foundation for next-preview evaluation
  and feedback; reflection/invocation and emit remain future extensions. Record
  the selected Object.GetType spelling and ExecutingAssembly property as design
  directions; the acquisition/context migration is not reported complete.

- Add reference-only `isinst` and `ref.isnull` instructions for runtime type tests.
  Successful tests preserve object identity through interface views; failed/null
  tests return typed nulls. Interface tests also accept boxed implementations and
  preserve intrinsic string representation. Boxed values also match System.Object;
  boxed value-type targets remain unsupported.

- Migrate the Raven profile's six Info contracts to sealed interfaces with internal
  Runtime*Info providers. Preserve permitted-type metadata in the reference and
  reject external implementations in the target importer. Materialize native
  snapshots as implementing classes and emit interface dispatch; rebuild callers.
  MemberInfo matches exhaustively over FieldInfo, MethodInfo and PropertyInfo;
  import and execute Raven's reference type patterns, and reject missing cases.
  System.Type/Info acquisition and array-return signatures remain transitional.
  Record the subsequent proposal to add TypeInfo as a member case with optional
  declaring-type ownership; that extension is not implemented yet. The reflection
  sample now expects no class base for FieldInfo; saved-project checks can resume
  from a named sample after a fixture correction. All 71 saved-project checks,
  29 reflection checks and 21 reference/boxing checks pass.

- Record the author's selected sealed Info hierarchy, replacing the proposed open
  implementation model. Permitted runtime providers remain internal; collection
  return contracts and RuntimeContext implementation are separate work.

- Record the open collection-return question and propose capability-based interfaces,
  with Sequence<T> a candidate for materialized introspection results. Distinguish
  read-only views from immutable snapshots; no array-return policy is implemented.

- Preserve declaring-type metadata for Raven-authored Option/Result cases by emitting
  them inside their companion containers. This repairs a full-port regression without
  changing managed method bodies or case storage. Check all four cases against their
  resolved owner identities; regenerate all 73 slices and the API inventory. All 12
  generic-union admission cases, 34 focused runtime tests and source ownership/snapshot
  checks pass. Record the final source-port Rust baseline: 1246 passing tests across
  all 178 integration binaries plus unit tests, with corrected failures rerun. All 65
  saved-project cases pass against the corrected library, closing the execution gate.
  RuntimeContext and the interface-based API migration remain separate work.

- Establish System.Runtime.rvnproj and its System.Runtime managed implementation
  identity, replacing System.rvnproj/NeoCLR.System without duplicating source owners.
  All 73 slices compile/import under the new identity; generated executable bodies
  are unchanged and ownership/snapshot checks pass. The bootstrap reference and
  executable neoIL retain their explicit mapping. RuntimeContext/interface API
  migration remains the next slice; published release artifacts are unchanged.

- Record the next API-alignment direction: establish System.Runtime, implement the
  basic RuntimeContext/provider model, retire System.Type for TypeInfo and use
  Object.GetTypeInfo for instance acquisition. This is the selected plan, not an
  implemented API; it supersedes the tentative Object.GetType spelling. RuntimeContext
  owns context-dependent discovery formerly represented by .NET Assembly static
  APIs; the first selected surface is executing AssemblyInfo → modules → types.
  Exact member spelling is provisional; loading remains outside the initial scope.
  Record a possible optional collector capability on RuntimeContext for later design,
  including heap/lifetime questions; no GC API or execution-policy change is made.

- Complete the Raven branch review and prepare an isolated VS Code development
  workspace with reproducible setup tooling. General editor-reference and ref-struct
  deconstruction fixes are on Raven main (65 and 31 focused checks); neoclr retains
  target policies. Rename its integration branch to neoclr and remove eight merged
  fix branches. Project checks pass on main/neoclr (47/52). Fresh copied-toolchain
  workspaces run the saved sample and pass editor checks; the installed editor logs
  confirm successful startup. No release or global SDK installation is performed.

- Refresh two validation fixtures after the source port: retain the Int64 CompareTo
  parameter name in its expected signature and allow the Console reachability graph
  to include generated carrier dependencies. Primitive admission and all 11 Console
  tests pass; runtime graph limits and service assertions are unchanged.

- Remove the obsolete TypeOf<T>.Of helper from runtime and Raven reference metadata;
  use typeof(T), or ldtoken/GetTypeFromHandle in direct IL, and rebuild existing
  helper callers. Update active samples while retaining published preview notes.
  Thirteen type/reflection tests and the saved reflection program pass; old helper
  calls are rejected. Add an ownership gate covering all 724 selected method/function
  declarations (54 explicit runtime services, the remainder generated). Record
  Object.GetType as a later preview API candidate and the development policy allowing
  deliberate compatibility breaks while retaining useful .NET ergonomics.

- Port managed Array members, callbacks and private iteration to Raven (73 slices).
  Preserve intrinsic storage and adapt the source iterator factory to the existing
  dispatch ABI. Retain static property metadata and reject fabricated array storage.
  Eight admission checks, 25 array/collection tests and four saved array programs
  pass. The historical Neo array profile remains unchanged.

- Port empty Object and UnionAttribute declarations to Raven (72 source slices).
  Check exact empty constructor and declaration shape while preserving the existing
  runtime root/marker ABI and compiler-facing recognition metadata. Nine admission
  checks, 22 attribute/union/reflection tests and the saved generic-union program pass.

- Port BindingFlags as a normal Raven Flags enum (70 source slices), preserving
  its six Int32 values, unknown bits and reflection filtering. Checked intrinsic
  enum lowering supplies the existing runtime ABI. Seven admission checks, 31
  enum/reflection tests and the saved flags program pass. Integrate Raven's general
  CLI enum backing-field metadata correction independently on main (266b457f5;
  13 focused checks), keeping neoCLR-specific lowering separate.

- Port the five Func declaration arities to Raven (69 source slices). Check exact
  CLI runtime delegate metadata, generic positions and the complete family;
  invocation and closure lifetime remain runtime-owned. Six admission checks,
  28 delegate tests and the saved Raven delegate program pass.

- Port System.Fault to Raven (68 source slices), preserving dynamic diagnostic text
  and terminal guest failure without aborting the host. Three admission cases and
  seven fault/query tests pass; a compiled Raven program verifies and raises the
  expected Unicode diagnostic. Refresh source ownership audits for Fault,
  descriptors and native allocation.

- Port NativeMemory overload composition to Raven (67 source slices), with checked
  bootstrap-only allocation/release/native multiplication operations. Preserve the
  consumer API, overflow, bounds and lifetime behavior. Five admission cases, 28
  native/pointer tests and the saved Raven allocation program pass.

- Port MemberInfo, FieldInfo, MethodInfo and PropertyInfo to a checked Raven source
  slice (66 total). Preserve inherited snapshot layout and runtime field names;
  Raven now owns parameter-array copies and accessor visibility filtering. The
  bootstrap-only vector read view is absent from consumer metadata. Nine admission
  checks, 23 reflection tests and the freshly compiled Raven reflection sample pass.
  Legacy Neo descriptors retain their existing representation.

- Integrate the independently validated Raven namespace/generic-constructor lookup
  correction on main (`d7292b935`) and the neoCLR feature branch (`d833ef2f3`). The
  ordinary .NET regression now returns 42 instead of silently returning 0; 78 focused
  main checks and 18 feature checks pass. No neoCLR policy was integrated into main.

- Extend the API-preserving Raven library port from 28 to 65 source slices: generic
  fundamental/collection contracts, SystemClock/LocalDateTime, native-sized integer
  comparisons, complete Int32 methods, Console, Environment, File.ReadAllText,
  String, opaque Error, five empty error types, Void, seven typed error carriers, Propagatable, Option/Result and their cases. Check intrinsic String storage and mixed receivers,
  preserve Error message delegation and runtime-owned payloads, and reject opaque
  allocation/defaults and String storage writes (11 admission cases pass). Preserve
  empty error defaults/constructors and nominal Void with nine further admission
  cases. Extract a general Raven unit-assembly lookup fix onto main (`c17cb8397`):
  same-named source types no longer shadow the configured metadata contract
  (19 .NET checks and 24 feature-branch checks pass). The empty-value slice passes
  14 focused Rust checks, four saved-program executions, and wrong-case/default
  error rejection checks.
  Check generic interface inheritance and nested calendar layout against reference
  metadata, and preserve library method parameter names for introspection. Keep native
  payload decoding bootstrap-only and preserve legacy Neo receiver/process conventions.
  Regenerate implementation snapshots and API audit.
  Typed carrier checks preserve case storage, value constructors, wrong-case faults
  and invalid-default rejection: 14 admission checks, 25 focused Rust tests and all
  64 saved-program cases pass with a fresh consumer core.
  Propagatable now has a checked Raven declaration preserving readonly receivers
  and true-only output initialization; five declaration admission checks and 20
  focused runtime propagation/union-output/generic-bound tests pass.
  Extract the ordinary CLI out-forwarding assignment fix onto Raven main
  (`5f6e17347`; 41 focused parameter checks pass); keep the target on its feature
  branch with cherry-pick `2d2a1d586`. Port Option/Result storage, factories and
  conditional extraction with checked generic constructors, readonly copy adapters
  and literal Boolean return proofs. Twelve admission cases, 41 focused Rust tests
  and all 64 fresh saved-project cases pass. A bootstrap-only miss intrinsic leaves
  output untouched; invalid defaults remain rejected.
  Source migration is still incomplete for remaining
  descriptors and runtime adapters; proposal API alignment remains a subsequent step.
  Pass 64 saved-project compile/import/verify/execute cases and retain native
  failure-status tests. Extract general Raven interface-binding and no-result delegate
  bridge fixes onto Raven main while retaining the neoCLR target on its feature branch.
  Update the inline managed-array fixture to embed generated collection declarations;
  the text assembler does not expand the new wrapper includes. All eight fixture
  checks pass after this correction; all 1241 Rust tests pass against the final
  foundation library, with all 178 integration-test binaries covered. The following
  String/Error slice adds native UTF-8 status injection and preserves existing
  introspection parameter names and String method order; all 21 focused
  string/error/default/interface Rust checks pass.

- Organize documentation with a top-level index and categorized guides, reference,
  design, Raven integration, history, contribution and experiment indexes, retaining
  existing document paths and published notes. Include the author-supplied original
  proposals with links to maintained design notes and explicit proposed status.
  Record source porting before the subsequent API-alignment step.

### 2026-09-17

- Plan System.Runtime as the minimal Raven-authored managed foundation, derived
  from the current System project. Document candidate scope, .NET comparison,
  namespace/assembly separation and coordinated identity migration. Link the plan
  from library docs and website; no assembly rename or new project is implemented.

- Add an executable RuntimeContext POC: opt-in Raven typeof returns the shared
  TypeInfo interface through Current.GetTypeInfoFromHandle, with internal runtime
  adapters. Run the source entry unchanged and separately check repeated/distinct
  scalar and array handles. Update integration docs, proposal and website sample.
  The installed core and production descriptor identities are not migrated yet;
  the bridge requires the updated Raven neoCLR feature branch.
  Validate 23 focused compiler tests, the context and existing introspection
  consumers, bootstrap snapshot consistency and the website build/highlighter.

- Extend the isolated Introspection prototype with TypeInfo.GetFields(flags) and
  FieldInfo interfaces. Field Type and DeclaringType use the shared TypeInfo model;
  preserve runtime filtering, ordering and unsupported-flag faults. Verify interface
  inheritance, structural signatures and rejection of dynamic field access. Update
  samples, docs and website. The eager Iterable adapter is experimental; production
  descriptor identities and RuntimeContext remain unmigrated.
  Record the required hidden Runtime*Info implementations and shared runtime-owned
  type-of/RuntimeContext acquisition yielding identical or value-equivalent descriptors.

- Fix neoCLR importer conversion of mixed constructor arguments: convert each
  argument in an adapter rather than repeatedly converting the top stack value.
  Cover class and value constructors, multiple Boolean arguments and single
  left-to-right source evaluation. Update integration docs; no Raven compiler or
  public API change.

- Admit checked nongeneric interface declarations in the Raven runtime-library
  importer and port the existing Clock contract to Raven. Preserve its Now property,
  System namespace and runtime-backed implementation. Reject mismatched interface
  shapes, properties and referenced type identities; validate FixedClock/SystemClock
  consumers and update samples, docs and website. This enables interface authoring
  but does not yet migrate production Info identities or introduce RuntimeContext.

- Add an isolated executable Raven probe for TypeInfo/MemberInfo interfaces with
  internal runtime-backed adapters and DeclaringType returning TypeInfo. Check
  emitted interfaces, execution and rejection of invocation/legacy Info access.
  Preserve BindingFlags in acquisition and expand existing member-consumer samples.
  Update docs and website; production descriptor identity and RuntimeContext are
  not migrated by this experiment.

- Refine the Introspection proposal to one `*Info` interface model without public
  Type/TypeInfo pairs, runtime-backed v1 through RuntimeContext, separate Reflection
  binding and Emit generation, and collection-oriented queries. Defer offline
  MetadataContext and typed introspection. Update migration docs and website;
  preserve superseded designs as history. These are target contracts, not changes
  to the existing executable API.
  Follow-up clarification retains BindingFlags during the current migration;
  collection-oriented querying remains a future option.

### 2026-09-16

- Port existing TypeInfo queries and its internal factory to Raven, retaining
  runtime-backed snapshots and filtering. Validate the opaque-handle layout and
  internal factory contract, and preserve internal visibility in generated code.
  Flag overloads now use the parameter name `flags`; regenerate metadata and
  update named calls using `arg0`. Clarify that Introspection need not be runtime
  independent: Reflection and Emit compose capabilities within one descriptive
  model. Update docs and website; member hierarchy bodies remain neoIL.
  Record future mixed-origin descriptor binding requirements for Emit, including
  context/provenance and identity validation, without adding new public APIs.
  Update Raven introspection samples with named `flags` arguments, an explicit
  TypeInfo binding and expanded readable blocks.

- Port System.Introspection.ParameterInfo's six read-only properties to Raven in
  the Raven runtime profile. Check snapshot field order/types and private construction;
  reject added/reordered fields, missing exports and public constructors. Regenerate
  bootstrap artifacts and update the documentation and website. Public APIs are
  unchanged; TypeInfo and member hierarchy bodies remain neoIL.

- Move existing TypeInfo, member/parameter descriptors and BindingFlags from
  System.Reflection to System.Introspection across runtime factories, reference
  metadata, Raven/neoIL consumers and bootstrap artifacts. Rebuild System and
  applications together and update imports; no old-name aliases are provided.
  Query behavior is preserved. Type remains Raven-authored; descriptor bodies
  remain neoIL and Info remains an instance property for this first slice.
  Update website status and migration documentation.

- Clarify the next release as an API-shape demo/POC across the text, time,
  globalization, async, introspection, filesystem and stream proposals. Plan Raven
  declarations and matching metadata alongside selected executable paths, with
  incomplete capabilities explicitly identified; full implementations are not a
  release prerequisite.
  Reflect this objective on the website with all seven proposal families, their
  benefits, design links and current implementation status.

- Record the proposal-driven Raven library migration priority: recommend porting
  existing introspection descriptors into System.Introspection next, followed by
  the existing time family under System.Time. Namespace moves and remaining Raven
  ports are planned; reference metadata and consumers must migrate together.

- Record a provisional file-system capability proposal: keep Path as a pure value,
  resolve it through an injectable FileSystem with a Default implementation, and
  separate File/Directory handles from byte-oriented streams. Compare the current
  static File/Directory APIs and System.IO.Abstractions, and leave authority,
  lifetime, enumeration, error, sandbox and async contracts open. No existing I/O
  API or runtime behavior changes.

- Record the requested Globalization, Introspection/Reflection and Stream proposals.
  Globalization separates immutable Culture/CultureId, resolution, contextual providers,
  formatting and collation; Introspection separates descriptive metadata from optional
  Reflection/Emit capabilities; Stream uses composable synchronous/asynchronous byte
  capabilities above FileSystem. Existing APIs and runtime behavior remain unchanged;
  ownership, data/versioning, capability availability, cancellation and exact members
  remain provisional.

### 2026-09-15

- Add the initial Clock.Now/Instant API with SystemClock, Raven-authored Instant
  and Duration values, and system-zone Instant.ToLocalDateTime conversion. Values
  use signed 100 ns ticks; Instant uses the Unix epoch. LocalDateTime now contains
  only Date and Time. Remove Clock.GetLocalNow and UtcOffsetSeconds; examples use
  Raven and clock injection. Configurable zones/calendars and duration arithmetic
  remain future work. Runtime tests replace the archived Neo clock smoke example.

- Replace placeholder parameter names in Raven-authored Date, Time and Type APIs
  and their reference metadata with descriptive names. Named calls must use the
  new labels (for example `year`, `ticks`, `other` and `index`); positional calls
  are unchanged. Exercise named arguments in calendar and reflection samples.

- Port System.Type to Raven for the Raven runtime profile. Type keeps identity and
  basic shape; metadata enumeration, base/interface lookup and enum metadata are
  exposed through System.Reflection.TypeInfo using the same opaque handle. Migrate
  Raven consumers to `.Info`; reflection descriptors remain in NeoIL. Validate Type
  layout and private construction, support matched class factories and self-array
  signatures, and require handle fields to be assigned before constructor return.
  The archived Neo profile retains forwarding APIs during migration. Descriptive
  parameter names are planned as the next slice.

- Clarify the closed reflection model: TypeInfo is a separate type-metadata view;
  MemberInfo remains the base for FieldInfo, MethodInfo and PropertyInfo.

- Introduce `System.TypeInfo` as the explicit member-lookup view returned by
  `Type.Info`. Type remains the identity and shape descriptor; field, method and
  property enumeration moves behind TypeInfo while the descriptor contracts remain
  unchanged.

- Add `System.Reflection.TypeInfo` and `Type.Info` as the first Type/Reflection
  separation slice. TypeInfo owns explicit member lookup while existing Type query
  methods remain forwarding compatibility shims during the transition.

- Move the memberless Value and RuntimeTypeHandle declarations to Raven. Check
  declaration-only imports against reference shape and reject added members,
  storage or constructor behavior. Preserve runtime erasure and opaque handle
  representation. Void remains declared in IL because the separate implementation
  assembly cannot define Raven's configured target-core unit type.

- Complete the Char struct port in System/Char.rvn, including CompareTo and all
  sixteen predicates. Move the previously standalone functions onto the type;
  retain existing UTF-16 code-unit and Unicode-category behavior through the native
  category service. The proposed text redesign is not part of this migration.

- Port Boolean, SByte, Byte, Int16, UInt16, UInt32, Int64, UInt64, Single and Double
  structs to Raven. Preserve comparison/NaN ordering and intrinsic storage; checked
  primitive backing fields lower to value loads and verified empty constructors are
  omitted. Reject extra storage, writes and effectful constructors. IntPtr/UIntPtr
  remain IL pending native-integer operator/conversion reference contracts.

- Port File.WriteAllText outcome handling to Raven, preserving all seven typed errors,
  Void success and the terminal fault for unknown native status. Keep native writes
  and ReadAllText in their existing implementation layers. Match the two target Void
  metadata encodings only in generic arguments, with core identity and empty-value
  checks; no-result return matching stays distinct.

- Organize Raven library sources into namespace folders under src/System, including
  Collections, Linq and function namespaces. Update project inputs, bootstrap source
  maps and documentation; public contracts and generated instructions are unchanged.

- Port Path.Combine and Path.GetFileName to Raven using the checked bootstrap host
  service catalog. Preserve lexical/native platform behavior, public parameter names
  and direct IL callers; no new path API or compiler special case.

- Complete the Math port with all 15 Double operations authored in Raven and the
  same native numeric services underneath. Add a bootstrap-only, signature-checked
  service catalog; guest imports cannot use it. Preserve numeric behavior and the
  public namespace contract without Raven compiler changes.

- Port Date to Raven, preserving Gregorian validation, day-number limits, component
  properties, comparison and Result errors. Reuse the checked Time value-import
  support without compiler/runtime changes. Keep API redesign separate from the port.

- Port Time to Raven with its existing Result factories, tick boundaries, properties
  and comparison contracts preserved. Extend matched value-library imports with
  interfaces, owned static factories, private constructors and verified readonly
  receivers; project matching calendar layout metadata. Share calendar regression
  cases across bundled and Raven profiles and test readonly/private access rejection.
  Date migration and API redesign remain separate work; no Raven compiler change.

- Record a new provisional Unicode-centred text-model proposal: canonical UTF-8 String
  storage, explicit UTF-8/UTF-16 representation views, constrained ASCII subset types
  and codec-based handling of other encodings. Mark the Raven-shaped examples as design
  notation only. Preserve the current UTF-16 code-unit Char contract until migration,
  metadata, interop and validation questions are resolved; no implementation changes.

- Refine the String proposal into a consolidated Unicode/text model: immutable String
  as scalar Char values, canonical UTF-8 storage, explicit encoded-string views for
  UTF-8/UTF-16/ASCII, separate Encoding transformations, and Option/Result/Fault
  handling. Keep the current UTF-16 Char and String-slicing implementations unchanged;
  the proposal's Raven-shaped examples, view lifetime, metadata and migration rules
  remain provisional.

- Update the date/time design proposal as NeoCLR Time API v1: layer Instant, Duration,
  civil values, offsets, timezones, calendars and Period; retain a narrow injectable
  Clock; and model DST gaps/overlaps and parsing/lookup failures explicitly with
  unions/Result. Keep the current Date/Time and local-clock implementation unchanged;
  the Raven-shaped examples, exact members, defaults, timezone data and arithmetic
  policies remain provisional.

- Add the first matched Raven value-library gate: sequential nongeneric scalar records
  must match reference field layout, representation and public instance contracts.
  Preserve instance ownership for value constructors and constructor debug identities;
  admit checked managed-reference `ldobj` copies without widening guest imports.
  Independent Int32/Int64 probes cover construction, copying, receiver mutation and
  private-field/layout/category rejection. Existing class/generic probes and bootstrap
  regeneration pass. Date/Time migration, value interfaces and static factories remain
  subsequent work; no Raven compiler or runtime instruction-set change is included.

- Port HashMap algorithms to Raven while retaining Map/MutableMap contracts in IL.
  Preserve explicit callbacks, chained hashing, growth, Option lookup, duplicate
  handling, shallow key snapshots and reentrancy faults. Store callbacks directly;
  diagnose capacity overflow before multiplication. Admit private nonvirtual instance
  helpers in matched library classes, preserving private visibility and validating
  every body without relaxing the public reference contract or guest imports.
  No Raven compiler or Runtime Contract change is required. All 18 collection/query
  runtime tests and 63 saved-project checks pass, together with private-method,
  private-storage and instance-contract probes and clean bootstrap regeneration.

- Port the complete ArrayList class and its private iterator to Raven, including
  constructors, indexing, growth, shallow copies and all seven predicate searches.
  Replace the legacy shared-state wrapper with direct checked array storage while
  preserving aliasing, captured-buffer iteration and Option outcomes. Remove the
  handwritten search fragment and constructor-rewriting adapter. Build the class as
  a checked slice of the shared project; preserve the reference/implementation gate.
  Capacity overflow now has an explicit list diagnostic. Update the reflection sample's
  build-local definition index and document the temporary bootstrap split. Recognize
  compiler array-invariance diagnostics when test projects inherit their configuration
  through MSBuild imports, while still requiring rejection without stale execution.
  Validation: 18 runtime tests, 30 query cases, eight mutation cases and 63 saved-project
  checks pass; bootstrap regeneration and API inventory/coverage checks pass.

- Port Where/Select sequences and iterators to Raven, completing all nine current
  query overloads in the shared runtime project. Remove handwritten deferred query
  bodies; preserve callback timing, cached Current, disposal and terminal faults.
  Retain generated internal classes in reproducible bootstrap fragments. Thirty Raven
  query cases, 33 delegate/query/reservation runtime tests, 203 scalar outcomes and
  13 cross-library cases pass, alongside authoring probes and clean regeneration.

- Add bootstrap-only checked generic array storage and scoped internal helper classes
  to matched Raven library imports. Keep consumer metadata and ordinary application
  admission unchanged. Validate Int32/String/Void storage and private helper dispatch,
  reject unwritten reads, invalid capacities and leaked helper contracts. Independently
  fix imported generic calls and generic array operations in Raven main with .NET
  execution tests and integration gates; retain neoCLR policy on its experimental track.

- Permit class constructors to initialize delegate fields. Reserve those fields as
  typed uninitialized storage instead of trying to manufacture a default delegate;
  reject early reads and constructor return with an uninitialized field. Other
  defaults are unchanged. All 25 delegate tests pass, including three new construction
  cases. This does not introduce null/default delegates.

- Add System.Fault(message) as a namespace function exposed to Raven through its
  existing CLI container contract. Preserve computed UTF-8 diagnostics and terminate
  guest execution through the existing Fault outcome; embedding hosts are not aborted.
  No cleanup guarantee or compiler non-returning-call analysis is introduced. Add
  runtime and Raven consumer checks and document the FailFast comparison. Deferred
  query migration uses the checked-storage and private-helper slices recorded above.

- Extend matched Raven library class imports to unconstrained generic parameters,
  constructed fields/methods, generic locals and supported interface declarations.
  An independent Cell<T> probe validates Int32/String/Void payloads, construction,
  mutation, copies and Iterable dispatch; arity/interface mismatches are rejected.
  Fix explicit generic self-construction independently in Raven and cherry-pick it
  into the experiment, with four new .NET execution cases and 17 focused tests.
  Raven main integration passes 311 compiler, 73 core and 249 language-server checks
  (three existing skips); existing neoCLR authoring/cross-library regressions pass.
  Subsequent same-day slices add checked storage, fault operations and private
  implementation dependencies, then port the deferred query bodies.

- Port Int32.Divide and seven Char predicates to the shared Raven runtime project,
  preserving Result errors, UTF-16 code-unit/Unicode behavior and existing public
  static APIs. Native parsing/category services remain in the runtime. Admit checked
  static implementation fragments on nominal core owners; refresh bootstrap snapshots
  and API coverage. Correct CLI Boolean joins for argument/local/field loads and
  ordinary call results while preserving conditional-out assignment proof. A separate
  Raven consumer passes 203 scalar and short-circuit outcomes; 30 query cases and
  19 focused runtime tests also pass. Raven compiler and
  Runtime Contract settings are unchanged; generic collection instance ports remain.

- Record Clock/SystemClock and List<T>.Create proposals, their .NET/Noda Time
  comparisons and open design choices. Confirm the current mutable List contract;
  no new clock abstraction, default accessor or collection factory is implemented.

- Add a bounded Raven instance-library import gate: validate one nongeneric class
  against its separate reference contract, retain its public runtime identity, and
  import constructors, private fields, methods and properties. Preserve ordinary
  guest assembly identities. An independent Raven execution probe and five invalid
  contracts pass, alongside 13 cross-library checks, generic-library checks and
  clean Math/query snapshot regeneration. Generic instance classes, interfaces and
  further System API ports remain subsequent work; Raven itself is unchanged.

### 2026-09-14

- Port seven query terminal overloads to the shared Raven System project: ToList,
  First, Last and Single, including predicate overloads. Import constructed generic
  collection/delegate/Option/Result signatures with scoped parameters and generic
  adapters. Preserve outcomes, cleanup and fault boundaries; five runtime terminal
  tests and 30 query integration cases pass, alongside generic/Math/Void checks.
  Refresh the signature probe for Math namespace metadata; all 121 contract checks pass.
  Build and flatten both Math and query snapshots for standalone use. Rename the
  extension-method container to System.Linq.Operators; rebuild consumers and core
  metadata together. Extension syntax is unchanged. Instance types/deferred iterator
  implementations remain neoIL pending shared implementation/reference identity.
  Extract general imported-signature, member-proxy, sibling-type lookup and empty
  union-case construction fixes to Raven main; retain target policy in its experimental
  branch and document both sides. Raven main CI passes 311 compiler, 73 core and
  249 language-server checks (three existing skips).

- Resolve generic unit-return invocation behavior through an independent Raven main
  fix (327335699; experimental cherry-pick ef352917e). Preserve the actual generic
  return value when consumed and pop it when discarded; ordinary no-result calls
  retain their existing behavior. All 22 focused .NET checks pass. Extend the generic
  library probe to consumed/discarded Void results; existing importer/runtime handling
  passes without changes. Update compiler/integration documentation and close the
  previously recorded unit-result candidate. Existing Void propagation and all
  69 Math results/six rejected contracts continue to pass.

- Admit bounded unconstrained generic namespace implementation bodies against an
  existing reference contract. Check generic arity/parameter positions, preserve open
  bodies and validate same-fragment calls using existing runtime generic functions.
  Add a test-only Int32/String probe and reject mismatched export contracts. Generic
  classes, constructed signatures and constraints remain later migration gates.
  Fix a general Raven generic-method projection crash independently on main
  (5f93eef6a; experimental cherry-pick 64a5497ec), with 12 focused .NET checks passing.
  Document the separate unresolved generic unit-return invocation issue in both repos.
  The generic probe, all 69 Math results/six rejected contracts, Void propagation
  and clean Math regeneration pass. No public System API or SDK package changes.

- Select System.Void through Raven's unit Runtime Contract in the neoCLR project
  properties. Document one platform unit type, no-result calls versus explicit value
  contexts, and the compiler/target integration boundary. Add metadata and runtime
  checks for Void arguments and Result<Void, E> propagation. Track the shared project
  properties in Math bootstrap snapshot inputs and regenerate with the new contract.
  General function return
  diagnostics were fixed on Raven main (0c66fbaa7), separately copied to the experiment
  (e20534894); invalid unqualified annotations no longer silently emit Object returns.
  Reusable Runtime Contracts are independently integrated into Raven main through
  2d17199a1; experimental unit projection is fc4592663. Require documentation of
  compiler-affecting changes in both repositories. The combined Raven contract suite
  passes 87 checks and its bounded CI passes; 13 experimental unit checks, the Void
  propagation probe and existing Math validation pass. No SDK refresh is included.

- Selected namespace functions for the Math migration and documented a general
  preference for namespaces over utility classes used only to group operations.
  Retained Raven's existing CLI container/TopLevelAttribute contract for now;
  recorded cross-language and reflection costs and the required migration checks.
  Added the shared Raven System.rvnproj with its first migrated Int32 Math bodies
  (Abs, Min, Max, Sign and Clamp), a reference-checked importer, generated bootstrap
  snapshots and source-release freshness validation. Typed Result outcomes and the
  existing internal runtime method owner are preserved; Raven consumers see the
  namespace API. Documented builds, bootstrap identities and the remaining generic
  implementation gate. Qualified lookup with local namespace declarations exposed
  a general Raven bug, fixed independently on Raven main (3ec32c96e) and copied to
  the experimental branch (008cb3245). The installed .14 SDK predates that fix.
  Math uses imported Ok/Error cases; recorded the remaining unqualified carrier
  return-annotation issue for independent Raven investigation.

- Reconciled the published Preview 7 documentation with the subsequent async/API
  direction notes, preserving both histories before resuming library migration.

- Recorded the selected task-based async direction with intended runtime-owned
  suspension, Task<Void> for no-payload completion, and optional transitional Raven
  state-machine lowering. Clarified that System APIs target modern language consumers
  with tasks as the normal async pattern; public runtime APIs may use callbacks where
  they better express the contract, and application code remains free to use them.
  Added a source-backed assessment of transitional compiler-generated async and its
  remaining library, lowering and execution-ownership gaps. Selected Result-bearing
  task completion for recoverable async errors. Selected ordinary async API names
  identified by Task return types, without mandatory Async suffixes; explicitly named
  blocking/sync alternatives remain exceptional. Documented .NET naming migration
  costs and provisional comparisons for markers, cancellation and operation ownership.
  Updated planning and conversation records; concrete task contracts and async
  implementation remain future work, without a requirement to preserve historical
  .NET compatibility.

- Published Preview 7 from 5da27a7 after all six source CI jobs and all 18 packaged
  validation suites passed. Updated the website and current walkthrough to point
  to the release. Recorded the validation outcome and installed a fresh local runtime
  bundle with the existing .14 SDK/VSIX. Candidate-preparation notes below describe their pre-publication
  state; published release content remains unchanged.

- Prepared Preview 7 candidate with the combined MSBuild/library workflow and
  Type.IsValueType. Refreshed the API declaration and coverage inventories to
  include the property and getter (616 declaration candidates). Publication remains
  subject to packaged validation and CI.

## 0.1.0-preview.7 — 2026-09-14

Release candidate; publication awaits packaged validation and all required CI checks.

### 2026-09-14

- Added read-only Type.IsValueType classification from runtime type categories,
  including the Raven reference/import surface and reflection sample. Managed arrays,
  classes, strings, interfaces, pointers and byrefs are distinguished from values;
  allocation location does not determine the result.

- Planned a needs-driven introspection/reflection review: compare the .NET
  Type/TypeInfo model, metadata inspection, execution capabilities and AOT retention
  before expanding descriptor APIs. No replacement hierarchy is selected. Extended
  API planning to distinguish a proposed common platform core, optional capabilities
  and host-specific services, with availability and conformance questions left open.

- Added a minimal standalone MSBuild build path for Raven `.rvnproj` applications
  targeting neoCLR. Props/targets select the supplied reference contracts and run
  the installed compiler, importer and verifier, without Microsoft.NET.Sdk or Raven
  changes. Build does not execute the program and invalidates prior runnable output
  on failure. Added a packaged demo, VS Code build-task configuration and regression
  checks. Installed a fresh local MSBuild demo with the existing .14 tools: 15 build
  scenarios, 68 language-server checks and the propagation demo pass; all 758 manifest
  files verified. Expanded the candidate to one application-to-library ProjectReference,
  with core-pack compatibility checks, changed-library rebuilds, failure invalidation
  and pre-build library completion. Added the two-project demo and made MSBuild the
  primary documented Build/Run workflow. Incremental builds, restore and deeper
  dependency graphs remain out of scope.

- Synchronized reviewed Raven main fixes into the experimental branch (`ee7b2e5af`)
  and installed SDK/VSIX `0.1.12-neoclr.14` with a fresh matching runtime bundle.
  Recorded installation instructions, revisions, hashes and packaged validation:
  63 saved-project, 15 application, 30 query, 5 compiler, 121 signature checks;
  68 editor checks each against bundled and installed servers; four array and four
  neoIL programs. All 751 manifest files verified. This is a local candidate only.

- Completed the independent Raven empty-array factory review (`f70ba5026` on main):
  use target metadata capabilities instead of assuming the host factory exists.
  All 93 focused checks pass, including .NET 10/.NET 11 reference-pack cases.
  Include the generic-array API execution verifier in future runtime bundles, and
  extend packaged editor checks for Empty, instance ForEach and indexed access.

- Recorded the proposed separation of Raven target profiles, symbol projections and
  emission backends, plus a staged evaluation plan. Preserved possible Raven compiler
  bootstrapping and neoCLR architecture/AOT/microcontroller work as long-term questions,
  not implemented features or preview commitments. Linked the roadmap and conversation
  record to the architectural proposal. Clarified the overarching theme as targetability
  and portability, with microcontroller architectures as potential targets. Added
  a future minimal MSBuild build path for Raven projects targeting neoCLR, independent
  of .NET SDK integration. Broader build features are optional later slices; this
  records a plan, not implemented build support.

- Implemented `Array<T>.Empty` and instance `ForEach(Func<T, Void>)` in the Raven
  runtime profile, reference surface and importer. Empty currently allocates a
  zero-length array without a shared-identity guarantee. Removed the profile's static
  ForEach helper; use `values.ForEach(action)`. Updated samples, reflection coverage
  and source validation instructions. Four Raven samples, 121 signature checks and
  32 runtime/collection regressions pass; installed tools still require refresh. Kept configured array-member projection
  and nominal generic Void emission on Raven's experimental branch.
- Independently fixed void-call stack-result tracking on Raven main (`8dbd96fb6`),
  then cherry-picked it into the experiment (`5c32d1d06`). Both ordinary and target-
  metadata .NET reproductions now run; 39 focused runtime and 21 initial checks pass.

- Recorded Raven main's qualified union type-pattern fix (`b0681f32b`), including
  closed variant member types and target-core locals. All 272 focused checks and the
  .NET 10/.NET 11 build/run matrix passed. Kept array-factory review, experimental
  synchronization and installed-tool refresh pending; library migration stays paused.

- Recorded the author's indexer-completion report and Raven main fix `ac4901f6b`:
  indexers require `[index]` access; their metadata names no longer behave as ordinary
  properties in lookup, hover or completion. All 440 focused checks passed. Kept
  installed-tool refresh explicitly pending and documented the general CLI/C# basis.

- Recorded Raven main's imported-union emission fix (`43f288b05`): closed pattern
  locals, valid primitive method signatures, retained assembly scopes and correct
  metadata containers for Raven union-case accessors. All 291 focused checks and the
  .NET 10/.NET 11 matrix passed. Kept bare type-pattern failures, boxing optimization
  and array-factory review explicit; experimental tools remain unsynchronized and
  runtime-library migration remains paused.

- Recorded Raven main's metadata core-library identity fix (`11e5c57ec`): imported
  structs retain their category using the supplied core assembly's full identity.
  All 33 focused checks, the full baseline (5,515 reported passes), and the .NET
  10/.NET 11 build/run matrix passed. Marked the general core-library prerequisite
  resolved while retaining the separate union-emission/array-factory reviews and
  the paused runtime-library migration. Experimental tools remain unsynchronized.

- Independently integrated imported member-union shorthand binding into Raven main
  as `b60e3635f`; 219 focused checks and the .NET 10/.NET 11 build/run matrix passed.
  Recorded the remaining target-core struct/signature failures exposed by execution
  and assembly inspection, and prioritized general metadata core-library identity
  alignment before further emission changes. Kept the boxing optimization and array-
  factory review separate; library migration and experimental synchronization remain
  pending.

- Expanded the website with project influences, six implemented feature summaries
  and nine excerpts sourced from executable Raven/neoIL samples. Added comparisons
  and tradeoffs, corrected stale importer limitations, and separated open research
  from preview capabilities. Led with Raven code, retained neoIL as a supporting
  example, and invited general feedback and discussion. The Pages workflow now
  rebuilds for Raven sample changes; desktop/mobile layout and link checks passed.
  Refined the narrative around familiar semantics and metadata, generic Void and
  Func callbacks, evolving class-library APIs including date/time, and Raven
  migration/tooling. Added planned library migration from neoIL to Raven and
  missing API work, without claiming those plans are implemented. Highlighted the
  current interpreter, tracing GC and terminal debugger; listed a possible JIT and
  collector improvements as future investigations, separate from Raven VS Code
  source-debugging support. Positioned neoCLR as an independent managed software
  platform inspired by .NET, evolving through community feedback, with Raven
  demonstrating the platform rather than defining its scope. Reorganized the page
  around .NET-familiar platform layers and colocated library examples, retaining
  prominent UTF-8 coverage. Used qualified Result factories and imported Some
  construction, with additional union forms behind an optional disclosure. Added
  build-time Raven TextMate highlighting using pinned tokenizer dependencies;
  syntax highlighting needs no browser JavaScript. Token escaping/state tests and
  focused shorthand execution checks passed. Added the requested Google Analytics
  tag (`G-SVXYRRCEEK`) to the page head and documented the analytics script.

- Completed the independent interface implementation metadata review: integrated
  Raven `f8f7568a1` into main after 54 focused checks and the .NET 10/.NET 11
  build/run matrix passed. Recorded the primitive-signature failure caught by .NET
  execution, independently extracted dependencies from the mixed candidate, and
  remaining union-pattern/array-factory reviews. The experimental branch remains
  separate and unsynchronized; runtime-library migration stays paused.

- Completed the independent application-generic metadata review: integrated Raven
  `b5ce4023b` into main after 42 focused checks and the .NET 10/.NET 11 build/run
  matrix passed. The review also fixed ordinary .NET generic field resolution.
  Recorded validation scope, the unchanged experimental branch and remaining
  interface/union-pattern/array-factory reviews. Marked the pre-Preview-6 release
  work order historical; library migration remains paused.

- Updated the website download link after publishing Preview 6 at `5c52b4b`, with
  experimental Raven `0.1.12-neoclr.13` from `246d697bf`. All six source CI jobs,
  15 extracted-package suites, both language-server checks and isolated VSIX
  installation passed. Verified all eight uploaded asset digests. Recorded the
  release conversation and completed outcome; published release sections stay frozen.

## 0.1.0-preview.6 — 2026-09-14

Stabilization and collection APIs, with a freshly synchronized experimental Raven
SDK/VSIX. See [release notes](docs/preview-6-release-notes.md). Source and package
validation are required before publication; release assets carry the evidence.

### 2026-09-14

- Refreshed the experimental Raven .13 extension notice inventory from its actual
  production inputs, adding preserved minimatch, brace-expansion and semver licenses.

- Fixed array-interface dispatch compilation on the declared Rust 1.85 minimum.
  Moved managed-array and native-memory IL samples into the current preview sample
  set so they use its System library, separate from historical Neo library tests.
  Both are now verified and executed by the packaged direct-IL smoke check.
  The packaged match-matrix verifier also accepts the matching System library
  explicitly, avoiding accidental fallback to the historical bootstrap library.

- Recorded the clarified CLI-compatible target-model direction and release-first
  stabilization plan: no neoCLR-specific code, mappings or tests enter Raven main
  yet; semantic-model nullability and alternative backends remain future questions.
  A broader Raven audit found an attribute-emission regression missed by focused
  checks. Corrected it on Raven main at 5a67d5d4c; 39 focused checks, four NanoFramework
  builds and the modern .NET matrix passed. The audit also passed 5,493 baseline
  tests, 173/172 standalone builds/runs and 38 eligible project runs. MacCatalyst
  remains blocked by its Xcode prerequisite; no full green release gate is claimed.
  Integrated the general delegate metadata fix at Raven 35a9df494 after 55 combined
  checks passed, and removed temporary branches. Added the current release work order.

- Removed completed Raven integration branches and superseded experiment branches
  locally/remotely after checking their history is retained in main or the active
  experiment. Removed the merged local neoCLR experiment branch as well. Active
  worktrees and unrelated branches were preserved. Integrated the independently
  reviewed reference-only constructor metadata fix into Raven main at f6b4748e6;
  26 focused checks and the repository .NET 10/.NET 11 build/run matrix passed.
  Removed its temporary branch after integration as well. Subsequently integrated
  closed generic method metadata into Raven main at e14d23d32: 27 focused checks and
  the same target matrix passed. Its completed branch was removed too. Integrated
  closed generic field metadata at ea6f3383b with 22 focused checks and the matrix
  passing. Recorded the author's clarification in both repositories: general Raven
  fixes, including those benefiting .NET Framework and NanoFramework, belong on main;
  neoCLR-specific integration stays experimental. These tests do not claim execution
  on .NET Framework or NanoFramework.

- Recorded the directive to integrate general Raven fixes into Raven main while
  keeping neoCLR experiments on separate feature branches. Updated repository
  instructions and the development conversation record; retained the explicit review
  queue for remaining general metadata-emission candidates. Raven main now contains
  the reviewed numeric, binding/dispatch and namespace fixes at 8fa59a967; 47 focused
  tests and the 5,490-test broader baseline passed. Synced main into the experiment
  at 5d1022ced with 50 focused checks passing. Subsequently integrated the general
  pointer metadata-emission fix into Raven main at 521711bec: retargeted void-pointer
  signatures now emit successfully, with 53 focused checks passing. The experimental
  branch already has the implementation; migration remains paused. No neoCLR runtime
  behavior change. Subsequently integrated closed-generic reference metadata emission
  into Raven main at 031b9aaaa; 18 focused checks and the repository .NET 10/.NET 11
  build/run matrix passed. Recorded the author's release ordering: complete the
  stabilization fixes before the next release, then revisit neoIL-to-Raven migration.

- Extended explicit Raven library imports to nongeneric class/value types,
  constructors, fields/accessors, inheritance, interface/virtual calls and static/class
  method-group delegates. Added cross-assembly type visibility checks and qualified
  delegate helper identities. Regressions cover GC, value copies, duplicate type names
  across assemblies and rejected hidden types/private constructors/generic bodies.
  Source maps now qualify type records by assembly. Rebuild generated IL/maps; no Raven
  or runtime opcode change, installed-tool refresh or System-library migration.

- Fixed direct wildcard imports of namespace functions from Raven libraries by
  adding the target TopLevelAttribute metadata declaration and correcting Raven's
  marker lookup and imported-member completion on an isolated feature branch.
  Added a target namespace probe and switched the library regression to direct calls.
  Regenerate core metadata and rebuild libraries/consumers with matching Raven tools;
  installed SDK/extension artifacts remain unchanged. Library migration stays paused.

- Added bounded separate Raven library importing for nongeneric static methods,
  including intra-library namespace functions through existing CLI metadata. Dependencies
  are explicit and closure-audited; cross-assembly private calls and unsupported
  library instance bodies are rejected. Method queues now distinguish assemblies
  with reused tokens; source maps carry assembly identities and explicit output labels.
  Breaking generated-map schema: rebuild IL/maps and use the updated tools. Library
  migration stays paused; generic bodies and library-owned instance types remain open.
  Seven library, 15 application and five compiler/import checks pass. Recorded a Raven
  gap in direct wildcard discovery of namespace functions from separate DLL metadata.

- Replaced application type/field/function row-token names in the Raven importer
  with assembly/signature-based identities and collision-safe member escaping.
  Added readable metadata-to-runtime mappings to source-map sidecars and regression
  checks for declaration insertion, assembly scope, overloads and closed signatures.
  Rebuild generated application IL/maps together. This does not enable separate
  library loading; runtime-library migration and installed tools remain unchanged.

- Added ordinary Raven compiler emission followed by independent artifact import.
  Projects now select matching metadata/emission cores; regenerate editor projects
  and use the matching Raven target-contract branch and bridge. Added five direct
  compiler/import checks; all 63 saved-project checks pass. Documented host-reference
  isolation fixes, the passing .NET 10/11 matrix and remaining importer boundaries.
  Extracted general Raven fixes onto a separate review branch (5,489 baseline tests
  passed); Raven main and installed SDK/extension artifacts remain unchanged.

- Paused the Raven runtime-library migration at the author's direction, preserving
  the scalar pilot locally. Recorded the Raven branch assessment, compiler-fix review
  batches and a plan for coherent target configuration and metadata-driven importing.
  Updated the roadmap and conversation record; no Raven branch was merged and broad
  library-authoring support remains unimplemented.

- Fixed Raven string equality/inequality by declaring the missing target String
  operators and binding them to existing value equality. Interface query predicates
  now compile without changing Raven or CLI reference comparison semantics. Added
  constructed-string, captured-predicate and metadata regression coverage; importer
  stack errors now identify the method, IL offset and expected/actual types.
  Rebuild the bridge and reference metadata together; installed .12 tools are unchanged.

### 2026-09-13

- Added Raven-profile First/Last/Single predicate overloads with existing Option/
  Result outcomes, forward callback order and normal-outcome iterator disposal.
  Direct iteration avoids Where wrapper allocations; no runtime/compiler change.
  Added callback/cardinality/fault/allocation tests, metadata rejection and editor
  signature-help checks, updated samples and .NET comparisons. Regenerate core
  metadata and System together; existing source-only overloads remain supported.
  Installed .12 tools are unchanged. Recorded the decision to migrate ordinary
  library authoring to Raven after a source checkpoint, retaining neoIL for low-level
  code/tests and documenting bootstrap and generic-import groundwork.

- Built and locally installed the side-by-side Raven SDK/VSIX .12 and matching
  neoCLR collection bundle from neoCLR 6fd3729 and Raven 854cd4d3d. The isolated
  VS Code profile opens the combined order-collection demo. Eight extracted-bundle
  suites, 61 installed-editor checks and the configured Run task passed. Recorded
  payload/artifact hashes, dependency-notice checks, build evidence and instructions;
  existing demos and normal Raven launchers were preserved. No public release.

- Added a combined Raven order-collection scenario using application-defined class
  payloads, HashMap duplicate checks, ArrayList filtering, array/interface LINQ and
  Option/Result propagation. Added application checks under GC pressure and hash
  collisions, plus completion checks for inferred Order payloads. Documented shallow
  filtered membership versus shared object state and how to run with fresh matching
  core metadata/System. No runtime/compiler change or installed-tool refresh.

- Added direct-storage Raven-profile ArrayList filtering: Find/FindLast return
  Option<T>, FindIndex/FindLastIndex return Option<Int32>, Exists/TrueForAll answer
  Boolean questions, and FindAll returns an independent shallow list. Breaking:
  FindIndex no longer returns Int32/-1; regenerate core metadata and System together
  and recompile callers. Scalar scans avoid managed iterator/query allocations and
  retain the initial buffer/extent across callbacks and GC. Added IL, Raven,
  metadata/editor checks, .NET comparison and API/migration documentation. Historical
  Neo and the installed .11 SDK/extension remain unchanged.

- Added Raven-profile LINQ terminals: First and Last return Option<T>; Single
  returns Result<T,SingleError> with Empty and Multiple cases. Existing IL and
  metadata support the APIs without compiler changes. Normal outcomes dispose
  iterators, including early exits; iterator/callback/disposal faults stay terminal.
  Added propagation/pattern samples, metadata/editor checks, direct IL tests and
  .NET comparisons. Core metadata and System must be regenerated together; installed
  .11 tools are unchanged. Recorded the preference for concrete collection helpers
  to avoid query allocations while retaining LINQ composition and specialization.
  Planned ArrayList filtering and outcome review as a separate upcoming slice.

- Added an experimental Raven-profile Map/MutableMap/HashMap slice with Option
  lookup, duplicate-preserving TryAdd, Set, count and independent key snapshots.
  Managed storage, collision chains and growth use existing IL; equality/hash
  callbacks are explicit constructor arguments, with no default comparer or uniform
  null-key policy yet. Added Raven samples, signature/editor/capability checks and
  direct IL tests for GC retention, collisions, snapshots and reentrancy faults.
  Corrected an older collection test to use the inherited indexer owner and the
  target library when assembling. Regenerate core metadata and System together;
  the installed .11 tools are unchanged. Documented .NET/Rust/LanguageExt comparisons
  and remaining Map work. LINQ terminal changes remain separate.

- Implemented a bounded Raven-profile collection capability prototype: Collection
  for count/iteration, Sequence for indexed reads, MutableSequence for replacement,
  and List adding growth through Add. Arrays and ArrayList share those contracts
  through ordinary inherited-interface dispatch without wrapper allocations. Added
  array Count, samples, negative/metadata tests and editor checks. Callers must
  regenerate declarations and recompile; installed .11 tools are unchanged. Recorded
  .NET comparisons and provisional names; immutable/frozen families and variance
  remain separate. Map/dictionary work and LINQ terminal Option/Result contracts
  follow as separate slices.

- Built and installed experimental Raven SDK/VSIX 0.1.12-neoclr.11 with the generic
  array integration in an isolated local demo/profile. All seven packaged suites
  passed, including 59 saved-project cases, 28 query checks and 52 editor checks;
  the installed VSIX and configured Run task passed. Verified archive/payload hashes,
  preserved existing source files and recorded provenance and launch instructions.
  This is a local build, not a published release.

- Added the generic managed System.Array<T> shape to the Raven runtime profile,
  sharing identity/storage with ordinary array signatures and explicitly declaring
  Iterable<T>. Added Length/Item members and closed generic/member reflection.
  Raven now reads vector interfaces from the generic array reference declaration,
  including interface inheritance and element substitution, through an optional
  target mapping. Raven now unifies Array<T> and T[] in source/imported signatures,
  including indexing, nested arrays, typeof and interface member calls. Added a
  readable end-to-end sample and editor completion/hover checks.
  Indexed loops and the default .NET target are unchanged.
  Mutable arrays remain invariant; no wrapper allocation or growable List contract
  is added. Removed the profile's native array descriptor without a compatibility
  alias; added bounded NativeMemory.Alloc/Free APIs and direct IL examples. Updated
  samples and documented remaining Raven native-cast gaps and historical Neo scope.

- Added a bounded read-only view experiment in Raven and direct IL, with .NET/API
  comparisons and checks for live aliases, shared element identity, GC retention,
  access restrictions, bounds and null behavior. Recorded generic-variance prerequisites
  and reviewed the supplied collection hierarchies, including Set/Map variance and
  fixed-size array mutation, without adopting a taxonomy. No library interface or
  runtime variance support is added by that adapter experiment; mutable arrays remain
  invariant. Its generic System.Array<T> direction led to the implementation listed above.
  Prioritized that mapping, minimal collection contracts and basic implementation
  prototypes in the API plan, ahead of broader text/clock expansion.
  Audited the existing native System.Array<T> naming conflict and recorded .NET-aligned
  separation of managed arrays, native allocation and borrowed views. The subsequent
  implementation removes the native descriptor from the Raven profile; borrowed
  views remain future work.

- Clarified platform direction: retain CLR-like type categories and language ergonomics,
  develop library APIs by concrete need, and evaluate text/encoding, memory views,
  nullability metadata, callable types, runtime async and injectable clocks. Updated
  stale default-semantics policy summaries and recorded the development conversation.
  Broadened substantive API reviews to other platforms, .NET feedback and independent
  .NET libraries, with source status, counterevidence and transfer costs recorded.
  Distinguished immediate reversible prototypes from evidence-backed adoption of
  major platform contracts; retaining existing behavior remains an explicit option.
  These are design reviews and plans, not newly implemented runtime capabilities.

- Built and installed experimental Raven SDK/VSIX 0.1.12-neoclr.9 with the recent
  array, numeric and invariance fixes, then refreshed to .10 with target-aware
  array conversion diagnostics. Both isolated builds passed all six packaged suites
  and the installed order-workflow task; 701 payload hashes were verified for .9
  and 702 for .10. Verified .10 editor diagnostic recovery, recorded provenance and
  updated the local walkthrough. Existing demos, the default SDK and published
  releases are unchanged. The .9 installation retains its earlier editor behavior.

- Made mutable-array invariance an explicit runtime contract: verifier, interpreter
  and Raven importer reject differing array element-type casts, including typed-null
  casts; exact interface-to-array casts retain allocation checks. Added inheritance,
  value-element, jagged-array and interface-view regressions. Documented the deliberate
  CLR covariance difference and a future read-only projection; read-only variance is
  not implemented. New Raven target projects now disable covariance through the companion
  compiler's target-neutral option. Added editor diagnostic/recovery checks and saved-project
  coverage so invalid array conversions fail during binding. Default .NET behavior is
  unchanged; the refreshed local .10 tools include this policy, while .9 remains unchanged.

- Admitted existing numeric division, remainder and shift instructions in the Raven
  importer with operand validation. The companion Raven fix preserves unsigned
  division/remainder/right-shift semantics instead of treating high-bit values as
  negative. Added signed/unsigned/floating execution and division-fault regression
  checks. Requires refreshed experimental tools; runtime semantics and published
  artifacts are unchanged.

- Extended companion Raven fixed-width implicit numeric conversions and corrected
  unsigned-to-floating emission using existing CLI instructions. Added a saved-project
  sample checking signed, unsigned, floating and Char widening at boundaries.
  Requires the updated experimental compiler; the runtime and published tools remain
  unchanged. Native-sized conversion rules and Decimal target APIs are not added.

- Stabilized mixed numeric operator binding in the companion Raven experiment:
  binary operator candidates now require implicit conversions, fixing the previously
  recorded `ulong`/signed lookup failure. Added target regression checks for valid
  mixed comparison promotion and compile-time rejection. No runtime instruction or
  metadata changes are needed; rebuilding the experimental compiler picks up the fix.

- Added runtime vector views as Iterable<T>, preserving array identity with separate
  iterators and no sequence wrapper. Raven opts into the target interface through
  project configuration; ordinary query extensions now accept arrays, including
  reflection results. Added runtime, GC, conversion, query and completion checks.
  This fixes Preview 5's array-query limitation for the next release. Projection is
  invariant and vector-only; existing tools/artifacts remain unchanged. The Rust
  ObjectReference::concrete_type accessor now returns an owned Type so array concrete
  identity can be recovered independently of its interface view.

- Fixed Raven vector `for` loops and ordinary numeric comparisons/query predicates
  by admitting the existing CLI numeric comparison and conditional branch families
  in the importer, including short and unsigned/unordered forms. Operand and
  control-flow validation remain enforced; Boolean results use the existing CLI
  adapter. Added array-loop and signed/unsigned/floating boundary regressions. This
  fixes Preview 5 importer gaps for the next release. A companion fix on Raven's
  experiment branch corrects unsigned and NaN comparison emission for both targets;
  refreshed tools are required. Published artifacts remain unchanged.

- Published Preview 5 at c76ee57 with eight verified assets after all six exact-source
  CI jobs and thirteen package suites passed. Recorded publication evidence, match
  checks and artifact provenance; published release notes remain unchanged.

## 0.1.0-preview.5 — 2026-09-13

Application and query preview: see [release notes](docs/preview-5-release-notes.md).
Includes the experimental Raven 0.1.12-neoclr.8 SDK and VSIX.

### 2026-09-13

- Extended the order workflow with a deferred pending-order query and materialized
  name summary, exercising queries, interfaces, Result/Option patterns and file I/O
  together on the installed .8 tools. Added failed-write and all-saved summary checks,
  prepared a separate local demo, and corrected stale preview-readiness descriptions.
  This sample update does not change the archived .8 build or its recorded results.

- Built and installed local Raven SDK/VS Code extension 0.1.12-neoclr.8 with the
  pattern, query and custom-iterator fixes. Prepared an isolated query demo and
  recorded six passing packaged suites and artifact hashes. Preserved existing
  demos and the default SDK; this build is not published.

- Enabled custom Raven Iterable/Iterator implementations in the query pipeline:
  close generic MethodImpl signatures before bridge validation and promote the
  earlier compiler repro to an executable sample. The companion Raven fix normalizes
  target interface references; ordinary CLR behavior is covered separately. Updated
  query documentation and regression coverage; cleanup limitations still apply.

- Added prototype Raven-target `System.Linq.Enumerable` extensions: deferred generic
  Where/Select and eager ToList returning ArrayList. Ordinary NeoIL classes retain
  source/callback state, cache current values and dispose upstream on exhaustion or
  explicit disposal. Added execution, GC, lifetime, negative and completion checks;
  documented cleanup, mutation and compiler boundaries. Corrected Void projection
  for generic method arguments. No query opcode, Raven compiler change, legacy Neo
  update or archived toolchain replacement.

- Enabled nongeneric Raven application extension methods by completing their
  generated marker metadata dependencies. Added readable value/reference/interface
  receiver and callback examples, execution and completion checks, and explicit
  rejection coverage for generic application extensions and executable marker
  construction. No new runtime opcode or Raven compiler change; archived toolchains
  remain unchanged. Documented the generic bridge boundary before the LINQ slice.

- Added Raven union payload destructuring through the existing extraction and
  payload APIs: imported `Ok(let text)` / `Error(let error)`, target-typed cases,
  and explicit generic case patterns. Updated the order workflow and added a
  readable pattern sample and executable compatibility checks. The bridge tracks
  unconditional deconstruction outputs and nested pattern assignment; the source
  experiment requires Raven 04c953d67 and refreshed declaration metadata, not
  archived .7 tools. Prepared a separate source-backed local demo and verified
  payload completion/hover with its updated language server.

- Added target completion checks inside constructors and installed a patched Raven
  language server after reproducing missing Int32 static members in constructor
  bodies. The general compiler fix is isolated on Raven at 55c0f7ef5; archived tools
  remain unchanged.

- Recorded the future modern-library API direction and mockable-clock investigation,
  comparing .NET TimeProvider with narrower library contracts. No clock behavior changed.

- Extended experimental bundle packaging with the application and order-workflow
  validation scripts and documented how to try the new samples. Updated bundle
  capability boundaries; package validation now includes dispatch, captures and GC.
  Built and installed local Raven SDK/extension 0.1.12-neoclr.7 with a separate demo
  and VS Code environment. All twelve package suites and installed-server checks pass;
  recorded artifact hashes and usage instructions. No new release was published.

- Extended runtime delegate binding to nominal heap receivers and the Raven source
  bridge to instance/virtual/interface method groups and shared or escaping lambda
  captures through Func, including Func<Void>. Added binding-time null checks,
  closure GC tests and ordinary CLI add/sub/mul admission; documented adapter costs
  and remaining boundaries. Raven's experimental method-group emission now preserves
  virtual/interface dispatch and explicit base selection. Expanded application sample
  blocks for readability and recorded that preference in the repository workflow.

- Added a Raven order-workflow application with domain classes, an application
  interface, ArrayList storage, Option lookup and Result<Void, FileWriteError>
  propagation. Isolated checks verify persistence and unchanged order/report state
  after rejected writes; documented source-toolchain setup and current limitations.

- The Raven source bridge now imports ordinary application classes/value types,
  fields, constructors and instance accessors/methods, including ArrayList storage.
  Extended this to application class interfaces, abstract/virtual inheritance and
  nominal constructor chaining; runtime checks preserve concrete dispatch and reject
  invalid chaining. Added runtime and Raven examples/checks for the contracts.
  Added class-alias/value-copy, saved-source, mapping and rejection checks. Recorded
  Raven targeting findings and the future generic-Void async requirement; the source
  bridge requires the updated experimental Raven branch, not the published Preview 4 SDK.

- Recorded the implementation-language discussion, distinguishing Rust build
  dependencies and implementation memory safety from neoCLR's guest GC; language
  migration remains an open question, with no runtime changes.

- Published Preview 4 at c135659 with eight runtime/Raven/source/validation assets
  after all six exact-source CI jobs and ten package suites passed. Updated the
  website download link, current installation guide and published validation record;
  published release notes and the Preview 4 changelog section remain frozen.

## 0.1.0-preview.4 — 2026-09-13

Runtime/Raven preview: see [release notes](docs/preview-4-release-notes.md) for the
prebuilt macOS arm64 distribution, matching Raven tools, migration and limitations.

### 2026-09-13

- Updated release smoke validation for the guest-only stdout contract: request
  --show-result explicitly and verify successful completion on stderr. Retained
  environment, clock and file content assertions without the retired output suffix.

- Added a responsive project website explaining neoCLR, Raven integration, runtime
  APIs and preview limits, with excerpts built from executable samples. Added a
  separate GitHub Pages workflow: pull requests build/check; main and manual runs
  publish through a restricted deployment job. Documented local preview and hosting.
  Enabled Pages and verified the successful deployment and live asset contents at
  https://marinasundstrom.github.io/neoCLR/.

- Built and locally validated the runtime/Raven candidate with direct neoIL samples,
  source archive, toolchain notices and matching .6 Raven SDK/VSIX. Ten extracted
  package suites pass; all runtime test executables pass after the recorded stale-test
  corrections, with strict Clippy/format checks. Recorded revisions, asset hashes and
  remaining final-version/CI/publication gates; refreshed the prepared local demo.

- Simplified the existing virtual-call admission condition and empty verifier match
  arms for strict Clippy validation while preserving the supported receiver modes.
  Updated stale initialization tests and String API prose to the implemented typed-null
  String default; non-defaultable Error/Value rejection remains covered. Updated
  legacy Neo test expectations for that shared runtime behavior without changing the
  Neo frontend.

- Preserved version-specific notices for 26 NuGet and five bundled JavaScript
  dependencies in the experimental Raven tools, with source/hash provenance and
  a dependency-manifest coverage checker. Future runtime bundles carry these texts;
  separate SDK/VSIX assets require the companion attribution archive.

- Accepted ordinary lowercase void returns in neoIL using the existing empty-stack
  calling convention, and updated preview samples. System.Void remains a unit type
  in generic/value contexts; legacy uppercase Void and noresult behavior is retained.
  Added return-stack and generic-unit regressions; binary CLI encoding is unchanged.

- Added a runtime/Raven release walkthrough and direct neoIL demonstrations for
  class aliasing/value copies and Result<Void, Error>, verified against the adapted
  library. Bundles now include these samples and their repeatable output check.
  Marked old Neo language guides as historical and recorded the two-part release
  direction; existing local archives remain unchanged pending a fresh candidate.

- Updated the repository introduction and roadmap to present the current Raven/CLR
  type-category direction and completed runtime API milestone, separating the legacy
  Neo experiment. Future portable bundles now include both repositories' license and
  notice files; existing .6 assets are unchanged. Binary dependency attribution review
  remains a publication check.

- Added published-bridge execution and a checkout-independent experimental bundle
  builder with pinned metadata/library/server, samples, setup and file-hash provenance.
  Shared validation scripts accept either source or published toolchains. All nine
  suites passed outside both checkouts. Installed experimental SDK/VSIX .6 and
  verified its server and saved demo; recorded artifact hashes and setup instructions.
  This is a local macOS arm64 build, not a published release.

- Completed the source-by-source existing API audit, including runtime-service
  callers, signature markers and explicit importer limits. Preserved writable union
  case payloads through Value setters and verified case-copy independence; requires
  the Raven value-property receiver fix, included in the validated .6 toolchain.

- Generalized Raven collection/union payload mapping to existing references, nested
  collections, interfaces, unions and delegates, with a nesting bound. Expanded
  managed-array access and ForEach to admitted defaultable elements, checking opcode
  widths/signedness and preserving canonical Boolean joins. Added executable shapes
  and exposed the opaque System.Value metadata identity.

- Added internal-library array.reserve allocation with ordinary managed-array
  identity and checked unreadable slots. Adapted ArrayList capacity now supports
  elements without defaults without inventing union cases; ordinary newarr retains
  default initialization. Added publication and direct/indirect-read regressions.

- Projected Equatable, Comparable, Clonable and Closable contracts into Raven metadata
  and calls. Restored Type equality conformance and value/interface implementations;
  added boxed primitive/calendar, String and class-reference samples and checks.
  Clonable/Closable have no existing concrete library implementations to demonstrate.

- Added bounded CLR-style box allocation and nominal interface dispatch into value
  payloads, preserving value-copy independence, alias identity and managed lifetime.
  Intrinsic String interface views use managed handles. Added GC, escape and invalid
  operation tests; generic reference/nullable boxing and unboxing remain separate.

- Exposed unsafe native Array<T> buffers through Raven for primitive elements,
  including allocation/views, fields, indexers, addresses and explicit release.
  Added saved-source success, bounds/lifetime faults, metadata checks and completion.
  Int32 pointer reads/writes are admitted; general native layouts remain bounded.

- Replaced provisional BindingFlags wrapper metadata with a standard Int32-backed
  CLI enum, literals and FlagsAttribute. Raven uses casts and bitwise operators
  instead of the wrapper factories/combinators; typed adapters retain runtime
  nominal storage. Added flags samples, metadata rejection and completion checks.

- Exposed Raven type tokens, reflection queries, descriptor properties and managed
  snapshot arrays, including base-class views and Option<Type/MethodInfo> results.
  Added full public-getter/query samples, signature checks and editor completion.
  BindingFlags currently projects its factory/combinator API as a value wrapper;
  true enum metadata/operator syntax remains a following slice.

- Migrated reflection snapshot storage in the Raven runtime profile to ordinary
  Type/descriptor classes and the MemberInfo hierarchy. Trusted query factories
  allocate nested snapshots under heap limits and retain them through GC. Source
  metadata/import projection remains separate; added runtime regression coverage.

- Added ordinary class ancestry, implicit base-reference conversions and checked
  downcasts, preserving allocation identity and inherited fields. Allow abstract
  class declarations without abstract members; reject mixed storage categories.
  Class constructor chaining and virtual dispatch remain separate groundwork.

- Projected Array.ForEach for managed Int32/String arrays and static Func<T,Void>
  callbacks. Resolve closed method-generic signatures against target metadata,
  retaining the existing callback algorithm and ordinary no-result API boundary.

- Admit no-result static generic IL methods while retaining the distinction between
  generic Void payloads and absent call results. Added execution and invalid-return
  regressions; generic instance-class methods remain outside this subset.

- Projected the five Func arities for static Raven callbacks, including named
  Func<Void> completion, and exposed ArrayList.FindIndex/Exists/Find. Preserve
  generic Void return context and the adapted no-result Dispose calls. Added
  executable callbacks and predicates; capturing/instance callbacks remain outside
  this importer slice. Requires the corresponding Raven experiment compiler fixes.

- Projected Boolean.CompareTo and Raven canonical Boolean literals/equality across
  locals, parameters, returns and generic API calls. Typed adapters preserve argument
  order and output contracts while retaining the runtime Boolean representation.
  Added saved-source coverage; arbitrary CLI Boolean bit patterns remain unsupported.

- Extended Raven collection bindings beyond Int32 to admitted primitive, String,
  calendar and empty-error elements, and exposed ArrayList.Copy. Added growth,
  aliasing, independent-copy, iteration, completion and invariant-signature checks.
  Elements without runtime defaults and predicate/delegate APIs remain separate.

- Projected Console.ReadByte and all three existing Environment APIs through Raven,
  including nested Result/Option propagation and managed string argument arrays.
  Added controlled environment/argument/byte-input tests, signature rejection and
  completion checks. Argument snapshots currently copy through an adapter; installed
  SDK/extension packages have not been refreshed.

- Added typed-null String defaults for managed locals, constructed fields and array
  elements, aligning string-array initialization with CLR expectations. This
  replaces the previous default-initialization fault; intrinsic string receiver
  representation and native pointer rules are unchanged. Updated generic constructor
  coverage to verify String initialization after the newly supported typed-null default.

- Generalized Raven Option/Result bindings for admitted primitive, string, calendar,
  error and nested union payloads. Exposed factories, predicates, checked case
  access and extraction/propagation members, retaining guarded-output validation.
  Handle both Raven pattern branch forms; added nested execution, rejection and
  completion checks. Arbitrary application/reference payloads remain unsupported.

- Projected existing error-value constructors, predicates, checked accessors and
  ToString APIs, including all 23 nested error cases and System.Error message APIs.
  Added complete case round-trips, wrong-case/default rejection checks and completion.
  Empty values have valid defaults; carriers/messages require initialization. Existing
  description differences are preserved; no Exception hierarchy or runtime change.

- Projected all 24 current public Date/Time/LocalDateTime/Clock methods/accessors
  through Raven, including validated Result factories and value-payload propagation.
  Added fixed calendar/tick cases, live host-clock validation, completion/signature
  checks and usage docs. Defaults use initobj with private storage preserved; no
  new globalization, formatting, runtime or installed-tool behavior is implied.

- Added Raven primitive storage/stack normalization, ordinary numeric conversions,
  concrete comparisons and all 16 existing Char classifiers. Added boundary/Unicode
  samples and completion/signature coverage. Requires the isolated Raven integral-cast
  fix `26907410f` (also tested on CLR output). Boolean normalization, general interface
  dispatch and arbitrary generic payloads remain pending; runtime and installed tools
  are unchanged.

- Added bounded Raven Double literal/slot/receiver import and projected all 15
  existing Double Math methods plus Double.CompareTo. Added all-method, rounding and
  NaN examples, signature checks and Math completion coverage. All 20 current Math
  methods are projected; conversions, floating operator lowering and formatting are
  not implied. No runtime, Raven compiler or installed SDK/VSIX changes.

- Exposed the existing Int32 Math.Clamp API through Raven with Result matching and
  propagation for InvalidRangeError. Added inclusive/equal/reversed-bound and Int32
  extreme-value examples plus completion coverage. The five current Int32 Math
  methods are projected; floating-point Math and the error's constructor/ToString
  remain pending. Runtime semantics and installed tools are unchanged.

- Exposed Int32.Equals/CompareTo/ToString through Raven, adapting ordinary managed
  receivers to the existing readonly/snapshot runtime contracts. Added bounded
  Int32 parameter-address import for these receivers, local/parameter examples,
  signature checks and completion coverage. No boxing, runtime or Raven compiler
  changes; general byref/constrained calls and interface projection remain separate.

- Projected the existing Int32.Divide Result API into Raven, including division-by-zero
  and overflow predicates, matching and propagation. Added signed/boundary examples,
  checked-signature coverage and completion. The helper remains an experimental
  API extension rather than a matching .NET member; arithmetic/runtime semantics and
  installed tools are unchanged.

- Exposed Int32.Parse through Raven with its existing Result<int,Int32ParseError>
  contract, typed matches, propagation and error predicates. Extended the shared
  carrier catalog without changing existing Ok<int> handling. Added boundary and
  unsafe-extraction checks, a saved-project sample and completion coverage. Documented
  disabling Raven's .NET parsing projections for this target. No parser semantics,
  Raven compiler or installed SDK/VSIX changes; other numeric/error APIs remain pending.

- Projected existing Path.Combine/GetFileName through the Raven target with a shared
  declaration/binding catalog, lexical-path sample, signature rejection checks and
  editor completion coverage. Host path semantics remain unchanged; Windows execution
  remains unverified. No Raven compiler or installed SDK/VSIX changes.

- Recorded the author's POC framing: visible rough edges, .NET familiarity,
  deliberate differences and unfinished API choices are useful material for evaluating
  the experiment. Updated preview/API design guidance while keeping support and
  test evidence explicit. Clarified the invitation for API feedback and the expected
  compatibility cost of Result-based error contracts.

- Exposed nine existing String methods through Raven: concatenation, ordinal
  comparison/search, equality, empty checks, UTF-8 byte counts and slicing. A shared member
  catalog generates declarations and checked bindings; guest adapters supply readonly
  receivers. Added readable and Unicode-boundary samples, invalid-call checks and
  editor completion coverage. Slicing shares a Result/error catalog with file APIs;
  range/boundary errors propagate and unsafe extraction is rejected. Error-carrier
  case constructors/accessors and ToString remain pending. No runtime semantics or
  Raven compiler changes, and no SDK/VSIX rebuild in these slices.

- Shared Raven bridge signature substitution and checking across file and collection
  calls, including constructors. Nested type arguments now use one recursive path;
  catalogs retain explicit API admission and Void stays distinct from no-result
  returns. Added 18 focused signature checks and verified existing execution paths.
  This is groundwork for broader runtime API projection, not additional API coverage;
  no Raven compiler or installed package changes.

### 2026-09-12

- Expanded the Raven POC plan to cover all existing public runtime APIs while
  retaining explicit compiler/CLR feature limits. Added a regenerable source
  declaration inventory and a coverage work plan; the inventory does not claim
  complete executable support. Documented a repeatable experimental distribution
  procedure with separately packaged Raven SDK/VSIX builds, focused integration
  checks and provenance instead of Raven's full release cycle. Recorded the author's
  directions and the remaining standalone bridge distribution gap; no release made.

- Projected bounded UTF-8 File.ReadAllText/WriteAllText into Raven with Result error
  predicates, typed matches and success/error propagation. The importer preserves
  conditional output initialization without inventing default error cases. Added
  temporary-file round-trip and rejection checks, saved-project execution and editor
  completion coverage. Extended named Void handling to generic return metadata;
  Raven's corresponding loader fix is isolated on its experimental branch. These
  calls require regenerated declarations and the updated compiler; installed .4
  packages are not claimed to contain this slice.

- Added an executable Raven match support matrix and readable Result/Option/Void
  samples. Six admitted fixtures verify and run; seven record compiler rejections,
  including exhaustiveness, deconstruction and arm-return limits. The bridge now
  admits definitely assigned string locals for match results and rejects uninitialized
  reads. Generalized the existing terminal-failure message beyond propagation.
  Documented two observations that need reconciliation with Raven's language docs;
  no Raven compiler or default .NET behavior changed.

- Extended Raven propagation to Option<Int32> and Result<Void,OverflowError>, including
  early absence/error returns and completion without a payload. Recorded Void's unit
  type semantics while retaining CLR no-result calls. Validate encoded Void storage
  separately from Cecil member resolution; reject CLI VOID markers in value slots.
  Added propagation workflow and editor checks. Raven's discarded-Void fix remains on
  its isolated target branch; ordinary .NET behavior is unchanged. Built and installed
  local Raven SDK/VS Code extension 0.1.12-neoclr.4; the prepared propagation demo runs,
  and installed-server checks verify target completion, constructors and inferred output
  types. Updated local setup instructions; these are not published release artifacts.

- Replaced ArrayList.Allocate with parameterless and initial-capacity constructors,
  including the Raven class projection. Both start empty; default capacity is zero,
  storage grows on demand and negative capacity faults. Updated callers and samples;
  no compatibility alias is retained. Raven also exposes Capacity for inspection.

- Added the runtime-library Propagatable<TSelf,TOutput,TResidual> extraction interface,
  implemented by Result and Option with readonly receivers and conditional output
  initialization. Added carrier FromResidual factories, including Option's Void residual.
  The static factory is provisionally outside the interface because static abstract
  interface members are not implemented. Documented that enforcement boundary and the
  bounded Raven target work. The bridge now runs `Result<int, OverflowError>` `?`
  with success extraction and early error return using target-selected Propagatable
  metadata, checked adapters and terminal faults for invalid carriers. Added repeatable
  execution and malformed-contract rejection checks. Option/Void propagation and an
  updated installed SDK remain pending. Four new runtime
  tests cover channels, Void, reconstruction and invalid output reads.

- Fixed pre-merge generic Clonable and Neo calculator regressions. Assembly field fixups
  now resolve layout with available constraints before complete linked validation;
  generic contract walks recognize repeated constructed types instead of recursively
  revisiting self-referential contracts. Constraint failures remain enforced. Primitive
  conversion syntax no longer gets intercepted by library constructor lookup. These
  restore existing behavior; no Raven target or library contract changes. Updated stale
  native-helper inventory assertions to include the existing enum helpers. Recorded a
  bounded preview acceptance matrix, including error flow, text files, date/time and
  target-aware VS Code completion. All 162 integration suites pass after the regression
  fixes and targeted reruns; unit/doc checks and the prepared Raven demo also pass.

- Normal CLI runs now emit only guest output; removed the automatic return-value suffix
  such as `=> Void`. Use `run --show-result` after the input path for opt-in return-value
  diagnostics on stderr. Scripts expecting the former stdout suffix must be updated.
  Updated CLI and Raven demo checks, and recorded propagation/text-file APIs as release
  requirements with reflection coverage desirable.

- Prepared local Raven SDK/VS Code extension 0.1.12-neoclr.3 from the isolated Raven
  feature branch, with a fresh combined demo workspace and updated try-it instructions.
  The SDK is installed alongside the default tools; the workspace selects the new server.
  Recorded plans for a NeoCLR preview bundle including matching Raven tools, runtime-owned
  propagation support and verified match syntax. Pre-merge testing found a Clonable generic
  assembly-resolution regression; main merge and public release remain pending.

- Fixed the Raven saved-project importer rejecting the advertised Int32 Math.Min,
  Math.Max and Math.Sign APIs. It now reuses the target declaration catalog with
  resolved-signature checks and executes the existing runtime methods. Added a sample
  covering integer limits, equal operands and all Sign outcomes to both project profiles.
  No runtime semantics, opcodes or Raven source changes.

- Combined the Raven collection declaration profile with Result/Option/Void and added
  a product-workflow demo using the existing adapted runtime library. Project checks
  cover the combined workflow, standalone unions, value-carrier copying, class/array
  aliasing and iteration. Editor checks now require both collection and union APIs.
  Regenerate older collection demo folders for the added declarations. No runtime
  opcode or Raven source changes; the compiler stays on its separate experiment branch.
  Documented the author's fundamental-demo scope and separate future Raven evaluation.

- Added a Raven for-loop demo using the feature-branch RuntimeIterationContract option
  to bind Iterable/Iterator/GetIterator rather than .NET enumerable names. The existing
  collection import profile executes normal completion, break and return; output is
  41, 42, 41, 41. Extended target selection to evaluated Raven project properties and the saved-project
  runner, which generates and verifies against the required collection library profile.
  Added a collection editor setup and actual LSP completion/loop-type checks; collection
  and existing Result/Option/Void project tests pass, including stale-output protection.
  Recorded missing automatic iterator Dispose and the proposed finally/defer mechanism
  as future work, per author direction. No cleanup behavior or Neo source changes are
  included. Future NeoCLR cleanup must leave default CLR support unchanged. Raven's
  four project configuration tests and nine related tests pass in this follow-up.

- Added CLR-compatible callvirt admission for nonvirtual nominal class instance methods,
  including closed generic classes and no-result calls. Null receivers fault before
  method entry; wrong types and byrefs to reference slots are rejected. This supports
  Raven's existing emitted calls without rewriting the instruction. Four new regressions
  and 49 related tests pass; nominal inheritance/virtual slots remain separate work.

- Added an isolated Raven collection library profile generated from existing System
  algorithms: nominal ArrayList/state/iterator classes, ordinary interface receivers,
  managed array buffers and no-result mutation/disposal. Runtime tests cover aliasing,
  growth, Copy, iterator GC retention, disposal and invalid accesses (seven new tests
  and 35 related tests pass). Neo and the default
  library remain unchanged. The profile requires defaultable element types and excludes
  predicate/delegate helpers. Documented how to generate and run it. Recorded proposed target-specific Raven language contracts
  for renamed iteration/disposal APIs while preserving the default .NET target; that
  project configuration is now implemented as described above.
  Follow-up admits two Raven Int32
  collection programs through an explicit import profile: ordinary class/interface
  calls and upcasts execute the adapted library, including iteration, growth, returned
  aliases and indexers. Six malformed/unsupported inputs reject; a null-default fixture
  verifies and faults at invocation. Added execution evidence and reproduction commands.
  Existing array and eight-program Result/Option/Void regression probes still pass.
  Raven and default VS Code project-profile configuration remain unchanged.

- Ran Raven-emitted static programs on neoCLR against the real System.Console through
  a bounded neoCLR-owned Cecil import bridge, with no Raven changes. Added verified
  empty/nested-call and Int32/local cases, invalid-input rejection checks, provenance
  maps and a reproducible runtime verification script. Direct PE loading and broader
  library/class-program support remain unfinished. Recorded staged VS Code development
  support as planned work, with target-aware Raven code completion as the editor MVP.
  Expanded the shared declaration/binding catalog to Math.Min/Max/Sign and integer
  Console.WriteLine, with an editable Raven sample and five verified runtime programs.
  Raven's completion API now passes checks against those target declarations without
  host API leakage; VS Code integration remains planned. Added a real Result/Option
  demonstration to MVP acceptance criteria; generic union import is not yet supported.
  Added an isolated Result metadata/type-pattern probe and incorrect-argument rejection.
  Recorded Raven's host-type-resolution emission blocker; the union sample binds but
  did not emit or execute in that slice. Kept probe-only declarations outside admitted
  runtime APIs. Follow-up now emits the sample after a feature-branch Raven fix for
  target metadata types and generic/byref member signatures. Corrected the probe's
  missing union-recognition Value property; verifies actual TryGetValue lowering instead
  of ordinary type tests. Added a bounded Result importer with control-flow stack and
  definite-assignment checks, case defaults, value-receiver/out-case adapters, and
  four malformed-input rejection probes. Raven's Math.Abs sample now executes against
  the real neoCLR System library and prints success (42) and overflow (Overflow).
  Added it as the sixth runtime regression program, with provenance and run instructions.
  Observable default Result carriers remain unsupported and are rejected. Recorded the
  POC priorities: own library, Result/Option, generic Void, and Raven VS Code completion.
  Extended the shared union import profile to Option<Int32>, case/carrier construction,
  supported case initialization and integer equality. A Raven price lookup now uses
  the real Option library and prints 42 and Product not found; seven runtime programs
  and three additional Option rejection probes cover the slice. Raven constructor
  emission support remains isolated on its feature branch. Added the eighth runtime
  demo: Raven Option<System.Void> constructs and matches Some(()) and None through
  a neoCLR-owned metadata projection, without further Raven changes. Ordinary no-result
  returns are unchanged; generic Void uses a named target type and the VM's existing
  inhabited Void marker. Added binary signature checks, three rejection probes, saved
  imports and run instructions. This is a bounded target adapter, not general CLI
  generic compatibility or a zero-stack optimization. Added project-backed editing
  with an explicit metadata core, reproducible LSP completion checks, a local VSIX build
  and installation walkthrough, plus a locally installed SDK alongside existing builds.
  Verified target Math suggestions in VS Code itself;
  normal Raven Build/Run buttons do not yet invoke the neoCLR import pipeline. Added
  dedicated VS Code build/run tasks using saved project sources, target emission and
  verifier-gated execution. Each build has fresh outputs; failed compilation/import
  cannot run stale IL. Verified Result/Option/Void, changed output and rejection cases.
  Initially recorded propagation as next; subsequent author direction puts the
  interface contract first and defers propagation. Closed the bounded Raven POC as
  an experiment checkpoint, with acceptance evidence, limitations and pinned Raven
  dependency for local tag milestone/raven-poc-2026-09-12; no preview release published.
  Clarified the broader desired demo: Raven should consume an adapted version of the
  existing library used by Neo, with shared implementations and matching declarations.
  Proposed existing collection/iteration contracts as the interface milestone scenario.
  Added the candidate interface contract and isolated Raven collection metadata probe:
  inherited callvirt emission and wrong-type diagnostics pass, while runtime admission
  remains deliberately rejected. Documented nominal interface dispatch and generic-class
  support as prerequisites to adapting the actual ArrayList/iterator library. Added
  non-generic nominal-class interface conformance, checked castclass views, inherited
  callvirt dispatch and identity-preserving GC-rooted interface fields/returns. Supports
  no-result interface contracts with exact return-mode matching and typed null faults.
  Ten new regressions and 108 related tests pass. Implicit storage
  conversions and collection import/adaptation remain unfinished; legacy borrowed
  interface behavior is retained. Added closed generic nominal classes
  using existing owner substitution for constructors, fields and interface dispatch.
  Nine additional regressions cover distinct instantiations, constraints, Void payloads,
  typed defaults and nested references surviving GC/artifact round trips; 96 focused
  tests pass for this generic-class slice. Constructor field-default
  limits (including String/arrays) remain; collection import/adaptation is still pending.
  Added ordinary interface-reference array elements and indirect interface-slot loads/
  rebinding, with typed null defaults, identity/GC retention and invariant element checks.
  Ten new regressions and 90 related tests pass.
  Added ordinary `arrayref<T>` storage for CLI-style arrays, including typed null field
  defaults, generic constructor allocation, jagged arrays and GC retention. Breaking
  neoIL change: `newarr` now returns an ordinary array reference; migrate old `T[]&`
  allocations to `array.new` or recompile Neo sources. Neo source behavior is unchanged.
  Runtime metadata identities distinguish both array forms; 13 new regressions and
  127 related tests pass. Added bounded Raven Int32 vector import and an executable
  alias/mutation/length sample, reusing Raven's standard IL and adding Array.Length to
  target declarations. Five malformed-IL imports reject without output; a typed-null
  local fixture verifies and faults at execution. Result/Option/Void compatibility checks
  still pass. Documented reproduction and target-core refresh instructions. Broader array
  import, String defaults and collection adaptation remain pending. Added implicit
  nominal-class/interface upcasts at runtime assignments, calls, returns, fields and
  array/indirect stores, preserving identity, typed nulls and GC roots. Byrefs and whole
  array types remain invariant; downcasts still require castclass. Eight new regressions
  and 104 related tests pass. Raven class/interface
  import and adapted collection execution are still pending. Clarified the target
  architecture: retire value-by-default/explicit-reference ordinary semantics in favor
  of .NET type categories, including reference-type arrays. Documented the existing
  Neo array split as transitional and planned its migration; no runtime behavior changes
  are made by this clarification. Recorded .NET semantics as the baseline unless a
  concrete improvement justifies divergence; Result handles recoverable failures and
  terminal host faults do not form a guest Exception class hierarchy. Corrected stale
  API/migration and CLI Void policy text; current Fault representation already follows
  the host-diagnostic model. Documentation only; cleanup/containment contracts remain open.
  Corrected experiment scope after author clarification: adapt neoCLR and its library
  for Raven; leave Neo outside this exercise. Removed the uncommitted Neo migration
  attempt and its helper instructions, and revised migration plans accordingly. Existing
  committed runtime and Raven behavior is unchanged.

- Began nominal class reference semantics: `.type class` uses managed heap handles in
  ordinary class-typed storage, while managed byrefs still address slots. Assignment,
  static calls, fields, return values and GC preserve object identity; existing value
  records still copy contents. Added alias/value-copy and .NET comparison tests.
  Added class instance constructors/direct calls with no-result bodies and constructor
  results supplied by newobj; class field stores now consume operands without a result
  (remove the previous trailing pop). GC retains objects throughout construction.
  Decodes standard constructor/field tokens and adds a runnable Console example.
  Added typed null defaults for nominal class fields/slots, alias-safe initobj reset,
  null comparison, null field-access faults and GC handling. Uninitialized slots remain
  distinct. Inheritance and Raven binary execution remain unfinished;
  legacy value/byref field-store behavior and System API returns still need migration.

- Added a bounded standard CIL code-stream decoder for the static-call experiment,
  preserving byte offsets and tokens and rejecting unsupported/truncated instructions.
  Added bounded PE32 CLI container and tiny/fat method-body inspection with .NET fixture
  comparisons; rejects truncated/overlapping ranges and unsupported header modes.
  Added a reproducible .NET-emitted byte fixture with matching neoCLR execution through
  test-only binding; PE loading and production token resolution remain planned.
  Clarified that real Void type participation need not occupy an evaluation-stack slot;
  the existing generic representation has not yet been migrated. Updated the experiment
  order to prioritize .NET value/reference type semantics and a useful Raven scenario,
  while retaining standard external metadata and instructions wherever possible.

- Added explicit no-result metadata and assembly syntax for static, non-generic IL
  methods. The existing call/ret instructions now preserve CLI-style empty results;
  verifier/runtime reject invalid stacks and unsupported return-mode combinations.
  Existing inhabited Void behavior remains the default. Hosting and reachability
  expose the return mode; generic Void storage/returns remain intact. Documented the
  separate declared-type/return-convention layers and runtime-async design comparison;
  binary loading and broader dispatch support remain planned. Recorded intended
  later alignment with .NET value/reference type semantics; no type-model migration
  is included in this slice.

- Added the first Raven integration map, pinned to the inspected compiler revision.
  Identified existing core-library retargeting, reflection/PE emission dependencies,
  framework discovery and semantic gaps in Void/Unit and error projections. Documented
  the next minimal-target probes. Added a reproducible emission probe and minimal
  contract map: target Console binding succeeds, while omitted-library resolution
  exposes host fallback and core declarations remain incomplete. Captured metadata,
  diagnostics and call-boundary alternatives; binary execution is not yet claimed.
  Added explicit-only metadata dependency auditing with positive and
  negative fixtures, and selected CLI PE reuse for the bounded binary experiment.
  Added probe coverage for Raven's opt-in explicit-only metadata import API: Console
  binds when supplied and is unavailable when omitted, even after host-mode binding.
  Added a minimal core reference assembly and core-only Console, empty/nested-call
  and Int32-return emission probes, with dependency and negative binding checks.
  Corrected application AssemblyRef inventories to exclude references synthesized
  by Cecil during inspection. Runtime loading/translation and executable System
  binding remain planned. Recorded the workflow requirement to keep Raven work
  isolated on a feature branch.

- Documented the Raven target/binary-artifact experiment on its own branch: inspect
  the existing compiler, choose an evidence-backed format/library contract, and
  compile/run HelloWorld against neoCLR's own System library. Recorded deliberate
  Result-based migration differences and acceptance criteria; no backend or binary
  loader implementation is included yet.

- Added a bounded reference-defaults architecture evaluation comparing the order
  workflow under current, shared type-default and language-only models. Includes
  illustrative source/lowering, .NET comparison, migration costs and a proposed
  compiler/import experiment. Validated current behavior with 22 focused tests,
  emitted IL and the pinned .NET baseline; no runtime or compiler semantics changed.
  Clarified after review that runtime improvement and useful CLR compatibility lead
  the evaluation; Neo demonstrates/tests the contracts. A runtime-contract assessment
  precedes the optional compiler experiment, with compatibility scope still open.

- Recorded evaluation criteria centered on familiar developer experience, possible
  type-level reference/value defaults, Result/Option APIs, Faults, nullable support
  and language/runtime boundaries. Identified migration questions without selecting
  or implementing an architecture change.

### 2026-09-09

- Added a development conversation record of author directions/questions, assistant
  proposals, subsequent decisions and actions/outcomes, in one chronological document covering
  the original shared chat from the founding brief through Preview 1. Earlier
  continuation notes explicitly identify unavailable assistant replies; recent exchanges retain attribution and open issues.
  Added workflow instructions to maintain it as a minutes-like record, separately from
  the implementation changelog, without inventing dates or missing conversation. Expanded the
  timeline with available founding exchanges on memory policy, signature/name mapping,
  ordinary union cases, output-reference corrections, equality, release scope, Windows
  CI diagnosis and verified Preview 1 publication. Preserved attribution, the distinction
  between unavailable replies and implementation evidence, and later changes of direction.
  Recorded the subsequent discussion of reference defaults and performance-oriented
  complexity, with .NET/Valhalla sources and an explicit distinction between runtime
  uniformity and demonstrated usability. Recorded the author's exploration
  of ordinary class references and opt-in value behavior, separating it from the
  assistant's metadata proposal, and linked existing evaluation documents. The author
  clarified that this is not a selected direction; the current model remains in place. No runtime
  or language contract migration is implemented by this documentation change. Extended
  the exploration with transparent class usage, value-design intent and Java wrapper
  evolution, distinguishing lifetime, identity and preview status. Added the clarification
  that lightweight values need not be short-lived and the preference for potentially
  inline value classes over a separate struct concept, without selecting a migration.
  Recorded the follow-up separation of storage, mutation, identity and equality using
  C# records as a comparison. Recorded the exploration of runtime contracts versus
  conventions, tentative inline/value metadata, and the clarified priority of
  object-like primitive and user-defined types with familiar .NET usage. Proposed
  syntax and evaluation work remain explicitly unselected and unimplemented.

- Added emit-il for inspecting Neo compiler output on stdout or in a new neoIL file,
  preserving original source sequence points. It validates without executing and
  refuses file overwrites. Added CLI round-trip/error regressions and debugging
  instructions. JSON artifact disassembly and binary encoding remain future work.

- Added initial Int32-backed enum/flags metadata and Neo declarations, named constants,
  typed bitwise/equality operations, explicit integer conversion and zero/unnamed values.
  Runtime validation enforces enum layout and nominal calls; enum construction accepts
  the underlying integer without granting private payload access. Added IsEnum,
  GetEnumNames and GetEnumUnderlyingType reflection APIs. Migrated BindingFlags to enum
  metadata while retaining its existing factories, bit values and filtering policy.
  Added artifact/source regressions, an example and a pinned .NET comparison probe.
  Older tools reject the new format-5 metadata and Neo reserves enum; other integer
  widths, formatting and general const/literal-field support remain future work.

- Added generic Neo union carriers composed from independent source case types, with
  substituted constructors, exact case conversions, exhaustive matching and conditional
  bindings. Cases retain their own generic parameters and existing reference/value
  contracts. Explicit constructor type arguments are required; inline generic cases
  remain deferred. Added an example, grammar/API notes and artifact regressions.
  Planned enums/flags with BindingFlags as the first API migration, plus CLI inspection
  of emitted Neo IL; binary encoding remains future work.

- Added initial runtime-enforced notvoid/notreference generic constraints on types and
  methods, neoIL .constraint directives and Neo where clauses. Restrictions apply to
  the outermost argument; symbolic forwarding is rechecked at concrete resolution.
  Extended this with nominal base/interface bounds, substituted/scoped bound metadata,
  and Neo constrained method lookup through explicit T& receivers. Calls reuse checked
  reference views and ordinary dispatch, preserving readonly access and concrete identity.
  Added metadata/host/source/IL regressions, an executable example and a reproducible
  .NET comparison probe.
  Added constrained T& conversions for API arguments, assignments and returned
  base/interface views, with inherited conformance and readonly/lifetime enforcement.
  Included a view/identity example and .NET reference-versus-boxing comparison,
  distinguishing platform capabilities from high-level language usability.
  Added constrained calls on addressable T values with notreference, reusing slot
  addresses and views with existing binding mutability, shallow copy and escape rules.
  Included value-copy and receiver-evaluation regressions and an executable example.
  Added preview ldreceiver adaptation for open generic parameter/local receivers,
  preserving stored reference identity or borrowing value storage with a chosen readonly
  capability. No boxing or nested managed references are introduced. Fixed inferred
  readonly-reference call signatures; added raw IL/artifact/Neo validation and CLR comparison.
  Broader receiver operands and member lookup remain later work; notnull awaits
  nullable metadata. Older tools reject constrained artifacts; Neo reserves the new keywords.

- Added plain generic Neo record declarations with explicit positional construction,
  substituted fields, generic function forwarding and heap construction. Emitted
  generic definitions reuse the existing runtime metadata and lifetime rules; reference
  payloads preserve identity. Added an example, grammar/API guidance, .NET comparison
  and regressions. Generic record methods/classes, source unions, inheritance and
  source constructor inference remain separate slices.

- Updated the order workflow to use imported inferred Ok/Error cases and conditional
  Option/Result bindings, with explicit PurchaseError conversion for nested errors.
  Added zero-quantity scenario coverage and regressions for rejected-purchase side effects,
  exact stock exhaustion, lookup identity and receipt snapshots. Documented the
  remaining nested-carrier annotation cost and next generic-source-type investigation;
  runtime reference, inference and conversion contracts are unchanged.

- Added wildcard imports of marked bundled union cases in Neo: `import System.Result.*`
  enables `Ok(42)`/`Error("message")`, and `import System.Option.*` enables Some/None.
  Public carrier constructors supply the imported case definitions. Short names work
  in type positions and explicit/inferred constructor calls, preserving shadowing,
  ambiguity diagnostics, reference intent and runtime lifetime checks. Updated the
  example and design guidance, with nested inference and lookup regressions.

- Added argument-based generic library constructor inference in Neo, so
  `System.Result.Ok(42)` constructs an independent `Ok<int>` before any carrier
  conversion. Inference preserves reference payloads, requires evidence for every
  owner parameter and never uses a carrier target to override argument types.
  Explicit type arguments remain available; arbitrary external assembly imports
  remain future work.
  Updated the example, language guidance and Raven/.NET comparison, with regressions
  for nested inference, once-only evaluation and incompatible carrier targets.
  Split source-call and constructor lowering to reduce recursive compiler stack use.

- Added implicit Neo case-to-carrier conversion for marked bundled union types using
  the unique public constructor accepting the exact case value type. Expected types
  in annotations, returns, arguments and storage supply the carrier; ordinary unmarked
  constructors do not enable conversion. Added Raven implementation research, an
  updated case example and type/reference/lifetime regressions. Runtime instructions
  are unchanged.

- Added explicit public library constructor calls in Neo, including standalone
  System.Result.Ok<T>/Error<E> and System.Option.Some<T>/None values and explicit
  overloaded carrier construction. Metadata supplies parameter context for reference,
  readonly, delegate and Void payloads; arguments execute once. Added an example,
  API/grammar guidance and regressions.

- Added file-wide Neo imports of declared source union cases, including constructor
  calls and type annotations, with duplicate-import handling, ambiguity diagnostics
  and local/declaration precedence. Updated the order workflow to import PurchaseError
  cases and documented grammar and .NET lookup comparison. Generic source union
  declarations remain future work;
  emitted case identities and runtime contracts are unchanged.

- Added a reference-view example covering generic forwarding, base/interface views,
  readonly fields and collection elements, with identity/virtual-dispatch checks,
  GC-pressure coverage and readonly-container write rejection. Documented the distinction
  between replacing a stored readonly reference and writing through a readonly holder.

### 2026-09-08

- Changed Neo reference-to-reference assignment to copy the reference into mutable
  locals, captured bindings, writable fields and array slots, consistent with binding
  initialization and reference-valued collection setters. Breaking source change:
  use an explicitly value-typed intermediate for the former referent-copy behavior;
  immutable bindings now reject reference RHS reassignment. Value RHS and output
  writes retain target semantics. Added migration guidance, .NET comparison and
  assignment/storage/lifetime regressions; existing compiled IL is unchanged.

- Added a reference-passing example and regressions for alias forwarding, explicit
  address creation/local retargeting and output writes. Corrected the new type-design
  notes: current `out Foo&` initializes Foo storage, not a caller's Foo& binding;
  reference-slot replacement remains unsupported. Confirmed explicit reference creation
  instead of implicit argument borrowing and documented the existing assignment difference from C#.

- Added Neo `if let` and `let … else` case bindings with single evaluation, scoped
  immutable bindings and checked failure-path exits. The order workflow now uses
  a custom PurchaseError union and guards instead of nested success matches.
  Added grammar, usage documentation and control-flow/capture regressions; runtime
  instructions are unchanged. Recorded contextual borrowing as a future language
  experiment, without changing explicit reference syntax.

- Added non-generic Neo union declarations from existing source types or inline
  nested record cases. Carriers generate one constructor per variant, support exact
  case-to-carrier conversion and exhaustive whole-variant matching, and retain the
  existing erased-storage/GC rules. Added a sample, metadata/lifetime tests and type/API
  design guidelines. `union` and `case` are now reserved identifiers. Generic case
  inference and imported shorthand remain future work; no new runtime opcode is added.

- Added ordinary Result<T,E>.Ok/Error library factories, usable from Neo with an
  explicitly closed owner such as Result<int,string>.Ok(42). The order workflow now
  uses Result instead of a placeholder outcome record. Unique library static signatures
  now supply argument context so managed-reference payloads retain identity. Typing and frame
  storage checks remain unchanged. Documented imported case construction, generic
  inference and declared-case carrier conversion as a separate compiler plan, with
  standalone variant types and both existing-type and inline-case union declarations.

- Changed ArrayList assignment to share count and buffer coherently across growth
  using managed backing state. Added readonly Copy() for independent sequences with
  shallow element copies. Breaking library layout/behavior change: rebuild artifacts
  and replace reliance on independent copied counts with Copy(). Each independent list
  now allocates an additional managed state object; no runtime opcode changed.
  GC-pressure tests, including reflection collections, account for that retained state.

- Added an executable order-workflow experiment and a .NET 10 comparison to assess
  explicit managed-reference ergonomics, with focused tests for receipt snapshots,
  list-copy aliasing and callback capture lifetimes. Recorded observed friction,
  language/library distinctions and provisional alternatives; no runtime contracts
  changed. See [reference experience](docs/experiments/reference-experience/README.md).

## 0.1.0-preview.3 — 2026-09-08

A source-only preview focused on runtime-library foundations and the object/callable
features needed to exercise them in Neo. See [release notes](docs/preview-3-release-notes.md)
for examples, compatibility guidance and exact-commit validation requirements.

### 2026-09-08

#### Added

- Runtime-enforced readonly managed parameters, receivers and storage signatures,
  including fields, locals, arrays, returns and generic arguments. Derived references
  preserve permissions; writable-to-readonly narrowing is allowed and permission
  escalation faults. Neo adds readonly syntax; reflection and debugging expose the
  qualifiers. Readonly remains shallow and does not imply immutable backing memory.
- Class/record inheritance and inherited value layout, managed base-reference views,
  virtual/override dispatch and abstract classes/methods. Base views retain the complete
  owner and prevent value slicing. Type.BaseType returns Option<Type>. No mandatory
  Object base, implicit boxing or public downcast facility is introduced.
- Managed constructor chaining with unpublished construction storage, base completion
  and initialized-field checks. Neo adds ordinary classes, body fields, initializers,
  init declarations, bounded parameterless-constructor synthesis and default(T).
  Defaults do not invoke constructors or invent invalid references. Base-first
  initializer ordering deliberately differs from C#; positional records remain.
- Interface inheritance, inherited class implementations, explicit declaration-to-body
  mappings and default interface bodies, including most-specific dispatch, diamond
  ambiguity checks and reabstraction. Neo supports qualified implementation bodies;
  managed receiver lifetime, readonly and output contracts remain enforced.
- Generic free functions and static methods with independent method parameters,
  explicit type arguments, substitution, host invocation and closed call graphs.
  Neo adds namespace functions, static members and argument-based generic inference.
- Nominal generic delegates with checked delegate.bind, Invoke calls, heap receiver
  retention, virtual/interface binding and GC tracing. Neo adds delegate declarations,
  contextual method-group conversion, function-style invocation and contextual lambdas.
  Closures use managed capture cells/environments, share captured mutable bindings,
  and create fresh range-loop captures; unsafe frame/output/constructor captures are
  rejected. Func supports zero through four inputs, including Void results, and
  Array.ForEach consumes Func<T,Void> without a separate Action family.
- Comparable<T>, readonly comparison/equality receivers, Iterable<T> and Iterator<T>
  with Disposable, and inherited Iterable support on List<T>. ArrayList iterators
  retain their original buffer and extent; writes remain visible, unlike .NET List's
  mutation invalidation. Find, FindIndex and Exists use Func predicates; Find returns
  Option<T>. Added Neo conformance to bundled interfaces and library callback examples.
- Abstract MemberInfo as the shared base of MethodInfo, FieldInfo and PropertyInfo,
  with readonly managed readers and internal chained constructors. Reflection adds
  base/abstract/virtual/override/readonly facts and transitive interface introspection;
  member queries remain declared-only metadata snapshots.
- String.CompareOrdinal over UTF-16 code-unit ordering, plus readonly ContainsOrdinal,
  StartsWithOrdinal and EndsWithOrdinal over valid UTF-8 storage. Added Unicode 16
  System.Char predicates and Neo single-UTF-16-unit character literals, including
  surrogate escapes; pinned category data is reproducible and attributed.
- Int32 Math.Min/Max/Sign and Result-based Clamp; fundamental Double arithmetic,
  rounding, exponential/logarithmic and trigonometric helpers. Preserved documented
  .NET NaN/signed-zero behavior and ties-to-even rounding. Neo adds finite Double
  literals and same-type numeric operations, without implicit widening.
- Separate Date and Time values with validated Result factories, readonly components,
  equality/ordering, Gregorian day numbers and 100 ns ticks. Clock.GetLocalNow captures
  local Date, Time and UTC-offset seconds in one host reading using Chrono. Documented
  precision, timezone fallback and the trusted host-import boundary.
- Read-only Environment APIs for per-execution guest arguments, Result-based current
  directory and Result/Option variable lookup. CLI run/debug forward arguments after
  `--`, with the guest input path first. Missing and empty variables remain distinct.
- Lexical Path.Combine/GetFileName and bounded UTF-8 File.WriteAllText returning
  Result<Void,FileWriteError>. Output byte limits are checked before opening; writes
  create/replace regular files without adding a BOM and are not atomic. A runnable
  file-report example combines arguments, paths, bounded I/O and Result handling.

#### Changed and migration

- Neo arrays accept checked local extents and optional heap initializer braces;
  samples omit empty braces. Single-index Item properties use bracket syntax,
  including interface dispatch and reference-valued elements.
- Recompile applications and external System libraries together. New readonly,
  inheritance, constructor, implementation-mapping and delegate metadata/opcodes need
  this runtime; the JSON format remains provisional. Rust metadata literals and
  exhaustive enum matches may need updating.
- Equatable, scalar Equals, comparison contracts and reflection descriptor readers
  now use readonly managed receivers. IL callers/implementers must match them; Neo
  borrows automatically. List implementations must supply inherited GetIterator and
  the current readonly getter contracts. Host wrappers must supply managed receivers.
- Derived constructors use managed receivers. Managed stfld produces Void; the
  value-form instruction remains a record update. Whole-value reads, resets, writes
  and out through projected base views fault to prevent slicing.
- Neo reserves class, default and readonly and generated neoCLR.Compiler names.
  Closures incur managed heap allocations. Rust ExecutionOptions gains arguments;
  exhaustive literals must initialize it or use defaults.
- Runtime immutable-slot proposals were superseded: immutable bindings remain a
  language feature. Nullable type signatures for both values and references are
  planned, with null distinct from zero/default and uninitialized storage; they are
  not implemented. Option remains preferred for domain optionality.
- Kept future enums/flags, constraints, async, dynamic hooks and broader framework
  growth on a research-backed .NET/CLR roadmap. LINQ, globalization, date/time
  parsing/formatting, generic instance methods, multicast/variance, reflective
  invocation and broader native/resource lifetime features remain later work.

#### Validation and packaging

- Expanded archive validation to 26 Neo source/artifact scenarios plus native interop,
  including semantic checks for clock readings, guest arguments and report output.
  CI covers Linux/macOS/Windows on stable and minimum Rust 1.85.0; publication requires
  a successful six-job run on the exact versioned candidate.
- Refreshed notices for all 47 locked registry packages with 94 byte-hashed license
  texts, preserving upstream bytes across checkouts. Added Char data attribution,
  API/.NET comparison probes, migration guides and synchronized Neo grammar/examples.
- Fixed the legacy Math.Abs boundary assertion to select its Int32 parameter signature
  rather than rejecting the Double overload. Recorded local validation and archived
  evidence; published Preview 1 and Preview 2 history remains unchanged.

## 0.1.0-preview.2 — 2026-09-08

Source-only prerelease. See [release notes](docs/preview-2-release-notes.md).

Changes since [v0.1.0-preview.1](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.1).
Publication requires the exact-commit validation linked from the GitHub release.

### 2026-09-08

#### Fixed

- Source validation disables Git line-ending conversion when archiving, preserving
  notice hashes across Windows and Unix. Its text I/O explicitly uses UTF-8, and
  CI runs every platform job even if another job fails.

#### Added

- **Preliminary reference identity:** Neo ReferenceEquals and IL ref.eq compare
  initialized managed locations across frame/heap storage, concrete/interface views,
  and interior paths. They preserve value equality behavior and reject raw pointers;
  no Object root, boxing, native address or identity hash is introduced.

- **Neo output parameters:** unconditional `out name: Foo&` declarations, explicit
  `out destination` arguments, and typed uninitialized var locals now project the
  existing runtime contracts. Source/interface forwarding and conditional library
  TryGet calls are demonstrated end to end. Caller initialization is checked by the
  verifier; callee read/assignment obligations remain runtime-enforced.

- **Reflection introspection:** added live managed-reference GetType discovery through
  `ref.type`, including concrete interface targets and interior field types, without
  boxing or retaining inspected objects. Existing declared methods keep their dispatch.
  The collection example now checks its concrete implementation through an interface.
  System.Type now enumerates fields, methods and
  properties through independent FieldInfo, MethodInfo and PropertyInfo records.
  ParameterInfo exposes names, positions, types and output contracts. Queries support
  a documented BindingFlags subset, accessor metadata, closed generic substitution,
  implemented interfaces and array/reference/pointer shape inspection. Metadata
  queries preserve managed-reference signatures and do not invoke accessors or retain
  guest objects. Includes Neo/IL examples and an [API guide](docs/reflection.md).
- **Interactive terminal debugger:** launch with `neoclr debug` to inspect call
  frames, arguments, locals, evaluation stacks, managed heap objects, GC counters and
  tracked native memory. Supports pause/continue, breakpoints, IL step-into, source
  step-over, step-out and bounded live snapshots. Neo source mappings survive artifact
  round trips and appear in Fault traces. Includes a controller API for embedding.
  See the [debugger guide](docs/debugger.md).
- **Managed and frame-owned arrays:** T[] owns copied elements; T[]& references an
  array location. Added newarr, array.create, ldlen, ldelem, stelem and ldelema, with
  bounds/type checks, lifetime validation, GC tracing and payload budgets. Neo exposes
  array literals, fixed-length annotations, heap allocation, indexing, Length and
  element references. The native System.Array<T> buffer API remains separate.
  See [managed arrays](docs/managed-arrays.md).
- **Neo interface declarations and conversions:** records can implement interfaces,
  and existing managed references can convert to implemented interface views for
  virtual dispatch. Views preserve frame or heap provenance without boxing.
  See [Neo interfaces](docs/neo-interfaces.md).
- **Reproducible candidate validation:** added a source-archive validator that checks
  tracked membership and dependency notices, runs all test targets on a selected Rust
  toolchain, compares nine Neo source/artifact programs and exercises native interop.
  The six existing CI jobs now run the archive check and retain reports/checksums.
  Actual publication still requires successful evidence for the selected commit.
- **Release changelog workflow:** reconstructed the published baseline and subsequent
  development history. Added a per-commit update rule, same-date consolidation and
  protection of published entries, with repository instructions for future work.

#### Changed

- **Fresh Neo declaration storage:** repeated local declarations and compiler
  temporaries now reset their storage before initialization, allowing differently
  sized array and nested-record results on successive loop iterations. The new
  `local.reset` instruction faults while aliases to the old local remain live;
  ordinary assignment retains its existing shape and reference-preservation rules.

- **ArrayList<T> now uses a managed T[]& backing array.** It grows automatically and
  no longer requires Free. Checked `array.alloc T` reserves uninitialized capacity
  without inventing default elements. Generic reference arguments and reference array
  elements allow ArrayList<Foo&> to store Foo references; GC traces initialized slots.
  All collection instance methods use managed-reference receivers. Copying a list
  descriptor copies Count and shares its backing array until growth; an explicit
  ArrayList<T>& shares the whole mutable descriptor. See [ArrayList](docs/array-list.md).
- **Reference-aware library calls in Neo:** added closed generic static member calls
  and implicit reference conversions for implemented bundled interfaces. A complete
  Neo ArrayList<Counter&> example now creates its owner, shares references through List,
  mutates a referenced object and demonstrates independent copied Count state.
  Existing library receivers use their declared
  value/reference contract, and unique instance methods supply contextual parameter
  types, including reference arguments. Exact-signature overload selection remains
  the current subset. Added the [library API design guide](docs/api-design.md).
- **Cross-layer lifetime documentation:** consolidated runtime and Neo rules for
  frame ownership, heap reachability, interior references, automatic managed-reference
  access and checked returns. Clarified the distinction between reference binding
  lifetime, target lifetime and resource cleanup. Reconciled older generic-reference
  and GC-monitoring descriptions with implemented arrays and debugger inspection.
  See [managed-reference semantics](docs/managed-reference-semantics.md).

### 2026-09-07

#### Added

- **Tracing garbage collection:** a single-threaded, nonmoving mark-and-sweep
  collector reclaims unreachable heap graphs, including cycles. Active frames,
  returned values, interior references and interface views preserve the required
  roots. Added heap statistics, bounded collection-event history, and CLI
  `--gc-stats` / `--gc-events`. Native allocation/free remains explicit and separate.
  See [garbage collection](docs/garbage-collection.md).
- **Managed-reference locals and returns:** checked T& locals, heap-backed reference
  fields and returned references extend the earlier call-scoped model. Returning a
  reference into an active caller or managed heap is supported; returning an address
  into the current frame faults, including indirect escapes. References neither
  promote locals nor require manual invalidation. See [reference slots](docs/reference-slots.md).
- **Managed initobj destinations:** default-initialize supported typed frame slots
  and addressed fields without a constructor or extra allocation. Initialization can
  satisfy output contracts. Unsupported defaults fault rather than manufacturing
  null managed references. See [managed initialization](docs/managed-initialization.md).
- **Neo companion compiler:** a small Raven-inspired language compiles `.neo` source
  to neoCLR, with records, functions, value/reference parameters, field access and
  explicit managed heap allocation. Added if/else, while, integer-range for, loop,
  break/continue, union-aware match expressions/statements and typeof(T). Public
  read-only System properties project through ordinary getters. Includes a bounded
  console calculator, executable examples, [language guide](docs/neo.md) and
  [grammar](docs/neo-grammar.md). Neo is maintained alongside runtime development.
- **Explicit lifecycle APIs:** System.Clonable<T>.Clone(), System.Disposable.Dispose()
  and System.Closable<E>.Close() are ordinary managed-reference interface contracts.
  Clone is independent of value copying; Dispose/Close require explicit calls.
  Includes [cloning](docs/cloning.md) and [disposal](docs/disposal.md) examples.

#### Changed

- **One managed-reference representation:** heap.new now returns T& directly, sharing
  ldobj, stobj and ldflda with frame-owned references. Removed the older Ref<T> /
  heap.load / heap.store path and advanced JSON artifacts from format 4 to format 5.
  newobj continues to construct values; explicit heap.new establishes heap storage.
  See [heap references and migration](docs/heap-references.md).
- **Transparent managed-reference access in Neo:** reading or assigning through T&
  automatically reads or updates the target. Forwarding preserves reference identity;
  reference formation remains explicit. Earlier development samples using `*age`
  now use `age`. Raw pointers retain separate low-level semantics and are not yet
  exposed by Neo syntax. Preview 1 itself did not contain a Neo compiler.
- **Documented platform direction:** clarified value/reference addressing, allocation
  and cleanup contracts while retaining CLR instruction/API familiarity. Recorded
  future designs for an optional Object hierarchy, native pinning and interop, and
  possible Neo library/compiler bootstrapping. These designs do not implement those
  features. See the [roadmap](docs/neo-roadmap.md) and [lifecycle design](docs/lifecycle.md).

### Migration notes for Preview 2

- Reassemble format-4 artifacts, including external System libraries, against the
  current runtime. Source and library artifacts must agree on signatures.
  New development instructions local.reset, ref.type and ref.eq require the current
  runtime; format 5 is still a provisional format, not a stable compatibility promise.
- Replace Ref<T> with T&, heap.load with ldobj T, and heap.store with stobj T.
  Remove a following pop if it consumed heap.store's old Void result: stobj produces
  no stack value. The runtime service is now ManagedHeap; heap.new also uses SlotReferences.
- Remove ArrayList.Free calls. Pass the collection receiver by managed reference.
  Use ArrayList<T>& to share one mutable list, or account for the documented
  descriptor-copy/backing-array sharing behavior. Native System.Array<T>.Free remains
  part of the separate native-buffer API.
- Earlier Neo development snapshots must remove explicit managed dereferences.
  Returning &local from its owning function remains invalid; allocate explicitly
  on the managed heap when a reference needs an independent lifetime.

### Remaining preview limits

- Guest destructors, finalizers, automatic scope cleanup and distinct runtime block
  lifetimes are not implemented. GC does not call Dispose or Close.
- Arrays retain fixed shape when replaced, including nested array fields. Neo renews declaration storage on each iteration; assignments to an existing
  array location still preserve its shape.
- The debugger requires launch-time integration; it does not attach to arbitrary OS
  processes or provide expression evaluation, memory editing, native-frame unwinding
  or editor/DAP integration.
- Reflection is metadata-only. Reflective invocation,
  field mutation, constructor queries, class hierarchies and module-level FunctionInfo
  queries remain future work.
- The compiler and library remain bounded prototypes. JIT/AOT execution, managed-object
  pinning, persistent host roots, cross-execution managed-reference inputs and a stable
  native ABI are not provided. New release claims require validation of the selected
  candidate; Preview 1's CI evidence does not certify current development.

## 0.1.0-preview.1 — 2026-09-07

Initial source-only prerelease, tagged at
[f11ec01](https://github.com/marinasundstrom/neoCLR/commit/f11ec01).
This historical section describes that tagged tree. The
[published release notes](docs/preview-1-release-notes.md) retain its full scope,
validation references and limitations.

### Added

- Standalone Rust interpreter, neoIL assembler, JSON metadata loader and optional
  typed control-flow verifier, using familiar CLI instructions where applicable.
- Primitive arithmetic/conversions, control flow, ordinary records, constructors,
  overloads, generic types, accessibility, properties and indexers.
- Platform-written System library with console and bounded file input, strings,
  numeric operations, ordinary Option/Result carriers and typed recoverable errors.
- Call-scoped managed references, output and conditional-output contracts, explicit
  reference receivers, interface views, List<T> and Equatable<T>.
- Native heap/pointer operations, frame-local localloc, native buffer arrays,
  native-layout ArrayList<T> with explicit release, and scalar/pointer P/Invoke.
- Minimal read-only type inspection, experimental Rust embedding, Fault stack traces,
  executable walkthroughs and source/artifact acceptance coverage.
- MIT licensing, locked dependency notices and source-preview packaging instructions.

### Compatibility and scope

- Prototype JSON format 4; no .NET assembly importer or stable artifact/ABI guarantee.
- Source-only distribution requiring Rust 1.85 or later and native build prerequisites.
- No Neo compiler, tracing GC or automatic destruction in this tagged preview.
  Raven-like examples in its documentation are explanatory pseudocode.

## Reconstruction provenance

The initial backfill on 2026-09-08 used the published tag and release notes, then
reviewed the 27 commits from that tag through reflection commit
[da2966d](https://github.com/marinasundstrom/neoCLR/commit/da2966d).
Development dates follow those commits. Related design, implementation and follow-up
documentation commits are consolidated into feature entries; superseded proposals
are not presented as implemented behavior. Subsequent commits maintain Unreleased
using the [workflow](docs/changelog.md).
