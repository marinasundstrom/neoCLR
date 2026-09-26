---
title: JSON serialization
---
# JSON serialization

**Development after Preview 9.** [JsonSerializer](xref:System.Data.Json.JsonSerializer)
reads/writes the closed JsonValue DOM and now has provisional flat-object overloads.
Rebuild consumers with the matching development reference and library; JsonError has
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

- Map String, Int32 and Boolean public instance properties with exact, case-sensitive
  names. No naming policy or attribute support; property order is not promised.
- Serialize public getters, including read-only properties. Deserialize public
  setters; read-only/private setters and static properties are ignored. Fields are ignored.
- Every writable mapped property must be present on input. Unknown JSON fields are
  ignored; duplicate decoded names remain invalid. This strict presence rule differs
  from .NET's default treatment of non-required missing properties.
- Reject null, nested model objects, collections, Option, enums, unsupported scalars
  and indexers in participating properties. Int32 requires a checked integer token;
  fraction/exponent tokens are not coerced.

Validate input property shapes and values before invoking the constructor. Setter
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
31 children per container. Object mapping is flat even though DOM values can nest.
Recursive models, configurable naming/null policy and
HTTP JSON extensions for requests and responses remain later work.

## Error payload reference

See [JSON error payloads](json-error-payloads.md) for the case properties and their
value types. Case signatures also appear on the generated JsonError union page.
