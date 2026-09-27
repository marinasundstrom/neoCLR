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
retaining the original causes through helpers and async methods. The completion
callback converts errors to display text at its reporting boundary. Ordinary steps use propagation. The
server handles invalid input explicitly where it chooses the HTTP response.

[Download the client/server sample](/samples/http-json.zip). It uses the runtime
library's JSON parser and includes checks against independent Python HTTP peers.
Serialization is synchronous; HTTP bodies are currently buffered. The provisional
DOM limits are 128 UTF-8 bytes, four container levels and 32 values. The development
sample has an opt-in variant using the provisional [object serializer overloads](/docs/json.html).
It maps both report and acknowledgement models through checked runtime reflection:
constructors, getters and setters execute normally. Flat public `string`, `int` and
`bool` properties are supported, using exact property names. Writable properties
must be present; extra JSON fields are ignored. Preview 10 does not support nested models, null mapping or
naming policies; the development nested case below extends the model shape. The sample's property names match its lowercase
wire names explicitly. Both mapped peers are checked against independent peers and each other.
Latency under load is not characterized.
The downloadable demo defaults to direct DOM mapping; its verifier selects this
experiment with `--mapped`.

## Run the client and server

[Download and extract the projects](/samples/http-json.zip). From that directory,
run with the matching Preview 10 runtime and SDK:

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

**Requires a matching development build after Preview 10.** The same report
exchange can now carry `{"station":{"name":"Café"}}`. The client first fetches
this model from GET `/report`, then submits it to POST `/reports`. The server
reads the nested station and returns the same 201 acknowledgement. This is still
an explicit HttpServer application; WebApplication and automatic endpoint binding
are planned separately.

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
produce 404. See the [complete server](/samples/http-json/json-object-mapping/HttpServer.rvn).

### Run the paired case

[Download the complete client/server projects](/samples/http-json.zip), extract
them, and run from the extracted directory. Set these paths to a matching
**development** runtime bundle and Raven SDK; the Preview 10 runtime supports
only the flat variant.

```sh
python3 http-json/verify.py \
  --toolchain-root /absolute/path/to/development/runtime \
  --sdk /absolute/path/to/matching/raven-sdk \
  --runner /absolute/path/to/development/runtime/tools/http-runner \
  --nested --case all
```

The verifier builds both projects, starts the server on a free loopback port,
checks it with Python requests, runs the neoCLR client/server pair, and checks the
client against a Python server. Expect 15 server cases, successful JSON client and
server reports, and a final `Public JSON DOM + HTTP checks passed: all` line with
an iteration count. The client response is `{"accepted":true}`. Individual server
or client checks can be selected with `--case server` or `--case client`.

Nested mapping retains the 128-byte JSON bound and permits four object levels
including the root. Nulls, typed collections and polymorphic property values
remain unsupported. Shared child references are serialized as repeated objects;
cycles fail at the depth limit. See [JSON mapping rules](/docs/json.html) for
construction, error and stream ownership contracts.
