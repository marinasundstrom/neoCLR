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
