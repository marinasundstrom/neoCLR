# Fixed-width scalar stores (2026-10-25)

The [optimized-code comparison](optimizer-results.md) retained millions of scalar
adapter calls and substantial guarded-sync cost. The native adapter serialized
bits into scratch storage before determining whether the destination required
mutation logging, then copied a dynamic number of bytes to the destination.

`live_store_bits` now uses a constant-size `memcpy` for each admitted width
(1/2/4/8). This supports unaligned destinations and preserves integer truncation
and native byte order without an extra dynamic-size copy. Scratch bytes are formed
only when an old allocation must pass through the mutation gate. Outside-arena,
inactive/new-allocation and cancelled-epoch writes use the same fixed-width helper.
The helper does not validate, authorize or synchronize writes: existing ownership,
context, width, overflow, allocation extent, snapshot identity and gate checks stay
in their previous order. No collection, root-publication exemption, metadata change,
managed allocation, public API or native ABI change is introduced. Existing CLR
heap-versus-stack/barrier research remains applicable; this is a native copy
implementation change, not a new managed value or GC contract.

`NEOCLR_LIVE_SERIALIZED_STORE` retains a comparison helper that serializes into a
scratch buffer and calls dynamic-size `memcpy`. Both variants share deferred
staging and the helper boundary; it is not a byte-identical reconstruction of the
previous complete adapter. The prior optimized-code measurements remain a separate
before cohort. The matched control isolates fixed-width versus serialized copying.

## Safety and assembly

Both variants pass the full native suite with O2 and UBSan/bounds, including the
three negative controls for dropped deletion logging and stale newest/older range
proofs. Added tests cover 291 outside/inactive/new unaligned ranges across every
supported width, checking neighboring bytes and four invalid widths without writes.
Existing 97 guarded-old ranges, detached identity, overflow cancellation, resource
lifetime, reclamation/reuse and owner reacquisition checks still pass.

The generated optimized managed consumers pass 600 entry and 840 record-array
cycles across JIT/AOT, forced/paced hosts and O2/UBSan C variants. A fresh full JSON JIT consumer also passes 128 responses, 134 completed epochs
and clean shutdown. Reuse the previous
compiler qualification: no compiler or managed objects changed in this slice.

Retained ARM64 assembly confirms the direct helper contains width branches and
`strb`/`strh`/`str` instructions, without scratch stack storage or a dynamic memcpy
call. The serialized helper retains that scratch storage and memcpy call. Clang
keeps both helpers out of line despite the inline hint; do not claim inlining.

## Measurements

Five alternating rounds of 16 warmup plus 256 measured requests use the same
`speed` objects for both collectors. All 13,600 plaintext/JSON responses pass exact
output, allocation equality, completed concurrent epochs and clean shutdown.

| Handler | Serialized live requests/s | Direct live requests/s | Serialized live CPU | Direct live CPU |
| --- | ---: | ---: | ---: | ---: |
| Plaintext | 1,562 | 1,651 | 0.183814 s | 0.181006 s |
| JSON | 927 | 1,006 | 0.299260 s | 0.275819 s |

Observed throughput rises 5.7%/8.5%, CPU falls 1.5%/7.8%. Guarded-sync CPU falls
from 0.165176 to 0.157004 seconds for plaintext and 0.269711 to 0.245525 for JSON.
Ordinary sync still reaches 4,388/2,388 requests/s with CPU 0.056945/0.090539 seconds.
This remains a sequential loopback fixed-work comparison, not saturated throughput,
framework parity, worker scaling or pause qualification. Background activity and
short runs limit small timing claims; especially the plaintext CPU effect is small.

Counts/bytes remain identical: plaintext 57,120 / 4,394,837 and JSON 124,873 /
7,163,755 per 272 responses. There is no allocation improvement to claim.

Ninety standalone runs compare both copy strategies with ordinary/guarded/live
collectors, five rounds each at 4,096 iterations. Outputs, counts/bytes and lifecycle
checks pass. The two copy cohorts ran sequentially, so these small CPU differences
are not statistically established effects:

| Sample | Serialized live CPU | Direct live CPU | Allocations / bytes |
| --- | ---: | ---: | ---: |
| Tasks | 0.082757 s | 0.081339 s | 65,548 / 2,261,361 |
| Collections | 0.423245 s | 0.408804 s | 237,580 / 7,864,689 |
| Arrays | 0.095208 s | 0.093555 s | 86,028 / 1,892,721 |

## Next evidence target

The scalar adapter's outside-arena path still resolves four separate macOS
thread-local descriptors: `owned_heap`, `candidate_active`, `candidate_context`
and `live_store_counts`. In the retained binary they occur at 0x100010080,
0x100010098, 0x100010110 and 0x100010188; assembly and `nm` agree. This is static
call-path evidence, not a measurement of the fraction of CPU spent in TLS.
Profile/consolidate genuinely related private state while preserving thread/heap
ownership, revocation and lifecycle semantics. Do not skip validation or assume an
outside-arena address proves stack ownership. Concurrent/default, pause and broader
worker acceptance remain open; production GC stays synchronous.

Reports: `direct-store-*.json.gz`, `serialized-store-*.json.gz`,
`direct-store-summary.json`, and both scalar helper/adapter assembly snapshots.
Use `concurrent-live/validate.py --sparse-index [--serialized-store]` for native
qualification. HTTP measurement adds `--compare-serialized-store
--include-guarded-sync --pairs 5`; ordinary measurement uses `--guarded-control`
with and without `--serialized-store`. Reuse the retained `optimizer-build` and
`optimizer-http-build` objects and finish native compilation before timing.
