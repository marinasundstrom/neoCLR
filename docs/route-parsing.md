# Route parsing inside an HTTP handler

Development after Preview 10, selected by the author on 2026-09-27. This follows
[the Web API plan](web-api-plan.md) without introducing WebApplication infrastructure.

## Two independently usable layers

`System.Web.Http.RoutePattern.Parse(pattern)` returns `Result<RoutePattern, string>`.
Compile the pattern once and reuse it. `Match(target)` returns
`Result<Option<RouteMatch>, string>`: Some carries captured values, None means a
well-formed path did not match, and Error describes an invalid target. Configuration
errors are returned by Parse, separately from incoming request handling.

`RouteMatch.Get(name)` returns a decoded string in `Result<string, string>`.
`GetInt32(name)` returns `Result<int, string>` using the platform's Int32 parser;
missing names, malformed integers and overflow are errors. Diagnostic strings are
for display, not stable machine-readable error codes. Callers needing another type
can pass Get's value to that type's parser. Uuid remains a separately tracked API.

The convenience layer belongs to the application: map captured parameters into a
standard Raven union, then use `match` to dispatch. It does not replace direct
matching and the library neither invokes handlers nor registers routes. The tested
station case uses ReadReports(Int32), SubmitReport(Int32), and Unmatched(String).
The last variant carries the original target for application-owned fallback logic.
The application chooses 400 for invalid parameters and 404 for unmatched routes,
unknown station IDs and unsupported methods; this is sample policy, not a library
status mapping. Pattern order and method handling remain explicit.

## Path contract

- Absolute paths starting with `/`; no scheme/authority matching. Each pattern and
  request target is bounded to 1,024 UTF-8 bytes and at most 16 path segments.
- A parameter occupies an entire segment: `{name}`. Names are unique and match
  `[A-Za-z_][A-Za-z0-9_]*`. No constraints, wildcards, catch-all, optional segments,
  defaults or embedded parameter tokens.
- Comparisons are case-sensitive. Root `/`, repeated slashes and trailing slashes
  remain distinct. Empty literal segments are allowed; parameters never capture an
  empty segment. Dot segments are not normalized.
- Match separates the query at the first `?`; it neither parses nor decodes query
  values. The total target byte limit includes the query. Raw `#` fragments fail.
- Split on literal `/` first, then decode each segment exactly once as strict UTF-8.
  Pattern literals are decoded the same way. `+` remains `+`; `%252F` becomes the
  text `%2F`, never a new separator. Reject malformed escapes, invalid UTF-8,
  decoded `/` or backslash, ASCII control bytes and DEL. Validate all path segments
  before deciding a count/literal mismatch. Query contents are outside this parser.
- Instances retain private snapshots and expose no mutation. Matching performs no
  I/O and does not own request/response lifetimes. Storage and instruction costs
  remain proportional to bounded path input; no performance claim is made.

## Comparison and tradeoffs

[ASP.NET Core routing](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/routing?view=aspnetcore-10.0)
provides endpoint selection, constraints and a much richer template language. Its
routing guidance distinguishes constraints from input validation: using constraints
for invalid input can produce 404 instead of an intended 400. Here structural
matching and typed conversion are separate operations, making that distinction
explicit at the call site. The cost is more application dispatch code and no
framework-managed ordering, link generation or automatic parameter binding.

A raw string comparison is sufficient for fixed paths; RoutePattern adds checked
parameters without a hosting framework. Regex-based matching would add another
engine and escaping policy for a much larger language than this case needs. The
small grammar is deliberate and rejects unsupported syntax rather than guessing.

## Validation and integration

[The executable case](experiments/http-routing/README.md) covers direct use, optional
union mapping, typed values, path/error boundaries and independent HTTP peers.
Implementation is Raven code in the existing HTTP library slice. No runtime service,
Raven compiler change or Runtime Contract configuration change is required. Rebuild
matching reference/library artifacts and consumers to use the development APIs.

The experiment also exposed two pre-existing target limitations: scalar-only
payload unions can use overlapping explicit CLR layout, which the bridge does not
admit, and nested constant payload patterns can emit an unsupported static
Object.Equals call. The case uses an Unmatched string payload and ordinary payload
extraction/comparison. These are recorded limitations, not fixes or general union
support claims. See [integration notes](experiments/raven-target/README.md).


## Next layer: attributed union mapping

**Design direction, not an implemented API.** The author proposes RoutePattern
attributes on application union cases, initially with an UnmatchedPattern marker,
then suggests UnmatchedRoutePattern and explicitly asks for a reusable parser object.
The author subsequently delegates the choice of using Result for unmatched routes.

The assistant selects `Result<AppRoutes, RouteMappingError>` for the dedicated
mapper. Successful cases represent actual routes. NoMatch, malformed target and
invalid parameter are distinct outcomes; normal no-match can become 404 without
turning failed integer conversion into a fallback. No unmatched attribute is needed
initially. This decision does not alter the fundamental Match contract, which remains
`Result<Option<RouteMatch>, string>`, nor require the application example to change
its existing explicit Unmatched case.

Build the mapper once at startup and retain it for the server lifetime. Creation
must read the case declarations, compile each RoutePattern, bind named parameters
to case payload types, select conversions and validate case construction. It must
reject malformed patterns, missing/unbound arguments, unsupported types and
ambiguous patterns before accepting requests. Per-request work matches a target,
converts captures and constructs one case; no attribute discovery, pattern parsing
or handler invocation occurs there. Request captures must not be retained in the
reusable mapper or shared between calls. Exactly how method selection participates
is still open: the current RoutePattern primitive handles only paths.

Primary-source comparison, reviewed 2026-09-27:

- [ASP.NET Core attribute routing](https://learn.microsoft.com/en-us/aspnet/core/mvc/controllers/routing?view=aspnetcore-10.0)
  associates templates and HTTP method constraints with action methods. The proposed
  layer associates patterns with union case construction instead; normal application
  match code performs dispatch. That saves hosting machinery but requires a separate
  contract for case payloads and conversion errors.
- [.NET CustomAttributeData](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.customattributedata?view=net-10.0)
  exposes constructor and named arguments without requiring attribute instances.
  Runtime metadata discovery can support late-bound route types, but needs retained
  attributes and checked value-case construction. Build-time generated mappings can
  avoid runtime discovery, at the cost of generator integration and rebuilding when
  declarations change. Neither mechanism is selected yet; both can yield a reusable
  startup-created mapper over the same RoutePattern primitives.

Inspection finds that Raven's UnionDeclarationParser delegates cases to
TypeDeclarationParser.ParseCaseDeclaration, which retains attribute lists in the
syntax tree. This is only parser evidence, not proof of correct attribute emission,
name resolution or neoCLR metadata preservation. The public neoCLR introspection
surface does not currently expose general custom-attribute reading. Before adding
an API, compile a reduced attributed union and inspect exactly where the attributes
and payload parameter names land; check RoutePatternAttribute suffix resolution
alongside the existing RoutePattern class. Also resolve the scalar-only union layout
limitation recorded above rather than requiring artificial reference payloads.

The next prototype should establish that metadata contract and choose discovery or
generation, then prove startup validation, repeated parsing, independent concurrent
captures, typed failure versus no-match, and exhaustive application dispatch. The
Json enum/Uuid/Option requests remain open; this mapper does not silently supply
missing primitive or JSON support.
