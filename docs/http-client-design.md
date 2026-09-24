# HTTP client pipeline — development experiment, 2026-09-24

The immediate case is a Raven application fetching a UTF-8 greeting over neoCLR TCP.
The author asks for a provisional client close to the [networking proposal](proposals/network-api.md),
with handlers attaching behavior to the request/response pipeline. The executable
[experiment](experiments/http-client/README.md) keeps HttpClient, HttpRequest,
HttpResponse and HttpContent separate from transport. They are now provisional System.Web.Http runtime-library APIs with generated reference
coverage. This is not a completed web application milestone.

## Comparison and provisional choice

.NET 10's [HttpMessageHandler](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpmessagehandler?view=net-10.0)
is the transport/behavior boundary; [DelegatingHandler](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.delegatinghandler?view=net-10.0)
composes an inner handler. The sample adopts that pipeline concept with an interface:
`Send(HttpRequest) -> Task<Result<HttpResponse, string>>`. A forwarding handler can
act before dispatch and after the outcome, while a terminal handler supplies the
response. Tests can replace the terminal handler. An interface is enough for this
probe; an abstract base and convenience delegation class remain open. Handlers do
not themselves imply a new scheduler or thread per request.

Hardwiring sockets into HttpClient would be smaller initially but couples mocking and
future host transports to client policy. A complete .NET-like handler stack now would
require premature decisions about cancellation, concurrency, disposal and streaming.
The small interface preserves the boundary at the cost of explicitly provisional
lifetime/error contracts. Returning Result adapts expected failures to neoCLR's style;
string errors are temporary and remain an explicit API evolution question.
Automatic retry requires request replayability and idempotence decisions; none is added.

The byte parser uses [RFC 9112](https://www.rfc-editor.org/rfc/rfc9112.html) as the
framing baseline, but deliberately accepts only GET/200, strict CRLF and one bounded
Content-Length. It rejects unsupported framing instead of guessing. This subset
cannot be described as a general compliant HTTP/1.1 client. UTF-8 decoding happens
after the complete byte body; character count is never used as the wire length.

.NET's [HttpCompletionOption](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpcompletionoption?view=net-10.0)
distinguishes headers-only and buffered completion. The sample buffers the entire
small response before Send completes. ReadText keeps the proposed Task contract even
though decoding currently completes immediately. The .NET comparison uses
ResponseHeadersRead and passes one five-second cancellation token through both send
and body reading. The Raven sample has separate DNS/connect bounds, but no total
request or transfer deadline. A verifier watchdog does not close that gap.

Primary sources above reviewed 2026-09-24. No performance equivalence is claimed.

## Integration and validation

The verifier compares a .NET client against the same controlled loopback peer, then
checks Raven success, early EOF and ambiguous framing. The earlier application-local decoder probe exercised limits, malformed lines/lengths,
unsupported encodings and invalid UTF-8; integrated coverage must exercise the transport
itself rather than rely on that earlier probe. Handler checks cover
nested order, error preservation, transport replacement and rejection before dispatch.
Managed objects must be fully reclaimed after completion under a small heap limit.

Generated async states nested in handlers require legal access to private containing
fields. The importer now admits callers nested within the declaring type, matching
[CLR-facing nested-type behavior](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/nested-types).
Cecil-level checks retain rejection for unrelated private-field callers and nested
constructors writing a containing type's readonly field. This is an importer access
fix; it does not broaden readonly writes or introduce runtime suspension. The sample
README records remaining compiler limitations and the source-level workarounds.

## Next gates

1. Add the author-selected HttpServer class, then a two-application text/JSON exchange.
   Test against independent peers as well as the neoCLR client.
2. Define request/transfer cancellation and deadlines at the existing operation-owner
   boundary, without exposing generated state-machine mechanics in HTTP contracts.
3. Decide which additional statuses, methods, request headers and content shapes the
   demo actually needs. Establish ownership/concurrent handler reuse before expanding the contracts.
4. Evolve the provisional System.Web.Http contracts with matching XML/API coverage.
   Continue encoding and stream integration as concrete body cases require it.

TLS, pooling, retries and a complete Uri abstraction remain separate future work.


## Lower-level implementation boundary

The author directs that managed HTTP APIs expose useful lower-level behavior and reuse
native facilities where appropriate, then explicitly accepts implementing HTTP over our
Socket API. The current transport reuses native DNS/TCP and their operation ownership;
HTTP framing is managed Raven code. A future host HTTP provider can implement HttpHandler
without changing client request/response consumption. That alternative could reuse TLS,
proxying and platform policy, at a portability/behavior-consistency cost. No such provider
is implemented, and this slice does not select a native dependency. After HttpClient
validation, HttpServer is the next author-selected API, with independent-peer tests to
avoid client/server agreement hiding a shared protocol mistake.

The transport uses a private continuation object with per-request state. It avoids
exposing TaskQueue, generated states or a Scheduler in the HTTP API. Completion closes
the socket and clears pending task references; full body buffering keeps response
lifetime independent of the transport. Buffering simplifies this POC but bounds body
size and adds copies. A 256-byte reusable transfer buffer avoids a task for every byte, while fragmented
peer input exercises partial transfers. No throughput benchmark is claimed. Public response/content constructors retain sequences, so
read-only interfaces do not promise immutable payload storage.

## Error unions — candidate, 2026-09-24

The author asks what Get's error union should look like, noting that union cases can
carry other union values and that copying .NET's exception hierarchy is unnecessary.
The current implementation still returns string errors. The following is a design
candidate, not shipped types or a frozen case list:

| Candidate HttpError case | Purpose |
| --- | --- |
| InvalidRequest(HttpRequestError) | Invalid URL or request input, before dispatch |
| Transport(HttpTransportError) | A failure obtaining or using the underlying connection |
| InvalidResponse(HttpResponseError) | Malformed framing or premature end of the response |
| Unsupported(HttpFeature) | A valid protocol feature this provider cannot process |
| LimitExceeded(HttpLimit) | A named configured/resource bound, such as headers or body |

For the socket provider, HttpTransportError could contain NameResolution(DnsError),
Connect(SocketError), Send(SocketError) and Receive(SocketError). This preserves a
concrete failure stage and a typed cause without forcing the common HttpError union
to flatten every socket or resolver case. It also couples that particular nested
surface to the provider; evaluate a provider-neutral category plus optional diagnostic
cause if another transport makes socket-shaped causes misleading. Do not require a
native HTTP provider to invent a SocketError.

Nest only where callers benefit from inspecting the cause. InvalidResponse can start
with the framing distinctions needed by these tests, not a case for every parser
branch or explanatory string. Unsupported is deliberately separate from malformed:
a valid chunked response is unsupported by this POC, not invalid HTTP. LimitExceeded
must identify which bound was hit; readable messages remain diagnostics, not codes.

HTTP status is a response outcome, not a transport failure: 404/500 should remain
HttpResponse values as status coverage expands. The current 200-only restriction
would be an Unsupported case, not an error category for each status. A future explicit
success-status helper can introduce application policy. UTF-8 failure belongs to
HttpContent.ReadText's own typed decoding error; Get handles bytes and should not fail
merely because they are not UTF-8. JSON parsing likewise belongs to its consumer.

Timeout/cancellation cases need alignment with Task and operation ownership before
adding another competing outcome channel. Existing nested DnsError/SocketError causes
already carry their native timeout/cancellation cases; there is no overall HTTP timeout
today. Runtime Faults remain distinct from expected errors, and programming-contract
violations should not automatically become retryable transport cases.

.NET 10 provides categories through
[HttpRequestError](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httprequesterror?view=net-10.0)
and exposes them on
[HttpRequestException](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httprequestexception?view=net-10.0),
alongside exception details. Borrow the useful category distinctions without adopting
exception inheritance. Primary sources reviewed 2026-09-24. Compare this nested design
with a flatter operation-oriented union using the client/server cases before replacing
string errors. Promotion requires reference/compiler support for nested payload unions,
executable matching/propagation tests, API documentation and a migration note.

### Integrated validation result

Local macOS validation on 2026-09-24 passes 18 controlled-peer cases plus Python's
HTTP server, with the same app importing System.Web.Http. All runs finish with zero
live managed objects under a 256-object limit. The .NET 10 baseline confirms fragmented
UTF-8 and Content-Length completion before EOF against the controlled peer. Private
field access checks pass, the API snapshot validates, and the combined site builds
961 pages with 17 website tests passing. These are local development results, not a
release or a cross-platform test claim. The remaining compiler/importer workarounds
are listed in the sample README rather than presented as resolved general fixes.

## BaseAddress after Uri — author direction, 2026-09-24

The author wants basic .NET HttpClient conveniences, including a parameterless
constructor and BaseAddress, with Uri implemented first. HttpClient() already selects
HttpSocketHandler in this POC. BaseAddress is not implemented. Add it only after Uri
has a deliberate absolute/relative representation and base-resolution behavior; do not
approximate resolution by concatenating strings. Compare .NET's
[BaseAddress contract](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.baseaddress?view=net-10.0)
when implementing it, including absolute-base validation, relative requests and policy
for changing client configuration after work starts. Uri precedes that convenience,
not the already-selected minimal HttpServer case.
