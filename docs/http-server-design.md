# Minimal HTTP server — development POC, 2026-09-24

**Current design guidance:** see the [2026-10-17 performance review](#performance-architecture-review--2026-10-17) for current implementation pointers and proposed next measurements. The dated POC sections below retain historical limitations.

**Development update 2026-10-11:** the [HTTP foundation](http-foundation.md) supersedes
older POC body/header bounds and adds shared chunked request reception, client interim
responses and content-oriented JSON APIs. Historical dated validation below records
its original bounds. Streaming, persistence, TLS and upgrades remain open.

The opening sections retain the initial checkpoint and its limitations. The
[current context/lifecycle contract](#context-ownership-and-response-configuration--2026-09-25)
supersedes the original listener-only Close and callback cancellation gaps.

The author selects HttpServer after the integrated HttpClient POC, testing both
against each other and independent peers. The networking proposal deliberately leaves
Run versus async request iteration open until cancellation and concurrency develop.
Start with a bounded ServeOne callback, not a permanent application hosting loop.

Implemented provisional development surface: HttpServer.Listen(address, port, backlog) returns
Result<HttpServer, HttpError>; GetLocalPort exposes a port-zero bind; ServeOne accepts
Func<HttpRequest, Task<Result<HttpResponse, HttpError>>> and returns a Task/Result once
one connection is handled and its response has been sent; Close stops listening.
Each operation owns its accepted connection. Closing the listener does not revoke an
already accepted connection; operation deadlines/cancellation remain explicit gaps.
This does not reuse the client HttpHandler as an application routing framework.

Compare .NET 10 [HttpListener.GetContextAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.httplistener.getcontextasync?view=net-10.0)
and [HttpListenerResponse](https://learn.microsoft.com/en-us/dotnet/api/system.net.httplistenerresponse?view=net-10.0):
request acceptance and response writing have distinct lifetimes. A context/stream API
would be more flexible but adds disposal and partial-body ownership now. A bounded
callback reuses buffered response/content values and owns cleanup at one boundary,
at the cost of rejecting streaming and serving only one request per operation.
Primary sources reviewed 2026-09-24. No OS-specific HTTP listener API or web framework
semantics become part of the public contract.

Initial protocol scope: HTTP/1.1 GET, origin-form target, exactly one Host, no request
body or transfer encoding, bounded ASCII headers; return status 200 with computed
Content-Length and Connection: close. Validate application-supplied response headers
before serialization and retain the client's body/header bounds. Expose parsed
request headers through a read-only sequence, separate from future client header
builders. Close connections on parse/handler/transfer errors. Unsupported requests
may close without an HTTP error response in this first POC; do not imply general
HTTP/1.1 compliance.

Validation: separate neoCLR client/server processes exchange UTF-8 greeting data;
.NET HttpClient also consumes the server; a controlled raw peer fragments a request
and probes invalid/missing/duplicate Host, framing and body limits. Under a small heap,
completion must reclaim all managed objects. Public APIs require matching compiler
reference/importer/library snapshots, API XML and website coverage in the same slice.

The callback model does not translate a cancelled callback Task or runtime Fault into
an HTTP Result. Those retain the existing Task/runtime behavior; a future cancellation
contract must cover both request ownership and callback completion. HTTP errors now use the standard HttpError union, retaining socket causes and
application-provided failures. ServeOne owns the
connection through completion, while request/response buffered values may be retained
by application code subject to their stable-sequence contract.

The local macOS POC passes an independent .NET 10 client, a separate neoCLR client,
a byte-fragmented request, eleven rejected request shapes and six invalid application
response/handler cases. Response checks include CRLF injection, framing override,
unsupported status, header/body bounds and a handler Result error. The Host authority
check rejects a slash that could otherwise be misread as a request target. Each
completed run uses a 256-object heap, collects and finishes with zero live objects.
The request/response snapshots and callback adapters add managed allocations; these
checks make no throughput or production robustness claim.

Typed-error regression: .NET and neoCLR clients and fragmented requests pass.
Malformed request, transfer timeout and application-response rejection paths retain
connection cleanup and finish with no live objects. The sample explicitly renders
HttpError.ToString at its reporting boundary. A transfer timeout now reports the
underlying TimedOut socket cause rather than the former generic receive-failed text;
application Handler errors preserve their message. No method/status/body expansion
is included in this error-model slice.

## Final status serialization — 2026-09-25

The development server now serializes final statuses 200–599. It rejects nonempty
content for 204/205/304, omits Content-Length for 204/304 and writes length zero for
205. Other statuses retain computed Content-Length. The reason phrase remains OK for
200 and is empty for other codes; HTTP does not require a descriptive phrase.
Invalid status/body combinations return InvalidRequest before writing a response.
See the [client status design](http-client-design.md#final-response-statuses--implemented-2026-09-25)
for the RFC/.NET comparison and [focused fixture](experiments/http-status/README.md)
for independent wire checks. Request methods and bodies remain the next slice.

## Buffered POST requests — 2026-09-25

ServeOne now accepts GET or POST and waits for the complete Content-Length body before
calling the application. Request.Content exposes received bytes; request Headers retain
lowercase parsed Content-Type. Absent Content-Length means zero. GET with a nonempty
body, duplicate/invalid lengths, Transfer-Encoding, Expect and Content-Encoding are
rejected. Request bodies share the 1,024-byte bound; incomplete EOF never invokes the
handler. No keep-alive, streaming or new cancellation/deadline policy is introduced.
See the [client design comparison](http-client-design.md#buffered-post-checkpoint--2026-09-25)
and [independent client/server checks](experiments/http-post/README.md).

## Explicit asynchronous acceptance — exploration, 2026-09-25

The author proposes an asynchronous Accept/AcceptRequest alternative to callback-based
ServeOne, returning Result with a request/response pair such as HttpContext. Record
this for the server-lifecycle slice; naming, an options argument and response shape
are not selected APIs. The current implementation remains callback-based.

Primary comparison reviewed 2026-09-25:
[.NET HttpListener.GetContextAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.httplistener.getcontextasync?view=net-10.0)
returns a Task<HttpListenerContext>, and
[HttpListenerContext](https://learn.microsoft.com/en-us/dotnet/api/system.net.httplistenercontext?view=net-10.0)
groups the incoming request and outgoing response. This is an API-shape comparison,
not adoption of HttpListener's implementation, URI-prefix model or platform behavior.

A candidate neoCLR shape is Accept(cancellationToken) returning
Task<Result<HttpContext, HttpError>>. AcceptRequest is an alternative method name;
do not introduce an AcceptRequest options type without concrete options. Expected
transport/protocol failures remain Result errors; cancellation follows the existing
Task cancellation contract. Returning a context transfers responsibility for completing
or abandoning that exchange to the caller, making cleanup part of the public design.

For the buffered checkpoint, prefer evaluating context.Request plus an explicit
Respond(HttpResponse, cancellationToken) returning Task<Result<unit, HttpError>>.
An outgoing response writer exposed as context.Response is another viable shape,
particularly with stream-backed content, but must be distinguished from the current
passive HttpResponse message used by clients and callbacks. A bare pair of passive
messages would not express sending, completion or connection ownership.

Contracts to settle and test before implementation:

- Accept completes once a valid request is available. Initially that can include the
  buffered body; streaming must revisit the header/body boundary.
- Context owns the accepted exchange, not the listener. Close/dispose ends its lifetime
  when the application is done; this is the normal scope boundary, not only an abort
  operation. Define response completion versus abandonment explicitly, including pending
  writes and observable send failures. Garbage collection is not the cleanup protocol.
  Sending completes one final response; duplicate sends and operations after close need
  explicit errors. Decide whether pre-send validation permits correction.
- Accept cancellation and response cancellation have distinct lifetimes. Cancelling
  a pending accept must not silently invalidate an already returned context. Specify
  how listener shutdown affects active contexts and pending operations.
- Caller chooses sequential or concurrent handling. Define concurrent accept support
  and active-context bounds instead of inferring unbounded concurrency.
- Build ServeOne as an adapter over the same accept/respond machinery where practical,
  with guaranteed cleanup on handler Result errors, cancellation and runtime failure.
  Preserve both public styles without duplicating HTTP parsing/framing.

This gives applications an explicit control-flow option and a foundation for future
response streams, at the cost of exposing lifetime management currently hidden by
ServeOne. Validate peer disconnects, malformed input, early return/abandonment,
duplicate response, cancellation and shutdown with independent peers and GC/resource
checks. This proposal does not require runtime suspension or a new public scheduler.

### Author clarification: the application context

The author identifies HttpContext as a foundational concept for building HTTP web
applications, explicitly closed or disposed when handling is done. This establishes
its architectural role beyond an accept-result container. Both explicit accept loops
and callback-based hosting should share this per-exchange context and lifecycle; the
exact callback migration remains to be designed. Future application handling can build
on the same request/response scope without exposing a socket or adding routing now.

A context represents one HTTP exchange, not necessarily one physical connection.
Closing it must not close the listener; later keep-alive or multiplexing must not make
its lifetime synonymous with closing a transport connection. Disposal should be safe
on repeated calls and release owned resources on normal completion and early exit.
Stream ownership, outstanding operations and retained request data need explicit rules.

The current System.Disposable.Dispose returns no result, whereas Closable<E>.Close
can return a Result. Async response completion may fail, so decide how callers await
and observe completion before releasing the context; synchronous disposal alone cannot
promise successful network delivery. The author's clarification does not select a
completion method, implicit flush policy or response writer signature. These are
implementation questions within the planned foundational context, not objections to
using close/dispose as the application scope boundary. No public context API exists yet.

## PUT, PATCH and DELETE — 2026-09-25

The parser now admits PUT/PATCH through the bounded Content-Length body path and
DELETE without content. Handlers receive the exact method and buffered content, and
continue selecting response status and fields. A nonempty DELETE body is Unsupported;
HEAD/OPTIONS remain outside this checkpoint. Accept/context lifetime work remains
planned; ServeOne is still the implemented server API. See the
[verb fixture](experiments/http-verbs/README.md) and the
[client comparison](http-client-design.md#common-verb-checkpoint--2026-09-25).

### HEAD response serialization — implemented 2026-09-25

The server now accepts bodyless HEAD. Its handler supplies the same bounded response
representation it would for GET; the encoder computes its byte Content-Length and
omits the body. Existing 204/205/304 content restrictions still apply. The .NET peer
in `experiments/http-verbs` checks a five-byte UTF-8 representation and empty received
content. See the [framing comparison](http-client-design.md#bounded-response-framing-and-head--2026-09-25).

## Context ownership and response configuration — 2026-09-25

This development checkpoint resolves the earlier explicit-accept exploration:

- `Accept(token)` returns `Task<Result<HttpContext, HttpError>>` after validating and
  buffering a request. Its token covers acceptance/reading, then detaches. The caller
  owns the returned scope and passes a token separately to `Complete` if needed.
- `context.Request` is the incoming message. `context.Response` is outgoing mutable
  configuration, initially status 200 with empty content.
- `HttpResponse.Respond(statusCode)` replaces status and clears content;
  `Respond(statusCode, content)` sets both. Content-Type is replaced from content
  metadata (removed if absent); other headers remain. These methods perform no I/O.
  `context.Respond` forwards both shapes. `RespondText(text)` and
  `RespondText(statusCode, text)` configure UTF-8 text/plain content through them.
- `context.Complete(token)` validates and snapshots the configured response, sends it,
  then closes the exchange. Validation/send failure and acknowledged cancellation
  also end it. Duplicate completion or completion after closure is InvalidRequest.
  Header/content mutations after the snapshot cannot change captured wire bytes;
  they can still change the local message object. Keep sequences stable while consumed.
- `Close`/`Dispose` end the scope without sending unsent content. They are idempotent;
  closing an in-progress completion causes cancellation after native acknowledgement.
  Bytes may already have reached the peer. Request/response data remain readable.
- `ServeOne` retains its request-to-response callback for compatibility, adds a token
  overload and uses the same context sender and cleanup. Handler Result errors and
  cancelled tasks close the scope. Cancellation during a handler wait stops waiting
  and ignores a later result; it cannot preempt code or cancel unrelated handler work.
- `HttpServer.Close` now cancels pending accepts and ends active contexts/callback
  waits. **Migration:** it previously closed only the listener. Shutdown does not drain
  handlers, block for native callbacks or guarantee delivery. Continue pumping the
  scheduler to observe pending operation acknowledgement. Terminal Faults do not
  promise managed cleanup; invocation teardown owns native resources in that case.

At most 16 accept/context scopes may be outstanding per server. Native listeners
still admit one pending accept, but applications can hold multiple completed contexts.
Reading has a provisional 15-second budget starting after acceptance; sending has a
separate 15-second budget starting after snapshotting. Both retain shorter native phase
bounds. Waiting for connections/handlers has no automatic deadline; cancellation and
server shutdown end those waits. These invocation-local APIs add no worker pool or
public scheduler dependency. Private continuation machines remain replaceable when
runtime suspension is available.

### Comparison and deliberate limits

Primary sources reviewed 2026-09-25: [.NET HttpListener.GetContextAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.httplistener.getcontextasync?view=net-10.0)
returns a request/response context, while [HttpListenerResponse.Close](https://learn.microsoft.com/en-us/dotnet/api/system.net.httplistenerresponse.close?view=net-10.0)
combines sending with cleanup. [ASP.NET Core HttpResponse.CompleteAsync](https://learn.microsoft.com/en-us/dotnet/api/microsoft.aspnetcore.http.httpresponse.completeasync?view=aspnetcore-10.0)
provides an asynchronous response-completion boundary. neoCLR separates synchronous
configuration, observable asynchronous completion and synchronous abandonment, so
sending errors are Result outcomes rather than hidden in disposal. The cost is an
explicit completion step; disposal alone does not deliver a response. This is a small
buffered contract, not ASP.NET Core's streaming writer or host feature system.

[Go Server.Close and Shutdown](https://pkg.go.dev/net/http#Server.Close) distinguish
immediate closure from graceful draining. This checkpoint chooses immediate scope
closure for bounded shutdown. Graceful draining needs a later explicit deadline and
in-flight-work policy; it must not be implied by the current Close name. An independent
.NET hosting-framework comparison beyond these platform APIs remains open; this slice
does not claim a better server architecture or performance.

**Author direction:** consider HttpRequest/HttpResponse interfaces with separate
InboundHttpRequest/OutboundHttpRequest and response implementations later. That could
make configuration legal only on outbound messages and support different behavior.
For now the existing concrete response carries the setter-style Respond methods.
Calling them on a received client response only changes that local object. This
provisional mutability is not a final interface/implementation split or a commitment
to the inbound/outbound names. No new class hierarchy is introduced prematurely.

The [context fixture](experiments/http-context/README.md) covers these ownership
boundaries and the compiled convenience sample; existing independent .NET verb/HEAD
checks exercise the callback path over the same context implementation.

Validation on 2026-09-25: the context stress/sample fixture, caller/server cancellation
peers and existing .NET verb/HEAD callback fixture pass with final live-object count
zero. Signature admission/private-helper rejection, Disposable conversion and matching
API/bootstrap checks pass. Website build remains skipped by author direction. The
frozen-toolchain callback/propagation limitations and retained reproduction are listed
in the [integration notes](raven-backend-integration-map.md#http-context-and-configured-responses--2026-09-25);
they remain stabilization work, not fixes claimed by this API slice.

## Paired client/server tracking — 2026-09-26

The [shared HTTP capability tracker](http-capabilities.md) is the active status and
POC completion record. The new client known-length upload source does not enlarge
server request admission: this server still buffers at most 1,024 bytes and emits
bounded fixed-length responses. Passing stream-backed content to a response returns
Unsupported during completion, rather than silently sending an empty body. Request
streaming, response writers, trailers and newer protocol providers remain deferred.
The author selects a finite POC finish line and thematic tracking consolidation next.


## Performance architecture review — 2026-10-17

The author supplied architectural optimization guidance covering asynchronous I/O,
byte ownership, incremental framing, scheduling, streaming, admission/backpressure,
and sustained-load measurement. Treat the suggested MapGet/Listen/OpenWriter forms
as illustrative, and the suggested version-0.1 order as input rather than a reset
of the [platform roadmap](platform-roadmap.md). No new public API, native provider,
or implementation placement is selected by this review.

### Existing implementation and evidence

- HTTP protocol and exchange control are already managed Raven code:
  [HttpServer](../runtime/raven/src/System/Web/Http/HttpServer.rvn),
  [HttpWire](../runtime/raven/src/System/Web/Http/HttpWire.rvn) and
  [HttpBodyInput](../runtime/raven/src/System/Web/Http/HttpBodyInput.rvn).
  Preserve byte-oriented incremental parsing and framing checks when optimizing.
- [Server options](http-server-options.md) bound headers, bodies and read/write
  deadlines; the current exchange cap is 16. These are bounded development
  contracts, not evidence for thousands of concurrent connections.
- [Body streams](../api-docs/http-body-streams.md) already provide headers-first
  acceptance and known-length response sources. This is narrower than arbitrary
  unknown-length response streaming. [Current server APIs](../api-docs/http-server.md)
  retain one-exchange connection ownership; persistence remains a separate change.
- Native transport already supports nonblocking socket operations. The
  [platform boundary](native-http-parity.md#platform-boundary-and-tradeoffs) and
  [Windows readiness measurements](../benchmarks/http-scheduling/README.md)
  distinguish provider behavior from HTTP logic. Do not describe the existing
  transport as a Tokio service or claim scalable IOCP/kqueue support from these checks.
- [Byte-path measurements](../benchmarks/http-server/byte-paths.md) and subsequent
  [allocation attribution](../benchmarks/http-server/allocation-sites.md) already
  motivate fewer parser temporaries. The [benchmark](../benchmarks/http-server/README.md)
  uses sequential loopback requests and connection close. It does not measure
  sustained capacity, fixed offered-rate latency or high concurrency.

### Placement and ownership recommendation — provisional

The assistant recommends retaining a managed HTTP engine over reusable socket,
stream and task primitives. This makes HTTP a demanding consumer of the runtime
and keeps the same foundations useful for other networking libraries. A native
protocol engine remains an alternative if equivalent measurements justify its
boundary and maintenance costs; Rust alone does not establish a speedup.

Compare three options before a substantial transport change: managed protocol over
runtime I/O, a runtime-owned native service, and a separate native library behind
an explicit ABI. Follow the [placement criteria](design-research.md#choose-runtime-services-or-foreign-libraries-deliberately):
GC roots, thread entry, cancellation, shutdown and interpreter/JIT/AOT availability
must be specified. A runtime adapter may use an external implementation without
making HTTP itself a privileged runtime service. Tokio/provider selection and
application-worker scheduling remain open.

Keep platform event mechanisms behind the runtime transport contract. Completion
and readiness providers can share public operation semantics without identical
internals. Inline completion needs a fairness budget; arbitrary handler execution
must not indefinitely delay other connections. Do not assume cooperative await
preempts CPU-bound code, or equate a suspended task with a dedicated OS thread.

Reuse the [byte ownership design](http-foundation.md#byte-ownership-and-lazy-materialization-direction--2026-10-14).
A native-backed view needs an enforceable owner/lease, bounded range and explicit
release semantics across await, cancellation and disposal. Retained public header
values must remain valid after buffer reuse; copy or retain ownership as needed.
An ordinary byte array remains appropriate for independently owned data. Pooling
must account for idle retention and global budgets, not just allocations/request.
Vectored writes, sendfile, larger read blocks and custom allocators are measurement
candidates, not prerequisites or universal zero-copy promises.

### .NET/protocol comparison and proposed bounded next work

Primary sources checked 2026-10-17: [.NET pipelines](https://learn.microsoft.com/en-us/dotnet/standard/io/pipelines)
describe segmented buffering, consumption lifetimes and backpressure;
[Kestrel .NET 10 limits](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/servers/kestrel/options?view=aspnetcore-10.0)
separate connection limits, header timeout, body/data-rate policy and keep-alive.
These are library contracts to compare with neoCLR's current smaller surface, not
reasons to change CLR semantics. [RFC 9112 sections 6 and 9](https://www.rfc-editor.org/rfc/rfc9112.html)
govern framing and connection management. Persistence must preserve message
boundaries and reject ambiguous framing; it cannot be enabled by changing a header.
Reuse the broader alternatives in the linked byte-ownership research. This review
adds no new contract requiring a new framework selection.

After current release qualification, the assistant proposes a bounded load-harness
extension for the existing connection-close server before choosing another hot-path
optimization. Sweep concurrency within current admission, then deliberately exceed
it; test slow headers, slow response readers and cancellation/shutdown. Record
errors and rejected work as well as throughput, p50/p95/p99, CPU, allocated bytes,
peak retained memory and cleanup. Use independently scheduled offered requests and
measure latency from their intended start, recording missed arrivals so overload
cannot disappear through client waiting. Keep warmup, artifact hashes, GC policy,
TLS mode and machine load explicit; use a separate generator host when available.
Compare equivalent .NET workloads with identical connection/body/TLS policies.
This is a proposed experiment; no new load results are claimed here.

Persistent HTTP/1.1 then requires explicit connection-versus-exchange ownership,
unread-body drain-or-close policy, preservation of surplus bytes, idle deadlines,
and per-connection/global budgets. Validate fragmented and coalesced requests,
ambiguous framing, disconnects and slow clients before measuring reuse benefits.
TLS already has development integration evidence; its load behavior remains a
separate measurement. HTTP/2, routing tries and additional write optimizations stay
follow-ups governed by evidence and the roadmap, not automatic scope additions.


## Measured buffer refinement — 2026-10-18

The author selected profiling and targeted buffer/response-copy improvements, then
clarified the objective of honestly measuring sustainable server capacity. See the
[implementation, profiles and validation](../benchmarks/http-server/buffers-20261018/README.md).
Response snapshots remain independent of caller mutations; exact storage removes
list-growth copies and admitted arrays avoid the transport staging copy. Writes
and stream blocks are bounded at 4 KiB; receive buffers grow after partial parsing.
This is development after Preview 16, not a new public buffer ownership contract.
Native transport still copies data. Sustained offered-rate/concurrency and retained
memory measurements remain necessary before describing a server capacity limit.


### Offered load and timeout follow-up — 2026-10-18

The [load investigation](../benchmarks/http-server/load/README.md) adds independent
arrival scheduling, explicit generator misses and failed responses, intended-start
latency, CPU/RSS sampling and graceful shutdown accounting. One/four/sixteen-worker
profiles isolate the cost of repeatedly collecting a larger live graph from HTTP
allocation rate. Paced collection remains an explicit experimental comparison.

A 5 ms waiting-producer probe reproduced incorrect cancelled tasks under paced GC:
network deadlines round upward, but the producer cancellation timer did not account
for that rounding. An extra millisecond on that timer prevents it preceding the
network deadline. Existing send deadline checks and caller cancellation precedence
are unchanged. The probe and three-mode stream checks validate the fix; the original
unindexed intermittent failure cannot be causally identified retrospectively.


### Lazy accept state and callback-path measurement — 2026-10-18

[The follow-up investigation](../benchmarks/http-server/lazy-accept/README.md) moves
request-only parser/buffer storage from queued exchange construction to the first
HTTP read. Cancellation and shutdown registrations remain live while queued, and the
16-exchange bound remains unchanged. Successful accepts no longer allocate their
unused rejection promise. There is no public API or ownership change.

The author clarified that `ServeOne` with an async callback is the usual application
path. The matched benchmark therefore includes an awaited queued operation inside
that callback, with bounded concurrent serving loops. A sequential `await ServeOne`
loop suspends efficiently but still serves one exchange at a time; explicit `Accept`
requires the application to arrange its own context handling. Compare the same
callback fixture across versions rather than treating raw-accept rates as callback
capacity. This retains the existing .NET-style asynchronous handler/transport
separation; it changes internal allocation timing, not the platform contract.

### Server-owned Serve lifecycle — 2026-10-18

Author direction now explicitly requests `Serve(handler)` and measurement of that
path. The managed dispatcher owns 16 reusable lanes and observes the full exchange
and handler task in each. Calling Serve starts work immediately; its task represents
the server lifetime. The application has no accept/dispatch loop. This implements
the author's correction; earlier Run/Serve suggestions above are historical proposals.

Compare .NET's [RequestDelegate](https://source.dot.net/Microsoft.AspNetCore.Http.Abstractions/RequestDelegate.cs.html)
and [WebApplication.RunAsync](https://learn.microsoft.com/en-us/dotnet/api/microsoft.aspnetcore.builder.webapplication?view=aspnetcore-10.0):
an asynchronous request callback and a server-lifetime task are distinct. Primary
sources checked 2026-10-18. neoCLR retains its existing Task<Result<HttpResponse,
HttpError>> callback and the author-selected Serve name. This is a small HTTP library,
not ASP.NET hosting: no dependency-injection host, middleware pipeline, keep-alive,
automatic error pages or thread-pool dispatch is implied.

The alternatives were an unbounded detached task per request, application-owned
ServeOne loops, and server-owned bounded supervision. The selected model bounds
memory and admission while keeping request outcome/response disposal inside the
library. Fixed concurrency reuses the existing 16-exchange contract; configurable
limits and graceful response draining remain separate work. Pre-cancellation leaves
the listener open; active cancellation/Close closes it and joins started handlers.
Explicit Accept/AcceptHeaders/ServeOne cannot compete with active Serve. Request
errors are observed, counted and retained as one last error while other requests
continue. Listener failure terminates the lifetime task. Runtime Faults are terminal.

Serve owns successful handler responses, including invalid or late streaming content.
A managed exchange retains its handler outcome subscription after Close, disposes a
late response, and only then acknowledges cancellation. This differs deliberately
from ServeOne's existing early cancellation acknowledgement. An explicit lane list
roots supervised operations until all finish. A handler that never settles prevents
joined completion; there is no forced cancellation of arbitrary managed work.
[Contract and load evidence](../benchmarks/http-server/serve/README.md) cover concurrency,
bounds, error recovery, shutdown and callback-path cost. Public API reference and
native/CLI documentation snapshots are refreshed; no Raven compiler change is needed.
