# HTTP header lookup checkpoint

Development after Preview 9. With a matching toolchain bundle:

```sh
NeoCLRRoot=/path/to/bundle RavenSdkRoot=/path/to/bundle/raven-sdk \
  dotnet msbuild docs/experiments/http-headers/HttpHeaders.rvnproj -v:minimal
target/release/examples/measure_async \
  docs/experiments/http-headers/bin/neoclr/Debug/App.neoil \
  /path/to/bundle/lib/System.neoil 256 100000000
```

Expect `HTTP header lookup checks passed` and zero live managed objects. The fixture
checks mixed-case request/response lookup, repeated Set-Cookie fields, comma-containing
values preserved intact, empty versus absent values, invalid/non-ASCII names and a
snapshot that does not change when the caller's source collection is later extended.
`RequestMediaTypes` demonstrates normal request-error propagation with `?`; explicit
union assertions belong to the surrounding checks. GET lookup does not invent Host
or other provider-generated headers that are not in its stored Headers collection.

No sockets or timing-sensitive server matrix is needed for this managed collection
operation. Signature checks cover public lookup and internal-only shared implementation.
See [the design comparison](../../http-client-design.md#header-lookup-checkpoint--2026-09-25).

`Reference.cs` is a small .NET 10 comparison for case matching, repeated cookies,
empty fields and extension fields containing commas. It passes with the .NET 10 SDK.
It is intentionally not a claim that all .NET typed header parsing matches raw-line
lookup. Run it as Program.cs in a net10.0 console project.

Validation on 2026-09-25: target build and execution passed (138 allocations, 3
collections, zero live objects). Public/internal signature checks and the regenerated
library/API snapshot checks passed. Website build skipped by author direction.
