# HTTP message roles (development)

System.Web.Http separates client and server messages. The names describe ownership;
these types have independent contracts and implementations, without a common message
base. The client configures a request and receives a response. The server receives
a request and configures a response.

| Type | Role | Headers |
| --- | --- | --- |
| [HttpClientRequest](api/System/Web/Http/HttpClientRequest/index.html) | Outgoing client request | MutableHttpHeaderCollection |
| [HttpClientResponse](api/System/Web/Http/HttpClientResponse/index.html) | Incoming client response | HttpHeaderCollection |
| [HttpServerRequest](api/System/Web/Http/HttpServerRequest/index.html) | Incoming server request | HttpHeaderCollection |
| [HttpServerResponse](api/System/Web/Http/HttpServerResponse/index.html) | Outgoing server response | MutableHttpHeaderCollection |

Read-only received data can still be parsed, materialized or cached lazily. It also
permits body consumption and disposal. Read-only does not imply eager construction,
deep immutability of caller-supplied inputs, or thread safety. Outgoing mutation is
currently provided through header operations and role-specific configuration methods;
not every property has a setter.

## Client contracts

`HttpClientRequest.Create(method, urlOrUri[, content])` constructs a validated
request, returning Result with HttpError. Get, Head and Delete are method-specific
factories; Post, Put and Patch additionally accept content. Method, Host, Port,
Target and IsSecure describe the selected request destination; Version currently
reports HTTP/1.1. Content retains the selected HttpContent instance.

Headers supports validated Set, Append and Remove. Client-owned collections reject
transport/content-controlled fields. `WithHeader(name, value)` returns a separate
request with the replacement field; it shares content ownership with the original.
Disposing either content-sharing request affects that content for every alias.

`HttpClientResponse` exposes StatusCode, IsSuccessStatusCode (200–299), Version,
ReasonPhrase, Headers, Content and its optional associated Request. Non-success
status responses remain available to inspect. Its `(intOrStatusCode, headers,
content)` constructors support custom client handlers. It has no Respond or
SetHeader API. Body completion/disposal follows the selected client completion
option; see [body streams](http-body-streams.html).

## Server contracts

HttpServerRequest is supplied by HttpServer; it has no client URL factories,
WithHeader or request-encoding API. Method, Host, Port, Target, IsSecure and Version
describe the accepted request. Headers exposes received fields and Content owns the
request body. Read-only access does not prevent consuming or disposing the body.

HttpServerResponse has an empty OK constructor and `(intOrStatusCode, content)` and
`(intOrStatusCode, headers, content)` overloads. StatusCode, Version and Content expose
the response configuration. Respond selects status and optionally content; SetHeader
returns `Result<unit, HttpError>` and uses the mutable Headers contract. A previously
obtained header view stays attached across these changes. No client Request association
or received ReasonPhrase is carried on the server response. See [server lifetimes](http-server.html)
for commitment, cancellation, completion and shutdown rules.

All four types provide Dispose, GetHeaderValues and GetContentType. Dispose releases
owned content without draining it and is safe to repeat. GetHeaderValues returns an
independent case-insensitive multiple-value snapshot. GetContentType returns
`Result<Optional<MediaType>, HttpError>`: absent, one parsed value, or Protocol for
malformed/duplicate fields. These operations do not read the body. See the
[header reference](http-headers.html) for validation and allocation behavior.

## Migration and scope

Replace the provisional HttpRequest/HttpResponse names according to the caller's
role and rebuild dependent libraries together. HttpClient and HttpHandler use the
client pair; HttpServer, its HttpContext and Neo.Web use the server pair. Replace
concrete header constructors with HttpHeaderCollection.Empty, Snapshot(headers), or
MutableHttpHeaderCollection.Create(). The native API reference covers the full
surface; MediaType-returning methods remain absent from the legacy CLI facade only.

.NET separates [client request messages](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httprequestmessage?view=net-10.0)
from [ASP.NET Core request/response surfaces](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/use-http-context?view=aspnetcore-10.0).
ASP.NET Core permits middleware to modify incoming request values. neoCLR's
read-only received-message contract is a deliberate author-directed difference:
it protects the received view, but middleware rewriting will need an explicit
future adaptation contract rather than direct header mutation.
neoCLR deliberately names both pairs by role and separates read-only/mutable header
interfaces. Benefits are explicit ownership and room for specialized storage; costs
are additional public types and a breaking migration. HttpContent remains shared in
this slice, with server-specific body APIs open for later work. Deferred wrappers
can reduce allocations, but throughput improvements must be measured against direct
HttpServer handling; the API split alone establishes no speedup.
