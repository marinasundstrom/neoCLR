# Start checked heap walks from admitted allocation hints (2026-10-25)

## Measured problem

The [concurrent-client matrix](client-concurrency-results.md) exposed worse guarded
store CPU under overlap. Optional `NEOCLR_LIVE_WALK_PROFILE` instrumentation now
counts fallback header visits, maximum visits per search and free-header visits.
Instrumentation is absent from ordinary builds. Both runners mark instrumented
cohorts diagnostic-only; their timings are not benchmark evidence.

For 272 responses, live plaintext visits 10,998,632 headers in 62,068 fallback
searches with one client (177/search), versus 34,162,149 in 68,303 searches with
four clients (500/search). JSON goes from 29,494,478 / 99,874 (295/search) to
55,876,481 / 98,106 (570/search). Maximum searches reach 680/1,264 headers for
plaintext and 938/1,354 for JSON. These are counts, not sampled CPU attribution.
The ordinary collections workload visits 95,387,468 headers in 282,177 live
fallbacks (338/search). Many calls to `block_valid` therefore represent repeated
linear traversal, not merely an expensive single validation predicate.

## Change and ownership argument

The store adapter already keeps four recent allocation hints. A hint that misses
containment can still establish a valid header boundary below the destination.
Remember the greatest such boundary among hints admitted by the current owned-base
proof and validated header, then start fallback traversal there instead of zero.
Continue checking every traversed header and the final payload/width bounds.

This adds one local offset, no persistent index, TLS cache, managed allocation or
object-header bytes. It never derives a starting header from arbitrary payload
bytes. Ownership admission precedes dereferencing each hint; reclamation, coalescing,
reset and release revoke those proofs. The hint must name a currently allocated
block with a payload address at or below the destination. Exact-base and snapshot
identity paths remain unchanged. Marker synchronization, mutation logging and
cancellation remain unchanged.

The heap API does not promise a whole-arena validation on every store: existing
exact-base/contained-range paths already skip unrelated earlier blocks. This change
uses the same current allocation proof to skip that prefix on a fallback. It is
still a checked forward walk, not a permanent promise about changing heap layout.
No hint means starting at zero. Cache-disabled/legacy linear paths also start at
zero. `NEOCLR_LIVE_FULL_STORE_WALK` retains the previous behavior for comparison.

This is an internal nonmoving owned-heap optimization, with no .NET-facing API or
semantics divergence. Existing [research/ownership context](packed-tls-results.md)
applies; shared heaps remain unqualified. It deliberately avoids introducing a
new address index before measuring how far current proofs can take us.

## Correctness

Native optimized and undefined/bounds-sanitized suites pass for anchored and full
walks. A new small fixture independently checks exact visit totals, excludes cache
hits, and demonstrates a forward search starting at an older hint. Existing
corrupt-header, stale interior-header, reclaimed/reused allocation, bounds,
unaligned write, cancellation and independent-owner tests pass. Deletion logging
and newest/older ownership negative controls still abort when deliberately removed.
Current generated-code fixtures pass 600 entry and 840 record-array JIT/AOT cycles.
No compiler code changed; unchanged compiler/full-server JIT evidence is reused.

## Results

Ninety ordinary executions preserve exact outputs, cleanup and allocation counts
and bytes. Live CPU medians (full walk / anchor):

| Sample | Full walk CPU | Anchored CPU | Reduction |
| --- | ---: | ---: | ---: |
| Tasks | 0.077803 s | 0.038721 s | 50.2% |
| Collections | 0.393315 s | 0.192219 s | 51.1% |
| Arrays | 0.090685 s | 0.065025 s | 28.3% |

Four HTTP cohorts, each with three alternating rounds and five modes, validate
123,840 responses including warmup. Median live results:

| Handler | Clients | Full walk req/s | Anchored req/s | Full walk CPU | Anchored CPU |
| --- | ---: | ---: | ---: | ---: | ---: |
| Plaintext | 1 | 1,892 | 2,110 | 1.120012 s | 0.952125 s |
| JSON | 1 | 1,080 | 1,328 | 1.876198 s | 1.438811 s |
| Plaintext | 4 | 1,289 | 1,916 | 1.607117 s | 1.097955 s |
| JSON | 4 | 831 | 1,291 | 2.517309 s | 1.656909 s |

Observed throughput gains are 11.5%/23.0% for one client and 48.7%/55.5% for four;
CPU reductions are 15.0%/23.3% and 31.7%/34.2%. Four-client live p99 trial medians
improve from 3.45 to 2.42 ms for plaintext and 5.40 to 3.52 ms for JSON. These are
end-to-end request latencies, not GC pauses. Guarded-sync also improves, consistent
with removing work common to the guarded store path rather than changing marker
parallelism. Ordinary sync still leads: about 5,151 plaintext / 3,050 JSON req/s
with four clients. The optimization does not yet qualify concurrent GC as default.

Every live variant retains exactly 429,856 allocations / 33,057,881 bytes for
plaintext and 943,817 / 54,065,775 for JSON per 2,064 responses. All modes match
except one four-client anchored guarded-sync JSON trial, which has one extra
528-byte allocation (943,818 / 54,066,303); retain that range rather than claiming
universal equality. No persistent memory is added by the optimization.

Separate diagnostic cohorts confirm less traversal, not merely different timing.
At four clients the plaintext live variants perform the same 68,303 fallback
searches, but header visits fall from 34,162,149 to 7,852,720 (77.0%). JSON performs
the same 98,106 searches, with visits falling from 55,876,481 to 8,710,510 (84.4%).
Ordinary live collections falls from 95,387,468 to 15,248,303 visits for the same
282,177 searches. These counters run only in diagnostic builds. Maximum searches
remain large (1,238 plaintext / 1,311 JSON headers), so hints do not replace a
bounded address index or solve worst-case lookup.

**Decision:** retain anchored traversal in the private concurrent candidate. It
removes demonstrated redundant prefix work without introducing a new lifetime or
cache contract. Further address-index work should target the remaining long
searches, accounting for allocation/reclamation maintenance and memory cost;
common publication/metadata costs and pause/shared-worker qualification remain
open.

## Reproduction and evidence

Native: `concurrent-live/validate.py --sparse-index --profile-store-walks
[--full-store-walk]`. The profile fixture is excluded with cache-disabled lookup.
Ordinary: `concurrent-workloads/measure.py --guarded-control [--full-store-walk]`
against `optimizer-build`. HTTP: `concurrent-store/measure-plaintext.py
--compare-store-walk --include-guarded-sync --pairs 3 --requests 2048
--concurrency 1` (then 4), against `optimizer-http-build` objects, both handlers.
`live`/`guarded-sync` use anchors; `*-full-walk` are matched controls. HTTP range
reporting is explicitly enabled to preserve scheduling-dependent allocation data.
Performance cohorts run sequentially after correctness compilation and do not use
walk instrumentation. Separate diagnostic runs use `--profile-store-walks` and
256 measured plus 16 warmup responses. Raw reports retain hashes and commands.

These are closed-loop, connection-close comparisons, not framework parity or
mutator pause distributions. The broader default/shared-worker/scaling goal stays
open even if this removes a large part of current store overhead.
