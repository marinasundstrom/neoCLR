---
title: Experimental Neo.Web hosting
---
# Experimental Neo.Web hosting

Neo.Web is an experimental web framework built on top of
`System.Web.Http.HttpServer`, available as an optional development extension.
It is not part of published Preview 19.
Its API is provisional. It composes the existing extension services and HttpServer;
generic handlers use the structural `func` contract. The follow-on implementation
is under qualification; matching compiler/runtime changes are required. All public types are in the
`Neo.Web` module and `Neo.Web.dll` assembly.

## Middleware

`Middleware` is an interface with one member:

```raven
func InvokeAsync(context: HttpContext,
    next: (HttpContext) -> Task<Result<unit, WebError>>) -> Task<Result<unit, WebError>>
```

Await `next(context)` at most once, or return to short-circuit. Register the
implementation with `builder.Services` before `Build()`, then call
`app.UseMiddleware<Implementation>()`. Registration adds an adapter without
creating the service. Each reached request resolves it through `context.Services`;
DI controls singleton/scoped/transient lifetime and ownership. The scope remains
alive until the outermost middleware settles. Missing services or recoverable
activation errors become the existing sanitized HTTP 500 response. Constructor
activation requires a publicly visible implementation and public constructor, and
needs constructor reflection retention in native/JIT deployments, as
shown by WebDemo's project retention file. There is no implicit service registration
or separate middleware factory service. Function middleware passed to `Use` keeps
its caller-owned captures and uses the same pipeline contract.

## WebApplicationBuilder

Create with `WebApplication.CreateBuilder()` or the parameterless constructor.

| Member | Contract |
| --- | --- |
| `Services: ServiceCollection` | Mutable registrations, snapshotted at Build. Supplied instances stay caller-owned. |
| `Configuration: ConfigurationBuilder` | Explicit sources; Build creates an owned root. No automatic files/environment/CLI loading. |
| `Logging: LoggerFactory` | Builder-owned default factory, Information threshold and no providers. Use AddConsole from Neo.Logging. |
| `Server: WebServerOptions` | Mutable settings copied into the application at Build. |
| `Build() -> Result<WebApplication, WebError>` | Consumes the builder on the first attempt, validates options, builds configuration and services, and transfers owned resources on success. Opens no socket. |
| `Dispose()` | Idempotent; releases default logging unless transferred to an application. |

Configuration and LoggerFactory DI registrations preserve explicit overrides. An
externally supplied factory remains externally owned; the default builder factory
is still owned and disposed by the builder/application. Configuration provider Load
faults retain their terminal behavior; they are not converted to WebError. After
Build fails, use another builder. Property mutation after Build does not change the
application's listener options or service registry.

## WebServerOptions

`Address: string` defaults to `127.0.0.1`, `Port: int` to 5000, and `Backlog: int`
to 128. Build rejects an empty address, a port outside 0–65535 and a nonpositive
backlog. Port zero requests an ephemeral port. Other address/listener failures are
reported by RunAsync. Listener configuration is explicit; configuration values are
not automatically bound. HttpServer's default transport options remain in effect.

## WebApplication

| Member | Contract |
| --- | --- |
| `CreateBuilder() -> WebApplicationBuilder` | Static factory with the defaults above. |
| `Services: ServiceProvider` | Root provider; resolve scoped services through request context instead. |
| `Configuration: ConfigurationRoot` | Application-owned live root. |
| `MapContext(method: string, pattern: string, handler: (Neo.Web.HttpContext) -> Task<Result<unit, WebError>>) -> Result<unit, WebError>` | Registers one callback atomically; invalid method/pattern, ambiguous overlap or frozen/disposed state returns failure. |
| `RunAsync(cancellationToken: CancellationToken = default) -> Task<Result<unit, WebError>>` | Freezes registration, opens a listener and returns its joined lifetime. A listen attempt consumes the single-run application. |
| `Use(handler: (HttpContext, (HttpContext) -> Task<Result<unit, WebError>>) -> Task<Result<unit, WebError>>) -> Result<WebApplication, WebError>` | Adds middleware around routing and terminal handling, in registration order. Await next(context) to continue, or return to short-circuit. Frozen/disposed registration returns InvalidState. |
| `UseMiddleware<T: Middleware>() -> Result<WebApplication, WebError>` | Adds a typed middleware stage; resolves its explicit DI registration from each request scope. Frozen/disposed registration returns InvalidState. |
| `Map(pattern: string, handler: (HttpContext) -> Task<Result<unit, WebError>>) -> Result<unit, WebError>` | Registers a fixed context handler for all HTTP methods at a route pattern. |
| `Run(handler: (HttpContext) -> Task<Result<unit, WebError>>) -> Result<unit, WebError>` | Installs the single terminal handler for requests left unmatched by routing; does not start the server. Duplicate/frozen/disposed registration returns InvalidState. |
| `MapGet<T: func>(pattern: string, handler: T) -> Result<unit, WebError>` | Registers a GET function value after inspecting its bound signature and caching its binding plan. |
| `MapPost<T: func>(pattern: string, handler: T) -> Result<unit, WebError>` | Registers a POST function value under the same binding/result contract. |
| `GetLocalPort() -> Result<int, WebError>` | Actual listener port while running; otherwise InvalidState. |
| `Stop()` | Requests application cancellation and closes connections; no effect before running or after completion. |
| `Dispose()` | Idempotent outside a run. Disposes root services, default logging and configuration. Active disposal is terminal misuse: Stop and await the lifetime first. |

`Use` and `UseMiddleware` return `Ok(self)` for composition. Inside a compatible
`Result`-returning function, postfix `?` propagates the first error and skips later
statements; `!.` is available for unwrap-or-fail chaining. Carrier `?.` maps the
accessed value, so calling a Result-returning method through it nests Results. Successful earlier registrations remain in
place when a later registration fails; a chain is not a transaction. `Map*` and
terminal `Run` retain `Result<unit, WebError>` in this slice; endpoint convention
builders remain future work. Prefer separate route declarations when clearer.

Pre-cancelled RunAsync opens no socket, returns a cancelled Task and leaves
registration available. Caller cancellation cancels the lifetime task; ordinary
Stop completes it successfully after started handlers settle. Listener failures
return Transport with the original HttpError. Stop closes connections immediately;
it does not gracefully finish in-flight responses. Handlers must settle
cooperatively. Runtime Panic remains terminal.

Routes reuse RoutePattern's UTF-8 decoding, limits, case-sensitive literals and
slash semantics. Literal segments outrank captures at the first differing segment.
Same-method, equal-precedence overlapping registrations fail; registration order
never resolves ambiguity. Methods use HttpMethod's existing comparison contract.
Malformed targets produce 400, unmatched paths 404, and unmatched methods on a
matching path 405 with Allow. HEAD and OPTIONS require explicit registration.

## Pipeline foundation

The fixed terminal shape is `(HttpContext) -> Task<Result<unit, WebError>>`.
`Map` registers an all-method route; `Run` installs an unpatterned terminal handler.
`RunAsync` remains the application lifetime operation. Registration freezes when
listening begins. Middleware is composed once when RunAsync starts. The first registered middleware
runs first and resumes last after `await next(context)`. The innermost continuation
performs target validation, routing, then terminal handling. Middleware may respond
and return without calling next. `Result` failures propagate through the same
sanitized 400/500 response policy; cancellation closes the exchange.

One request scope covers all middleware and the selected handler, including work
after awaiting next, and is disposed after the outermost task settles. The same
context flows through every stage. RouteValues is empty before routing and contains
the selected captures after next returns. Pass the supplied context to next; call
and await it at most once. Retaining the continuation or context after completion,
fire-and-forget next calls, and reentrant execution are unsupported.

Literal route precedence applies across method-specific and all-method routes.
At equal route precedence, a matching method-specific endpoint wins over an
all-method endpoint. Equal-precedence overlapping all-method registrations reject.
The terminal handler receives empty route values and the same request scope,
awaited lifetime and buffered-response contract as a routed handler. It runs when
no route matches; requests reaching routing with malformed targets receive 400 and route method mismatches receive
405 with Allow. Without a terminal handler, unmatched paths remain 404.

Unlike ASP.NET Core's path-prefix `Map` middleware branch, this `Map` performs
full route-pattern matching. Unlike an ordered middleware builder, registering
`Run` before routes does not make them unreachable: it configures the fixed final
stage. Middleware ordering applies to both Use and UseMiddleware registrations; explicit routing-stage insertion
and path-prefix branches remain future work.

## Neo.Web.HttpContext

This is distinct from `System.Web.Http.HttpContext`. Applications normally import
only `Neo.Web.*`; add `import Neo.Web.HttpContext` when also importing
`System.Web.Http.*`. The transport
context, direct completion and mutable streaming content are not exposed.

| Member | Contract |
| --- | --- |
| `Request: System.Web.Http.HttpRequest` | Incoming request/body, borrowed for this handler's lifetime. |
| `Services: ServiceProvider` | Per-request scope. The application owns disposal; handlers must not dispose it. |
| `RouteValues: System.Web.Http.RouteMatch` | Decoded captures; use Get/GetInt32 for explicit binding. |
| `ApplicationStopping: CancellationToken` | Application shutdown signal, not client-disconnect cancellation. |
| `RespondText(text: string)` | Sets status 200 and independent UTF-8 text/plain content. |
| `RespondText(statusCode: int, text: string)` | Sets explicit status and buffered text. |
| `Respond(statusCode: int)` | Sets explicit status and empty content. |

Explicit status must be 100–599; invalid values and response mutation after handler
completion call Panic. Transport-specific status/body validation still applies
when sending. There is no public constructor: Neo.Web creates the view per matched
request. Request values must not escape the pipeline lifetime.

The request scope survives handler and middleware awaits, then is disposed on success, typed
failure or cancellation. Responses own buffered data independently of services.
A successful callback sends its configured response (default 200 empty). BadRequest
returns sanitized 400; other WebError values produce a logged sanitized 500.
Cancelled handlers close their exchange. Terminal target faults are not HTTP error
results and have no additional cleanup guarantee. Returned error messages are not
included in client responses.

## WebError

A standard Raven union: `InvalidState`, `InvalidOptions(message: string)`,
`Routing(message: string)`, `BadRequest(message: string)`,
`Services(error: ServiceError)` and `Transport(error: HttpError)`.
Builder and lifecycle operations preserve recoverable service/transport causes.
Use BadRequest for explicit recoverable input failures in a context handler.

## Example and limits

The [reference app](/cases/neo-web/) demonstrates DI factories, configuration,
logging, awaited handlers and shutdown. See the
[feature guide](/features/neo-web/) for the supported slice.

General hosting, query/body binding, arbitrary result serialization, streaming,
authentication, automatic HTTPS, configuration binding and process signal
integration remain future work. Existing libraries stay independently usable.
The generic handler layer requires matching development compiler/runtime support.
See the [follow-on validation record](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/neo-web-generic-20261031/README.md)
for execution-mode evidence and limitations. The earlier context-only foundation
has [separate historical evidence](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/neo-web-20261031/README.md).

## Generic handler contract

`T: func` supplies a structural member contract, including the read-only `Function`
property. It does not require a nominal delegate or invent a callable signature.
Neo.Web reads `handler.Function` once during registration. Parameter metadata,
binding attributes, route-name matching and result adaptation are retained in the
endpoint descriptor. Its parsed route and executable callback are published only
after validation succeeds. Generic registration parses the pattern once.
Each request supplies its own argument array and uses checked dynamic invocation;
boxing, argument validation and ordinary DI resolution still occur per request.
No runtime code generation is required.

Parameters bind in this order:

1. `FromRoute` or `FromServices` explicitly selects the source, overriding inference.
2. `Neo.Web.HttpContext` receives the active request context.
3. A parameter whose name matches a route capture binds as `string` or `int`.
   Invalid integer text produces 400 without invoking the handler.
4. Remaining reference parameters resolve through the request service provider.
   Missing services produce 500. Registration does not instantiate services to
   determine availability, and does not currently validate the entire DI graph.

Open signatures, ref/out parameters, unsupported route types, unbound scalar
parameters and unsupported return types fail registration with `InvalidOptions`.
No query/body binding, default argument substitution, optional service policy or
conversion beyond route `int` parsing is provided. Source parameter names matter;
generated adapters without useful names cannot infer route binding.

## Binding attributes (development)

Both attributes target handler parameters and are read through the function's
parameter metadata once, at registration. They do not alter direct function calls.
Native/JIT generic endpoints must retain custom-attribute metadata for the handler's
declaring type or declared module, even if that handler has no binding attributes:
otherwise the planner cannot establish whether an override exists. Missing retention
is an explicit runtime capability fault, not a silently ignored annotation. The
sample's project policy demonstrates [module attribute retention](https://github.com/marinasundstrom/neoCLR/blob/main/docs/native-retention.md);
this is a development tooling requirement, not intended final application ceremony.
Two binding-source attributes on one parameter fail registration with InvalidOptions.
Unrelated attributes do not affect binding.

| Type/member | Contract |
| --- | --- |
| `FromRouteAttribute()` | Binds string/int from a route capture. `[FromRoute]` uses the parameter name. |
| `var FromRouteAttribute.Name: string?` | Defaults to null; `[FromRoute(Name: "id")]` selects that capture instead. Empty or missing capture names reject registration. |
| `FromServicesAttribute()` | `[FromServices]` resolves the declared reference type from the request scope, even if the parameter name matches a capture. Explicit string services are supported. |

Route names follow HttpServer's current case-sensitive capture contract. Open,
ref/out, pointer, function and value-type service parameters remain unsupported.
Services are activated per request; absence/activation errors produce the existing
sanitized 500 response. Route conversion errors produce 400 before the handler runs.

WebDemo's imported `Greeting` handler uses `FromRoute(Name: "name")` on its `visitor`
parameter; its service remains inferred. Its unit tests still call it with ordinary
arguments. Query, header and body source attributes remain unimplemented rather
than being accepted without a matching binder.

## Typed buffered results

`HttpResult<T>` is a standard Raven union, closed over four alternatives:

| Case | Payload and response |
| --- | --- |
| `Ok(value: T)` | Typed payload, HTTP 200. |
| `Created(value: T)` | Typed payload, HTTP 201. |
| `BadRequest` | No payload, HTTP 400. |
| `NotFound` | No payload, HTTP 404. |

`val StatusCode: int` is a read-only instance property on the union. It projects
200, 201, 400 or 404 from the active case for every `T`, without executing the result,
serializing its payload or resolving services. It is non-nullable because every
case has a defined status. The integer matches HttpContext's response methods and
requires no System.Web.Http import. The response writer consumes this same property;
there is no separate mutable status that could disagree with the case.

Values preserve `T` independently of hosting. The initial endpoint adapter accepts
`HttpResult<string>` and `Task<HttpResult<string>>`; successful strings become
UTF-8 `text/plain` responses and the other cases have empty bodies. Other payload
types can be constructed and tested, but fail endpoint registration until a writer
is supported. No implicit JSON serialization or arbitrary `Task<T>` adaptation is
provided. A match can cover all four alternatives without a wildcard:

```raven
func Describe(result: HttpResult<string>) -> string => match result {
    HttpResult<string>.Ok(let value) => value
    HttpResult<string>.Created(let value) => value
    HttpResult<string>.BadRequest => "400"
    HttpResult<string>.NotFound => "404"
}
```

The generic union is the bounded implementation of the proposed closed result
hierarchy. The payload type is on the union; nongeneric `HttpResult.Ok<T>` nested
class cases and existential generic-case patterns are not part of this API.
The current return type admits all four cases; endpoint-specific subsets and
independent payload types per alternative remain future work.

Framework behavior is tested in `extensions/Neo.Web/test`. WebDemo's separate test
project references the sample and calls its ordinary handler functions directly,
providing configuration/services as arguments and matching the returned results.
It needs no test server or separate web testing framework.
