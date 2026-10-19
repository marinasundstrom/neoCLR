# Avoid temporary text in JSON processing

Author direction, 2026-10-19: avoiding unnecessary string materialization is an API
requirement, including the JSON serializer. This began as a source audit. The byte-output portion is now implemented below;
byte-input parsing and incremental reader/writer APIs remain planned.

The preceding [borrowed decoding change](../../benchmarks/http-server/serve/decode-slice-20261019/README.md)
removes temporary arrays when text is actually needed. JSON also needs to avoid
creating text merely to feed the next byte-oriented parser or output operation.

## Current conversion paths

| Path | Current implementation | Avoidable work to measure/remove |
| --- | --- | --- |
| Http.Json.JsonContent.Read/ReadNode | HttpText.Decode(content.Bytes), then JsonSerializer.Deserialize | Full document bytes → string → bytes in DocumentReader |
| JsonSerializer.DeserializeNode(InputStream) | StreamReader.ReadToEnd, then string overload | Whole document materialization and re-encoding before parsing |
| DocumentReader.ReadString | SliceUtf8 creates a token string; MessageReader re-encodes it and unescapes | Intermediate quoted token string and byte snapshot, separate from the actual returned JSON string |
| JsonContent.Create/CreateNode | Serialize into string, then HttpText.Encode | Full result string when the consumer wants bytes |
| JsonSerializer.SerializeNode(OutputStream, ...) | Serialize into string, then StreamWriter.Write | Full result string and encoding buffers before output |
| DocumentWriter.Append | String.Concat for each fragment | Repeated copies of the accumulated document |
| ObjectMapper.Read | Builds/consumes JsonValue DOM and its property names first | Potential unused DOM/string materialization when mapping directly; separate from the initial byte-path migration |

Source: runtime/raven/src/System/Data/Json/{JsonSerializer,JsonDocument,JsonSyntax,ObjectMapper}.rvn
and runtime/raven/src/System/Web/Http/Json/JsonContent.rvn. Current JSON limits
(1024 encoded bytes, 32 values, depth four) and formatting/validation behavior are
existing provisional contracts, not new limits introduced by this direction.

## API and ownership direction

- Immediate parsing should accept ReadOnlySlice<byte>. Inspect punctuation, literals,
  numbers and property-name tokens as byte windows; create owned strings when the
  returned DOM/object actually requires them. Existing string overloads remain useful
  for callers that already own text, but must not define the internal byte pipeline.
- Asynchronous reads/writes retain owned buffers or Memory/ReadOnlyMemory across
  awaits. Borrow slices only within synchronous parsing/encoding. Do not put a
  borrowed input into an ordinary parser object's field or let views escape into
  long-lived objects. A parser cursor can store positions/state independently of
  the borrowed input.
- Serialize directly into bounded byte storage for HTTP and streams. A string-returning
  API may decode the final bytes once because its caller explicitly requested text.
  Keep whole-document validation before writing: current invalid/quota failures do
  not partially write a stream. True progressive output would be a separately
  documented change in failure/atomicity behavior.
- Preserve Unicode validation, escapes/surrogate pairs, duplicate-property behavior,
  number spelling, quotas, formatting, ownership and error precedence. An unescaped
  string token can be decoded directly once; escaped tokens can require scratch
  storage. Avoid retaining a large input just to delay a small string allocation.
- Typed mapping without a DOM could avoid additional names/values, but do not claim
  it is implemented by adding a byte-input overload that still builds the same DOM.

.NET already exposes [JsonSerializer.Deserialize over ReadOnlySpan<byte>](https://learn.microsoft.com/en-us/dotnet/api/system.text.json.jsonserializer.deserialize)
and [SerializeToUtf8Bytes](https://learn.microsoft.com/en-us/dotnet/api/system.text.json.jsonserializer.serializetoutf8bytes).
Use these as the API/behavior comparison, not a mandate to reproduce every overload.
neoCLR's benefit would be removal of redundant materialization under its UTF-8 model;
costs include parser state/lifetime restructuring, output ownership and coverage of
error/formatting contracts. Borrowed input does not imply borrowed returned strings.

## Next bounded implementation and evidence

First capture JSON-specific text/byte allocation callers for small numeric documents,
repeated property names, unescaped strings, escaped Unicode and near-limit documents.
The existing plain HTTP trace does not exercise JSON and cannot quantify this work.
Record allocations and payload bytes separately from retained memory, CPU and latency.

Then replace DocumentWriter's concatenated text accumulation with bounded UTF-8 byte
output, expose the owned byte result through the appropriate serializer API, and route
HTTP/stream serialization through it. Qualify exact compact/indented output and
validation-before-write before comparing allocation traces. Migrate byte-input parsing
next, including fragmented stream input and selective token materialization. Keep each
slice independently committed and documented; no new API here is marked shipped.

## Byte-output implementation, 2026-10-19

DocumentWriter now copies borrowed String.Utf8Bytes into bounded byte storage and
escapes directly into that buffer. It preflights each token before appending, keeping
existing quota precedence and lowercase control escapes. Public SerializeToUtf8Bytes
returns independent owned arrays; HTTP and OutputStream serialization use byte output.
Text-returning calls decode the completed buffer once. Byte-input parsing is unchanged.

The author considered TextReader/TextWriter overloads, then clarified that the API is
based on System.Text.Json and should allow streams. Existing InputStream/OutputStream
overloads provide that entry point. System.Text.Json's dedicated Utf8JsonReader and
Utf8JsonWriter are distinct from general text readers/writers; those incremental
contracts are future work. Newtonsoft.Json's TextReader/TextWriter overloads were
researched as a comparison, not adopted in this change.

Benefit: no accumulated-prefix or quoted-token strings on byte output. Costs: bounded
scratch storage, growth copies and a final owned array; text-returning serialization
now requires byte accumulation and one decode. This is not zero allocation. OutputStream
still requires array/range; no borrowed descriptor is retained across await. See
[allocation and correctness evidence](../../benchmarks/json-byte-output/20261019/README.md).

### Native string-view prerequisite

The first native attempt rejected ViewUtf8 despite interpreter support. The author
explicitly directed fixing that gap. Native string-backed readonly views are now
implemented and separately qualified, so the adopted writer uses borrowed string
bytes; the intermediate fallback retaining token encoding buffers was not adopted.
See [native bridge contract](../raven-cli-bridge.md#native-stringutf8bytes-lowering-2026-10-19).
The Raven source compiler and public String.Utf8Bytes signature are unchanged.
