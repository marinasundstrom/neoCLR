# Changelog

Notable changes to neoCLR, grouped by release and development date. Every commit
updates the Unreleased section; related changes on the same date share an entry.
Published sections are frozen. See the [maintenance workflow](docs/changelog.md).

## Unreleased

### 2026-09-24

- Investigate String storage and identity with an allocation-counting Release probe,
  a private shared-text prototype and a .NET alias/record/array baseline. Record
  migration, GC, host-lifetime and accounting gates, and consolidate stale roadmap
  status. Implement immutable shared UTF-8 String payloads: value copies and String
  Object display retain text; owned producers transfer their buffer. Keep String
  identity disabled, worker transfers owned and array byte quotas logical. Rust hosts
  migrate owned constructors to `Value::String(text.into())` and extraction to
  `into_owned()`. Cover GC/host lifetimes and record future System.Text and general/
  string comparers, casing and comparison methods without introducing those APIs.
  Use .NET as a design reference, allowing justified API differences. Add direct VM
  owner-retention checks across Object/interface conversions, display, fields, arrays,
  erasure and local byrefs, including GC pressure and host/cyclic/fault teardown.
  Keep guest identity disabled pending a complete comparison/hash contract.

- Support String Object content equality, matching UTF-8 hashes and unchanged text
  display through existing wrappers. Cover Object map keys, casts and GC; retain
  explicit rejection of String identity operations. Add generated reference for
  existing String methods/properties/operators and document the representation and
  provisional hash limitations. Collect before casts/type tests allocate String
  wrappers, retaining stack roots; default class/array equality rejects String
  operands without invoking identity. No compiler or managed layout change.

- Align boxed Char equality, hashing and display with its exact grapheme text,
  preserving copied values and distinct box identity without Unicode normalization.
  Add combining-text/emoji map and GC checks, and generated reference coverage for
  Char’s existing typed methods. Unlike .NET Char, the value remains a grapheme,
  not a UTF-16 code unit.

- Add boxed Single/Double Object equality and hashing with .NET-compatible NaN and
  signed-zero semantics. Preserve exact types, copied boxes and IEEE operators;
  floating boxed display remains unsupported. Add native and Raven map/GC checks,
  a .NET baseline and API documentation. Record generic math interfaces as a later
  exploration, not an implemented API or release commitment.

- Adapt the landing hero and inset code background to Light/Dark/Auto while
  preserving Raven syntax highlighting. Correct development async documentation
  and the executable sample to use `await input?`: await first, then propagate
  Result Error or Option None; explicit parentheses remain valid on older tools.
  Document manual website-only publication without a runtime build or release.

- Mark custom case carriers, including StreamError and TextReadError, with
  UnionAttribute in runtime source and reference metadata. Raven imports the
  typed constructor/IsCase/GetCase contract as IUnionSymbol and RavenDoc groups
  cases under their union. Standalone error structs remain ordinary structs.
  Use direct case construction in runtime implementations now that imported
  carriers have union semantics. Refresh bootstrap fragments and correct the
  carrier admission test's moved namespaces. This changes metadata classification,
  not storage or extraction lowering. Integrate Raven's existing configured-unit
  and structural-array identity fixes on its neoCLR branch for imported Flush
  and byte-stream interface signatures.

- Support Object.ToString for boxed Int32, Int64 and Boolean: culture-independent
  decimal integers and True/False text. Check integer limits and copied values
  after GC, and update API/website documentation. Other primitive formatting,
  format strings and culture providers remain unsupported.

- Migrate the website and API reference to one RavenDoc build. Adapt guides to
  Markdown and use an HTML landing page with a separate hero and feature cards.
  Add front-matter title/layout/outline controls, shared branding and a compact
  unreleased-documentation notice. Vendor the portable .NET 10 publisher with an
  immutable upstream revision, checksum and update script; remove DocFX and Node
  build dependencies. Generate Raven type/member pages from the checked reference
  assembly and authored documentation, including previously excluded callback and
  unit-result signatures. Preserve legacy reference routes, sample downloads and
  manual publication; retain explicit reference-coverage gaps. Simplify API lists
  to name-first signatures with parameter/property/field and return types plus a
  static icon marker, with full Raven list signatures available by opt-in.
  Distinguish classes (C), interfaces (I), enums (E), unions (U), delegates (D)
  and structs (S) on lists and detail pages. Add a shared
  RavenDoc API Browser with expandable namespace/type navigation, current-type
  highlighting and an off-canvas drawer on small screens. Compile authored
  nested `toc.yml` menus into section navigation, separate from main menus and
  page outlines. Add an N-logo favicon and a persistent Light/Dark/Auto theme
  menu. Share Raven website syntax highlighting for snippets and API
  signatures, with the engine bundled locally. General publishing changes are
  maintained upstream in Raven.

- Add boxed Int64 Object equality over the complete copied payload and a .NET-compatible
  lower/upper-half hash. Preserve exact-type comparisons and distinct box identity.
  Cover limits, deliberate hash collisions, Object-keyed maps and collection; boxed
  formatting and other primitive contracts remain separate work.

- Admit intrinsic Object in supported generic API signatures, matching ordinary
  import mapping. This enables HashMap<Object, Object> with explicit equality/hash
  callbacks and Option<Object> results. Check mixed Path/type/class/boxed keys,
  payload aliases, collisions, replacement, table growth and GC; all 955 objects
  are reclaimed across eleven collections. No default comparer, new primitive
  boxing behavior or runtime/library layout change is introduced.

- Give parameter snapshots owner-aware Object equality/hash and Name display,
  using closed declaring type, member kind/index and position even with absent
  parameter tokens or shared property accessors. Retain compact owner keys without
  member/parameter cycles; a public Member property remains future work. Document
  ParameterInfo and BindingFlags in the generated API reference. The internal
  parameter layout changes: rebuild the development runtime/library/SDK together;
  the archived value-descriptor profile retains its original layout. The parameter
  sample reclaims all 894 objects over 25 collections.

- Align field, method and property descriptor Object equality with kind, closed
  declaring type and definition index, with consistent hashes and Name display.
  Preserve Object overrides when projecting the shared descriptor base metadata.
  Add generated API coverage and document declaration-only queries; inherited
  traversal remains unimplemented. Add focused member
  identity and map/GC validation, reclaiming all 785 objects over ten collections;
  hashes and indexes are not persistent identities.

- Give AssemblyInfo and ModuleInfo wrappers scoped catalog equality, consistent
  Object hashing and assembly/module display names. Equal descriptors may remain
  distinct allocations; full assembly identity and module scope are required, not
  short names or tokens alone. Cover their public APIs in DocFX and explain the
  one-loaded-program limitation on the website. Extend descriptor map/GC validation
  and add colliding-name catalog regression coverage. The expanded Raven fixture
  reclaims all 476 managed objects across ten collections.

- Align RuntimeTypeInfo Object equality with represented type identity; hash its
  FullName consistently and display the represented type rather than the wrapper.
  Keep Equals(TypeInfo) non-nullable and Object.Equals(Object?) explicitly null-aware.
  Validate repeated, generic and array descriptors, typeof/GetType agreement and
  HashMap use under collection pressure. Add browsable TypeInfo/MemberInfo API
  reference and website guidance; hashes are neither unique nor persistent
  identifiers. Validation:
  the fixture reclaims all 422 managed objects across ten collections; three
  descriptor regressions and the Object reachability regression pass.

- Align Storage.Path typed and Object equality, hashing and display; implement
  Equatable<Path> and reuse the contract in HashMap lookup, replacement and rehashing.
  Keep typed equality operands non-nullable (Equatable<T>.Equals(T)); the existing
  Object.Equals(Object?) overload explicitly rejects null and unrelated objects.
  Preserve ordinal spelling and provider-independent semantics. Retain Object
  ancestry/constructor chaining for library class overrides and admit Path interface
  conversions. Preserve the archived Neo lexical Path surface in generated bootstrap
  fragments. Fix reachability analysis of nonvirtual class callvirt, exposed by
  Path becoming an Object subclass: retain its one static target and runtime null
  check. Update API reference and website. Audit other library classes and
  reproduce RuntimeTypeInfo's typed/Object inconsistency as the next bounded repair;
  provider/resource identity and general comparers remain separate decisions.

- Investigate value-type async state machines as the next author-directed priority.
  Add a reproducible Release heap/value probe: the initial heap baseline succeeds;
  struct emission exposes an import rejection and boxing at builder boundaries. Record
  by-reference startup, suspension ownership, GC and allocation measurement gates;
  keep the existing heap default. Implement the bounded ref builder projection,
  in-place startup and one retained state across suspensions, with completion cleanup.
  Add checked class field references and no-result byref value/interface methods.
  Validate ready/pending/cancelled states, unit/Result payloads and awaitless results
  under GC pressure; ready completion saves one managed object, pending counts match
  heap states. Keep broader performance claims open. Update API reference and website;
  builder APIs are transitional pending future runtime-owned suspension. Validation:
  37 runtime regressions, six Raven policy tests, twelve target matrix runs, metadata
  rejection checks, library/API snapshots and the combined website build. Raven
  target policy fix is 89a40051e; no change is integrated into Raven main.

- Add boxed Boolean Object equality and hashing alongside Int32: compare copied
  values only with the exact Boolean type, with hashes 1/0 for true/false. Preserve
  separate box identity and explicit Object base behavior. Extend the Raven sample,
  .NET comparison, runtime regressions and generated API reference; other primitive
  virtual equality/hash implementations remain outside this bounded slice. Check
  boxed reference-field tracing under allocation pressure and reclamation after
  scalar return. Record the author-directed library-wide Object/GC consistency audit
  and select Path contract alignment next. Validation: 44 runtime regressions,
  37 .NET assertions, the Raven Object sample, API reference and website build.

- Enable Raven's nullable-value declaration restriction for the development target:
  RAV0407 reports "Value types can't be declared as nullable" while nullable reference
  annotations remain valid. General Raven defaults stay compatible with .NET;
  Option remains the preferred absence model. Preserve inherited nullable metadata
  on generated record Object.Equals, with class/struct nullable Object comparisons
  and compiler rejection cases in the checked sample. Update integration, roadmap,
  on-site guidance and the development conversation record. Requires a matching
  development compiler; packaged SDKs are not refreshed by this change. The target
  record-struct generator narrows its comparison argument after an exact-type guard
  before unboxing. Validation: 117 integration compiler checks, record/Object samples,
  385 API entries and the combined website. Regeneration changes 102 input/compiler
  manifests only; runtime IL is unchanged. Follow up with typed record-class
  Equals(Record?) metadata and overload selection, preserving non-nullable record
  structs and explicit Equals declarations. Repair general Raven interface dispatch
  for top-level reference annotations and target nested-record component lookup.
  Extend the checked record sample and pinned .NET comparison; no library API changes.
  Typed-equality validation includes 61 target record checks, all three record programs,
  32 pinned .NET assertions, API/snapshot checks and the combined website build.
  Align generated record-class ==/!= nullable annotations with existing behavior,
  preserving value operands for structs and explicitly authored operators. Fix
  internal null guards to use identity, avoiding operator recursion and misleading
  custom operators in component equality/hash/display. Extend the sample and on-site
  guide. Operator validation: 66 target compiler checks, 34 pinned .NET assertions,
  all three record programs and rejection cases, API/snapshot and website checks.

- Implement the first non-generic struct/record-struct Object integration: explicit
  named-value overrides dispatch into boxed payloads with readonly protection;
  value isinst preserves exact boxes and unbox.any copies their payloads. Null
  unboxing uses NullReference and type mismatch uses the new InvalidCast Fault code.
  Extend Raven's configured record contract and importer for typed/Object/interface
  equality, hashing, display and deconstruction. Add a checked sample covering
  ordinary struct copies, box independence, default fields and string/reference
  components. Update the API reference, website, research and roadmap. General
  ValueType fallback equality, generic record components, nullable
  boxing and address-returning unbox remain unsupported. Validation: 55 compiler tests,
  47 focused runtime tests, 26 .NET baseline assertions, the compiled Raven sample,
  385 API items and the combined website. No VS Code build or release.

  Extend record components to same-compilation non-generic record structs, in both
  record classes and structs. Use typed value equality/hash without reference null
  guards and admit exact struct writes to declared instance output parameters for
  deconstruction. Add a separate Point/Rectangle/Drawing sample covering nested
  defaults, copy independence, equality, hashes and display. Keep the importer method
  limit unchanged and include both sample projects in the website download. Nullable
  struct components and arbitrary byref stores remain unsupported. Validation:
  58 focused compiler tests, both target samples, 28 pinned .NET assertions and
  combined website/API documentation checks.

  Repair default record structs with non-nullable string fields: generated methods
  guard null components, contribute zero to hashes and display empty component text,
  while preserving null in storage/deconstruction. Add a Defaults sample reproducing
  the former Utf8Encode fault, alongside null-versus-empty checks. No runtime layout,
  HashCode.Add(string) or nullable API contract changes. Validation: 59 compiler
  tests, all three target samples, 30 .NET assertions and website/API checks.

  Align Object.Equals(Object?) and ReferenceEquals(Object?, Object?) declarations
  with existing runtime null behavior. Annotate the source library and compiler/
  bootstrap references, admit literal-null core Object locals/call arguments through
  typed importer adapters, and regenerate matching library/API snapshots. Check
  null literals, nullable locals, overrides and boxed equality, while non-nullable
  Object assignments remain rejected. Runtime signatures/storage are unchanged.
  Record reference annotations as current Raven compatibility, with future metadata
  representation undecided; generated record-specific annotations remain separate.
  Validation: Object plus three record samples, non-nullable assignment rejection,
  regenerated bootstrap snapshot checks, 385 API items and the combined website.

  Clarify the author-directed absence model: prefer Option<T> for value and reference
  types; retain nullable reference annotations for Raven compatibility and existing
  null contracts. Defer nullable structs and nullable-value boxing rather than treating
  them as the next slice. Update roadmap, design record and on-site guidance; no
  runtime/compiler behavior or final metadata-format commitment changes.

- Implement bounded virtual Object equality/hash for boxed Int32: compare exact
  type and integer value, and return the stored integer hash. Preserve separate-box
  identity and explicit Object base behavior. Other boxed primitives and primitive boxed ToString
  remain unsupported; named struct overrides are covered separately. Extend the
  Raven Object sample, .NET comparison and API/website documentation. Validation:
  12 Object runtime tests, 22 pinned .NET assertions, the compiled Raven sample and
  combined website/API build. Include the sample in the local SDK workspace.

- Record the future minimal HTTP application namespace map, optional HTTPS
  dependencies, conceptual System.Web.WebApplication and longer-term time/String/
  StringBuilder needs. Extend the roadmap and HTTP plan with provisional layering,
  .NET comparisons and validation questions; no APIs or milestone priorities change.

- Extend experimental Raven record components to non-null strings and nested
  same-compilation record classes, including nullable record references.
  Preserve Equatable<Record>, use content/typed
  equality and matching hashes, and support string/reference deconstruction outputs.
  Typed importer call adapters preserve null argument positions; absent record
  components retain null through equality, hashes, display and deconstruction.
  Nullable strings/values and external record components remain unsupported;
  record structs are covered separately. The configured hash provider now requires Add(string) as well as
  Add(int). Expand the checked sample, design comparison, API guide and website. Validation:
  46 compiler tests, 32 reference-slot tests, Unicode/nested/nullable-record sample, editor
  completion and the combined site/API build. Prepare a separate matching local
  SDK snapshot, preserving the previous workspace.

- Prepare a matching local neoCLR/Raven development SDK workspace for VS Code,
  with records, Storage and Console projects and pinned compiler/server paths.
  Verify all three build/run paths and HashCode/Concurrency editor completions.
  Remove the Storage sample’s Console wildcard import to avoid the collision
  between Console.Error and the Result.Error case constructor. This is a local
  development snapshot, not a published SDK release.

- Add a bounded System.HashCode value accumulator (Add for integer/non-null string,
  ToHashCode and two-integer Combine) and opt-in Raven record-class integration.
  Integer records preserve reference identity while generating component equality,
  hashes, display and deconstruction. Other record shapes report RAVT004; generic
  hashing/comparers, boxed-value equality and runtime init-only field flags remain
  unsupported. Import recognized init-only setters and private readonly field writes
  with explicit context checks. Add a checked record sample and API/website coverage.
  Requires matching experimental Raven compiler, reference and runtime artifacts;
  default Raven/.NET record synthesis is unchanged. Validation: 40 compiler tests,
  three hash runtime tests, record/Equatable sample and shape rejection, readonly
  metadata checks, full library regeneration, 385 API items and the website build.

- Add a website namespace overview describing the current library and runtime
  support areas, with reference/guide links and explicit coverage gaps. Link it
  from the homepage and API navigation; keep Console under System and extend
  the inventory as implemented APIs introduce namespaces.

- Plan a pre-release Raven example portability pass after shared compiler fixes
  are independently verified on Raven main, released and integrated into the
  neoCLR branch. Track pinned baselines, minimal ports, failure classification and
  fixes/retests while retaining the efficient platform-specific CI split.

- Implement bounded Object.ReferenceEquals and virtual class Equals/GetHashCode,
  including default array identity, custom overrides, explicit base calls and a
  stable execution-local identity hash. Instance null receivers fault; static
  identity handles two nulls. Separate boxes retain distinct identity. String
  identity calls explicitly fault; boxed virtual equality/hash remain unsupported.
  Keep native identity imports exact and report ManagedHeap service requirements.
  Update importer static/virtual Object handling, generated library and API docs,
  and add a downloadable class sample. Record syntax is the author-selected next
  end-to-end gate: the initial acceptance probe exposed a missing comparer
  contract and HashCode dependency. The author then directed HashCode and adapted record synthesis,
  implemented in the follow-up entry above. Validation: nine equality/service tests, six identity cases, six
  display regressions, both Raven samples, 15 .NET assertions, full library
  regeneration, 378 API items and the combined website build.

### 2026-09-23

- Record System.Networking.Sockets and System.Web.Http as candidate future namespace
  boundaries, with a .NET comparison and open naming/package questions. This is a
  roadmap consideration, not an implemented API or a priority change.

- Characterize Object identity prerequisites before adding equality/hash APIs.
  Add artifact-roundtrip checks for aliases, mutation/GC, boxes, arrays, typed nulls
  and execution-local allocation IDs; expose the existing String wrapper identity
  gap in the review and website. Expand the pinned .NET baseline to 14 assertions,
  including custom equality/hash and String conversion behavior. No public equality
  method, hash algorithm or String representation change is implemented here.
  Validation: six new identity cases and four existing reference-identity tests,
  all 14 .NET baseline assertions and the combined website build passed.

- Make Object abstract with validated base construction, following author direction;
  direct Object allocation is rejected. This differs from .NET's concrete Object.
  Implement the first Object.ToString slice with a concrete-type-name fallback and
  ordinary class overrides. Preserve Object ancestry in imported application classes
  and distinguish virtual calls from explicit base calls. Rootless nominal classes
  and arrays use Object's default slot. Boxed-value and intrinsic-string virtual
  formatting remain unsupported; GetType support is unchanged. Add a checked Raven
  sample, raw dispatch regressions, refreshed API docs and an on-site source download.
  Application ToString declarations must use override; same-name hiding is rejected.
  No equality/hash implementation or erased Value migration is included.
  Validation: 21 dispatch/construction tests, the positive/negative Raven sample,
  three existing reflection/inheritance samples, matching library/API snapshots
  (375 API items) and the combined website build.

- Review Object and temporary Value storage against .NET reference/value semantics.
  Supersede older value-like Object inheritance and universal structural-equality
  proposals; record current implementation gaps and the next Object display/override
  slice. Add a pinned .NET 10 comparison program and generated reference coverage
  for Object.GetType and Value, with compiler-only Object members clearly separated.
  No Object runtime method or Value representation changes in this review.
  Validation: eight .NET baseline assertions, 32 focused runtime regressions,
  374 documented API items and the combined website build.

- Repair library composition: include StorageProvider only with the Raven
  StorageItem/File/Directory hierarchy, rather than in the legacy bootstrap library
  with static File helpers. Restore the legacy Console suite (11 tests); preserve
  the composed library's provider contract. Remove stale completed-release gates
  from the roadmap and record the upcoming Object/Value consistency review.
  Validation: 11 legacy Console tests, three composed-library Console stream tests,
  and matching generated-library/API snapshots.

- Keep Console a static class and add In/Out/Error, standard byte-stream factories,
  bounded UTF-8 ReadLine, Write and blank WriteLine. Add TextWriter and StreamWriter;
  extend TextReader with ReadLine (implementers must supply the new member).
  Calls remain synchronous; lines use LF/CRLF, with lone CR preserved as data.
  Add opt-in host byte-write/flush hooks and byte-exact Execution.stdout/stderr
  capture without a live host. Existing hosts retain WriteLine; new output hooks
  default to Unsupported until implemented. Refresh reference metadata, generated
  library and API docs. Adapt importer dispatch and Console's legacy Void returns.
  Show tested Result propagation and Option bindings on the Console feature page
  and dedicated Raven error-handling section; reserve match for useful case handling.
  Validation: three native boundary tests; greeting input cases; reader/writer
  ownership, partial-write and range contracts; both propagation/binding samples;
  dedicated/pooled worker output capture and budget regression;
  library/API snapshots and combined website build. The older raw-System Console
  suite remains blocked by an unresolved StorageItem dependency in that library.

- Characterize queue ownership in the isolated worker adapter with a pending await
  entered from an explicit caller queue: the body resumes through the producer's
  queue, while result observation waits for the caller queue to drain. Keep this as
  evidence of current behavior, not a future affinity guarantee. Record the author's
  direction to evolve TaskQueue and the scheduling model when concrete needs arise,
  including runtime suspension, without committing to retention or replacement.
  Update the roadmap, Task contracts, on-site explanation and source download;
  runtime and public APIs are unchanged. Record two unreduced callback-capture
  integration failures for independent investigation.
  Validation: exact cross-queue output, four worker cancellation/sibling outcomes
  with GC and zero final live objects, UserFault and bootstrap boundary checks,
  API snapshot, website tests/build and matching ten-file source archive.

- Connect acknowledged per-job cancellation to Promise.Cancel in an isolated Raven
  worker adapter. Admit the two existing services in bootstrap RuntimeServices only;
  normal Thread/Task APIs and installed worker implementation stay unchanged. Add
  dedicated/pooled awaiting consumers, sibling and GC checks, genuine UserFault and
  forbidden-service fixtures, and on-site adapter documentation/source download.
  Record the generated-field collision with an async parameter named state as a
  deferred integration issue; the tested sample uses destination. Refresh the matching
  normal-core API snapshot without adding managed public APIs.
  Validation: four cancellation/sibling outcomes with GC and zero final live objects,
  UserFault and normal-core boundary checks, original delayed-copy/busy-queue consumers,
  runtime/API snapshots, matching source archive and combined website checks.

- Give isolated jobs independent cooperative cancellation tokens. Add experimental
  raw RequestWorkerCancellation and JoinWorkerResult runtime services: requests retain
  pending roots, and acknowledged job cancellation returns an erased Void outcome for
  adapters. Cached success and unrelated Faults are preserved; host cancellation and
  legacy JoinWorker behavior remain terminal. Release pooled per-job state before
  result publication and retain all-job cancellation/join on invocation teardown.
  Ordinary Raven Thread/Task/Storage APIs are unchanged; document the raw services
  on-site and keep guest Task-adapter integration as the next slice.
  Validation: 10 worker-registry tests, 16 worker integration tests including both
  execution backends and cancellation/GC, service reachability, normal build and
  website tests/build.

- Connect the pending-read experiment to real isolated host workers and the existing
  notification adapter. Defer requested cancellation until joined producer completion,
  then discard the owned payload; preserve completed results against late requests.
  No public Storage/runtime API or worker interruption is added. Reuse the delayed-copy
  harness for the new consumer; record the private-field async bridge limitation and
  add on-site explanation and consumer-source download.
  Validation: five host-backed outcomes with actual GC and zero final live objects,
  original delayed-copy/busy-queue consumers, website tests/build and source archive.

- Add a pending-read contract experiment using real Task/Promise, Raven await,
  TaskQueue and managed GC. Six ordered scenarios distinguish cancellation requests
  from terminal acknowledgement, preserve bytes on failure/cancellation and reject
  duplicate terminal writes, releases and notifications. The buffer stays private
  while pending; producer callbacks and resource release are modeled, with no new
  public async Storage API. Add on-site documentation and a source download.
  Validation: all six guest scenarios, actual collections and zero final live objects;
  website tests/build and source archive verification.

- Add a bounded File Transformer sample connecting platform Storage and System.IO
  to the existing experimental JSON document consumer. Validate and serialize before
  exclusive output creation; preserve input and existing destinations, close streams
  and document partial-output risk after write/flush failure. Add an on-site walkthrough
  and downloadable source bundle. JSON remains sample code and I/O is synchronous.
  Validation: nine isolated file cases, unchanged source/existing-output checks,
  independent JSON output parsing, matching source archive, website tests and build.

- Complete the synchronous Storage POC with a platform-only file-access sample,
  downloadable on-site walkthrough and API reference. Move development byte streams
  from System.Streams to System.IO (recompile consumers and update imports). Add
  TextReader/StreamReader with bounded strict UTF-8, named errors and explicit input
  ownership; add optional SeekableStream with absolute byte positions on file input.
  Fix strict bridge admission of reader locals and constructor Boolean conversions.
  Keep Task-returning Storage as future exploration pending suspension, scheduling
  and cancellation contracts; no asynchronous I/O is claimed.
  Validation: standalone POC, disk/memory reader and provider contracts, nine native
  file-resource tests, strict interface probe, runtime/API snapshots, 332 documented
  API items and combined website build.

- Integrate FileSystem as the host StorageProvider with internal File/Directory
  implementations. Add relative Directory.GetItem/GetDirectory and bounded
  GetItems snapshots of mixed StorageItem interfaces. Bounds fail explicitly
  rather than truncating; StorageLookupError gains InvalidRange and LimitExceeded.
  Enumeration is synchronous and non-atomic with child lookup. Update the disk/memory
  sample to exercise platform host resolution and refresh API/runtime documentation.
  Validation: eight native file-resource tests, full SDK product/contracts and
  negative callers, runtime/API snapshots, 262 documented API items and the
  combined website pass.

- Consolidate StorageProvider around GetItem/GetFile/GetDirectory with interface
  results. Remove the temporary StorageLookup contract and provider-level byte
  methods; byte routing belongs to concrete File implementations. Development
  consumers must migrate lookup to StorageProvider and keep byte helper contracts
  inside implementations. Add generic disk/memory lookup and resolution-only provider
  coverage; refresh runtime and API reference artifacts. Record the complete POC
  scope, including System.IO alignment, text readers and a seekability case.
  Validation: full SDK sample/contract suite, strict interface import, bootstrap
  snapshot, 243 documented API items and combined website pass.

- Add synchronous StorageLookup.GetDirectory(Path), returning the public Directory
  interface with typed missing/wrong-kind errors. The disk/memory product obtains
  its root through provider lookup. Disk validates native kind; the bounded memory
  provider exposes only its root and does not infer directories from file keys.
  Development migration: StorageLookup implementers must add GetDirectory; byte-only
  providers are unchanged. Document the contract and refresh API/runtime snapshots.
  Provider consolidation, generic item lookup and enumeration remain follow-ups.
  Validation: SDK disk/memory product and contract/negative suite, strict interface
  import, runtime/API snapshots and combined website pass. Reaffirm the bounded POC
  checkpoint and record TextReader/StreamReader as planned follow-up work and
  seekability as an open design question, not implemented APIs.

- Implement StorageItem as a closed root over provider-implemented File and Directory
  interfaces. Provider result contracts expose interfaces; concrete classes remain
  implementation details. Move descriptor classes into the sample providers; test common
  Name/Path access over mixed items and reject unrelated branches in Raven and the
  strict importer. Raw neoIL does not enforce the closed-hierarchy metadata.
  Development migration: construct provider implementations rather than File/Directory;
  native static text helpers move to FileText because interface static methods are
  not supported. Legacy raw File aliases remain. Update API docs, generated runtime
  and reference snapshots, website and roadmap. Provider resolution and enumeration
  remain next; StorageLookup/FileAt/CreateNew are still transitional POC contracts.
  Validation: disk/memory product and contract/negative callers, mixed StorageItem
  arrays, Raven and raw-CIL closure rejection, strict interface/foundation and static
  file import checks, ten native file/Path regressions, runtime/API snapshots,
  244 documented API items and the combined website pass.

- Record the author's selected Storage model: StorageItem is a closed interface
  hierarchy over provider-implemented File and Directory interfaces; providers
  resolve paths and GetItems enumerates StorageItem values. Correct proposal class
  examples, update roadmap priority and distinguish the target from current concrete
  descriptors on the website/API guide. Interface migration and enumeration remain
  planned; no new runtime API is implemented by this direction update.
  Record queryable metadata as a future extension independent of native filesystem
  attributes, with System.IO/WinRT comparisons and query shape, availability and
  freshness left open. The interface/provider migration remains the next priority.

- Integrate development System.Storage.File and Directory descriptors and the optional
  StorageLookup provider capability. File retains provider/Path, derives Name and
  opens directional streams; Directory constructs child addresses and explicitly
  looks up files. Construction/properties perform no storage queries. Preserve the
  existing static string-based File.ReadAllText/WriteAllText helpers. The sample now
  imports platform descriptors; text conveniences stay on fixture providers.
  Directory.FileAt changes from the sample FileReadError to StorageLookupError.
  Add complete descriptor/lookup and legacy text-error API reference coverage, with
  an exact manual WriteAllText entry for DocFX's unit-result limitation. Regenerate
  runtime/reference snapshots and update the website and roadmap.
  Validation: disk/memory product and contract checks (including construction without
  provider calls), strict interface/foundation checks, ten native file/Path artifact
  regressions, runtime/API snapshots, 245 documented API items and the combined
  website pass. Concrete host-provider integration is next.

- Integrate development System.Streams.InputStream and OutputStream interfaces.
  File streams implement them directly; the disk/memory sample imports the platform
  contracts and drops forwarding disk adapters. Preserve blocking partial transfers,
  typed errors and explicit close. Add generated API coverage, a manual OutputStream.Flush
  entry for DocFX's unit-result limitation, and opposite-direction negative checks.
  Custom implementations require the matching core/runtime and a development Raven
  compiler with the imported-array and configured-unit identity corrections;
  Preview 9 is unchanged.
  Record the author's minimal WinRT-informed Storage direction: useful address/name
  properties and explicit operations first, richer metadata only when needed.
  Simplify the sample File constructor to (provider, path), deriving Name from Path;
  remove the redundant StorageProvider.FileAt factory. Directory.FileAt remains an
  address operation. Reject independent names; preserve existing lookup and I/O.
  Integrate the minimal System.Storage.StorageProvider byte contract with OpenRead
  and CreateNew. Sample lookup/text conveniences extend it; file byte operations
  dispatch through the platform interface for both disk and memory. Text encoding,
  descriptor lookup and richer metadata are not mandatory provider methods.
  File/Directory integration follows in the entry above. Validation:
  disk/memory SDK contracts and negative callers, interface import checks, runtime
  snapshot, 151 API summaries and the combined website pass.

- Add development System.Storage.Metadata.GetKind(string), EntryKind and
  StorageLookupError over the existing native metadata service, with generated API
  reference coverage. Add typed GetFile lookup to the disk/memory provider sample;
  distinguish missing and wrong-kind outcomes without opening file contents.
  Extend experimental Directory.GetFile with relative Path lookup, including nested
  paths. Absolute paths remain valid for providers but are rejected by a directory;
  its string overload still accepts one direct child. Document this provisional
  distinction, flat-memory-directory limits and tested .NET comparison.
  Lookup is an observation, not stable identity or a guarantee of later access.
  Record Path as Storage-specific rather than mandatory throughout the platform;
  host metadata and file-stream APIs retain string parameters. Matching regenerated
  development runtime/reference artifacts are required for these additions.

- Replace the experimental MemorySlotStorage with MemoryStorage: text helpers and
  streams now share byte contents per address. Support up to eight addresses with 64 KiB
  current payloads, strict UTF-8 reads and cross-API exclusive creation. Keep old
  payloads alive for existing streams after text replacement; this provisional
  memory policy is not a disk/portable identity guarantee. Extend the disk/memory
  sample and contract checks, and update the on-site API reference and roadmap.

- Add isolated host Storage lookup probes comparing .NET FileInfo caching with
  metadata observations, path replacement, open-handle identity and provider roots.
  Record typed metadata lookup as a provisional direction, not an implemented
  GetFile API. Make coherent memory-provider text/byte contents the next prerequisite.

- Integrate the tested immutable Path value into the development System.Storage
  library, with Parse returning Result<Path, InvalidPathError>, private construction,
  read-only text and lexical equality. Preserve existing Combine/GetFileName string
  helpers and add generated reference for all Path members. The Storage sample now
  imports the platform type; its Path.rvn retains only a fixture helper. Regenerate
  matching runtime/reference artifacts. Logical grammar and provider APIs remain
  provisional; Path is not required by unrelated string-taking APIs. Prioritize
  integrating the remaining working Storage slice over expanding isolated experiments.
  Record future Unix/Windows parsing and normalization, and an analogous Uri value
  direction and Path-taking helper overloads, as plans rather than implemented
  behavior; keep immediate integration scoped to a minimal Storage POC.

- Add host-visible FaultCode classifications, including StackOverflow for the
  interpreter frame limit, arithmetic/memory limits and host cancellation. Explicit
  guest faults always use UserFault; System.Fault still accepts only a message.
  Keep a RuntimeError fallback for diagnostics not yet classified. Preserve codes
  with stack traces and expose debugger fault_code; CLI messages include the code.
  Document all codes and the Rust host API on-site. Hosts constructing Fault with
  struct literals must provide code; formatted CLI diagnostics also change.

- Add development System.Streams FileInputStream/FileOutputStream APIs with typed
  StreamError results, bounded caller-buffer transfers, exclusive file creation,
  flush and explicit close. These calls block; they do not implement async I/O or
  durable flush. Connect the Raven provider sample to byte streams on disk and in
  memory, including deliberate short transfers and actual UTF-8 disk verification.
  Keep native IDs unique across invocations so retained wrappers cannot access a
  later invocation's files. Add browsable API reference and source downloads.
  Keep provider-owned child-address construction and the earlier bounded whole-text
  workflow as exploratory comparison cases. Following the author's clarification,
  align Storage and explore a Path value object next; application-owned provider contracts remain provisional.

- Require on-site reference coverage for public API changes, including namespace
  navigation and documented renderer exclusions. Add System.Concurrency Thread and
  ThreadPool reference pages, a namespace browsing index, and an explicit backlog
  for older APIs that still lack member reference coverage.

- Select the post-release concurrency, Storage and file-stream checkpoint. Keep
  Task/Promise in System.Tasks; move explicit thread APIs to System.Concurrency.
  Add retained Thread instances with one-shot Start, IsStarted and a stable Task,
  plus static Thread.Run; successful completion waits for native thread teardown.
  Update bindings, samples, generated library, API docs and the development website
  section. Preserve the website's tested Preview 9 worker example. This breaks the
  old namespace and static Thread.Start spelling; rebuild with matching artifacts.
  The first file Stream APIs and disk read/write acceptance app are now implemented
  as described above; Storage alignment remains pending.
  Task.Run will need completion-only and generic
  value-returning overloads; keep it pending suspension/scheduling design rather
  than publishing a string-only worker facade.

- Skip the runtime matrix for documentation, website and API-reference-only pushes
  and pull requests; keep executable experiments covered and provide manual runs.
  Release tags no longer automatically repeat the validated candidate matrix.
  Document the trigger rules and retain the broader CI redesign for the next release.

- Record publication of Preview 9 at `834028c` with six passing platform/toolchain
  jobs and extracted macOS arm64 package/editor evidence; mark the async checkpoint
  complete and resume foundational Streams, Storage and Encoding priorities.
- Align website, setup and API reference with Preview 9. Reduce the main menu to
  five consistent destinations, remove Experimental from the API header/title and
  add logo spacing. Make DocFX HTML and client-side navigation links work under a
  GitHub Pages project path, with regression coverage. Publish the combined site and
  `/docs/` from `5bffbca` through the manual Pages workflow; verify the public pages.
- Plan next-release CI efficiency: run shared validation once and isolate host-specific
  checks instead of repeating all samples per platform. Workflows are unchanged.


## 0.1.0-preview.9

Prepared candidate; publication date and final evidence belong to the release manifest.
The dated entries below record development, including explicitly labelled future plans.

### 2026-09-23

- Prepare version 0.1.0-preview.9 for the async/Tasks release, with matching Cargo
  metadata and release/migration guidance. Source and package validation of this
  versioned candidate must pass before publication; earlier candidate results remain
  separately attributed. Retain a fresh Unreleased section for subsequent work.

- Continue async release preparation with a fresh e9bb28a evaluator bundle, eight
  passing Workbench cases, all six packaged Task probes, dependency/hash checks, a runtime-only execution check
  and interactive VSIX build/run/type-hover evidence in a separate profile. Start exact-commit local source checks and six-job platform/toolchain
  CI, without claiming results before completion. Draft release notes and correct
  README's stale threading/async limitation while retaining published Preview 8 links.

- Make the low-level worker-cancellation sample self-contained by declaring its
  worker service imports in the neoIL source. The host example now uses that same
  source directly. This fixes the full-suite standalone sample assembly failure;
  verifier assembly diagnostics now identify the failing sample path. All nine
  verifier checks pass on stable Rust and Rust 1.85, and the host cancellation
  example passes. Preserve both earlier full-source failures separately; neither
  constitutes a passed release gate.

- Record the author-selected post-release System.Concurrency namespace direction
  and proposed Thread.Run versus retained Thread/Start/Task API shapes. Document
  optional thread packaging, platform limits, .NET comparisons and validation needs;
  update website future direction. Clarify abstraction-first concurrency, separating
  Task completion, possible portable worker execution and target-specific Thread
  capabilities. Record the subsequent Task.Run-style proposal for general work with
  platform-selected concurrent execution. Capture the author's confirmation of Task's
  general submission role and Thread's explicit, optional platform capability under
  System.Concurrency; Web Workers are a possible Task backend for WebAssembly.
  Exact contracts remain open. Current release APIs remain unchanged.

- Correct two stale release samples after the Error-wrapper removal and Storage
  namespace migration; preserve the case-payload and error-carrier regression intent.
  Validate all 84 saved-project outcomes against the rebuilt runtime and clean SDK payload.
  Document sidecar-free archive creation for the known macOS packaging issue,
  including staged-payload hash comparison and fresh extraction requirements.
  Preserve follow-up archive hashes, eight Workbench results, 84 saved-project
  outcomes and focused extracted-fixture checks in a separate readiness record.
  Extend migration review through the subsequent documentation changes, explicitly
  keeping proposed concurrency APIs outside this release's migration instructions.

- Preserve Rust 1.85 compatibility in worker notification dispatch by replacing
  unsupported let-chain syntax with equivalent nested conditions. The source-archive
  minimum-version check exposed the failure; the supported minimum remains unchanged.
  Validate all targets on Rust 1.85, strict stable Clippy and 14 worker regressions.
- Draft migration guidance from Preview 8 and intermediate Task builds. Reaffirm
  that this changelog continues permanently after the async release, with frozen
  published entries and a fresh Unreleased section for later development.

- Add a DocFX development API reference at `/docs/`, with an API overview, authored
  XML descriptions for the initial Task surface, and metadata-generated signatures.
  Document three unsupported callback signatures separately in Raven notation.
  Integrate the reference into website navigation, validation and the Pages artifact;
  publication remains manual. Keep this release’s documentation scope to a useful
  overview and main APIs, with exhaustive coverage deferred.

- Prepare async preview packaging: include the current Task probes in the evaluator
  bundle, make their helper/sample paths work after extraction, replace the obsolete
  explicit-worker-queue expectation and add an eight-sample saved-project Async
  Workbench check with a JSON report. Document the packaged commands; release
  certification remains separate from these focused checks. Fix three pre-existing
  rustfmt discrepancies and strict Clippy findings (redundant match guards and
  test idioms) found by the release-readiness pass; no runtime behavior changes.
  Persist toolchain roots in the Workbench project so Raven’s separate project load
  resolves the extracted bundle; record passing local package probes and remaining gates.
  Verify the repackaged eight-sample Workbench, all 22 MSBuild cases, Task editor
  completion, full library regeneration and bundle/SDK notices; preserve revisions,
  hashes and case results in a local-readiness record. Final release gates remain open.

- Make the author-approved async/Tasks preview checkpoint the immediate
  priority: stabilize the existing completion/await/composition and isolated-worker surface, require a
  fresh evaluator bundle and exact-candidate release gates, and leave HTTP and the
  notification adapter outside the supported release scope. Record the author’s
  release-timing question, update website direction and correct stale Task contract
  descriptions. Prioritize Streams, Storage and Encoding before networking afterward.
  Mark the website header Experimental across all pages and describe neoCLR as an
  experimental application platform, including APIs, language integration and
  development tools. No release version/date or publication is selected.

- Revise the website toward technical project documentation: current capabilities,
  limits, .NET tradeoffs and the active foundations-to-HTTP roadmap. Add a project
  overview with background, goals and contribution paths; keep general information
  on-site and repository design records optional. Preserve tested sample sources
  and distinguish published behavior, development experiments and proposals. Validate
  all 13 pages, site tests and responsive overview rendering.

- Add bounded JSON string-message and sensor-report document experiments using
  existing text primitives. Validate Unicode escapes, read/write all six value
  kinds, preserve number spellings, check Int32 conversion and reject duplicate
  decoded keys; bound bytes, nesting and value counts. Compare behavior with .NET
  10 and check construction, round trips and writer failures. Record importer
  limitations and advance the roadmap to delayed guest lifetimes. These are
  application-local experiments, not a public JSON library.

- Add an application-local UTF-8 chunk decoder experiment reusing Byte Copy's
  managed arrays and short reads. Demonstrate scalar boundaries versus graphemes,
  strict finalization, caller-buffer reuse and synchronous GC retention. Compare
  41 valid/malformed/split-input cases with strict .NET 10 decoding; keep public
  Encoding contracts, efficient output buffering and pending native I/O open.
  Record partial S2 evidence and link the experiment from the Strings feature page.

- Add a Raven Byte Copy experiment over existing managed arrays: validate ranges
  before mutation, preserve overlapping copies and demonstrate partial reads/writes
  with a greeting transfer. Validate 1,024 range/alias cases, extreme/empty ranges,
  output failure and synchronous array retention through guest GC. Keep public
  stream/buffer contracts and pending native I/O unselected; record the deferred
  compiler fallthrough-emission observation and partial S1 roadmap evidence.
- Add a dedicated Arrays feature page comparing neoCLR with .NET arrays, including
  generic shape, invariance, aliasing, bounds and collection capabilities. Include
  a tested downloadable readings/snapshot sample and exact output; link from the
  homepage and Collections. Validate 12 website pages, website tests, the new sample
  and 32 existing runtime array tests. No website publication is implied.

- Start M1/S0 with an isolated host-side external-completion experiment. Seven cases
  cover callback progress, cancellation acknowledgement, completion ordering and
  teardown. Add eight test-only delayed Byte Copy checks against the real managed
  heap, array slots and delegate receiver tracing: pending-to-ready root handoff,
  collection/reclamation, invalid-input rejection and terminal cleanup. Record
  owned-byte delivery as a candidate. Add experimental worker completion notification
  to the real VM, rooting callbacks at both GC paths and posting ready outcomes to
  the default TaskQueue. Validate a Raven await/copy consumer using an isolated
  worker-library adapter, failure/cancellation teardown and one-shot delivery. Poll
  ready notifications at default-queue callback returns so self-reposting guest work
  no longer requires queue quiescence; verify Raven and direct-IL consumers and
  explicit-queue isolation. Bound successful worker result text and captured output
  with a shared per-worker logical-byte quota (default 1 MiB); reject overflow before
  capturing a line or publishing success, preserving unavailable console input.
  Cover UTF-8, empty lines and notification failure delivery. Larger worker outputs
  now fault unless an embedder raises `Limits.worker_result_bytes`; exhaustive Rust
  Limits literals need the new field. This is not a total host-memory bound.
  Poll host cancellation after worker waits and between output writes; a request
  observed during delivery now skips remaining lines and stops before returning the
  joined value. Keep already written output. Add a runnable host/guest-IL sample,
  controlled notification orderings and teardown acknowledgement checks; guest
  operation cancellation remains unimplemented.
  Normal Thread/ThreadPool implementations retain queued
  joins; preemption, per-operation cancellation, bounded native I/O ownership and
  real I/O remain open.

- Prioritize an HTTP client/server application POC in the roadmap, with smaller
  stream, encoding, JSON and TCP cases, explicit acceptance criteria and complete
  proposal triage. Record external I/O progress, cancellation, buffer ownership and
  cleanup as urgent experiments; distinguish planned APIs from current behavior.
  Align planning entry points and the website proposal overview; no runtime APIs
  or release commitments are added by this documentation change. Add a unified
  platform roadmap with themed sample products: HTTP apps, a file catalog, a
  download queue, a time-aware report, an assembly explorer and a portable sample
  pack. Make that roadmap the default authority for work in AGENTS.md, subordinate
  to explicit author directions. Keep post-POC milestones and separate research
  products provisional. Refine M1 delivery to memory copy, text/JSON transformation,
  delayed guest-lifetime checks and a bounded file transformer before TCP/HTTP;
  retain HTTP as the first major application destination and allow earlier APIs
  to evolve as subsequent samples expose gaps.

- Add a development Tasks feature page with tested, downloadable worker, Promise,
  cancellation, MapResult and awaited-propagation examples. Link it from the
  homepage and proposals, distinguishing current behavior from future scheduling
  and cancellation-token work. Add a dedicated homepage Tasks box with a tested
  async worker excerpt and a direct feature-page link. Validate 12 pages and website
  tests; retain the five locally installed .rvnproj build/run checks and Task editor
  completion evidence. Prepare a
  matching local VS Code development workspace without changing release selection.

- Integrate Raven's propagation-temporary lifetime fix for `(await input)?`, with
  immediate and resumed Ok, Error and cancellation regressions. Preserve current
  postfix-first precedence; record ergonomic shorthand as future design work.
  Validate eight target propagation cases, 40 focused Raven target-branch tests
  and 21 independent Raven main tests; integrate only the general lowering fix.

- Add the explicit Task<Result<T,E>>.MapResult extension: queue Ok transformations,
  preserve Error payloads and propagate cancellation without invoking the mapper.
  Ordinary Task.Map still handles the complete value. Include a runnable sample
  and six passing outcome, dispatcher, GC and fault checks; validate 272 metadata
  signatures and the generated runtime snapshot.

- Add invocation-local TaskQueue.Default and Promise<T>() using the active queue
  or the default. Async functions and isolated workers no longer require queue setup.
  The runtime dispatches default-queue callbacks automatically before invocation
  return, preserving the entry result and retaining work through GC. Explicit queues
  remain caller-driven; external I/O completion and a public scheduler are future work.
  Validate 11 automatic-dispatch source scenarios, async regressions, ten runtime
  access/worker checks, metadata, snapshots and website checks.

- Add opt-in Raven cancelled-await propagation for named async functions, both
  immediate and resumed, through provisional IsCancelled/SetCancelled hooks.
  Nested calls cancel without fabricating a value; Result remains an ordinary
  payload. Await inside for loops is diagnosed pending suspension-aware iterator
  cleanup. Rebuild the compiler, reference library and callers together. Validate
  18 neoCLR scenarios, 31 focused Raven tests, metadata signatures, runtime
  regressions, bootstrap snapshots and website checks.
- Correct the development status after 606a597: Map and Then composition is
  implemented and tested; cancellation tokens remain outstanding.

### 2026-09-21

- Add the core development Task model: normal TaskState enum, TaskOutcome<T> union,
  and Task.State/Outcome. Rename TaskCompletionSource<T> to Promise<T> and
  TrySetResult to Complete; add Cancel with first-terminal-transition semantics and
  queued observers. Result.Error remains an ordinary completed payload. Rebuild
  references, library and callers together. Tokens, Map/Then and automatic await
  cancellation propagation remain outstanding; inspect cancellation through Outcome.
  Validate 24 source contract scenarios, 10 async and four worker regressions, nine
  direct runtime checks, 263 metadata checks and the website.

- Record Promise-style Task composition and continuation as API direction, with
  names chosen for neoCLR rather than .NET parity. Keep Result failure independent;
  the updated Task model proposes State/Outcome, distinct cancellation and Map/Then.
  Record runtime-first implementation slices and Raven lowering gaps; distinguish
  cancellation requests from terminal outcomes. Update the proposals overview
  without claiming implementation of cancellation or composition.

- Select provisional heap async states and exception-free lowering in the neoCLR
  project props, matching the development compiler and language server. Keep the
  installed development build separate from published Preview 8 artifacts. Verify
  the async, worker and storage projects through MSBuild and the language server.

- Rename the development System.IO namespace and source folders to System.Storage,
  including File, Path and file errors; move ConsoleReadError to System. Update
  references, samples and current documentation. Rebuild callers and library
  artifacts together; local synchronous
  file behavior is unchanged and storage providers remain future work. Validate
  27 runtime checks, 257 signature checks, website generation and LSP completions.

- Add an isolated-worker PoC in System.Threading: Thread.Start and ThreadPool.Queue
  exchange text through static callbacks and return Task<string> on the caller queue.
  Use separate interpreter heaps, a two-thread pool, bounded submissions and joined
  invocation cleanup. Queue pumping may block for results; guest objects and Task
  state are not shared across threads.

- Execute provisional compiler-generated async/await in the Raven profile using
  System.Tasks.Task<T>, TaskCompletionSource<T> and explicit TaskQueue.Run/Drain.
  Rebuild references and replace System.Threading.Tasks imports. Result remains an
  ordinary payload; exception capture and threading are not introduced. Validate
  ten async scenarios, fifteen completion scenarios and 47 focused runtime checks.
  This development behavior is not part of the published Preview 8 artifacts.

- Permit constructors to initialize fields containing erased values without inventing
  a default payload. Unassigned fields remain unreadable and constructor publication
  still requires initialization; explicit invalid defaults remain rejected.

### 2026-09-19

- Record the decision to defer Thread/ThreadPool work and finish compiler-generated
  async integration first. This is implementation direction, not shipped async
  support or a thread-safety guarantee. Independently reproduce and fix Raven's
  struct-field receiver addressing on main; add a provisional heap state-machine
  option only on Raven's neoCLR branch. Modern .NET compiler tests pass, while
  neoCLR builder/importer integration remains in progress.

- Add a provisional Raven-profile Task<T>/TaskCompletionSource<T> completion PoC
  with an explicit TaskQueue, ordinary generic payloads (including unit/Result),
  first-completion semantics and queued continuations. Preserve private storage
  and internal producer/consumer implementation access in guest metadata. Validate
  source scenarios, direct-IL access boundaries and GC retention; document the API,
  scheduling limits and outstanding compiler-generated async work. This development
  API is not in Preview 8. Update the proposals page and runtime API audit.

- Retire the legacy System.Error message wrapper, its native helpers, intrinsic
  runtime/host value and type, ErrorValues service and `error` instruction. Use
  ordinary strings or domain-specific Result payloads; typed error unions remain.
  This breaks old message-wrapper metadata/host APIs: rebuild references, library
  and callers together. Published Preview 8 artifacts are unchanged. Simplify the
  outcome samples to imported `Error(...)`, document the convention and migration,
  refresh the library inventory and update the outcomes feature page.

- Start the provisional async mechanism with an executable Raven completion model:
  validate pending heap-owned state across GC, multiple consumers, queued callback
  ordering, duplicate completion, Result/unit payloads and terminal fault boundaries
  in 10 scenarios. Record current Raven builder/exception-lowering gaps and the next
  compiler-integration steps. Distinguish public Task semantics from replaceable
  state-machine, builder and awaiter contracts. Update the proposals overview; this
  experiment does not introduce a public Task<T> or generated async/await support.
  Specify exception-free target lowering: Result errors complete tasks as values,
  Faults remain terminal, and Raven must omit its generated catch wrapper rather
  than merely omit SetException. Record cleanup and .NET regression requirements,
  with the current CreateMoveNextBody entry point identified for integration.
  Clarify that Task lowering is uniform in its payload T: Result is an ordinary
  type, with no special failure/completion path or Result-aware scheduling.
  Add Raven's provisional opt-out for generated async exception capture on its
  neoclr branch, with compiler execution tests for propagation before/after pending
  awaits and unrelated union payloads. Document the API-only configuration,
  unchanged .NET default and remaining target integration and generic-unit gaps.
  Validate 61 focused and 119 feature-selected compiler tests (overlapping sets),
  and use ordinary ? in the manual neoCLR continuation probe's application body.
  All 10 neoCLR probe scenarios pass, including retained state through 46 GC cycles.
  Refresh and validate the website proposals overview. Independently fix generic
  unit-task return binding and completion in Raven main, then cherry-pick that fix
  to neoclr; 145 async/resource tests pass on .NET. Guest builder/Task integration
  and the configured System.Void ABI remain separate work.

- Port 25 Option/Result operator overloads from Raven.Core to the development
  library, including transformations, recovery, branch actions, conversions and
  nested Option flattening. Use Filter and ToIterable for neoCLR's vocabulary;
  throwing/default/context-error helpers remain deferred. Implement both iterable
  and outcome operators with Raven extension declarations and validate their
  extension metadata against bootstrap references. Add branch/callback, signature,
  editor and saved-project coverage; document compatibility and allocation costs.
  Rebuild callers, references and System together; published Preview 8 is unchanged.
  Validate 9 outcome scenarios, 251 signatures, 86 editor sections, all 80 library
  slices, 58 iterable regression checks and the MSBuild example. Prefer inferred
  callback types in samples and record the remaining compiler limitations. Include
  the outcome verifier and the query
  verifier's helper in future bundles.
  Simplify redundant lambda signatures in query-terminal, deferred-query and
  delegate samples; retain the generic Map callback signature required by current
  inference. All three edited examples retain their checked execution output.
  Add the website's complete Option/Result operator table, a tested composition
  example and downloadable source/output. Link the .NET iterable mapping and
  distinguish development additions from Preview 8. Validate all 10 pages,
  cross-page links and highlighting, and review the operator table in the browser.
  Record natural, minimally annotated Raven as a project-wide convention for all
  hand-authored code, including runtime implementations, tooling, tests and examples.
  Make the scope explicit in AGENTS.md; retain annotations required for compilation
  or a clear contract and the documented bootstrap/test exceptions.

- Document imported union case patterns in the Raven conventions: use Some, None,
  Ok and Error without a leading dot when their case namespaces are imported.
  Update the convention examples, including nested patterns.

- Add the basic development Iterable operators before Task work: Any, All, Count,
  seeded Fold, Take, Skip, Concat and FlatMap. Preserve short-circuiting, empty-input
  outcomes, ordered lazy composition and normal iterator disposal. Count faults on
  Int32 overflow. Rebuild references and the System library together; Preview 8
  packages remain unchanged. Add executable samples, signature and editor coverage,
  and .NET/Rust contract comparisons. Support nested open collection arguments in
  the neoCLR library import bridge without changing Raven's compiler. Validate
  58 query outcomes, 174 signatures, 86 editor sections, 5 compiler-path checks,
  24 focused runtime tests and exact reproduction of all 77 library slices.
  Add a website guide with the tested basic sample and a .NET-to-neoCLR operator
  table, including outcome differences and unsupported operators. Keep development
  additions distinct from published Preview 8. Validate all 10 pages and review
  the mapping table in the browser.

- Record the general Raven SDK AppleDouble packaging candidate and the validated
  sidecar-free Preview 8 archive workaround; no upstream script fix is claimed.
- Rename the development query API after Preview 8: Where becomes Filter and
  Select becomes Map, with initial capitals and no legacy aliases. Rebuild callers
  and target references together; deferred execution and iterator behavior are retained.
  Update samples, metadata bindings, editor checks and the website migration guide.
  Validate 83 saved-project outcomes, 33 query outcomes, 142 signatures, 86 editor
  sections, 5 compiler-path checks, 15 application checks, 24 focused runtime tests
  and reproducible regeneration of all 77 library slices.
  Prefer converged terminology while retaining .NET terms where conventional or
  clearer; do not mechanically choose Fold/Reduce or Drop/Skip. Preview 8 still
  exposes the existing names.
- Record Task and async state-machine contracts as a post-Preview 8 foundation:
  upcoming APIs need the completion contract before runtime suspension arrives.
  Keep continuation scheduling and logical context flow as explicit open design work,
  including the goal of avoiding routine ConfigureAwait-style boilerplate.
- Select Preview 8 and Raven 0.1.12-neoclr.15 as the release candidate; prepare
  notes, migration guidance and the validation checklist. Repair the Windows
  calendar test's newline-dependent mutation and strict-Clippy test issues;
  document the independent parameter tables at the Introspection helper boundary.
  Select the release scope/date and keep actual publication evidence in the manifest;
  exact-candidate validation passed and all eight published assets were hash-verified.
  Refresh website feature status and installation instructions for Preview 8. Generate the advanced
  runner fixture from shared props with inline contracts, so packaged verification
  can copy it to a temporary directory without breaking relative imports. Update
  the full editor matrix to expect Sequence members from Introspection queries;
  align standalone array/match probes with current contracts. Keep the reduced
  probe core compilable by omitting collection-returning String members when its
  collection profile is not selected; the full runtime reference API is unchanged.
  Compare the clock sample’s six local components against the host time interval,
  including DST folds, rather than expecting the retired offset output.

## 0.1.0-preview.8 — 2026-09-19

Raven-authored System.Runtime, unified Introspection and grapheme text. See the
[release notes](docs/preview-8-release-notes.md) for the resulting API and migration.
The dated development history below preserves intermediate choices superseded by
later entries; it is not a list of simultaneously supported contracts. The release
manifest records the exact published candidate and its validation evidence.

### 2026-09-19

- Prepare Preview 8 packaging: use the current direct reference-core generator,
  share the target props across runner and MSBuild demo projects, include nested
  documentation and refresh text/introspection instructions. Format the existing
  ordinal/UTF-8 tests for the stable-Rust CI gate. Publication remains pending
  exact-candidate source and extracted-package validation.

- Implement the approved grapheme text direction: Char owns one validated Unicode
  16 extended grapheme cluster; String.Length and iteration use graphemes, with
  explicit GetScalars returning Sequence<uint> and UnicodeScalar classification.
  Preserve UTF-8 storage/conversion and ordinal equality. Numeric Char casts and
  native integer layout are removed; use the matching RavenGraphemeChar toolchain.
  Iteration currently copies snapshots and integer indexing remains deferred.
  Runtime character payloads count toward array limits. Record the .NET/Swift/Rust
  comparison and migration, regenerate 77 source slices, and validate the compiler,
  saved-project programs, signatures and editor contract. The earlier scalar-Char
  slice below is superseded.
  Independently fixed Raven expression-bodied indexer emission remains on main.
  Refresh the String feature/proposals pages with tested grapheme samples, costs and
  future encoding-specific types. Prepare a separate local VS Code snapshot; its
  expanded saved program and 20 editor completion sections pass.

- Route homepage sample boxes to feature pages and add a Raven language page with
  source-backed examples, explicit mutability and patterns, target distinctions and
  an on-site .rvnproj install/build/run guide, troubleshooting and sample downloads.
  Distinguish published Preview 7 setup from development API availability; record
  the website-first content rule for future updates. Clarify that neoCLR and its
  guest programs run without .NET; the Raven build tools and language server use
  .NET, with the VS Code extension connecting to that server.

- Migrate runtime Char values and native layout to validated Unicode scalars in
  four bytes. Reject surrogate/out-of-range values, preserve supplementary values,
  extend the pinned Unicode 16 classification table and support scalar literals in
  the Neo frontend. This breaks 16-bit Char layout; Raven integration uses
  a separate compiler contract. Complete Raven literal/pattern/array integration
  with `RavenUnicodeScalarChar`, remove surrogate-only predicates and regenerate
  the 76 library slices. Supplementary and invalid Int32 conversion programs run
  through the saved-project pipeline; compiler/runtime/editor checks pass. Wide
  numeric casts retain narrowing before validation; scalar String access remains
  a separate slice. Refresh the local VS Code workspace and website String status
  with the working scalar sample and remaining String-access limits.

- Record the preview-readiness review and fresh local VS Code UTF-8 snapshot.
  Identify scalar Char as the remaining text-model gap and retain full candidate,
  package and cross-platform validation gates. Correct old validation-page context
  so it does not select Preview 6 as the current candidate. No release is published.

- Add concise implementation pages for Option/Result, collections/queries,
  dates/clocks and bounded UTF-8 files. Reuse executable source excerpts, describe
  current limits and link future directions; add the feature index and collection
  proposal summary. All eight pages build with checked links and sample extraction.

- Adopt native UTF-8 as the text direction and change CompareOrdinal from UTF-16
  code-unit order to UTF-8/scalar order. U+10000 now sorts after U+E000; re-sort
  data that depends on the previous ordering. Keep the original proposal and
  decision history; mark scalar Char as migration work, not a compatibility goal. Thirteen ordinal/String runtime tests and the Raven
  boundary sample pass with the new ordering.

- Document the homepage/feature/proposal structure and website review with each
  feature and release. Separate Pages publication from code pushes: pushes and PRs
  still validate; only manual dispatch on main deploys. Isolate event concurrency
  so a push cannot cancel manual publication. No deployment is performed here.

- Add a separate proposals overview and feature-specific “Where we’re heading”
  sections. Distinguish working foundations, open designs and deferred work;
  link source proposals and .NET comparisons without release commitments.
  The site builder now checks all four pages, including proposal links.

- Add a concise String/UTF-8 implementation page with one executable example,
  downloadable checks and explicit open-design status. Keep detailed edge cases
  in technical docs and tests. Correct the Introspection page's stale hierarchy
  limitation. Three site pages build with checked local links and source excerpts.

- Add System.Text.Utf8.Encode(String) → Sequence<Byte> and strict
  Decode(Sequence<Byte>) → Result<String,InvalidUtf8Error>. Preserve BOM, NUL and
  normalization forms; reject malformed UTF-8 without replacement. Conversion
  snapshots bytes; no Encoding framework or Utf8String is introduced. Change
  String.IsEmpty() to the IsEmpty property; rebuild callers and target metadata.
  Retain Char semantics and UTF-16 ordinal ordering. Add executable boundary,
  allocation-limit, service-dependency, signature and editor checks. Four String
  sample programs, 23 edit/rejection checks and 16 runtime tests pass; clean
  bootstrap regeneration reproduces all 76 implementation slices.

- Clarify the final source-spelling decision: retain Raven's unit keyword and ()
  value mapped to neoCLR System.Void for now. Supersede the earlier suggested void
  keyword in the conventions guide; no compiler or runtime behavior changes.

- Document idiomatic Raven conventions using Raven's style and feature guides.
  Simplify getter-only runtime properties, prefer let for local bindings, use case
  constructors and destructuring in ordinary union callers, and expand compact
  sample control flow. Preserve carrier ABI accessors and copy-semantics fixtures;
  this cleanup does not remove the compiler/runtime union contract. Align the
  direct file probe with saved projects’ core/unit and managed-array profile for
  Ok(()); its verifier can select the matching System library. Record the desired
  void source spelling separately from the existing () → System.Void mapping.
  Regenerate the library; 81 saved-project checks, 15 application checks, 18 runtime
  reflection tests, descriptor admission and process/order workflows pass.

- Preserve source parameter-token alignment when the importer lowers an instance
  method to a free function. Reserve token zero for the synthetic receiver, which
  has no CLI Param row. This restores value-type constructor admission and the
  class-identity/value-copy sample after source metadata retention; Raven compiler
  behavior and Runtime Contract settings are unchanged.

- Make TypeInfo the fourth sealed MemberInfo case. Inherit Name, Module and
  MetadataToken; return Option<TypeInfo> from DeclaringType so top-level types have
  no fabricated owner. Preserve nested source ownership through module-scoped
  metadata and reject missing/cyclic owners. Consumers must rebuild, add the
  TypeInfo match arm and unwrap ordinary member owners. Author the sealed family
  together in Descriptors.rvn; use union patterns and case constructors in the
  changed APIs/samples and record that readability rule. Update the feature guide.
  Validate 25 runtime/metadata tests, 19 admission cases, 130 signature checks,
  38 editor checks and five saved samples plus 23 edit/rejection checks.

- Add the first in-depth Introspection feature guide with a runnable Raven tour,
  source-backed highlighted excerpts, shared expected output and sample downloads.
  Explain acquisition, discovery, scoped tokens, Sequence results and sealed member
  matching, with current limits and feedback questions. Update stale homepage API
  descriptions and validate nested pages, relative links and cross-page anchors.
  The tour and 22 shared saved-project checks pass, alongside highlighting and three
  cross-page validation tests. Desktop/narrow layouts were checked locally.
  This development guide is built locally; it does not publish a site or release.

- Plan dedicated next-preview feature pages with in-depth API walkthroughs, tested
  Raven samples, expected output, design comparisons, limits and feedback questions.
  Suggest Introspection first and strings after its later API/runtime story; no pages
  are built or published by this planning slice.

- Complete the Raven Introspection collection migration to Sequence<T>: generic
  arguments, implemented interfaces, enum names, fields, methods, properties and
  parameters now match assembly/module query capabilities. Use Count instead of
  Length and Sequence<Element> instead of array annotations; indexing, iteration and
  query extensions remain available, with no mutation members. Rebuild consumers.
  Preserve independent snapshots and private native array storage; no compiler or
  native-service behavior changes. Eighteen runtime tests, 19 admission cases, 130
  signature checks, nine saved samples plus rejection checks and editor checks pass.
  Close this Introspection story; String API/runtime behavior remains later work.

- Implement RuntimeContext.ExecutingAssembly and sealed AssemblyInfo/ModuleInfo
  interfaces with direct references and module/type discovery through Sequence<T>.
  Add module-scoped MetadataToken to Info interfaces; retain source rows and assign
  tokens to merged runtime definitions. System.Runtime appears as the foundation
  reference. Queries cover retained loaded metadata, never load files, and fault on
  unresolved references or unsupported open-generic member queries. Rebuild consumers
  for the new provider layouts. Add a
  runnable assembly-info sample and editor/interface, token and discovery checks.
  Record dynamic loading as future RuntimeContext work, without adding a loader.
  All 75 slices reproduce; 61 focused runtime tests, 27 admission cases, 24 selected
  saved-project checks and interface/editor checks pass.

- Preserve descriptive source assembly/module identity and definition tokens through
  Raven import, neoIL assembly, JSON artifacts and linking. Record the logical
  System.Runtime dependency instead of compiler bootstrap reference names; retain
  executable IDs separately and reject invalid/duplicate module-scoped tokens.
  Library slices do not copy colliding source rows into their merged module.
  Eighteen metadata/attribute/scope checks and a saved acquisition sample pass.
  This supplies the source identity for the discovery and token APIs above.

- Record AssemblyInfo.ReferencedAssemblies and module-scoped MetadataToken on the
  public Info interfaces as
  requirements for the next discovery slice. Require the System.Runtime reference
  mapping, distinguish reference identity from loading, and document source-token
  collisions when combining bootstrap slices. These requirements are implemented in the discovery slice above.

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
