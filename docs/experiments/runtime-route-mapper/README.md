# Case: an attributed HTTP catalog

This reusable application sample discovers route declarations through runtime
introspection, validates them once, and retains compiled patterns and constructor
bindings for the server lifetime. It runs on the matching development toolchain;
RouteUnionMapper and RoutePatternAttribute are sample source, not installed SDK APIs.
No generator or WebApplication infrastructure is required.

`Routes.rvn` declares ListItems and GetItem(Int32). `AppRoutesParser` is a small typed
facade over the reusable `RouteUnionMapper`; changing cases needs no mapper edits.
`Server.rvn` constructs the parser before listening, dispatches with `match`, and
returns a small in-memory catalog. `Client.rvn` lists items and reads item 42.
The fixed JSON responses keep routing visible; the station case demonstrates typed
JSON. The handler owns HTTP method, status and completion policy.

## Preparation and requests

`RouteUnionMapper.Create(type: TypeInfo) -> Result<RouteUnionMapper, string>` requires
one public nongeneric standard value union with 1–16 public direct nested cases.
Each case requires exactly one RoutePattern string attribute and one public
constructor. Captures must exactly match distinct constructor parameter names;
only String and Int32 payloads are supported. A matching public carrier constructor
is retained for each case. Invalid schemas and structural overlaps fail preparation.
No partially prepared object is published. Attribute constructors are not run—the
sample deliberately faults if one executes.

`Parse(target: string) -> Result<Object, RouteMappingError>` returns a boxed carrier.
The typed facade returns `Result<AppRoutes, RouteMappingError>` for ordinary pattern
matching. NoMatch, InvalidTarget, InvalidParameter and ConstructionFailed are separate
outcomes. The server chooses 404, 400 and 500 respectively; user constructor Faults
remain terminal. Unknown item IDs and unsupported HTTP methods also produce 404.

Preparation caches patterns, parameter names/types, and exact case/carrier descriptors.
Requests do not rediscover attributes, constructors or parameter metadata. They match
paths, build fresh argument collections, convert captures, and invoke retained
constructors. The reflection boundary still validates access and arguments and builds
its execution adapter per call. This is not a compiled delegate/native-plan cache.
Each attempted pattern currently decodes the target; no performance claim is made.

Overlap checking uses the existing decoded segment grammar, including empty literal
segments and nonempty parameters. It conservatively ignores the target's total byte
limit and conversion domains: `/items/new` overlaps `/items/{id}` even with an Int32
payload. No ordering, fallback or literal-priority rule resolves overlaps. Encoding
limits and conversion remain enforced when matching requests. Generic unions, other
payload types, method attributes, wildcard/default/optional captures and source
generation are outside this sample. JSON enum/Uuid/Option support remains separate.

## Run

From the repository, with a matching development bundle and execution runner:

```sh
python3 docs/experiments/runtime-route-mapper/verify.py \
  --toolchain-root target/experiments/nested-json/bundle \
  --runner target/release/examples/measure_async
```

In the website archive use `runtime-route-mapper/verify.py` instead. The saved
Mapper, Server and Client projects compile through the ordinary SDK targets.
The verifier builds temporary copies and checks reuse, snapshots/overlap boundaries,
malformed targets, name-based mixed bindings, rejected schemas, nine independent
HTTP requests and a two-request neoCLR client/server pair. [Saved validation](validation.json)
records the exact source hashes and outcomes. Use `--consumer-only` for direct
parsing checks or `--schemas-only` to rerun only invalid-schema checks. The importer
can reject unsupported union layouts before runtime mapping; the unsupported Long-payload
fixture records importer rejection separately from startup mapper rejection.

The .NET comparison and design tradeoffs are recorded in
[route parsing](../../route-parsing.md#runtime-attributed-union-mapper).
