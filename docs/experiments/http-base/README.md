# HTTP base-address contract fixture

Development part of networking/web release slice 4. The [design](../../http-client-design.md#baseuri-and-address-overloads--implemented-2026-09-25)
records the author-directed addressing rules and comparison with .NET.

```sh
python3 docs/experiments/http-base/verify.py \
  --toolchain-root /path/to/matching/neoclr-bundle \
  --runner target/release/examples/measure_async
```

The verifier compiles a temporary Raven application from Main.rvn and cases.json.
A forwarding handler and terminal fake observe the request, so failures must be
rejected before dispatch. Ten valid cases run through both string and Uri overloads;
ten invalid base/reference pairs check rejection, using the Uri overload whenever
syntax parsing succeeds. Additional checks cover clearing BaseUri, a relative URL
without a base, malformed percent escapes, query-only absolute URLs and direct Send.
The independent .NET 10 URI probe records encoded-dot normalization and authority
override differences explicitly instead of treating neoCLR's narrower policy as a
compatibility claim.

Validation on 2026-09-25: 1,356 allocations, 27 collections, peak 64 live allocations
and zero live at teardown under a 256-byte GC budget. The .NET comparison passed.
The [HTTP client sample](../http-client/README.md) now sets BaseUri and sends a relative
URL through its handler chain to an independent Python HTTP server: UTF-8 body matched,
509 allocations, ten collections and zero live at teardown. Only that relevant
interoperability case ran; the framing matrix and website build were skipped.

The compiler bridge's signature probe checks both address overloads and rejects a
plain-string BaseUri setter in place of the optional-string contract. Public API
reference and generated-library snapshots are refreshed with the matching bridge.
Token-aware Send/Get, GetString and native cancellation remain pending.
