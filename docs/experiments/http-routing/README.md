# Case: station routes inside an HTTP handler

Development after Preview 10. A reusable RoutePattern matches
`/stations/{stationId}/reports`. Direct.rvn shows ordinary parameter access with no
route union. Routes.rvn adds an optional application-owned StationRoute union;
Server.rvn dispatches it with match and Client.rvn fetches/submits a station report.
The model source is shared with ../json-object-mapping/NestedHttpApplication.rvn;
the HTTP verifier copies it into each temporary project as Application.rvn.

From the repository root, with a matching development bundle and measured runner:

```sh
python3 docs/experiments/http-routing/verify.py \
  --toolchain-root target/experiments/nested-json/bundle \
  --runner target/release/examples/measure_async
python3 docs/experiments/http-json/verify.py --routed --case all \
  --toolchain-root target/experiments/nested-json/bundle \
  --runner target/release/examples/measure_async
```

From the website download root, omit docs/experiments/ from both command paths.
The standalone client/server projects require copying the shared model file to
Application.rvn in this directory first; the verifier performs that step itself.

Expected contract output: `Route parsing checks passed`. The client prints
`{"accepted":true}`. The server checks 29 independent requests, including query
separation, decoded numeric parameters, overflow, invalid separators, unknown IDs,
path mismatch and the existing nested/collection JSON cases. It also checks the
managed pair and client against Python. Only station 42 exists; unsupported methods
and missing routes return 404 as explicit sample policy. There is no route registry,
automatic handler invocation or new hosting lifecycle.

See [the contract and .NET comparison](../../route-parsing.md) and the
[validation record](validation.json). Existing target union/pattern limitations
are recorded in the design; no compiler fix is claimed by this case.

Development test migration (2026-10-10): HTTP route parsing, captures, decoding, numeric conversion and quotas now also run as 8
attributed framework tests in `runtime/raven/tests/http-route-matching`. The framework README
links native/interpreted evidence. This original consumer retains its integration purpose.
