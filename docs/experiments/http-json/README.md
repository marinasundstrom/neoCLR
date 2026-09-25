# Public JSON DOM over HTTP

Development sample after Preview 9. It uses the integrated System.Data.Json,
HttpClient and HttpServer APIs, with no application-local JSON parser.

The client constructs `{"station":"Café"}` with JsonObject/JsonString, serializes it, and POSTs
UTF-8 application/json to `/reports`. The server owns the exchange through
HttpContext, parses the body, validates its shape, and returns status 201 with:

```json
{"accepted":true}
```

GET `/report` returns the example input. Malformed JSON, duplicate fields, invalid
UTF-8 and the wrong report shape receive a 400 JSON response; unknown routes get
404. This is deliberately a small report acknowledgement, not persistent storage
or a routing framework. The server requires a string-valued station field.

`Application.rvn` contains the shared DOM operations and an AppError union that
retains HttpError and JsonError causes. Local implicit error converters let `?`
propagate these causes through helpers and async methods. Both client and server
retain structured AppError results until the final process-reporting callback. The server deliberately handles invalid user input at its
HTTP boundary. Accept owns the request/response exchange; Complete sends and closes
it. A configuration failure explicitly closes the context before propagation.

Serialization remains synchronous and buffered. HTTP already buffers each body;
this sample uses string serialization plus UTF-8 content. Stream overloads and
borrowed-stream ownership are covered by the adjacent public json-dom fixture.
The provisional JSON limits remain 128 UTF-8 bytes, four container levels and
32 value occurrences; HTTP has its own independent bounds. No TLS, reflection,
object mapping, generic PostJson/GetJson helper or async JSON API is introduced.

```sh
python3 docs/experiments/http-json/verify.py \
  --toolchain-root /absolute/path/to/matching/development/toolchain \
  --runner target/release/examples/measure_async
```

Use matching development references, bridge, runtime and SDK. The verifier compiles
both applications, checks the server with independent Python requests, runs the
neoCLR client/server pair, and checks the client against a Python server. It checks
statuses, content type, UTF-8 byte lengths, JSON payloads and zero final live heap
objects. Process/instruction guards are test bounds, not request timeout policies.
Both programs accept Main(string[]) arguments: the client takes a base URL; the
server takes an optional number of requests to serve (default one).

Compared with .NET System.Text.Json + HttpClient, this POC keeps DOM access and
recoverable errors explicit and demonstrates separate HTTP/JSON error causes.
It does not add a framework-style JSON endpoint binder or claim equivalent scope.
See the public JSON DOM design for the existing standards and .NET comparison.

The pinned compiler requires some expressions to be written in separate steps.
Nested propagation inside a method argument can leave a receiver on the evaluation
stack. Combined type-pattern captures can fail importer definite-assignment checks.
The sample uses named values and separate type checks. Async methods live on small
application classes; top-level variants exposed RAV2704 during this integration.
These are compiler integration limitations, not intended API restrictions.

See [frozen compiler observations](limitations.md) for observed failures and the
workarounds used here; their root causes are not all isolated.

The initial larger report also carried readings; its acknowledgement echoed the
station and reading count. Both peers
passed independently, but the managed pair reached the provisional transport timeout.
A smaller acknowledgement alone still timed out. The final station-only report
and minimal acknowledgement pass the managed pair. This keeps the first integration focused; it is not evidence
that the larger case performs adequately. Timeout policy is unchanged. Use `--case server`, `--case pair`, or `--case client` to run only the relevant peer checks.

Final local validation (2026-09-25): all eight independent server cases, the managed
pair and the client against Python pass. The managed pair allocates 310 client and
309 server objects, with seven and six collections respectively; both finish with
zero live objects. The independent server batch and client also finish with zero
live objects. Website sample/download inputs and API snapshot were checked; the
website build was skipped as directed. These are correctness checks, not benchmarks.


## Reflected report investigation

An opt-in variant reuses [one reflected model](../json-object-mapping/README.md)
in the same endpoint. Pass `--mapped` to the verifier to compile HttpApplication.rvn
and HttpServer.rvn with Mapping.rvn. The default sample above remains DOM-based.
Both mapped peers pass against independent Python peers (12 server cases, plus the
client), with zero final live objects. The isolated managed pair also passes with
335 client and 336 server allocations and zero final live objects. An earlier pair
attempt reached the existing 15-second transport deadline while another test ran;
request latency under load remains unverified. This variant is not yet the default
demo. The standalone mapper passes string and MemoryStream round trips.
