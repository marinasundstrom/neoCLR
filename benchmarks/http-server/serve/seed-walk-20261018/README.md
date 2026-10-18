# Skip an unnecessary collection seed walk — 2026-10-18

The author requested another measured improvement after returning to GC/Serve
benchmarking. The [new profile](../gc-return-20261018/README.md) identifies allocation,
collection and root publication as the leading costs.

## Change and safety

The native collector already validates every allocation header when building its
per-collection lookup index. That same pass now records stale mark bits and up to
32 allocated intern entries. With clean marks and no overflow, it seeds those
entries directly, avoiding the full reset/seed walk. Otherwise the original full
walk runs unchanged. Dense index rebuilding collects the same information from
already validated spans.

The bounded list costs 256 bytes of temporary pointer storage per index construction
on this 64-bit target; the nested dense path can retain two lists (512 bytes). It
adds no heap allocation or retained per-connection memory. Overflow at the 33rd
entry is a normal fallback, never truncation of the roots.

The flag and candidate list are local to one synchronous collection. No guest code, cleanup callback
or heap mutation occurs between index construction and its use. Nothing is cached
across collections, contexts or reused image addresses. All header, exact-base,
fault, parked-frame, conditional-root and resource checks remain in place before
marking. A malformed later header still fails without partially resetting marks.
Intern entries retain their original String owners, and stale valid mark states
still require a complete reset and retrace. Allocation layout, public APIs,
collection policy and object lifetimes are unchanged.

This follows the existing [.NET/collector comparison](../../../../docs/experiments/aot-console/native-gc.md#comparison-and-provisional-choices):
reusing observations within a validated, non-mutating collection phase is an
implementation optimization, not a semantic divergence. It applies to the shared
native collector used by JIT/AOT; the interpreter collector is unchanged.

## Rejected allocation experiment

Splitting the allocator's first-fit search from its mandatory trailing validation
walk did not improve early-hole cases and slowed late-hole/append cases. It was
reverted before this collection change. Five alternating pairs across 64/1,024
block heaps are retained in [the raw results](evidence/allocator-rejected.json),
with [workload](evidence/allocator-workload.c), [build commands](evidence/allocator-build.json)
and [rejected patch](evidence/allocator-rejected.patch). This is a synthetic
allocation-only experiment; it is not a Serve improvement.

## Intermediate no-intern-only version

The first collection candidate skipped the walk only when no intern entries existed.
It improved the graph matrix but slightly increased Serve CPU per completion in all
four matched pairs. A separate diagnostic found 166,134 seed walks in 166,194
collections: Serve almost always retains intern entries. This motivated the bounded
list in the final implementation rather than claiming a graph-only result as an
HTTP gain. The [intermediate patch](evidence/initial-seed-only.patch),
[collection confirmation](evidence/gc-confirm.json), [diagnostic counts](evidence/initial-seed-counts.json)
and [full alternating HTTP run](evidence/initial-comparison.json) are retained,
including missed arrivals and failed Go generator/latency controls.

The final diagnostic sees at most two intern entries and zero fallbacks in 166,735
collections. Those counts include startup/shutdown and come from a separately
instrumented run, excluded from capacity timing. [Counts](evidence/intern-seed-counts.json),
[diagnostic source](evidence/intern-diagnostic.c), [build](evidence/intern-diagnostic-command.json)
and [request outcomes](evidence/intern-diagnostic.json) preserve provenance.

## Collection benchmark

The existing graph matrix runs five alternating baseline/candidate pairs, after
warm-ups, with collection scale 10. Setup/allocation are outside collection timing.
The baseline is `2d3ff4890`; source and workload hashes, commands, individual samples
and separate diagnostic counts are retained in [the final report](evidence/gc-final.json).
macOS ARM64, Clang -O2; these are collection-only results, not HTTP throughput predictions.

| Shape | Nodes | Baseline median seconds | Candidate median seconds | Collection throughput gain |
| --- | ---: | ---: | ---: | ---: |
| chain | 64 | 0.082836 | 0.071242 | 16.3% |
| chain | 1024 | 0.166008 | 0.142949 | 16.1% |
| chain | 2048 | 0.213474 | 0.190546 | 12.0% |
| chain | 2049 | 0.153545 | 0.132059 | 16.3% |
| chain | 4096 | 0.191842 | 0.166710 | 15.1% |
| chain | 16384 | 0.318280 | 0.281877 | 12.9% |
| wide | 16384 | 0.241591 | 0.205162 | 17.8% |
| holes | 4096 | 0.314959 | 0.265978 | 18.4% |
| atomic | 64 | 0.062475 | 0.049047 | 27.4% |

Reproduce with `benchmarks/gc-index/benchmark.py --baseline 2d3ff4890
--output NEW_DIRECTORY --rounds 5 --collection-scale 10`.

### Intern and fallback controls

The expanded workload adds two intern entries (matching the observed Serve maximum)
and a 33-entry overflow case. Five alternating pairs at collection scale 10 show
16.5% higher collection throughput for a 64-node chain with two interns, and 6.5%
for 4,096 nodes. The 33-entry fallback is **3.1% slower** than baseline (about 3.0%
lower throughput): collecting the bounded seed information has a cost even when
the full walk is ultimately needed. This is a deliberate bounded scratch-space
tradeoff, not an improvement for every heap. It should be revisited if intern-heavy
workloads become common. No fallback roots are omitted.

[Full paired controls](evidence/gc-intern-controls.json) include checks that interned
owners survive tracing and ordinary graph objects reclaim before explicit entry
reset. Reproduce with the same benchmark command plus `--shapes intern intern-overflow`.

## Correctness

All 46 native GC/root test executions pass with UBSan/bounds; the
[full report](evidence/validation.json) retains commands, expected rejections and
source hashes. The new sixteen-case regression covers sparse/dense indexes, clear
and stale mark bits, zero/one/32/33 intern entries, child tracing and exact reclamation.
Existing corruption, ownership, resource and parked-frame tests remain enabled.
Windows was not rerun in this slice.

## Matched Serve impact

Two reversed-order rounds use the existing matched Serve/Go workload: 16 lanes,
64 client slots, ten bytes, HTTP/1.1 connection close, queued handler work and
30 seconds per rate. The qualified guest object and experimental paced-GC policy
are fixed; Go uses GOMAXPROCS=1/GOGC=100. Builds, tests and diagnostics finish before
timing. This does not measure the new compiler/library cohort from the ancestry work.

| Entry/round | Offered/sec | Timed completions/sec | Missed arrivals | CPU ms/completion | Offered p99 ms | Generator lag p99 ms | Target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| baseline 1 | 300 | 279.87 | 604 | 2.430 | 736.23 | 505.89 | Fail |
| baseline 1 | 400 | 399.93 | 0 | 2.305 | 150.10 | 1.18 | Pass |
| baseline 2 | 400 | 385.27 | 380 | 2.506 | 217.21 | 2.06 | Fail |
| baseline 2 | 300 | 299.60 | 12 | 2.527 | 181.86 | 7.02 | Fail |
| candidate 1 | 300 | 299.97 | 0 | 2.397 | 8.48 | 0.50 | Pass |
| candidate 1 | 400 | 399.30 | 20 | 2.212 | 123.64 | 1.57 | Fail |
| candidate 2 | 400 | 399.97 | 0 | 2.207 | 24.91 | 0.51 | Pass |
| candidate 2 | 300 | 299.50 | 14 | 2.506 | 160.74 | 11.17 | Fail |
| go 1 | 300 | 300.00 | 0 | 0.227 | 8.79 | 3.39 | Pass |
| go 1 | 400 | 400.00 | 0 | 0.188 | 9.41 | 3.31 | Pass |
| go 2 | 400 | 400.00 | 0 | 0.197 | 5.83 | 2.14 | Pass |
| go 2 | 300 | 300.00 | 0 | 0.239 | 24.49 | 6.72 | Fail |

At 400/sec the candidate uses 2.212/2.207 ms CPU per timed completion, compared with
2.305/2.506 ms for baseline. The first pair is about 4.0% lower CPU/completion;
the second baseline deteriorates, so its larger apparent reduction is not attributed
entirely to this change. The 300/sec trials are affected by generator stalls/losses
and do not establish a repeatable CPU improvement. There is no universal HTTP
speedup claim and no higher reliable sustained rate: the candidate misses arrivals
in two of four runs, and one Go control fails the dispatch-lag target. All admitted
requests validate, server failures are zero, and shutdown accounting agrees.

The comparison holds the guest and link flags fixed while changing native collection
code. Results remain same-host observations with visible scheduling noise. GC graph
throughput is a more repeatable signal for this bounded improvement; HTTP capacity
needs longer quiet-host runs. Collection defaults remain unchanged.

[Run order/commands](evidence/comparison.json), [candidate build](evidence/candidate-build.json),
and lossless compact/raw pairs are retained for [baseline 1](evidence/baseline-1.json),
[baseline 2](evidence/baseline-2.json), [candidate 1](evidence/candidate-1.json),
[candidate 2](evidence/candidate-2.json), [Go 1](evidence/go-1.json) and
[Go 2](evidence/go-2.json).

The updated local website build checks 4,449 pages. Publication is separate and was
not performed. The original nine-case graph report uses the pre-existing workload
from `2d3ff4890`; the additional intern controls use the expanded workload committed
with this slice. Each comparison holds its workload constant and records its hash.
