# One reflected JSON report — development, 2026-09-25

This directory retains the original application-owned mapping experiment and now
also verifies public JsonSerializer object overloads. Mapping.rvn remains a useful
explicit-schema comparison; the HTTP variant has moved to the library mapper.
Use `--public` with verify.py for the integrated API, or omit it for the original
application-owned experiment.

`EncodeReport` finds StationReport.Station through TypeInfo.GetProperties and reads
it through PropertyInfo.GetValue. `DecodeReport` validates the DOM, creates a report
through TypeInfo.CreateInstance, and assigns Station through PropertyInfo.SetValue.
There is no direct field assignment or application-local reflection implementation.
The setter increments Assignments; the fixture verifies one call after each decode.

The schema explicitly maps the wire name `station` to the property `Station`.
A station must be a JSON string; an empty string is accepted. Missing fields, JSON
null and other kinds fail. Extra fields are ignored. These are this application's
choices, not serializer-wide rules. There are no attributes, naming policies,
recursive mapping, caches, field mapping, automatic null mapping or generic APIs.

MappingError retains JsonError and ReflectionError as separate cases. Nearby implicit
converters allow `?` to propagate the causes; Schema reports application-schema
mismatches. Runtime Faults from invoked user code still remain terminal.

The standalone fixture exercises a UTF-8 string round trip, a borrowed MemoryStream
write/rewind/read round trip, real setter effects, missing fields, null and an invalid
root. It uses the real library parser and reflection services. Existing focused
reflection tests cover visibility, incompatible receiver/value types and nullable
access separately.

```sh
python3 docs/experiments/json-object-mapping/verify.py \
  --toolchain-root /absolute/path/to/matching-development-bundle \
  --runner target/release/examples/measure_async
```

Use the compiler, bridge, reference and runtime described in the
[reflection consumer](../reflection-execution/README.md). The local check passed
with a 512-object heap: 195 allocations, peak 72, four collections and zero final
live objects. These figures are correctness observations, not a performance claim.
The website build was skipped as directed.

## Comparison and next question

Reuse the [.NET/reflection comparison](../../json-dom-design.md#next-investigation-one-mapped-report--2026-09-25).
.NET System.Text.Json normally supplies object mapping itself; this experiment keeps
the mapping in the application and supplies only checked runtime operations. That
makes naming and error decisions visible while requiring per-model code. Reflection
adds descriptor lookup, boxing and execution costs; there is no measured advantage
over the previous direct DOM construction.

The next question is which small mapping contract belongs in System.Data.Json.
This example proves the execution path, not a general contract for arbitrary objects.
Keep the DOM API intact while evaluating supported property types, schema decisions
and errors before adding public object-mapping overloads.


## Earlier application-mapper HTTP validation and remaining limit

The earlier HttpApplication.rvn and HttpServer.rvn reused the application mapper.
They now use public serializer overloads; the observations below describe the earlier
checkpoint, not the new public mapper. The HTTP verifier's `--mapped` option selects these sources in temporary
projects; it does not change runtime or transport settings.

```sh
python3 docs/experiments/http-json/verify.py \
  --toolchain-root /absolute/path/to/matching-development-bundle \
  --runner target/release/examples/measure_async --mapped --case server
# Repeat with --case client for the independent client check.
```

All 12 independent server cases pass, including missing/null/wrong-kind station,
empty string, extra fields, invalid JSON/UTF-8 and unknown routes. The mapped client
also passes against Python. Both finish with zero live objects (3,596 server and
349 client allocations). The server distinguishes bad input (400) from reflection
or schema execution errors (500); user Faults still terminate execution.

The managed pair also passes when run alone: 335 client allocations and 336 server
allocations, seven collections each, and zero final live objects. An earlier attempt
reached the existing 15-second transport deadline while another test was running.
The passing isolated retry is evidence for end-to-end correctness, not predictable
latency under load; no specific root cause or timeout-policy change is established.
Keep this mapper opt-in until request-path cost and the public mapping contract have
been evaluated. The original larger DOM report also exposed a transport/performance
limit. Reproduce the mapped pair with `--mapped --case pair`, running it in isolation.


A follow-up [cost investigation](cost.md) found avoidable identity-string copying
in method resolution. Rejecting unrelated methods earlier reduces both preparation
and execution time for this fixture while retaining the same GC results. It does
not change the HTTP deadline or establish predictable performance under load.


## Public serializer checkpoint

Public.rvn consumes only the integrated JsonSerializer overloads: three scalar
property types, string/stream round trips, getter/setter behavior, exact-name/missing
property checks, invalid numbers/kinds, private constructor causes, unsupported
nested models, untouched output on mapping failure and DOM values held as Object.

```sh
python3 docs/experiments/json-object-mapping/verify.py \
  --public --toolchain-root /absolute/path/to/matching-development-bundle \
  --runner target/release/examples/measure_async
```

The current `--mapped` HTTP variant uses those public overloads for both ReportPayload
and Acknowledgement. Their lowercase property names intentionally match the existing
wire schema without a naming policy. The library retains JSON/reflection causes;
the server reports invalid input as 400 and reflection/unsupported model errors as
500. This composes existing HTTP and serializer APIs; dedicated HTTP JSON extensions
and generic serializer methods are still future work. See the
[public contract](../../../api-docs/json.md) and [design comparison](../../json-dom-design.md#provisional-public-object-mapping--2026-09-25).


Public checkpoint validation (2026-09-25): the expanded consumer passes with
1,373 allocations, peak 188, 20 collections and zero final live objects. The existing
DOM/stream regression and .NET memory/JSON baseline pass; the current compiler's
terminal Fault analysis required removing 26 redundant returns from that older fixture.
The public API HTTP pair passes with 408 client / 369 server allocations and zero
live objects (nine and seven collections). These are correctness observations;
no load/latency guarantee is inferred. API signatures and snapshots are checked;
the website build remains skipped.

The fixture binds the stream length and compares it normally: a constant inside
`Ok(0)` currently emits a host static Object.Equals call rejected by the importer.
This existing compiler/bridge limitation is not changed by the serializer slice.


All 12 independent HTTP server cases pass with the public mapper (3,784 allocations,
54 collections, peak 198, zero final live objects), including missing/null/type-mismatch
input, extra fields and an empty string. The reference bridge passes 541 signature
checks, including all new public overloads, internal-mapper rejection and a forged
TypeInfo-parameter rejection. Bootstrap and API snapshots match their sources.


## Typed reads — 2026-09-26

The public fixture also exercises `Deserialize<Reading>` from strings and borrowed
streams, constructor/accessor effects, missing fields, inaccessible constructors and
unsupported value-type models. These overloads wrap the same mapper; they do not add
recursive or value-type mapping. The typed consumer passes with 1,787 allocations,
26 collections and zero final live objects. The unchanged DOM/stream regression
also passes, with 665 allocations and zero final live objects. All 551 bridge
signature checks and the refreshed API snapshot validation pass. The HTTP application now uses typed
ReportPayload/Acknowledgement results and no longer needs Object type patterns.

The matching compiler must include Raven's generic-arity fix, and the runtime must
support reference-target `unbox.any`; see [integration details](../../raven-system-library.md#typed-json-wrappers-and-generic-arity--2026-09-26).

The typed managed HTTP pair passes in isolation: 408 client allocations and 369
server allocations, with zero final live objects on both sides. Library source and
bootstrap snapshot hashes match. The website build remains skipped by direction;
API snapshot verification is complete. These are development checks, not a release
or a claim about latency under load.


## Shared HTTP content conversion — 2026-09-26

The mapped HTTP variant now uses System.Web.Http.Json.JsonContent on both peers:
Create serializes outgoing models; Read<T> reads incoming request/response models.
The application still chooses response status, propagates application errors and
completes/closes its server contexts. Error response literals remain explicit
application-owned content.

The public fixture additionally covers CreateNode/ReadNode and TypeInfo reads,
repeat reads of the same buffer, ignored media type, JSON null versus empty bytes,
malformed JSON, invalid UTF-8, the byte limit and unsupported mapping. It passes
with 2,396 allocations, 35 collections and zero final live objects. No full suite
or website build is used for this slice; validation stays on the JSON/HTTP boundary.

The managed pair passes with 407 client / 369 server allocations and zero final
live objects. The client also passes against the independent Python server, which
checks the JSON media type and body (421 allocations, zero final live objects).
All 559 signature checks and matching API/library snapshot checks pass.

After client request association/default-header integration (2026-09-26), the
managed pair passes again in isolation: 425 client / 369 server allocations,
zero final live objects. Generic JSON verb helpers remain pending; the sample
uses the working shared JsonContent conversion APIs.

## Generic JSON verbs — 2026-09-26

The mapped client now uses GetFromJson<ReportPayload> to fetch `/report`, then
PostAsJson to submit that model to `/reports`. ReadReply inspects the HTTP status
and reads the acknowledgement through JsonContent.Read<Acknowledgement>. Local
implicit converters preserve HttpJsonError causes in AppError. The server still
reads request content and writes response content with shared JsonContent helpers.

The updated neoCLR pair passes in isolation with 761 client / 626 server allocations,
zero final live objects. The client against the independent Python peer also passes
(789 allocations, zero final live objects). That peer verifies GET/POST routes,
JSON media type, UTF-8 body and acknowledgement. Run `--mapped --case pair` or
`--mapped --case client` with the matching development toolchain.

A run overlapping library regeneration timed out. The isolated run passed without
changing any deadline. This remains a limitation under load, not evidence of
production readiness or a reason to weaken the timeout checks. The DOM sample
remains available separately; no full suite or website build was run.


## Nested objects — development, 2026-09-27

The public mapper now supports nested nongeneric reference properties, retaining
exact names, required writable properties and String/Int32/Boolean leaves. Full
input validation precedes all model constructors/setters. Nulls, polymorphic
property values, collections and generic/value models remain unsupported. Four
object levels including the root are allowed; cycles/deeper graphs return
LimitExceeded. Shared children are repeated in JSON and restored independently.
The first nested slice retained the 128-byte and 32-value document bounds; the
byte-budget follow-up below expands documents to 1,024 bytes.

Run the existing `--public` command for the expanded contract consumer. Constructor
and setter markers prove successful execution; the verifier asserts that the
invalid-input region emits no model side effects. Tests also cover shared children,
four/five levels, cycles, nulls, unsupported collections, inaccessible nested
constructors and stream output remaining untouched on mapping failure.

The same station-report client/server case can now use
[NestedHttpApplication.rvn](NestedHttpApplication.rvn): GET /report returns
a station name and sensor description (the initial slice carried only the name),
and POST /reports reads that nested model and
returns `{"accepted":true}` with status 201. The original flat variant is unchanged.
Run the nested case with a matching development bundle and measured runtime:

```sh
python3 docs/experiments/http-json/verify.py --nested --case all \
  --toolchain-root /absolute/path/to/matching-development-bundle \
  --runner target/release/examples/measure_async
```

The website extracts the models, client and server conversion directly from these
sources and ships the complete project/verifier download. This is explicit HTTP
composition, not WebApplication, automatic binding, persistence or a release.
See the [nested mapping design](../../json-dom-design.md#nested-typed-objects--development-2026-09-27)
and [validation evidence](nested-validation.json).


Validation on macOS arm64: the public consumer passes with 6,295 allocations,
96 collections, peak 239 and zero final live objects. All 15 nested HTTP server
cases pass; the neoCLR pair uses 971 client / 799 server allocations, and the
independent-peer client uses 999, all with zero final live objects. These are
correctness results, not latency/load guarantees. The general HttpClient website
example also passes its independent fragmented-UTF-8 peer, checking its Accept
header and decoded output (570 allocations, zero final live objects).
The bridge was rebuilt before final validation to retain the current GC and
Reflection reference surface. No runtime/compiler behavior outside JSON changed.


## API payload budget — development, 2026-09-27

Documents and number tokens now allow 1,024 UTF-8 bytes, matching buffered HTTP
bodies. The same `--public` consumer checks nested models at 1,023 and 1,024 bytes,
HTTP-content reads, and rejection above the cap without stream writes. The DOM
consumer checks UTF-8, escaping, number spelling and borrowed stream boundaries.
The nested case adds a required `station.description` string to its wire model;
its verifier checks 17 server cases, including 1,023/1,024-byte request bodies,
the managed pair and an independent server. Preview 10 and the flat sample are
unchanged. See [current validation](payload-validation.json) and the
[budget design](../../json-dom-design.md#web-api-byte-budget--development-2026-09-27).
