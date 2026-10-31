# HTTP header collection qualification — 2026-10-31

Development slice on macOS ARM64. This adds a domain view over HTTP fields and
records the next client/server message-boundary task; it does not implement the
message split or change JIT/collector defaults.

## Implemented contract

- Request/response Headers return read-only HttpHeaderCollection, still assignable
  to Sequence<HttpHeader>. Existing retained-source aliases and message replacement
  behavior remain explicit. Public collection construction snapshots its input.
- Reuse and publish the existing HttpHeaderNames catalog, adding SetCookie. There
  is no additional HttpHeaders class.
- Named indexing returns an independent ordered Sequence<string>, preserving
  duplicates and unsplit commas. Contains avoids a value list. GetSingle returns
  Result<Optional<string>, HttpError>; invalid names and duplicates are Protocol.
  GetContentType parses a single field without reading content.
- HttpHeaderCollection.Empty is a cached immutable empty view. Replacing headers
  on a response using it leaves the shared collection unchanged.
- Incoming request/response parsers retain alternating name/value strings, without
  constructing each HttpHeader. Direct named lookup and message Content-Type
  inspection use that storage. Positional access and iterator Current materialize
  an entry on demand. The parser's validation/framing/limits are unchanged.
- Name comparison uses borrowed UTF-8 views, removing per-name encoding buffers.
  Strings themselves are still decoded during parsing. Array growth, view creation,
  result snapshots and typed parsing are not claimed allocation-free.

The compact buffer holds two references per field instead of one reference to a
separate two-field object. This saves entry objects but changes projection costs:
repeated positional reads can create repeated objects. Entry reference identity
is not promised. General outgoing mutation and a specialized HttpHeaderValues
representation remain follow-ups. See the [API contract](../../../api-docs/http-headers.md).

## Focused validation

Final reports are retained here; commands and artifact hashes are in those reports.
The final Empty addition has its own matching input bundle and consumer checks.
Framework/WebDemo checks made immediately before that additive property change
are retained rather than repeating unaffected pipeline/DI/abort coverage.

- Header contract suite: 13 tests in interpreter, AOT and JIT.
- Interpreter allocation suite: two tests, including an actual HttpServer/HttpClient
  exchange. Repeated Contains/GetSingle and warm Empty access allocate zero managed
  objects. Parsed request and response positional access allocates exactly one
  HttpHeader as a positive control. These counters are interpreter-only; native/JIT
  GC allocation-counter binding is an existing gap.
- Primitive benchmark consumer: actual HTTP requests in all three modes, using
  HttpHeaderCollection.Empty. This is a correctness smoke, not a capacity or
  before/after throughput comparison.
- Neo.Web: 6/6 framework tests and 4/4 WebDemo logic tests in each mode. WebDemo
  additionally passes 30 external HTTP responses plus a disconnect check in each
  mode. These cover DI, body/query binding, middleware, cache bypass, cancellation
  and joined shutdown. The primitive smoke passes 15 HTTP responses in total.
- API snapshot checks pass (4 tests); the website build and local-link checks pass.
- Native API rendering includes both public types and every member. Legacy bridge
  metadata covers the collection/names; GetContentType remains native-only because
  MediaType is absent from the legacy facade. No native reference API is excluded.

Early iterations corrected Raven local-pattern naming and nested propagation
syntax, an assertion with the wrong error propagation type, the name-catalog
correction, and test separation for interpreter-only counters. One runner attempt
passed its assertions but failed the expected manual reporting order, and another
passed all modes but detected an unrelated test-source edit during validation.
Fresh final runs supersede those attempts. Overlapping local website builds caused
an output-directory race; the final website build runs alone. No runtime/compiler
semantic changes were needed for this slice.

## Reproduce

Start with the qualified request-abort development bundle documented in
[its validation](../neo-web-abort-20261031/README.md). The bundled compiler is Raven
813dad20c. The native documentation renderer used here is separately identified
in the committed API snapshot and audit provenance; compiler and renderer revisions
must not be conflated.

```sh
python3 docs/validation/http-header-collection-20261031/compose-input-bundle.py \
  --base-bundle /path/to/qualified-bundle --output target/header-inputs
python3 scripts/test-runtime-library.py --bundle target/header-inputs/bundle \
  --output target/header-contracts --aot target/release/neoclr-codegen \
  --runtime target/release/neoclr --suite http-header-values \
  --mode interpreter --mode native --mode jit \
  --jit-host target/header-inputs/bundle/bin/neoclr-jit-host.dylib
python3 scripts/test-runtime-library.py --bundle target/header-inputs/bundle \
  --output target/header-allocations --suite http-header-allocations --mode interpreter
python3 scripts/validate-neo-web.py --bundle target/header-inputs/bundle \
  --output target/header-web --codegen target/release/neoclr-codegen \
  --mode interpreter --mode native --mode jit
python3 scripts/build-native-project.py --profile http \
  --bundle target/header-inputs/bundle \
  --project benchmarks/http-server/primitive/HttpServerBenchmark.rvnproj \
  --aot target/release/neoclr-codegen --output target/header-primitive
python3 docs/validation/http-header-collection-20261031/smoke-primitive.py \
  --bundle target/header-inputs/bundle --build target/header-primitive \
  --output target/header-primitive-smoke.json --runtime target/release/neoclr \
  --codegen target/release/neoclr-codegen
```

Use a matching source-built runtime/code generator. The smoke script's dylib path
is for this macOS qualification. Windows execution and throughput capacity are not
requalified by these results. Website publication is a separate manual operation.
