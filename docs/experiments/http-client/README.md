# neoCLR HTTP client sample

Development after Preview 9. This app imports `System.Web.Http` from the neoCLR
runtime library: HttpClient, HttpHandler, HttpSocketHandler, HttpRequest, HttpResponse,
HttpHeader and HttpContent. [Design and remaining gates](../../http-client-design.md).
The public API reference is available on the website under `/docs/`.

## Run

Use matching development runtime, Raven SDK, core reference, importer and System.neoil
artifacts. From the neoCLR repository:

```sh
cargo build --release --example measure_async
python3 docs/experiments/http-client/verify.py \
  --toolchain-root /path/to/development-bundle \
  --runner target/release/examples/measure_async
```

Python 3 and .NET 10 are required for the local peers and comparison client.
The verifier compiles the Raven app once and selects an ephemeral loopback port.
It replaces Main.rvn's port 19091 in a temporary copy. No external service is needed.
Keep Main.rvn, Handlers.rvn and HttpClient.rvnproj together. The published Preview 9
SDK cannot run this example. The optimized runner keeps verification practical;
this is not a performance benchmark.

Successful output:

```text
Handler checks passed
Other work runs while HTTP is pending
HTTP 200
Café 🌍
```

Forwarding handlers verify nested before/after order and error preservation. A fake
handler replaces transport. Invalid URLs are rejected before handler dispatch.
The controlled peer fragments headers and UTF-8 bytes, stays open until the client
closes, and exercises framing errors. Content-Length, rather than EOF, determines
successful completion. A small 256-object heap exercises collections across pending
operations and every run must finish with zero live managed objects. The trusted
fixture has an explicit instruction budget; runtime defaults are unchanged.
The external watchdog is a test guard, not an HTTP request deadline.

## Contracts and limits

`HttpClient()` selects HttpSocketHandler; `HttpClient(handler)` allows composition.
`Get(string)` parses a limited URL and delegates to `Send(HttpRequest)`.
Handlers return `Task<Result<HttpResponse, string>>`. A forwarding handler can act
before/after the inner handler's result; the socket handler owns connection cleanup.
Each socket-backed Send has separate buffers, pending tasks and a connection.
Custom mutable handlers must define their own concurrent-use policy; there is no
handler disposal, automatic retry or cancellation contract yet.

Only plain HTTP/1.1 GET with a 200 response and exactly one Content-Length is admitted.
Limits: 1,024 URL bytes, 2,048 header bytes, 16 fields, 1,024 body bytes. Header names
are ASCII tokens stored lowercase; values trim surrounding spaces/tabs. Invalid
framing, duplicate lengths, Transfer-Encoding and Content-Encoding fail explicitly.
Content.ReadText strictly decodes UTF-8 after the complete body is buffered. Public
response/content constructors retain their sequences; producers must keep them stable.

No chunking, TLS, redirects, connection reuse, arbitrary methods/statuses, request
body/headers, streaming content, charset selection or overall request deadline.
Send/Receive have no deadline. The socket path stops at Content-Length and closes;
it does not consume subsequent bytes. A 256-byte reusable buffer handles short transfers;
no throughput claim is made. This is not a general HTTP implementation.

## Integration observations

Async forwarding handlers exercise nested private-field access in the importer.
The runtime transport uses an internal continuation object, not generated async
state machinery in public contracts; this can change with runtime suspension.

Earlier application-local probes exposed unresolved compiler shapes: a source-defined
response in an async Task result was rejected with RAV2704 while a referenced response
worked; propagated field assignment left a receiver on an error-return stack; a hoisted
non-null Sequence local was cleared with null. The integrated library avoids those
shapes through callback methods and local-before-assignment. `<` comparisons with
`||` also exposed parser ambiguity, avoided with inclusive ranges. None is claimed
fixed; general compiler changes require independent .NET reproductions.

An additional compiler observation: the multi-part request interpolation did not
produce a usable value against this core surface. Explicit binary String.Concat
construction is used until a reduced compiler investigation establishes the cause.
The importer's Boolean argument adapter also loses private member scope; the private
header helper uses an integer normalization flag to avoid that adapter. Neither
workaround broadens private access or claims a compiler fix.

## Validation — 2026-09-24

On the local macOS development toolchain, all 18 controlled-peer cases and the Python
HTTP server case pass. The .NET 10 comparison receives the same fragmented greeting
and completes before peer EOF. Cases cover empty bodies, mixed-case/trimmed headers,
missing/duplicate/negative/overflowing lengths, unsupported status/encodings, bare CR/LF,
invalid names, header count/byte limits, body limits, truncated bodies and invalid UTF-8.
Every neoCLR run has multiple collections and zero live managed objects afterward.
The fragmented success allocates 359 objects with eight collections; Python HTTP server
success allocates 379 with eight. These are fixture observations, not throughput or
cross-platform claims. The compiler has no code changes in this slice.

### Stalled transfer checks — 2026-09-24

The current native backend bounds each nonempty Send/Receive to five seconds. Run
`verify.py` with repeated `--case` arguments to select `stalled headers`, `stalled body`
and `fragmented UTF-8`; the .NET baseline still runs. A peer keeps its write side open,
so timeout must reach the managed error/Close path without EOF. The verifier checks
zero final live objects. This is not a whole-request deadline; short progress can
restart the per-transfer budget. Full runs now include these two extra cases.
