# Neo.Web query/body, DI and caching qualification — 2026-10-31

Development work on macOS ARM64 (Apple M1, 8 logical CPUs, 16 GiB RAM), using
Raven main `813dad20c`. Neo.Web remains an experimental web framework built on top
of System.Web.Http.HttpServer. These results do not qualify every ASP.NET Core
behavior, a published package or Windows execution.

## Implemented contract

- Nonactivating `ServiceProvider.IsService` controls inferred service binding;
  explicit FromServices overrides inference. Keyed-only registrations do not
  imply unkeyed availability. Request scopes retain service lifetime ownership.
- FromRoute/FromQuery/FromBody/FromServices select sources at registration.
  Required string/int/bool inputs use route captures or query values; unregistered
  POST DTOs infer JSON bodies, while GET requires explicit FromBody.
- JSON body plans cache constructor/property descriptors and nested type plans.
  Input validation precedes DTO construction. Web property names ignore case;
  unknown fields are ignored and omitted fields retain constructor defaults.
- HttpContext/HttpRequest/HttpResponse framework values share request identity.
  HttpResponse.SetHeader preserves response identity for 405 Allow handling.
- WebDemo combines ordinary handler tests with a bounded application response-cache
  middleware example. Cached values outlive request scopes; responses do not.

## Validation

| Consumer | Interpreter | AOT (`native`) | JIT |
| --- | --- | --- | --- |
| Neo.Web framework | 5/5 | 5/5 | 5/5 |
| WebDemo direct logic tests | 4/4 | 4/4 | 4/4 |
| WebDemo external HTTP checks | 30/30 | 30/30 | 30/30 |

Framework evidence comes from validation6. Validation7 reruns the changed sample
and sample tests after adding caching, with the same implementation and one
FromRoute XML-summary correction in Neo.Web. The binary hashes differ; the source
fingerprints identify that documentation-only library change. Unaffected framework
checks were reused instead of rerunning the entire matrix.

The complete validation6 report also records the superseded sample-test build:
the caching test was being added after its sample dependency had already compiled.
It therefore could not resolve the new cache symbols. Validation7 rebuilt both
projects together and passed. Its report is the final sample qualification.

The HTTP consumer covers alias/UTF-8 query decoding, absent and invalid scalar
inputs, repeated integer values, invalid UTF-8, DTO JSON, case-insensitive property
binding, ignored fields, structured +json content types, malformed JSON,
unsupported media types, explicit scalar bodies, cache reuse and bypass. Cache
counters establish one execution for two cacheable requests, followed by executions
for an uncached route and a Cache-Control bypass.

The initial native graph contained 8431 functions, exceeding the 8192 intermediate
inventory bound. The limit is now 16384; selected executable-image and native-host
bounds are unchanged. The over-limit boundary test passes (1 test), and the combined
web/JSON consumers execute in all three modes. Before/after evidence is retained.

API audit and snapshot checks pass (4 tests). The website build checked 6935 pages
before the benchmark report was added. The final build checks 6936 pages, including
the new report; see website-final.log.
Existing missing-description notices remain; no new public API was excluded.

## Inputs and reproduction

`artifacts.json` records the runtime, compiler, code generator and rebuilt library
hashes. `compose-input-bundle.py` records the local composition procedure from the
previous qualified descriptor bundle. System.Web, DI abstractions/implementation
and Neo.Web were rebuilt; reused libraries retain their previous provenance.
The native API audit's component evidence records exact source/compiler hashes.

```sh
python3 scripts/validate-neo-web.py \
  --bundle target/neo-web-bindings/input-bundle \
  --output target/neo-web-bindings/validation7 \
  --codegen target/release/neoclr-codegen \
  --component sample --component sample-tests \
  --mode interpreter --mode native --mode jit
```

Use a new output directory. Omit component filters for a complete run. The matching
.NET host is required for compilation. Build JSON files retain commands, inputs,
artifact hashes and outcomes; large disassembly strings are replaced with their
byte counts, hashes and local full-log paths.

## Remaining gaps

Request-abort CancellationToken binding still needs a transport-owned notification
primitive; ApplicationStopping is not substituted for client disconnection.
Optional/defaulted endpoint parameters, array/nullable JSON bodies, custom
converters, arbitrary result serialization and prepared invocation remain open.
The JSON reader's existing 1024-byte/four-container-level limits apply. Body plans
cache metadata discovery; checked constructor/property/function invocation still
runs per request. No production-throughput or general caching-policy claim is made.

## Performance comparison

| Mode | Direct HttpServer | Neo.Web context | Neo.Web generic |
| --- | ---: | ---: | ---: |
| Interpreter | 5.55 | 4.90 | 4.78 |
| AOT | 25.24 | 19.26 | 18.93 |
| JIT | 24.43 | 19.05 | 17.68 |

The first table is warmed req/s for identical two-byte responses. The next table
compares the same deterministic computation with caching disabled/enabled.

| Mode | Uncached req/s | Cached req/s | Change |
| --- | ---: | ---: | ---: |
| Interpreter | 3.60 | 3.86 | +7.2% |
| AOT | 15.38 | 14.36 | -6.6% |
| JIT | 14.51 | 13.60 | -6.3% |

There are 15 cases and 546 validated HTTP responses, including six counter reads.
Cached cases execute once (35 hits); uncached cases execute 36 times. These short
single trials do not establish a general speedup: native/JIT caching was slower
for this small workload. JIT readiness is approximately one minute for the shared
comparison image and remains a performance gap. Exact timings, raw samples,
platform information, commands, counters and artifact hashes are retained.

```sh
python3 scripts/benchmark-neo-web.py \
  --bundle target/neo-web-bindings/validation7/bundle \
  --codegen target/release/neoclr-codegen \
  --output target/neo-web-bindings/benchmark2
```

Use a new directory. Run without competing builds. All cases share one executable;
readiness is not a comparison of separately minimized applications. The first AOT
process also includes cold OS loader effects. Default requests use fresh connections
and include Python/transport cost. No allocation, concurrency, keep-alive or
production capacity claim is implied.
