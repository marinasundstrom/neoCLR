# JSON through strings and streams

Development experiment, 2026-09-25. This is the first I/O slice toward the provisional
System.Data.Json serializer, not a public class-library API or a reflection serializer.
The experiment-local `JsonSerializer` reuses the tested bounded JSON document codec.

## Contract exercised

- `Deserialize(string)` and `Deserialize(InputStream)` return the same JsonValue tree.
- `Serialize(JsonValue)` returns text; `Serialize(OutputStream, JsonValue)` writes UTF-8.
- StreamReader and StreamWriter implement the actual text/byte boundary. One-byte
  providers split multibyte characters and force repeated writes.
- Streams are borrowed on success and error. The serializer closes only its own
  leave-open wrappers. The caller controls flushing and closing the underlying stream.
- Input is read from the current position through EOF, up to 128 bytes. Detecting
  overflow consumes one extra byte. Failure does not restore the input position.
- Serialization validates/encodes the complete document before any output. An I/O
  error can still leave a prefix written; writes are not transactional or rolled back.
  Success means accepted writes, not flushed or durable storage.

Existing limits remain 128 UTF-8 bytes, four nested containers and 32 value occurrences.
Duplicate names are rejected, number spelling is retained, and strings decode strictly.
This is whole-buffer I/O, not incremental parsing, asynchronous I/O or cancellation.
The local Result error is temporarily a string, with Read/Write prefixes for I/O errors;
the public API needs structured errors retaining parse/encoding/stream causes. Do not
adopt these diagnostic strings as a public error contract.

[Main.rvn](Main.rvn)'s RoundTrip uses propagation for the ordinary path. Assertions
explicitly match errors where their behavior is the purpose of the test. The model
here is a JSON DOM, not an application object mapped through reflection.

## Comparison and provisional choice

Primary .NET API references checked 2026-09-25:
[JsonSerializer.Deserialize](https://learn.microsoft.com/en-us/dotnet/api/system.text.json.jsonserializer.deserialize?view=net-10.0)
and [JsonSerializer.Serialize](https://learn.microsoft.com/en-us/dotnet/api/system.text.json.jsonserializer.serialize?view=net-10.0)
provide both string and stream routes; the stream deserializer consumes one UTF-8
JSON value through completion. The documentation also includes prerelease material;
this experiment does not rely on newly introduced overloads. neoCLR uses its split
InputStream/OutputStream contracts and Result errors. The existing
[document comparison](../json-document/README.md#comparison-and-choice) covers RFC
8259, System.Text.Json, Serde and Json.NET tree/number/duplicate policies.

Reusing text wrappers tests actual platform layers without adding runtime instructions.
The cost is full-document buffering, repeated copies and synchronous execution. Direct
UTF-8 token processing could avoid those copies but would not exercise the selected
text APIs and requires more state. Neither performance nor general .NET serializer
compatibility is claimed. Public extraction and checked runtime reflection follow
this experiment; manually authored member mapping is not a substitute for reflection.

## Run and evidence

```sh
python3 docs/experiments/json-streams/verify.py \
  --toolchain-root /absolute/path/to/matching-development-bundle \
  --runner target/release/examples/measure_async
```

The focused verifier compiles the adapter/sample, checks its emitted JSON with Python,
and runs with a bounded managed heap. It also executes a .NET 10.0.100
MemoryStream/JSON baseline for shared cursor, overwrite, gap padding, EOF and borrowed
JSON stream ownership. It covers UTF-8 one-byte transfers, exact input
limit, oversized input, malformed JSON/UTF-8, input errors, partial output errors,
zero-progress writes, rejected cyclic output before I/O, and borrowed ownership.
This adds I/O coverage to the earlier parser suite; it does not rerun that whole suite.
The DOM also passes through the public development MemoryStream: write, seek to zero,
read and close. Its byte contract checks cover interface dispatch, range/bound errors,
zero-fill and closed operations; see [the design](../../memory-stream-design.md).

Validation on 2026-09-25: focused fixture passed with the frozen Raven compiler
SHA-256 `57b6c6e33727de470fe529ee4b5c2b8fb338aeb60d1d6405d9e515276dccd96a`
and matching neoCLR bridge/reference/library. Heap bound 512 objects; 663 allocated,
184 peak live, 13 collections and zero final live objects. This covers synchronous
managed retention, not pending native I/O. No website build or broad platform suite.
Pending-I/O GC, files and the public JSON DOM/error contract remain subsequent work.
Object deserialization/reflection comes after that DOM milestone, as directed.

Future HTTP conveniences can layer `GetJson<T>` and `PostJson` extensions over the
serializer and the existing HttpClient pipeline. Their signatures, response/error
policy, cancellation and stream-content ownership remain to be designed after the
serializer contract; these helpers are not implemented here.
