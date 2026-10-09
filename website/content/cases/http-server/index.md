---
title: "Case: Building a Http server app"
---
# Case: Building a Http server app

A station client sends its name to a report service. The service validates the
request and acknowledges it. Follow the server and the client that connects to it. For individual
client capabilities, start with [HttpClient and HTTP](/features/web/).

| Step | Client | Server |
| --- | --- | --- |
| Prepare | Build a report containing the station name `Café` | Listen on loopback and accept an exchange |
| Submit | POST JSON to `/reports` with `application/json` content | Parse the body and require a string-valued `station` |
| Acknowledge | Check the status and read the acknowledgement | Return 201 with `{"accepted":true}` |
| Reject bad input | Handle the error status explicitly | Return 400 for invalid JSON/report data, or 404 for an unknown route |
| Finish | Report the result to the caller | Complete the response and close the exchange |

This case acknowledges a report; it does not save it. The downloadable project
contains the client, server, shared JSON/error code and an interoperability verifier.
Use the [setup guide](/try/) for the matching runtime and compiler.

<a id="accept-loop"></a>

## Server: accept requests in a loop

The listener and the application model are separate. This sample calls
`HttpServer.Listen("127.0.0.1", 0, 4)` once, then repeatedly awaits `Accept()`.
Each successful accept gives the application one `HttpContext`. `Configure`
chooses the response; `Complete` sends it and ends the exchange.

The loop below is extracted from the downloadable server:

```raven
{{HTTP_REPORT_ACCEPT_LOOP}}
```

This example handles exchanges sequentially and stops after a configured number
of attempts so the verifier can finish. Rejected protocol, limit and unsupported
requests count toward that limit and leave the server listening. Other failures
end the sample. Accept cleans up rejected exchanges; configuration failures close
the acquired context explicitly, and Complete closes it after sending or failure.
The program's completion callback closes the listener.

`Configure` is where this application chooses its model. The later routing and
attribute-mapper examples change dispatch inside the handler; they do not require
a different listening model. A long-running application also needs an explicit
shutdown policy. `Accept(token)` supports cancellation, while `server.Close()`
stops the listener and closes its owned exchanges; it does not drain handlers.
See [exchange ownership and limits](/features/web/#own-an-http-exchange).

## Server: validate and acknowledge

```raven
{{HTTP_REPORT_SERVER_SAMPLE}}
```

## Client: connect and send the report

```raven
{{HTTP_JSON_SAMPLE}}
```

The client program connects to the server URL supplied as its command-line argument:

```raven
{{HTTP_REPORT_CONNECT_SAMPLE}}
```

For example, pass `http://localhost:8080/` if the server reports port 8080. The server binds a free loopback port,
prints it, accepts the request and completes the response. The verifier connects
these two programs using that port; no public service is required.

The [application sample](/samples/http-json/http-json/Client.rvn) constructs a report
with the public System.Data.Json DOM and POSTs it to a neoCLR server. The server
uses HttpContext to parse the body and return a JSON acknowledgement with status
201. Bad JSON or report shapes return 400; unknown routes return 404.

Application-owned converters project HttpError and JsonError into AppError while
retaining the original causes through helpers and async methods. Async `Main` awaits the submission and converts errors to display text at its
reporting boundary. Ordinary steps use propagation. The
server handles invalid input explicitly where it chooses the HTTP response.

[Download the client/server sample](/samples/http-json.zip). It uses the runtime
library's JSON parser and includes checks against independent Python HTTP peers.
Serialization is synchronous; HTTP bodies are currently buffered. The provisional
DOM limits are 1,024 UTF-8 bytes, four container levels and 32 values. The
sample has an opt-in variant using the provisional [object serializer overloads](/docs/json.html).
It maps both report and acknowledgement models through checked runtime reflection:
constructors, getters and setters execute normally. Flat public `string`, `int` and
`bool` properties are supported, using exact property names. Writable properties
must be present; extra JSON fields are ignored. Nested models and typed arrays
are supported as shown below; null mapping and naming policies remain deferred. The sample's property names match its lowercase
wire names explicitly. Both mapped peers are checked against independent peers and each other.
Latency under load is not characterized.
The downloadable demo defaults to direct DOM mapping; its verifier selects this
experiment with `--mapped`.

## Run the client and server

[Download and extract the projects](/samples/http-json.zip). From that directory,
run with the matching runtime and SDK:

```sh
python3 http-json/verify.py \
  --toolchain-root /absolute/path/to/runtime \
  --sdk /absolute/path/to/matching/raven-sdk \
  --runner /absolute/path/to/runtime/tools/http-runner \
  --case all
```

The verifier builds the two .rvnproj projects, starts the server, reads its chosen
port and passes that URL to the client. It also tests each side against Python.
The client prints `{"accepted":true}`; the server prints `Reports served` after
completion. The default case uses DOM conversion and one POST in the paired run.
Use `--mapped` for the flat typed-model variant, which fetches then submits a report.

## Development case: a report with a nested station

The report
exchange carries a station name and a description of where the sensor is
installed and why staff use it, plus a `readings` array of integer measurements. The client first fetches
this model from GET `/report`, then submits it to POST `/reports`. The server
reads the nested station and returns the same 201 acknowledgement. This is still
an explicit HttpServer application. The routing case below uses a
route parser inside the handler, with named parameters and typed parsing;
WebApplication infrastructure remains deferred.

### Shared models

```raven
{{HTTP_NESTED_MODELS_SAMPLE}}
```

### Client: fetch and submit the model

```raven
{{HTTP_MAPPED_CLIENT_SAMPLE}}
```

`ReadReply` checks the status and reads the acknowledgement. The shared
application error union retains both HTTP and JSON causes.

### Server: read the nested request

```raven
{{HTTP_NESTED_READ_SAMPLE}}
```

The server callback calls this operation, serializes the acknowledgement with
`JsonContent.Create`, and completes the response. Invalid nested values such as
`{"station":{"name":12}}`, a missing name, or null produce 400; unknown paths
produce 404. In the development sample, protocol, limit and unsupported-request
errors close the rejected connection and leave the server available for the next
request. Rejected attempts count toward its configured request count. Header
rejection occurs before a context exists, so this path does not send a 400/431
response. See the [complete server](/samples/http-json/json-object-mapping/HttpServer.rvn).

### Run the paired case

[Download the complete client/server projects](/samples/http-json.zip), extract
them, and run from the extracted directory. Set these paths to a matching
matching runtime bundle and Raven SDK.

```sh
python3 http-json/verify.py \
  --toolchain-root /absolute/path/to/development/runtime \
  --sdk /absolute/path/to/matching/raven-sdk \
  --runner /absolute/path/to/development/runtime/tools/http-runner \
  --nested --case all
```

The verifier builds both projects, starts the server on a free loopback port,
checks it with Python requests, runs the neoCLR client/server pair, and checks the
client against a Python server. Expect 19 server cases, successful JSON client and
server reports, and a final `Public JSON DOM + HTTP checks passed: all` line with
an iteration count. The client response is `{"accepted":true}`. Individual server
or client checks can be selected with `--case server` or `--case client`.

Nested mapping allows 1,024 UTF-8 bytes per document, matching the buffered HTTP
body limit, and permits four object levels
including the root, counting arrays as containers. Typed arrays preserve order and
empty values; each array allows up to 31 items within the shared 32-value document
limit. Nulls, generic lists and polymorphic property/element values
remain unsupported. Shared child references are serialized as repeated objects;
cycles fail at the depth limit. See [JSON mapping rules](/docs/json.html) for
construction, error and stream ownership contracts.


## Development case: routing with typed parameters

A station is
addressed by ID: GET `/stations/42/reports` reads its report and POST to the same
path submits it. Station 42 is the small case's known sensor. The shared nested
JSON models above are unchanged.

Compile `RoutePattern.Parse("/stations/{stationId}/reports")` once. The fundamental
API matches a target and exposes named values; no route union is required:

```raven
{{HTTP_ROUTE_DIRECT_SAMPLE}}
```

An application-defined union is an optional convenience for dispatch. These cases
carry already parsed parameters; the fallback carries the original target:

```raven
{{HTTP_ROUTE_UNION_SAMPLE}}
```

Ordinary application code converts a match into its chosen route variant:

```raven
{{HTTP_ROUTE_PARSE_SAMPLE}}
```

### Server dispatch

The handler uses `match` to choose its own logic. Invalid parameters become 400;
unmatched paths, unknown stations and unsupported methods become 404 in this case.
The existing HttpContext completion and cleanup remain in the surrounding server.

```raven
{{HTTP_ROUTE_SERVER_SAMPLE}}
```

### Connecting client

The client reads the report and submits it back using the same station path:

```raven
{{HTTP_ROUTE_CLIENT_SAMPLE}}
```

Matching is case-sensitive; trailing slashes matter. Query text is ignored. Segment
values are decoded once as UTF-8, while encoded separators and malformed escapes
are rejected. Each target/pattern allows at most 1,024 UTF-8 bytes and 16 segments.
`GetInt32` checks spelling and bounds; other types can use `Get` and their own parser.
See the [route API guide](/docs/routes.html) for the complete contract.

The [project download](/samples/http-json.zip) includes `http-routing`, the shared
model source and verifier. From the downloaded projects, select the matching runtime and SDK:

```sh
python3 http-json/verify.py --routed --case all \
  --toolchain-root /path/to/runtime --sdk /path/to/raven-sdk \
  --runner /path/to/runtime/tools/http-runner
```

The HTTP checks cover route parsing and union dispatch, 29 independent server requests,
the neoCLR client/server pair and the client against an independent server. The
client prints `{"accepted":true}`. The examples are packaged sample source; they do not introduce a hosting framework.

<a id="development-experiment-attributed-item-routes"></a>

## Case: attributed item routes

A small catalog API can declare its routes on the union returned to the handler:

```raven
{{HTTP_ATTRIBUTED_ROUTES}}
```

`AppRoutesParser.Create()` inspects these attributes once before the server starts.
It validates the schema and retains compiled patterns, capture conversions and
constructors:

```raven
{{HTTP_ATTRIBUTED_STARTUP}}
```

`Parse(target)` returns `Result<AppRoutes, RouteMappingError>`.
NoMatch is separate from InvalidTarget, InvalidParameter and ConstructionFailed;
no fallback union case or unmatched attribute is needed. The handler owns method
selection, item lookup and response policy:

```raven
{{HTTP_ATTRIBUTED_HANDLER}}
```

The connecting client lists the catalog and reads item 42:

```raven
{{HTTP_ATTRIBUTED_CLIENT}}
```

The server returns 404 for unmatched routes or missing items, 400 for invalid route
input and 500 for construction errors. This small in-memory case uses fixed JSON
responses; the station case above demonstrates typed JSON.

The [source download](/samples/http-json.zip) includes `runtime-route-mapper`, its
three projects, reusable mapper, typed application facade and verifier. With a
matching runtime and SDK, run:

```sh
python3 runtime-route-mapper/verify.py --toolchain-root /path/to/runtime \
  --sdk /path/to/raven-sdk --runner /path/to/runtime/tools/http-runner
```

**Application-owned sample code; not installed as an SDK mapper.**
One public nongeneric union, up to 16 cases and String/Int32 payloads are supported.
Startup rejects invalid schemas and structurally overlapping patterns. Overlap
checks conservatively ignore typed conversion domains and the total target byte
limit. Requests still decode targets, allocate arguments and perform checked
constructor invocation. Attribute constructors never execute. Source generation
remains a possible future alternative. The ordinary [route primitives](/docs/routes.html)
remain independently usable.
