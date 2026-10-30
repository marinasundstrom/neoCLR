---
title: Experimental Neo.Web hosting
---
# Experimental Neo.Web hosting

Neo.Web is an experimental web framework built on top of
`System.Web.Http.HttpServer`, available as an optional development extension.
It is not part of published Preview 19.
Its API is provisional. It composes the existing extension services and HttpServer;
there is no generic minimal API handler binding yet. All public types are in the
`Neo.Web` module and `Neo.Web.dll` assembly.

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
| `GetLocalPort() -> Result<int, WebError>` | Actual listener port while running; otherwise InvalidState. |
| `Stop()` | Requests application cancellation and closes connections; no effect before running or after completion. |
| `Dispose()` | Idempotent outside a run. Disposes root services, default logging and configuration. Active disposal is terminal misuse: Stop and await the lifetime first. |

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
request. Request values must not escape the handler lifetime.

The request scope survives all handler awaits, then is disposed on success, typed
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

General hosting, automatic parameter/result binding, streaming/custom responses,
middleware, authentication, automatic HTTPS, configuration binding and process signal
integration remain future work. Existing libraries stay independently usable. No
compiler or runtime semantic extension is required by this context-handler layer.

This slice is qualified on macOS ARM64 with source-built native code generation.
The combined graph needs grouped type-shape lowering; no Raven syntax or runtime
metadata contract was added. The interpreter probe currently fails at Build despite
successful metadata verification. See the [validation record](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/neo-web-20261031/README.md)
for exact compiler/bundle revisions and execution-mode evidence.
