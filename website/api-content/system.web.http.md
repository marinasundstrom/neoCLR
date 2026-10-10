---
uid: N:System.Web.Http
---
## HTTP requests, responses and handlers

HttpClient and HttpServer provide the bounded HTTP proof of concept. Requests,
responses, headers and content describe exchanged messages; HttpContext and handler
contracts connect server behavior and the replaceable client pipeline. RoutePattern
and RouteMatch support the implemented routing surface.

See [Web](/features/web/) for the current cleartext scope, limits and tested samples.
This preview is not a general production web stack. System.Web.Http.Json supplies
JSON content and client helpers.

Development HttpClient.Timeout is an Option<Duration>, defaulting to Some(100 seconds).
None disables the client countdown. Configure it before sending. Expiry requests
cancellation and waits for handler cleanup before returning HttpError.TimedOut; caller
or disposal cancellation remains task cancellation. Independent transport phase bounds
still apply. See [HTTP contracts](/features/web/) for limits and handler responsibilities.

HttpMethod provides canonical method properties, custom token construction and typed
Parse. HttpRequest.Method returns HttpMethod; Create accepts it with URL/Uri and optional
content. Exact spelling is available through Method.Method; value equality ignores
ASCII case. CONNECT is representable but the socket transport does not implement tunnelling.
