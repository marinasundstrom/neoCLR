# Route matching and parameters

**Development after Preview 10.** Routing is optional, reusable parsing inside an
existing HttpServer handler. It does not own dispatch or response completion.

| API | Result |
| --- | --- |
| [RoutePattern.Parse](xref:System.Web.Http.RoutePattern) `(pattern: string)` | `Result<RoutePattern, string>`; validate and compile a pattern once |
| `pattern.Match(target: string)` | `Result<Option<RouteMatch>, string>`; captured values, no match, or malformed input |
| [RouteMatch.Get](xref:System.Web.Http.RouteMatch) `(name: string)` | `Result<string, string>`; decoded named value or missing-name error |
| `parameters.GetInt32(name: string)` | `Result<int, string>`; parsed value or missing/invalid/overflow error |

Match `/stations/{stationId}/reports` against a request target, then read
`stationId`. Applications can use the values directly or construct their own union
variants for dispatch with `match`; see [the tested server/client case](/cases/http-server/#development-case-routing-with-typed-parameters).
No union or callback is required by RoutePattern.

Parameters occupy whole, nonempty segments and have unique ASCII identifier names.
Matching is case-sensitive and preserves trailing/repeated slashes and dot segments.
The first `?` separates an ignored query; fragments are rejected. Percent escapes
are decoded once per segment using strict UTF-8, including pattern literals.
Encoded slashes/backslashes, ASCII controls, DEL, malformed escapes and invalid
UTF-8 fail. Plus signs remain plus signs. Both inputs are bounded to 1,024 UTF-8
bytes and 16 path segments; root `/` has no segments. No wildcard, constraint,
optional/default value, URL generation or route-registration support is implied.

A well-formed structural mismatch returns `Ok(None)`. Invalid patterns/targets and
parameter conversion failures return diagnostic strings; do not branch on their
wording. Int32 uses the platform parser, including checked bounds. Use `Get` and
another parser for custom types. A Uuid parser is not yet available.

See [the route design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/route-parsing.md)
for policies, .NET comparison and costs. Preview 10 does not include these APIs.
