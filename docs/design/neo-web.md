# Neo.Web hosting and minimal API design proposal

**Proposal, 2026-10-30.** Build an optional `Neo.Web` extension over
`System.Web.Http.HttpServer`, composing `Neo.DependencyInjection`,
`Neo.Configuration` and `Neo.Logging`. Expose `WebApplicationBuilder`,
`WebApplication` and generic endpoint registration. Structural function types are
sufficient for handler identity; a nominal Delegate base class is unnecessary.
Parameter binding, result adaptation and lifetime ownership are the main work.

This revisits the previously deferred hosting layer in the
[Web API plan](../web-api-plan.md). It does not replace the
[platform roadmap](../platform-roadmap.md), change release priorities, or resume
broader structural-type work. All Neo.Web declarations and examples below are
**uncompiled proposal notation**, not public APIs or executable samples.

## Baseline and deliberate differences

ASP.NET Core's [WebApplication and WebApplicationBuilder](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/minimal-apis/webapplication?view=aspnetcore-10.0)
provide the composition and application model. Adopt that separation and familiar
MapGet/MapPost naming. General hosting, background services and host-environment
abstractions can be extracted later when a second concrete host needs them.

ASP.NET Core's [RequestDelegateFactory implementation](https://source.dot.net/Microsoft.AspNetCore.Http.Extensions/RequestDelegateFactory.cs.html)
inspects a handler's method and creates a uniform HTTP callback. Neo.Web follows
that adaptation model with a structural function parameter F. Its first invocation
path uses the existing checked boxed-call facility, rather than assuming an
expression compiler or runtime code generation service exists. Boxing and argument
arrays are costs; this is not a performance improvement claim.

[ASP.NET Core parameter binding](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/minimal-apis/parameter-binding?view=aspnetcore-10.0)
is the semantic baseline for route, query, service and body inputs. Neo.Web starts
with fewer sources and result shapes. Unsupported shapes are gaps diagnosed at
startup, not new permanent platform restrictions. Retain neoCLR's documented
UTF-8 route semantics and typed recoverable errors; explicitly document their
differences from ASP.NET rather than implying full compatibility.

## Assembly and ownership boundaries

Place implementation in
`extensions/Neo.Web/src/Neo.Web/Neo.Web.rvnproj`, contracts in a sibling
`Neo.Web.Abstractions` project and tests under `test/Neo.Web.Tests`.
`Neo.Web.Abstractions` contains request context, endpoint binding/result contracts
and web errors; it references System.Runtime, System.Web and the DI/configuration/
logging abstractions as needed. `Neo.Web` owns builders, routing, invocation plans
and default integration, referencing the default extension implementations.
Keep JSON integration optional in `Neo.Web.Json`; it can depend on System.Data
without forcing every web application to carry JSON mapping.

The runtime remains independently usable. No dependency from System.Web to Neo.Web,
no new friend-assembly grant and no hosting-specific compiler primitive are proposed.
Do not introduce `Neo.Hosting` merely to hold a single implementation's lifecycle.

## Public API shape

The essential proposed surface is:

```raven
public class WebApplicationBuilder : Disposable {
    val Services: ServiceCollection
    val Configuration: ConfigurationBuilder
    val Logging: WebLoggingBuilder
    val Server: WebServerOptions
    func Build() -> Result<WebApplication, WebBuildError>
    func Dispose()
}

public class WebApplication : Disposable {
    static func CreateBuilder() -> WebApplicationBuilder
    val Services: ServiceProvider
    val Configuration: ConfigurationRoot

    func MapGet<F>(pattern: string, handler: F)
        -> Result<RouteHandlerBuilder, WebRegistrationError> where F: func
    func MapPost<F>(pattern: string, handler: F)
        -> Result<RouteHandlerBuilder, WebRegistrationError> where F: func
    func MapContext(method: string, pattern: string,
        handler: (WebContext) -> Task<Result<unit, WebError>>)
        -> Result<RouteHandlerBuilder, WebRegistrationError>

    func RunAsync(cancellationToken: CancellationToken = default)
        -> Task<Result<unit, WebRunError>>
    func Stop()
    func Dispose()
}
```

`WebLoggingBuilder` collects minimum level and provider registrations; its initial
`AddConsole()` convention is applied to the DI-owned LoggerFactory at Build.
`WebServerOptions` holds one validated address, port, backlog and HttpServerOptions.
Default to loopback port 5000 and backlog 128; default transport limits remain those
of HttpServerOptions. Explicit options override optional configuration keys
`Web:Address`, `Web:Port` and `Web:Backlog`. Invalid settings fail Build.
Multi-listener hosting and automatic HTTPS certificates are later work.

CreateBuilder initializes an empty configuration builder, default service
collection and Information logging with no output provider. It does not silently
load files, environment variables or command-line settings. Applications explicitly
add existing configuration sources, including JSON when referenced. These reduced
defaults avoid implying ASP.NET host-provider parity. An args-taking overload waits
for a defined command-line provider and precedence contract.

Build constructs the configuration root, registers it as a borrowed DI instance,
adds the logging defaults, then builds the provider. It transfers owned resources
to the application only on success. It does not open a socket. A successful builder
is consumed; further Build returns an invalid-state error. Disposal before success
releases any builder-owned resources. Failure cleans up partial owned resources
and leaves the builder terminal; create another builder to retry. Existing
configuration providers that terminate on Load failure still terminate: WebBuildError
does not promise recovery from those faults.

Service registration is snapshotted by the provider. Configuration values remain
live through the root, but listener options, endpoint binding and DI registrations
are fixed for a run. Configuration reload must not silently restart listeners or
reclassify handler parameters. Caller-supplied service instances remain caller-owned.

Use standard Raven unions for recoverable web errors, preserving underlying causes:

| Error contract | Proposed cases |
| --- | --- |
| `WebBuildError` | InvalidState, InvalidOptions, Services(ServiceError). Configuration failures remain governed by the existing provider contract. |
| `WebRegistrationError` | InvalidState, InvalidPattern, InvalidBinding, DuplicateName. |
| `WebRunError` | Preparation(endpoint diagnostics), Listen(HttpError), Server(HttpError), InvalidState. |
| `WebError` | BadRequest, ServiceFailure(ServiceError), InvocationFailure(ReflectionError), InvalidResult, Transport(HttpError). |

Diagnostic cases carry a useful message and endpoint/parameter identity where
available. Preparation collects endpoint diagnostics before listening. These unions
describe supported recovery paths; terminal runtime faults remain outside them.

## Minimal application example

```raven
async func Main() -> int {
    use builder = WebApplication.CreateBuilder()
    builder.Server.Port = 8080
    builder.Logging.AddConsole()

    let Ok(app) = builder.Build() else { return 1 }
    use owned = app

    let Ok(_) = app.MapGet("/", () => "Hello from neoCLR") else {
        return 1
    }
    let Ok(route) = app.MapGet("/items/{id}", (id: int) => {
        return id.ToString()
    }) else {
        return 1
    }
    let Ok(_) = route.FromRoute(0, "id") else { return 1 }

    let Ok(_) = await app.RunAsync() else { return 1 }
    return 0
}
```

The annotation on `id` is needed to establish the lambda's signature: a generic F
alone provides no contextual parameter type. Locals otherwise use inference.
Explicit position-based binding makes this example independent of lambda parameter
name retention. Inference from a fully typed lambda or named function into F must
still be qualified through a separately compiled Neo.Web assembly. An explicitly
typed function local/type argument is an acceptable initial fallback.

## Registration and routing

MapGet/MapPost validate the route pattern and retain the handler. They return a
RouteHandlerBuilder with the following proposed configuration operations, each
returning `Result<unit, WebRegistrationError>`:

| Operation | Meaning |
| --- | --- |
| `FromRoute(position, name)` | Bind a parameter from a named route capture. |
| `FromQuery(position, name)` | Bind from a query key in the later query slice. |
| `FromServices(position)` | Resolve the parameter type from the request scope. |
| `WithName(name)` | Set a diagnostic endpoint name, unique within the app. |

Zero-based positions refer to public function parameters, excluding receivers and
captures. Reject out-of-range positions and conflicting source declarations.
Immediately invalid patterns return registration errors without adding an endpoint.
Final binding checks run when RunAsync freezes the registry, before listening;
late registration/configuration returns an invalid-state error. Fixable preparation
errors leave the application configurable. Once serving has started the application
is single-run, including after a listener failure; recreation is the restart path.

Reuse [RoutePattern](../route-parsing.md): compile patterns once, preserve its
case-sensitive literals, slash distinctions, percent-decoding and limits. There is
no new `{id:int}` syntax in the initial slice. For each method, literal segments
outrank parameter segments at the first differing segment. Equal-precedence
overlapping patterns are rejected before listening, including `/items/{id}` and
`/items/{name}`. Registration order does not break ambiguity. Implement comparison
from the parsed route structure; if no public segment inspection is available, add
a suitable routing contract or a validated library parser, not internal access.

A malformed request target produces 400. No path match produces 404. A matching
path without a matching method produces 405 with Allow derived from registrations.
Do not implement automatic HEAD-to-GET dispatch or OPTIONS in the first slice;
explicit method registration can follow. These routing policies are new framework
work; the existing routed sample's choice of 404 for unsupported methods remains
its own policy.

## Generic handlers and endpoint adaptation

`where F: func` accepts structural function types and retains their exact signature.
It does not allow an open generic body to write `handler(a, b)` for every F.
Neo.Web creates an endpoint-specific plan containing:

1. The retained bound function, including its receiver/captures.
2. Ordered parameter types, sources and conversion operations.
3. A supported result adapter selected from the closed return type.
4. A uniform `(WebContext) -> Task<Result<unit, WebError>>` callback.

The heterogeneous endpoint registry stores uniform callbacks and route plans. An
internal `Endpoint<F>` may retain F, or the checked invocation path may retain its
Object view; both preserve binding identity. Neither requires a Delegate base.
Wrap the dispatcher once in HttpServer's existing
`(HttpContext) -> Task<Result<unit, HttpError>>` callback.

Use the **Result-returning Object overload** of FunctionReflection.Invoke for the
initial boxed adapter. Argument binders produce exact supported boxes/references;
reflection errors remain explicit adapter failures. The development generic
`Invoke<F>` returns `ReturnType<F>` but panics on validation failure. It is useful
for typed forwarding, not required for the first endpoint dispatcher. A future
typed adapter can use it only after establishing the invocation contract.

Whole-signature declarations such as `func Invoke: F with args` solve a different
problem: exposing a callable method whose signature follows F. A route dispatcher
starts with an HTTP context and constructs arguments; it does not need to expose
that projected method. Neither parameter packs nor projected async methods are
prerequisites for the first Neo.Web implementation.

Build plans once per endpoint. Signature-only conversion helpers may be shared by
type, but a complete plan cannot be keyed solely by F: `(int) -> string` could bind
`id` on one endpoint and `page` on another. Include target/binding policy in cache
identity, and do not globally cache arbitrary request input.

## Parameter metadata and binding

Existing FunctionTypeInfo exposes ordered types and modes; its synthetic parameter
names are empty and attributes are absent. A function value's `Function` property
describes the actual bound target. That distinction is essential: structural type
identity must not depend on source parameter names.

Use explicit RouteHandlerBuilder metadata first. Otherwise infer special request
types, named route captures, registered services, then supported named query
scalars, in that order. In the first slice only route string/Int32, request context
and explicit service binding are enabled; later inference must preserve this
documented precedence. Never probe service availability by constructing services.
Use the build-time descriptor snapshot, or add a provider-neutral service-availability
query before offering inference with replaceable providers.

| Input | Initial policy and remaining work |
| --- | --- |
| Route `string` / `int` | Use Get/GetInt32; missing or invalid values produce 400 and do not invoke the handler. |
| `WebContext` / `HttpRequest` | Supply request-owned values; callers must not retain them past the request. |
| Reference service type | Resolve by TypeInfo from a per-request scope; missing explicit registration is a startup error, activation failure is a logged 500. |
| Query scalar | Follow-up library binder: define duplicate keys, decoding, empty/missing values and byte/count limits before enabling. |
| JSON body | Follow-up Neo.Web.Json binder; one body consumer, explicit body source initially, supported mapper types only, bounded input and media-type validation. |
| Nullable/default/Optional | Follow-up binding policy; function signatures alone cannot supply declaration defaults. Do not equate Optional with missing input without a specified rule. |
| CancellationToken | Later typed adapter or value-marshalling support; initially expose application-stopping cancellation through WebContext. Do not call it per-request disconnect cancellation. |
| ref/out, arbitrary value aggregates | Reject during preparation until explicitly supported by invocation and binding. |

Convenient name/attribute inference requires probes for named module functions,
instance methods, captured/noncaptured lambdas and compiler-generated adapters.
Verify names, defaults, annotations and closed generic types after import and native
retention. Target metadata currently has limitations for module-function attributes
and generated adapters. If a name is unavailable, return a useful preparation
diagnostic requiring explicit position binding; never guess by route order.
If compiler changes preserve declaration metadata through an adapter, keep that
metadata separate from structural signature equality and record its provenance.

## Results and asynchronous handlers

Return-shape adaptation is library policy. Inspecting a return type does not make
an open F's result statically awaitable or provide an existential `Task<T>` await.
Start with a finite set of concrete result adapters:

| Handler result | Proposed HTTP behavior |
| --- | --- |
| Non-null `string` | 200, UTF-8 text/plain, buffered owned content. Null is an invalid result and becomes a logged 500. |
| `WebResult` | Execute the explicit status/header/body policy. Initially framework-owned buffered results only. |
| `Task<string>` / `Task<WebResult>` | Cast to that exact supported Task shape, await, then use the same result policy. |
| `unit` / `Task<unit>` | Later 204 convention, gated on unit invocation qualification in all supported modes. |
| `T` / `Task<T>` arbitrary DTO | Later explicit JSON result adapters/codecs, not automatic reflection-based generic Task construction. |
| `Result<T,E>` | Explicit adapter with an application-selected error-to-response mapping; no universal E-to-status rule. |

`WebResult` is a proposed reference contract implemented initially by sealed
framework result classes returned by `Results.Text`, `Results.Empty(status)` and
`Results.Status(status, text)`. Their content is independent of request-scoped
services when execution completes. JSON adds `Results.Json<T>` only for qualified
mapper types. Arbitrary custom streaming result implementations are deferred.

The normalized context callback can return `Result<unit, WebError>` directly,
without sending that union through boxed handler reflection. Transport errors,
binding errors and application error results remain distinct. A bad client input
can become 400, an internal service/adapter failure a sanitized 500, and a configured
application error its selected status. Do not send reflection or service diagnostic
details to clients. Target Panic remains terminal and is not caught as a 500.

For a valid configured error response the HttpServer callback returns Ok so Serve
sends it. Returning HttpError from that callback closes the exchange instead of
generating an HTTP error response. Awaited handler cancellation must settle the
request scope through the existing Task outcome/cleanup facilities; qualify this
explicitly rather than assume exception-style finally semantics. A documented
outcome-observation adapter may use OnCompleted when cancellation bypasses ordinary
await continuation. Ordinary asynchronous handler code uses await.

## Application and request lifetimes

RunAsync prepares endpoints, opens HttpServer and awaits Serve. Stop requests the
application-stopping token and calls Close; the RunAsync task remains pending until
started handlers settle. Cancellation of RunAsync follows the existing cancelled
Task contract rather than inventing a WebRunError.Cancelled. Listener and startup
failures preserve their typed causes. Close currently ends connections immediately;
this is joined shutdown, not graceful response draining. A handler that never
settles can prevent completion. No forced managed-task abort is promised.

WebApplication.Dispose is idempotent after RunAsync has settled or before running.
It releases root services, then the application-owned configuration root. Calling
Dispose during an active run is a lifecycle misuse and must not free dependencies
under running requests: require Stop followed by awaiting the retained run task.
Async disposal and a general host's coordinated shutdown policy are later work.
Startup failure releases partial listener/run resources; the built application
retains its root services/configuration until Dispose. Caller-owned instances remain borrowed.

Each selected endpoint gets a scope and WebContext exposing Request, Services,
route values and ApplicationStopping. Keep the scope alive across awaited handler
execution and result materialization. The first slice admits only response content
that is fully materialized and independent of scoped resources before releasing
the scope. The raw transport context stays private to the dispatcher; MapContext
uses WebContext and its controlled buffered-response operations.

This restriction matters because [HttpServer.Serve](../../api-docs/http-server.md)
sends the response **after** the handler returns. Disposing a service-backed stream
at callback return would be premature. Before enabling streaming/custom content,
choose and qualify one of these ownership mechanisms:

- A general System.Web exchange-completion registration, invoked exactly once after
  content transfer/disposal or failure, retaining scope cleanup through shutdown and
  late callbacks. This is a managed HTTP-library contract unless implementation
  exposes a demonstrated runtime gap.
- A response-content ownership wrapper that releases the scope on disposal, only
  if all no-body, replacement, rejection, cancellation and late-response paths prove
  that disposal is sufficient and exactly once.
- A supervised Accept/Complete dispatcher that owns the scope until Complete settles.
  Existing APIs permit this but require reimplementing bounds, task observation and
  joined cleanup already provided by Serve; it is not the preferred first slice.

Do not retain every scope until server shutdown as a workaround. Do not promise
cleanup after terminal Panic beyond the platform's existing cleanup contract.
Logging initially uses explicit request/endpoint fields; Neo.Logging has no ambient
async scope flow. Concurrent HTTP progress remains cooperative serialized managed
execution, not a new shared-memory multithreading contract.

## Existing capabilities and required work

“Existing” below means repository development support, not a promise about every
published package or every combination of features.

| Capability | Current evidence | Neo.Web work or support gap |
| --- | --- | --- |
| HTTP dispatch and shutdown | [Serve contract](../../api-docs/http-server.md), [source](../../runtime/System.Web/src/System.Web/System/Web/Http/HttpServer.rvn) | Compose lifecycle and buffered responses in library code; completion-owned scopes for streaming need a further contract. |
| Route matching | [Route parser and executable evidence](../route-parsing.md) | Registry, precedence, ambiguity, 405 and binding plans are new library work. |
| DI scopes and runtime-type resolution | [DI contract](../../api-docs/dependency-injection.md) | Compose per-request lifetime. Native constructor retention remains required; generic implementation activation has limitations. Factories are an existing fallback. |
| Configuration and logging | [Configuration](../configuration.md), [logging](../../api-docs/logging.md) | Builder integration, options precedence and request diagnostics are new. Automatic host providers and ambient logging flow are absent. |
| Structural F constraints and retention | [Function-kind qualification](../validation/function-kind-constraint/README.md) | Generic method/class constraints exist; qualify actual Neo.Web lambda inference, separate-provider imports and callback lifetime. No nominal Delegate requirement. |
| Signature and target reflection | [Function reference](../../api-docs/functions.md) | Types/modes and bound Function metadata exist. Qualify parameter names/defaults/attributes and metadata roots; explicit binding avoids source-name dependency. |
| Checked boxed invocation | [Dynamic invocation evidence](../validation/dynamic-function-invocation/README.md) | Existing scalar/reference profile has interpreter/JIT/AOT evidence. The assembled HTTP adapter and retained return shapes still require qualification. No arbitrary aggregate marshalling. |
| ReturnType and typed Invoke | [Generic function method evidence](../validation/generic-function-methods-20261030/README.md) | Implemented development support, with interpreter consumers. Native signal execution is blocked by inhabited-unit boxing; do not infer full JIT/AOT qualification. Not required for the initial Object-overload path. |
| Async result processing | Existing ordinary Task/await and Serve | Concrete Task<string>/Task<WebResult> adapters are library work to qualify. Automatic arbitrary Task<T> adapters need generated closed code or an explicit typed adapter API; generic reflection alone does not provide this. |
| Unit and aggregate invocation | [Current projection limits](generic-function-methods.md) | Runtime/codegen unit boxing/unboxing qualification and broader value marshalling are needed for those boxed shapes. Avoid blocking the initial string/reference slice on them. |
| Automatic optimized adapters | No Neo.Web generator | Compiler/build-tool work only if selected: preserve source metadata, emit direct calls and retain specialized async/result adapters. Not required for correctness of the bounded reflection slice. |

Compiler-facing changes, if probes demonstrate a gap, belong behind the explicit
neoCLR Runtime Contract. General inference/emission fixes should be developed and
validated on Raven main and carried deliberately to `codex/neoclr`. Native-only
metadata support stays target-specific. Record semantic/emission changes in both
repositories' bridge/compiler docs and changelogs. Current projected signatures use
native format 6; CLI executable and neoIL text export are unsupported. CLI reference
documentation must use linked manual entries for unrepresentable signatures.
No new transport carrier or Runtime Contract capability is specified by this proposal.

## Alternatives and tradeoffs

| Approach | Benefit | Cost and decision |
| --- | --- | --- |
| Fixed WebContext callback only | Fully known call shape, no handler reflection | Useful initial host acceptance and escape hatch, but does not deliver automatic parameter binding. |
| Generic F with checked boxed invocation | Any admitted arity through one registration API; captures retained | Per-request array/boxing and limited marshalling; chosen bounded minimal API baseline. |
| Arity-specific typed overloads | Direct calls and explicit typed binders/writers using existing generics | Overload growth and inference ambiguity; useful optional fallback, not the sole long-term public model. |
| Generated endpoint adapters | Direct binding/invocation and explicit metadata retention | Build integration, compiler/tool coupling, code size and diagnostic work; consider after measuring the baseline. |
| New Delegate base or variadic generics | Could change the expression of the problem | Neither is necessary for this design; would expand runtime/language scope without removing HTTP policy work. |

Preserve validated route/input primitives internally and create public wrappers only
where they add meaning. Reuse supplied configuration/metadata, cache endpoint plans,
and avoid reflection per request beyond the initial invocation path. Measure before
claiming typed generation or fewer wrappers improve throughput or allocation.

## Bounded implementation and validation sequence

1. **Hosting and explicit context endpoint.** Implement builder ownership, frozen
   registry, route matching and buffered MapContext responses over Serve. Test build
   failures, registration freeze, 400/404/405, stop/join and disposal ordering with a
   real HttpClient consumer. No new compiler machinery should be assumed necessary.
2. **Generic scalar/reference handlers.** Add MapGet<F>/MapPost<F>, explicit route
   positions, scoped reference services, checked invocation and string/WebResult
   output. Probe named functions, typed lambdas, captures and a separate provider
   assembly in interpreter/JIT/AOT. Reject unsupported signatures before listening.
3. **Concrete async handlers and metadata convenience.** Qualify Task<string> and
   Task<WebResult>, cancellation cleanup, service disposal after awaited use, and
   name-based binding where target metadata survives. Add startup diagnostics and
   explicit-binding fallback for missing metadata. Fix only demonstrated compiler
   or native metadata gaps.
4. **Broader binding/results and streaming.** Add query/JSON policies, explicit
   application Result adapters and unit results as their prerequisites pass. Add
   completion-owned scopes before service-dependent streaming. Keep general hosting,
   middleware/filter pipelines, auth, OpenAPI and arbitrary Task<T> generation out
   of the first acceptance slice.

Use descriptive NeoClr.Testing tests for framework behavior and thin host harnesses
only for independent HTTP peers, packaging and execution-mode orchestration. Include
negative controls: equal F with different parameter names, ambiguous patterns,
invalid scalar input without handler entry, unsupported ref/out/aggregate shapes,
missing metadata, DI activation failure, exact-once disposal and cancelled tasks.
Exercise overlapping requests to detect scope reuse and closure-rooting errors.
Use a subprocess for terminal-fault assertions; do not disguise Panic as recovery.

Before public API delivery, add RavenDoc selection, XML summaries, signatures,
errors and examples, matching reference artifacts and manual coverage for any
unrepresentable generic signatures. Publish feature content only from tested
samples, clearly marking development scope. This proposal does not require a website
build. Baseline HTTP/allocation measurements become relevant when choosing the
optimized adapter path; no benchmark is needed to establish this design document.
