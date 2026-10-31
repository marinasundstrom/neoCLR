# Primitive HttpServer performance baseline

A standalone Raven program using `System.Web.Http.HttpServer` directly. It returns
HTTP 200 with the two UTF-8 bytes `42`. It has no Neo.Web calls, middleware, DI setup,
or application project dependencies. The external consumer reads the `LISTENING`
line, validates every response, and terminates the benchmark process.

```sh
python3 scripts/build-native-project.py --profile http --bundle /path/to/bundle \
  --project benchmarks/http-server/primitive/HttpServerBenchmark.rvnproj \
  --aot target/release/neoclr-codegen --output target/http-primitive
```

Host policy and execution mode are explicit in the harness; the default native
build uses stress GC. Do not mix those measurements with paced synchronous or
private concurrent-host results. The current public server closes each response's
connection. The fixture is the starting point for a load-generator/concurrency
sweep, not proof of maximum capacity from one sequential consumer.

See the [throughput roadmap](../../../docs/design/http-throughput-roadmap.md) and
[current eager/lazy and GC screen](../../../docs/validation/neo-web-jit-layout-20261031/README.md).
Neo.Web's [separate benchmark](../../neo-web/README.md) measures framework overhead;
its larger shared image is intentionally not this primitive startup baseline.
