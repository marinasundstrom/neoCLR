# neoCLR runtime bottleneck investigation — 2026-10-24

This is the consolidated investigation requested by the author, replacing isolated
optimization results as the basis for the next runtime work. Backward compatibility
is not a constraint during experimental development. Correctness and GC safety
remain requirements. The findings below distinguish measured costs, explanations
supported by source inspection, and changes still requiring validation.

**Conclusion:** there are two substantial, distinct problems. Startup repeatedly
resolves and copies metadata; native HTTP spends substantial work maintaining
execution/root state and tracing the heap. The evidence supports redesigning those
contracts together. It does not support treating allocation count, collector choice,
or adding more threads as a complete explanation.

## Current measurements and scope

Measurements use production source at `db0290fb5` on the local macOS ARM64 machine,
with newly built release interpreter/codegen executables and freshly linked native
hosts. Qualified Raven application/library assemblies are retained, hashed inputs;
this is not a rebuild of every library from current Raven sources. The HTTP native
object is retained and independently recompiled **byte-identically** before mapping
profile symbols. The earlier uncommitted object-tracing prototype is not used.

The new evidence consists of 18 checked ordinary executions, two separate startup
profiles, 14 sequential peer capacity trials, a separate four-worker stack capture,
a separate timed-GC plaintext run, and fixed four/twelve-request plain/JSON lookup
runs in JIT and AOT. Build, profiling and diagnostic work do not overlap capacity
trials. [Raw evidence, scripts and summary](../../benchmarks/execution/bottleneck-review/README.md).

These are local short-run observations, not sustained production capacity, a hardware
counter study or a confidence interval. Repetitions, ranges, first-launch outliers,
commands, source hashes and failed historical experiments remain available.

## 1. Startup: resolution and copying dominate reading metadata

Median of three executions; milliseconds. AOT pays loading/verification/codegen at
build time. Its process time below is launch plus invocation, not equivalent work.

| Sample | Interpreter process | JIT process | JIT metadata load | JIT selection/verification | JIT compile/finalize | JIT invocation | AOT process |
|---|---:|---:|---:|---:|---:|---:|---:|
| Task queue | 3,011 | 2,402 | 99.3 | 2,165.9 | 95.3 | 0.311 | 4.75 |
| Collections | 3,043 | 2,591 | 100.5 | 2,266.2 | 196.8 | 0.464 | 4.75 |

Phase medians are independent and need not sum to process medians. The first AOT
launches took 499/244 ms; subsequent launches took about 4.6–4.8 ms. Those outliers
are retained, not silently removed or attributed to guest GC. Interpreter process
CPU medians are 2.99/3.03 seconds; JIT process CPU is 2.38/2.59 seconds. These tiny
samples do not measure steady-state interpreter throughput.

One-second startup captures beginning 150 ms after launch find the same shared
work in interpreter and JIT. The collection sample yields 666/659 wall-clock self
samples respectively: `vm::resolve` 103/104, SipHasher write 78/94, Vec clone 55/62,
Type clone 23/20, plus native allocation/free and byte comparison/copying. These are
selected startup windows, not whole-process CPU percentages or allocation counts.

Source inspection explains why the recent indexes helped but did not solve startup:

- `src/library.rs::link_modules_with_object_root` clones modules, normalizes
  identities/types, joins definitions and binds references. Its work serves both
  interpreter execution and native preparation.
- `runtime/codegen/src/linking.rs::prepare` validates supplied sources and constructs
  and verifies a `LoadedProgram`, then builds another joined/specialized module.
  Closed-world selection does not currently imply that only reachable source bodies
  are validated. Deferring validation would change error timing/admission behavior.
- `src/vm.rs::resolve` returns an owned Function, with generic substitution and
  signature checking downstream. Indexed candidates still incur hashing, rich
  structural comparisons and copying. `runtime_lookup` indexes live within scopes;
  they are not a persistent resolved execution image.

**Priority:** separate source metadata from a load-set-owned resolved graph. Use
module/revision handles and stable definition indices, borrow/share immutable
method bodies and signatures, and represent closed generic substitutions separately
from full definition copies. Resolve call/type references once per relevant immutable
scope. Keep source tokens and names in a cold mapping for diagnostics/reflection.
The recent TypeDefId microbenchmark is evidence for one repeated query, not a startup
speedup; these new startup numbers explicitly include that change.

Before implementing lazy body loading, measure source/read/normalize/bind/verify/
specialize separately, including input/selected definition counts and native allocated
bytes. The current broad selection timer cannot tell which complete phase should be
removed or shared. Preserve source access, generic identity and verification behavior,
or document a deliberate change to when errors occur.

### Interpreter execution is a separate issue

The [2026-10-17 qualified corpus](../../benchmarks/execution/candidate-refreshed-20261017/README.md)
measured a 10,000-iteration integer sum at 6.39 operations/s in the interpreter,
40,932 in JIT and 41,436 in AOT, with startup excluded. JSON DOM serialization
measured 4.00/444.98/464.31 operations/s; deserialization 4.21/346.38/361.11.
These historical results establish a substantial execution-mode gap in that fixture;
they are not current-head measurements or an isolated GC cost. A compiler-only
removal of a proven redundant range guard previously recovered 47.7% interpreter
throughput, demonstrating that generated instruction shape can materially matter.

The attempted current-interpreter refresh retains package/application hashes and
changes only the executable. It fails before READY because the older package fails
canonical primitive representation validation. No batch result is reported. Rebuild
this worker and its library set coherently before profiling dispatch, type checks,
slot locking/cloning and interpreter heap lookup. The successful tiny task/collection
startup samples cannot replace a sustained compute/JSON measurement. This is an
explicit additional execution workstream, with qualification as its first gate.

## 2. HTTP baseline: workers help, per-request CPU remains high

Same checked ten-byte response, HTTP/1.1 `Connection: close`, GET over loopback,
32 connections, Go client with two processors, two-second warmup and ten-second
measurement. Two runs per configuration in forward/reverse order. CPU covers server
lifetime including startup/warmup/drain/shutdown, excluding the client. All responses
and server exits pass. One/four are runtime worker/processor settings, not affinity
or equal hard thread limits. neoCLR owners have separate 1 MiB heaps and 16 Serve
lanes each; this does not measure shared-heap worker throughput.

| Server setting | Mean requests/s | Range | Server CPU µs/response |
|---|---:|---:|---:|
| neoCLR, 1 owner | 2,886.7 | 2,858.8–2,914.6 | 344.5 |
| Kestrel, 1 processor | 22,691.4 | 22,181.2–23,201.5 | 62.2 |
| Go, 1 processor | 22,600.1 | 21,236.9–23,963.2 | 30.4 |
| Node, 1 loop | 22,316.4 | 21,922.9–22,709.9 | 32.9 |
| neoCLR, 4 owners | 7,907.2 | 7,879.9–7,934.5 | 447.6 |
| Kestrel, 4 processors | 18,298.8 | 18,081.1–18,516.4 | 162.4 |
| Go, 4 processors | 21,026.9 | 20,101.2–21,952.5 | 69.6 |

neoCLR is about 7.86×/2.31× below Kestrel throughput at these settings, with
5.53×/2.76× its CPU/response. Four neoCLR owners provide 2.74× throughput while
CPU/response rises 30%. This is evidence of useful parallel execution with substantial
per-owner work, not a server confined to the main thread. Other servers also scale
poorly in this particular connection-close/loopback setup; their figures are not
maximum capacities. Peer latency ranges and client CPU remain in summary.json.

The earlier 12–15× single-owner gap is historical. Do not compare today's neoCLR
number against yesterday's peer number or multiply successive improvement claims.

## 3. Native hot path: root maintenance is distinct from collection

A separate five-second stack capture validates 53,467 responses over the surrounding
load. It accounts for all 11,044 HTTP-worker self samples; coordinator samples are
kept separately. Each sample belongs to exactly one leaf group below. Wall-clock
sampling can include blocked time and is **not** exact CPU attribution.

| Exclusive worker leaf group | Samples | Share of worker wall samples |
|---|---:|---:|
| Root publication and frame bookkeeping | 3,100 | 28.07% |
| Managed generated code | 2,038 | 18.45% |
| Collector | 1,975 | 17.88% |
| TLS address resolution | 1,403 | 12.70% |
| Allocation and object admission | 574 | 5.20% |
| Socket/poll leaves | 693 | 6.27% |
| Native stack checks | 132 | 1.20% |
| Other retained leaves | 1,129 | 10.22% |

Categories do not distribute inlined work back to its source or charge callees to
callers. For example, TLS is separate, even when called by a GC helper. Its largest
immediate callers are native stack checks (356 samples), published boundaries (244),
frame leave (243), root publication (239), and frame entry (182). Remaining `strlen`
samples occur under frame entry (204) and transient publication (59). This is a
concrete reason to consolidate execution state and immutable execution descriptors.

`runtime/native/gc/root-probe.c::stack_roots` still traverses the ancestor frame
chain before reaching paced collection, even when no collection is due. Existing
owned-descriptor flags avoid some validation but do not remove that traversal.
A collection also walks roots. A minor/generational collector alone would not remove
publication-time ancestry traversal, frame entry/leave or repeated TLS access.

**Priority:** design one execution-state contract across generated calls, root
publication, faults, native stack checks and worker rendezvous. Admit immutable image
maps once; keep current root *values* visible at every required collection/suspension
point. Investigate an O(1) normal poll/publication path with expensive ancestry scans
on actual collection or exceptional validation paths. Requires explicit lifetime,
context transitions, exception unwinding, native reentry and parked-worker proofs.
Do not simply skip safepoints or ancestor roots. API compatibility does not constrain
this redesign; preserving tracing and cross-thread safety does.

## 4. Collection, allocation and heap lookup

A separate diagnostic run uses production pacing and an optimized build to check
plaintext responses with GC timing enabled. Approximate work per completed response,
including amortized setup/teardown:

| Metric | Value |
|---|---:|
| Managed allocations | 208.05 |
| Requested payload bytes | 15,999.2 |
| Root publications | 3,676.42 |
| Collections | 3.59 |
| Inclusive collection elapsed time | 80.93 µs |
| Largest collection pause in this invocation | 67 µs |
| Peak post-collection live payload | 51,180 bytes |
| Peak used arena extent | 85,544 bytes |

There are zero failed collections. 38,955 of 39,024 pauses fall in the
16,384–32,767 ns histogram bucket. GC time includes validation/mark/sweep and timing
instrumentation; it cannot be subtracted from uninstrumented CPU/response to obtain
an exact remainder. Peak live payload excludes allocation headers and padding;
arena extent is neither RSS nor total reserved memory. All objects are eventually
reclaimed at shutdown, which says nothing about their age at intermediate collections.

The current native collector is nonmoving, validates and builds an address index
per collection, conservatively scans ordinary object payload words, and sweeps and
coalesces allocation descriptors. Owned allocation already has a free-space index
and an exact-base cache; the historical full first-fit allocation scan is not the
current owned fast path. JIT and AOT use this native implementation; the interpreter
uses the distinct Rust heap and synchronized slots.

Current four/twelve-request JIT/AOT diagnostic pairs quantify candidate-word and
lookup work in `summary.json` (`lookupPerAdditionalResponse`). They remove fixed invocation work by differencing,
but collection scheduling still varies, so mode differences in these counters do not
establish different generated allocation behavior. JSON adds tracing work as well as
managed work. The matched generated-map cohort separately establishes 208 vs 457
allocations and 3,676 vs 5,535 publications per additional plain/JSON response.

Current AOT increments below use the same collector and owned-host configuration;
the complete JIT rows are retained in the summary. These are work counts, not timings.

| Per additional response | Plaintext | Reflection JSON |
|---|---:|---:|
| Exact-base lookups | 102 | 233 |
| Exact-base cache misses requiring validation walks | 3.5 | 8.5 |
| Descriptors visited on those walks | 1,801.4 | 4,984.6 |
| GC index construction descriptor visits | 2,127.9 | 3,533.9 |
| Ordinary object words scanned | 8,604.6 | 13,730.6 |
| GC address-index queries | 9,696.1 | 15,970.1 |
| Queries rejected outside the arena | 6,515.5 | 10,843.5 |
| GC address-index block probes | 11,666.1 | 19,158.0 |

About two-thirds of candidate queries reject nonheap words. Base-cache misses are
few but walk hundreds of descriptors each. These observations support precise
reference maps and direct address admission, but do not establish their eventual
speedup. The prior survivor-cache change reduced descriptor visits substantially
without a convincing throughput gain; the larger GC index also failed to produce
a consistent benefit. Keep CPU/response as the acceptance metric.

**Priority:** a shared hot type-layout descriptor and a heap-address lookup structure,
with cold names/reflection/signatures separate. Prove layout/producer completeness
before precise scanning. An address region/span map can locate an allocation without
walking preceding descriptors; it must handle interior pointers, exact-base admission,
reuse, fragmentation and concurrent ownership. A metadata token is not a heap address
index. A source token needs module scope; native image ordinals and source identities
remain separate even if an internal handle encodes a partition.

The image-specific precise-object prototype currently fails before HTTP readiness
with a disposed-cancellation-source fault. It is not an integrated tracing scheme.
Source review also finds separate physical layout and diagnostic recipe derivations,
including inherited-field handling: unify and test those before treating reports as
collector contracts. The exact cause of that failure remains unproven.

Generational GC remains a reasonable architectural candidate, but the data does not
yet choose it. Obtain object age/survival by size/kind, live-set evolution, root and
heap scanning time separately, and old-to-young store frequency. Then compare a
nonmoving segregated allocator/precise full collector with a nursery plus write
barriers. Moving collection additionally requires writable roots, interior-reference
relocation or pinning, native handle rules and worker coordination. The small measured
arena extent does not currently establish fragmentation or lack of compaction as the
main HTTP limitation.

## 5. JIT placement and managed HTTP work

Prior matched owned-host trials show JIT uses more CPU than AOT despite matching
allocation/publication counts. The retained JIT placement experiment improves
throughput 19.4% and CPU/response 16.1% with identical total code size. It changes both
code and static-data placement; it does not prove a hardware cache/TLB explanation.
Long-range calls and helper dynamic linking were separately tested and do not explain
the whole gap. [Placement evidence](../../benchmarks/execution/jit-layout/README.md).

**Priority:** qualify a bounded/growing executable arena policy and its cleanup,
reservation failure, concurrent images and platform behavior. Measure JIT startup,
RSS and steady-state CPU together. A fixed 64 MiB experimental reservation is not a
finished memory policy. Keep the native runtime improvements common to JIT/AOT.

Managed parser/header-buffer methods remain visible in symbolized profiles. The
root site inventory and managed parser work should be considered together: per-byte
helper calls can multiply checks and safepoints as well as actual parsing operations.
Measure calls and boundaries per parsed byte, then examine inlining, bounds-check
hoisting and nonallocating helper contracts. Do not rewrite the HTTP API based only
on raw allocation count. Async context switches and OS/socket work deserve a separate
runnable/wait/context-switch measurement before choosing a new scheduler.

## 6. Design guidance and ordered work

CoreCLR's [type-loader design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/type-loader.md)
separates module/token lookup from loaded handles and hot MethodTable from colder
EEClass data. Its [GC design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/garbage-collection.md)
separates allocation contexts, collection policy and generational tracing.
[Go's allocator](https://go.dev/src/runtime/malloc.go) supplies another concrete
alternative: size-class spans and free bitmaps. These are implementation guidance,
not a promise that copying their representation will reproduce their performance.
Existing [Java/.NET/Go research](allocation-metadata-runtime-comparison.md) records
further costs and semantic differences. The proposal below is neoCLR-specific.

| Order / workstream | Change to investigate | Evidence / success gate |
|---|---|---|
| 1 — native execution state | Image-owned maps plus cheap normal polls; consolidate root/fault/stack/TLS state | Root traversal and TLS dominate repeated bookkeeping. Reduce CPU/response in both native modes, preserve forced-GC, faults and shared-worker tests. |
| 1 — shared metadata graph, independent startup track | Stable handles, shared immutable definitions, cached resolved references/substitutions | ~2.2 s selection vs ~0.1 s load. Add subphase/allocation counters first; reduce startup and AOT preparation across multiple sample sizes with no validation loss. |
| 2 — tracing and heap lookup | One authoritative physical reference layout, precise ordinary tracing, region/slot admission | High scalar candidate volume and repeated index work. Require complete object/array/box/static/closure producers and safe fallback; compare CPU and memory, not probes alone. |
| 2 — JIT placement | Qualify executable/static-data arena policy | Prior 19% throughput signal; establish lifecycle, capacity and platform behavior before adoption. |
| 3 — collector/allocator architecture | Size classes or nursery with barriers, informed by lifetime data | No survival or mutation-age data yet. Quantify barrier/scan cost, fragmentation and pause tails; do not infer from shutdown reclamation. |
| 3 — HTTP and scheduler | Reduce managed per-byte work; measure queue delay, context switches, runnable/blocking time | Generated code remains material; four owners already run concurrently. Compare plaintext first, then JSON/header/body variants. |
| Separate interpreter track | Rebuild current batch corpus, then profile dispatch/type/slot costs | Historical large native/interpreter gap; old package fails current admission. Do not charge startup to execution or infer a Rust heap fix from native profiles. |

Use the same response contract, compiler/library identities, GC/ownership policy,
CPU scope and one/four-owner settings for every candidate. Keep an unprofiled baseline
in both run orders; retain rejected/failing runs. Add longer repeated trials and
keep-alive/off-host offered-load checks before claiming sustained peer parity.

Outstanding measurement gaps are explicit: steady-state interpreter execution,
shared-heap worker capacity, JSON capacity under the same peer protocol, metadata
subphase/native-allocation counts, object survival ages, barriers, hardware cache/TLB
counters, scheduler queue delay and non-macOS behavior. None is silently assigned a
zero cost. The next implementation should address the two first-order workstreams,
not resume a series of unrelated local cache tweaks.

## Implemented follow-up: owned root-chain proofs

The first execution-state slice now records a same-context, immutable-ancestor proof
at owned frame entry and skips repeated ancestry validation on qualifying publications.
Collection still traces current root values, and mutable/mixed chains retain the
checked walk. [Implementation and qualification](../../benchmarks/execution/owned-root-chain/README.md)
record paired AOT one/four-owner throughput gains of 18.3%/12.1%, CPU reductions of
15.6%/12.2%, and matched owned-JIT throughput/CPU gains of 7.7%/7.7%. These results
are a separate cohort; the original investigation measurements above remain intact.
The new profile places fewer samples in stack_roots while collection, TLS and frame
entry remain prominent. This advances the execution-state workstream without claiming
to complete it or improve interpreter metadata processing.

The following [binary slot-header slice](../../benchmarks/execution/slot-maps/README.md)
removes storage/transient string scans and discarded diagnostic-layout construction.
Paired AOT one/four-owner throughput improves 6.1%/4.0% and CPU/response 5.6%/4.3%;
owned JIT improves 3.4%/2.9%. Objects shrink about 1.3%. It completes fixed execution
headers, while per-frame tables, TLS, collector work and resolved metadata remain.
This does not establish once-per-image admission, interpreter gains or peer parity.
The validation record preserves a worker minimum-concurrency assertion failure and
successful identical-binary replays; the gate remains scheduling-sensitive.

The next [execution-state slice](../../benchmarks/execution/state-access/README.md)
shares root/GC TLS and reuses it for the normal publication pacing check. Final
paired AOT one/four-owner throughput improves 3.6%/2.0% with CPU reductions of
3.9%/1.8%; owned JIT improves 2.2%/2.5%. Initial four-owner evidence was near-flat,
and latency tails remain mixed. This is a modest measured step, not resolution of
the server gap. Disassembly verifies one fewer TLS-resolving call on the normal
publication path. A fresh profile leaves collection, frame entry and other TLS access
prominent. Full execution-state, immutable metadata graph and precise layout work
remain open; do not infer that further small TLS groupings will close that gap.

A [four-word conservative scan experiment](../../benchmarks/execution/scan-batches/README.md)
now tests the idea of rejecting scalar candidates before full heap lookup. Both
variants preserve liveness and six tracing counters across exhaustive word/offset
cases. The initial filter reduces plaintext throughput 1.9–2.0%; the adjustment has
only overlapping, sub-percent changes. Scalar scanning is restored. Disassembly
shows added scalar control work rather than the hoped-for SIMD filter. This rejects
these implementations, not all batching. The useful broader questions remain how to
avoid scanning scalar fields using authoritative layouts, and how much collection
work comes from each trigger before adjusting allocation/publication/idle policy.

[Collection-origin diagnostics](../../benchmarks/execution/gc-triggers/README.md)
now separate trigger frequency from scan implementation cost. At 32→64 requests,
plaintext adds three publication-budget and one idle collection/request; JSON adds
five publication-budget and about one idle collection. Neither adds allocation-budget
or pressure requests in the owned 1 MiB hosts. JIT/AOT agree on the main counts.
This supports testing a larger publication budget before more scan-loop changes,
while retaining pressure/idle behavior and measuring retention/cleanup/latency.
Counters are opt-in caller-thread requests, not generally collector completions;
worker reason transfer and per-reason timing remain open. Defaults are unchanged.


The [publication-budget follow-up](../../benchmarks/execution/publication-budget/README.md)
retains a 4,096-publication threshold. Plain request-phase collections drop from four
to one; JSON from about six to two (allocation + idle), with unchanged allocations.
Unprofiled AOT one/four-owner throughput improves 15.9%/12.1%, CPU/response falls
13.9%/11.9%; owned JIT improves 9.4%/8.2%. Peak arena extent grows about 8/13 KB
for plain/JSON. This establishes collection frequency as a meaningful cost in these
workloads; it does not resolve frame-entry/TLS costs or establish precise-layout,
generational, interpreter or peer-parity gains. Idle collections and shared-worker
coordination remain unchanged. Longer retention and fourfold publication-count
cleanup delay are explicit policy tradeoffs; further tuning needs workload evidence.


A [post-pacing profile](../../benchmarks/execution/post-pacing-profile/README.md)
records 10,696 worker samples: TLS resolution 1,476, owned entry 1,063, publication
636, collection 612 and exact-base validation 532. The coordinator is waiting in
all its samples. These are wall-clock samples, not CPU shares. Retain idle collection
pending a cleanup/progress design; its scheduler boundary can reclaim parked stacks
and abandoned native resources before waiting. A [256-entry exact-base cache](../../benchmarks/execution/base-cache-size/README.md)
reduces checked descriptor walks with about 3% plaintext capacity/CPU gains at
1.5 KiB/thread cost. Frame/TLS overhead is now the larger remaining native target;
this does not establish that caching alone can close the framework gap.
