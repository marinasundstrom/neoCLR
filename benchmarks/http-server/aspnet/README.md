# Kestrel comparison application

This is a retained HTTP comparison fixture, not a disposable experiment. Keep it
alongside the Raven plain-text and object-JSON fixtures as neoCLR's HTTP server
improves. Record new measurements instead of overwriting historical results.

It uses ASP.NET Core's slim host and Kestrel with a **terminal request delegate**.
There is no endpoint routing, MVC, binding, filters, authentication, compression or
application middleware. The measured entry point is after Kestrel parses the request
and before any routing. Hosting, HttpContext and Kestrel overhead remain; it is not
a direct `IHttpApplication` implementation or a raw socket microbenchmark.

The author explicitly chose this boundary rather than comparing neoCLR's basic
HTTP server against a routed ASP.NET application. neoCLR's existing handler checks
method/path for fixture correctness but does not use a router. Kestrel's terminal
handler accepts any path; the controller sends only the fixed valid request.
Invalid-request semantics are not being compared.

## Fixed workload

- macOS ARM64, loopback HTTP/1.1, `GET /greeting`, a new TCP connection per request,
  concurrency one, `Connection: close`, no TLS or database.
- Plain: encode `Café 🌍` as UTF-8 on every request; no pre-encoded response cache.
- JSON: create `{id: 42, name: "Café team", active: true}`, serialize to text, then
  encode UTF-8 on every request. This matches the Raven handler's operation shape.
- Both supply Content-Type and Content-Length and write the buffered body. Kestrel
  retains its Date header; its Server header is disabled. Framing implementations,
  allocation strategies and scheduling are not artificially made identical.
- Same C# source for JIT and Native AOT, .NET SDK 10.0.100 pinned in global.json.
  Both use System.Text.Json source generation and unescaped non-ASCII JSON output.
  neoCLR currently uses runtime metadata-based object serialization. This compares
  the supported serializers, not two identical serialization algorithms.
- Kestrel keeps normal .NET GC/tiering defaults. neoCLR keeps its existing qualified
  native host and memory admission. No claim of equal memory budgets is made.

[Microsoft's Native AOT documentation](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/native-aot?view=aspnetcore-10.0)
describes slim hosting and JSON source generation. The fixture uses those supported
paths without adding routing. JIT is a framework-dependent Release publish; AOT is
a self-contained native Release publish. Shared-framework precompiled code can be
part of .NET's normal JIT configuration.

## Build and run

```sh
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.2.sdk \
python3 benchmarks/http-server/aspnet/prepare.py --output target/kestrel-comparison-build

python3 benchmarks/http-server/compare-platforms.py \
  --neoclr target/http-ascii-comparison-20261013/results.json \
  --kestrel target/kestrel-comparison-build/build.json \
  --go target/go-comparison-build/build.json \
  --node target/node-comparison-build/build.json \
  --output target/kestrel-comparison-run
```

The neoCLR argument is a passing `compare-library.py` report; the controller uses
its **candidate** commands and retains/verifies its input hashes. See the parent
[benchmark README](../README.md) and [byte-path procedure](../byte-paths.md) for
preparing and comparing neoCLR libraries. Both output directories must be new.
Prepare the [Go](../go/README.md) and [Node](../node/README.md) fixtures too.
Build everything before timing and avoid other builds/profilers during measurement.

Each app prints its ephemeral port, handles 32 requests, then prints DONE and exits.
The comparison reuses the exact neoCLR client and response assertions. Each of twelve
workload/runtime configurations receives a warmup process, then five measured
rounds rotate configuration order. Each process discards eight requests and records 24.
Reports preserve commands, source/artifact hashes, build logs, environment, startup
and every measured duration. Inputs are checked before and after measurement.

These are short fixed-count **pilots**, not steady-state JIT, saturation or concurrent
throughput measurements. Eight requests are not a sufficient general warmup for
.NET tiering/dynamic PGO. Python loopback/client cost is included and can dominate
the faster implementation. Compare only matching workload revisions and report
ranges. Future sustained-load tests must be a separate versioned series with an
explicit warmup and concurrency contract; do not silently change this one.
