# Experimental Neo.Web HTTP overhead

Run `scripts/benchmark-neo-web.py --bundle <qualified-bundle> --codegen <matching-codegen> --output <new-directory>`
with the matching .NET compiler host on PATH. The harness builds one executable
and compares direct HttpServer, a Neo.Web context handler and a generic handler
in interpreter, JIT and AOT (`native`) modes. Every response must be 200 with `42`.

The workload uses sequential fresh loopback connections, identical UTF-8 payloads
and no logging providers or middleware. Readiness and the first request are timed
separately; five warm-up requests precede thirty measured requests by default.
The external Python client's connection/HTTP overhead is included. This is a
focused initial comparison, not a concurrency, keep-alive or production capacity
claim. Report raw latencies and run order; repeat on an idle machine before using
it as a regression threshold. The harness terminates benchmark processes after
completed exchanges; lifecycle correctness is covered by the framework suite.

All three handler choices live in the same executable and load set. Startup figures
therefore describe this shared comparison program, not separately minimized direct
and framework deployments. No allocation counters or concurrency claims are implied.

The `uncached` and `cached` cases reuse WebDemo's actual response-cache middleware
and deterministic 5000-step computation. Both request `/cached` and return the same
computed text; only the cached case installs middleware. Counters must show one
execution for caching and one per request without caching. These cases have a
different payload/workload from the constant-body baseline and should only be
compared with each other. The cache holds one immutable response without expiry.

[Recorded 2026-10-31 qualification and timings](../../docs/validation/neo-web-bindings-20261031/README.md)
include all three modes. The cache avoided computation but did not improve native/JIT
throughput in this short run; JIT startup remains a separate performance gap.

## Collector-policy and layer profiling

The original native host uses stress GC (full collection at frequent generated
boundaries). Its low throughput is not a measurement of the existing paced policy.
The interpreter uses its own collection policy. Do not compare those results to
production .NET capacity or present a stress host as a production configuration.

Build the expanded benchmark with the normal native-project tool and a qualified
bundle. Then compare both C hosts using the **same app.o**, heap budget and compiler
output, changing only `NEOCLR_GC_PACED`:

```sh
python3 scripts/profile-neo-web.py --build /path/to/benchmark-build \
  --output target/web-policy-check --requests 30 --repetitions 3
python3 scripts/profile-neo-web.py --build /path/to/benchmark-build \
  --output target/web-paced-long --policy paced --requests 1000 --warmup 20 \
  --repetitions 3 --sample-seconds 0
```

This macOS-only profiling harness relinks both hosts before timing, alternates
policy order, rotates case order and validates every response. Optional `sample`
runs are separate from timed comparisons. It changes no runtime defaults. Current
pacing budgets come from runtime/native/gc/execution-state.h; scheduler idle/pending
collection still applies. Promotion requires the platform's broader lifetime,
pressure, pause and platform qualification.

Cases distinguish request/context HttpServer callbacks, an empty scope, Neo.Web
context/middleware, generic invocation, scoped service activation, integer query
binding and a cancellable request token. Direct variants retain the same empty
provider/handler to compare scope creation against the same live setup. Query
binding includes integer formatting; these are incremental workloads, not additive
per-component cost counters. Fresh connections and the external client remain part
of the measurement. The larger shared image also affects startup, so historical
startup figures are not interchangeable.

[Collector-policy results and profiles](../../docs/validation/neo-web-performance-20261031/README.md)
record the bounded evidence and open work.
