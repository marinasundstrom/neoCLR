# A JSON report over HTTP

Development experiment after Preview 9. This composes the integrated HttpClient and
HttpServer APIs with the existing application-local JSON document experiment. It
adds no public JSON API and does not settle its future contract.

The server constructs and serves a report at `/report`:

```json
{"station":"Café","readings":[21,22.5]}
```

The client fetches it, decodes UTF-8, parses the document, explicitly reads the station
and first reading, and constructs a local acknowledgement:

```text
Station: Café
First reading: 21
{"station":"Café","accepted":true,"count":2}
```

The acknowledgement is displayed locally; it is not POSTed back. GET-only transport
is sufficient for this slice. This is a two-process application case demonstrating
DNS, sockets, HTTP framing, Task/Result, encoding, JSON values and GC together.

```sh
python3 docs/experiments/http-json/verify.py \
  --toolchain-root /absolute/path/to/matching/development/toolchain \
  --runner target/release/examples/measure_async
```

Requires Python 3, the development toolchain, and the trusted runner built with
`cargo build --release --example measure_async`. The published Preview 9 SDK does
not contain these HTTP APIs. The verifier builds temporary applications, selects
loopback ports and checks both a neoCLR pair and independent Python HTTP peers.
It parses produced JSON independently and checks collection and zero final live
objects with a 256-object heap. Process/instruction guards are test bounds, not
application request deadlines.

The JSON codec remains shared source from the adjacent json-message/json-document
experiments. Downloaded archives preserve those sibling directories. Its limits
remain 128 UTF-8 bytes, four nested containers and 32 values; duplicates are rejected
and number lexemes preserved. These are demonstration policies, not HTTP limits or
a proposed general serializer contract. HTTP retains the bounded GET/200,
Content-Length-only, no-TLS contract described by the Web guide.

Compared with .NET System.Text.Json, this experiment uses owned managed value objects
and explicit checked access rather than disposable document-backed elements. Its
cost is allocation and linear lookup. The existing JSON document research and
conformance cases remain the evidence for those choices; this slice tests their
composition with I/O, not a new JSON parser.

Local macOS validation: the Python-consumed server allocates 262 objects; the neoCLR
pair allocates 236 on the server and 312 on the client; the Python-served client
allocates 326. These runs record six or seven collections and zero final live objects.
They are lifecycle evidence, not a performance benchmark. Existing JSON conformance
checks were not rerun because the shared codec is unchanged.
