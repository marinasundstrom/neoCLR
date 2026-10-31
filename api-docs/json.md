---
title: JSON serialization
---
# JSON serialization

[JsonSerializer](xref:System.Data.Json.JsonSerializer) reads and writes JSON values
and maps supported objects and arrays. Use JsonValue for a document tree, or the
typed overloads for nongeneric reference objects, typed arrays and root String,
Int32 or Boolean values.

In-memory operations return Result with
[JsonError](xref:System.Data.Json.JsonError). The mapping rules and size limits
below define the supported subset. Stream operations return Tasks; use `await`.


## UTF-8 byte output (development)

`JsonSerializer.SerializeToUtf8Bytes(value)` and
`SerializeToUtf8Bytes(value, options)` return `Result<byte[], JsonError>` containing
newly owned UTF-8 storage. They accept the same supported objects and JsonValue
nodes as Serialize, with the same mapping errors, 1024-byte output limit, 32-value
limit and maximum container depth of four. Formatting counts toward the byte limit.
Changing a returned array does not change the source node or another result.

HTTP JsonContent creation and OutputStream serialization use this byte writer.
They do not construct a whole-document string or quoted-token strings. Text-returning
Serialize/SerializeNode still produce owned strings because their caller requests text.
Development input parsing borrows the source string’s UTF-8 bytes. Unescaped JSON
strings decode directly into their final owned text; escaped strings use scratch
storage. Numbers retain their original spelling, and boolean/null tokens need no
temporary strings. HTTP byte input uses the borrowed overloads below; stream input still materializes
whole-document text before invoking the parser. Incremental parsing and Utf8JsonReader/Utf8JsonWriter
contracts remain future work.

Stream APIs validate the complete document before the first write and leave the
stream open without flushing it. An output I/O failure can follow partial output;
its original StreamError remains wrapped by JsonError.Write. A pending write retains
its owned byte array until completion. Shared OutputStream currently accepts an
array and range; the wider Memory signature migration remains separate.

## UTF-8 byte input (development)

The native reference includes these synchronous overloads:

```text
DeserializeNode(utf8: ReadOnlySlice<byte>) -> Result<JsonValue, JsonError>
Deserialize<T>(utf8: ReadOnlySlice<byte>) -> Result<T, JsonError>
Deserialize(utf8: ReadOnlySlice<byte>, type: TypeInfo) -> Result<Object, JsonError>
```

Only the selected window is read. Input is borrowed for the call and never stored;
returned nodes, property names, strings and number lexemes own their text.
Subsequent changes to the input array do not change a completed result. Mapping
uses the same supported types and errors as the text overloads.

Complete strict UTF-8
validation runs without creating a document string. Malformed UTF-8 returns
`JsonError.Read(TextReadError.InvalidUtf8)`, even if a JSON syntax error appears
earlier in the bytes. JSON parsing and mapping follow validation. Overlong sequences,
surrogate encodings, truncated sequences and values above U+10FFFF are rejected.
A BOM remains a JSON syntax/number error; it is not stripped. Empty input fails
with `Syntax("Missing value")`; raw NUL is valid UTF-8 but invalid inside a JSON string.

String-backed readonly slices and nonzero array windows are supported. Explicit
byte input does not change String's grapheme indexing or add an incremental JSON
reader. The legacy CLI documentation scaffold omits these three borrowed signatures;
its opaque view types cannot safely project them. This section and the native
JsonSerializer member pages provide their reference.

## JSON text and indentation

`JsonValue.ToJsonString()` returns compact JSON, including quotes and escaping for
string values. Pass [JsonSerializerOptions](xref:System.Data.Json.JsonSerializerOptions)
with `WriteIndented = true` for two-space indentation and LF newlines on every platform.
Empty containers remain `{}` and `[]`; no trailing newline is added.

```raven
func FormatJson(value: JsonValue) -> Result<string, JsonError> {
    let options = JsonSerializerOptions()
    options.WriteIndented = true
    return value.ToJsonString(options)
}
```

`JsonSerializer.SerializeNode(value, options)` and `Serialize(value, options)` use
the same formatting. Their output-stream overloads also accept options and return
Tasks. Options are read when serialization begins. Complete validation precedes
stream writes; the stream remains open. Formatting counts toward the existing
1,024 UTF-8 byte output limit. Invalid graphs and exceeded quotas return `JsonError`.

Use the explicit Node methods for the DOM boundary:

```text
DeserializeNode(text: string) -> Result<JsonValue, JsonError>
DeserializeNode(input: InputStream) -> Task<Result<JsonValue, JsonError>>
SerializeNode(value: JsonValue) -> Result<string, JsonError>
SerializeNode(output: OutputStream, value: JsonValue) -> Task<Result<unit, JsonError>>
```

Object mapping uses these methods:

```text
Deserialize<T>(text: string) -> Result<T, JsonError>
Deserialize<T>(input: InputStream) -> Result<T, JsonError>
Deserialize(text: string, type: TypeInfo) -> Result<Object, JsonError>
Deserialize(input: InputStream, type: TypeInfo) -> Result<Object, JsonError>
Serialize(value: Object) -> Result<string, JsonError>
Serialize(output: OutputStream, value: Object) -> Result<unit, JsonError>
```

Use `JsonSerializer.Deserialize<YourClass>(text)?` for a typed result, or pass
`typeof(YourClass)` to the non-generic read overload. Both use the same mapper and
return the same structured errors, including UnsupportedMapping for unsupported
targets such as unsupported value types and non-string map keys. Object construction requires a
runtime-backed public nongeneric reference class with a public parameterless
constructor. The serializer invokes real constructors/getters/setters through
[runtime reflection](reflection.md); it never writes backing fields directly.
JsonValue inputs to the Object overload still use the DOM codec, including when
held as Object. Prefer SerializeNode when explicitly working with nodes.

<a id="dynamic-json-content-inside-typed-models-development"></a>

## Dynamic JSON content inside typed models

Declare a property or array element as `JsonValue` to retain any JSON kind inside
an otherwise typed model. `JsonObject`, `JsonArray`, `JsonString`, `JsonNumber`,
`JsonBoolean` and `JsonNull` require the matching kind; a mismatch returns
`JsonError.TypeMismatch` before model constructors or setters run. The same rule
applies to root `Deserialize<T>` calls and the TypeInfo overload.

Serialization embeds node content directly. It does not reflect over node members
or quote an object/array as text. Number tokens keep their spelling. JSON null is an
explicit `JsonNull` node; CLR null references remain unsupported. `Object`-typed
properties do not infer JSON node mapping. The complete mixed document shares the
existing depth, occurrence and byte limits, including cycles inside embedded nodes.

The [tested NodeEnvelope sample](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/native-json/Main.rvn) combines
`JsonValue`, `JsonObject` and `JsonValue[]` properties. Supported lists and maps
can also contain explicitly declared JSON nodes.

## Value-record serialization (development)

`Serialize` also accepts public nongeneric record structs, including read-only
properties and nested class/record properties. For example, a public
`record struct Person(val Name: string, val Age: int)` holding `"Bob"` and `42`
serializes to `{"Name":"Bob","Age":42}`. See the
[tested consumer](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/json-records/Main.rvn).
Native builds must retain property metadata and getters for each model type through
`--reflection-roots`; the test driver demonstrates that configuration.

Getters execute against the boxed value. Mutating getters can change that box;
the original unboxed value remains unchanged. Nested record results are copied into
new boxes and traced normally. The same JSON, errors and quotas apply in interpreter,
JIT and AOT. Empty records serialize as `{}`; fields alone are not JSON properties.

This adds serialization, not constructor mapping for record deserialization.
Deserializing a non-scalar value record returns `UnsupportedMapping` before executing
model constructors. Generic records, enum/union mapping, custom value-record array
or collection elements, nonpublic model types and unsupported scalar properties
remain outside the supported mapper subset. Existing bounds still apply.

## Provisional property rules

- Map String, Int32 and Boolean public instance properties, plus nested nongeneric
  reference objects, record structs for serialization, and typed arrays in development, with exact, case-sensitive
  names. No naming policy or attribute support; property order is not promised.
- Serialize public getters, including read-only properties. Deserialize public
  setters; read-only/private setters and static properties are ignored. Fields are ignored.
- Every writable mapped property must be present on input. Unknown JSON fields are
  ignored; duplicate decoded names remain invalid. This strict presence rule differs
  from .NET's default treatment of non-required missing properties.
- Reject CLR null references, unsupported collection families, Optional, enums, unsupported scalars
  and indexers in participating properties. Int32 requires a checked integer token;
  fraction/exponent tokens are not coerced.

Development nested writes require each property value to have its declared runtime
type; polymorphic properties are rejected except for explicit JsonValue declarations
and the supported collection interfaces described below. Reads use each declared class and its
public parameterless constructor. Shared children serialize as repeated JSON objects;
deserialization constructs independent instances rather than preserving identity.

Validate the entire input tree before invoking any model constructor or setter. Setter
failures and user Faults are not transactional; side effects are not rolled back.
The Reflection error case retains ReflectionError. UnsupportedMapping identifies
unsupported shapes through diagnostic text; ordinary syntax, missing-field,
type-mismatch, number and stream causes retain their existing JsonError cases.
Terminal Faults from user accessors/constructors are not wrapped.

<a id="typed-arrays--development"></a>

## Typed arrays

Use `Deserialize<int[]>("[21,22]")` or `Deserialize<ReportPayload[]>(text)` for
root arrays. Array-valued model properties use the same mapper; strings, Int32,
Boolean, supported models and jagged arrays can be elements. String/Int32/Boolean
also work as root values. The non-generic TypeInfo, borrowed stream, JsonContent
and generic HTTP helper paths share this behavior.

Order and empty arrays are preserved. Each array admits at most 31 elements;
every object or array consumes a container level and contributes to the existing
32-value document bound. Element types are exact, with no coercion or polymorphic
substitution. Null arrays/elements are rejected; missing writable array properties
remain errors. Rectangular arrays and unsupported scalar elements remain outside
this slice; supported collection elements use the rules below. Shared model elements
serialize repeatedly and deserialize independently. The complete input tree,
including later elements, is validated before any model constructor/setter runs.

<a id="lists-sequences-and-string-keyed-maps--development"></a>

## Lists, sequences and string-keyed maps

| Declared type | JSON shape | Deserialized implementation |
| --- | --- | --- |
| `Sequence<T>`, `List<T>`, `ArrayList<T>` | Array | `ArrayList<T>` |
| `Map<string, T>`, `MutableMap<string, T>`, `HashMap<string, T>` | Object | `HashMap<string, T>(StringComparer.Ordinal)` |

These exact built-in families work at roots, properties and recursively as elements.
Supported element types include nested models, arrays, collections and explicit
JSON nodes. Empty collections and sequence order are preserved. Map keys are
case-sensitive UTF-8 strings; writing uses the map's observed key order. Reading
uses ordinal comparison, so a custom comparer is not preserved by a round trip.

Object mapping retains its four-container-level and 32-value bounds even though
DOM parsing now supports larger documents. Serialization retains its 1024-byte
output bound. Invalid later elements fail
preflight before any model constructor or setter executes. Each occurrence is
constructed independently; shared references are not preserved.

Arbitrary collection implementations, custom construction factories and non-string
keys are unsupported. `Object` element declarations still do not infer JSON nodes.
Ordinary model inheritance, converters, naming policies and union mapping remain
follow-up work. This is bounded development support, not general .NET serializer
compatibility.

## Streams and limits

Streams are borrowed, left open and not flushed. Reads consume through EOF and may
consume input before failure. Writes validate mapping and the entire encoded document
before touching output; a stream failure can still leave a written prefix. This is
buffered synchronous conversion, not async stream parsing.

DOM reads no longer impose a fixed byte, value-count or per-container entry quota.
Runtime allocation budgets still apply. Stream input is decoded in chunks and
buffered in full; the text reader's existing per-grapheme 64 KiB bound remains.
DOM writes still allow 1024 UTF-8 bytes (including formatting), 32 value occurrences
and four container levels. Object mapping also retains its four-level/32-value
bounds; reference identity and cycles are unsupported. These remaining limits are
development gaps, not intended permanent JSON contracts.

## Document parsing options

[JsonDocumentOptions](xref:System.Data.Json.JsonDocumentOptions) is a value type,
matching the core design of
[System.Text.Json.JsonDocumentOptions](https://learn.microsoft.com/en-us/dotnet/api/system.text.json.jsondocumentoptions?view=net-10.0).
The text, borrowed UTF-8 and asynchronous InputStream DeserializeNode overloads
accept it. Omitting options uses the same defaults:

| Property | Default | Behavior |
| --- | --- | --- |
| AllowTrailingCommas | false | Accept one comma before an array/object closing delimiter when true. |
| CommentHandling | JsonCommentHandling.Disallow | Skip accepts and discards line and block comments. Allow is reserved for future token readers and is rejected by document parsing, as in .NET. |
| MaxDepth | 0 | Zero means 64 nested containers. A positive value overrides this; scalar roots consume no container depth. |

Options are copied for the call. Negative depth, Allow or undefined comment policies
return `JsonError.InvalidOptions` before consuming input. This uses neoCLR's Result
error model rather than .NET's throwing property setters. Exceeded depth returns
LimitExceeded. Duplicate decoded property names still return DuplicateField;
.NET's duplicate-property policy is not yet exposed. Generic object mapping still
uses strict defaults; serializer read options are a follow-up API gap.

```raven
var options = JsonDocumentOptions()
options.CommentHandling = JsonCommentHandling.Skip
options.AllowTrailingCommas = true
let result = JsonSerializer.DeserializeNode("/* settings */{\"enabled\":true,}", options)
```

Comments are only recognized between tokens, never within strings or token text.
Malformed/truncated comments, double commas, missing values and trailing non-comment
content still fail. [Neo.Configuration.Json](configuration.html) opts into these
settings, adds a 1 MiB input budget, and strips an initial UTF-8 BOM. Direct JSON
parsing continues to reject a BOM. Large strings are decoded on UTF-8 scalar
boundaries, including escaped strings and surrogate pairs.

## HTTP content conversion

[JsonContent](xref:System.Web.Http.Json.JsonContent) in `System.Web.Http.Json`
shares these operations between client requests, server requests, server responses
and client responses:

```text
Create(value: Object) -> Result<HttpContent, JsonError>
CreateNode(value: JsonValue) -> Result<HttpContent, JsonError>
Read<T>(content: HttpContent) -> Result<T, JsonError>
Read(content: HttpContent, type: TypeInfo) -> Result<Object, JsonError>
ReadNode(content: HttpContent) -> Result<JsonValue, JsonError>
```

Creation sets `application/json; charset=utf-8`. Reads interpret buffered bytes as
UTF-8 JSON; the caller selects header and status policy. All operations are
synchronous, preserve the serializer's errors and limits, and leave content reusable.
Invalid UTF-8 is `JsonError.Read(TextReadError.InvalidUtf8)`. Empty bytes fail JSON
syntax validation; a `null` document is a JsonNull node, while model mapping
continues rejecting it. The byte limit is checked before UTF-8 validation. Array-backed content is borrowed
directly; other Sequence implementations retain one contiguous snapshot per read.
No whole-document string is created. Existing sequence alias behavior is preserved
between calls. Unbuffered content returns Read(IoFailure) without reading or closing
its stream.

These methods do not send a request, complete/close a context or add cancellation.
The [mapped HTTP sample](/cases/http-server/) uses the same helpers on both peers.
Generic HttpClient verbs are described below; automatic endpoint binding remains
future work.

## Error payload reference

See [JSON error payloads](json-error-payloads.md) for the case properties and their
value types. Case signatures also appear on the generated JsonError union page.


<a id="generic-http-client-helpers-development"></a>

## Generic HTTP client helpers

Import `System.Web.Http.Json.*` to use
[HttpClientJsonExtensions](xref:System.Web.Http.Json.HttpClientJsonExtensions).
`GetFromJson<T>` sends GET, requires a 2xx response, and reads the model with the
same buffered mapping rules as `JsonContent.Read<T>`. `PostAsJson<T>` serializes a
model before sending and returns `HttpClientResponse`, preserving non-success statuses
and content for the application to inspect. Both accept string or Uri addresses,
with a final `cancellationToken: CancellationToken = default`, and honor BaseUri and DefaultRequestHeaders.

Development failures return `HttpError | JsonError` directly: HttpError retains
HTTP/transport/status errors; JsonError retains mapping, syntax, UTF-8 and limit
failures. This replaces the previous HttpJsonError return signature; rebuild
consumers and use typed patterns after extracting Error. The nominal
[HttpJsonError](xref:System.Web.Http.Json.HttpJsonError) remains available for
explicit application contracts. Cancellation remains task cancellation. No per-call headers,
serializer options, streaming codec or asynchronous serialization are introduced.

The managed report sample gets a typed report, posts it back, then reads the
acknowledgement from the response. The application explicitly adapts the two client error alternatives into its
AppError union; single-cause operations retain application-owned converters. The response's Request
property identifies the effective POST, including client default headers.

<a id="dom-collection-views-development"></a>

## DOM collection views

JsonObject implements `Map<string, JsonValue>`; JsonArray implements
`Sequence<JsonValue>`. Object indexing uses exact field names and array indexing
uses zero-based integers. Indexers return the original node, including JsonNull;
a missing field or invalid index causes a terminal Fault. Use object `Find`/`Field`
or array `Item` when absence or invalid bounds should be recoverable.

Both support ordinary `for` iteration. Object keys/pairs and array elements are
shallow ordered snapshots: later additions do not extend an existing iterator,
while child contents remain shared. Count and indexers read the current container.
Mutation continues through Add; mutable collection interfaces are not
implemented. Typed Serialize still recognizes nodes through these interface views.

Converter registration, serializer options and configurable property naming remain
future work. Their developer experience should follow familiar .NET JSON workflows,
with justified neoCLR differences rather than exact API duplication.

## Asynchronous HTTP content reads

For incoming request content, use `await content.ReadFromJson<T>()`,
`await content.ReadFromJson(type)` or `await content.ReadJsonNode()`. These handle
both buffered and streaming bodies. Streaming reads consume the body once, keep
content ownership with the exchange and pass the optional CancellationToken to
transport reads. Strict UTF-8 and default JSON syntax/depth rules still apply;
object mapping and writing retain the separate limits described above. The legacy
buffered byte-sequence adapter still has a 1024-byte copy cap (a separate gap).
Buffered conversion remains synchronous within the returned task.

The current stream serializer uses a bounded UTF-8 text reader before parsing, so
it still materializes document text. This is not yet an incremental JSON parser
or an allocation-free path. Synchronous `JsonContent.Read` remains a buffered-only
operation and must not be used for incoming streamed requests.
