---
uid: N:System.Data.Json
---
## JSON values and serialization

JsonValue and its object, array and scalar forms represent JSON data. JsonSerializer
provides parsing, writing and supported value mapping, with JsonError describing
syntax, type, field, size and mapping failures.

See [JSON](/docs/json.html) for supported shapes and limits, and
[error payloads](/docs/json-error-payloads.html) for inspecting failures.
System.Web.Http.Json adds HTTP integration on top of these contracts.

`JsonObject` implements `Map<string, JsonValue>`. Use `value["name"]`
for an existing field; a missing field causes a terminal Fault. Prefer `Find`
(`Some`/`None`) or `Field` (typed JSON error) when absence is expected. A present
`JsonNull` remains a node, distinct from absence. Keys and pair iteration are shallow,
insertion-ordered snapshots retaining original node references. Mutation still uses
`Add`, which rejects duplicates and reports the container quota through Result;
`JsonObject` does not implement `MutableMap` yet.

`JsonArray` implements `Sequence<JsonValue>`. Use `array[index]` for
known valid indices, or `Item(index)` for a recoverable `InvalidIndex` result.
Negative and past-end indexer accesses cause a terminal Fault. `for` iteration
captures a shallow ordered snapshot: later additions are excluded, while child
nodes remain shared. Count and indexing read the current array. Mutation continues
to use quota-aware `Add`; `MutableSequence` is not implemented.

Use `node.ToJsonString()` for compact JSON text. For readable output, pass a
`JsonSerializerOptions` with `WriteIndented = true`. The same option is available
on `JsonSerializer.SerializeNode` and `Serialize`, including stream overloads.
[Formatting and limits](/docs/json.html#json-text-and-indentation).

Development byte output: `SerializeToUtf8Bytes(value[, options])` returns an owned
byte array. HTTP content and stream serialization use the same bounded UTF-8 writer,
avoiding intermediate document strings. Stream input still parses through text.
See [byte output and ownership](/docs/json.html#utf-8-byte-output-development).


Development parsing borrows the source string's UTF-8 bytes and creates owned text
only for string values, names and number lexemes. Escaped tokens still need scratch
storage. HTTP and stream deserialization still decode full text before parsing;
direct byte input and incremental readers remain future work.
