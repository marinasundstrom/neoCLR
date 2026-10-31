# Neo.Web reference application

Neo.Web is an experimental web framework built on top of
`System.Web.Http.HttpServer`. This is its reference application. Keep it
small and evolve it alongside hosting and generic minimal API handlers. It demonstrates
explicit configuration, logging, injected scoped services, singleton state, typed
buffered results, awaited work and joined shutdown. The follow-on implementation
requires the matching development compiler/runtime and is under qualification.

Composition and lifetime live in [Main.rvn](src/WebDemo/Main.rvn); the handler
functions and application services live in [Endpoints.rvn](src/WebDemo.Logic/Endpoints.rvn). Normal application code
imports `Neo.Web.*`; `System.Web.Http` is unnecessary here. `MapGet` and `MapPost`
accept function values through `T: func`, inspect `handler.Function` at registration,
and cache route/service bindings. The greeting handler receives `name: string` and
`GreetingService` directly. `Home` returns `Task<HttpResult<string>>`; `Item` declares
`HttpResult<string>` and chooses Ok, BadRequest or NotFound from the closed union.

## Build and run

Use a matching development bundle containing Neo.Web. From this repository:

```sh
python3 scripts/validate-neo-web.py --bundle /path/to/verified-bundle \
  --codegen target/release/neoclr-codegen \
  --output target/neo-web-check --mode native
```

This creates a bundle with the source extension, runs the framework HTTP consumer,
and builds/runs this app against an independent HTTP peer. To build just this
project using that staged bundle:

```sh
python3 scripts/build-native-project.py --profile http \
  --bundle target/neo-web-check/bundle \
  --project samples/web/src/WebDemo/WebDemo.rvnproj \
  --aot target/neo-web-check/bundle/bin/neoclr-codegen \
  --output target/web-demo
./target/web-demo/app 8080 60
```

The build tooling also has a `windows-http` profile and `.exe` executable names;
this Neo.Web slice is currently qualified on macOS ARM64, not Windows.
Arguments select port and run duration in seconds (1–3600). Port zero selects an
available port. Startup prints `LISTENING http://127.0.0.1:<port>`. The timer calls
Stop after the selected duration; Main awaits all started handlers before disposing
the application and printing `STOPPED`. The timer is sample policy, not a hosting
time limit. A production signal/lifetime adapter is separate work.

## Try the routes

```sh
curl http://127.0.0.1:8080/
curl http://127.0.0.1:8080/hello/Ada
curl http://127.0.0.1:8080/hello/current
curl -X POST http://127.0.0.1:8080/visits
curl -X POST http://127.0.0.1:8080/visits
```

| Request | Response |
| --- | --- |
| `GET /` | 200, `Hello, neoCLR!` after queued asynchronous work |
| `GET /hello/Ada` | 200, `Hello, Ada!` |
| `GET /hello/current` | 200, `The current visitor is neoCLR`; literal wins over `{name}` |
| First `POST /visits` | 201, `Visit 1`, with a log entry |
| Second `POST /visits` | 201, `Visit 2`; singleton state survives requests |
| `GET /visits` | 405 with `Allow: POST` |
| `GET /items/42` | 200, `The answer` |
| `GET /items/7` | Empty 404 |
| `GET /items/-1` | Empty 400 returned by the handler |
| `GET /items/nope` | 400; handler is not invoked |
| `GET /echo/hello` | 200, `GET:hello` |
| `POST /echo/hello` | 200, `POST:hello`; the same all-method route |
| Unknown path | 404, `No endpoint matched`, from the terminal handler |

Change the in-memory `Greeting` entry to change greeting text. Each greeting
request creates its own GreetingService, retained across await and disposed when
the handler settles. VisitStore is a container-owned singleton. Factories avoid
requiring constructor-reflection retention in this first sample. All responses are
buffered text; no service-dependent stream outlives a scope.

The listener is loopback-only by default. There is no authentication, JSON binding,
persistent storage in this slice. Visit state is local to one
process and uses the existing cooperative dispatcher model.

The handlers in `Endpoints.rvn` are ordinary functions with explicit arguments.
The sample's `test/WebDemo.Tests` project calls those functions without starting a
server and asserts the payloads and closed HTTP outcomes. Shared testing helpers
remain local to the sample. Framework behavior has its own NeoClr.Testing project
under `extensions/Neo.Web/test`.

Build the direct handler tests with the same bundle:

```sh
python3 scripts/build-native-project.py --profile http \
  --bundle target/neo-web-check/bundle \
  --project samples/web/test/WebDemo.Tests/WebDemo.Tests.rvnproj \
  --aot target/neo-web-check/bundle/bin/neoclr-codegen \
  --output target/web-demo-tests
./target/web-demo-tests/app
```

This reference to the sample project retains the actual handler implementations;
the tests do not copy them. No HTTP listener is started by these logic tests.

`Map("/echo/{value}", Echo)` demonstrates an all-method context endpoint.
`Run(Unmatched)` installs the unpatterned terminal stage, distinct from `RunAsync`.
Use(LogRequest) wraps routing and terminal handling with awaited before/after logging;
the sample's typed business handlers remain directly testable ordinary functions.

The host and test project both reference `src/WebDemo.Logic/WebDemo.Logic.rvnproj`.
This library owns the named handlers and application state; the executable owns
composition and lifetime. Native project references currently require library
outputs, so the tests reference this shared logic instead of the executable.

`HttpResult<T>.StatusCode` is available when testing handler outputs directly:
WebDemo.Tests asserts the projected 200/201/400/404 status alongside exhaustive case
matches and payload checks. No HTTP listener or transport namespace import is
needed to inspect the result. The property also drives the framework response writer.

### Typed middleware (development)

`Use(handler)` and `UseMiddleware<T: Middleware>()` share the same
context/continuation pipeline. `Middleware` is an interface whose `InvokeAsync`
method awaits `next(context)` or short-circuits. Register its implementation with
DI before building the application. Resolution happens only when a request reaches
that stage; the request scope lives through the complete outer pipeline.

WebDemo combines `Use(LogRequest)` with `UseMiddleware<MaintenanceMiddleware>()`.
The latter receives Configuration through constructor injection; setting
`Maintenance` to `true` produces a 503 response. Its project retention file keeps
the constructor available in native/JIT builds. Endpoint query/body binding,
`FromBody`/`FromServices` and other ambiguity-resolving attributes remain planned.

`Use` and `UseMiddleware` return `Result<WebApplication, WebError>` with the same
application on success. Chaining is optional; WebDemo keeps route declarations
as separate statements with postfix `?` to propagate registration failures.
`Map*` retains its existing unit result in this slice.

The imported `Greeting` function demonstrates `[FromRoute(Name: "name")]` on a
parameter named `visitor`. Ordinary service parameters remain inferred; attributes
are overrides for ambiguity or an explicit external name, not required annotations. These attributes affect endpoint registration;
the sample test project continues calling both functions with ordinary arguments.
The route, parameter-source and result plans are cached per endpoint; request
arguments and scoped service instances are never shared between requests.
