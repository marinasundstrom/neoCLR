# Experimental Neo.Web overhead and response caching

Development measurement, 31 October 2026, on Apple M1 (8 logical CPUs, 16 GiB),
macOS 27.0.1 ARM64. One process per case, five warm-up requests and thirty measured
requests after a separately timed first request. The Python client uses sequential
fresh loopback HTTP connections; client/transport overhead is included. No other
repository build ran during measurement. Workstation background activity was not
controlled. This short run is a demonstration, not a regression threshold or
production capacity claim.

## Constant two-byte response

Warmed requests per second, with identical `42` responses:

| Mode | Direct HttpServer | Neo.Web context | Neo.Web generic |
| --- | ---: | ---: | ---: |
| Interpreter | 5.55 | 4.90 | 4.78 |
| AOT | 25.24 | 19.26 | 18.93 |
| JIT | 24.43 | 19.05 | 17.68 |

## Same computation with cache disabled and enabled

Both cases execute the WebDemo computation and return identical text. Only the
cached case installs its response-cache middleware. Counters verify 36 executions
without caching and one execution with 35 hits when caching is enabled.

| Mode | Uncached req/s | Cached req/s | Change |
| --- | ---: | ---: | ---: |
| Interpreter | 3.60 | 3.86 | +7.2% |
| AOT | 15.38 | 14.36 | -6.6% |
| JIT | 14.51 | 13.60 | -6.3% |

Avoided computation is verified, but the cached native/JIT cases are slower in this
run. The small computation may cost less than the middleware and cache policy;
these measurements do not isolate that cost or establish statistical significance.
The example is useful for making this tradeoff measurable without a sophisticated
store. It does not establish that caching helps every endpoint.

## Startup and scope

All five choices share one executable/load set. Readiness ranged from roughly
4–5.5 seconds for interpreter, 0.018–0.541 seconds for AOT and about one minute for
JIT. Exact readiness, first-request and per-request times are retained in the raw
report. Run order and OS loader caches affect the first AOT process; these are not
separately minimized deployments. JIT startup remains an explicit performance gap.

The cache stores one immutable public text response, bounded to 4096 bytes. It has
no expiry or concurrent-miss coalescing. Auth/cookie/cache-control requests bypass
it. General HTTP cache policy, concurrency, keep-alive and allocation profiling
are outside this run.

All 546 HTTP responses and all six counter snapshots were checked across
interpreter, JIT and AOT. Neo.Web remains an experimental web framework built on top
of System.Web.Http.HttpServer; this is not a published-package benchmark.

[Source and reproduction](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/neo-web)
· [Raw timings and counters](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/neo-web-bindings-20261031/benchmark-results.json)
· [Build and contract qualification](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/neo-web-bindings-20261031/README.md)
