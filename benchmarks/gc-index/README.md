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
