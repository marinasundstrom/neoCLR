# API reference maintenance

RavenDoc publishes this reference inside the single neoCLR website at `/docs/`.
It reads a checked-in compiler reference assembly and the authored XML sidecar,
then renders Raven signatures with the same layout, navigation and development
notice as the Markdown guides. No DocFX build, metadata YAML or second site exists.

The site documents Preview 11; proposals remain labeled separately.
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

Development adds the lazy `System.Linq.Operators.OfType<T, U>` extension, written
`source.OfType<U>()`. Its XML member entry and matching reference document filtering,
ordering, disposal and the existing Object conversion limits. Function syntax in
Raven library callbacks remains backed by main's nominal Func metadata contracts.

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
the two namespace-function containers and IsReadOnlyAttribute. These remain part of
the public inventory, with linked manual entries and checked routes. No public type
is excluded from the inventory. `exclusions.json` records intentional page exclusions
with reasons: non-generic Array, Option, Result and TaskOutcome are importer/exporter
scaffolds, not additional application types. Their generic APIs remain documented;
CLR case-carrier types inside the non-generic containers are also explicitly
excluded. Coverage checks honor these exact type exclusions.
Keep [reference support](reference-support.md) aligned
when those renderer limitations change.

RavenDoc now renders the previously excluded TaskQueue Post/Run, Task OnCompleted,
ITaskAwaiter OnCompleted, FileText WriteAllText and OutputStream/FileOutputStream/
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
three explicit manual entries and ten explicitly excluded metadata scaffolds. Collections, arrays, delegates, query operators,
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
HasValue, Value and conditional TryGetValue members are documented. IUnion is a
provisional ordinary compiler-support interface. RavenUnionCaseAttribute is excluded
as compiler-reference metadata, with the exact reason in exclusions.json; it is not
an executable runtime API. Rebuild the bridge after changing the embedded source.

DnsError and UriError also use embedded-source projection. Their generated HasValue,
Value and conditional TryGetValue members replace handwritten per-case Is*/Get*
helpers. Their default values are inactive. The core projection runs sequentially
so every family uses the same supplied IUnion identity.

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

## Native Self integration

The development reference exposes nongeneric System.Number and System.Clonable,
using the fieldless Self transport marker for implementing-type signatures.
These APIs require matching Raven neoCLR target settings and the updated runtime.
Structural Function metadata remains excluded from main. The snapshot is rebuilt
from the nominal main-based bridge, not copied from the Function feature branch.

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
`MethodBuilder`, `LocalDefinition`, `BranchLabel` and `OpCode` in that Model namespace,
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
