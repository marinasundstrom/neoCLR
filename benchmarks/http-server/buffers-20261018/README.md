# HTTP buffer and response snapshot refinement — 2026-10-18

Development after Preview 16. The author requested current-build profiling,
targeted improvements to 256-byte transfers and buffered-response duplication,
and matched measurements. This does not change the roadmap's general priorities.

## Implementation and tradeoffs

Buffered response bytes must be snapshotted before Complete returns: later changes
to the response or its supplied sequence cannot change captured wire bytes. Keep
that contract. Encode validated headers in the existing small list, then copy
headers and buffered content into one exact-sized array. This removes geometrically
growing full-response storage; it deliberately retains one independent body copy.

Send snapshots directly when their array length is at most 64 KiB, the current
native socket array admission limit. Each submission is at most 4 KiB. Larger
snapshots use a reusable 4 KiB staging block; stream producers also use 4 KiB
blocks and wait for completed writes before refilling. Tiny buffered responses
allocate no staging block. The native adapter still copies submissions into
host-owned storage, so this is not zero-copy. Bounded writes avoid reserving an
entire large snapshot against the provider's global pending-transfer byte budget.

Request reception starts at 256 bytes and grows to 4 KiB only after incomplete
parsing. The parser has copied consumed data before the old block is replaced.
Headers-first body streams retain the handed-off block and use its actual length;
client reception remains unchanged. A fragmented small request can now allocate
an additional 4 KiB block. Sustained concurrency/slow-client retained-memory
qualification remains open; do not infer global scalability from sequential tests.

Reuse the [.NET pipelines and byte-ownership comparison](../../../docs/http-foundation.md#byte-ownership-and-lazy-materialization-direction--2026-10-14).
Like the existing design, owned snapshots preserve managed lifetimes across await;
no borrowed public buffers, pooling, new public API, compiler change or runtime
policy change is introduced. Native host copies and interpreter Value storage
remain distinct memory costs.

## Current-build allocation profiles

Both libraries were rebuilt from this checkout with the Preview 16 compiler
`c0854cdda4115c9ec3153a26e73375db14011bd8` and identical other bundle components.
[Provenance](provenance.json) records exact compiler/source hashes; only
HttpServer.rvn and HttpBodyInput.rvn differ among HTTP sources. The preparation
script now recognizes native-core library configuration and excludes consumer
await-provider source imports, matching NativeLibrary.props. Earlier failed
preparations used obsolete bootstrap/consumer settings, not benchmark results.

AOT on Apple M1/macOS. Each process validates 32 responses; allocation differences
between connection closes 8 and 32 cover 24 requests. Diagnostics are separate
from timings. Allocated bytes are requested managed payload, not RSS or native
socket/TLS storage; peaks cover the whole process.

| Metric | Tiny response before → after | 8 KiB response before → after |
| --- | ---: | ---: |
| Allocations/request | 372 → 371 | 2,266 → 488 |
| Allocated bytes/request | 19,322 → 18,980 | 189,936 → 57,216 |
| Peak live managed payload | 7,888 → 7,306 | 56,806 → 40,080 |
| Peak arena extent | 12,728 → 12,072 | 93,056 → 44,848 |

The 8 KiB case reduces allocations by 78.5%, allocated payload by 69.9%, and peak
live payload by 29.4%. All four profiles have zero GC failures and reclaim every
managed allocation at teardown. Allocation attribution shows substantial baseline
send-loop/promise/await overhead, alongside full-response list growth. Profile GC
time includes instrumentation and concurrent validation; it is not a speedup score.

Raw profiles: [baseline tiny](baseline-plain-profile.json),
[candidate tiny](candidate-plain-profile.json), [baseline 8 KiB](baseline-bulk-profile.json),
[candidate 8 KiB](candidate-bulk-profile.json).

## Matched uninstrumented timing

[All raw samples and build identities](timings.json): seven alternating AOT rounds,
one full-process warmup per fixture, 32 validated requests/process with the first
eight excluded from timing. **1,024 validated responses**, zero timing-run errors;
168 measured requests per variant/workload. Builds, profilers and validation
processes finished before timing. Interactive desktop/indexing activity remained;
process snapshot and before/after load averages are retained.

| Response | Baseline median req/s (range) | Candidate median req/s (range) | Ratio |
| --- | ---: | ---: | ---: |
| Tiny UTF-8 | 158.93 (158.31–159.97) | 160.98 (160.06–162.06) | 1.013× |
| 8 KiB bytes | 9.66 (9.48–9.67) | 28.71 (28.63–28.89) | 2.973× |

This is a short, sequential, connection-close, cleartext loopback comparison on
macOS ARM64 with the standalone native host's stress-collection configuration.
Both fixtures include handler work; the 8 KiB handler creates/fills a fresh byte
list per request. Client wall time includes connection setup, validation and EOF.
The small tiny-response change is modest local evidence, not a general speedup
claim. The larger-body result supports the targeted change under this workload;
it does not establish saturation capacity, Windows/TLS performance, or a comparison
with Kestrel. Do not extrapolate these rates to paced collection or persistence.

Exploratory pooled request latencies in milliseconds (nearest-rank percentiles):

| Workload / variant | Samples | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: |
| plain / baseline | 168 | 6.27 | 6.37 | 6.63 |
| plain / candidate | 168 | 6.20 | 6.29 | 6.36 |
| bulk / baseline | 168 | 103.64 | 105.55 | 106.29 |
| bulk / candidate | 168 | 34.68 | 35.35 | 36.00 |

These percentiles describe the closed-loop sequential control only. At 168 samples
p99 is coarse, and client waiting hides overload; none is a latency-at-offered-load
claim. CPU/RSS and sustained offered-rate measurement remain follow-up work.

## Validation and reproduction

The [boundary fixture](../buffer-validation/Server.rvn) checks fragmented 4,097-byte
POST bodies and response lengths 0, 255, 256, 257, 4,096, 65,410, 65,536 and 70,000.
It mutates the supplied body and replaces response content after Complete returns;
the peer must still receive the original bytes. The interpreter uses explicit
larger array admission for these cases (2,097,152 elements, 256 MiB representation
budget); these are test settings, not new runtime defaults. Earlier attempts hit
ArrayLimitExceeded while constructing test data and are not HTTP regressions.

Use prepare-http-foundation-bundle.py for both variants, then benchmark.py
--prepare-only --workload plain|bulk. profile.py --allocation-sites accepts both
workloads. compare-native-library.py --workloads plain bulk performs alternating
uninstrumented AOT rounds after builds/validation finish. Build artifacts and exact
commands are retained under target/http-buffers-20261018; raw reports retain hashes.


[Wire validation](wire.json) passes 24 cases; [buffer/snapshot validation](boundaries.json)
passes 24 cases; [expanded stream validation](streams.json) passes 30 cases. Each
covers interpreter, JIT and AOT. The stream fixture retains 71-byte deferred reads
and adds a borrowed 8,193-byte source delivered in three producer reads, covering
4 KiB blocks and a final short block. It also covers HEAD/no-body, source errors,
invalid read counts, ownership, caller cancellation and waiting-producer timeout.
The allocation-attribution accounting unit test passes.

An [earlier expanded stream attempt](streams-first-attempt.json) passed JIT but
failed AOT with a cancelled task where the test expected ordinary completion; its
exception occurred while awaiting the case marker. The original diagnostic did not
retain the case index, so the precise cause remains unproven. The
[focused retry](streams-aot-retry.json), full three-mode retry, and
[five additional candidate AOT runs](candidate-stream-repeats.json) pass.
[Five unchanged-library AOT runs](baseline-stream-repeats.json) of the earlier
1,025-byte fixture also pass. These do not prove the failure pre-existed or has
been repaired. Keep it as an intermittent cancellation/lifecycle investigation,
not a silently discarded successful run or evidence of a throughput regression.

## Ranked follow-up measurements

1. **Establish sustained capacity.** Add longer workloads with concurrency sweeps
   through the current 16-exchange admission limit and deliberate overload beyond
   it. Drive an independent offered-rate schedule, record intended/actual send
   times, queue delay and missed arrivals, and verify the generator has headroom.
   Report successful responses/sec separately from attempted, rejected, failed and
   timed-out work. Define sustainable rate using a stated latency/error target and
   bounded memory over the full interval; do not select the highest short burst.
2. **Attribute remaining operation and GC costs.** The final 8 KiB profile still
   attributes 129 allocations/request inclusively to SendNext and 62 to request
   Parse; these overlap other call paths and must not be summed. Measure scheduler
   boundaries, native copies, CPU/request and GC pauses before choosing another
   implementation change. Keep profiler instrumentation outside timed trials.
3. **Measure resource pressure.** Sweep payload sizes and fragmented/slow peers;
   measure managed live/arena memory plus process RSS/native transfer storage.
   A 4 KiB transfer reserves more native memory than 256 bytes; aggregate provider
   budgets remain unchanged. Sequential passes do not qualify full-duplex or
   maximum-concurrency admission under this tradeoff. Reproduce the intermittent
   cancellation observation with case-indexed diagnostics.
4. **Separate protocol and application costs.** Retain connection-close controls;
   implement and qualify persistent connection ownership before benchmarking reuse.
   Compare plain, JSON, streamed and TLS workloads independently. Routing and
   serializer changes need their own profiles, not inference from the plain case.
5. **Compare equivalent servers.** Run .NET/Kestrel and other implementations with
   the same HTTP version, connection reuse, payload validation, TLS policy,
   concurrency and offered load. Prefer a separate load-generator host; retain
   warmup, host activity, artifact hashes, all rounds and failure counts.

This is a ranked investigation plan, not a claim that all bottlenecks are known
or a commitment to adopt every proposed optimization. New evidence may reorder it.
