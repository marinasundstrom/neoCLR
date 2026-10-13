# Known request header byte matching

Development slice, 2026-10-14. The request decoder recognizes seven fixed ASCII
names directly in its already-owned header bytes: `host`, `connection`,
`content-length`, `transfer-encoding`, `expect`, `content-encoding`, and
`content-type`. Length checks precede byte probes; ASCII letters match without
case and hyphens match exactly. All token validation still runs before dispatch.
Unknown names retain the existing lowercase buffer/decoding path.

This follows the known-name byte matching used by
[Kestrel's generated request headers](https://github.com/dotnet/aspnetcore/blob/main/src/Servers/Kestrel/Core/src/Internal/Http/HttpHeaders.Generated.cs),
using the existing research in [the HTTP design](../../docs/http-foundation.md).
It preserves neoCLR's existing lowercase header representation, duplicate handling,
framing rejection, value trimming and eager value decoding. No public API changes,
mutable global cache, request-buffer borrowing, value reuse or pooling are added.

The returned fixed spellings are literals, not a new permanent string pool.
`runtime/codegen/src/values.rs` copies literals when text identity is observable;
otherwise it can use immutable image data. Thus the JSON fixture still allocates
the name strings, while the plain fixture avoids them. Do not describe this as
universal interning or rely on reference identity. A permanently shared-name
contract needs a separate review of literal/explicit-intern semantics and GC
ownership; this slice preserves the existing policy.

## Allocation effects

Matched rebuilt libraries share the pinned Raven compiler, bootstrap and runtime
inputs with the preceding URI candidate. Only `HttpServer.rvn` differs among the
common library inputs. The AOT timing fixtures explicitly use state machines and
stress GC; they are separate from the runtime-await correctness fixture.

| Per measured request | Plain before | Plain after | JSON before | JSON after |
| --- | ---: | ---: | ---: | ---: |
| Managed allocations | 388 | 378 | 820 | 812 |
| Requested payload bytes | 19,759 | 19,513 | 33,764 | 33,548 |
| Native TEXT blocks | 13 | 11 | 124 | 124 |

Both candidate profiles validate 32 responses, have zero GC failures and reclaim
every allocation: 12,116 plain and 26,006 JSON including startup/teardown. Plain
peak live bytes decrease from 8,223 to 8,193; JSON remains 12,646. Peak arena extent
decreases from 13,344 to 13,232 plain and 19,904 to 19,744 JSON.

The unrolled matching adds 227,344 bytes to the plain guest object and 234,240 to
JSON (about 2.0% and 1.5%). That object-size cost includes generated code and data;
it is not a resident-memory measurement. A future compact constant-data matching
facility may reduce it. Avoid growing this manually maintained list without
measuring workload benefit and code size.

## Validation and timing

All 87 wire checks pass: 29 each in interpreter, JIT and AOT with default
runtime-await. Seven alternating AOT timing rounds validate another 1,024 responses.
Plain median throughput is 158.73 → 161.53 requests/s (+1.8%); JSON is
66.69 → 67.34 (+1.0%). JSON ranges overlap, so this is not an established throughput
improvement. The allocation reduction is directly measured. These are short,
sequential macOS loopback runs, not Windows or sustained-load qualification.

Evidence: [wire checks](header-names-validation-20261014.json),
[timings](header-names-timings-20261014.json),
[matched source inputs](header-names-provenance-20261014.json),
[plain profile](header-names-plain-20261014.json), and
[JSON profile](header-names-json-20261014.json).
Baseline profiles are the candidate captures in the [URI refinement](uri-byte-parsing.md).

## Reproduction

`HeaderNames.rvn` checks the public request header collection through real sockets.
The controller sends known mixed/uppercase names, exact-length and longer
lookalikes, unknown/repeated/empty values, chunk framing, small writes, unsupported
encodings/expectations, duplicate and ambiguous framing, malformed length, and
invalid token spelling. It uses the SDK's default runtime-await lowering.

```sh
python3 benchmarks/http-server/check-header-names.py \
  --bundle MATCHING_BUNDLE --aot MATCHING_CODEGEN --output NEW_CHECKS
python3 benchmarks/http-server/profile.py --build BUILD_JSON \
  --workload plain --allocation-sites --output NEW_PROFILE
python3 benchmarks/http-server/compare-native-library.py \
  --baseline URI_FIXTURES --candidate HEADER_FIXTURES --rounds 7 --output NEW_TIMINGS
```

Use `--workload json` for the JSON profile. Timing roots contain matching
`plain/native/build.json` and `json/native/build.json`, as described in the
[URI refinement](uri-byte-parsing.md). The existing JIT/AOT comparison controller
remains available independently. Public API reference signatures are unchanged.
