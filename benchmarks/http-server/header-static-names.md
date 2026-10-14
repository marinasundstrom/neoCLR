# Shared static HTTP header names

Development slice, 2026-10-14. `HttpHeaderNames` is an internal static class with
13 read-only lazy properties: Host, Connection, ContentLength, TransferEncoding,
Expect, ContentEncoding, ContentType, Accept, AcceptEncoding, Authorization,
Cookie, UserAgent and CacheControl. Validated ASCII bytes select a known spelling
before the unknown-name lowercase/decoding path. Framing and token checks remain
unchanged. Only these fixed literals are interned; request input is never interned.

The first getter access publishes a canonical String into invocation-owned static
storage. Later accesses read the field directly. Unused names allocate no strings;
the nullable slots share one field block. This establishes identity reuse across
requests and collection. It does not introduce lazy values, pooled request buffers,
concurrent first-access qualification or a general type-initializer implementation.
See [storage semantics and remaining limits](../../docs/runtime-static-storage.md).

This extends the [seven-name byte matcher](header-name-bytes.md), following the
existing [Kestrel/.NET research](../../docs/http-foundation.md). Compared with CLR
statics, lifetime belongs to the hosted invocation, not the process. Fixed-name
intern publication tolerates duplicate first-use attempts, at the cost of intern
quota/pool records; no arbitrary network strings enter that pool.

## Correctness

All 44 wire cases pass in interpreter, JIT and AOT with default runtime-await:
132 checks covering mixed/uppercase and fragmented input, exact/long lookalikes,
unknown/repeated names, malformed framing, and reference identity after explicit
GC. Native tests cover separate `Cache<int>`/`Cache<string>` storage, repeated entry,
GC and non-GC context isolation, child retention and complete reclamation. Native
runtime type-handle fields without an admitted default fail closed.

The private native context grows to 1,080 bytes; hosts must be rebuilt. Static
initializers, static readonly writes and constructed generic **static class** field
owners remain compiler/runtime gaps. Ordinary generic classes with static members
are tested. Windows x64 object emission succeeds; Windows execution is unqualified.
The installed SDK and published website are unchanged.

## Measurement method

Matched macOS ARM64 AOT fixtures compare the previous seven-name library with the
13-name static catalog using the same backend and native host. These performance
fixtures explicitly use compiler state machines and stress GC. Functional wire
checks use runtime-await; these timings do not qualify its performance.

Allocation profiles measure 24 requests after eight warmup requests, separately
from uninstrumented timings. Seven alternating timing rounds validate 1,024
responses including warmup processes. Loopback concurrency is one, connections
close after each request, and the client runs on the same machine. These are
exploratory sequential measurements, not production throughput or regression gates.

An initial implementation allocated one table slot for every selected type. Its
extra live storage regressed throughput by about 15–16%. The final implementation
uses dense ordinals only for types declaring statics, eliminating unrelated slots.
The initial result is retained to explain the correction, not hidden as noise.

## Results and tradeoff

| Per measured request | Plain before | Plain after | JSON before | JSON after |
| --- | ---: | ---: | ---: | ---: |
| Managed allocations | 378 | 378 | 812 | 810 |
| Requested payload bytes | 19,513 | 19,513 | 33,548 | 33,518 |
| Native TEXT blocks | 11 | 11 | 124 | 122 |

The plain fixture already used immutable image literals where identity was not
observable. The JSON fixture previously copied these names. The new catalog
provides explicit shared identity in both, rather than depending on that compiler
optimization. Each profile validates 32 responses with zero collector failures and
complete reclamation: baseline/candidate plain 12,116/12,120 objects and JSON
26,006/25,948. Peak live bytes rise by 144 in each workload; peak arena extent rises
by 296. Those startup/retention costs are distinct from per-request allocation.

Final median requests/s are **161.63 → 154.43 plain (−4.5%)** and
**67.18 → 64.28 JSON (−4.3%)**. Plain ranges overlap; JSON ranges do not in this run.
This is a remaining performance cost, **not a throughput improvement**. Dense
static-owner storage removes most of the initial 15–16% regression, but static
roots and getter execution still have costs. Their individual contributions are
not isolated here. Further profiling should address this before treating the
catalog as a performance win; the measured allocation and identity benefits stand.

Evidence: [wire checks](header-static-validation-20261014.json),
[final timings](header-static-timings-20261014.json),
[initial sparse-table timings](header-static-sparse-table-timings-20261014.json),
[compiler/library provenance and focused checks](header-static-provenance-20261014.json),
[baseline plain](header-static-baseline-plain-20261014.json),
[candidate plain](header-static-candidate-plain-20261014.json),
[baseline JSON](header-static-baseline-json-20261014.json), and
[candidate JSON](header-static-candidate-json-20261014.json).

A [subsequent collector optimization](../gc-index/README.md#direct-marking-of-atomic-payloads--2026-10-14)
removes empty tracing work for text/byte leaves. Matched HTTP runs improve about
4.7–4.8% against this static-catalog implementation, approximately recovering its
measured cost. The original results above remain the baseline evidence.

## Reproduction

Use the controllers retained alongside this report:

```sh
python3 benchmarks/http-server/check-header-names.py \
  --bundle MATCHING_BUNDLE --aot MATCHING_CODEGEN --output NEW_CHECKS
python3 benchmarks/http-server/profile.py --build BUILD_JSON \
  --workload plain --allocation-sites --output NEW_PROFILE
python3 benchmarks/http-server/compare-native-library.py \
  --baseline BASELINE_FIXTURES --candidate STATIC_FIXTURES --rounds 7 --output NEW_TIMINGS
```

Use `--workload json` for JSON. Timing roots contain `plain/native/build.json` and
`json/native/build.json`. Public API reference signatures are unchanged: the catalog
is internal. Compiler changes were validated from working binaries subsequently
committed as Raven `fac4ac736`; recorded binary hashes identify the tested inputs,
rather than claiming an exact clean-commit build.
