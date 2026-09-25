# Minimal HTTP server — development POC, 2026-09-24

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
