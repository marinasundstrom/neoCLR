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
blocks it rebuilds a 4,096-entry index before tracing, reusing validated spans. Smaller populations retain
the 256-entry index. A no-inline helper confines the additional 32 KiB stack
reservation to the dense path. There is no native or managed allocation, retained
index, new object header, write barrier, or change to collection frequency.

Costs: dense collections perform another bucket-building header walk and temporarily
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

## Index construction profiling — 2026-10-14

Diagnostic copies now count header validation as well as address-lookup visits,
with counters reset after construction and before timed collections. Ordinary
measurement binaries remain uninstrumented. The identical-source control in
`validation-profile-macos-20261014.json` confirms one validation per block on
small heaps and two on dense heaps: a 16,384-object chain validates 32,768
headers per collection. This identifies redundant validation work, not its exact
CPU share or a measured optimization. Small control timings remain noisy.

## Reuse validated spans for the dense index — 2026-10-14

The dense lookup table now reuses the complete validation performed immediately
before refinement in the same synchronous collection. No guest code, cleanup
callback or heap mutation intervenes. Its bucket-building walk remains; it skips
only redundant header validation and population counting. Small heaps retain the
existing path. This is a refinement of neoCLR's current nonmoving implementation,
not a change to the .NET-style reachability contract, collection policy, generations
or public APIs. The cost is a separate private table-fill loop whose precondition
must remain explicit if the collector becomes concurrent.

The seven-pair, tenfold-duration macOS repeat
(`validation-reuse-macos-20261014.json`) measures 2.1–3.3% higher collection
throughput on dense chains, wide graphs and fragmented heaps. Small/sparse controls
are within 1%. Validation calls halve on dense heaps; lookup visits and reclaimed
counts are unchanged. This modest microbenchmark gain does not establish HTTP or
whole-runtime improvement. The shorter pilot is also retained; it overlapped GC
contract compilation and is superseded for timing conclusions by the longer run.

All 35 sanitized local GC/root executions pass, including trailing dense-heap
corruption rejection with byte-for-byte heap preservation, unchanged conditional
outputs, no resource cleanup, and successful collection after repair. Existing
interior roots, holes, suspended ownership, abandonment and full reclamation pass.
Use `--collection-scale 10` to reproduce longer timing intervals. Hosted platform
qualification follows separately.

The matched AOT plain/JSON HTTP comparison passes all 768 responses (five
alternating pairs plus warmups) with no observed regression. Ranges overlap; the
1.01× JSON and 1.04× plain medians do not establish an application gain. These
small-heap, state-machine guest fixtures control the shared collector change; they
are not fresh runtime-await SDK or saturation benchmarks. The two separate phase
profiles per workload pass another 128 responses with complete reclamation and
zero collector/poll failures. Input identities and all timings are retained.

The profiles still show 86–91% of stress collections with no intervening allocation:
160,756–160,760 of roughly 177,100 plain collections, and 233,588–233,590 of
roughly 271,950 JSON collections. Root seeding/tracing remain larger intervals than
index construction. This supports the existing next priority of sustained/concurrent
pacing qualification rather than inferring that dense lookup tuning solves HTTP
throughput. No collection can be skipped solely because allocation has not changed:
roots and ownership may have changed, and idle cleanup must remain reliable.

Hosted [run 38475210948, attempt 1](https://github.com/marinasundstrom/neoCLR/actions/runs/38475210948/attempts/1)
at `eb0223df1` passes macOS and Windows GC/root and socket/package gates. Windows
dense collection throughput improves 3.6–6.9%; small controls are within 0.1%.
All reported diagnostic visits/reclamation counts preserve the expected invariants.
Hosted macOS timings are noisy, including a sparse atomic median ratio of 0.794
with baseline 0.081–0.198 s and candidate 0.074–0.126 s. These samples are retained;
no hosted macOS speedup or regression-free claim is made from this attempt. A
macOS-only repeat checks the unresolved variability without discarding this run.

The macOS-only repeat (attempt 2) also passes all contracts. Dense ratios range
1.04–1.22×, while the sparse control is 0.988×; small controls range 0.970–1.063×.
The reversal and variability confirm that hosted macOS is unsuitable for resolving
this small effect precisely. Retain both attempts; use the stable local seven-pair
macOS run and hosted Windows pairs for the bounded improvement claim. Neither
platform run changes pacing defaults. Website content is updated locally only.

## Direct marking of atomic payloads — 2026-10-14

Following the static HTTP catalog, matched profiles identified collector overhead
as the next target. The collector previously enqueued text and byte payloads, then
popped them only to discover that they contained no references. It now marks those
leaves directly. Reference-bearing objects, arrays and intern records still enter
the worklist. An invocation-local counter of newly marked leaves keeps dependent
and parked-root iteration running when a newly reachable owner creates no worklist
entry. A reverse-ordered dependent chain of atomic owners tests this case explicitly.

This is an internal optimization of the existing nonmoving collector, compared
with the CLR implementation layers in [the GC design](../../docs/runtime-gc.md).
It changes no managed contract, collection policy, object layout or runtime ABI.
It adds a kind branch and a counter update for each newly marked leaf; the counter
is local to a collection. Full heap validation, interior-pointer checks, corruption
recovery, suspension roots and eventual reclamation remain required.

Against `70155c01e`, seven alternating macOS ARM64 AOT HTTP rounds validate 1,024
responses. Identical guest objects and host settings isolate the collector change:
plain throughput is **156.17 → 163.50 requests/s (+4.7%)**, JSON
**64.92 → 68.02 (+4.8%)**. Ranges do not overlap in this run. These short sequential
stress-GC/state-machine measurements recover roughly the earlier static-catalog
cost; they do not establish saturation throughput, runtime-await performance or
Windows performance. Allocation counts remain 378 plain / 810 JSON per request.
Separate diagnostic processes reclaim every allocation (12,120 plain; 25,948 JSON).
Diagnostic timers ran with other validation work and are not timing comparisons.

The collection-only matrix uses five alternating pairs at collection scale 3.
Atomic-buffer throughput improves **12.2%**. Most reference-graph controls are
within about 1.1%; the 4,096-node chain improves 5.6%, which is retained without
attributing that control's change to eliminated leaf work. Graph survival and final
reclamation pass in every worker. All 35 shared GC/root checks and all 132 HTTP
wire checks pass; the wire suite uses runtime-await in interpreter/JIT/AOT. The
native static-storage GC/non-GC tests also pass. Windows execution is not qualified
by this local slice.

Evidence: [collector matrix](leaf-mark-matrix-macos-20261014.json),
[HTTP timings and exact relink commands](leaf-mark-http-macos-20261014.json),
[shared GC contracts](leaf-mark-contracts-macos-20261014.json),
[HTTP contracts](leaf-mark-http-contracts-macos-20261014.json),
[plain allocation profile](leaf-mark-plain-profile-macos-20261014.json),
[JSON allocation profile](leaf-mark-json-profile-macos-20261014.json).
The HTTP candidate was built before a comment/read-only parameter-qualifier cleanup;
source hashes identify the tested collector copies. No generated guest code changed.

Reproduce the isolated graph test with:

```sh
python3 benchmarks/gc-index/benchmark.py --baseline 70155c01e \
  --rounds 5 --collection-scale 3 --output NEW_DIRECTORY
```

HTTP evidence retains both link commands: relink the same static-catalog guest
objects against the baseline and candidate collector copies, then use
`benchmarks/http-server/compare-native-library.py`. Keep profiler instrumentation
out of timing binaries.
