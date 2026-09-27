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
payload unions can use overlapping explicit CLR layout, which the original routing slice did not
admit, and nested constant payload patterns can emit an unsupported static
Object.Equals call. The case uses an Unmatched string payload and ordinary payload
extraction/comparison. These are recorded limitations, not fixes or general union
support claims. See [integration notes](experiments/raven-target/README.md).


## Attributed union mapping experiment

**Implemented as a bounded development generator experiment, not an SDK API.** The author proposes RoutePattern
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

The [implemented experiment](experiments/route-union-mapper/README.md) selects
build-time generation. It inspects emitted union-case attributes and constructor
parameter names, validates String/Int32 binding and rejects structural overlap.
Generated `Create()` compiles RoutePattern instances once at startup. Retain this
object for the server lifetime; Parse matches, converts and constructs one case
without retaining captures. No attribute discovery or handler invocation happens
per request. Methods remain application-owned. This adds an explicit generation
step; integration into the SDK and runtime late-bound discovery are not supplied.

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
  declarations change. This experiment selects generation; runtime discovery remains an alternative.

The reduced metadata probe found that Raven parsed case attributes but omitted
them from semantic symbols and CLI metadata. The independently tested general
compiler fix now preserves them on nested case value types and validates their
AttributeUsage. The short RoutePattern attribute spelling resolves alongside the
core RoutePattern class. The neoCLR-specific bridge admits Int32-only standard
union carriers through logical fields without general explicit-layout aliasing;
no artificial reference payload is required. Details and validation are in the
[experiment README](experiments/route-union-mapper/README.md).

The Json enum/Uuid/Option requests remain open; this mapper does not supply missing
primitive or JSON support. Runtime custom-attribute reading, generic schemas,
other payload conversions and automatic build integration remain future work.

### Author refinement: startup reflection

After reviewing the generated prototype, the author selects runtime attribute
reflection for the immediate implementation: discover and validate metadata once
at startup, then cache the binding and construction information. Source generation
remains a future alternative. The completed generator is retained as experimental
contract evidence, not the chosen application workflow. Bounded [member and parameter metadata reading](attribute-introspection.md) is
implemented without attribute-constructor execution. Retained constructor invocation now supports [checked union construction](experiments/union-construction/README.md). Next prepare the general reusable mapping, with schema validation and capture binding.
