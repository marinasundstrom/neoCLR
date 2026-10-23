# Exact-base cache for owned heaps — 2026-10-23

This extends the [explicit owned-heap scope](../owned-heap/README.md), using its
exclusive descriptor-write and thread-affinity contract. It does not change the
checked allocator/admission path used by external arenas or shared-object workers.

## Representation and lifetime

A direct-mapped cache holds 64 proven payload base addresses (512 bytes on the
tested 64-bit target). Allocation inserts its newly initialized payload base.
Successful checked admission can insert its proven value/owner bases. A collision
evicts the old proof and causes a later miss; the full pointer must match, not just
the hash. There are no negative entries, object-header additions or dynamic
allocations. Including the existing scope fields, owned bookkeeping is 552 bytes
of TLS on this target, even on threads that do not activate the scope.

Admission first checks the arena identity/shape. In an acquired scope, matching
positive entries satisfy the exact-base test; a miss performs the existing full
validation walk. Null retains its allowed-root meaning. Interior, tagged, padding,
foreign, freed and fabricated payload addresses cannot become cache entries
without an actual allocation or successful exact-base proof. This cache is not a
GC root: cached unreachable allocations are still reclaimed.

Successful sweep clears every entry, including proofs for surviving objects,
before returning. Release clears the entire scope, including on reported corruption.
Acquisition starts from empty scope/arena state. This deliberately favors simple
invalidation over retaining survivors or introducing per-object generations.
Raw pointers still have the existing address-reuse semantics; the cache does not
turn them into generational handles.

As with the owned allocator, native code must not overwrite descriptors or mutate/
alias the acquired arena behind the runtime. A hit does not rediscover unrelated
header corruption. GC and release still validate all descriptors, and misses still
use the checked walk. This is a trusted native ownership contract, not memory
protection. The cache cannot be used for shared heaps or cross-thread handoff.

## Comparison and alternatives

The [Java/Go/CoreCLR research](../../../../docs/design/allocation-metadata-runtime-comparison.md)
supports moving repeated execution work into bounded, purpose-specific metadata.
This implementation is a neoCLR-specific positive cache, not a claim that those
runtimes use this exact structure. It reuses already proven runtime addresses and
never interns arbitrary external input.

A heap-wide bitmap could provide predictable membership lookup, but one bit per
eight-byte position costs 16 KiB for a 1 MiB arena, plus updates and lifecycle
management. The 512-byte cache is a smaller first step with fallback on eviction.
A side index remains an option if profiling shows misses dominate. A plausible
header at value-minus-header-size is insufficient: it could be fabricated inside
another object's payload, which the tests explicitly exercise.

## Validation and reproduction

The expanded owned-scope test uses 129 allocations to force cache collisions,
checks exact allocation sizes/kinds, rejects forged/interior/tagged/header/padding/
foreign/end pointers without output mutation, exercises dependent host-root
admission, then verifies reclamation invalidates a cached middle block while
other objects survive. Reuse with a different size and release/reacquisition at
the same arena address must not retain old proofs.

Native GC/root checks, repeated forced-GC HTTP with UBSan/bounds, and optimized/
UBSan checked-path shared-worker stress are run separately from measurement.
The shared-worker result is regression evidence, not shared-cache qualification.

Use the same SDK, compiled object and source bundle as the preceding experiment:

```sh
python3 scripts/validate-native-gc.py --output target/owned-bases-validation
python3 benchmarks/http-server/primitive-comparison/owned-heap/forced.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-slot-metadata/candidate.o --output target/owned-bases-forced
python3 benchmarks/http-server/primitive-comparison/owned-bases/measure.py \
  --build target/primitive-comparison-build/build.json \
  --client target/primitive-comparison-eof/client \
  --object target/plain-slot-metadata/candidate.o \
  --baseline c4e582329 --output target/plain-owned-bases
```

Both baseline and candidate enable NEOCLR_GC_OWNED_HEAP. Baseline is the previous
allocation cursor without base caching, not the older fully checked allocation
baseline. One/four isolated owners each have a 1 MiB heap, 16 Serve lanes and
256/1,024 GC pacing. Thirty-two connections use a two-second warmup and six-second
measurement, forward/reverse. Responses are ten bytes and checked through EOF.
CPU/response includes startup, warmup, drain and shutdown. No builds or tests
overlap timed trials. These are short local comparisons, not sustained limits,
cross-platform qualification or parity with reference servers.


## Results

| Variant | Owners | Requests/s (two trials) | CPU µs/response | p99 ms |
| --- | --- | --- | --- | --- |
| owned cursor baseline | 1 | 933.0, 856.3 | 1,070.9, 1,109.7 | 35.97, 71.81 |
| owned cache candidate | 1 | 1,071.7, 1,057.2 | 936.4, 943.3 | 31.40, 46.75 |
| owned cursor baseline | 4 | 3,343.0, 3,063.7 | 1,166.4, 1,255.0 | 16.29, 26.57 |
| owned cache candidate | 4 | 3,570.7, 3,520.0 | 1,058.6, 1,070.6 | 13.76, 16.19 |

Mean throughput improves 19.0%/10.7% at one/four owners; estimated CPU per response
falls 13.8%/12.1%. Tails are lower in these paired samples, but baseline throughput
drifts about 8% between its trials and p99 varies substantially. These results
support retaining the bounded optimization, not a precise guaranteed speedup or
latency claim. Do not compare absolute rates with earlier desktop sessions.

All 24 native GC kernel groups plus root/profiling/owned-body/idle-drain/memory-view
checks pass. Forced-GC HTTP completes eight responses across two invocations and
clean shutdown. Checked-path shared-worker regression passes optimized and UBSan:
each build completes 32 healthy and 32 fault invocations, 24,544 graph collections
and 448 fault collections. Retained reports match the tested implementation.

Integration remains opt-in through NEOCLR_GC_OWNED_HEAP in the source HTTP host.
No compiler, managed API, object header or shared-heap worker contract changes.
