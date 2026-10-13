# Adaptive native GC lookup

This collection-only benchmark isolates address-to-allocation lookup in the shared
native collector used by JIT and AOT. It is not an interpreter benchmark, an
allocation-throughput measurement, or a comparison with another runtime.

## Bottleneck and change

The collector validates block spans and builds a 256-entry address index. Each
candidate reference then walks headers within its bucket before checking exact
payload bounds. Large populations of small objects make those repeated walks
expensive, even though the worklist visits each live object only once.

The candidate counts blocks during the existing validation pass. Above 2,048
blocks it rebuilds a 4,096-entry index before tracing. Smaller populations retain
the 256-entry index. A no-inline helper confines the additional 32 KiB stack
reservation to the dense path. There is no native or managed allocation, retained
index, new object header, write barrier, or change to collection frequency.

Costs: dense collections perform another validated header walk and temporarily
use an additional 32 KiB of native stack. The threshold and capacity are provisional,
measured engineering choices, not managed contracts. Populations include free
spans. A fixed maximum index does not remove asymptotic lookup growth for very
large heaps or uneven spatial distributions. Constrained native stacks need their
own qualification before enabling this path on a new target.

The prior fixed-table pilot was rejected as the shipping design because it made
small collections pay for a larger table. Its first temporary rewrite also missed
one table-fill bound and failed; those timings are not evidence. The implementation
uses the same explicit capacity for table sizing and filling.

## Semantics and .NET comparison

This refines the existing nonmoving collector described in
[the runtime GC comparison](../../docs/runtime-gc.md). CLR's generational collection
and precise managed references remain the design baseline for future work; this
change does not implement generations or claim to outperform CLR GC. It improves
neoCLR's present conservative address lookup without weakening exact payload/base
checks, malformed-heap rejection, conditional ownership, suspended roots or
resource cleanup. Choosing a denser private lookup table changes implementation,
not `GC.Collect` or managed lifetime semantics.

## Protocol

Run from the repository root with a supported C compiler (Xcode Clang on macOS,
MSVC x64 developer environment on Windows):

```sh
python3 benchmarks/gc-index/benchmark.py --baseline 924b45110 --output NEW_DIRECTORY
```

The controller copies only the baseline collector from Git; candidate and baseline
use the same current root helper and workload. It retains source hashes, commands,
all samples, warmups and separate diagnostic runs. Each cell has one warmup process
per variant and five alternating measured pairs. Construction and final reclamation
are outside the measured collection interval. Every worker checks the surviving
graph and complete final reclamation. The holes workload reclaims interspersed dead
objects before timing, then traces across free spans. The atomic workload retains
64 large leaf buffers through a reference array.

Diagnostic copies count traversed lookup headers; their timings are not used for
speedup claims. Ordinary timed binaries have no counters. CPU contention, power
management and short sampling intervals limit small-effect conclusions. Keep raw
slower samples and report ranges; do not use this microbenchmark to predict whole
application throughput.

Local initial matched evidence: `macos-20261013.json`. At 16,384 live objects,
chain and wide collections are 4.46× and 5.24× faster respectively. Lookup header
visits fall from 70,726,400 to 5,222,400 for the chain's 100 collections. Small
64/1,024/2,048-object chains and sparse atomic buffers are within about 2% in median;
that is not a claim of improvement. Platform qualification and HTTP regression
results are recorded separately as they complete.

## Qualification and retained controls

[The initial hosted run](https://github.com/marinasundstrom/neoCLR/actions/runs/38416246629)
passes macOS/Windows socket contracts and all 18 Windows GC/root executions,
including the StackTrace pressure fixture and dense/interior-root cases. Windows
chain/wide speedups at 16,384 objects are 4.46×/5.05×; hosted macOS gives
5.62×/6.30×. These are separate within-machine comparisons, not platform rankings.
The initial Windows small controls cost about 0.4–4.2%; the hosted macOS 1,024-node
cell is 9.4% slower in median with broad overlapping ranges. All samples are retained.

A subsequent bookkeeping refinement accumulates population locally until index
validation finishes. Its local graph run still improves large chains/wide graphs
4.38×/5.24×, but a short atomic sample was 16% slower. A longer seven-pair, three-way
check (`local-small-repeat-20261013.json`) retained baseline, first adaptive and
refined binaries, increased iterations tenfold, and observed refined costs of
1.1% atomic, 1.5% small chain and 0.3% 1,024-node chain. The repeat supplements
rather than replaces the short sample. Small-heap bookkeeping is a real possible
cost; this is not a claim that every workload improves.

`local-http-20261013.json` passes all 1,536 plain/JSON JIT/AOT responses (960 timed).
Median ratios are 1.006/0.997 for plain AOT/JIT and 1.001/0.982 for JSON AOT/JIT,
with overlapping ranges. The selected graph optimization does not materially
improve these small-live-heap HTTP fixtures. Separate phase-instrumented profiles
pass 128 responses and reclaim every allocation (16,340 plain, 38,358 JSON per
worker), with zero collector/poll failures. Diagnostic timings are not compared
to uninstrumented results.

Local sanitised GC/root contracts and the conditional/host-owned/suspended-body
cleanup runner pass (`local-gc-contracts-20261013.json`,
`local-owned-roots-20261013.json`). The latter rebuilds current sources using a
previously passing toolchain recipe. Website publication remains on hold.

Final bookkeeping qualification at `8dcda0e0c` passes in
[run 38416801565](https://github.com/marinasundstrom/neoCLR/actions/runs/38416801565).
Windows chain/wide ratios are **4.63×/5.19×** and hosted macOS **4.48×/4.90×**.
Windows small/sparse median costs are now 0.4–1.8%. Hosted macOS short controls
remain variable: the 64-node chain is 12.5% slower, the 2,048-node chain 5.7% slower
and atomic buffers 4.6% slower, while the 1,024-node chain is 1.1% faster. Those
results are retained, not reclassified as improvements; local longer controls
are supplemental evidence, not grounds to discard hosted results. All dense gains
have non-overlapping observed ranges. No universal non-regression claim is made.

[Windows HTTP/HTTPS gate 38416342406](https://github.com/marinasundstrom/neoCLR/actions/runs/38416342406)
passes at `00e34b5c5`: eight default-JIT HTTP cases cover greeting, fragmented
input, duplicate length, handler error/fault, DNS, timeout cleanup and delayed
success; nested JSON over managed SSL streams passes interpreter/default-JIT/AOT.
This is the focused HTTP gate, not a full runtime-library suite. The later population
accumulator refinement is independently covered by final GC/socket qualification.
The new dense conditional-owner/cleanup regression also passes all 18 local
GC/root executions (`local-final-contracts-20261013.json`).

The final dense conditional-owner/cleanup regression also passes Windows in
[run 38417769718](https://github.com/marinasundstrom/neoCLR/actions/runs/38417769718),
with all 18 GC/root executions and macOS/Windows socket contracts successful.
`windows-final-contracts-38417769718.json` retains the exact tested input hashes.
