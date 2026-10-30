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

Home resolves a request-scoped service, awaits queued work, and responds with text.
The scope remains valid across await. `Neo.Web.HttpContext` supplies the web-facing
operations; the sample does not import `System.Web.Http`.

| Request | Result |
| --- | --- |
| `GET /` | `Hello, neoCLR!` |
| `GET /hello/Ada` | `Hello, Ada!` |
| `GET /hello/current` | Literal route wins: `The current visitor is neoCLR` |
| `POST /visits` | 201 with `Visit 1`, then `Visit 2`; a console log accompanies each visit |
| `GET /visits` | 405 with `Allow: POST` |
| Unknown route | 404 |

```sh
curl http://127.0.0.1:8080/hello/Ada
curl -X POST http://127.0.0.1:8080/visits
```

All responses are buffered text. State is local to the process; there is no database,
authentication, automatic body binding or general middleware. The
[Neo.Web guide](/features/neo-web/) explains the current boundaries and lifecycle.
