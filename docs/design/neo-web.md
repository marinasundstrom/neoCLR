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

## First experimental implementation

Neo.Web is an experimental web framework built on top of
`System.Web.Http.HttpServer`. The author selected the context-handler hosting
slice on 2026-10-31. Its concrete
contract is documented in [Neo.Web API](../../api-docs/neo-web.md), with an evolving
[reference application](../../samples/web/README.md). The author clarified that the
web-facing context should be named `Neo.Web.HttpContext`, normal applications
should not need `System.Web.Http` imports, and Neo.Web remains experimental.

Long-term direction (author clarification, 2026-10-31): improve the underlying
primitives, especially HttpServer, and let WebApplication benefit through composition.
Keep connection ownership, transport scheduling and server completion policy below
Neo.Web; do not duplicate them in the application layer.

The intended experience is ASP.NET Core-like minimal APIs with services injected
into handler parameters. Explicit `context.Services` lookup in this first slice
establishes request-scope lifetime; it is not the final handler-binding experience.
The next handler slice should qualify mixed route/service parameters and preserve
scoped dependencies across awaited work before expanding binding convenience.

The first implementation uses a single optional Neo.Web assembly, a default owned
LoggerFactory, explicit listener options, MapContext returning Result<unit, WebError>,
and a shared WebError union. It does not introduce the proposed abstraction split,
WebLoggingBuilder, RouteHandlerBuilder or configuration-to-listener binding yet.
Internally it uses HttpServer's response callback so 405 responses can carry Allow
through existing public APIs; no transport extension or friend grant is needed.
The remaining sketches below preserve the broader proposal, not the implemented API.

## Implemented follow-on (2026-10-31 development)

The bounded follow-on now includes generic MapGet/MapPost with cached route,
context and service binding, the closed HttpResult<T> union, all-method Map,
terminal Run, and fixed-shape Use middleware. A single scope spans the outermost
middleware task, including work after next returns. The host and direct handler
tests share WebDemo.Logic. See the [implemented contract](../../api-docs/neo-web.md)
and [execution evidence](../validation/neo-web-generic-20261031/README.md);
query/body binding, general serialization and broader hosting remain future work.
The proposal sketches below retain their exploratory status.

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

**Historical proposal:** the position-based From* operations below are superseded
by the author's 2026-10-31 direction toward parameter binding attributes. See
[endpoint binding and ambiguity](#endpoint-binding-and-ambiguity-2026-10-31).
They are not implemented APIs.

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

**Author direction, 2026-10-31:** Typed handler results are next-slice work toward
the C# minimal API experience. The representation may be either a sealed hierarchy
or a union; neither is selected by this first slice. Preserve the declared response
alternatives and their payload types through registration and result adaptation,
rather than requiring applications to erase them to an untyped response object.
HTTP result objects should themselves be typed at construction, including their
payload and response contract. A handler's sealed hierarchy or union describes the
alternatives among those already-typed results; registration does not retrospectively
add types to an untyped result object. Keep concrete result types through factories
and handler returns, even if execution eventually uses a uniform internal adapter.
The context-handler `Result<unit, WebError>` below remains the implemented contract;
it does not implement this richer response model.

The next result-adapter slice should qualify a concrete multi-result handler and its
awaited counterpart, define each alternative's status/body policy, and diagnose
unsupported alternatives at registration. Exhaustive handling and response metadata
must follow the chosen representation's actual compiler/runtime capabilities.
Nominal sealed result classes can use the existing reference invocation path;
union carriers or generalized closed result families need separate invocation and
async-adapter qualification. Do not equate an existing union declaration with
support for arbitrary union payloads through boxed reflection. This direction
supersedes treating the initial WebResult reference sketch as the final result model.

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
The first hosting consumer exposed a native codegen capacity limit: type-shape
reflection duplicated equal flag-result bodies per token. Sharing those bodies
retains token checks, selector validation and the existing per-method bound; it
does not change Raven metadata or the reflection contract.

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

## Pipeline direction and bounded terminal registrations (2026-10-31)

The author requested a way to handle any request, with or without a route pattern,
and to think of Neo.Web as a pipeline with future middleware, similar to ASP.NET
Core. The [ASP.NET Core middleware contract](https://learn.microsoft.com/aspnet/core/fundamentals/middleware)
uses terminal Run handlers and wrapping middleware with a next continuation.
Its Map middleware branches by path prefix; endpoint routing is a separate concern.

Neo.Web's bounded foundation uses the existing structural context function shape:

```raven
(HttpContext) -> Task<Result<unit, WebError>>
```

`Map(pattern, handler)` registers a full route pattern for every method. `Run(handler)`
sets the one terminal, unpatterned stage; `RunAsync` still owns listening/lifetime.
Malformed targets terminate with 400, method-only mismatches with 405, and unmatched
paths reach the terminal handler (or default 404). Both terminal and endpoint
handlers use the same scope and response-completion adapter. Empty terminal route
values are obtained once through the public route matcher and reused; request
paths are not reparsed to manufacture an empty match on the hot path.

A method-specific route wins over an all-method route at equal pattern precedence.
Overlapping all-method routes of equal precedence reject. The terminal stage is
configured once and always follows routing, independent of registration order.
These are deliberate bounded differences from an ordered middleware builder and
path-prefix Map: predictable routing now, but no arbitrary pipeline insertion yet.

Future `Use` can wrap a fixed next function with the same context/result contract.
Define before/after ordering, short-circuiting, scoped lifetime across the whole
pipeline, cancellation and response completion before adding it. Generic endpoint
binding remains an adapter into this terminal contract; it is not the pipeline
contract itself. General hosting and streaming are still separate work.

The author then explicitly requested middleware implementation in this slice.
`Use(handler)` now takes `(HttpContext, next) -> Task<Result<unit, WebError>>`, where
next has the terminal function shape above. RunAsync composes immutable wrapper
steps once, reversing the registration list to make the first middleware outermost.
The innermost step performs validation/routing/terminal handling. A single request
scope and response adapter surround the complete composed pipeline; post-next work
therefore retains scoped services. RouteValues starts empty and is populated by
routing on the same context. Failure and cancellation remain observable through
the task/result contract. Middleware may short-circuit before routing.

Call next at most once with the supplied context and await it. Retained/reentrant
continuations and fire-and-forget execution are outside the contract. No general
middleware service activation, explicit UseRouting insertion or branch-builder API
is introduced. WebDemo's LogRequest function demonstrates before/after logging;
framework tests cover ordering, short-circuiting, failure, cancellation and scope
lifetime with real HTTP requests.

## Common result members (author direction, 2026-10-31)

The author clarified that unions allow members: `HttpResult<T>` can expose common
response properties, such as `StatusCode`, by projecting information from its active
case. The current union need not become a class hierarchy to provide that contract.
The status property exhaustively maps Ok to 200, Created to 201, BadRequest to
400 and NotFound to 404 while retaining typed case payloads.
The bounded follow-up now implements `val StatusCode: int` with that exhaustive
projection and uses it in the response writer. The integer follows the existing
HttpContext response methods without requiring a transport namespace import.

Keep sealed/closed class hierarchies as a future alternative, evaluated for their
own requirements rather than as a prerequisite for common result members. Unlike
nominal result interfaces/classes, union members can share the exhaustive case
model already used by direct handler tests. Additional common properties remain future design work: require information that
is meaningful for every case and preserve the typed case payloads. No mutable
status override or replacement of the union with a class hierarchy is introduced.

## Variant enforcement and ASP.NET Core familiarity (2026-10-31)

The author clarified the purpose: resemble ASP.NET Core result types while enhancing
variant enforcement. ASP.NET Core already supports typed results and
[`Results<T1, ..., Tn>`](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/minimal-apis/responses?view=aspnetcore-10.0#resultstresult1-tresultn),
which reject return types outside the declared alternatives and retain endpoint
metadata. Neo.Web should preserve that familiar typed-response experience rather
than claim that ASP.NET Core has no compile-time variant checking.

The intended additional benefit is Raven's language-level closed variants,
typed payload patterns and exhaustiveness checking, with common members derived
from the active case. The current `HttpResult<T>` closes the four supported cases
and fixes each case's status; it does not yet let each endpoint declare a narrower
subset or different payload types per alternative. Those richer result sets remain
future design work, together with any sealed-hierarchy alternative. The tradeoff
of the present union is that adding a case affects exhaustive consumers, and every
handler returning it advertises all four alternatives. Status projection alone
must not be described as completing that broader goal.

## Middleware registration consistency (development, 2026-10-31)

The author selected `TMiddleware : Middleware`, with `Middleware` an interface,
and requested middleware consistency before endpoint signature ergonomics.
The implemented typed path is `UseMiddleware<T>()`; the existing
`Use(handlerFunc)` remains the direct function-value path. Both compose
through exactly the same `(context, next) -> Task<Result<unit, WebError>>` pipeline.
The interface's `InvokeAsync` method has those same inputs and output.

Resolve the registered concrete middleware type from the request's service scope
when its stage is reached. Constructor dependencies therefore follow DI lifetimes;
short-circuited downstream middleware is not activated. The outer pipeline already
keeps the scope alive through post-next work and cancellation. Missing registration
or activation errors should propagate as WebError.Services and use the existing
sanitized failure response. Register services explicitly; pipeline registration
should neither mutate an already built provider nor instantiate scoped middleware
at startup. No method-name reflection or convention activation is needed.

This follows ASP.NET Core's [interface-based middleware activation](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/middleware/extensibility?view=aspnetcore-10.0)
in preferring strongly typed invocation and request-scoped resolution. The bounded
Neo.Web adapter can reuse its public DI service scope without introducing a separate
middleware factory service. A factory replacement/activation policy remains a later
extension point if needed. Function captures remain caller-owned; typed middleware
lifetime/disposal follows its DI registration. Neither form gains reentrant next,
retained continuations or independent scope ownership.

After that slice, review endpoint signatures: sync versus async typed results,
explicit context versus injected parameters, route/query/body/service binding and
startup diagnostics. Preserve the cached registration plan and structural function
contract. ASP.NET Core familiarity is the target, but unsupported binding families
remain gaps rather than silently guessed semantics.

## Endpoint binding and ambiguity (2026-10-31)

The author requests request-parameter binding and DI injection, including special
parameters for HttpContext, HttpRequest, HttpResponse and CancellationToken. They
clarified that explicit overrides are **binding attributes**, named FromBody and
FromServices, with the necessary source attributes to resolve ambiguities. This
supersedes earlier helper-method wording. FromRoute and FromServices are now
implemented in the descriptor slice below; query/header/body families remain planned.

The next binding slice should preserve the familiar [ASP.NET Core ordering](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/minimal-apis/parameter-binding?view=aspnetcore-10.0):
explicit source attributes, special framework types, supported scalar route/query
conversion, registered services, then supported body binding. The author selected the ASP.NET Core `FromServices` spelling.
Explicit attributes select a source rather than merely influencing a heuristic.
Conflicting source attributes and unsupported source/type combinations should fail
registration with parameter-specific diagnostics, never be silently ignored.

| Proposed parameter attribute | Ambiguity resolved |
| --- | --- |
| `FromRoute` | Select a route capture, with an optional external-name override. |
| `FromQuery` | Select a query value even when its name matches a route capture. |
| `FromHeader` | Select a header, including an external-name override. |
| `FromBody` | Select body deserialization rather than DI or scalar query inference. |
| `FromServices` | Select DI even when the type/name could bind from request data. |

Add form-specific attributes when form binding is actually supported; do not ship
an attribute whose binder cannot honor it. Keyed service selection, repeated query
values, optional/default parameters and null/empty-body policy require explicit
contracts and tests. Keep one consuming body binding per endpoint unless an
intentional shared-body model is introduced. Parsing/required-data failures should
produce sanitized client errors before invoking the handler; DI activation failures
remain server errors. Attribute constructors and allowed targets must be reflected
and cached at registration alongside the function signature, not reread per request.

### Existing foundations and required work

- HandlerPlan already caches function metadata, parameter names and route/service
  plans; it binds Neo.Web.HttpContext, named string/int route values and remaining
  reference-type services. There is no query/body/header inference yet.
- ParameterInfo exposes GetCustomAttributesData. Qualify parameter attributes on
  bound module functions, bound instance methods and imported function values in
  interpreter/JIT/AOT before relying on this bridge for binding attributes.
- HttpContext already retains the request. Response injection needs one stable
  response identity shared with the context: current 405 handling replaces its
  private response, so exposing that object blindly would leave stale aliases.
  Preserve the context's response-validation and lifetime rules when defining the API.
- CancellationToken should reflect request abort, as in ASP.NET Core, independently
  of ApplicationStopping. HttpServer currently exposes application/operation tokens
  but no public request-abort token on its context. Improve that primitive and
  qualify disconnect/stop/cleanup behavior before claiming equivalent injection.
- DI has no public nonactivating registered-service probe. Add an explicit provider
  capability or retain registration metadata before inferring service versus body;
  do not instantiate scoped services at startup merely to choose a binding source.
- Keep simple handlers free of transport imports. Decide how explicitly typed
  request/response parameters are presented through Neo.Web without introducing
  unnecessary duplicate request/response objects or claiming aliases already exist.

After middleware activation is qualified, the bounded endpoint implementation can
start with special parameters whose lifetime/identity is already supported and
FromServices plus route/query/header attributes. Body deserialization and request
abort need their own primitive/metadata checks. Keep sync/async handler inputs and
typed-result outputs consistent; richer result subsets and serializers remain
separate from inferring where an argument comes from.


Middleware registration returns `Result<WebApplication, WebError>` from `Use` and
`UseMiddleware`, permitting composition while preserving recoverable failures.
This does not imply all registration methods should return the application:
ASP.NET Core's [MapGet source](https://source.dot.net/Microsoft.AspNetCore.Routing/Builder/EndpointRouteBuilderExtensions.cs.html)
returns an endpoint convention/route-handler builder. Neo.Web retains unit results
for Map* in this slice; endpoint-specific configuration is future work. WebDemo
prefers separate `MapGet(...)?` statements. Conditional access (`?.`) continues to
map members into carriers; postfix `?` is the separate propagation operation.

The author subsequently selected the shorter generic name `T`: `UseMiddleware<T: Middleware>()`.


## Registration-owned descriptors and binding overrides (development, 2026-10-31)

The endpoint descriptor owns the parsed route, precedence, method and cached
execution callback. Generic handlers inspect Function, parameters, source attributes
and result shape before publication. The same parsed route feeds binding validation
and routing; it is no longer parsed independently by both paths. Failed descriptor
construction leaves the registry unchanged. Request argument arrays and services
remain request-owned, including overlapping async calls.

The bounded binding extension implements FromRoute and the author-selected
FromServices. It follows ASP.NET Core's [FromRoute Name property](https://raw.githubusercontent.com/dotnet/aspnetcore/main/src/Mvc/Mvc.Core/src/FromRouteAttribute.cs)
and [explicit service selection](https://raw.githubusercontent.com/dotnet/aspnetcore/main/src/Mvc/Mvc.Core/src/FromServicesAttribute.cs).
Name is a nullable mutable attribute property; null uses the parameter name.
Explicit selection precedes context/route/service inference. Conflicting source
attributes, missing captures and unsupported source/type combinations fail at
registration. Missing services remain a request activation error, not startup
activation. Explicit string service binding resolves a scalar/route ambiguity.

Current limitations remain visible: only parameter targets are admitted, route
names follow the existing case-sensitive HttpServer primitive (ASP.NET Core uses
case-insensitive names), and the framework does not yet expose common binding
metadata interfaces for custom providers. Query/header/body families need their own
binding plans and tests. In particular, future body inference needs a nonactivating
DI service probe and a cached serializer contract; do not activate scoped services
or inspect serializer properties on each request to infer a source.

This slice removes duplicate registration parsing and validates cached source
selection. It does not claim a throughput improvement: routing still scans endpoint
patterns, route values use named lookup, and checked dynamic invocation, argument
arrays/boxing and DI activation remain request costs. A prepared invocation adapter
and a route lookup index require separate runtime/primitive work and measurements
against direct HttpServer before claiming low-overhead parity.


Author clarification: use ASP.NET Core as a researched reference, not a mandatory
blueprint. The service/body inference requirement is a nonactivating availability
probe; a separate .NET-shaped IServiceProviderIsService interface is not required.
NeoCLR can expose the capability through an existing provider contract or a separate
capability. The former is simpler for consumers but changes provider implementations;
the latter supports optional provider capabilities but adds discovery and fallback
policy. Choose and qualify that contract with the body-inference slice. Do not infer
availability by resolving a service at registration. ASP.NET Core's
[factory implementation](https://raw.githubusercontent.com/dotnet/aspnetcore/main/src/Http/Http.Extensions/src/RequestDelegateFactory.cs)
is evidence of the separation between probing and request-time activation.

## Query/body and DI binding implementation (2026-10-31)

The author expanded the DI slice to query and request-body binding and identified
ASP.NET Core-like foundational behavior across interpreter/JIT/AOT as the goal,
including measured overhead against HttpServer. The implementation uses explicit
source overrides, framework values, scalar route/query binding, nonactivating DI
availability, then inferred POST JSON bodies. GET bodies require FromBody.

Primary comparison: [ASP.NET Core parameter binding](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/minimal-apis/parameter-binding?view=aspnetcore-10.0)
and [IServiceProviderIsService](https://github.com/dotnet/runtime/blob/main/src/libraries/Microsoft.Extensions.DependencyInjection.Abstractions/src/IServiceProviderIsService.cs).
Neo puts IsService on the existing provider: no extra capability service or
fallback activation is needed, at the cost of requiring custom providers to
implement the probe. Availability is not an activation guarantee.

Body plans cache constructor/property descriptors rather than repeatedly using
the serializer's metadata enumeration. The bounded DTO/scalar mapper keeps
case-insensitive web property matching, ignored unknown fields and constructor
defaults for omitted fields. Existing strict UTF-8 JSON limits remain visible;
arrays, nullable/defaulted endpoint parameters and general converters remain gaps.
Prepared invocation, transport-owned request-abort notification and comparative
benchmarks remain separate completion items; no throughput improvement is claimed.
