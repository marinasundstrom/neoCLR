# Lazy JIT through stable trampolines — 2026-10-26

Author-directed post-Preview-18 implementation and investigation. Published Preview 18
packages are unchanged. `NEOCLR_JIT_LAZY=1` opts managed JIT into first-call method
lowering and machine-code compilation; unset or `0` retains eager compilation. This does
not select a collector. Scalar probes, AOT and interpreter execution are unchanged.
Windows execution needs separate qualification.

## Design and CLR comparison

CoreCLR uses a prestub and stable entry points to select and publish method code ([code
versioning](https://github.com/dotnet/runtime/blob/main/docs/design/features/code-versioning.md),
[prestub
implementation](https://github.com/dotnet/runtime/blob/main/src/coreclr/vm/prestub.cpp)).
Its transition frames and cooperative/preemptive GC modes protect references while
runtime work occurs
([threading](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/threading.md)).
Those are implementation precedents, not a claim of CLR binary compatibility.

neoCLR retains closed-world selection, metadata/flow verification, signatures,
GC-map/static-data preparation and small entry adapters before execution. Selected
method bodies are neither lowered to compiler IR nor compiled to machine code at
startup. The first call performs both stages. Each selected managed method gets a typed
trampoline with its existing status/out-buffer calling convention. The first call
requests machine-code compilation, final frame validation, relocation and
instruction-cache publication; only then is its target published with release/acquire
ordering. Later calls use the cached target. Recursive/mutually recursive calls and
stored function pointers continue to name stable trampolines. No instruction patching or
executable-memory rewriting is needed. This is lazy method compilation, not lazy
metadata loading, tiering or speculative background compilation. Metadata verification
and flow/GC-map analysis still scan the selected module; fully demand-loaded metadata is
separate work.

A dedicated compiler thread owns the JIT module and retained lowering state. It never
receives managed references, calls guest code or manipulates the managed heap. The
mutator blocks on a channel during a cache miss. This avoids running Cranelift on
bounded managed/fiber stacks. The parent releases its compilation containers after
preparation; the worker retains the selected module, options and lowering metadata until
invocation cleanup. Unused compiler IR is never materialized. Per-method IR is released
after publication. This retains more source/metadata state than eager compilation. Code,
slots and the native library outlive host cleanup; the compiler thread is joined before
reporting a compilation failure. Compiler errors are terminal invocation failures after
the normal host cleanup path, without Rust unwinding across guest frames. Failures
formerly discovered while eagerly compiling unused bodies may now occur on first use.

The portable typed trampoline retains an extra frame and indirect call. Combined
body/trampoline machine-frame charges must fit the existing 64 KiB bound. The extra
frames may reach the stack limit at a shallower recursion depth than eager code; both
modes must return the checked stack fault and unwind roots. The current ABI does not use
Cranelift's tail-call convention, so eliminating that frame requires a separately
validated entry stub or ABI change. Do not infer better warmed throughput from lower
preparation time.

## Synchronous versus concurrent GC

Both supported paths use an owned nonmoving heap. Caller-published frames and host-owned
callback roots remain linked while the mutator waits. Argument bits in the trampoline do
not replace those roots. The compiler uses native allocations, not GC allocations, and
introduces no collection or managed safepoint during the wait. The synchronous collector
resumes normal owner-triggered collection when execution resumes. The concurrent marker
may continue its existing epoch while the owner waits; publication, barriers and
owner-only reclamation remain intact. Lazy compilation does not turn synchronous GC into
concurrent GC and does not remove concurrent store/root costs.

The prepared GC maps are typed root-storage/phase descriptors, not native-PC stack maps.
`gc_probe::Probes::enter` emits `stack_addr` for the actual argument/local slots into a
per-frame table, together with each slot's width/flags; the backend resolves those stack
addresses when compiling the method. Pre-call transient tables remain published
throughout the call, and `Probes::finish` instruments all returns with shared cleanup.
Thus startup descriptors do not pretend to encode machine instruction offsets before
machine code exists. The new deferred-lowering test exercises the same root-publication
lowering with both collector options.

Compilation is offloaded but synchronous from the guest's perspective: the calling
mutator blocks until the target is published. It does not yield to another guest task
during the wait. The concurrent GC marker can continue. Scheduling unrelated guest work
or speculative background compilation is outside this iteration. A moving collector or
arbitrary concurrent managed callers would require a new root/transition and ownership
contract.

The concurrent-GC call gate accepts only declared generated methods as deferred targets;
imported providers still require exact admitted contracts. Each requested body passes
store lowering and call/provider checks before publication. Calling a deferred target
leads through the same gate before that target can run, including recursive calls.
Whole-program stack-argument guard elision is disabled in lazy mode because a partial
body set cannot establish that proof. Local checked-store lowering remains in place.
This preserves correctness at a possible warmed CPU cost, rather than assuming that an
uncompiled body satisfies an IR-level proof.

The worker may compete for CPU with a concurrent marker on a cold call. The marker may
also finish background work while the owner is waiting. Neither effect is a GC pause
improvement without measurement. Report readiness, cold response, warmed traffic, total
process CPU, allocations and collection counts separately. The retained server profiler
uses `RUSAGE_SELF`: invocation CPU includes the compiler worker and collector worker,
first-use work, requests and cleanup. It is not a warmed-request-only CPU measure.
Whole-process CPU also includes preparation.

## Validation procedure

`benchmarks/gc-suite/matrix/prepare-lazy-jit.py` pins the new compiler runner and
sources, and pairs both modes against identical hash-verified retained HTTP assemblies,
reflection policy and native host binaries. Those hosts are frozen controls, not a
rebuild claim for current native sources. Final timed runs use production system code
placement in both modes. The preparer also accepts `--placement fallback` for controlled
follow-up work; the recursive correctness checks cover both providers. The original
manifest is retained as provenance. The HTTP driver validates response bodies and
shutdown, records readiness and each of 16 sequential cold/warmup responses before timed
pipelined traffic, and alternates mode order across rounds. The first response and
observed nearest-rank p99 of those 16 responses are retained; the latter is a small
sample statistic (effectively its maximum), not a production tail-latency claim. Builds
and qualification finish before timing. Processes are fresh, but filesystem caches are
not flushed; normal desktop activity remains present. Diagnostics split module
preparation, trampoline preparation, entry finalization, first-call lowering and
first-call code generation/publication. These phases exclude CLI metadata loading and
selection, which retain their existing separate fields.

## Bounded acceptance criterion

Good enough for this development iteration means: unused methods are not lowered or
machine-compiled at startup; first-use publication and failure cleanup are safe; typed
argument and deferred-call gate tests pass; both collector modes pass real
compute/allocation and HTTP consumers; and repeated medians report startup,
cold-response and warmed-throughput tradeoffs. It does not require a default switch,
tiering, lazy metadata loading, a custom assembly tail stub, or universal platform
qualification. Those require evidence beyond this bounded implementation.

The implementation passed 206 active compiler tests (35 opt-in tests remain ignored),
including a new assertion that preparation lowers no selected methods and requesting one
method does not lower its callee. Twelve native recursion cases cover eager/lazy, both
collectors, normal/stack-exhaustion paths, and system/fallback placement for lazy
execution: three used methods compile once and one selected method stays unlowered. Four
injected first-call error/panic failures after 20 successful method compilations reach
native cleanup/reporting and surface the expected compiler error, without a native abort
or hang. Panic containment is inside the request loop: a failed compiler is terminal,
but its code/slots remain owned until host cleanup. The final HTTP cohort below verifies
every response and shutdown against the same hardened compiler. The production compiler
also passes `cargo check`.

## Standalone startup medians

Apple M1, macOS 27.0.1 ARM64, normal system code placement. Five alternating rounds per
mode. Startup means fresh-process launch to the first `READY` in managed `Main`; the
original sample work follows. Times are seconds.

| Sample | GC | Eager median (range) | Lazy median (range) | Median change |
| --- | --- | ---: | ---: | ---: |
| compute | Sync | 2.350 (2.314–2.595) | 2.303 (2.274–2.323) | -2.0% |
| compute | Concurrent | 2.404 (2.353–2.573) | 2.349 (2.311–2.468) | -2.3% |
| steady | Sync | 2.331 (2.316–2.384) | 2.350 (2.276–2.549) | +0.8% |
| steady | Concurrent | 2.644 (2.350–2.932) | 2.330 (2.306–2.583) | -11.9% |

All 40 processes pass output, lifecycle and allocation-work equality. Compute
lowers/compiles 45 of 68 selected methods; steady allocation lowers/compiles 47 of 70.
Changes are mixed and ranges overlap; these small programs do not establish a universal
startup win. Invocation CPU includes first-call compilation and must not be presented as
warmed execution CPU. `samples.json.gz` retains every trial.

## HttpServer readiness and post-warmup throughput

Five alternating rounds, four clients, pipeline depth 16, 4,096 timed requests per case
after 16 sequential cold/warmup requests. All 164,480 responses pass body/header and
shutdown checks. These are loopback client measurements, not a saturation test or
framework ranking. Native GC hosts and libraries are retained hash-verified controls; no
published Preview 18 measurements are replaced.

| Handler | GC | Eager ready s | Lazy ready s | Change | Eager req/s | Lazy req/s | Change |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| plain | Sync | 13.554 | 10.664 | -21.3% | 4,315 | 3,793 | -12.1% |
| plain | Concurrent | 15.942 | 10.246 | -35.7% | 3,685 | 2,301 | -37.5% |
| json | Sync | 12.878 | 10.159 | -21.1% | 2,513 | 2,106 | -16.2% |
| json | Concurrent | 16.780 | 10.490 | -37.5% | 2,023 | 1,321 | -34.7% |

Readiness ranges remain in `summary.json`; no outliers are dropped. In particular, lazy
plaintext/sync includes a 21.425-second run despite its 10.664-second median. Plaintext
allocates exactly 620,331 objects / 46,647,335 bytes in every run; JSON allocates
1,508,523 / 86,258,231. The allocation work is identical across modes. Collection counts
can differ with timing and collector policy. Throughput is measured after warmup, but
paths first exercised by later traffic may still compile; it is not an isolated fully
precompiled throughput measurement.

### Cold response and observed tail

| Handler | GC | Eager first response ms | Lazy first response ms | Eager launch→first s | Lazy launch→first s | Lazy observed cold p99 ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| plain | Sync | 1.68 | 1242.46 | 13.556 | 11.907 | 1242.46 |
| plain | Concurrent | 2.29 | 2249.07 | 15.945 | 12.588 | 2249.07 |
| json | Sync | 2.38 | 1878.85 | 12.880 | 12.038 | 1878.85 |
| json | Concurrent | 2.63 | 3207.12 | 16.783 | 13.681 | 3207.12 |

Cold p99 is the median of each run’s nearest-rank p99 over its 16 cold/warmup requests.
With only 16 requests per run this is effectively the observed maximum, not a production
p99 estimate. All individual cold latencies are retained. Launch-to-first still improves
in the medians, but by less than readiness alone.

### Compilation phases and GC interaction

Plaintext lowers/compiles 692 of 1,599 selected methods (907 unused); JSON
lowers/compiles 870 (729 unused). Counts are identical in every run and both collectors.
Unused selected methods never acquire compiler IR or machine code.

| Mode | Metadata load s | Selection/verification s | Initial preparation/finalization s |
| --- | ---: | ---: | ---: |
| eager-sync-jit | 0.129 | 9.069 | 4.019 |
| lazy-sync-jit | 0.138 | 9.257 | 0.789 |
| eager-live-jit | 0.137 | 9.301 | 6.884 |
| lazy-live-jit | 0.137 | 9.096 | 0.788 |

These phase medians pool the ten runs per mode (both handlers); medians are not
additive. Lazy module/GC-map preparation is about 0.71 seconds, trampoline creation
0.071–0.073 seconds, and entry finalization below 0.001 seconds. Trampolines are not the
dominant startup cost at this 1,599-method scale. Tens of thousands of methods are
unmeasured; do not extrapolate a scalability guarantee.

| Handler | GC | First-call lowering s | Codegen/publication s | Total first-call compilation s |
| --- | --- | ---: | ---: | ---: |
| plain | Sync | 0.197 | 1.405 | 1.603 |
| plain | Concurrent | 0.317 | 2.563 | 2.879 |
| json | Sync | 0.341 | 1.848 | 2.189 |
| json | Concurrent | 0.446 | 3.379 | 3.822 |

First-call totals span the whole invocation, including startup, requests and shutdown.
They are not only first-response costs. Concurrent mode has more code generation/store
work, and its marker can run while the mutator waits. This cohort does not isolate the
causes of the throughput loss or measure GC pauses. The trampoline frame/indirect call,
per-method code placement/publication and conservative concurrent stores are candidates
for separate attribution.

## Decision and next slice

**Good enough as an opt-in development baseline; not good enough to replace eager JIT by
default.** Unused method lowering/codegen is avoided, correctness/failure lifetime
checks pass, and repeated runs show both benefits and costs. Keep `NEOCLR_JIT_LAZY=1`
explicit. Synchronous GC remains default. Windows lazy execution, moving GC, arbitrary
concurrent managed callers, memory-residency quantification and performance beyond these
inputs remain unqualified.

Author-directed follow-up after this commit: use extensive profiling to investigate optimizations and compare
sync/concurrent GC separately. Attribute trampoline/code-placement costs shared by both
collectors, then assess safe local or cached proofs for the concurrent store checks.
Selection/verification (~9 seconds here) is the separate dominant remaining startup
phase. Choose a bounded change from evidence; do not restore eager lowering merely to
recover a whole-program optimization.

## Retained evidence and reproduction

- `build.json.gz`, `sample-build.json.gz`: exact runner/source/guest/host hashes.
- `http.json.gz`, `samples.json.gz`, `summary.json`: all trials, ranges and medians.
- `recursion.json.gz`, `failure.json.gz`: native recursion/stack fault and injected error/panic cleanup.
- `tests.txt.gz`, `production-check.txt.gz`: compiler tests and production compilation.

Build the release test runner, then prepare with `matrix/prepare-lazy-jit.py` (default
`--placement system`). Run `validate-lazy-jit-failure.py` and
`validate-lazy-jit-recursion.py` against that manifest. Use `lazy-samples.py --phase
prepare` for READY-instrumented compute/steady fixtures, then `--phase measure --rounds
5`. Run `http.py --modes eager-sync-jit lazy-sync-jit eager-live-jit lazy-live-jit
--samples plain json --protocols pipeline --requests 4096 --rounds 5`. Each output
directory must be fresh. Summarize the two reports with `summarize-lazy-jit.py`. Keep
builds and qualification outside timed cohorts.
