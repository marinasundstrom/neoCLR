# One reflected JSON report — development, 2026-09-25

This is an application-owned mapping experiment using the public JSON DOM and
System.Runtime.Reflection extensions. It is not a new JsonSerializer overload or
a generic serializer. The same [Mapping.rvn](Mapping.rvn) is compiled into the
[HTTP report investigation](../http-json/README.md#reflected-report-investigation)
when its verifier is run with `--mapped`; the default sample remains DOM-based.

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


## HTTP validation and remaining limit

HttpApplication.rvn and HttpServer.rvn reuse the mapper with the existing HTTP client
entry point. The HTTP verifier's `--mapped` option selects these sources in temporary
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
