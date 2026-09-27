# Case: attributed item routes

Development experiment after Preview 10, not an installed SDK generator or a new
System.Web.Http API. Requires a matching development compiler, bridge and library.

`Routes.rvn` declares `/items` and `/items/{id}` on ordinary Raven union cases.
`Generator.cs` reads **emitted CLI case metadata**, binds constructor parameter names
and produces `AppRoutesParser` in `Generated.rvn`. `Create()` compiles the patterns
once; keep the parser for the server lifetime. `Parse(target)` constructs a fresh
case with parsed payloads. It neither discovers attributes nor reparses patterns
per request, and retains no captures between calls.

## Run the complete case

From this repository (or the extracted website sample archive):

```sh
python3 docs/experiments/route-union-mapper/verify.py \
  --toolchain-root /path/to/development-bundle --runner /path/to/measure_async
```

In the archive, use `route-union-mapper/verify.py` instead. The verifier builds the
host generator, compiles a temporary schema library, generates Raven code, compiles
and imports consumers, and executes them. It also runs the item server with an
independent HTTP client and with the accompanying Raven client. Generated files and
project files are temporary; all hand-authored sources are included here.

For application integration, compile `Attributes.rvn` and `Routes.rvn` as a Raven
library using the bundle's NeoCLR.Raven.props/targets. Then run:

```sh
dotnet build Generator.csproj -p:NeoCLRRoot=/path/to/development-bundle
dotnet bin/Debug/net11.0/Generator.dll Schema.dll \
  /path/to/development-bundle/demo/NeoCLR.CoreProbe.dll Generated.rvn
```

Compile Attributes.rvn, Routes.rvn, Generated.rvn and Server.rvn together as the
application. The generator does not load or execute the schema assembly. A failed
generation deletes stale generated output. Automatic SDK/MSBuild integration is
future work; `verify.py` is the executable build recipe for this slice.

## Generated application contract

- `AppRoutesParser.Create() -> Result<AppRoutesParser, RouteMappingError>`:
  compiles each pattern once; InvalidConfiguration carries the primitive's reason.
- `Parse(target: string) -> Result<AppRoutes, RouteMappingError>`:
  Ok carries a route case. NoMatch means a valid path matched no pattern;
  InvalidTarget(reason) means malformed input; InvalidParameter(name, expectedType)
  means a structurally matched capture failed conversion. Names/types are stable
  for the generated declaration; diagnostic reason strings are for display.
- No unmatched marker/case is required. The fundamental RoutePattern.Match API
  still returns Result<Option<RouteMatch>, string> and can be used independently.
- Generation rejects missing attributes, malformed/overlapping patterns, unbound
  parameters and unsupported payloads before the application can start. Pattern
  overlap is rejected even when literal precedence might otherwise be possible.

Initial scope: one public nongeneric union in the global namespace, 1–16 cases,
ASCII identifier names, and exact name binding of String/Int32 constructor payloads.
Pattern rules match the [primitive contract](../../route-parsing.md). Empty cases
are supported. No method attributes, generic unions, defaults, optional parameters,
Uuid, enums, handler registration or runtime attribute API are supplied.

The server returns an in-memory catalog item (42, Desk lamp), lists it at `/items`,
and retrieves it at `/items/42`. Fixed JSON literals keep the example focused on
routing; the station case separately demonstrates typed JSON. The client prints
both responses. This case chooses 400 for malformed targets/conversion failures,
404 for unknown paths/items and unsupported methods. These are application choices.

## Compiler and bridge contract

Raven now preserves/validates attributes on nested union case value types; it does
not copy them to constructors. General compiler fix `3179cd21e` is on Raven main,
independently tested with 16 attribute and 189 union checks, then cherry-picked as
`2f62361ef` onto neoclr. Runtime Contract configuration is unchanged.

The neoCLR bridge additionally admits the bounded explicit-layout carrier shape
for standard nongeneric unions with only private Int32 case fields (and empty
cases). It projects independent logical fields; it does not implement arbitrary
CLR explicit-layout aliasing. Reference overlays, other scalar types, public
payload fields and malformed tags remain rejected. Mixed String/Int32 unions use
Raven's existing sequential carrier. Copy, boxing and pattern extraction are
exercised by Main.rvn; ScalarUnionChecks.cs checks admission mutations.

Runtime reflection discovery remains an alternative, with additional metadata
retention and construction APIs. Generation avoids those runtime requirements but
adds a build step and rebuilds when declarations change. See the
[design comparison](../../route-parsing.md#attributed-union-mapping-experiment).

## Subsequent direction

The author selected startup runtime reflection and cached mappings after this
prototype. This generator remains experimental evidence and a future alternative;
it is not the chosen immediate application workflow.
