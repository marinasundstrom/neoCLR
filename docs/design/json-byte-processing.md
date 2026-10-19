# Avoid temporary text in JSON processing

Author direction, 2026-10-19: avoiding unnecessary string materialization is an API
requirement, including the JSON serializer. This began as a source audit. The byte-output portion is now implemented below;
borrowed byte-input parsing is also implemented; incremental reader/writer APIs remain planned.

The preceding [borrowed decoding change](../../benchmarks/http-server/serve/decode-slice-20261019/README.md)
removes temporary arrays when text is actually needed. JSON also needs to avoid
creating text merely to feed the next byte-oriented parser or output operation.

## Current conversion paths

| Path | Current implementation | Avoidable work to measure/remove |
| --- | --- | --- |
| Http.Json.JsonContent.Read/ReadNode | Borrow array storage; snapshot other sequences once; deserialize ReadOnlySlice<byte> | No whole-document string; general sequence snapshot remains |
| JsonSerializer.DeserializeNode(InputStream) | StreamReader.ReadToEnd, then borrowed string overload | Whole document materialization remains |
| DocumentReader.ReadString | Borrow token bytes; directly decode or unescape into owned result | Returned DOM strings remain necessary |
| JsonContent.Create/CreateNode | SerializeToUtf8Bytes | Owned output buffer |
| JsonSerializer.SerializeNode(OutputStream, ...) | Serialize bytes, then write | Bounded staging preserves validation before output |
| DocumentWriter.Append | Bounded UTF-8 byte storage | Final owned copy for byte-returning API |
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


## Borrowed parser implementation, 2026-10-19

The synchronous document and string-token readers now receive ReadOnlySlice<byte>
parameters. Their objects retain only cursor/count state; no borrowed descriptor
is stored on the heap. String input passes Utf8Bytes without encoding the complete
document again. Property names and string values skip the quoted-token string and
encoding array; unescaped tokens decode directly into final owned text. Escaped
tokens retain bounded scratch storage for Unicode escape/surrogate decoding.
Boolean/null recognition compares bytes, and number grammar borrows its owned
lexeme rather than encoding it again. DOM results retain owned text.

This follows System.Text.Json's byte-oriented parsing approach without changing
public signatures or introducing a public incremental reader. The benefit is fewer
temporary allocations; costs include passing views through parser calls and an
extra scan before the escaped-token fallback. HTTP and stream entry points still
decode complete text first. Direct byte input must preserve strict UTF-8 validation,
stream quota/read consumption and I/O error ordering before that boundary changes.
See [focused contracts and measurements](../../benchmarks/json-byte-input/20261019/README.md).

The round-trip probe reduces allocations 32.5% but increases mean CPU 31.6%.
Short stack samples are dominated by GC/root processing in both builds. The
allocation benefit is not a speedup; investigate root/boundary and checked-view
costs before expanding performance claims.


### Safepoint follow-up

The first timing used every-boundary synchronous collection. Explicit policy
comparison confirms 31.5% more root publications/collections with the initial
borrowed parser, and a remaining CPU regression under pacing. The next bounded
correction reuses the first unescaped-token scan, caches view lengths, rejects
impossible keyword lengths before comparison and reuses explicit token windows.
Escape/control validation and native safety checks stay intact. See
[policy-separated evidence](../../benchmarks/json-byte-input/20261019/gc-followup/README.md).


The corrected parser passes all 90 focused executions and resolves the regression
in the fixed round-trip probe: 36.7% fewer allocations and 9.1%/10.0% lower mean CPU
under paced/every-boundary synchronous GC versus the original parser. No policy
change or HTTP throughput improvement is implied. Whole-document stream/HTTP input
decoding remains the next integration boundary.

## Borrowed public input and HTTP integration (2026-10-19)

JsonSerializer now accepts ReadOnlySlice<byte> for node, generic and TypeInfo-based
deserialization. The returned DOM/objects own their text; no source window escapes.
The 1024-byte limit is checked before strict UTF-8 validation, which precedes JSON
syntax and mapping. This preserves buffered HTTP's malformed-text precedence,
including invalid bytes following an earlier syntax error. The validator follows
[RFC 3629 section 4](https://datatracker.ietf.org/doc/html/rfc3629#section-4),
rejecting overlong sequences, surrogate encodings and scalars above U+10FFFF.

HTTP reads borrow existing byte arrays and snapshot other Sequence<byte> storage
once. Unbuffered content is rejected without consuming its stream. Compared with
.NET's ReadOnlySpan<byte> overload family, the deliberate difference is neoCLR's
existing typed JsonError and bounded document contract; the borrowed input/owned
result lifetime model is the same. Stream input still materializes whole text.
See [the HTTP input probe](../../benchmarks/json-http-input/20261019/README.md)
for allocation and synchronous-collector measurements, including the cost of full
UTF-8 validation. This does not establish server throughput or concurrent-GC gains.
