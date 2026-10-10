# API reference maintenance

RavenDoc publishes this reference inside the single neoCLR website at `/docs/`.
It reads a checked-in compiler reference assembly and the authored XML sidecar,
then renders Raven signatures with the same layout, navigation and development
notice as the Markdown guides. No DocFX build, metadata YAML or second site exists.

The current site build is a development snapshot with a separate published-release link;
new behavior and proposals remain labeled separately.
Comparer policies, the HashMap policy constructor and explicit String comparison
modes are included in Preview 11. StringComparison, both comparison methods and
StringComparer.OrdinalIgnoreCase have type/member coverage; the reference describes
simple-fold/.NET differences and invalid modes. StreamReader.ReadToEnd
now decodes incrementally with earlier malformed-input failure; its signatures are
unchanged and the reader guide records cursor/error-ordering implications. Encoding
selection is implemented through Encoding/Decoder and Encodings.Utf8/Ascii,
with strict typed errors, independent conversion state and reader/writer overloads.
Encoder, EncoderProgress/EncoderState, CreateEncoder and StreamWriter.Finish are
included APIs; custom Encoding implementations must add the factory. Keep generated signatures and authored guides
aligned with the matching runtime and compiler reference artifacts.

Number, NumberParseError/BooleanParseError and concrete primitive Parse are also
included APIs. Number's static operators/identities and inherited ordering have
matching type/member documentation. The numeric guide records the current closed
primitive-only generic specialization limits; no Parsable interface is exposed.

The cached route-mapper slice adds documented RoutePattern.GetParameterNames/Overlaps,
TypeInfo.IsVisible and the ConstructorInfo.Invoke Sequence<Object?> overload. All
use existing selected types; the matching reference and snapshot include their members.
RavenDoc currently labels the Invoke group `self(...)` and spells the emitted
extension signature `static func self`; both overloads and their XML descriptions
remain present. The [manual reflection guide](reflection.md)
provides the intended Invoke spelling. This renderer naming limitation adds no exclusion.

The Function descriptor slice includes FunctionTypeInfo.InvokeMethod and
TypeInfo.IsFunctionType in the generated reference. The [Function family page](functions.md)
documents synthetic Invoke, optional declaration metadata and the dynamic invocation
limit. The API browser now includes authored Array, Function, Tuple, Union and
Intersection family pages through RavenDoc's existing table-of-contents support.
Current nominal Tuple/union declarations and proposed structural forms are explicitly
distinguished. Automatic extraction of arbitrary structural shape members remains
future publisher work; the authored family member references cover current APIs.

Development FunctionTypeInfo also exposes Parameters and ReturnType directly.
InvokeMethod remains the member-reflection view; no generalized function-info
interface is introduced.

Development (2026-10-02) renames the terminal namespace function to `System.Fail`.
The selected namespace container and XML member entry include the new name; host Fault
and FaultCode retain their manual reference. Source callers must migrate from
`System.Fault` and use the matching reference/compiler/runtime bundle.

## One class-library reference across assemblies

Author direction (2026-10-07): present the class library as one reference, organized by
namespace and type. Splitting Runtime/Data/Networking/Web into assemblies or distribution
packages must not create separate API sections, duplicate namespace trees or per-DLL
landing pages. Keep `/docs/` navigation, cross-references and search unified.

A type/member page should identify its actual declaring assembly, and may additionally
show package and file provenance when known. An assembly is the logical metadata and
identity unit; a PE file is its container, often named `.dll`. Package, namespace,
assembly and filename are different facts and should not be conflated. Inherited and
extension members should retain their declaration provenance rather than acquire the
viewed type's assembly. Never guess ownership from a namespace.

Current rendering still uses the aggregate `NeoCLR.CoreProbe.dll` documentation bridge.
That displayed assembly is its real bridge identity, not a Runtime/Data production owner.
Development (2026-10-08): RavenDoc now supports an explicit native loader through
`--native-core-reference` or site `nativeCoreReference`, with `apiInputs` grouping
native libraries into one namespace/type tree and `references` supplying dependencies.
It shares the existing renderer, sidecar reader, navigation, search and source-link
configuration. Native identity/type-name conflicts and missing dependencies reject.
A compiler build with `NeoClrMetadataProject` enables this provider; ordinary .NET
loading remains the default. See the [native loader validation](../docs/experiments/native-core-bootstrap/ravendoc-validation.json).

The author explicitly requires the full API reference experience to remain intact.
The native fixture validates XML/Markdown and namespace comments, overloads, generic
classes/interfaces, fields/properties, inheritance/extensions, cross-library links,
authored API content, configured source links, search and local link integrity.
The existing .NET generation/site suite remains green (63 checks). This does not prove
all production signatures or replace the current complete documentation bundle with
a reduced fixture. Migrate actual library inputs and validate their public-API inventory
before changing the published snapshot. Do not relabel CoreProbe or create one `apis`
group per split class-library DLL. The fixture's native owner labels come from metadata,
even when the file has a different name.

Acceptance for that migration: Runtime/Data/Networking/Web share one namespace tree;
a cross-assembly member type links to its unique type page; declaration ownership is
correct on type, ordinary member, inherited member and extension-member pages; duplicate
identities reject rather than silently overwrite; existing URLs remain stable where
possible. This is a presentation direction, not a requirement to merge runtime binaries.

## Planned bundles and declaration source links

Author clarification (2026-10-07): RavenDoc needs an explicit **assembly bundle**:
a list of assemblies presented as one API structure. This is a documentation grouping,
not a merged assembly. Resolve symbols using assembly-qualified identities before
combining navigation, search and cross-references. Retain each input's documentation
and source provenance, including inputs from different repositories or revisions.
The native loader now accepts such a group through `apiInputs`; richer per-input
source provenance remains future RavenDoc work.

Source links must identify the actual file and declaration location for a type or
member on GitHub, for both .NET and NeoCLR inputs. Repository links or namespace-based
file guesses do not satisfy this requirement. Proposed shared declaration facts are:

- Declaring assembly/artifact identity and unambiguous declaration identity, including
  overloads, generic arity and module-level functions.
- Repository URL, immutable commit, repository-relative document path and source checksum.
- Declaration start/end line and column, with multiple locations retained for partial
  declarations and an explicit generated-source origin when applicable.

The renderer consumes these facts independently of the metadata target; a repository
provider converts them to a browser URL (GitHub file and line anchors first). Inherited
and extension-member entries link to their original declaration, even when displayed
under another type or assembly. Documentation text and source location are separate:
external Markdown can document an API without becoming its declaration source.

Comparison: [.NET Source Link](https://learn.microsoft.com/dotnet/standard/library-guidance/sourcelink)
provides repository provenance alongside symbols. The
[Portable PDB specification](https://github.com/dotnet/runtime/blob/main/docs/design/specs/PortablePdb-Metadata.md)
defines documents/checksums, method sequence points, Source Link mappings and
TypeDefinitionDocument information. These are useful inputs, but method sequence points
are not a universal declaration-span index for types, fields, properties and abstract
members. Reuse available standard information; investigate a compiler-produced declaration
map for missing locations rather than infer a declaration from its first executable line.
The map's encoding, distribution and compatibility remain open decisions, not a new
NeoCLR metadata extension approved by this note.

Implementation order: audit both target readers and RavenDoc source lookup; define one
origin contract; populate it from compiler source declarations and matching artifact
symbols/maps; render links; then integrate bundle inputs. Carry matching provenance
artifacts through project builds and packaging. Validate artifact/symbol identity and
checksums before accepting locations. Missing provenance should omit the precise link
with an actionable diagnostic; stale/conflicting provenance must not silently link to
another version. Local uncommitted sources must not be represented as published commits.

Acceptance includes exact links for types, constructors, overloads, properties, fields,
interface members and module functions; cross-assembly inherited/extension members;
partial and generated declarations; path escaping; multiple repositories; missing and
mismatched symbols. Run equivalent .NET/NeoCLR fixtures. Neither source availability nor
network access should be required to load native metadata or execute an assembly.
The benefit is accurate, portable navigation; the cost is producing and shipping reliable
declaration provenance beyond execution metadata. No new source-link support is claimed
as implemented here.

## Build and refresh

Author direction (2026-09-27): run only validation needed for the change; do not
routinely run the full suite or website build. Keep matching API artifacts current
and run necessary snapshot checks. Use the website commands below when relevant
to the change, explicitly requested, or required by hosted validation.

```sh
python3 scripts/build-website.py
python3 -m unittest discover -s scripts -p 'test_build_website.py'
```

Normal CI requires only Python and .NET 10, using the pinned portable
[RavenDoc build](../tools/ravendoc/README.md). To refresh after a public API change,
first build the matching Raven-target bridge following its README, then:

```sh
mkdir -p target/api-docs/input
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --reference-core target/api-docs/input/NeoCLR.CoreProbe.dll
python3 scripts/build-api-docs.py \
  --refresh target/api-docs/input/NeoCLR.CoreProbe.dll --check
python3 scripts/build-website.py
```

Commit `reference/NeoCLR.CoreProbe.dll`, `snapshot.json`, `types.json` and relevant
XML/Markdown changes together. The assembly is a small documentation/compiler
reference, not executable runtime code. The build copies `NeoCLR.CoreProbe.xml`
beside it locally; that generated copy is ignored. Source fingerprints and the
assembly SHA-256 reject stale/corrupt snapshots. A fingerprint is not proof that an
arbitrary input DLL was produced by those sources: always regenerate from the
freshly built matching bridge. CI never requires that compiler checkout.

## Public API coverage

RavenDoc generates every public type, even without XML documentation. `types.json`
is an automatically refreshed inventory of all externally visible types in the
reference assembly, not a curated allowlist. `tools/api-inventory` uses .NET metadata
reading without loading or executing the reference. Snapshot checks reject an
inventory that omits a public type or includes an internal type.

Provide useful type/member summaries, parameters, results, ownership and limitations
as APIs develop. Missing prose is reported but does not block page generation.
Missing public type pages still fail validation. Authored XML entries must resolve
to real member pages. Documentation is maintained in the XML sidecar, not claimed
to be extracted from Raven source comments.

`manual-types.json` records the exact pinned-renderer limitations and manual routes:
the two assembly-level-function containers and IsReadOnlyAttribute. These remain part of
the public inventory, with linked manual entries and checked routes. No public type
is excluded from the inventory. `exclusions.json` records intentional page exclusions
with reasons: non-generic Array, Option, Result and TaskOutcome are importer/exporter
scaffolds, not additional application types. Their generic APIs remain documented;
CLR case-carrier types inside the non-generic containers are also explicitly
excluded. Coverage checks honor these exact type exclusions.
Keep [reference support](reference-support.md) aligned
when those renderer limitations change.

RavenDoc now renders the previously excluded TaskQueue Post/Run, Task OnCompleted,
TaskAwaiter OnCompleted, FileText WriteAllText and OutputStream/FileOutputStream/
TextWriter/StreamWriter Flush signatures, including Func<Void> and Result<Void,E>.
Their Markdown guides remain behavioral reference, not renderer exclusions.
Existing selected types include the HTTP client/server/context and response configuration APIs,
Tasks, Thread/ThreadPool, Storage, IO, Console,
Object/Value, HashCode, EquatableTo, compiler async support and Introspection descriptors.
Host Fault/FaultCode and debugger fields remain a complete manual [reference](faults.md),
not synthetic CLI types. The Rust-only StringValue payload and owned-text migration
are documented in the [Object guide](objects.md#rust-host-string-payloads) and source
rustdoc; StringValue is not a guest CLI type selected for RavenDoc.

Sequence, Collection, Iterable and Iterator now have generated reference coverage,
including String construction and its read-only grapheme indexer. String Count is
an explicit Collection implementation, visible through Sequence/Collection only.

The current audit covers every public reference type through generated type pages,
explicit manual entries and explicitly excluded metadata scaffolds. Collections, arrays, functions, query operators,
Option/Result, TaskOutcome, numeric types, text/encoding, environment, time/calendar,
resource capabilities and interop now have type/member descriptions. Compiler-reference
scaffolds are identified as such; they do not promise executable CLR services.
Future work can deepen examples and parameter/error descriptions without withholding
undocumented public types from generation. The snapshot fingerprints all Raven
library source files, so older API families receive the same freshness checks.

The generated reference starts at `/docs/api/`. `legacy-routes.json` preserves the
former DocFX type/namespace routes and member anchors as redirects into RavenDoc.
Keep those routes stable for external links. New links should use xrefs or the
current generated routes. The build validates all local page, sample and asset links
under both root and project-path hosting. Publication remains manual and separate
from runtime releases.


`exclusions.json` records the one reference-only ThreadPool constructor that is not
an application API. This preserves the former scaffold exclusion; it is not a
renderer workaround. The combined build verifies every public type and authored member has a generated
or explicit manual route, reports missing summaries, and validates links across the
entire result. Documentation-only XML changes can
refresh the manifest against the existing matching reference assembly; source/API
changes still require regeneration from the bridge.

Char now has generated coverage for FromString, ToString, Equals(Char) and CompareTo.
Its grapheme semantics and boxed Object behavior are documented; the other public primitive types are now included too.

String's existing public text methods, properties and operators now have generated
coverage. Its reference-only parameterless scaffold constructor is excluded in
exclusions.json and explained in the [Object guide](objects.md#string-through-object-development);
it is not an executable application API. UTF-8 slice errors are described on the
method; remaining text/encoding type coverage is still incremental.


String.Intern now has generated member documentation. Its execution-owned retention
and host quotas are described there and in the manual Fault reference, including
InternPoolLimitExceeded. This does not add a public interning-pool type or IsInterned.

Socket and SocketError now have generated reference coverage under
System.Networking.Sockets, including every error case. The on-site socket guide
describes the first TCP client slice and pending listener/send/addressing work.
Private completion classes and runtime operation handles are not public APIs.

Socket.Send has generated member coverage. Transfer snapshots, short writes, shared
budgets and simultaneous send/receive behavior are described in the socket guide.
The private transfer-result service replaces the former receive-only result service;
refresh matching library, importer and runtime artifacts together.

Dns, DnsError and every error case now have generated reference coverage. The public
lookup returns Sequence<String>; private operation and completion types stay hidden.
The Rust reachability service `RuntimeService::NameResolution` identifies host lookup
requirements; submission additionally requires TaskDispatch, not SocketIo or
IsolatedWorkers. Result consumption requires NameResolution without TaskDispatch.


Socket listener coverage includes Listen, Accept, GetLocalPort, AddressInUse and
InvalidOperation, with XML descriptions and generated member routes. The guide
links the separate-process echo sample; private handles/completions remain hidden.


Socket.Connect now includes a `Sequence<string>` overload with generated member coverage.
The socket guide documents full preflight validation, snapshot ownership, duplicate
removal and fixed shared/per-address connection deadlines. The native array service
and completion helpers remain internal; matching reference/bridge/library/runtime
artifacts are required. This does not supply a combined DNS/HTTP request deadline.


System.Web.Http has generated coverage for HttpClient, HttpHandler, HttpSocketHandler,
HttpRequest, HttpResponse, HttpContent and HttpHeader, including constructors and all
public members. The bounded response parser and per-request continuation object are
internal and excluded from the public inventory by visibility. The Web guide documents
provisional string errors, body/ownership limits and the fixed socket-handler exchange budget and remaining handler/server lifetime gaps.
Socket Send/Receive and TimedOut documentation cover the new five-second per-transfer
bound, one-shot completion and connection preservation. Signatures are unchanged.

HttpServer and HttpRequest.Headers have generated member coverage. The server guide
covers one-request ownership, malformed-request closure, computed framing and the
unsupported cancellation/deadline cases. Internal request parsing, encoding and
operation adapters remain outside the public inventory.

The HTTP socket handler now carries a private monotonic stamp through DNS, connection
and transfer submissions. Public signatures remain unchanged. Deadline helpers and
Until methods stay out of the normal application/reference surface; bootstrap-only
cross-slice visibility is restored to internal before importer contract validation.
Refresh reference, bridge, library and native runtime together for this private ABI.

System.Uri and UriError now have generated type/member coverage. Parse and both
Resolve overloads document strict ASCII grammar, the 4096-byte bound, unsupported
IP literals, lexical equality and RFC relative resolution. HttpError is integrated.
HttpClient.BaseUri now provides optional-string base configuration and
matching string/Uri Get overloads. URI syntax failures preserve UriError; HTTP policy
remains bounded. Token-aware HTTP Send and GetString remain pending.


SocketError is now projected from its normal Raven union source when the bridge
builds either core reference. Its old Is*/Get* helpers are removed; the generated
HasValue, Value and conditional TryGetValue members are documented. UnionValue is a
provisional ordinary compiler-support interface. RavenUnionCaseAttribute is excluded
as compiler-reference metadata, with the exact reason in exclusions.json; it is not
an executable runtime API. Rebuild the bridge after changing the embedded source.

DnsError and UriError also use embedded-source projection. Their generated HasValue,
Value and conditional TryGetValue members replace handwritten per-case Is*/Get*
helpers. Their default values are inactive. The core projection runs sequentially
so every family uses the same supplied UnionValue identity.

The same generated union documentation now covers StreamError, TextReadError,
StorageLookupError, FileReadError, FileWriteError, ConsoleReadError, Utf8SliceError,
NumberParseError, IntegerDivisionError and SingleError. EntryKind is an enum instead:
its named fields replace nested union cases and Is*/Get* accessors. Zero is unnamed.

Development cancellation source/token/registration APIs now have generated type and
member descriptions. They are invocation-local and do not yet wire HTTP/native
operations to tokens. Keep this boundary visible when adding consumer overloads.

Native `SocketCancel`/`DnsCancel` operation hooks are bootstrap-only implementation
services, not new public reference APIs. Normal application reference metadata still
omits RuntimeServices; the importer admits these calls only while building the
runtime library. HTTP and managed DNS/socket token forwarding remain pending.

Development networking token overloads (2026-09-25) are included on the existing
Dns and Socket reference pages, with per-member cancellation/ownership descriptions
and the [socket guide](sockets.md#per-operation-cancellation-development). Private
provider registration and shared-deadline helpers remain excluded from application
reference navigation. The matching reference snapshot includes all eight new overloads.

Development HTTP token/text contracts (2026-09-25) are covered on the existing
HttpClient, HttpHandler and HttpSocketHandler reference pages. HttpHandler's former
tokenless Send entry is removed; implementations must accept and forward or honor
CancellationToken. Both GetString address forms, token variants, strict UTF-8 errors,
success-status policy and ownership behavior are documented with the matching
reference assembly. Website building remains skipped by explicit author direction.

The final-status slice adds HttpResponse.IsSuccessStatusCode and the source-projected
HttpError.UnsuccessfulStatus case. The matching reference documents 200–599 responses,
GetString's 200–299 policy and bodyless status rules. Website sources are updated;
building the website is deferred by the author's focused-validation direction.

The named-status checkpoint adds HttpStatusCode and its initial common constants to
reference navigation. HttpResponse.StatusCode and UnsuccessfulStatus carry the enum;
the typed response constructor is documented alongside the retained integer overload.
The signature change requires rebuilding development consumers. Unnamed codes and
optional property-pattern inspection are covered by the target status fixture.

### Buffered POST checkpoint (2026-09-25)

The selected HTTP types include Post string/Uri/token overloads, HttpRequest.Post and
Content, HttpContent.FromText and the content-type constructor. XML documents limits,
body ownership, errors and cancellation. Encode and MediaType remain library-internal.
The snapshot is refreshed with the matching bridge. Website build is skipped for this
slice by author direction; feature source and downloadable sample are maintained.

### Header lookup checkpoint (2026-09-25)

Both selected request/response types document GetHeaderValues, including ordering,
case rules, absence and invalid names. The shared HttpHeader.FindValues implementation
remains internal. Stream-backed content remains planned and has no public signature.

### Request header construction (2026-09-25)

HttpRequest.WithHeader is included in the selected request type and XML reference.
Its copy/replacement behavior, shared content, reserved fields and validation limits
are documented with the matching bridge/library snapshot. No new type is omitted.

### Common verb helpers (2026-09-25)

The selected HttpClient/HttpRequest types now cover Put, Patch and Delete, including
all string/Uri/token overloads and request factories. XML distinguishes supported
buffered bodies, BaseUri resolution, cancellation and response-status policy. No new
type selection is needed; snapshots must match the updated bridge and managed library.

MemoryStream is included with all public members in the development reference. Its
64 KiB bound, shared cursor, zero-fill behavior and close/error contracts are documented.
The later System.Data.Json slice now exposes a provisional public DOM and DOM-only
JsonSerializer. JsonError retains typed lookup/conversion and nested I/O failures.
Every public type is inventoried; codec helper classes remain internal.

System.Runtime.Reflection now has generated type/member coverage and a linked
[execution guide](reflection.md). The Rust host's MetadataOrigin additionally carries
optional publicly_visible and member_access source-access fields; SourceAccess lists
Public, Private, Assembly, Family, FamilyOrAssembly, FamilyAndAssembly and
CompilerControlled. These host/artifact fields are documented in that guide, not
invented as guest CLI APIs. The website build was explicitly skipped for this slice;
the matching reference assembly, XML and inventory were refreshed and checked.


2026-09-25 JSON mapping update: the four Object/TypeInfo serializer overloads and
JsonError.Reflection/UnsupportedMapping cases are included in the matching reference
snapshot and XML summaries. The linked JSON guide describes exact-name shallow
properties, strict presence/null rules and stream/error ownership. No public mapping
API is excluded from RavenDoc. Website build is skipped at the author's direction;
snapshot and sample archive source checks remain required.

2026-09-25 directional interfaces: EquatableTo and ComparableTo replace their old
identities; ConvertibleInto adds Convert() returning T. All three have public type/member
coverage. Refresh the matching reference and inventory; website build is skipped by
explicit author instruction for this change.

`manual-members.json` supplies explicit member routes when RavenDoc renders a type
but omits individual member pages. JsonError case payloads currently use
[the linked payload reference](json-error-payloads.md). Every entry records the
renderer limitation; the site checks the destination and anchor.

System.Web.Http.Json.JsonContent now has type/member XML and automatic type selection
for synchronous buffered Create/CreateNode/Read/ReadNode conversions, including
generic and TypeInfo reads. The API landing page links the namespace's public type.


The generic HTTP JSON client slice adds HttpClientJsonExtensions and HttpJsonError,
including eight string/Uri and optional-token overloads. Type/member XML, namespace
navigation and the API landing page describe the buffered limits and error/cancellation
policy. HttpJsonError case payload properties use the same explicitly linked manual
payload reference as JsonError until RavenDoc produces those individual pages.

### Calendar/globalization checkpoint (2026-09-27)

Calendar, Culture, Language, DateTimeFormat, CultureProvider, FixedCultureProvider and SystemCultureProvider are automatically selected from the matching reference. Date arithmetic/display and LocalDateTime.Create have XML coverage. Internal calendar/formatter rules are deliberately not public APIs; no public type is excluded. System discovery is a ProcessEnvironment service. The API snapshot is refreshed and checked; website source is updated without an unrelated full build.

### Time and zones checkpoint (2026-09-27)

DateTime, TimeOffset, TimeZone, ZonedDateTime, LocalTimeMapping and TimeZoneError are
source-projected development APIs, automatically selected into the reference.
DateTime is a parenthesized union of existing types, not nested wrapper cases.
Every public addition has XML coverage; LocalTimeMapping payload properties use
[the explicit manual member routes](time-zone-mappings.md) for the existing RavenDoc
inline-case limitation. Time display/arithmetic and LocalDateTime/Instant arithmetic
are documented with ranges, errors and precision. Website samples come from executed
consumers; a full site build remains outside this focused slice.

Focused RavenDoc rendering confirms the DateTime type and member pages. Its heading
omits the parenthesized variant list; [the linked declaration supplement](date-time-union.md)
records the exact signature. No type is excluded for this limitation. LocalTimeMapping
payload routes were confirmed absent and are supplied by manual-members.json.

## Casing and Int64 development slice

String.ToUpperInvariant/ToLowerInvariant, Int64.Parse/ToString/MinValue/MaxValue
and standard NumberParseError now have generated type/member coverage and XML
contracts. [Casing and decimal reporting](text-numbers.md) explains Unicode 17 full
mappings, .NET differences, typed failures and static bounds properties. Matching
native runtime, library and reference artifacts are required; no manual exclusions.

### Reflection member checkpoint (2026-09-27)

ConstructorInfo, GetConstructors, typed/argument-based CreateInstance, MethodInfo.Invoke,
FieldInfo.GetValue/SetValue and the additional ReflectionError cases have matching
XML contracts and selected reference coverage. The feature page is separate from
introspection; the API guide owns signatures and restrictions. Source field access
and read-only admission metadata require matching development artifacts.

Development `System.Runtime.GC` is selected in RavenDoc with all six counter properties
and Collect/KeepAlive methods; [the guide](gc.md) records execution scope and count units.

The GC/Reflection website build also exposed omitted LocalTimeMapping Unique and
Ambiguous Deconstruct pages. Their out-parameter signatures now have explicit
[manual member entries](time-zone-mappings.md), registered in manual-members.json;
no public API is excluded.

### Task.Run development checkpoint (2026-09-27)

The static System.Tasks.Task owner and all three Run overloads have public inventory
and XML coverage. The matching bridge/reference/library implement shared captures,
completion-only and typed work, and async unwrapping. ScheduleTask and helper carriers
remain private implementation details. [The callback guide](callbacks.md#task.run-development)
records native execution, invocation limits, default-queue behavior and the bounded
generic application import contract. Inline block callbacks now infer their value result.
Ordinary async mutable-local sharing is corrected by the integrated Raven closure fix.
Direct completion-only await is corrected by the target compiler unit-result fix.
Generic capture metadata is corrected in Raven; neoCLR admits bounded ordinary closed
static generic helpers and their constructed async state-machine/closure types. Two
forced suspensions, captures, identity and cancellation have dedicated consumers. The XML-only API snapshot
refresh preserves the unchanged reference assembly; the
existing library snapshot is reused. No full suite or website build is part of this slice.


RoutePattern and RouteMatch (development 2026-09-27) add four public methods.
All have XML contracts and automatic RavenDoc selection; the [routing guide](routes.md)
covers direct matching, typed parameters, errors and optional application unions.
There are no manual exclusions. Match their reference snapshot to the HTTP library.

Attribute data (development 2026-09-27) adds MemberInfo/ParameterInfo retrieval,
CustomAttributeData and CustomAttributeTypedArgument. All public signatures have
XML coverage and automatic type selection; the introspection guide documents exact
retention, constants, non-execution and source limitations. No manual exclusions.


The 2026-09-27 constructor execution increment includes ConstructorReflectionExtensions
and Invoke in generated type/member coverage, with exact boxing/access limitations
in [the reflection reference](reflection.md). TypeInfo activation is unchanged.

### Tuple development checkpoint (2026-09-28)

All seven System.Tuple arities, constructors and fields have XML documentation and
automatic public type selection. [The guide](tuples.md) records the bounded surface
and naming difference from .NET. TupleElementNamesAttribute is a compiler-reference
scaffold with an exact exclusion, not an executable guest API.

### Structural family documentation direction (2026-09-28)

The development reference exposes nongeneric System.Number and System.Clonable,
using the fieldless Self transport marker for implementing-type signatures.
These APIs require matching Raven neoCLR target settings and the updated runtime.
Structural Function metadata remains excluded from main. On codex/extended-cli-metadata,
the snapshot is now regenerated from the merged Function bridge with System.Fail preserved.

## Experimental .NET metadata tooling (2026-09-30)

`NeoCLR.Metadata.Experimental.MetadataEnvelope` and
`NeoCLR.Metadata.Experimental.MetadataSection`,
`NeoCLR.Metadata.Experimental.TypeExpression`,
`NeoCLR.Metadata.Experimental.SignatureContext` and
`NeoCLR.Metadata.Experimental.StructuralSignature`, plus the same namespace’s
`MetadataReference`, `MetadataDefinition`, `ReferenceBindings`, `ReferenceTable`,
`StructuralIdentity`, `ResolvedTypeIdentity`, `StructuralMemberReference`,
`ResolvedMemberIdentity`, `StructuralMemberDescriptor`, `StructuralMembers`,
`MetadataProfile`, `MetadataProfileDocument`, `MetadataArtifactReader` and
`MetadataArtifact`, `RuntimeAssemblyContainer`, `NativeModuleContainer`, plus `NeoCLR.Metadata.Experimental.Model.AssemblyDefinition`,
`NativeAssemblyDefinition`, `NativeLibraryDefinition`, `NativeFunctionDefinition`, `ModuleDefinition`, `TypeDefinition`, `TypeReference`, `AssemblyIdentity`,
`AssemblyReference`, `IAssemblyResolver`, `MethodDefinition`, `MemberReference`, `ImportedMethodReference`, `AssemblyBuilder` (including native emission
and top-level functions), `TypeBuilder` and
`MethodBuilder`, `PrimitiveType`, `PrimitiveMethodSignature`, `MethodSignature`, `SignatureType`, `LocalDefinition`, `BranchLabel` and `OpCode` in that Model namespace,
are .NET-host-only types in
`tools/metadata/NeoCLR.Metadata.Experimental`, not types in NeoCLR.CoreProbe or the
Raven guest library. They therefore cannot be added to that assembly's RavenDoc type
selection or refreshed from a guest compiler bridge. Their complete signatures,
parameters, ownership, limits, failures and compiled-consumer example are maintained
in [the manual host reference](experimental-metadata.md), linked from `/docs/`.
This is an explicit host/guest assembly boundary, not a RavenDoc rendering defect or
an undocumented public-type exclusion. The separate project generates XML docs with
warnings treated as errors. The existing guest API snapshot remains unchanged and
must still pass `scripts/build-api-docs.py --check`. When native/guest metadata APIs
are introduced, add those actual types and members to the matching guest reference.

The host reference also covers the development String signature/local contract,
Emit(OpCode, string), Ldstr and the stack-consuming WriteConsoleLine() overload.
These remain host-only APIs under the same explicit guest-reference exclusion above.

The host reference includes Starg/StoreArgument and its typed by-value slot contract.

The host reference covers TypeVisibility, the explicit AddType overload and
TypeBuilder.Visibility, including native/reference projection consistency.

The host reference includes OpCode.Div and MethodBuilder.Divide with typed stack and execution-fault contracts.

OpCode.Rem and MethodBuilder.Remainder are covered in the same host reference, including the CLR edge-case qualification.

And/Or/Xor and BitwiseAnd/BitwiseOr/BitwiseXor are covered in the manual host reference.

Shl/Shr and ShiftLeft/ShiftRight include count-width validation and the CLI/native out-of-range count distinction in the manual host reference.

MethodVisibility, the explicit AddMethod overload and MethodBuilder.Visibility are covered by the manual host reference; they remain excluded from guest RavenDoc selection.

Assembly-function namespace overloads, MethodBuilder.Namespace and the extended ImportedMethodReference.Namespace contract are covered in the manual host reference under the same guest RavenDoc exclusion.

AddClass, TypeBuilder.IsStatic/Fields/AddField, FieldVisibility/FieldBuilder/FieldDefinition, TypeDefinition.Attributes/Fields and ModuleDefinition.Fields/GetFieldDefinition are documented in the manual host reference, under the existing explicit guest RavenDoc exclusion. AddField and FieldBuilder.FieldType now use the same host-only SignatureType for primitive and owned nominal fields; the manual reference includes that development API migration.

The same manual host reference now includes root constructors, instance methods,
receiver slots, Dup/Newobj/Ldfld/Stfld and their overloads. These remain C# host APIs
excluded from the RavenDoc guest assembly; no guest runtime API was added.

PropertyBuilder, TypeBuilder.Properties and AddProperty are also covered by the
manual host reference; they are C# host APIs outside the RavenDoc guest assembly.

PropertyDefinition and module/type property collections, token lookup, copied signatures
and owned accessor links are documented in that same host reference, outside RavenDoc.

The manual host reference covers root-class DeclareLocal, nullable LocalDefinition.Type
and ClassType, including the development API migration. These C# host APIs stay outside
the RavenDoc guest assembly.

PropertyBuilder.PropertyType and TypeBuilder.AddProperty also use the host-only
SignatureType for owned nominal properties (2026-10-01). Their signatures, accessor
identity rules and rebuild migration are covered in the manual host reference; the
existing experimental .NET API exclusion from guest RavenDoc remains unchanged.

FieldBuilder.IsReadOnly and AddField's optional isReadOnly argument (2026-10-01) are
also covered in the manual host reference under the same RavenDoc exclusion. The
reference documents writer restrictions, native runtime enforcement and rebuild/runtime
compatibility requirements; no guest Raven public API is added by this slice.

The same host-only exclusion covers SignatureType.ArrayOf/ArrayElement,
LocalDefinition.SignatureType and MethodBuilder vector helpers/raw typed Emit overload
(2026-10-01). Their signatures, errors, CLI/native representations and current element
limits are documented in the [manual metadata reference](experimental-metadata.md#vector-declarations-development-2026-10-01).

PropertyBuilder.ParameterTypes and indexed AddProperty associations remain host-only
under the existing RavenDoc exclusion. See the [indexed property reference](experimental-metadata.md#indexed-property-associations-development-2026-10-01) for accessor validation and projection limits.

SignatureType.MethodParameter/MethodParameterIndex and MethodSignature.GenericParameterNames
are host-only and covered by the [manual generic reference](experimental-metadata.md#generic-method-declarations-development-2026-10-01), under the existing explicit RavenDoc exclusion.

The experimental host C# `GenericMethodInstance` and generic call overloads are covered
by [the manual metadata reference](experimental-metadata.md); these host APIs are not
RavenDoc input types.

Host metadata local-address/default helpers and Ldloca/Initobj are documented in the
manual experimental reference, outside RavenDoc's Raven type selection.

The C# static-generic-owner APIs (AddGenericType, TypeParameter and
ConstructedMethodReference) use the linked experimental manual reference and remain
outside RavenDoc's Raven source type selection.

The host-only experimental metadata manual also covers GenericTypeInstance (identity,
arguments, equality/hash), AddGenericClass, TypeBuilder.MakeGenericInstance, constructed
class signatures and constructed NewObject/Emit overloads. These C# producer APIs remain
excluded from RavenDoc's guest reference assembly; see [manual reference](experimental-metadata.md#generic-reference-classes-development).

The experimental metadata manual includes the expanded AddProperty contract for
static/instance generic owners, scoped value/index signatures and canonical native
accessor-owner validation. It remains part of the host-only C# manual coverage, outside
RavenDoc's guest reference assembly; see [manual reference](experimental-metadata.md#properties-on-generic-owners-development).

ConstructedFieldReference and the constructed LoadField/StoreField/Emit overloads are
covered by the [host-only metadata manual](experimental-metadata.md#constructed-fields-development),
with the same explicit RavenDoc guest-assembly exclusion as the other C# producer APIs.

GenericTypeConstraint and TypeBuilder.GenericConstraints/AddBaseTypeConstraint are
covered in the [host-only metadata manual](experimental-metadata.md#nominal-type-constraints-development),
with explicit guest RavenDoc exclusion as for the other C# producer APIs.

TypeParameterConstraints and TypeBuilder.SpecialConstraints/SetSpecialConstraints are
covered in the [host-only metadata manual](experimental-metadata.md#special-type-parameter-requirements-development).
They remain explicitly excluded from guest RavenDoc selection because they are C# producer APIs.

AddInterface/AddGenericInterface, TypeBuilder.IsInterface/AddInterfaceMethod and
MethodBuilder.IsAbstract are covered by the [host-only metadata manual](experimental-metadata.md#interface-declarations-development).
These C# producer types remain outside guest RavenDoc selection; abstract interface
methods retain no body in the native reader's CLI reference projection.

The same interface manual section now covers TypeBuilder.BaseInterfaces/AddBaseInterface
and AddProperty associations for abstract interface accessors. Host-only C# coverage
remains separate from the guest RavenDoc snapshot.

The host-only metadata manual also covers interface-valued SignatureType identities
and MakeGenericInstance constructions; these remain within the same explicit C#
RavenDoc exclusion. No guest runtime public API was added.

ImplementedInterfaces/AddInterfaceImplementation, CallVirtual and OpCode.Callvirt
are documented in the host-only metadata manual; the existing explicit RavenDoc C#
producer exclusion still applies. Guest runtime API selection is unchanged.

TryGetStaticValueSignature and the primitive-vector import/resolution extensions are
covered by the [host metadata manual](experimental-metadata.md#imported-primitive-vectors-development-2026-10-01).
These C# producer APIs remain explicitly excluded from guest RavenDoc selection;
no guest API reference assembly change is needed.

ImportedGenericMethodReference, ImportedMethodReference.MakeGenericInstance,
TryGetStaticGenericValueSignature and imported-generic Call/Emit are host C# APIs
covered by the [manual reference](experimental-metadata.md#imported-generic-methods-development-2026-10-01),
explicitly outside guest RavenDoc type selection. No guest API was added.

ImportedTypeReference, the type-definition ImportReference overload, generic
construction/equality members and SignatureType.ImportedType/conversion are covered
in the [host-only metadata manual](experimental-metadata.md#imported-type-signatures-development-2026-10-01).
They are C# producer APIs explicitly excluded from the guest RavenDoc type selection;
no guest reference assembly change is required.

Consumer-scoped arguments and emission-time generic scope checks for imported generic
calls are updated in the same host-only manual; guest signatures remain unchanged.

Nominal method imports extend the existing host C# ImportReference contract; the
[manual reference](experimental-metadata.md#imported-nominal-method-signatures-development-2026-10-01)
records signature support and reader limitations. Guest RavenDoc selection is unchanged.

AddValueType/AddGenericValueType and producer/snapshot IsValueType properties are
host C# metadata APIs covered by the [manual reference](experimental-metadata.md#owned-value-types-development-2026-10-01).
They remain explicitly excluded from guest RavenDoc selection; no guest reference
assembly/signature changed. ClassType/GenericTypeInstance category semantics are
updated in that same entry.

Authored assembly/type/field constructors, facade Definition properties, ForDefinition,
explicit Module.ImportReference and authored WriteNativeAssembly are host-only C# APIs
covered by [the manual reference](experimental-metadata.md#authored-definitions-first-migration-slice).
They are excluded from guest RavenDoc selection for the same host-language reason.

MethodBuilder.Definition and MethodDefinition.AuthoredSignature/Namespace are also
covered in the manual host metadata reference; guest RavenDoc selection is unchanged.

The host manual reference also covers direct MethodDefinition function construction,
MethodBuilder.ForDefinition, append-only Module.Functions and authored EntryPoint assignment.

Direct static type-method MethodDefinition construction and append-only TypeDefinition.Methods
are covered in the host manual reference; no guest RavenDoc type selection changes apply.

The host MethodDefinition CLI-attribute constructor’s instance/constructor cases, errors
and limitations are documented in the manual reference; guest RavenDoc remains unchanged.

Direct nongeneric interface TypeDefinition and abstract MethodDefinition construction
are covered in the manual host reference; guest RavenDoc selection is unchanged.

InterfaceImplementation and TypeDefinition.Interfaces are host C# APIs documented in
the manual metadata reference; they are excluded from guest RavenDoc selection.

MethodBodyDefinition and MethodDefinition.Body are host C# APIs documented in the manual
reference, including local/label views and clearing; guest RavenDoc selection is unchanged.

PropertyDefinition authored constructor/signatures, PropertyBuilder.Definition and authored
property collections/views are documented in the manual host reference; no guest selection changes.

The direct generic TypeDefinition overload and authored GenericParameterNames/constraint
views are covered in the manual host reference; no guest RavenDoc selection changes apply.

ImportedTypeReference.IsValueType and expanded value import behavior are host C# APIs
covered in the manual reference. The native value_type_references annotation is documented
there as a transport contract; guest RavenDoc selection is unchanged.

ImportedConstructedMethodReference and its properties, imported dispatch category
properties, MakeConstructedReference, typed Call/CallVirtual/Emit overloads, and the
closed generic InterfaceImplementation constructor/TypeArguments and builder overload
are covered in the manual [experimental metadata reference](experimental-metadata.md).
They remain host C# APIs excluded from the guest RavenDoc type selection; the guest
reference assembly/snapshot does not change. Native reader projection of those
relationships is documented alongside the APIs and validated by C# execution tests.

The experimental host-only `MethodBuilder.LoadObject`, `StoreObject`, and `OpCode.Ldobj`/
`Stobj` are documented in the [typed local operations reference](experimental-metadata.md#typed-local-initialization-development).
They remain excluded from the guest RavenDoc assembly because the metadata producer is
C#, not a guest runtime API. The guest API snapshot is unchanged by this slice.

Host-only `SignatureType.ByReference`/`ByReferenceElement` and updated method-signature
readers are covered by the [managed-reference manual](experimental-metadata.md#writable-managed-reference-parameters-development-2026-10-02).
The existing guest RavenDoc exclusion applies: these APIs belong to the C# metadata
producer, so no guest reference-assembly snapshot changes are required.

The host-only `MethodSignature` outParameters constructor argument and OutParameters
property are covered in the [output parameter manual](experimental-metadata.md#output-parameter-contracts-development-2026-10-02).
Existing C# producer exclusion from guest RavenDoc applies; the guest snapshot is unchanged.

The C# metadata host's `ImportedMethodReference.RequiresManagedReceiver`, expanded
`TypeBuilder.AddInstanceMethod`/definition attachment and receiver validation contract
are covered by [managed value receivers](experimental-metadata.md#managed-value-receivers-development-2026-10-02).
The existing host-only RavenDoc exclusion applies; guest reference signatures are unchanged.

Host metadata development adds `MethodBuilder.Fail(string)` and `OpCode.Fail` to the
existing manual [experimental metadata reference](experimental-metadata.md). They remain
host C# APIs outside the guest RavenDoc assembly/type selection; the manual entry covers
CLI/native differences, operands, errors and flow restrictions.

The host metadata manual reference also covers value constructor admission,
`ImportedMethodReference.IsConstructor`, imported `NewObject` overloads and matching raw
Newobj emission. These host C# APIs remain outside the guest RavenDoc selection; no
public guest signature changes are part of this constructor slice.

Host metadata nested declaration APIs (NestedTypes, AddNestedClass, AddNestedValueType)
are covered in [the experimental manual](experimental-metadata.md#nested-declarations-development-2026-10-02).
They remain host C# APIs outside the guest RavenDoc selection.

The same host manual covers ImportedTypeReference.DeclaringType and
AddNestedGenericValueType; neither is a guest RavenDoc API.

Structural types can have members and extension members without a declared type
name. TypeInfo member queries remain common; NominalTypeInfo adds declaration
identity. Future RavenDoc support should describe Array, Tuple, Union, Intersection
and Function families, including their members, extensions and shape signatures,
with ordinary API-reference detail. Family page titles and navigation keys are
documentation identities, not synthesized nominal type names. The current snapshot
covers the implemented common/nominal interfaces; generic structural-family rendering
remains open. See [the Function design](../docs/function-types.md#structural-members-and-documentation).

The Function family's current manual member reference is [functions.md](functions.md).
CLI Delegate/MulticastDelegate, Func (one through five generic arguments) and
Action (zero through four arguments) remain in the complete reference inventory
but have explicit scaffold exclusions: they carry compiler metadata and do not
define nominal runtime types. Their public callback signatures remain documented
under their actual CLI IDs. Their target-only synthesized Function property is
documented in the manual Function family page, including MethodInfo results and
GetProperties/GetMethods discovery. This is distinct from omitting a runtime API.


The development Number reference is nongeneric and uses the public fieldless
`System.Runtime.CompilerServices.Self` transport marker. Both types are included
in the inventory and XML documentation. RavenDoc displays that metadata marker;
[text and numbers](text-numbers.md) explains Raven's `Self` spelling and migration.
The marker is not an executable CLR API or a native value constructor.

FunctionSignature, FunctionBinding, SignatureType.Function/FunctionSignature and
MethodBuilder BindFunction/InvokeFunction/raw operands are host C# development APIs,
covered in [the structural Function manual](experimental-metadata.md#structural-function-bodies-development-2026-10-02).
They are not omitted guest RavenDoc types.

The host metadata manual covers concrete imported value overrides and their managed
receiver/direct-call restrictions (2026-10-02); no Raven public API was added.

The manual host API reference includes CastReference and raw Castclass operands,
reference-only validation, native conversion limits and executable evidence (2026-10-02).

Development explicit library binding (`AssemblyBuilder.BindNativeLibrary`) and the
Rust host `metadata_origin` binding structs/AssemblyMetadata vectors are covered by
[the host metadata manual](experimental-metadata.md#explicit-translated-library-linkage-development-2026-10-02).
They remain excluded from the guest RavenDoc reference because that assembly contains
Raven class-library APIs, not these C#/Rust host APIs. No guest snapshot signature changes.

The development constructed-interface `AddBaseInterface` and constructed `CallVirtual`
overloads are documented in the [host metadata manual](experimental-metadata.md#constructed-interface-inheritance-development-2026-10-02).
They share the existing C# host-library exclusion from the guest RavenDoc snapshot;
no guest class-library public signature changed in this slice.

The development host APIs AssemblyDefinition.ReadNativeAssembly/IsNative and
MethodDefinition.TryGetSignature are covered in the [direct native declaration manual](experimental-metadata.md#direct-native-declaration-reading-development-2026-10-02).
They are C# tooling APIs outside the guest reference assembly/RavenDoc selection;
no guest runtime API is added by this reader slice.

ImportedFieldReference, AssemblyBuilder.ImportReference(FieldDefinition, core) and the
MethodBuilder LoadField/StoreField/raw Emit overloads are host C# development APIs covered
in [the imported-field manual](experimental-metadata.md#imported-primitive-field-operands-development-2026-10-02).
They are outside the guest RavenDoc selection; no guest public type/signature is added.

SignatureType.ReferencedType and the extended native nominal method import profile are
covered in [the host metadata manual](experimental-metadata.md#native-local-nominal-signatures-development-2026-10-02).
They remain excluded from guest RavenDoc selection because they belong to the C# host
metadata library; no guest reference assembly signature changes.

FieldDefinition.TryGetSignature and native nominal field import are covered in the
[host metadata manual](experimental-metadata.md#native-local-nominal-fields-development-2026-10-02).
These C# tooling APIs remain outside guest RavenDoc selection; the guest public API
and reference assembly snapshot are unchanged.

The resolver-taking AssemblyBuilder.ImportReference method/field overloads and native
assembly-scoped TypeReferences are documented in the [host metadata manual](experimental-metadata.md#external-native-nominal-signatures-development-2026-10-02).
They remain C# tooling APIs excluded from guest RavenDoc selection; guest public
signatures and the reference assembly snapshot are unchanged.

The expanded native array signature profile uses existing host APIs and is covered in
[the metadata manual](experimental-metadata.md#native-vector-signatures-development-2026-10-02).
These C# host APIs remain excluded from guest RavenDoc selection; no guest reference
assembly signature changes.

PropertyDefinition.TryGetSignature and the native property read profile are documented
in [the host metadata manual](experimental-metadata.md#native-non-indexed-properties-development-2026-10-02).
They are C# host tooling APIs, explicitly excluded from guest RavenDoc selection;
the guest reference assembly and documentation snapshot signatures are unchanged.

The indexed PropertyDefinition.TryGetSignature overload is covered by the
[host manual](experimental-metadata.md#native-indexed-property-signatures-development-2026-10-02).
It remains excluded from guest RavenDoc because it is a C# host API; the guest
reference assembly and snapshot are unchanged.

Native TypeDefinition.Interfaces materialization and interface import/conformance are
covered in [the host manual](experimental-metadata.md#direct-native-interface-definitions-development-2026-10-02).
These C# host APIs remain excluded from guest RavenDoc; guest snapshot signatures are unchanged.


The development C# `NeoCLR.Metadata.Experimental.Introspection` namespace now exposes
MetadataLoadContext, AssemblyInfo, ModuleInfo, TypeInfo and NominalTypeInfo. All public
members are covered in the [host manual](experimental-metadata.md#metadata-only-introspection-facade-development-2026-10-02).
These host-only types are not guest Raven APIs and do not belong in the RavenDoc input
assembly. Subsequent constructed/member facade coverage is recorded below.

Host facade coverage now also includes PrimitiveTypeInfo, ArrayTypeInfo,
GenericParameterTypeInfo, ConstructedTypeInfo and FieldInfo, with every current member
in [constructed and field views](experimental-metadata.md#constructed-and-field-views-development-2026-10-02).
Subsequent method/property facade coverage is recorded below.

MethodInfo, ParameterInfo and MethodGenericParameterTypeInfo are now included in the
[manual host facade reference](experimental-metadata.md#method-and-parameter-views-development-2026-10-02),
including scope projection and module/type enumeration. Guest snapshot selection is unchanged.


PropertyInfo and NominalTypeInfo/ConstructedTypeInfo.GetProperties/GetDeclaredInterfaces
are covered member by member in the [host manual](experimental-metadata.md#property-and-direct-interface-views-development-2026-10-03).
These are host C# APIs, excluded from the guest RavenDoc input for that reason; guest
reference and documentation snapshot remain unchanged.

Host GetInterfaces APIs on nominal/constructed views are covered in the
[manual reference](experimental-metadata.md#inherited-interface-views-development-2026-10-03);
these C# APIs remain excluded from guest RavenDoc.

Host MethodInfo generic construction/definition APIs and updated GetGenericArguments
semantics are covered in the [manual reference](experimental-metadata.md#generic-method-construction-development-2026-10-03).
They remain outside guest RavenDoc inputs because they are C# host APIs.

Host MetadataAccessibility and declaration/constructor view members are covered in the
[manual reference](experimental-metadata.md#declaration-facts-development-2026-10-03).
These C# APIs do not belong in the guest RavenDoc input; guest signatures are unchanged.

2026-10-03: external interface completion/relationship overloads and authored-graph binary PE emission are covered in the [host metadata manual](experimental-metadata.md#external-interface-declarations-development-2026-10-03); these C# host APIs remain outside guest RavenDoc selection.


### Expanded reference refresh blocker (2026-10-03)

Adding the primitive-only bootstrap command changes a tracked reference-generator input.
The primitive mode and source-owned iteration/collection execution pass on both targets.
Regenerating the expanded consumer reference with the freshly built bridge fails in
`SourceUnionReferences.Project` with `RAV0103: 'None' is not in scope`. The previous
verified `reference/NeoCLR.CoreProbe.dll` and its snapshot manifest are preserved; the
source-fingerprint check currently reports stale. Do not bless the old binary with new
input hashes. Reproduce with the normal `--reference-core` refresh command above, using
Raven `f9841b0f6` and the current bridge, then repair and regenerate before claiming a
current expanded API snapshot. No published API contract changed in the primitive-mode slice.

2026-10-03: `SignatureType.Self`/`IsSelf` and `Introspection.SelfTypeInfo` are documented
in the [host API manual](experimental-metadata.md#native-self-signatures-development-2026-10-03).
These .NET host APIs are excluded from guest RavenDoc selection because that generator
loads Raven guest declarations. The existing expanded-reference fingerprint blocker above
still causes `build-api-docs.py --check` to fail; no snapshot hashes have been relabelled.

The 2026-10-03 storage-bootstrap generator mode also changes tracked generator inputs.
It adds no public guest or host metadata API; the existing expanded-reference refresh
blocker remains open. The old reference snapshot is intentionally preserved.

2026-10-03: `ParameterInfo.PassingMode` and the `ParameterPassingMode` enum are covered by
[the host metadata manual](experimental-metadata.md#parameter-passing-modes-development-2026-10-03).
These C# host APIs remain excluded from guest RavenDoc because it consumes Raven guest
declarations. The separately recorded expanded-reference refresh blocker remains open.

2026-10-03 union payload update: inline value fields and direct native value snapshots
are covered in the [host-only manual](experimental-metadata.md#union-payload-foundation-development-2026-10-03)
and XML summaries, under the existing C# host RavenDoc exclusion. Snapshot check remains
stale at the previously recorded SourceUnionReferences.Project / None regeneration
blocker. Existing verified guest reference artifacts are retained; hashes were not advanced.

2026-10-03 native nested case reading/imports extend the existing host-only API profile;
see [manual reference](experimental-metadata.md#native-nested-case-metadata-development-2026-10-03).
The C# host API remains excluded from guest RavenDoc. The previously documented stale
reference regeneration blocker is unchanged; no reference hashes are advanced.

Host metadata Byte signature and `OpCode.Conv_U1` coverage is maintained in the
[manual host reference](experimental-metadata.md#byte-signatures-and-il-generation-development-2026-10-03)
and matching XML comments. The existing reference/snapshot regeneration blocker remains;
this host-only API is not represented as a regenerated runtime reference assembly.

Value-type interface relationships and the host-only IILGenerator constrained-call/typed
Emit overload are documented in the [manual host reference](experimental-metadata.md#value-interfaces-and-constrained-calls-development-2026-10-03)
and XML. These C# host APIs remain excluded from RavenDoc's runtime reference assembly;
the previously recorded reference regeneration blocker is unchanged.

Host-only TypeBuilder.AddOverride and the detached MethodDefinition override profile
are covered by XML and the [manual reference](experimental-metadata.md#value-override-authoring-development-2026-10-03).
These C# APIs remain outside guest RavenDoc type selection. The snapshot check was rerun
and still reports the existing stale runtime reference; no snapshot hashes were changed.

The value override profile now includes explicit retained-System native binding, native
round trips and call-name preservation; the same manual entry and XML document the
bounded contract. C# native execution uses the checked-in CoreProbe bootstrap and a
freshly assembled System bundle. The existing runtime snapshot check remains stale.

Host-only CoreObjectType, IILGenerator.Box and OpCode.Box are documented in XML and
[typed boxing](experimental-metadata.md#typed-boxing-development-2026-10-03).
They remain outside the guest RavenDoc selection; the existing snapshot regeneration
blocker is tracked above, not silently removed from API coverage.

Host-only IILGenerator.LoadFieldAddress overloads and OpCode.Ldflda are covered by XML
and [owned field addresses](experimental-metadata.md#owned-field-addresses-development-2026-10-03).
The imported-field overloads still reject Ldflda. These C# APIs remain outside guest
RavenDoc selection; the known stale runtime snapshot remains unchanged.

Host-only IILGenerator.IsNull/IsInstance, their raw opcodes, and String CastReference
support have XML and [manual reference](experimental-metadata.md#reference-tests-development-2026-10-03)
coverage. The known stale guest RavenDoc snapshot remains tracked; no snapshot mutation.

2026-10-03: bounded core Object.ToString import/call behavior is covered in the host
metadata manual and XML; no new guest API selection. Existing stale snapshot and
SourceUnionReferences.Project refresh blocker remain unchanged.


Host-only type custom-attribute model and introspection APIs are covered by XML and the
[manual reference](experimental-metadata.md#type-custom-attributes-development-2026-10-03).
These C# APIs are not guest Raven types and are excluded from RavenDoc's guest type selection.
The existing stale guest snapshot/source-union refresh blocker remains separate; this
change does not overwrite the last verified reference assembly or claim regeneration.

2026-10-03: host-only parameter-name and authored value/nested reference APIs are
covered in `experimental-metadata.md`, outside the guest RavenDoc type inventory.
The API snapshot check still reports stale guest artifacts; refreshing remains blocked
by the recorded source-union bridge binding issue. The last verified core reference
assembly has not been replaced by unverified output.

2026-10-03 source-union slice: AddInterfaceConversion now accepts top-level value
implementation sources; the host API manual records this and unchanged explicit-boxing
requirements. The guest API snapshot check remains stale for the previously recorded
bridge issue; no guest snapshot or reference assembly was overwritten.

2026-10-03: host-only FunctionTypeInfo is documented in experimental-metadata.md with
its members, resolution errors and limits. It is not a guest RavenDoc type. The API snapshot
check still reports the previously recorded stale guest snapshot; the last verified guest
reference assembly remains intact.

2026-10-03: host-only IILGenerator.UnboxAny and OpCode.UnboxAny are documented in the
experimental metadata manual and XML. No guest type selection changes are needed.
The snapshot check still reports the recorded stale guest artifacts; this slice does
not replace the verified guest snapshot with unverified output.


2026-10-03: host-only GenericParameterTypeInfo.Name is documented in
experimental-metadata.md, including native declaration identity and the CLI
NotSupportedException limitation. It is not a guest runtime API and does not require a
RavenDoc guest type selection. `build-api-docs.py --check` still reports the previously
recorded stale guest snapshot; verified guest artifacts were not overwritten.

2026-10-03: host metadata ArrayOf now admits nested vectors within its existing
16-level signature bound; manual reference and C# contracts updated. No guest API
added. The guest snapshot check still reports the previously stale snapshot; no
reference artifacts were overwritten for this host-only API adjustment.

2026-10-03: FunctionBinding/BindFunction now admit owned nongeneric nonvirtual
reference-instance targets, consuming a receiver. Host manual/XML contracts and C#
execution tests updated; no guest API added. The existing guest snapshot is still stale.


2026-10-04 Tasks/Concurrency integration: guest public signatures are unchanged;
experimental C# constructed callback binding and FunctionTypeInfo identity are documented
in `experimental-metadata.md`. `build-api-docs.py --check` still reports the existing stale
reference snapshot. It has not been relabelled as refreshed; regenerate from a matching
working bridge before publication. Website publication remains separate.


### Floating-point metadata writer checkpoint (2026-10-04)

The experimental C# metadata API adds Single/Double signatures and IL generator
literal overloads, documented in the existing manual host API reference. The existing
host-API exclusion from guest RavenDoc remains unchanged. `build-api-docs.py --check`
still reports the previously recorded stale guest snapshot; this slice neither changes
guest APIs nor refreshes them from an unmatched bridge. No website build was run.


### Static interface metadata checkpoint (2026-10-04)

The manual experimental host reference covers the static AddInterfaceMethod overload,
detached static contract flags, completed external contracts and inherited Self views.
The 136 C# metadata groups pass, including CLI constrained static dispatch and native
round trips. Guest API declarations did not change. The required snapshot check still
reports the existing stale guest snapshot; no mismatched bridge refresh was performed.


### Generic Number metadata integration (2026-10-04)

The experimental C# host manual covers external constrained IL-generator overloads,
caller-bound forwarding validation and explicit imported primitive designation. These
host APIs remain excluded from guest RavenDoc as documented above; no guest public
signature changed. The existing guest snapshot check still reports stale input. Do not
refresh or publish it using an unmatched bridge. No website build was needed.


Development validation (2026-10-04, source-owned String): the snapshot check still reports
stale cached input. This slice implements the already documented String op_Equality and
op_Inequality in Raven source; both signatures and XML entries already exist in the bridge
reference. No new guest API is omitted. The native gate uses TextCore1004 plus native Numbers,
not the full documentation bridge. Refreshing the full legacy snapshot from a matching
bridge remains open; do not substitute this subset's assembly as the documentation core.

Development validation (2026-10-04): the source-owned Char/String gate implements the
already documented String(Sequence<char>) constructor directly in Raven source. No new
guest signature is added to the bridge reference. `build-api-docs.py --check` still reports
the previously stale full snapshot; do not replace it with the subset native Numbers.dll.
Host metadata storage/constructor contracts are documented in experimental-metadata.md.

### Native class-base reader checkpoint (2026-10-04)

The .NET-host reader's local nongeneric base relationships and standalone assembly
input are documented in [the manual metadata API reference](experimental-metadata.md#native-class-base-reader-views-development-2026-10-04),
including NominalTypeInfo.BaseType, TypeDefinition.BaseType and rejection contracts.
Host C# APIs remain outside the guest RavenDoc selection. The required snapshot check
still reports the pre-existing stale guest snapshot; no mismatched bridge refresh or
website build was performed.

### Local class-base authoring (2026-10-04)

The host-only AddClass base overload, definition parity and IL-generator constructor
calls are covered by the [manual reference](experimental-metadata.md#local-class-base-authoring-development-2026-10-04)
and C# executable tests. They remain outside guest RavenDoc selection. The existing
stale guest snapshot is unchanged; it is not replaced by a partial library snapshot.

Runtime protected constructor and closed-class validation changes are documented in
[the experimental reference](experimental-metadata.md#runtime-protected-constructors-and-closed-class-families-2026-10-04).
Constructor authoring now includes C# MethodVisibility.Protected, covered by that manual
reference and executable C# tests. No guest API signatures change; the existing guest snapshot
is unchanged and its previously recorded refresh blocker remains open.

Closed-class host authoring, introspection classification/direct children and external
reference-base contracts are covered by the experimental manual reference and 146 C#
contract groups. These host-only additions do not alter guest signatures or refresh
the pre-existing stale guest reference snapshot.


The development host-only MethodDefinition/MethodBuilder.SetInternalCall APIs are
covered by the [manual metadata reference](experimental-metadata.md#runtime-internal-calls-development-2026-10-05).
They remain under the existing C# host-library RavenDoc exclusion; the guest reference
assembly/snapshot is unchanged. Native runtime-service authoring is not a new guest API.
The 2026-10-05 `scripts/build-api-docs.py --check` run still reports the recorded
stale guest snapshot; no partial or mismatched reference refresh was performed.

The 2026-10-05 native service-source gate adds only internal Raven runtime declarations.
The temporary CLI bootstrap's MethodImplAttribute/MethodImplOptions are compiler-facing
markers consumed into implementation flags, not new executable guest APIs. They are
excluded from RavenDoc with the other bootstrap-only declarations; see the
[source-service contract](../docs/experiments/extended-cli-metadata/source-internal-calls-2026-10-05.md).

2026-10-05 final-class/Boolean host metadata update: SetSealedClass and canonical
Boolean storage are covered in the manual experimental metadata reference. These
C# host APIs are outside the guest RavenDoc selection; new runtime snapshot/service
adapters are internal. The API snapshot check was run and still reports the previously
recorded stale guest snapshot. Do not replace that full reference with the incremental
JsonIntrospection library.

2026-10-05 cumulative-library row-budget update: the host manual reference and XML
now record 4,095 authored/native declarations within the existing 4,096 CLI TypeDef-row
budget. Boundary tests cover manual/builder attachment and CLI/native/facade round trips.
No guest public signature or RavenDoc selection changes. The full guest snapshot remains
the previously recorded stale input; a subset library is not an appropriate replacement.

2026-10-05: Host metadata generic FunctionBinding/ILGenerator overloads are documented
in experimental-metadata.md. These C# APIs are outside the guest RavenDoc selection.

2026-10-05: Optional String[] entry-point behavior and host argv semantics are covered
in experimental-metadata.md. These host APIs are outside the guest RavenDoc selection.
The API snapshot check still reports the pre-existing stale guest snapshot; this change
does not replace it with a subset assembly.

Development validation (2026-10-05, source-owned RuntimeTypeHandle): the host metadata
primitive designation now accepts the fieldless handle declaration. XML and the linked
experimental metadata manual document it; no guest API signature or RavenDoc selection
changes. The required snapshot check still reports the previously recorded stale guest
snapshot. It was not replaced with a partial library snapshot.

2026-10-05: `LoadedProgram::with_modules_and_object_root` and
`assembler::read_modules_with_object_root` are covered by
[runtime hosting](runtime-hosting.md), linked from the API landing page. These are
Rust host APIs and cannot be extracted from the guest CLI reference by RavenDoc;
no guest type is excluded or added to the type selection. The API snapshot check
still reports the pre-existing stale guest reference; no partial replacement was made.


### Release preparation snapshot (2026-10-05)

The current snapshot was regenerated from the repository bridge using Raven main
`08f34891b` (.NET 11), not copied from an older reference. It includes the existing
`System.ParamArrayAttribute` compiler marker with type/constructor documentation and
corrects the manual assembly-level-function link to `System.Fail`. The bridge now names the
source owner in projection errors. Source fingerprints and the assembly checksum are
recorded in `snapshot.json`.

The same bridge built against native integration `9a4f74884` currently fails reference
regeneration in `System.Option<T>` with RAV0103 (`None` is not in scope). This is an open
compiler-line compatibility regression; main succeeds. The website's CLI reference
producer uses the validated main compiler. Native source-library/sample evidence is a
separate workflow and does not prove this bridge regeneration works on that branch.
Keep this regression visible for integration repair before release qualification.

Validation: the complete website builds and checks 1,803 pages and local links; all
18 website publisher tests pass. The homepage and native-target page were inspected
in the browser. Existing missing-summary reports remain documentation coverage debt;
this refresh does not claim they are all resolved. No publication was performed.


The reference-producer compatibility regression above is resolved by Raven integration
`2b683df67` (general fix on main `edff20273`). The rebuilt current bridge successfully
projects unchanged Option source and generates a reference byte-for-byte identical to
the snapshot. No snapshot refresh or new public API is needed. See the
[dual-target execution evidence](../docs/experiments/extended-cli-metadata/union-lexical-cases-2026-10-05.md).

Host native-width integer signatures and Conv_I/Conv_U are documented in the
[manual metadata reference](experimental-metadata.md#native-width-integers-development-2026-10-06).
They are host-only API additions; the guest reference snapshot is unchanged.

### Source Attribute base (2026-10-07)

The existing System.Attribute API page now describes its source-built abstract base
and protected constructor. The matching reference projection and XML sidecar preserve
that shape; the [reference-support guide](reference-support.md#source-attribute-base-development)
distinguishes it from bootstrap-only attribute scaffolds and unimplemented .NET helpers.

The source-unit bootstrap gate (2026-10-07) adds no guest API signatures. Existing
System.Void and NativeMemory sources compile with explicit native unit ownership;
inhabited unit parameters and no-result calls remain distinct. The integration guide
records the source/native owner configuration and current full-bootstrap limitation.

The 2026-10-07 `NativeReflection` facade is internal native build infrastructure,
not a new public reflection API. The public parameterless CreateInstance signature
is unchanged. Its two public declarations in the temporary bootstrap are translation
scaffolding, like RuntimeServices, and are intentionally outside RavenDoc selection.
Source constructor execution and access checks are documented in the
[construction gate](../docs/experiments/extended-cli-metadata/reflection-construction-2026-10-07.md).

Development 2026-10-07: ArrayReflection is included in the generated reference and XML
member documentation, with source-backed GetLength/GetValue/Create and vector/Fault
limits in [the reflection guide](reflection.md). The reference bridge supplies only
matching declarations; native acceptance compiles the actual Raven implementations.


### Split native bundle IDE documentation (2026-10-07)

The source-built Runtime/Data/Networking/Web bundle now carries each build's XML and
Markdown sidecars next to its native assembly. The manifest hashes every nested file
and associates the sidecars with the declaring assembly. Relocation and editor tests
check imported Networking IPAddress help. This uses native symbols, not the aggregate
CLI documentation projection, and changes no public API signatures.

Coverage is incomplete: the current generated XML contains 136 Runtime, zero Data,
41 Networking and 136 Web member entries (including synthesized declarations; these
are not counts of fully documented public APIs). Authored website XML remains a
separate source; this slice does not partition/merge it into native outputs. Unifying
that content by canonical declaration identity and producing one RavenDoc model from
the split inputs remains required. Do not advertise complete class-library IDE help.
[Commands, artifacts and validation](../docs/experiments/extended-cli-metadata/native-bundle-documentation-2026-10-07.md).


### Native provider direction (2026-10-07)

The author explicitly reaffirmed that CoreProbe is not an acceptable production owner
label. RavenDoc's current loader imports CLI references; its renderer already accepts
compiler symbols from multiple assemblies. The next native provider should use the
explicit NeoCLR catalog and current compiler-symbol adapter, preserving real declaring
assembly identities. Extract a stable documentation model incrementally rather than
build another resolver or merely replace the visible assembly name.
[Inspected boundary and acceptance](../docs/experiments/extended-cli-metadata/native-documentation-provider-2026-10-07.md).


### Release scope correction (2026-10-07)

The author subsequently deferred the native RavenDoc provider/model migration and
CoreProbe assembly-label correction, while considering a future Raven rewrite of
RavenDoc. The earlier provider plan remains recorded for future use, not as an active
release gate. Keep the existing bridge identity and limitations explicit; do not invent
production assembly ownership in the current reference. Normal accuracy/coverage upkeep
and the already shipped IDE sidecars remain useful; renderer redesign is deferred.

The protected `System.Attribute()` constructor is documented in the
[manual introspection reference](introspection.md#attribute-constructor), with a route in
`manual-members.json`: the pinned RavenDoc renderer omits protected constructors.

### Shared host fault diagnostics (2026-10-07)

FaultDiagnostic, Fault::diagnostic and FaultCode::standard_message are Rust host APIs,
covered in [the manual fault reference](faults.md); they are not Raven class-library
types and do not enter the RavenDoc type selection. The guest reference assembly and
snapshot remain unchanged. The snapshot check passes for this slice.

Development 2026-10-08: StringBuilder and String.Join(string, string[]) are included
in the aggregate reference, complete type inventory and authored XML. The builder
uses explicit UTF-8 quota units and fluent faulting append; Join preserves empty
elements and rejects null inputs. Native class-library ownership is System.Runtime;
the documentation assembly remains the temporary CLI aggregate described above.

### Unprefixed async interfaces (2026-10-08)

`AsyncStateMachine` and `TaskAwaiter` replace the prefixed development protocol
names. The selected type inventory, authored XML, matching bridge reference and
snapshot were refreshed together; the API snapshot check passes. Recompile old
consumers with the matching runtime/compiler bundle. The wider legacy runtime
snapshot check still reports a pre-existing missing StringBuilder input in Tuple;
the regenerated Tasks slice's own input/output fingerprints pass. No website build
or publication is claimed. The following union slice replaces the legacy `IUnion` protocol with `UnionValue`.

### Union bridge refresh (2026-10-08)

The reference now uses `System.Runtime.CompilerServices.UnionValue`. Its existing
boxed Value getter is unchanged. All 27 generated union slices were regenerated
with the current compiler, including explicit constructor field initialization.
Their `TryGetValue` failure paths now clear outputs to defaults, matching Raven's
shared body contract and native emission; older bridge fragments preserved them.
The XML reference and executable contract sample now describe/test clearing.
Do not rely on a previous output after a false result. Matched references, compiler,
application and runtime fragments must be rebuilt together.

Raven integration commit `be58723fb` completes the protocol mappings and keeps
async interface signatures in target metadata for CLI emission. Sixteen focused
profile/emission/.NET controls pass; the legacy async sample prints `Suspended`
and `42`. Native-enabled compiler rebuild succeeds. This is tested on
`codex/source-object-metadata-resolution`; the general cache fix is independently
on Raven main as `d0a115dcf`. No bridge-free bootstrap or general native async
entry-draining support is claimed.

### Math assembly-level constants (2026-10-08 development)

Pi, E and Tau are literal Double members on the already selected System.Math namespace
container. Their XML descriptions and matching reference/snapshot are refreshed together.
The native library stores them directly as namespace metadata; the CLI carrier here is
only the documentation bridge. The new host AssemblyConstantDefinition,
AssemblyBuilder.AddConstant and ModuleDefinition.Constants APIs are
covered in [the manual metadata reference](experimental-metadata.md#native-assembly-level-double-constants-development-2026-10-08), not silently omitted guest APIs.

Assembly-level member naming/ownership (2026-10-08): new host AssemblyMemberKind,
AssemblyMemberDefinition, AssemblyMemberInfo, both GetMembers methods and constant
ownership/FullName properties have [manual API coverage](experimental-metadata.md#assembly-level-member-model-development-2026-10-08).
They are host metadata APIs, outside the guest RavenDoc assembly. The renamed constant
APIs and assembly manifest key require rebuilding earlier same-day prototype artifacts.


### RavenDoc consequence of native-only core work (2026-10-08)

The author notes that retiring the importer should also remove Probe artifact names
from RavenDoc. The temporary CLI projection/import path is the part to retire; native
metadata reading and symbol resolution remain necessary. `Probe.dll` is generator
tooling, while `NeoCLR.CoreProbe.dll` is the aggregate documentation reference whose
identity currently appears in generated ownership. The inspected loader still builds
portable references and adds framework references; the renderer uses declaring symbols.

The new compiler API native-only metadata mode is a prerequisite, not an automatic
RavenDoc migration. Reuse the [native-provider plan](../docs/experiments/extended-cli-metadata/native-documentation-provider-2026-10-07.md)
with actual native core/library inputs and canonical documentation identity. Preserve
route/coverage checks and ordinary .NET documentation as controls. The earlier explicit
release deferral remains recorded; this follow-up confirms the desired outcome without
selecting a renderer rewrite or silently establishing a new release gate. Existing
reference snapshots and truthful owner labels remain until the input migration works.


### Full-bundle loader admission (2026-10-08)

The existing split bundle `native-math-bundle1008b` still selects `Core.dll` through
RavenNeoClrCoreReference. Feeding that core to the explicit native-only RavenDoc loader
rejects because it has no native payload, before any site is written. This is a producer/
bundle migration requirement, not permission to silently use a CLI projection or remove
public APIs. [Exact inputs and result](../docs/experiments/native-core-bootstrap/ravendoc-bundle-audit.json).
The separate native-core fixture already qualifies the loader and renderer; it cannot
stand in for production Runtime/Data/Networking/Web coverage. Next produce a matching
native core and rebuild those libraries, then compare public types/members, documentation
and routes before switching website inputs.


### Declaring-library migration audit (2026-10-09)

Actual split native libraries render 1,632 pages with System.Runtime.dll ownership
on BooleanParseError and CancellationTokenSource and no CoreProbe labels. Raven
needed assembly-level function/constant documentation ID and comment fixes.
[Audit and evidence](../docs/experiments/native-library-documentation-audit/README.md).
The historical bundle explicitly uses a CLI primitive bootstrap. Keep the complete
current snapshot until semantic member coverage, comments and routes are compared;
384 versus 346 projected type names alone cannot establish lost APIs or equivalence.


### Integrated development preview (2026-10-09)

The audit's themed mode now integrates native pages at normal `/docs/api/` routes
in the local website. It restores reviewed XML by exact native member IDs and records
all mappings. Unmatched legacy pages retain truthful owners and a migration-gap
notice (136 pages after rebuilding the bundle; Math constants are now native).
Publishing remains unchanged; regenerate a matching production bundle, sidecars and
full coverage inventory before the upcoming release. See the audit README for the
explicit local integration command and its complete HTML/link checks.

### Release website input (2026-10-09)

Clean website builds now select `native-reference.json` and its checksum-pinned
`native-reference.zip` before publication. The archive contains reviewed rendered
native API pages, not executable libraries. Its manifest records library artifact
hashes, the source/compiler revisions and the renderer revision. This bounded
snapshot avoids silently reverting published pages to CoreProbe when a local
preview-selection file is absent. A corrupt snapshot fails before publication.

Refresh it after rebuilding the four-library bundle and generating a fresh themed
audit as documented in `docs/experiments/native-library-documentation-audit/README.md`:

```sh
python3 scripts/update-native-api-snapshot.py /absolute/path/to/audit \
  --renderer-revision EXACT_RAVEN_REVISION
python3 -m unittest discover -s scripts -p test_native_api_snapshot.py
python3 -m unittest discover -s scripts -p test_build_website.py
python3 scripts/build-website.py
```

For publication validation, use a clean checkout without the ignored
`target/native-api-preview.json` override. Review the resulting website in a browser
before committing the snapshot. Rendering still uses the explicit temporary
primitive bootstrap and reviewed XML restoration described in the audit; this is
not native-only core bootstrap. The 136 unmatched legacy pages remain visibly
marked migration gaps. Keep the existing reference metadata and XML snapshot for
that compatibility coverage. Regenerate the rendering when the release links,
RavenDoc presentation, native APIs or authored documentation change.

### System.Time identity migration (2026-10-09)

Date/time values, calendars, clocks, zones and their errors now appear under
`System.Time`, including `TimeOfDay` (formerly the System.Time struct). The
reference producer, selected type inventory, XML member IDs and native library
ownership are updated together. LocalDateTime.Time returns TimeOfDay.
Formatting types stay under System.Globalization. Recompile previous consumers;
there is no same-name compatibility type at the new namespace root.

The local integrated native preview includes the renamed family with actual
System.Runtime.dll owners, summaries, generic interfaces and member links. The
normal publishing path still uses the checked bridge snapshot until the wider
native documentation migration is qualified. See the explicit native audit preview
procedure above; no deployment is implied by local website validation.

2026-10-09: logical declaration-module host APIs are covered in the
[manual metadata reference](experimental-metadata.md#declaration-modules-development-2026-10-09).
They do not change guest ModuleInfo or require a new guest reference snapshot.

2026-10-10: host logical `ModuleInfo`, assembly/context `GetModules()`,
`Resolve(DeclarationModuleDefinition)`, member `Module` and type/method
`MetadataScopeName` have XML summaries and complete
[manual reference coverage](experimental-metadata.md#context-owned-declaration-views-development-2026-10-10).
The temporary DeclarationModuleInfo facade is removed. These C# host APIs remain
outside the guest RavenDoc selection; the guest reference and signature snapshot
are unchanged pending their separate migration.


### String replacement (2026-10-09)

Development String.Replace(String,String) is included in the existing String type
selection, with argument, result, fault and ordinal UTF-8 documentation. The matching
bridge reference and source snapshot are refreshed. It requires non-null arguments;
empty replacement deletes and empty search faults. Native fixture union escaping is
qualified; complete native source-core bootstrap remains unfinished.

2026-10-09: host-only ValueIs/ValueUnpack metadata operands are documented in
[the metadata reference](experimental-metadata.md#native-erased-result-operands-development-2026-10-09); guest API signatures are unchanged.

2026-10-09: host primitive bootstrap references are documented in the
[manual metadata reference](experimental-metadata.md#primitive-bootstrap-references-development-2026-10-09).
This host authoring/reading API does not change guest String signatures or snapshots.

### Module introductions (2026-10-09)

All 24 modules in the current native API navigation have authored introductions in
`website/api-content`. RavenDoc attaches them using `N:` documentation IDs for both
module and legacy namespace projections. They explain existing capabilities and
link to the relevant guides; they do not fill the separately recorded type/member
summary gaps or qualify additional execution modes. Refresh the native rendered
snapshot whenever this authored content changes.

The legacy reference build stages four function-only module introductions against
its static carrier types (Environment, FileText, Metadata and NativeMemory). Native
rendering retains the original `N:` IDs and module pages. This adaptation belongs
to the temporary website bridge, not the native documentation contract.

### Public extension visibility regression (2026-10-09)

Receiver lists must exclude public methods declared by internal extension types.
The production native audit checks that Boolean does not acquire ConsoleFlush or
ConsoleWriteBytes. Shared RavenDoc regression coverage exercises source and .NET
metadata, preserving genuine public extensions and excluding internal generic
extensions as well. See the native reader limitation in
[the integration contract](../docs/raven-cli-bridge.md#public-documentation-and-native-extension-visibility-2026-10-09).

### Completed task factories (2026-10-09)

`System.Tasks.Task.CompletedTask` and `Task.FromResult<T>(value)` are development
APIs in System.Runtime. Their public property/method pages are rendered in the
native snapshot, with matching XML help and legacy bridge reference. The temporary
CLI unit projection now covers property signatures as well as accessors; no public
API is excluded. See [the contract](../docs/task-completion-factories.md). Published
Preview 13 libraries must be rebuilt before compiling consumers of these helpers.

### Embedded JSON nodes (2026-10-09)

The existing JsonSerializer overloads now support explicit JsonValue/concrete node
declarations at roots, model properties and vector elements. Type/member XML and
the JSON guide describe kind validation, JsonNull, Object non-inference and shared
document limits. The matching bridge reference was regenerated (no public signature
change), checked, and the native rendered snapshot refreshed from the rebuilt
libraries with RavenDoc 6c90bf2c. No new API is excluded.

### Basic collections (development, 2026-10-10)

Queue/ArrayQueue, Stack/ArrayStack and Set/MutableSet/HashSet are selected in the
reference with member XML and a matching native snapshot. Removal/peek Option
results, bool membership-change results, explicit comparers, snapshot iteration and
unsynchronized behavior are documented. No public type is excluded for this slice.
Map now inherits Iterable of KeyValuePair, with its public constructor,
getters/init accessors and Deconstruct selected and documented. HashMap.GetIterator documents
snapshot cost, lifetime and concurrency limits. The native positional-record storage
restriction is explicit; equality/hash/display helpers are not native APIs.
The matching compiler is required to rebuild the native library snapshot.

### Native init accessor metadata (development, 2026-10-10)

The [metadata manual](experimental-metadata.md#init-accessors-development-2026-10-10)
covers IsInitOnly definition/introspection queries and optional authoring arguments.
KeyValuePair Key/Value reference signatures now expose init accessors; native and
CLI snapshots must use the matching init-aware compiler. No new runtime reflection
query was added; its ordinary getter/setter invocation policy remains unchanged.

## Iterable map materialization (development, 2026-10-10)

HashMap's iterable constructors and pair/selector ToMap overloads have generated
native type/member pages, XML parameter/result/error documentation and matching
reference declarations. MapOperators is included in the public type inventory.
These additions target the native metadata library; the historical bounded CLI
application importer is not extended with these new pair-materialization calls.

The follow-up iterable constructors for ArrayList, ArrayQueue, ArrayStack and HashSet
are covered by their existing type selections, matching reference declarations,
source XML parameter/ordering/error contracts and refreshed native member pages.


### Scoped iteration implementation refresh (2026-10-10)

The runtime source audit changes implementation bodies only; public signatures
and XML contracts are unchanged, so the existing compiler reference assembly is
reused with refreshed source fingerprints. The native documentation snapshot is
rendered from the matching source libraries and compiler/renderer 494dede84.
No website build or publication is implied.


General host metadata attributes now include callables, fields, properties and
parameters; see the [manual host reference](experimental-metadata.md#member-custom-attributes-development-2026-10-10).
Guest reference signatures and snapshots are unchanged. AttributeUsage, Raven source
emission and guest discovery remain tracked in the [attribute plan](../docs/custom-attributes.md).


### Named attribute guest data (development 2026-10-10)

CustomAttributeNamedArgument and CustomAttributeData.GetNamedArguments have matching
XML, automatic RavenDoc type selection and a refreshed reference snapshot. The
[introspection reference](introspection.md#named-attribute-arguments-development-2026-10-10)
covers every new member and the MemberInfo/wider-value gaps. Source-built native
libraries provide the same API under interpreter and AOT. Regenerating the separate
legacy CLI Descriptors implementation slice currently fails because its project
omits ArrayReflection; its old fixed-only snapshot remains unchanged. This is not
an API reference refresh failure or evidence for named-data support in that legacy
snapshot. The runtime rejects named data with that older library explicitly.


### Runtime usage policies (development 2026-10-10)

AttributeTargets now documents every .NET flag value; AttributeUsageAttribute includes
ValidOn, constructor defaults and mutable options in the matching reference snapshot.
These declarations now have executable source-runtime implementations. Inherited
queries and some annotation targets remain gaps; enum membership is not a support
claim. Primitive bootstrap pruning is separate from this complete API reference.

### Test framework source surface (2026-10-10)

`NeoClr.Testing.TestAttribute` belongs to the source-included test framework, not
System.Runtime/CoreProbe. It is therefore outside the guest RavenDoc type selection;
[the linked manual reference](testing.md) covers both constructors, Description,
discovery errors/limits, and the existing assembly-internal runner contract.
No guest reference assembly/XML changed. Packaging the framework will require its
own reference assembly/XML and generated navigation; this is an explicit coverage
boundary rather than a silently excluded runtime API.


### Guest logical module migration (2026-10-10)

ModuleInfo remains selected in RavenDoc; its physical MetadataToken property is
removed from source, bridge and XML, and the matching reference snapshot is refreshed.
Assembly/module/member ownership now uses logical names. The [introspection guide](introspection.md#logical-modules-development-2026-10-10)
documents interpreter traversal, bounded AOT retention and the unstable backend recipe
helper. Website compilation is not required for this signature removal.

The independent legacy CLI implementation regeneration was attempted with `--slice
ModuleInfo` and stops at the existing `Incompatible map contract: Map` check. The
checked ModuleInfo fragments received only a mechanical deletion of the removed getter
and property; their manifest records that maintenance and preserves prior compiler/core
provenance. This is not a successful clean legacy regeneration. Source-built native
Runtime/Data/Networking/Web libraries and the reference-only bridge rebuilt successfully.


Development (2026-10-10) adds ModuleInfo.GetFunctions, already within RavenDoc's
selected type set, and method-level attribute inspection for ownerless functions.
Matching XML, reference signatures and source snapshot are refreshed. The manual
introspection guide documents interpreter-only support, open generic and parameter
attribute gaps, and absence of native retention/invocation. Legacy CLI implementation
regeneration remains separately blocked by the existing Map contract issue.

### Map indexers (development, 2026-10-10)

Map declares a getter indexer, MutableMap adds a setter, and HashMap implements both.
Missing-key reads terminate with Fault; Find remains the safe alternative. Setters
insert or replace using the existing comparer and preserve the stored key. Existing
implementations must add the accessors. All three existing selected public types
have Item property XML summaries, parameter/value descriptions and native source
comments. Refresh both the aggregate reference and native snapshot with this API.
No public type exclusion or renderer workaround is needed.
