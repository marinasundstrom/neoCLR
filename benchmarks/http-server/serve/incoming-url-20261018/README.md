# Direct incoming request construction — 2026-10-18

## Fresh baseline

Rebuilt all four managed libraries from current source at 4014257ba, using
the qualified Raven compiler 478e0dd85 (compiler-primitive-4), current release
backend and current native sources. Recompiled Server.rvn and rebuilt Go.
This replaces the older managed guest identified in the concurrent-GC review.
Build manifests include source/tool/artifact hashes and commands.

The baseline uses paced synchronous GC, the existing 32 MiB native heap and
unchanged Serve callback fixture. The initial 10-second 300/sec smoke completes
all 3,000 measured requests; it verifies the rebuilt fixture, not sustained capacity.
Matched baseline/candidate/Go load measurements follow in a separate slice.

Rebuild with scripts/build-native-class-library.py and the shared
../task-completion-20261018/build.py, using the retained final-load/native link
recipe. Evidence contains both library build provenance and guest build commands.
No public HTTP API or GC policy changes in this baseline slice.

## Direct incoming construction

`FromIncoming` now validates the separated authority and origin-form target, then
constructs HttpRequest directly. It does not concatenate a scheme/authority/target,
allocate Uri, split URI components, or parse the resulting URL again. Host and
Target remain strings. Target and authorities without a port reuse existing owned
strings; explicit ports extract the hostname. No pooled-buffer views escape.

The decoder remains responsible for origin-form admission, framing, headers and
body ownership. The incoming constructor preserves the old ASCII/percent grammar,
URI error precedence, 4096-byte parser bound, 1024-byte combined URL-equivalent
bound, 253-byte hostname bound, ports and malformed Host suffix handling. The
limits count the implied scheme length without constructing it. The byte grammar
intentionally matches the existing Uri parser; future grammar changes must update
the differential cases too. Two temporary field encodings remain; this is not
zero-copy parsing or a complete elimination of per-request allocation.

[ASP.NET Core HttpRequest](https://learn.microsoft.com/en-us/dotnet/api/microsoft.aspnetcore.http.httprequest?view=aspnetcore-10.0)
exposes Host, Path, QueryString and Scheme separately, with URL-combining extension
methods. That supports keeping reconstruction optional. This slice preserves
neoCLR's existing raw Target string and separate hostname/Port rather than changing
them to ASP.NET's HostString/decoded Path contracts. It changes internal work, not
the public API, HTTP feature support or outbound URL/Uri overloads. The matching documentation reference is regenerated and remains
binary-identical; only its source fingerprint changes.

The public-wire Probe deliberately uses Accept to inspect parser outcomes. Its
50 cases compare exact success fields, error cases/messages, response framing,
body and content type against the fresh pre-change AOT oracle. All input is
fragmented. The performance fixture continues to use Serve and a queued await.
Plain HTTP is exercised here; TLS transport itself is unchanged and is not
requalified by these wire cases.

All 50 cases pass in interpreter, JIT and AOT against the fresh baseline AOT
oracle. The first probe build exposed an unrelated SocketError.ToString backend
restriction through HttpError.ToString; the final probe explicitly matches tested
error payloads. An initial interpreter attempt used the stale release executable
and failed enum metadata admission before listening. The qualified primitive/enum
runtime passes all cases. Both failures are retained, not treated as passes.

Reproduce with the shared guest builder and Probe.rvn, then verify.py --build
BUILD --library LIBRARY --runtime RUNTIME --backend CODEGEN --host JIT_HOST
--output REPORT. Establish the old-cohort oracle with --modes aot, then add
--oracle REPORT for the candidate's three-mode comparison. Tool and module hashes
are retained in validation-inputs.json and the build manifests.
