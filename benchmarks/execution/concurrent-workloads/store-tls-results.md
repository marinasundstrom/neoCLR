# Group private store hints with candidate state (2026-10-25)

## Evidence and boundary

After [anchored traversal](walk-anchor-results.md), a fresh four-client plaintext
CPU diagnostic checks 24,624 responses. Live top-frame observations include TLS
resolution (103), the scalar store adapter (97), header validation (80), owned-base
validation (35), owned root entry (27) and stack publication (17). The main thread
has 537 observations; the worker has 537, with 522 condition-wait top frames.
These are wall-stack observations, not CPU percentages. Diagnostic timing is not
used as performance evidence. The profile supports checking private TLS resolution
alongside the remaining heap searches and common root costs.

Group private store lookup counters, last header/payload hint, recent hints and
replacement cursor inside the existing candidate owner state. Optional walk
counters join that state only when instrumented. The prior split layout remains
available through `NEOCLR_LIVE_SPLIT_STORE_TLS`; the older `SPLIT_TLS` control still
separates all candidate fields. Range configuration is available before the state
layout is declared. No managed allocation or public contract changes.

Physical grouping does not join lifetimes: individual counters and hints retain
their previous initialization/update behavior; there is no whole-state reset.
Heap ownership and its exact-base admission proofs remain in the existing
`owned_heap` service. The marker still receives detached work explicitly and never
uses another thread's TLS. All current proof checks, reclaim/reset revocation,
header/bounds validation and mutation barriers remain in place.

This is an internal storage-layout change with the existing
[research/qualification context](packed-tls-results.md), not a .NET API divergence.
The performance question is especially relevant to Darwin's TLS resolution; no
cross-platform speedup is inferred from these measurements.

## Validation

Packed and split native optimized/undefined-bounds suites pass, including exact
walk counts, corruption, stale-header/reuse and simultaneous independent-owner
checks. Deletion and newest/older proof negative controls still abort as expected.
The optional cached-header configuration receives its own native/negative checks.
Current generated fixtures pass 600 entry and 840 record-array JIT/AOT cycles with
the retained private `speed` setting. Compiler code and managed objects are
unchanged; prior complete compiler/full-server JIT evidence is reused.

## Results

Ninety ordinary runs preserve exact outputs, lifecycle and allocation counts/bytes.
Live CPU medians (split / grouped) are 0.038847 / 0.037006 seconds for tasks,
0.192332 / 0.183235 for collections and 0.064993 / 0.063812 for arrays: about
4.7%, 4.7% and 1.8% lower CPU respectively.

Three alternating rounds across five modes and two handlers validate 61,920 HTTP
responses, including warmup. Four-client live medians:

| Handler | Split requests/s | Grouped requests/s | Split CPU | Grouped CPU |
| --- | ---: | ---: | ---: | ---: |
| Plaintext | 1,888 | 1,989 | 1.112331 s | 1.066967 s |
| JSON | 1,288 | 1,329 | 1.659756 s | 1.622684 s |

Throughput rises 5.3%/3.1%, with 4.1%/2.2% less process CPU. Guarded-sync CPU
falls from 0.919835 to 0.884722 seconds for plaintext and 1.418937 to 1.367686
for JSON. Ordinary sync remains faster (5,165/3,073 requests/s). Live end-to-end
p99 trial medians are 2.76 to 2.23 ms for plaintext and 3.39 to 3.23 ms for JSON;
these closed-loop samples do not establish pause distributions or universal tails.

Grouped live allocates 429,856 objects / 33,057,881 bytes for plaintext and 943,817
/ 54,065,775 bytes for JSON in every trial. The split plaintext live control has
one additional 528-byte allocation in one trial, and ordinary JSON sync has the
same one-object variation. All other mode/trial totals match their handler's base
count. Preserve ranges; this is not a managed allocation optimization.

ARM64 disassembly and symbol mapping confirm **twelve TLS-resolution call sites
become five** in the scalar adapter, whose instruction count falls from 446 to
433. These are static call sites across branches, not twelve resolutions on every
store. The outside-heap path still resolves separate heap ownership and candidate
state. Removed sites resolve last-payload, lookup-counter, recent-hint and cursor
TLS descriptors; the grouped owner provides those fields directly.

For the complete plaintext executable, Mach-O `__thread_vars` shrinks from 1,248
to 1,152 bytes (four fewer descriptors), and `__thread_bss` from 158,064 to 158,048
bytes. `__thread_data` stays 40 bytes. These image-section measurements account for
alignment; they are not an RSS guarantee. Filtered TLS `nm` symbols, full `size` output, extracted adapter
assembly and binary hashes are retained.

**Decision:** retain the grouped private layout. It removes verified resolution
work and modestly improves CPU/throughput without changing ownership or hint
validity. Remaining long address searches, common entry/publication costs and
collector pause/shared-worker/default qualification remain open.

## Reproduction

Use `concurrent-live/validate.py --sparse-index --profile-store-walks
[--split-store-tls]`; separately test `--header-proof`. Ordinary comparison:
`concurrent-workloads/measure.py --guarded-control [--split-store-tls]` with
`optimizer-build`. HTTP: `concurrent-store/measure-plaintext.py
--compare-store-tls --include-guarded-sync --requests 2048 --pairs 3
--concurrency 4`, both handlers with `optimizer-http-build` objects. Runs finish
compilation before timing and execute sequentially. The new `*-split-store-tls`
modes retain the previous store layout while keeping earlier optimizations.

Raw `store-tls-*` reports preserve commands, hashes, output, allocation ranges and
individual trials. The pre-change CPU profile is retained separately. End-to-end
client latency is not a GC pause distribution. Sync stays the platform default;
shared-worker/scaling and default qualification remain open.
