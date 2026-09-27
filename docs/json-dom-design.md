# JSON DOM — development, 2026-09-25

The first System.Data.Json milestone is a DOM, before reflective object mapping.
The author selected a **closed JsonValue class hierarchy with kind-specific APIs**
after comparing it with a six-case union. Earlier application-local trees were
exploratory evidence, not a preselected public contract.

## Selected shape

JsonValue is abstract and closed to JsonObject, JsonArray, JsonString, JsonNumber,
JsonBoolean and JsonNull. Containers are mutable: object Add(name, value) rejects
rather than replaces a duplicate; array Add(value) appends. Field, Item, Name and
Count belong to the appropriate container. Scalar properties expose their actual
types. JsonNumber.Parse validates a JSON token and Text preserves its exact spelling;
AsInt32 accepts only checked integer-token conversion. No implicit floating-point
rounding or coercion from exponent/fraction tokens is performed.

A present null is a JsonNull node. MissingField is a distinct error. Nodes have ordinary
reference identity, not deep value equality. Containers retain child references, so
sharing and cycles are possible; writing a cycle reaches the depth bound. There is
no cloning, ownership tree, parent pointer, replacement/removal or thread-safety API.

JsonSerializer retains its JsonValue DOM overloads. Provisional Object/TypeInfo
overloads now add shallow mapping as described below. String and stream Deserialize
parse one complete value; Serialize writes compact JSON. Streams are borrowed, not
flushed or closed. StreamReader/StreamWriter provide strict UTF-8 and partial-transfer
handling. The author explicitly selected synchronous Result-returning APIs for this slice.
Asynchronous reads remain a possible later track. This is whole-buffer I/O, not
incremental or cancellable I/O.
A failing read may consume input; a failing write may leave a prefix. Validation of
the complete output occurs before touching the destination.

The inherited experiment bounds remain explicit: 128 UTF-8 bytes, four nested
containers, 32 value occurrences and 31 children per container. Revisit these small
POC limits against the release app; they are not general JSON or HTTP limits.
Duplicate names compare ordinally after unescaping. Reject trailing values/commas,
comments, BOMs, malformed numbers and unpaired surrogates. JSON grammar acceptance
is separate from numeric conversion.

JsonError uses standard Raven union syntax. Syntax and LimitExceeded carry diagnostic
text; MissingField, DuplicateField and InvalidIndex retain their arguments. InvalidNumber
and NumberOutOfRange distinguish token validation and integer conversion. Read and
Write retain TextReadError/StreamError. A stream size failure therefore remains a Read
cause while a string size failure is a JSON LimitExceeded. TypeMismatch lets a DOM
consumer report an expected kind when pattern matching fails. No stable diagnostic
strings or source-offset tracking are promised in this iteration.

## Comparison and decision

Primary sources checked 2026-09-25:

- [.NET DOM documentation](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/use-dom)
  distinguishes a read-only JsonDocument/JsonElement model from mutable JsonNode,
  JsonObject and JsonArray. Our kind-specific reference types are closest to the
  mutable model. Deliberate naming difference: neoCLR JsonValue is the root of all
  kinds; .NET JsonValue represents scalar values under JsonNode.
- [Serde JSON Value](https://docs.rs/serde_json/latest/serde_json/enum.Value.html)
  demonstrates a six-variant enum with typed payloads, including mutable containers.
  A union does not imply immutability. A union wrapper could retain mutable object/
  array references; it would add case construction around those container APIs.
- The earlier [document comparison](experiments/json-document/README.md#comparison-and-choice)
  records RFC 8259, .NET and Json.NET duplicate-name/number policies. Reuse that
  evidence and the .NET 10 corpus rather than inferring a new conformance claim.

Both a union and a closed hierarchy can support exhaustive matching; this decision
is about the public shape, identity and kind-specific operations, not claiming that
only unions can be exhaustive. The hierarchy avoids exposing array/object mutation
on scalars, but introduces separate reference objects and checked casts/patterns.
A union would make the set of cases particularly concise, at the cost of a different
payload/copying model. Neither has a demonstrated performance advantage here.

Keep the closed marker and permitted derived types consistent across reference
metadata and managed implementation. Reject external subclasses and forged method
signatures at the importer boundary. Internal parser/codec helpers are not public
capabilities. Standard union projection is reused for JsonError; no new runtime
instruction, compiler policy or runtime Raven-metadata dependency is added.

## Validation scope

The [public DOM fixture](experiments/json-dom/README.md) consumes only the matching
library/reference, not copied parser sources. It covers kind-specific construction,
structured and nested causes, borrowed streams, memory/short-transfer round trips,
limits, duplicate decoded names, and managed GC. A separate public-API corpus replays
54 valid/invalid documents and 12 integer conversions against .NET 10 and Python.
Signature checks cover public members, private helpers, upcast direction and closed
hierarchy enforcement. API reference coverage accompanies this development surface.
Website build and publication remain separate; this slice does not ship a release.

## Next investigation: one mapped report — 2026-09-25

After the [public DOM Web sample](experiments/http-json/README.md), the smallest
candidate is a class with a public parameterless constructor and one writable
string property representing the station. Map that object to the existing DOM and
back, initially outside HTTP, then reuse the same endpoint. This is an investigation
scope, not an implemented serializer overload or committed public signature.

Inspection of `System/Introspection/Descriptors.rvn` confirms that PropertyInfo
already exposes its type, read/write flags, index parameters and getter/setter
metadata. FieldInfo supplies type, visibility and definition index. `src/reflection.rs`
only performs metadata queries; none of those member descriptors currently executes
accessors, writes values or constructs an instance. Runtime-backed execution therefore
needs a separate bridge/VM path, not merely JSON code calling an existing SetValue.

The .NET baseline is [PropertyInfo.SetValue](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.propertyinfo.setvalue)
and [Activator.CreateInstance](https://learn.microsoft.com/en-us/dotnet/api/system.activator.createinstance?view=net-10.0)
(reviewed 2026-09-25). Preserve familiar creation/access concepts while placing
operations in System.Runtime.Reflection extensions over introspection. First prove
public parameterless class construction and a public, non-indexed property accessor.
A property must execute its setter, not bypass it by writing its backing field.
Field access is a separate candidate, not required to demonstrate this first mapping.

Before exposing the operations, validate that the descriptor belongs to the loaded
runtime type and module; an index alone is not authority. Check receiver type,
visibility, member shape and value assignability. Metadata-only or unsupported
shapes need explicit Result failures; retain terminal Fault semantics for faults
raised by executed user code. Validate receiver/value rooting through allocation
and accessor calls, plus rejection of wrong receivers, read-only properties and
foreign descriptors. Keep binder coercions, private access, indexers, value-type
mutation, arbitrary method invocation and constructor overload selection outside
the first case. Exact error cases and public names remain open.

Hand-written DOM mapping remains the functioning baseline and is easier to bound.
Reflection reduces per-model mapping code but adds execution, lifetime and metadata
validation costs. Do not introduce serializer caches, attributes, automatic null
mapping or a new metadata convention until this single round trip establishes the
needed contract. The larger report's transport/performance limitation remains open
independently of this reflection investigation.


The first [private construction checkpoint](reflection-execution.md) now executes
public parameterless nongeneric class constructors through the normal interpreter.
The public extensions now expose checked creation and property execution; the mapped report remains next.
Checked instance property execution now preserves accessor code and virtual dispatch,
with exact scalar boxing and reference/null support. The Result-based reflection
facade and [compiled consumer](experiments/reflection-execution/README.md) are in place.


## Mapped report checkpoint — 2026-09-25

The [mapping experiment](experiments/json-object-mapping/README.md) now constructs
StationReport with CreateInstance, reads Station with GetValue and assigns it with
SetValue. A setter counter verifies actual accessor execution. The string and
MemoryStream round trips pass with zero final live objects. JSON/reflection causes
propagate into an application-owned union. An opt-in HTTP variant shares this mapper
and unchanged wire schema; independent peers and the isolated managed pair pass.
An earlier overlapping run hit the existing transport deadline; predictable latency
under load remains unverified. The default demo remains DOM-based.

This is explicit schema mapping implemented by the application, not automatic
JsonSerializer object serialization. The public serializer remains DOM-only. The
next investigation is the mapped HTTP request-path cost, followed by extracting
only a reusable mapping contract justified by the sample; null policy, recursion, naming rules, attributes and supported property
kinds are not silently decided by this checkpoint.


## Future HTTP JSON extensions — 2026-09-25

Author clarification: the JSON extension story covers the full exchange, not only
HttpClient verb methods. These are future capabilities, not implemented signatures.

| Boundary | Intended operation |
| --- | --- |
| Client sends a request | Serialize an object into JSON request content; PostJson and similar verb helpers compose this with sending. |
| Server receives a request | Deserialize HttpRequest content directly into the requested model. |
| Server sends a response | Serialize a model into response content, with response/context conveniences. |
| Client receives a response | Deserialize response content, including responses obtained from Send; GetJson can compose GET with this read. |

Assistant-proposed layering: share serializer/content conversion across requests
and responses, then add HttpClient verb and request/response/context conveniences.
Keep the same mapping rules and structured JSON causes across both peers. This
avoids separate client/server codecs; it also requires clear ownership and error
boundaries. Content decoding should not silently choose HTTP status policy or
complete/dispose a server context. Preserve the current separation between setting
response status/content and asynchronously completing the exchange unless explicitly
revisited. Sending a JSON request and decoding a JSON response are distinct operations;
PostJson need not assume every successful response contains JSON.

Comparison reviewed 2026-09-25: .NET's
[HttpClientJsonExtensions](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.json.httpclientjsonextensions)
provides verb conveniences, while
[HttpContentJsonExtensions](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.json.httpcontentjsonextensions)
provides content deserialization. ASP.NET Core separately supplies
[request JSON reading](https://learn.microsoft.com/en-us/dotnet/api/microsoft.aspnetcore.http.httprequestjsonextensions?view=aspnetcore-10.0)
and [response JSON writing](https://learn.microsoft.com/en-us/dotnet/api/microsoft.aspnetcore.http.httpresponsejsonextensions?view=aspnetcore-10.0).
These are existing APIs used for comparison, not a promise to copy their overloads,
exception behavior, package split or asynchronous serializer design. neoCLR can
share its content model across both sides, but still needs distinct send/read and
response-lifecycle semantics.

Open choices include names/placement, content-type validation, empty bodies versus
JSON null, mapping failures versus transport/status failures, cancellation and future
stream ownership/consumption. The serializer remains synchronous today; future HTTP
helpers can compose asynchronous transport with synchronous conversion of buffered
content without claiming asynchronous stream parsing. Validate an object request
and object response in one round trip when this later slice is implemented.


## Provisional public object mapping — 2026-09-25

JsonSerializer adds non-generic Deserialize(text/input, TypeInfo) returning
Result<Object, JsonError>, and Serialize(Object)/Serialize(output, Object).
The [public consumer](experiments/json-object-mapping/Public.rvn) uses the real
library, not an application copy of the mapper. The opt-in HTTP variant now uses
these overloads for both report and acknowledgement objects, exercising all four
client/server conversion boundaries without introducing HTTP JSON extensions yet.

The deliberately shallow contract supports String, Int32 and Boolean properties on
nongeneric reference classes. Public readable instance properties serialize; public
writable instance properties deserialize. All writable properties must be present;
unknown fields are ignored. Exact case-sensitive property names are used. Fields,
static properties and inaccessible accessors are skipped. Participating indexers,
unsupported types and null values fail. There is no recursive mapping, attribute,
naming or coercion policy. DOM objects held as Object retain the DOM codec.

Compare [.NET object deserialization](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/deserialization)
and [property naming](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/customize-properties)
(reviewed 2026-09-25). Explicit runtime-type overloads and exact-name matching are
familiar .NET concepts. Unlike .NET's broad mapper, this POC rejects nested models
and nulls and requires all writable properties, even without a required-member
annotation. That strict rule catches missing data without inventing defaults for
absence, but prevents partial DTO inputs; it is a provisional cost, not a claim of
superiority. Naming policies/attributes remain future work. The HTTP payload types
explicitly use lowercase wire-property names to preserve the established schema.

Validate the complete set of input values before constructor/setter execution.
JsonError.Reflection retains ReflectionError; UnsupportedMapping carries a diagnostic
for unsupported shapes. Constructor and accessor Faults remain terminal. User-code
side effects are not transactional. Serialization validates the complete DOM/output
before stream writes; streams stay borrowed and synchronous. Existing JSON limits
still apply. API XML and the [on-site guide](../api-docs/json.md) describe each overload.

The JsonValue slice enables the already-existing target typeof Runtime Contract;
see [compiler integration](raven-system-library.md#json-mapping-typeof-configuration--2026-09-25).
The subsequent typed-read slice adds `Deserialize<T>` for string and InputStream
inputs using this same mapper. Unsupported T shapes return UnsupportedMapping; no
value-type or recursive mapping is added. Shared HTTP JSON content helpers remain
next candidates.

## Node naming and typed reads — exploration, 2026-09-25

The author suggested `DeserializeNode`/`SerializeNode` to make the DOM boundary
explicit. This is a usability choice independent of the Raven generic-arity
regression: `Deserialize(string)` and `Deserialize<T>(string)` are distinguishable
by generic arity. Retaining overloads minimizes churn; separate node names make
intent visible and avoid depending on inference to select DOM versus model APIs.
Evaluate the names together before changing this development API and its samples.

The author also proposed selecting `ConvertibleTo<T>.Into()` by expected result
type. [C# signatures](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/basic-concepts#75-signatures-and-overloading)
include method generic arity but exclude return type; this proposal is a deliberate
language extension, not a compatibility fix. [Rust Into](https://doc.rust-lang.org/std/convert/trait.Into.html)
places the destination in the trait parameter (and recommends implementing From
for its blanket Into implementation). It is not evidence for arbitrary methods
with return-type-only differences. Sources checked 2026-09-25.

A later Raven experiment should compare target-directed interface selection with
ordinary generic methods and explicit interface qualification. Cover absent target
types, equally convertible return types, argument/return inference cycles, method
groups, diagnostics, metadata identity and .NET consumers. Existing target typing
can support the implementation but does not establish a complete selection rule.
No new conversion interface or return-type-only overload rule is adopted here.


## Explicit DOM names — 2026-09-26

The author selected the Node naming direction. Public DOM reads/writes now use
`DeserializeNode` / `SerializeNode`; object mapping retains `Deserialize<T>`,
TypeInfo-based `Deserialize`, and `Serialize(Object)`. String and stream variants
follow the same split. Old DOM signatures are renamed without compatibility aliases
in this development API. Rebuild consumers with matching reference/library artifacts.

This builds on the .NET comparison above: System.Text.Json separates DOM parsing
through JsonNode/JsonDocument entry points while JsonSerializer handles typed
mapping. neoCLR keeps one provisional serializer class and marks its DOM operations
by name. The benefit is explicit call intent without target-type inference; the cost
is call-site migration and an API spelling different from .NET. No parser, mapping,
error, stream ownership or overload-resolution semantics change. A node passed to
Serialize(Object) retains the existing DOM behavior, avoiding an accidental reflection
mapping of its implementation properties. Return-directed overload selection remains
an independent Raven exploration.


## Shared buffered HTTP conversion — 2026-09-26

The first shared layer is `System.Web.Http.Json.JsonContent`, a non-instantiable
helper class with Create(Object), CreateNode(JsonValue), ReadNode(HttpContent),
Read(HttpContent, TypeInfo) and Read<T>(HttpContent). It composes the existing
serializer and UTF-8 codec; it is not another HttpContent implementation. Generic
reads use the existing bounded generic-function bridge representation. The JSON
source slice now includes this HTTP integration helper; the eventual package split
remains provisional and adds no transport dependency to the serializer itself.

This follows the previously researched .NET separation of content conversion from
client verbs, with two deliberate differences: shared request/response content on
both peers and synchronous Result-based conversion of already buffered bytes.
The benefit is one error/mapping policy for all four boundaries; the costs are
whole-buffer work, a provisional static-call surface instead of extension syntax,
and no cancellation during conversion. Future stream content must revisit lifetime,
consumption, async reads and limits before reusing this buffered policy.

Create selects application/json; charset=utf-8. Reads explicitly interpret the
buffer as UTF-8 JSON without checking Content-Type or HTTP status. This permits
calling code to decode known payloads even when headers are absent or misleading,
but callers requiring a media-type policy must enforce it themselves. Empty input
is a syntax error; JSON null remains a node and is not a supported flat model.
The 128-byte bound is checked before decoding; invalid UTF-8 retains
JsonError.Read(TextReadError.InvalidUtf8). Serializer/reflection errors are preserved.
No transport errors, automatic status rejection, completion or disposal are added.

The public mapper consumer checks all five operations, repeat reads, non-JSON media
type with valid JSON, empty/malformed input, invalid UTF-8, size limits and unsupported
models. It passes with zero final live objects. The mapped HTTP example now shares
these helpers for request creation/read and response creation/read; HTTP status and
server completion stay application decisions. Client verb and instance extension
conveniences remain the next bounded layer, not implemented here.


### Next layer: generic client verbs — author direction, 2026-09-26

The author selects generic-only client conveniences and asks about .NET naming.
[.NET HttpClientJsonExtensions](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.json.httpclientjsonextensions?view=net-10.0)
uses GetFromJsonAsync<T> and PostAsJsonAsync<T>, not GetAsJson<T>. The neoCLR direction
is GetFromJson<T> and PostAsJson<T>, omitting Async consistently with existing verbs.
GET decodes a successful response into T; POST serializes its T request value and
returns HttpResponse, leaving response decoding to a separate content operation.
No TypeInfo, Object-only or DOM-specific client verb overloads are required now.
Preserve existing string/Uri base-address rules and cancellation behavior when
implementing these conveniences. The combined HTTP/JSON error contract still needs
to be established without erasing either structured cause.

The author also proposes HttpResponse.Request: Option<HttpRequest>. The assistant's
proposed default is None for standalone/server-created responses, with client Send
associating the originating request even through custom handlers. Validate request
identity, pending completion, cancellation, handler failures and GC retention before
adopting that policy; do not imply redirect/final-request behavior the POC lacks.
These are next-slice directions, not implemented APIs in this checkpoint.


### Response association checkpoint — 2026-09-26

The author explicitly prioritizes HttpResponse.Request next. The implementation uses
Option<HttpRequest> with a read-only public getter. HttpClient.Send associates each
successful Result response with the effective request passed to its handler, for
socket and custom handlers alike. Without applied defaults this is the caller's
original request; defaults can produce a derived request. Non-success HTTP statuses still carry a response and are
associated. Standalone/server-created responses start with None; direct handler calls
have no association guarantee. Respond changes status/content without clearing Request.
Failed Results and cancelled tasks do not mutate an unrelated response.

Compared with [.NET RequestMessage](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpresponsemessage.requestmessage?view=net-10.0),
neoCLR uses explicit Option absence and no public setter. .NET documents the actual
final request after redirects/authentication; this POC records the request after
applying client defaults and does not implement those rewrites. This limitation must be revisited
when adding redirects or request-transforming handlers. A response reused by a custom
handler across sends is mutable: its last association replaces its previous one.

Already-completed handler tasks preserve their readiness. Pending tasks use a
per-send Map continuation on the existing dispatcher, retaining the request until
completion. The extra continuation/allocation is a provisional cost; no scheduler
or runtime suspension contract is introduced. Keeping a response alive retains its
request and content through ordinary GC references without transferring ownership.


### Client defaults and deferred per-call customization — 2026-09-26

The author postpones per-call header parameters on JSON helpers and requests client
DefaultRequestHeaders. The property uses Sequence<HttpHeader>, with copies on both
get and set. Send validates the snapshot before invoking the handler and derives a
request only when defaults are actually added; the caller request is unchanged and
its content is shared. Explicit headers override defaults by case-insensitive name.
Repeated default values for absent fields remain ordered. Transport-controlled
headers and Content-Type are rejected, as with WithHeader; JSON content keeps its
media type. Invalid defaults fail before custom or socket handlers are called.
More than 13 default/merged application fields returns LimitExceeded. Already-requested
cancellation takes precedence over preparation. These snapshots do not promise
thread-safe concurrent mutation of configuration or supplied collections.

This follows [.NET's client-wide default-header role](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.defaultrequestheaders?view=net-10.0), using the current sequence model
rather than adding a new specialized header collection. The benefit is a small API
composing with existing header validation; the cost is replacement-style configuration
and copied collections instead of mutating a property-owned header collection.
Per-call JSON-helper options and richer header APIs remain later design work.

### Generic client integration — 2026-09-26

The implemented `GetFromJson<T>`/`PostAsJson<T>` layer follows the previously
recorded .NET naming comparison, without an Async suffix. Both string and Uri
forms have optional cancellation overloads. GET validates successful HTTP status
before decoding; POST returns HttpResponse without implicit response decoding or
success enforcement. Existing BaseUri and default request headers are reused.

A standard `HttpJsonError` union retains either `Http(HttpError)` or `Json(JsonError)`;
it avoids flattening transport, status, parser and mapping failures into text.
Applications can project both cases into their own error union via local implicit
converters and continue using propagation. Cancellation remains task cancellation,
not another result case. Pre-cancellation skips serialization; later cancellation
uses the existing HTTP contract. Buffered JSON conversion stays synchronous.

The public surface has only generic verbs. Internal non-generic operations carry
TypeInfo and adapt the HTTP result; a bounded generic callback returns T. This keeps
reflection and scheduler details out of the public API. The cost is continuation
allocation/queue progression and the mapper's existing limitations. It introduces
no public scheduler contract and can be revised when runtime suspension arrives.
Per-call headers, streaming serialization and serializer options remain deferred.

The independent compiler reduction and general fix are recorded in the
[prototype investigation](experiments/http-json-client-prototype/README.md).
Reference-instantiated generic boxing now preserves Object identity; values retain
copied boxing. See [runtime semantics](boxed-interface-values.md#generic-reference-boxing--2026-09-26).


## Nested typed objects — development, 2026-09-27

The author-selected Web API direction now extends the existing mapper with nested
nongeneric reference properties. Reuse the System.Text.Json comparison in the
[Web API plan](web-api-plan.md#comparison-alternatives-and-costs): typed recursion
closes a neoCLR implementation gap while retaining its existing UTF-8 byte bounds,
Result causes and explicit reflection behavior. This is a library change with no
new public signatures, runtime mechanisms or compiler configuration.

Reads use declared property types and public parameterless construction. A first
pass validates the complete input tree before any model constructor or setter; a
second pass constructs children and parents and assigns properties. Exact names,
required writable properties, ignored unknown fields, String/Int32/Boolean leaves
and null rejection remain unchanged. Construction/accessor side effects are not
transactional; reflection failures retain their structured causes and user Faults
remain terminal. The two passes add traversal and temporary-list allocation; no
performance improvement is claimed. A retained mapping plan could avoid repeated
work but would introduce additional lifetime/storage machinery before it is needed.

Writes require a nested property's runtime type to equal its declared type. This
avoids silently serializing a derived shape that the declared read type cannot
reconstruct; it also excludes polymorphic DTO properties. Shared children are
serialized repeatedly and read back as independent instances. Four object levels,
including the root, are allowed; deeper graphs and cycles return LimitExceeded.
Depth bounding reuses the existing DOM contract and avoids a reference-tracking
protocol, but does not distinguish a cycle from an ordinary overly deep graph.
Nulls, collections, Option, generic/value models, naming policies and configurable
limits remain deferred. The existing 128-byte/32-value document limits still apply.
Mapping and encoded-document validation finish before stream writes.

The private traversal uses integer pass selection (0 validate, 1 construct) because
the current bridge rejects Boolean argument conversion on multi-argument private
calls. This local workaround does not weaken access or change Raven; a general
bridge conversion improvement remains outside this serializer slice.

The [focused consumer](experiments/json-object-mapping/Public.rvn) covers string,
stream and HTTP-content round trips, nested invalid values without constructor or
setter output, shared siblings, four/five levels, cycles, nulls, inaccessible nested
constructors and unsupported shapes. The station-report case has an opt-in nested
variant on the [HTTP case page](../website/content/cases/http-server/index.md); its independent
peers and complete exchange are recorded with the implementation evidence.


## Web API byte budget — development, 2026-09-27

The next bounded increment raises complete JSON documents and number tokens from
128 to 1,024 UTF-8 bytes, matching the existing buffered HTTP body limit. A station
report with a nested sensor name and installation description now fits through
string, borrowed stream and HTTP-content conversion. Preview 10 remains unchanged.
The limit includes input whitespace, names, punctuation and escaped output; it is
not a character count. Four container/object levels, 32 value occurrences and 31
children per container remain unchanged. Typed collections and optional/null mapping
are separate next decisions, not implicitly enabled by a larger document budget.

The cap remains fixed to keep this increment within an already supported transport
budget. Configurable larger budgets would require transport and runtime resource
policy together. In .NET, [JsonSerializerOptions.DefaultBufferSize](https://learn.microsoft.com/en-us/dotnet/api/system.text.json.jsonserializeroptions.defaultbuffersize)
is a temporary-buffer setting (16,384 bytes by default), not a document-size cap;
[MaxDepth](https://learn.microsoft.com/en-us/dotnet/api/system.text.json.jsonserializeroptions.maxdepth)
defaults to an effective 64 levels. Sources reviewed 2026-09-27. neoCLR's fixed 1 KiB
and four-level contract is deliberately narrower. The benefit is room for this
concrete API report without conflicting JSON/HTTP byte caps; the cost is more
whole-buffer storage and synchronous traversal. No performance advantage is claimed.

String and buffered HTTP input reject more than 1,024 bytes with LimitExceeded.
Borrowed stream input reads one extra byte to detect overflow and retains the
existing Read(TextReadError.LimitExceeded) cause without rewinding or closing.
Writers validate escaped output and the complete document before writing to a
stream; I/O failures after that may still leave a prefix. Number tokens longer
than 1,024 bytes retain InvalidNumber. HTTP transport overflow remains its existing
error contract; this change does not introduce automatic HTTP 413 responses.

[Payload validation](experiments/json-object-mapping/payload-validation.json) records
focused DOM/stream and nested-model consumers plus independent HTTP peers. Tests
exercise UTF-8 and escape expansion, long number spelling, the exact byte boundary,
one byte over, borrowed ownership and untouched output on validation failure.


## Typed arrays — development, 2026-09-27

The collection slice maps vectors at the root and as properties, with String,
Int32, Boolean, model or vector elements. Root scalar values use the same value
mapper. Empty arrays and ordering are preserved; jagged arrays reuse recursive
mapping. Objects and arrays both count toward four container levels. The existing
1,024-byte, 32-value and 31-item bounds apply. Null, polymorphic elements, generic
lists/interfaces, dictionaries and rectangular arrays remain unsupported here.
Whole-input validation still precedes model constructors/setters, including when
an invalid value occurs in a later array element.

[System.Text.Json collection support](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/supported-types)
(reviewed 2026-09-27) covers arrays and many generic/custom enumerable collections;
its byte arrays have a special base64 mapping. neoCLR starts with exact typed
vectors because it already has checked array storage and type descriptors, while
general generic-class activation is outside its current reflection contract.
This closes the concrete report-array use case at the cost of a narrower surface;
byte arrays and arbitrary enumeration are not silently treated as supported JSON.

Three private array services build small execution adapters using ordinary array,
boxing and cast instructions. They do not write managed memory through a native
shortcut; bounds/type checks, allocation limits, instruction budgets and GC roots
remain interpreter-owned. JSON policy stays in the Raven mapper. The private
ObjectMapper.Write bridge contract now returns JsonValue so the root can be an
array or scalar; public JsonSerializer signatures are unchanged.

See [focused collection validation](experiments/json-object-mapping/collection-validation.json)
for runtime adapter, typed mapper and independent HTTP peer results.
