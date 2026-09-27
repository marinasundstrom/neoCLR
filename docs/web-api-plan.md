# Minimal Web API increment

**Author-selected direction, 2026-09-27; first nested JSON slice implemented in development.** The
[platform roadmap](platform-roadmap.md) sets priority and the
[HTTP tracker](http-capabilities.md#active-direction--minimal-web-api) owns
application acceptance. The sequence and detailed contracts below are assistant
proposals implementing that direction, not settled signatures or released behavior.

## Consumer and layering

Build one small API that accepts a model containing another object, stores it in
memory, and returns it through an endpoint. A proposed sample is a contact with a
nested address: POST /contacts and GET /contacts/{id}, followed by a bounded list
endpoint when collection mapping is available. SQLite may later replace storage
without changing the HTTP wire contract.

Keep three independently usable layers:

- Existing HTTP transport, messages, context and cancellation in System.Web.Http.
- Shared typed JSON conversion in System.Data.Json and the existing HTTP JSON helpers.
- WebApplication and endpoint routing in a **separate library project**, consumed
  by a separate sample application. System.Web remains a candidate namespace from
  the earlier discussion; project/package names are not yet selected.

SQL contracts and a SQLite provider remain separate from the web framework. Start
with ordinary application composition; DI, a general host builder, middleware
infrastructure and automatic discovery are not needed to prove these endpoints.

## Existing foundation and gaps

The [HTTP matrix](http-capabilities.md#capability-matrix) records verbs, headers,
status handling, buffered bodies, cancellation and server-context completion.
Application routing and WebApplication do not yet exist. The
[object mapper](../runtime/raven/src/System/Data/Json/ObjectMapper.rvn) handles
String, Int32 and Boolean properties on nongeneric reference classes, with public
accessors and exact property names. Typed nesting is now implemented in development; see the
[nested mapping evidence](experiments/json-object-mapping/nested-validation.json). The [JSON design](json-dom-design.md) records the existing mapping policies.

Development JSON now allows 1,024 UTF-8 bytes, matching buffered HTTP bodies;
Preview 10 remains at 128. Four container levels and 32 values remain the shape
limits. This is a bounded increment, not the final Web API budget.
Larger bodies must be checked across parser, writer, stream reader, HTTP conversion,
transport and runtime resource limits together; changing one constant is insufficient.

## Proposed delivery sequence

1. **Nested typed JSON — implemented in development.** The bounded slice maps
   nested nongeneric reference properties using existing scalar rules and public
   parameterless construction. It retains exact names, required writable properties,
   ignored unknown fields and null rejection. The complete supplied tree is validated
   before constructors/setters; execution side effects remain nontransactional. Four
   object levels bound recursion and cycles; structured reflection/JSON errors are
   preserved. Shared children become independent subtrees; polymorphic properties
   and reference IDs are unsupported. See the linked implementation evidence.
2. **Useful API payloads — byte budget implemented in development.** JSON now
   accepts 1,024 UTF-8 bytes, with below/at/above-boundary checks and a longer nested
   station report. See [budget evidence](experiments/json-object-mapping/payload-validation.json).
   Four levels and 32 values remain unchanged. Next add a bounded
   collection shape for list results and make missing versus null versus optional
   values explicit. These are separate decisions from accepting a nested object;
   broader numeric/date/converter support is selected only through a consumer.
3. **Separate WebApplication project.** Prove project references on the neoCLR
   target. Add method/path registration, literal paths and single-segment route
   parameters, asynchronous handlers, and one owner of response completion and
   cleanup. Familiar MapGet/MapPost names are candidates. Start with explicit
   context/route access and shared JSON reads/writes; infer arbitrary delegate
   parameter binding only if a later compiled experiment justifies it.
4. **End-to-end API.** Compile and run the contact sample against an independent
   client and the neoCLR client. Specify duplicate/ambiguous route registration,
   path decoding, query separation, 404 and 405 with Allow, malformed JSON (400),
   unsupported media types (415), and oversized payloads (413). Keep domain-result
   to status mapping explicit. Check cancellation, handler failures, shutdown and
   exactly-once response completion; define HEAD behavior using current no-body
   semantics. New public APIs require matching reference artifacts and API docs.
5. **Optional SQL/SQLite consumer.** If selected, reduce the
   [SQL proposal](proposals/sql-data-access.md) to opening/closing a connection,
   parameterized execution, a forward reader, explicit SQL NULL/value mapping,
   and transactions with commit/rollback. Separate contracts from the provider;
   preserve provider causes through Result. Demonstrate persistence across reopen,
   parameter safety, rollback, busy/error behavior and deterministic statement/
   connection cleanup. Resolve native-library distribution and blocking-call impact
   on the request dispatcher before making it the API's default storage.

The first four steps define the proposed Web API acceptance scope. SQLite is not
a gate. HTTP/2/3, streaming, pooling, TLS, authentication, OpenAPI, ORM, migrations
and multiple SQL providers remain separately scoped work. The first consumer uses
the existing local cleartext transport; completion does not claim public deployment
or production service readiness.

## Website acceptance

The author additionally requests presentable samples organized by case around
client and server, rather than disconnected snippets. Each delivered slice should
advance a coherent walkthrough: the problem, shared models and wire payloads,
server endpoints, client operations, successful responses and representative errors.
Keep cases small and realistic; give selected cases their own documentation page
and link from the general API guide. The HttpClient overview precedes the separate
“Case: Building a Http server app” walkthrough. Provide tested source excerpts,
a complete project download, matching setup/run
instructions and expected output. Keep API reference details linked separately.
The existing station-report client/server case is the current starting point;
introduce the richer nested-model case only with executable evidence. SQLite, if
selected, extends the same case with persistence rather than creating an unrelated
API snippet. The website is part of application acceptance, not a final cosmetic task.

## Comparison, alternatives and costs

Primary sources reviewed 2026-09-27:

- [ASP.NET Core Minimal APIs (.NET 10 documentation)](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/minimal-apis?view=aspnetcore-10.0)
  combines WebApplication and route handlers. Adopt the small endpoint registration
  experience as an ergonomic target. neoCLR's separate library composes existing
  Result/Task and HTTP lifecycle contracts; it does not require ASP.NET Core's host,
  service container or delegate binding machinery. Explicit binding is easier to
  validate but needs more handler code. Keeping raw HttpServer only avoids another
  project but repeats routing/completion policy in applications.
- [System.Text.Json deserialization](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/deserialization)
  supports nested models and collections. neoCLR's missing typed recursion is an
  implementation gap, not a limitation of .NET. Extend the current reflection mapper
  before adding generated codecs: this reuses current metadata but retains reflection
  costs and bounded whole-document work. Manual DOM mapping remains a working
  alternative with more per-model code. Preserve UTF-8 byte limits and existing strict
  property rules initially; rejecting null and requiring every writable property are
  stricter policies than ordinary .NET defaults, with reduced DTO flexibility.
- [SQLite's C interface](https://www.sqlite.org/cintro.html) exposes connections and
  prepared statements with prepare/bind/step/read/finalize lifetimes. The SQL proposal's
  .NET/ADO.NET comparison motivates a smaller connection/command/reader contract
  without DataSet/DataAdapter. This reduces initial surface but sacrifices direct
  ADO.NET compatibility; one provider cannot establish cross-provider portability.
  SQLite-first avoids a database network stack, while adding native packaging,
  resource-lifetime and blocking-execution obligations. Exact bindings and SQL value
  types require a focused design review before implementation.

These are library/application policies above the current runtime, not evidence of
required CLR/compiler changes. Introduce such changes only for a demonstrated target
limitation. No performance improvement is claimed. Use focused contract consumers
and independent HTTP peers; run performance checks only if payload growth or database
blocking exposes a material supported-use problem. Keep API/website documentation
aligned when behavior ships; this planning change requires no website build.
