# Address-ordered owned-heap free index — 2026-10-23

The [cost investigation](../cost-investigation/README.md) found approximately 195
block-header visits per allocation on plaintext HTTP. The runtime now uses an address-ordered
free list in explicitly owned native heaps: allocation searches only free blocks,
while preserving the previous first-fit placement and splitting policy. External
arenas retain the complete validating walk, including the tail after a fit.

## Contract and tradeoffs

Each free descriptor reuses its unused `bytes` word as the next free offset plus one;
zero terminates the list. Allocated descriptors retain their original byte-count
meaning. The thread-owned `free_head` replaces `first_free`; persistent thread-state
size, the 32-byte descriptor, object identity/payload layout, zeroing, arena reservation,
GC budgets and address stability are unchanged. No free-list node allocation, array
index or per-object header expansion is introduced.

Sweep already walks and coalesces blocks. It rebuilds the list as each free run closes
before a live block; the trailing free run is trimmed from arena extent and omitted.
Splitting replaces the selected node with its remainder; consuming a node unlinks it.
New tail allocations leave existing free nodes intact. Release clears the owner state;
reacquisition still requires an empty arena. This is a free-space index, not an object
membership index, collector generation, TLAB, or shared-heap allocation protocol.

Allocation checks every visited node's span, free state, offset alignment and forward
link bounds before changing metadata. Links must advance beyond the current span,
preventing cycles during the walk. Collection and ownership release reconcile the
entire list against the authoritative block traversal: missing nodes, links into
allocated blocks, duplicates and extra trailing links fail. A failed collection does
not rebuild the list or reclaim objects; a failed release still relinquishes the
scope. External callers cannot acquire a nonempty arena to import a fabricated list.
Owned descriptors remain runtime-controlled, as in the preceding ownership contract.

Compared with the [.NET/Go/HotSpot allocation guidance](../../../../docs/design/allocation-metadata-runtime-comparison.md),
this is a deliberately smaller optimization: skip known live blocks without changing
placement, lifetime or introducing size classes. Worst-case search remains linear in
free runs; very fragmented heaps can still be expensive. Size-segregated indexing or
a generational allocator remains a later option. Validation adds free-list checks to
existing full walks; this is included in the measured end-to-end result.

## Validation

`native-gc-free-index-test.c` compares indexed allocation against the unchanged
external first-fit allocator for 4,000 deterministic varied-size operations, retaining
and dropping roots through collections. It requires equal returned offsets, arena
extent, failure outcomes, zeroed payloads and surviving contents. It also checks
reclamation, scope reset, malformed/cyclic/out-of-range/missing links and recovery
after rejected collection/release. Threaded builds repeat the differential workload
on two simultaneous independent owners, in addition to the main-thread run.

- The native GC/root suite passes (168 recorded commands, including expected rejection
  cases), with the new fixture included in the maintained cross-platform runner.
- Focused optimized and UBSan/bounds runs pass all three differential executions
  (12,000 allocation rounds per configuration) and corruption cases.
- Recompiled shared-worker tests pass in optimized/UBSan configurations: each completes
  32 graph and 32 fault invocations, reaches four active participants, and performs
  24,544 graph plus 448 fault collections. Shared workers retain the checked external
  allocation path; this is regression evidence, not shared-heap async qualification.
- Forced-GC sanitized HTTP validates eight responses across two complete invocations.
- Separate optimized/sanitized counter fixtures check exact allocation-search counts
  on known append, split and failed-fit paths, followed by owner reset.

The broad suite ran before adding the optional threaded part of the new fixture;
that addition is excluded without `NEOCLR_TEST_THREADS`, and the final focused builds
exercise it explicitly. All production runtime source hashes match the retained runs.
The initial shared-worker replay found that the older report did not hash its output
object; the final runner instead checks the recorded guest/library hashes, recompiles
with the current backend, then links current runtime sources. No old unverified object
is used as final evidence. Windows execution was not available in this local slice;
its maintained C11 test inclusion is not a claim of a Windows run. UBSan is not TSan.

## Mechanism and capacity

A separate four-owner diagnostic run records **7.62 free-block visits per allocation**,
versus 195.12 full block visits in the preceding source-matched diagnostic: about 96%
less search work. Counters still show approximately 208 allocations, 16 KiB payload,
3,677 root publications and 3.60 collections per response. This changes the cost of
allocation, not how many objects the managed pipeline creates.

Maximum arena extent among the four diagnostic owners is 99,960 bytes, versus 92,560
in the previous diagnostic; both are far below the 1 MiB reservation. Those are separate
network runs with different interleavings, not a claim of identical resident memory.
The differential test establishes identical placement/extent for identical workloads.
The candidate retains zero GC failures, complete invocation reclamation and balanced
async stack mappings. Diagnostic timings are not used as capacity results.

Eight uninstrumented trials compare the prior allocator (`e65aefc1f`) with the indexed
allocator using the same generated object, native flags, context reuse, owned scope,
pacing and ten-byte response. One/four isolated owners run forward/reverse; each trial
has two seconds warmup and ten measured, 32 connections and the same Go EOF driver.
Only `native-gc.c` differs. Builds and diagnostics do not overlap measurement.
CPU/response is server lifetime CPU minus client CPU divided by all warmup/window/drain
responses, so it includes startup/shutdown. Every response and process exit is checked.

| Trial | Requests/s | Server CPU µs/response | p99 ms |
|---|---:|---:|---:|
| baseline-w1 | 1,897.9 | 527.2 | 22.06 |
| baseline-w4 | 5,861.3 | 635.2 | 6.46 |
| candidate-w1 | 2,280.4 | 437.5 | 21.27 |
| candidate-w4 | 6,889.2 | 534.7 | 6.02 |
| candidate-w4 | 6,246.0 | 572.3 | 6.95 |
| candidate-w1 | 2,220.5 | 447.5 | 25.54 |
| baseline-w4 | 5,200.8 | 684.1 | 10.98 |
| baseline-w1 | 1,800.7 | 550.8 | 31.32 |

With 1 owner, mean throughput rises from 1,849.3 to 2,250.4 requests/s
(**21.7%**), while CPU/response falls from 539.0 to 442.5 µs
(**17.9%**).
With 4 owners, mean throughput rises from 5,531.1 to 6,567.6 requests/s
(**18.7%**), while CPU/response falls from 659.7 to 553.5 µs
(**16.1%**).
Both candidate trials outperform both baseline trials in each worker configuration.
Baseline throughput drifts down about 5%/11% at one/four owners across the run;
tail latency is variable, so these means are evidence of a gain rather than precise
steady-state capacity estimates.

Short desktop trials do not establish sustained capacity, precise attribution of all
CPU savings or Kestrel parity. No fresh peer comparison is made. Root publication and
TLS work remain; ThinLTO, generational collection and concurrent collector evaluation
are separate follow-ups. No compiler/Raven bridge or public API changes are involved.

## Reproduce

Use the recorded Xcode SDK environment and matching compiler/library/application
artifacts. Output directories must be new. Reports preserve commands and input hashes.

```sh
python3 scripts/validate-native-gc.py --output target/free-index-gc
python3 benchmarks/http-server/primitive-comparison/free-index/validate.py \
  --output target/free-index-focused
python3 benchmarks/http-server/primitive-comparison/context-reuse/forced.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-owned-root/candidate.o --output target/free-index-http
python3 benchmarks/http-server/primitive-comparison/free-index/diagnose.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-owned-root/candidate.o \
  --client target/primitive-comparison-refresh/client --mode timed --workers 4 \
  --output target/free-index-diagnostic
python3 benchmarks/http-server/primitive-comparison/free-index/validate-counters.py \
  --diagnostic target/free-index-diagnostic/profile.json --output target/free-index-counters
python3 benchmarks/http-server/primitive-comparison/free-index/measure.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-owned-root/candidate.o \
  --client target/primitive-comparison-refresh/client --output target/free-index-capacity
```
