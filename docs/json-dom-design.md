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

JsonSerializer currently operates only on JsonValue. String and stream Deserialize
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
This does not yet expose creation on the Raven API or implement the mapped report.
Checked instance property execution now preserves accessor code and virtual dispatch,
with exact scalar boxing and reference/null support. The Result-based reflection
facade and a compiled consumer are next, before the mapped report.
