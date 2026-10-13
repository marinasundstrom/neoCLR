# HTTP comparisons before routing

**Local development results — publication on hold pending neoCLR HTTP improvements.**

These retained applications help us measure progress on neoCLR's HTTP server.
Each handles requests directly, without a router: neoCLR, Kestrel with a terminal
ASP.NET Core delegate, Go net/http and Node http.createServer. They create plain
text or a JSON object on each request. No MVC, Express, filters or binding are involved.

The first pilot shows a substantial neoCLR performance gap. That is a starting point
for profiling and improving the runtime and HTTP implementation, not a production
benchmark or a claim about the general ranking of these platforms.

## First development comparison

13 October 2026, Apple M1, macOS 27.0. All **2,304 responses passed**.
Values are median requests per second, with the full five-round range in parentheses.

| Configuration | Plain text req/s (range) | Object JSON req/s (range) |
| --- | ---: | ---: |
| neoCLR JIT | 73.9 (66.4–75.7) | 38.9 (35.7–39.0) |
| neoCLR AOT | 77.3 (71.6–78.7) | 40.5 (37.6–40.7) |
| Kestrel / .NET JIT | 4,516.0 (2,342.8–5,124.5) | 4,307.3 (2,727.6–4,369.9) |
| Kestrel / .NET AOT | 5,235.4 (4,134.8–6,449.9) | 4,606.0 (3,527.2–5,209.7) |
| Go native | 3,969.2 (3,830.1–6,225.4) | 3,906.4 (3,636.1–4,238.6) |
| Node / V8 | 2,811.6 (2,788.7–2,865.5) | 2,728.3 (2,672.7–3,178.5) |

neoCLR uses the development GC and byte-framing improvements. These results do not
replace the [published Preview 14 HTTP baseline](/benchmarks/http/).

## What is being measured?

The client sends HTTP/1.1 GET /greeting over loopback, one request at a time, with a
new connection for every request. The response closes the connection and includes
Content-Length. Plain text is ten UTF-8 bytes, `Café 🌍`. JSON contains the same
three-field person (`id`, `name`, `active`) on every platform. There is no TLS or database.

Each of twelve workload/configuration combinations gets a warmup process, followed
by five rounds rotating configuration order. Every process serves 32 requests;
the first eight are discarded, leaving 1,440 measured requests. Timing includes
connection setup, the server, Python client validation and connection close.
Builds and profiler runs are excluded. Startup is recorded separately.

## How comparable are the applications?

The request and response contracts match; implementation layers still differ.
Kestrel retains slim hosting and HttpContext, but no routing. Go and Node use direct
server callbacks. All construct payloads per request without a response cache.
Go's JSON serializer returns bytes directly; the other fixtures serialize text and
encode it. .NET uses source-generated JSON in both JIT and AOT; neoCLR uses its
current metadata serializer. Runtime GC, tiering and memory defaults differ.

The toolchains are .NET 10.0.100 / runtime 10.0.0, Go 1.26.3 and Node 22.21.0.
[Projects, build evidence and exact methodology](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/http-server/platforms.md)
remain in the repository for future comparisons, alongside
[every recorded sample](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/http-server/platform-comparison-20261013.json).

## Limits and next work

These are short-run pilots. Eight discarded requests do not establish warmed
.NET or V8 JIT performance. The shared Python client and loopback costs matter,
especially for faster servers, and some ranges are wide. This does not measure
saturation, concurrency, keep-alive, production capacity or Windows behavior.

Existing neoCLR profiles point to collector/root handling as a major remaining
cost. We will use the retained setups to assess HTTP and runtime improvements,
then validate longer runs with calibrated warmup and an independent load generator.
New results will preserve this history rather than overwrite it.

[HTTP benchmark history](/benchmarks/history/) · [HTTP server case](/cases/http-server/)
