# Neo.Web reference application

Neo.Web is an experimental web framework built on top of
`System.Web.Http.HttpServer`. This is its reference application. Keep it
small and evolve it alongside hosting, context APIs and eventually generic minimal
API handlers. It currently demonstrates explicit configuration, logging, a scoped
greeting service, singleton visit state, awaited work and joined shutdown.

The complete source is [Main.rvn](src/WebDemo/Main.rvn). Normal application code
imports `Neo.Web.*` and uses `HttpContext`, `RespondText` and integer status codes;
`System.Web.Http` is unnecessary here. No automatic parameter binding is implied:
handlers explicitly read `context.RouteValues` and resolve `context.Services`.
The intended minimal API experience is handler parameters populated from routes
and request-scoped services, similar to ASP.NET Core. This app will migrate to
that form when the generic binding slice is qualified.

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
| Unknown path | 404 |

Change the in-memory `Greeting` entry to change greeting text. Each greeting
request creates its own GreetingService, retained across await and disposed when
the handler settles. VisitStore is a container-owned singleton. Factories avoid
requiring constructor-reflection retention in this first sample. All responses are
buffered text; no service-dependent stream outlives a scope.

The listener is loopback-only by default. There is no authentication, JSON binding,
persistent storage or general middleware in this slice. Visit state is local to one
process and uses the existing cooperative dispatcher model.
