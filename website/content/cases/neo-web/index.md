# Case Building an experimental Neo.Web application

The Neo.Web reference application greets visitors and records visits in memory.
It is the starting point for further Neo.Web development. Its APIs are provisional
and require a matching development bundle with the optional Neo.Web library.

[Complete source and build instructions](https://github.com/marinasundstrom/neoCLR/tree/main/samples/web)
· [Download the reference app](/samples/neo-web.zip)
· [API reference](/docs/neo-web.html)

## Compose and run

The application adds an explicit in-memory greeting setting and console logging.
VisitStore is a singleton, while each greeting request receives a scoped
GreetingService. Factories construct these services without reflection activation.

```raven
{{NEO_WEB_MAIN}}
```

The command-line arguments select port (8080 by default) and run duration (60 seconds
by default). Use port zero for an available port. Startup prints the actual URL.
The sample's timer requests Stop, Main awaits the lifetime, and disposal follows.
The timer is a demonstration of application-controlled shutdown, not a hosting limit.

## Serve requests

```raven
{{NEO_WEB_HOME}}
```

Home receives an injected request-scoped service, awaits queued work, and returns
a typed text result. The scope remains valid across await. Registration inspects
the structural function contract and caches bindings. The sample does not import
`System.Web.Http`. This follow-on development slice is under qualification.

| Request | Result |
| --- | --- |
| `GET /` | `Hello, neoCLR!` |
| `GET /hello/Ada` | `Hello, Ada!` |
| `GET /hello/current` | Literal route wins: `The current visitor is neoCLR` |
| `POST /visits` | 201 with `Visit 1`, then `Visit 2`; a console log accompanies each visit |
| `GET /visits` | 405 with `Allow: POST` |
| `GET /items/42` | `The answer`; other integers produce an empty 404 |
| `GET /items/nope` | 400 without entering the handler |
| `GET /search?q=Ada&page=2` | Query values and an injected service produce a greeting |
| `POST /greetings` | JSON GreetingInput plus an injected service produces 201 |
| `GET /cached` | One small immutable computed response is cached by example middleware |
| `GET /uncached` | The same computation executes on every request |
| `GET /cache-stats` | Hit, miss and execution counters |
| Unknown route | 404 |

```sh
curl http://127.0.0.1:8080/hello/Ada
curl -X POST http://127.0.0.1:8080/visits
```

All responses are buffered text. State is local to the process; there is no database,
or authentication. Query values, JSON request DTOs and function/typed middleware
are supported within the development limits. The
[Neo.Web guide](/features/neo-web/) explains the current boundaries and lifecycle.

The handlers in `Endpoints.rvn` are ordinary functions with explicit arguments.
The sample's `test/WebDemo.Tests` project calls those functions without starting a
server and asserts the payloads and closed HTTP outcomes. Shared testing helpers
remain local to the sample. Framework behavior has its own NeoClr.Testing project
under `extensions/Neo.Web/test`.

Response caching is deliberately an application example: one immutable public text
endpoint, a bounded singleton store, and fresh response content on every hit.
Authentication/cookie/cache-control requests bypass it. There is no expiry or
general HTTP cache policy. A dedicated harness compares the same computation with
caching enabled and disabled across interpreter, JIT and AOT.

## Request cancellation

`GET /wait-for-abort` waits on an injected `CancellationToken`. Closing the client
connection makes the handler log `REQUEST ABORT OBSERVED` and finish. The HTTP
validator opens a real TCP connection, waits for handler entry and then closes it;
the separate logic test supplies a cancelled token directly. Neither requires a
special web testing framework. Successful responses leave their request token
uncancelled, while server shutdown cancels active requests and joins the handlers.
Unread socket data can delay disconnect detection in this experimental provider.
