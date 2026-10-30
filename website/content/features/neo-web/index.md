# Neo.Web experimental web framework

Neo.Web is an experimental web framework built on top of
`System.Web.Http.HttpServer`. It combines dependency injection, configuration and logging.
A WebApplicationBuilder composes the services; a WebApplication registers routes
and owns the server lifetime. This is an optional experimental development
extension with provisional APIs, not a capability included in Preview 19 packages.

Context handlers receive `Neo.Web.HttpContext`. They read decoded route values,
resolve services from a request scope, and configure a buffered text or empty
response. Normal application code imports `Neo.Web.*` without importing the
transport namespace. Request scopes remain alive across asynchronous work.

```raven
{{NEO_WEB_HOME}}
```

This handler comes from the [reference application](/cases/neo-web/), which also
shows configuration, logging and singleton state. Build instructions and complete
source accompany the case. The [API reference](/docs/neo-web.html) describes each
member, ownership and failure contract.

## Routing and lifetime

Routes support literal segments and named captures. Literal routes win over
capture routes; ambiguous registrations fail. Invalid targets return 400, missing
paths 404, and unsupported methods 405 with an Allow header. Parameters are read
explicitly; generic minimal API binding is later work.

Await RunAsync for the application lifetime. Stop closes connections and waits,
through that task, for started handlers to settle. Dispose the application only
after the lifetime completes. Shutdown does not yet drain in-flight responses
gracefully. Buffered responses are independent of request services before scopes
are released; streaming and general middleware remain outside this slice.

The [lower-level HTTP APIs](/features/web/) remain available independently.

The intended next steps include dependency injection into handler parameters and
HTTP result objects typed from construction, with handler response alternatives
represented by a sealed hierarchy or a union. Those
are design directions, not capabilities of this context-handler slice.
