# URI byte parsing in the HTTP path

Development measurement, 2026-10-14, macOS ARM64. `Uri.Parse` now encodes one
bounded buffer and inspects ASCII bytes instead of allocating one-byte substrings
and comparing them with strings. The 4,096-byte limit is checked before encoding.
Component strings and resolution semantics remain unchanged.

This is an internal implementation improvement to the existing `System.Uri`
subset, not a new divergence from .NET's public URI contract. See the
[URI design and its existing .NET differences](../../docs/uri-design.md).
The temporary buffer is ordinary rooted managed storage; no pooling, borrowed
native storage or GC policy change is introduced.

## Matched allocation profiles

Both source library builds use Raven `cb4721e7d`, the same bootstrap and metadata
translator. Comparing the common hashed inputs finds only `System/Uri.rvn`
changed. Both HTTP fixtures are rebuilt with the same code generator, explicit
state-machine lowering, standalone native host, stress collection and 1 MiB heap.
This isolates the library change; it does not qualify runtime-await or publish a
new SDK performance matrix.

Each process validates 32 responses and measures 24 requests after warmup.
Allocation attribution is separate from throughput timing.

| Per measured request | Plain before | Plain after | JSON before | JSON after |
| --- | ---: | ---: | ---: | ---: |
| Managed allocations | 510 | 388 | 1,198 | 820 |
| Requested managed payload bytes | 20,793 | 19,759 | 37,102 | 33,764 |
| Native TEXT blocks | 137 | 13 | 504 | 124 |

Allocations decrease by 23.9% for plain text and 31.6% for JSON. JSON also removes
allocations attributed directly to `Uri.Parse`, `Alpha` and `Digit`, beyond the
124 character-probe allocations seen in both baselines. Do not assume that every
literal is permanently interned: these figures describe these exact closed guest
images and their runtime allocation behavior.

All four profiles pass response validation, report zero GC failures, and reclaim
every allocation including startup/teardown. Peak live bytes remain 8,223 plain
and 12,646 JSON. Peak arena extent changes from 13,336 to 13,344 plain and 19,776
to 19,904 JSON: fewer allocations do not imply a lower peak for every layout.

## Throughput and correctness

Seven alternating, uninstrumented rounds, with one warmup process for each of the
four fixtures. All **1,024 HTTP responses** pass. Median successful requests/s:

| Workload | Before (range) | After (range) | Ratio |
| --- | ---: | ---: | ---: |
| Plain text | 100.20 (96.09–100.46) | 156.28 (148.35–156.48) | 1.56× |
| Object JSON | 53.13 (52.74–53.24) | 65.92 (65.11–66.21) | 1.24× |

These are short sequential loopback runs on the same developer machine with
connection close and stress GC. They do not establish sustained/concurrent
capacity, Windows performance, or competitiveness with Go/Kestrel. Do not compare
these medians directly with published SDK results built from different inputs.

All 12 URI tests pass in interpreter, JIT and AOT, including resolution,
ASCII/percent/Unicode boundaries, equality and the maximum-length quota. The
unchanged parser's interpreter corpus exceeded the existing 180-second runner
limit; the candidate completes. This is recorded as a baseline timeout, not a
controlled interpreter speedup. Candidate native execution exercises the ordinary
managed-array lifetime; separate HTTP profiles verify complete reclamation.

Evidence:

- [Paired timings and build provenance](uri-bytes-timings-macos-20261014.json)
- [Matched library inputs and baseline timeout](uri-bytes-provenance-20261014.json)
- [Three-mode URI tests](uri-bytes-tests-macos-20261014.json)
- [Plain baseline](uri-bytes-baseline-plain-20261014.json) / [candidate](uri-bytes-candidate-plain-20261014.json)
- [JSON baseline](uri-bytes-baseline-json-20261014.json) / [candidate](uri-bytes-candidate-json-20261014.json)

## Reproduction

Build baseline and candidate libraries with `scripts/build-native-class-library.py`,
using the same compiler/bootstrap/translator and only the URI source change.
Stage each into a separately hashed development bundle. Build
`benchmarks/jit-http/Server.rvn` and `benchmarks/http-server/JsonServer.rvn` with
`Model.rvn`, the `http` native profile and an explicitly empty
`RavenNeoClrRuntimeAsyncHelper` property; retain the `Person` reflection root for
JSON. The retained comparison records build commands and input hashes.

```sh
python3 benchmarks/http-server/profile.py --build BUILD_JSON \
  --workload plain --allocation-sites --output NEW_PROFILE
python3 benchmarks/http-server/compare-native-library.py \
  --baseline BASELINE_FIXTURES --candidate CANDIDATE_FIXTURES \
  --rounds 7 --output NEW_TIMINGS
```

Fixture roots contain `plain/native/build.json` and `json/native/build.json`.
Use `--workload json` for the separate JSON allocation profile. The comparison
runs no profiling instrumentation and alternates baseline/candidate order.
