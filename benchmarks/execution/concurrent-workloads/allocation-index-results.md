# Owned allocation-start index experiment (2026-10-25)

## Research and choice

The [anchored walk evidence](walk-anchor-results.md) leaves individual searches
above a thousand headers. Increasing recent-hint capacity would not establish a
complete address mapping and still depends on exact-base-cache residency.

.NET's current `find_object` uses brick metadata and segment lookup, with explicit
heap-range preconditions and different paths for object regions. That is evidence
for indexed location, not permission to treat every address as a live object.
See [CoreCLR gc.cpp](https://github.com/dotnet/runtime/blob/main/src/coreclr/gc/gc.cpp)
and its [bookkeeping layout](https://github.com/dotnet/runtime/blob/main/src/coreclr/gc/card_table.cpp).
Go's arena metadata maps pages to spans; `spanOfHeap` also checks span state and
bounds. Its source documents restrictions on lock-free index reads across
safepoints. See [Go mheap.go](https://go.dev/src/runtime/mheap.go).
These are source observations checked on 2026-10-25, not claims that neoCLR copies
either collector or implements their complete concurrent safety protocols.

For neoCLR's existing aligned, nonmoving, exclusively owned arena, test a bitmap
of allocated payload starts plus a summary bitmap of nonempty leaf words. This
avoids pointer-per-allocation tables and full rebuilds after each collection.
A page/brick map remains an alternative with less metadata but additional local
walking and split/coalesce maintenance. The bitmap trades about 1.59% of reserved
arena capacity for direct predecessor selection and simple per-allocation updates.
No public .NET-facing API or object layout is changed.

## Implemented opt-in boundary

`NEOCLR_LIVE_ALLOCATION_INDEX` affects generated experimental native collectors.
The original runtime source and production defaults are unchanged. The owned heap
gets two pointers and two word counts (32 bytes on this target), and index metrics
use three owner-thread counters (24 bytes). A single native `calloc` reserves leaf
and summary words at successful acquisition. For capacity C, bytes are
`8 * (ceil(C/512) + ceil(ceil(C/512)/64))`: 16,640 for a 1-MiB arena. Zero-capacity
acquisition needs no index allocation. Allocation failure rejects acquisition
without publishing ownership or changing the caller's arena.

Set bits only after successful allocation establishes its payload. Clear bits
before reclamation poisons or coalesces storage; clear both levels during the
coordinated owned entry reset. Release frees native metadata even when the owned
arena fails its corruption check. The marker never reads this owner-local index;
there is no new cross-thread access. Existing exact-base proofs and recent hints
are retained. Bitmap state is authoritative allocation metadata, not another hint.

Only the scalar store fallback uses the new predecessor lookup. It validates the
selected current header, allocated state and destination extent, then retains the
existing width check and old-object mutation gate. Snapshot identity validation
still precedes this path. A pointer into free space or padding does not become
valid merely because it has a predecessor. No marker, root-map, record/struct,
compaction or public API contract changes are introduced.

A fallback validates at most one allocation header. Finding its predecessor may
scan empty summary words: this is not an unqualified constant-time lookup. Each
summary word covers 32 KiB, so a 1-MiB arena has 32 summary words. Larger arenas
may need another level; cost and memory must be measured before claiming scalable
bounds. Existing collection-time indexing and ordinary native helper lookup remain
unchanged in this slice.

## Validation

The native optimized/undefined-bounds safety suite passes with the index enabled,
including existing stale-cache and deletion-logging negative controls. An independent
model tracks allocations and compares both predecessor results and actual checked
byte stores over 72,000 sampled addresses per build (optimized and sanitized).
It covers reclamation, coalescing/reuse, a large empty-summary hole, entry reset,
release/reacquisition, forced metadata-allocation failure and zero-capacity ownership.
Metrics confirm three successful metadata allocations, zero bytes retained after
release, and 16,640-byte peak metadata in the model fixture. Removing reclaim or
reset updates deliberately breaks the model test. Initial 60,000-comparison reports
are retained separately from the expanded final fixture.

Generated code passes 600 entry and 840 record-array JIT/AOT fixture cycles using
the opt-in collector. Those runs precede addition of metadata counters; the final
model and HTTP/ordinary runs exercise the counted implementation. The counters
change only acquisition/release bookkeeping, not lookup or reclamation decisions.
No compiler lowering changed. Shared heaps, arbitrary interleavings and production
default admission remain outside these results.

An additional AddressSanitizer attempt timed out before `main`. A live process
sample shows sanitizer initialization entering a recursive allocator path and
spinning in its initialization lock. Preserve the failed report, timeout log and
stack sample as `allocation-index-asan-*`; this is **not an ASan pass** or a finding
in the model/runtime code. `validate-allocation-index.py --address-sanitizer` keeps
that check available for a host where the sanitizer can initialize. The final
default validator rerun passes optimized and undefined/bounds modes and both
broken-revocation controls. ASan coverage remains an explicit validation gap.

## Measurements

Ninety ordinary executions pass exact outputs, cleanup and equal managed
allocation counts/bytes. Live CPU medians (anchored baseline / indexed):

| Sample | Baseline CPU | Indexed CPU | Reduction |
| --- | ---: | ---: | ---: |
| Tasks | 0.037259 s | 0.024962 s | 33.0% |
| Collections | 0.184151 s | 0.144629 s | 21.5% |
| Arrays | 0.063734 s | 0.059489 s | 6.7% |

Three alternating rounds across five modes and two handlers validate 61,920 HTTP
responses, including warmup. Four-client live medians:

| Handler | Baseline req/s | Indexed req/s | Baseline CPU | Indexed CPU |
| --- | ---: | ---: | ---: | ---: |
| Plaintext | 1,994 | 2,441 | 1.067439 s | 0.881259 s |
| JSON | 1,325 | 1,538 | 1.607595 s | 1.401087 s |

Throughput rises 22.4%/16.0%, with 17.4%/12.8% less CPU. Guarded-sync CPU falls
from 0.886419 to 0.741091 seconds for plaintext and 1.380539 to 1.215966 for JSON.
Live client p99 trial medians improve from 2.23 to 1.81 ms and 3.26 to 2.83 ms;
these are response latencies, not GC pauses. Ordinary sync still leads at about
5,150/3,069 requests/s, so this does not justify making concurrent GC the platform
default.

All HTTP modes/trials allocate exactly 429,856 managed objects / 33,057,881 bytes
for plaintext and 943,817 / 54,065,775 for JSON per 2,064 responses. Every indexed
ordinary and HTTP invocation records **one additional native metadata allocation,
16,640 bytes peak and zero bytes retained at shutdown**. Those bytes are requested
allocator sizes, not resident-memory measurements. The cost is about 1.59% of the
1-MiB reserved arena, a larger fraction of the much smaller live working set.
This is a CPU-for-native-metadata tradeoff, not an allocation reduction. A final
nine-run smoke cohort verifies the ordinary tool's explicit metadata-cleanup
acceptance check after the recorded totals were independently inspected.

Separate instrumented four-client runs check another 2,720 responses. Plaintext
live fallback searches validate 68,303 headers rather than 7,852,720, for the same
68,303 searches. JSON validates 98,106 rather than 8,710,510 for the same 98,106
searches. Both indexed variants have maximum one allocation-header visit; baseline
maxima are 1,238/1,311. These diagnostics do not count summary-word probes and their
timings are not used as performance evidence.

**Decision:** retain the owned index as an opt-in native candidate. Its measured
CPU/throughput gain and memory cost warrant continued integration work. Broader
reuse for exact-base/helper lookup, larger arenas and default-candidate admission
need their own checks; this change does not silently replace those services or
admit shared-worker heaps. All synchronous comparisons remain available.

## Reproduction

`concurrent-live/validate.py --sparse-index --allocation-index` checks native safety;
`validate-allocation-index.py` runs the independent model and broken-revocation
controls. Optional `--address-sanitizer` requires a working host ASan runtime.
Generated entry/record-array validators accept `--allocation-index`.
Ordinary runs use `concurrent-workloads/measure.py --guarded-control
[--allocation-index]` with retained `optimizer-build` objects. HTTP uses
`concurrent-store/measure-plaintext.py --compare-allocation-index
--include-guarded-sync --concurrency 4 --requests 2048 --pairs 3
--allow-allocation-variation`, both handlers against `optimizer-http-build`.
`live` is the existing anchored implementation, `live-index` the opt-in; the
corresponding guarded-sync control isolates work without concurrent epochs.

Raw `allocation-index-*.json.gz` reports retain commands, hashes, outputs and
individual trials. `INDEX_METADATA` reports successful native allocation count,
current bytes and peak bytes; HTTP checks zero retained metadata at shutdown.
Managed allocation counts do not include this native memory. CPU includes the
index's initialization, maintenance and cleanup. Timing cohorts run sequentially
after correctness compilation and do not use lookup/stack sampling instrumentation.
These are fixed-work closed-loop comparisons, not framework parity or pause
qualification. Sync remains the platform default.
