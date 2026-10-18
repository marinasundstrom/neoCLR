# HTTP terminal-handler comparison

These are permanent comparison projects for improving neoCLR HTTP, as requested by
the author. Use the same direct-handler boundary in each implementation: receive
and parse a request, create the payload and write a response **without routing**.
Kestrel uses a terminal ASP.NET Core delegate; Go and Node use their HTTP server
callbacks. This avoids MVC/Express/router work but does not eliminate each host's
request-context and scheduling overhead.

**Publication hold:** The author requests keeping these comparisons on the local
website but publishing nothing until neoCLR HTTP API/handling improvements are
ready. These pilots and small allocation gains do not lift that hold.

## Retained setups

- [neoCLR plain](../jit-http/Server.rvn) and [object JSON](JsonServer.rvn): JIT/AOT.
- [Kestrel](aspnet/README.md): identical C# source in .NET JIT and Native AOT.
- [Go](go/README.md): native net/http, no ServeMux.
- [Node.js](node/README.md): direct http.createServer, default V8 execution.
- [Controller](compare-platforms.py): one client, response validation and timing
  protocol for all six configurations. No third-party package dependencies.

Workload series **http-terminal-loopback-v1**, cases **plain-v1** and **object-json-v1**:
HTTP/1.1 GET /greeting on loopback, concurrency one, new connection each request,
Connection: close, Content-Length, no TLS/database. Plain is ten UTF-8 bytes
`Café 🌍`; JSON is the same three-field Person value in every implementation.
Payloads are constructed per request, not cached. Go's JSON API returns bytes;
the other fixtures serialize text then encode. Kestrel uses source-generated JSON
in both modes; neoCLR uses its current metadata serializer. These are documented
implementation differences, not claims of identical allocation work.

Kestrel and Go/Node retain standard Date headers; neoCLR does not emit one. Kestrel
still uses HttpContext and slim hosting. The neoCLR fixture validates method/path
without a router; the others accept the controller's fixed request directly. Invalid
requests, feature completeness, maximum concurrency and protocol robustness are
outside this comparison. Standard runtime GC/tiering/scheduling defaults remain;
neoCLR uses its qualified 32 MiB native heap admission. Memory budgets are not equal.

## Pilot — 2026-10-13

[Raw results and provenance](platform-comparison-20261013.json). Apple M1, macOS 27,
.NET SDK 10.0.100 / runtime 10.0.0, Go 1.26.3, Node 22.21.0 (V8 version in report).
neoCLR uses the development fixed-ASCII framing change and optimized native GC host,
not the published Preview 14 library/host combination. Build reports are retained in
each comparison project; raw report hashes identify the exact local artifacts.

All **2,304 responses passed validation** across 72 processes. Twelve whole-process
warmups precede five rounds rotating six configurations for each workload. Each
process serves 32 requests and discards eight: 1,440 measured requests total, 120 per
configuration/workload. Timing includes Python client, connection setup, transfer,
validation and EOF. Builds and profiling are excluded. Tables show median req/s
and all-round range, not best-case throughput.

| Configuration | Plain req/s (range) | Object JSON req/s (range) |
| --- | ---: | ---: |
| neoCLR JIT | 73.9 (66.4–75.7) | 38.9 (35.7–39.0) |
| neoCLR AOT | 77.3 (71.6–78.7) | 40.5 (37.6–40.7) |
| Kestrel / .NET JIT | 4,516.0 (2,342.8–5,124.5) | 4,307.3 (2,727.6–4,369.9) |
| Kestrel / .NET AOT | 5,235.4 (4,134.8–6,449.9) | 4,606.0 (3,527.2–5,209.7) |
| Go native | 3,969.2 (3,830.1–6,225.4) | 3,906.4 (3,636.1–4,238.6) |
| Node / V8 | 2,811.6 (2,788.7–2,865.5) | 2,728.3 (2,672.7–3,178.5) |

Startup is recorded separately, not included in request throughput. neoCLR JIT
medians are 4.14 s (plain) and 5.76 s (JSON); its AOT startup is about 12 ms.
Do not infer that native runtime startup and JIT compilation are interchangeable
costs. Exact startup values for every configuration are in the raw report.

## What this establishes

neoCLR has a substantial HTTP execution gap even against these small direct-handler
applications. The [latest framing optimization](ascii-framing.md) reduces allocations
but does not close that gap. Existing native profiles keep collector/root handling
prominent, making runtime boundary and collection overhead the next investigation.
The comparison itself does not isolate the cause; serialization, byte handling,
allocation, GC and scheduling require separate profiles and controlled changes.

These are **short-run pilots**, not saturation benchmarks or steady-state JIT scores.
Eight discarded requests do not establish tiered-JIT/dynamic-PGO warmup for .NET
or V8. Python/client/loopback cost matters especially for the faster servers.
Some ranges are broad and overlap: do not turn this table into a general ranking of
Kestrel, Go and Node or claim production capacity. Longer calibrated warmups, an
independent load generator, keep-alive and concurrency sweeps are separate future
series. Interpreter, Windows, TLS and SQL are not requalified by these runs.

## Reuse and history

Follow each fixture README to prepare fresh artifacts, then:

```sh
python3 benchmarks/http-server/compare-platforms.py \
  --neoclr target/http-ascii-comparison-20261013/results.json \
  --kestrel target/kestrel-comparison-build/build.json \
  --go target/go-comparison-build/build.json \
  --node target/node-comparison-build/build.json \
  --output target/http-platform-comparison-new-run
```

The neoCLR input is a passing matched-library report; its candidate becomes the
neoCLR configuration. Every command and source/artifact digest is retained and
checked before/after timing. .NET's installed shared-framework versions are reported
by dotnet --info; they are not copied into the fixture. Node's executable is hashed;
Go's compiler version/environment and resulting executable are recorded.

Keep workload sources and historical reports in the project. New runtime/compiler
versions receive new reports, never replacement scores. Changes to payload, timing,
connection policy or validation require a new workload series under the
[benchmark history policy](../../docs/benchmark-history.md). The installed SDK and
published website baseline are unchanged; publication is a separate operation.

## Subsequent sustained-load comparison

The [matched Serve/Go net/http comparison](serve/go/README.md) is a separate
connection-close, queued-handler workload with offered-rate scheduling and loss
accounting. Do not combine it with the 32-request pilot results above.
