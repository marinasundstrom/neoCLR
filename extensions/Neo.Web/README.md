# Neo.Web

Neo.Web is an experimental web framework built on top of
`System.Web.Http.HttpServer`. It composes DI, configuration and logging, accepts
context and generic minimal API handlers, and supports function middleware plus
`UseMiddleware<T: Middleware>()`. Generic handlers currently bind context, named
string/int route values and scoped services through a registration-owned plan.
API names and assembly boundaries remain provisional; query/body binding, streaming
results and a general host remain later work.

Start with the [reference app](../../samples/web/README.md), which will evolve with
Neo.Web. Its application code imports `Neo.Web.*`; it does not need to import the
transport namespace. `Neo.Web.HttpContext` exposes request services, decoded route
values and buffered response methods. When lower-level code also imports `System.Web.Http.*`, add
`import Neo.Web.HttpContext` to select the framework context.

See the [API contract](../../api-docs/neo-web.md),
[design proposal](../../docs/design/neo-web.md) and
[validation record](../../docs/validation/neo-web-middleware-20261031/README.md).

## Build

Use a matching development compiler/runtime and source-built dependencies. The
combined hosting graph requires the source code generator with grouped type-shape
reflection; build it with `cargo build --release -p neoclr-codegen --bin neoclr-codegen`.

```sh
dotnet /path/to/rvnc.dll neoclr --project extensions/Neo.Web/src/Neo.Web/Neo.Web.rvnproj
```

`build-native-class-library.py --with-web` includes Neo.Web as an optional final
library in a development bundle. It is not added to published Preview 19 packages.
The focused validator builds it against a verified bundle and stages the extension,
framework tests and reference sample:

```sh
python3 scripts/validate-neo-web.py --bundle /path/to/bundle \
  --codegen target/release/neoclr-codegen \
  --output target/neo-web-check
```

Use `--mode native`, `--mode interpreter` or `--mode jit` to select execution modes.
Default is native; native executable compilation is shared across modes. The
framework consumer owns behavior assertions; the Python harness drives builds and
an independent HTTP peer against the reference app.

The framework and reference app are qualified in interpreter, JIT and native modes
with the matching development toolchain. The validation record identifies exact
builds and limitations. Native/JIT constructor activation uses the explicit retention
policies included with the tests and sample.
