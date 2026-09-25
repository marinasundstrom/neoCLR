# HTTP client pipeline — development experiment, 2026-09-24

The immediate case is a Raven application fetching a UTF-8 greeting over neoCLR TCP.
The author asks for a provisional client close to the [networking proposal](proposals/network-api.md),
with handlers attaching behavior to the request/response pipeline. The executable
[experiment](experiments/http-client/README.md) keeps HttpClient, HttpRequest,
HttpResponse and HttpContent separate from transport. They are now provisional System.Web.Http runtime-library APIs with generated reference
coverage. This is not a completed web application milestone.

## Core request contract — author direction, 2026-09-25

The integration target is:

```text
Send(request: HttpRequest, cancellationToken: CancellationToken)
    -> Task<Result<HttpResponse, HttpError>>
```

`Send` is the fundamental client operation. `Get` and later verb helpers construct
requests and delegate through it; `GetString` additionally reads/decodes the response
body. String and Uri overloads share request construction and BaseUri resolution.
The handler boundary must carry the same token so decorators and replacement
transports participate in cancellation. Convenience methods must not bypass that
pipeline or create a separate transport implementation.

This follows the operation split in .NET 10's
[SendAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.sendasync?view=net-10.0)
and [GetStringAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.getstringasync?view=net-10.0)
(reviewed 2026-09-25), adapting expected failures to Result and retaining neoCLR's
Send naming. .NET's GetStringAsync also rejects non-success status codes; neoCLR's
convenience-method status policy still needs an explicit choice and test. A received
HTTP error response and a transport failure must remain distinguishable.

**Implementation gap:** the current client and handler expose tokenless
Send with HttpError results. Tasks have a cancellation outcome and invocation-local CancellationToken foundations
now exist, but the HTTP/native adapters do not carry tokens yet. The new signature is a target, not a shipped overload.
Define cooperative cancellation at the operation-owner boundary, including behavior
before dispatch, during DNS/connect/transfer, completion races and socket cleanup.
Decide explicitly whether observed cancellation uses the task cancellation outcome
or an HttpError case; do not silently provide both channels for the same event.
This does not require implementing runtime suspension or a new scheduler now.

Validation must exercise a replacement handler, token forwarding through decorated
handlers, Get/GetString delegation, typed error preservation, decoding failures,
pre-cancelled requests and cancellation of an in-flight exchange with resources
reclaimed. Keep URI/error integration moving; the broader cancellation infrastructure
is not a reason to expand into unrelated task redesign.

## Optional base address — author direction, 2026-09-25

Keep the current `BaseUri` spelling for the property, now integrated with `Option<string>`
(the library's optional-value type), rather than requiring a Uri object as
configuration. The author described this as `BaseUrl/BaseUri: Optional<string>`.

| BaseUri | Verb-method address | Construction |
| --- | --- | --- |
| None | Absolute URI | Validate and use the supplied absolute address |
| Some(absolute base URL) | Relative URL | Parse and resolve against the base |

For example, base `http://localhost:8080/api/` and relative `items` resolve to
`http://localhost:8080/api/items`. With no base configured, callers supply the full
`http://localhost:8080/api/items`. A relative address without a base is an error.
With a base configured, the expected input is relative; do not add an implicit
absolute-address override as part of this slice. This supersedes the earlier
proposed absolute-override test. Retain the requested string/Uri overloads for
addresses, applying the same addressing rules to each.

Use the existing Uri parser/resolver internally rather than string concatenation.
Trailing-slash, parent-path, root-relative and query-only resolution need explicit
samples and tests. Invalid base configuration or a mismatched address kind must be
reported before handler dispatch. The implemented validation/error choices are recorded below; malformed base text
is reported at request construction without silent fallback.

.NET's [BaseAddress](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.baseaddress?view=net-10.0)
uses a Uri property. This planned adaptation keeps string configuration convenient
and models absence explicitly, at the cost of validating text at the HTTP boundary.
The user-directed addressing rule is narrower than .NET's ability to accept an
absolute request address even with a base configured.

## Comparison and provisional choice

.NET 10's [HttpMessageHandler](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpmessagehandler?view=net-10.0)
is the transport/behavior boundary; [DelegatingHandler](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.delegatinghandler?view=net-10.0)
composes an inner handler. The sample adopts that pipeline concept with an interface:
`Send(HttpRequest) -> Task<Result<HttpResponse, HttpError>>`. A forwarding handler can
act before dispatch and after the outcome, while a terminal handler supplies the
response. Tests can replace the terminal handler. An interface is enough for this
probe; an abstract base and convenience delegation class remain open. Handlers do
not themselves imply a new scheduler or thread per request.

Hardwiring sockets into HttpClient would be smaller initially but couples mocking and
future host transports to client policy. A complete .NET-like handler stack now would
require premature decisions about cancellation, concurrency, disposal and streaming.
The small interface preserves the boundary at the cost of explicitly provisional
lifetime/error contracts. Returning Result adapts expected failures to neoCLR's style;
the initial string errors have now been replaced by the development HttpError union.
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
and body reading. The Raven socket handler now also has a shared 15-second exchange budget, retaining
shorter DNS/connect/transfer bounds. This does not yet cover arbitrary custom handlers. A verifier watchdog does not close that gap.

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

1. The [HttpServer POC](http-server-design.md) now supplies the greeting exchange,
   checked with neoCLR and independent peers. The [JSON report](experiments/http-json/README.md)
   now composes the existing application-local JSON consumer with these APIs.
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

## Stalled transfer checkpoint — 2026-09-24

The private socket operation owner now applies a five-second bound to each nonempty
Send/Receive. The existing HTTP client/server error paths close their owned connection
when the operation expires; string errors currently retain the stage, not the typed
SocketError cause. Stalled response headers/body and partial request headers are
integration cases. No managed timer race, new HTTP signature or public scheduler is
introduced. See the [socket policy and comparisons](socket-api-design.md#pending-transfer-deadline--2026-09-24).
A single request budget, handler cancellation and slow-trickle protection remain open.

## Shared native deadline checkpoint — 2026-09-24

The private resolver/socket owners now accept an optional absolute monotonic deadline.
Each operation uses the earlier of its existing phase bound and that shared deadline.
Already expired valid submissions reject before native work; retries and short I/O do
not reset it. The [native loopback probe](experiments/request-budget/README.md) carries
one deadline across lookup, connect and successive body reads, with deterministic
expiry and existing cleanup. This is implemented native infrastructure, **now wired
through the private Raven HTTP bridge** as described below. Public Socket calls keep
their independent phase bounds.

This reuses the existing .NET HttpClient cancellation comparison. Additional primary
contracts checked 2026-09-24:

- .NET 10 [CancellationTokenSource.CancelAfter](https://learn.microsoft.com/en-us/dotnet/api/system.threading.cancellationtokensource.cancelafter?view=net-10.0)
  resets its countdown on another call while not yet cancelled. Reusing a source with
  one initial cancellation time can express a whole budget; restarting a full delay
  for each transfer cannot. Absolute expiry makes that distinction explicit internally.
- [Task.WaitAsync](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.task.waitasync?view=net-10.0)
  gives a wait that completes with the task, timeout or cancellation. That waiting
  contract alone does not establish native buffer quiescence; neoCLR keeps settling
  and releasing buffers at their existing operation owner.
- Tokio 1.53.1 [timeout_at](https://docs.rs/tokio/1.53.1/tokio/time/fn.timeout_at.html)
  uses an absolute instant, but permits an immediately completed future to succeed
  regardless of the deadline. The neoCLR private path deliberately rejects expired
  admission and gives expiry precedence over unobserved native readiness. This is a
  stricter policy with the cost of rejecting ready work after scheduling delays, not
  a claim of equivalent semantics or a general improvement.

No new scheduler, task state or managed timer race is needed for the native path.
The next bridge slice must give HttpExchange one owner-scoped deadline and pass it to
DNS/connect/send/receive. A private opaque handle or checked time representation
remains an implementation choice; do not expose host Instant through public metadata.
A late successful connect must still be closed if the exchange has expired, and a
completed response must not become success after the request budget has expired.
Expiry cannot preempt arbitrary guest code or force a blocking host resolver to stop.
Custom handler pipelines, accepted server connections and accept waiting need explicit
cancellation ownership before claiming a general HttpClient/HttpServer timeout API.

## Socket-backed HTTP exchange budget — 2026-09-24

HttpExchange now creates one private monotonic deadline immediately before lookup.
The default budget is 15 seconds. DNS, fallback connect and each send/receive receive
that same deadline and retain the earlier five-second phase limit. The adapter consumes
each native outcome before checking expiry, closes a late successful connection, and
checks again before reporting a buffered response. Expiry returns the provisional
string `Request deadline exceeded` and closes the exchange's connection. Short body
or header progress cannot renew the budget. Completed native outcomes retain their
original result; the enclosing HTTP exchange can still expire before delivery.

The private stamp is a checked process-monotonic millisecond offset, rounded upward
when created. It is not a wall-clock time, public value type, persistent identifier or
resource lease. It allocates no registry entry or managed root. Deadline services
remain native SocketIo helpers; DNS Until submission still requires NameResolution
and TaskDispatch, while socket Until submissions require SocketIo and TaskDispatch.
The new private ABI needs matching native runtime, reference, importer and library.

Bootstrap references temporarily make only the four cross-slice Until methods public
for Raven compilation. Normal application references keep them internal. Import
validation restores the internal contract, and the socket catalog permits internal
signatures only in library-import mode. The dedicated visibility check rejects
application imports and checks that generic signature matching stays public-only by
default. No Raven compiler semantics, public scheduler or cancellation-token API changes.

This is a socket-handler budget, not all of .NET HttpClient.Timeout: URL/request
construction and custom handlers before/after transport are outside it. Server accept
and application response tasks are also outside it. Non-yielding guest code and an
unpumped callback queue are not preempted. The 15-second fixed choice is provisional
and may reject slow legitimate exchanges. A complete configurable client/server
lifetime contract still needs cancellation ownership and structured errors.

Validation uses peers that send body bytes every three seconds or header bytes every
second. Each would keep the five-second transfer timeout alive; the shared exchange
must instead fail and close around its original deadline. A fragmented success case,
a silent-header case, handler checks and independent .NET baseline guard existing
behavior. These are lifecycle tests, not throughput measurements.

Local validation passes: trickling response body and headers both end with the request
expiry error and zero live objects (nine and eleven collections). Silent headers still
use the shorter phase error, and ordinary fragmented UTF-8 succeeds. The independent
.NET client still consumes the neoCLR server. All 32 socket, nine resolver, one clock
stamp and nine service-analysis tests pass, as do the bootstrap/application visibility
checks. The API snapshot and 967-page website validate; all 17 website tests pass.

## Uri, HttpError and BaseUri priority — 2026-09-24

The author now explicitly prioritizes these three contracts, using BaseUri as the
current spelling and retaining both string and Uri address overloads. This supersedes
the earlier BaseAddress name and defers the next cancellation exploration. The first
[Uri slice](uri-design.md) supplies managed parsing, typed UriError and string/Uri
relative-resolution overloads. It does not yet change HttpClient's string results or
absolute-URL-only request factory. HttpError and BaseUri are the following integration
slices. URI/URL encoding utilities are author-directed later work, not implicit
escaping in Parse. Preserve the error-union comparisons above when selecting typed
HTTP failures; do not turn every diagnostic message into an unrelated case.

The author's subsequent direction makes standard Raven union syntax the default,
including authored members. Establish that path before implementing HttpError; do
not extend the manual carrier catalog by default. The
[direct-union investigation](experiments/http-error-unions/README.md) now executes
nested standard declarations, empty-case-only unions and separately compiled
dependencies through the bridge. An empty-case bootstrap fragment also executes
against a matched core reference. Mixed legacy carrier defaults and production
reference/consumer integration remain open. This is not a shipped HTTP error
contract or a fixed physical ABI. Existing nested-error comparisons still guide the eventual public cases.

## Typed error integration — 2026-09-25

The public client, request factory, handler and server contracts now use HttpError
from normal Raven union source. NameResolution(DnsError) and Transport(SocketError)
preserve native causes. InvalidUri(UriError) prepares URI integration; InvalidRequest,
Protocol, Unsupported and LimitExceeded carry diagnostic text. TimedOut denotes the
shared exchange deadline, while Handler carries an application failure. Status codes
remain response data; this slice does not expand the GET/200 protocol subset.
ReadText is deliberately separate and retains its provisional decoding-error string.

This adapts .NET's exception-based failure surface into typed Result cases without
copying an exception inheritance tree. Inspecting a cause no longer requires parsing
text, at the cost of larger generated union values and source/artifact migration.
No stack-trace capture or future Error interface is added. The reference seed is
replaced by compiled union metadata; consumer case binding follows that projected
family rather than a second handwritten carrier layout. Inactive defaults are not
meaningful failures. No new Runtime Contract option or Raven compiler emission rule
is required. HTTP token wiring remains following work; BaseUri integration is recorded below.

Validation: the public handler sample checks nested causes, inactive defaults,
copies and boxed payloads under allocation churn. Fragmented UTF-8, early EOF,
ambiguous framing, body bounds, invalid UTF-8, stalled transfer, shared deadline
and an independent Python HTTP server pass; measured client runs finish with zero
live objects (fragmented success: 486 allocations, ten collections). The JSON
application passes with independent peers. Signature checks reject changed payload
and extraction types and private formatter calls. Payload-projection mismatch and
overlapping-layout negative probes remain passing. Clean regeneration of HttpError
and HttpClient matches; API reference checks pass. The site build completed before
the author's instruction to skip future per-slice website builds.


## BaseUri and address overloads — implemented 2026-09-25

`HttpClient.BaseUri` is now `Option<string>`, defaulting to None. Get(string) and
Get(Uri) share request construction, then call Send through the existing handler.
`HttpRequest.Get(Uri)` complements its string factory; both require an absolute URI
and apply the same bounded HTTP policy. String syntax failures preserve their UriError
inside HttpError.InvalidUri. This changes malformed escape/control-character failures
from the older ad hoc request-parser errors to structured URI errors.

Base text is validated at request construction, not assignment. Some requires relative
references; None requires absolute ones. A network-path reference (`//other.test/path`)
is rejected with a configured base: although it is syntactically relative, RFC resolution
would replace the authority. This bounded choice keeps base-address selection explicit;
it costs the ability to use RFC authority replacement through Get. Callers can construct
an absolute request and pass it to Send, which does not consult BaseUri.

The existing Uri resolver supplies trailing-slash, parent-path, root-path, query-only
and empty-reference rules. The HTTP factory now recognizes an authority ending at `?`,
so `http://example.test?x=1` produces `/?x=1`. Fragments remain unsupported rather than
silently removed. A malformed or unsupported base is rejected even when the reference
would otherwise discard its path. Some/None assignment is accepted; an inactive default
Option faults as contract misuse.

.NET 10 [BaseAddress](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.baseaddress?view=net-10.0)
was reviewed again on 2026-09-25. It uses a Uri, rejects a relative base on assignment
and prevents changing configuration after requests start. neoCLR's provisional property
keeps author-requested string configuration, reports text errors through Get's Result,
and allows changes between calls. Request construction snapshots configuration before
handler dispatch. The benefit is explicit recoverable validation and simple configuration;
the cost is repeated parsing and no general thread-safe mutation contract. The existing
invocation ownership rules apply.

The focused [address fixture](experiments/http-base/README.md) compares ten resolution
cases against .NET, exercises both public overloads, validates ten rejected pairs before
handler invocation, clearing BaseUri and direct Send. The lexical Uri policy deliberately
preserves `%2e%2e` while .NET normalizes those encoded dots. This is not a filesystem
containment guarantee. The network sample also uses BaseUri with an independent Python
server. GetString, token forwarding, native cancellation and configurable timeout remain
pending; this is one part of release slice 4, not completion of that slice.

## HTTPS feasibility checkpoint — 2026-09-25

The current socket2 backend supplies TCP; neither the managed HTTP code nor Cargo
dependencies contain a TLS engine. Recommendation: add TLS below the HTTP framing
layer, retaining the managed request/response pipeline. Do not implement cryptography
in Raven or introduce an unrelated native HTTP client that bypasses these contracts.

[Rustls 0.23 documentation](https://docs.rs/rustls/latest/rustls/), reviewed 2026-09-25,
describes a TLS engine independent of network I/O, with explicit encrypted input/output,
plaintext access and certificate validation using configured trust roots. That makes a
host-side adapter over the existing socket owner plausible. This is a feasibility
inference, not a tested neoCLR integration or a dependency selection. Provider/platform
support, trust-root loading and packaging must be evaluated against the release targets.

An executable client spike must establish: one budget across connect/handshake/transfers;
hostname and certificate-chain verification; bounded ciphertext/plaintext buffers;
cancellation acknowledgement and close while handshake/read/write are pending; and
interoperability with an independent HTTPS server. Do not expose a certificate-validation
bypass to make a demo work. An OS TLS backend is an alternative with platform-specific
adapter/validation costs. Server certificates/configuration are a separate scope decision.
The release HTTPS decision remains open until this evidence exists, before the application
slice as required by the roadmap. This checkpoint does not add HTTPS support.


## Native cancellation prerequisite — 2026-09-25

The [native acknowledgement checkpoint](cancellation-design.md#native-operation-acknowledgement--implemented-2026-09-25)
adds bootstrap-only DNS/socket operation cancellation hooks and pending connect/accept
cancellation. Ready outcomes survive late requests; operation slots survive until
completion delivery/result consumption. Blocked DNS work retains its host capacity
permit even when guest delivery is cancelled. The managed adapters and HttpHandler
still need token forwarding and registration disposal. No token-aware Send/Get or
GetString overload is claimed by this checkpoint.


### Managed networking cancellation prerequisite — 2026-09-25

DNS and sockets now accept CancellationToken on public operations and internal
shared-deadline paths. They dispose registrations before consuming native results,
and acknowledge a winning native cancellation through Task cancellation. Existing
HTTP calls remain tokenless. The next step is forwarding the token through HttpHandler
and HttpExchange, observing cancelled child tasks without calling GetResult, closing
the exchange-owned socket, then exposing terminal HTTP cancellation. The public
Send/Get token overloads and GetString are not implemented by this prerequisite.

## HTTP token forwarding and GetString — 2026-09-25

Development HttpHandler now requires `Send(request, cancellationToken)`. HttpClient
and HttpSocketHandler retain tokenless convenience overloads, and HttpClient adds
token-aware Get plus GetString for string and Uri addresses. Existing custom handlers
must add the token parameter and forward or honor it; rebuild callers and library
with the matching reference assembly. No runtime instruction or compiler policy is
changed. These are ordinary method/interface contracts over the existing token value.

A pre-requested token returns a cancelled task before request parsing or dispatch.
The socket handler passes one token through its private shared-deadline DNS/connect/
send/receive paths. It checks child-task cancellation before GetResult and closes its
owned connection before cancelling its output Promise. Native outcome commitment
still decides individual operation races. Cancellation after a successful phase
prevents the next phase; a fully received response can survive a later cancellation
request before managed delivery. Existing deadline checks can still produce timeout
errors. Timeout is not reported as caller cancellation. Blocked host DNS can outlive
guest acknowledgement under its existing bounded-capacity policy.

The client does not race custom handlers against cancellation. Handlers own their
work and cleanup: forwarding preserves the token, and a replacement handler must
acknowledge only after its owned work ends. Forced client-side completion could hide
live resources. Source disposal still unregisters without cancelling work. Independent
exchanges use separate buffers, tasks and connections. There is no injected-handler
disposal or cross-invocation thread-safety promise.

GetString uses Get and the existing cancellation-aware Task.Map to decode a completed
buffered response. HTTP errors remain HttpError; malformed UTF-8 becomes Protocol.
The helper now accepts 200–299; other statuses, including custom handler replies,
produce UnsuccessfulStatus with the original code (see the status checkpoint below). ReadText keeps its existing
provisional string-error contract. No charset negotiation, decoding replacement,
streaming or decompression is implied. Conversion does not reread the token after
its response task completes, so an already settled response is not retroactively
cancelled while waiting for its mapping continuation.

### Comparison and tradeoffs

Reviewed 2026-09-25: .NET 10's
[HttpMessageHandler.SendAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpmessagehandler.sendasync?view=net-10.0)
accepts a cancellation token, while
[HttpClient.GetStringAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.getstringasync?view=net-10.0)
has string/Uri and token overloads, reads the complete response body, and requires a
successful 200–299 status. neoCLR adopts the pipeline/token and buffered-text roles.
It returns typed Result errors and Task cancellation instead of HTTP exceptions.
Its strict UTF-8 policy remains a temporary restriction, with reduced encoding
interoperability. Final status support is recorded below.

Using the existing Task.Map avoids a second response buffer or another public async
abstraction, but still allocates a continuation and decoded string. An async method
or dedicated text adapter was an alternative; neither needs new runtime suspension
machinery. The private exchange callback state remains replaceable when suspension
arrives. No speed or allocation-performance advantage is claimed.

The [focused peer-controlled fixture](experiments/http-cancellation/README.md) verifies
pre-cancellation, custom/nested handlers, native acknowledgement and connection close,
request isolation, text errors and completion winning a late request. The independent
.NET comparison establishes the common behavior while preserving the stated POC
differences. Selected existing client interoperability checks now exercise GetString.

## Final response statuses — implemented 2026-09-25

Send/Get now preserve final HTTP/1.1 statuses 200–599 as response data. The new
IsSuccessStatusCode property reports 200–299; GetString applies that policy and
returns HttpError.UnsuccessfulStatus(statusCode) otherwise. This matches the roles
of .NET's [IsSuccessStatusCode](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpresponsemessage.issuccessstatuscode?view=net-10.0)
and GetStringAsync, while exposing a Result case rather than an HTTP exception.
A caller can inspect a 404 body through Get without treating it as a transport error.

Following [RFC 9112 message body length](https://www.rfc-editor.org/rfc/rfc9112.html#section-6.3),
204/304 complete after headers without waiting for EOF. A 204 Content-Length is
rejected; a 304 length is representation metadata, bounded here to Int32.MaxValue,
and does not allocate a body. A 205 requires Content-Length: 0 in this bounded
transport. Other responses still require exactly one Content-Length and the existing
1,024-byte bound. Empty reason phrases are accepted; malformed/out-of-range codes
are Protocol errors. Informational 1xx is explicitly Unsupported, pending framing
work. Redirects are returned without following them. Chunking, close-delimited bodies
and content decoding negotiation remain out of scope.

The alternative of accepting only known named status codes would reject extension
codes without helping framing. The bounded numeric range permits extensions such as
599 while retaining status-specific body rules. Strict framing remains deliberately
less interoperable than .NET's general HTTP implementation.

The [status fixture](experiments/http-status/README.md) compares .NET and neoCLR
against an independent raw peer, checks server wire output and demonstrates error
propagation with an application-owned implicit extension conversion. No global
HttpError-to-application-error conversion is added to the library.

### Follow-up: HttpStatusCode enum

The author asks whether status codes should have an enum. The proposed follow-up is
System.Web.Http.HttpStatusCode, compared with [.NET's HttpStatusCode](https://learn.microsoft.com/en-us/dotnet/api/system.net.httpstatuscode?view=net-10.0).
Named constants improve readability; unlisted numeric codes must remain representable
for protocol extensions. This is not a closed union and names do not imply transport
support (notably 1xx). Verify casts, unknown-value formatting and metadata before
changing HttpResponse and UnsuccessfulStatus signatures. The current slice retains
int and does not claim the enum is implemented.

### Later pattern-based inspection

The author also identifies Raven patterns as a way to inspect/deconstruct both
HttpResponse and, on the server, HttpRequest. A Deconstruct contract remains proposed;
no positional signature is selected yet. Evaluate request method/URI and response
status/header/content shapes against an actual handler sample as request content is
added. Properties remain the primary contract. Deconstruction adds positional coupling
and overload choices, so avoid exposing every field simply because it exists.

The author clarifies that record-like deconstruction should be considered for ordinary
API classes independently of value-object semantics. Treat this as a general design
consideration, not a request to make requests/responses structurally equal or to freeze
all their content. Explore semantic patterns in real samples before committing an
ordered Deconstruct signature; property inspection remains available.

## Named response statuses — development, 2026-09-25

HttpStatusCode is now an Int32-backed CLI enum in System.Web.Http. Its initial named
set follows familiar [.NET HttpStatusCode](https://learn.microsoft.com/en-us/dotnet/api/system.net.httpstatuscode?view=net-10.0)
values/names for common statuses; aliases and an exhaustive registry mirror are not
required for this POC. Unnamed values remain representable, including 599. Enum names
are not a transport capability claim: 1xx still remains unsupported. IsSuccessStatusCode
continues to classify the underlying 200–299 range, independently of defined names.

HttpResponse.StatusCode and HttpError.UnsuccessfulStatus now use the enum. A typed
response constructor is added; the existing Int32 constructor remains for callers
using numeric status codes. This changes development metadata signatures: rebuild
library, reference and consumers together. Cast to int for numeric output; enum
ToString gives a name for a defined value and numeric text for an unnamed value.
HttpError.ToString retains numeric status diagnostics.

Compared with retaining only int, the enum supplies discoverable names and useful
static typing at the cost of explicit numeric casts and maintaining the named set.
A closed union would wrongly suggest only listed HTTP statuses can exist. No new
runtime enum representation or Raven convention is introduced; the bridge admits
this exact enum and enum-typed union payloads through its existing enum mapping.

The author clarifies that property/positional patterns are optional ways to inspect
responses, not a preferred coding style. Property patterns need readable properties;
they do not require Deconstruct. No positional signature, structural equality or
copying semantics are introduced by the enum slice. Target property-pattern evidence
is retained in the status fixture rather than assuming the illustrative syntax compiles.

### Cancellation regression follow-up

The prior token/text checkpoint passes the original headers fixture, so the later
failure was not dismissed as pre-existing. The fixture unnecessarily waited for the
uncancelled third request before signalling cancellation of the other two. It now
signals as soon as the two relevant requests arrive; unrelated work remains concurrent
and must still finish successfully. Cancellation and peer-observed connection-close
assertions remain strict. No native deadlines, scheduler behavior or HTTP cancellation
implementation changes are made. This removes that fixture ordering dependency,
without proving the cause of every timing/performance difference between checkpoints.

## Buffered POST checkpoint — 2026-09-25

HttpClient.Post now accepts string/Uri addresses, HttpContent and optional cancellation,
using the same resolution and Send pipeline as Get. HttpRequest.Post creates a request
for direct Send. Content.FromText encodes UTF-8 with text/plain; charset=utf-8;
byte content optionally carries an outbound Content-Type. Request.Content exposes the
buffered bytes to custom handlers and servers. Non-success statuses remain responses.

Like [.NET PostAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.postasync?view=net-10.0),
this supports content and cancellation overloads and buffers the response. Unlike the
abstract .NET HttpContent family, this provisional concrete wrapper only carries bytes;
streaming, custom serializers and general header mutation remain later work. Result
represents expected failures, with cancellation remaining a Task outcome.

The socket provider snapshots at most 1,024 body bytes before DNS, computes byte
Content-Length (including zero), and bounds encoded headers to 2,048 bytes. Content-Type
must be nonempty printable ASCII; control characters cannot inject headers. This is not
a complete media-type parser or charset negotiation. Callers must keep content stable
while consumed. Framing stays provider-owned; no caller-supplied Content-Length.

[RFC 9112 section 6.3](https://www.rfc-editor.org/rfc/rfc9112.html#section-6.3)
defines request body length from framing, including zero when no framing is present.
The server now buffers bounded POST bodies before invoking the callback, rejects
ambiguous/unsupported framing and closes each connection. These bounds make the POC
reviewable but deliberately exclude ordinary larger uploads and chunked producers.
The proposal's fluent builders, general headers and additional verbs are not committed
by this checkpoint. See the [focused fixture](experiments/http-post/README.md).

No native socket/scheduler or Raven compiler policy changes. Bridge signatures admit
new public members and retain internal-only request serialization/content metadata.
The reference, managed library and generated fingerprints must be refreshed together.
