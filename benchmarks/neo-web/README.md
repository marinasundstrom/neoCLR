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
