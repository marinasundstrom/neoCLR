# Neo.Web experimental web framework

Neo.Web is an experimental web framework built on top of
`System.Web.Http.HttpServer`. It combines dependency injection, configuration and logging.
A WebApplicationBuilder composes the services; a WebApplication registers routes
and owns the server lifetime. This is an optional experimental development
extension with provisional APIs, not a capability included in Preview 19 packages.

The follow-on development slice adds generic `MapGet<T: func>` and `MapPost<T: func>`
handlers, injected request services and typed buffered results. Focused framework
HTTP tests pass in interpreter, JIT and native execution on macOS ARM64 with the
matching development compiler/runtime; the initial interpreter timeout is fixed.
These checks are not a throughput or cross-platform qualification. Registration inspects
`handler.Function` and caches the endpoint's binding plan. Normal application code
imports `Neo.Web.*`; request scopes remain alive across asynchronous work.

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
paths 404, and unsupported methods 405 with an Allow header. Named string/int parameters bind from route captures; remaining service parameters
resolve from the request scope. Invalid integer text returns 400 before invocation.

Await RunAsync for the application lifetime. Stop closes connections and waits,
through that task, for started handlers to settle. Dispose the application only
after the lifetime completes. Shutdown does not yet drain in-flight responses
gracefully. Buffered responses are independent of request services before scopes
are released; streaming remains outside this slice.

The [lower-level HTTP APIs](/features/web/) remain available independently.

`HttpResult<T>` is a closed result union: `Ok(T)`, `Created(T)`, `BadRequest`
and `NotFound`. The initial adapter accepts `HttpResult<string>` and its task form,
writing successful payloads as text. Handlers can be called directly in tests with
ordinary arguments, and matches cover the result alternatives exhaustively.
WebDemo includes its own logic-test project; framework tests live under
`extensions/Neo.Web/test`.

Query/body binding, JSON serializers and streaming remain future work.
Reflection planning is cached, but per-request argument arrays, checked invocation
and DI resolution remain costs to measure.

## Pipeline foundation

`Map(pattern, contextHandler)` handles all HTTP methods for a full route pattern.
`Run(contextHandler)` configures the terminal handler for otherwise unmatched
requests; `RunAsync` starts and joins the application lifetime. Both use the fixed
context-handler shape and per-request scope. Malformed targets and wrong-method
routes retain 400/405 handling. `Use` adds middleware around validation, routing and terminal handling. The first
registered middleware runs first and resumes last after awaiting next. It can also
short-circuit with a response. One request scope remains alive through the entire
pipeline, including middleware work after the endpoint returns. The sample logs
requests before and after next; explicit routing-stage placement and path-prefix
branches remain future work.
