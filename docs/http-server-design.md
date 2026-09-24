# Minimal HTTP server — development POC, 2026-09-24

The author selects HttpServer after the integrated HttpClient POC, testing both
against each other and independent peers. The networking proposal deliberately leaves
Run versus async request iteration open until cancellation and concurrency develop.
Start with a bounded ServeOne callback, not a permanent application hosting loop.

Implemented provisional development surface: HttpServer.Listen(address, port, backlog) returns
Result<HttpServer, string>; GetLocalPort exposes a port-zero bind; ServeOne accepts
Func<HttpRequest, Task<Result<HttpResponse, string>>> and returns a Task/Result once
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
contract must cover both request ownership and callback completion. String errors remain
provisional pending the separately recorded nested-union design. ServeOne owns the
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
