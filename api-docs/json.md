---
title: JSON serialization
---
# JSON serialization

**Preview 10.** [JsonSerializer](xref:System.Data.Json.JsonSerializer)
reads/writes the closed JsonValue DOM and has provisional flat-object overloads.
**Development after Preview 10** also maps nested nongeneric reference objects.
Rebuild consumers with the matching Preview 10 reference and library; JsonError has
new mapping cases. All operations are synchronous and return Result with [JsonError](xref:System.Data.Json.JsonError).

Use the explicit Node methods for the DOM boundary:

```text
DeserializeNode(text: string) -> Result<JsonValue, JsonError>
DeserializeNode(input: InputStream) -> Result<JsonValue, JsonError>
SerializeNode(value: JsonValue) -> Result<string, JsonError>
SerializeNode(output: OutputStream, value: JsonValue) -> Result<unit, JsonError>
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
targets such as value types. Construction requires a
runtime-backed public nongeneric reference class with a public parameterless
constructor. The serializer invokes real constructors/getters/setters through
[runtime reflection](reflection.md); it never writes backing fields directly.
JsonValue inputs to the Object overload still use the DOM codec, including when
held as Object. Prefer SerializeNode when explicitly working with nodes. The earlier
DOM Deserialize/Serialize signatures have been renamed; migrate DOM callers and
rebuild with matching references and library artifacts.

## Provisional property rules

- Map String, Int32 and Boolean public instance properties, plus nested nongeneric
  reference objects in development, with exact, case-sensitive
  names. No naming policy or attribute support; property order is not promised.
- Serialize public getters, including read-only properties. Deserialize public
  setters; read-only/private setters and static properties are ignored. Fields are ignored.
- Every writable mapped property must be present on input. Unknown JSON fields are
  ignored; duplicate decoded names remain invalid. This strict presence rule differs
  from .NET's default treatment of non-required missing properties.
- Reject null, collections, Option, enums, unsupported scalars
  and indexers in participating properties. Int32 requires a checked integer token;
  fraction/exponent tokens are not coerced.

Development nested writes require each property value to have its declared runtime
type; polymorphic properties are rejected. Reads use each declared class and its
public parameterless constructor. Shared children serialize as repeated JSON objects;
deserialization constructs independent instances rather than preserving identity.

Validate the entire input tree before invoking any model constructor or setter. Setter
failures and user Faults are not transactional; side effects are not rolled back.
The Reflection error case retains ReflectionError. UnsupportedMapping identifies
unsupported shapes through diagnostic text; ordinary syntax, missing-field,
type-mismatch, number and stream causes retain their existing JsonError cases.
Terminal Faults from user accessors/constructors are not wrapped.

## Streams and limits

Streams are borrowed, left open and not flushed. Reads consume through EOF and may
consume input before failure. Writes validate mapping and the entire encoded document
before touching output; a stream failure can still leave a written prefix. This is
buffered synchronous conversion, not async stream parsing.

Existing bounds apply: 128 UTF-8 bytes, four container levels, 32 value occurrences,
31 children per container. Development object mapping supports up to four object
levels including the root; deeper graphs and cycles return LimitExceeded. This
bounded recursion does not preserve reference identity. Collections, configurable
limits and naming/null policies remain later work. Shared content conversion is
described below.

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
continues rejecting it. The byte limit is checked before decoding.

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
model before sending and returns `HttpResponse`, preserving non-success statuses
and content for the application to inspect. Both accept string or Uri addresses,
with optional CancellationToken overloads, and honor BaseUri and DefaultRequestHeaders.

Failures return [HttpJsonError](xref:System.Web.Http.Json.HttpJsonError): `Http(reason)`
retains HTTP/transport/status errors; `Json(reason)` retains mapping, syntax, UTF-8
and limit failures. Cancellation remains task cancellation. No per-call headers,
serializer options, streaming codec or asynchronous serialization are introduced.

The managed report sample gets a typed report, posts it back, then reads the
acknowledgement from the response. Application-owned implicit error converters
allow `?` to propagate either cause into its AppError union. The response's Request
property identifies the effective POST, including client default headers.
