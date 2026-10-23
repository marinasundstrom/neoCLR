# HTTP/runtime cost investigation — 2026-10-23

The author requests a thorough investigation after completed-context reuse becomes
the macOS default, and asks whether generational GC remains an option. **Yes:** it
remains a substantive design candidate, alongside immediate allocation and compiler
work. No production runtime/compiler policy changes are made in this investigation.
Experiments use temporary native sources; wire correctness is not full qualification
of a new allocator or collection policy.

## Conclusions

1. **Fix allocation search, not object-header size first.** The current owned heap
   examines about 195 block headers per allocation, about 40,600 per response. This
   is repeated first-fit work, despite only about 208 allocations/request. Introduce
   a bounded free-space index or size classes, preserving the checked external-arena
   path. Do not simply append until the entire arena is occupied.
2. **Qualify native ThinLTO for normal builds.** The confirmation gives roughly 5%
   throughput and CPU improvements with no source-level algorithm change. It is a
   practical bounded follow-up, not a route to Kestrel parity by itself.
3. **Reduce publication frequency and per-publication metadata work.** About 3,677
   publications/request accompany byte-wise parsing/encoding and small accessor
   calls. Compiler-proven non-collecting leaves and bulk byte operations are more
   promising than further small TLS-layout changes.
4. **Design generational GC alongside those changes.** Measure survival and establish
   store barriers before selecting a nursery/promotion design. Generation policy,
   allocation policy, relocation and concurrency are separate decisions.
5. **Keep scheduler, persistent connections and shared heaps in scope, but do not
   assume more threads solve this profile.** The current four-owner server is doing
   substantial CPU work inside each owner. Its coordinator is waiting normally.

## Current baseline and method

Current native source is commit `0675a8cf3`; macOS ARM64, eight physical/logical CPUs,
16 GiB RAM. Reuse the verified generated application object
`4fdc93183f8be7f5d5fd2c143c9847875d0e5923f6ccde68cc0161f0ced6ad8c` and matching SDK bundle.
Each server has a 1 MiB arena per isolated owner, owned-heap validation/base cache,
context reuse, and 256-allocation/1,024-publication GC budgets unless stated otherwise.
The same external Go driver checks a ten-byte plaintext response through EOF at 32
connections. All peers would need connection-close for a matched comparison.

The first matrix has 16 trials (baseline, ThinLTO, O3, fourfold GC budget; one/four
owners; forward/reverse), two-second warmup and six-second measured windows. Its final
baseline trials suffer marked desktop drift: four-owner throughput falls 28%, and
one-owner throughput falls 22%. **Do not use the screening mean as a speedup claim.**
The follow-up has 12 trials of baseline, ThinLTO and append-first, using ten-second
windows and forward/reverse order. All builds finish before timed trials; diagnostics
and correctness tests do not overlap them. CPU/response uses child process CPU minus
client CPU over all warmup/window/drain completions, including startup and shutdown.

These remain short same-machine experiments with two observations per configuration,
not confidence intervals, production capacity or a new Kestrel comparison. No JSON,
TLS, large-body, keep-alive or shared-heap throughput claims follow from them.

## Fresh profile

A fresh four-owner capture validates 35,479 responses. The analyzer replays codegen
and requires byte-identical object output before assigning generated method names.
It reconciles 10,948 worker self-samples, separately from 2,737 coordinator samples
in a condition wait. **These are wall-clock stack samples, including blocked time,
not measured CPU-time shares.**

| Worker leaf location | Self-samples | Share |
| --- | ---: | ---: |
| Allocation | 2,017 | 18.42% |
| Root publication | 1,634 | 14.93% |
| TLS address resolution | 1,488 | 13.59% |
| Indexed collection | 956 | 8.73% |
| String length | 562 | 5.13% |
| Owned-frame entry | 522 | 4.77% |
| Transient roots | 446 | 4.07% |
| Exact allocation-base validation | 307 | 2.80% |

TLS samples occur under boundary checks (381), native stack checks (275), transient
publication (261), and other helpers. String-length samples occur under transient
publication (246), root publication (186) and owned-frame entry (130). Root metadata
is compact already, but the code still checks its C-string length at hot boundaries.
Removing checks on mutable/external plans would change the validation contract.
Any trusted-image path must be explicit, versioned, lifetime-bound and preserve the
checked path; global pointer caching is not a valid shortcut.

Network leaves appear, but are smaller in this capture: close 111, send 106, poll 70,
accept 60 and receive 31. This is evidence to prioritize bookkeeping investigation,
not proof that I/O or scheduling can never dominate another workload.

## Allocation search and GC interaction

`diagnose.py` generates a diagnostic-only GC source with per-owner search counters,
exported once at ownership release, plus the existing runtime snapshot counters.
A search step means one block header visited in the allocation loop. It is not a
collection scan, CPU sample or allocation. Counter tests exercise tail appends,
coalesced free space, splitting, skipped holes, retained objects and scope reset.
Optimized and UBSan/bounds versions must produce the exact expected counts.

The healthy owner snapshots reconcile allocated/reclaimed counts, zero collection
failures, and balanced stack mappings. Approximate per-response baseline totals are
208 allocations, 16,003 payload bytes, 3,677 publications, seven async bodies and
3.60 collections. Total reclamation at shutdown does **not** measure object age or
prove how many allocations would die in a nursery.

| Policy | Headers searched/allocation | Collections/response | GC elapsed µs/response | Maximum arena extent across owners |
| --- | ---: | ---: | ---: | ---: |
| Current first-fit | 195.1 | 3.60 | 85.1 | 92,560 B |
| Fourfold collection budgets | 325.9 | 0.91 | 26.5 | 153,104 B |
| Append before searching holes | 63.8 | 3.60 | 133.4 | 1,048,576 B |

These are separate instrumented runs; GC time is inclusive elapsed time, not an
uninstrumented CPU decomposition. The append-first candidate preserves ordinary
first-fit as fallback when the tail lacks space. It reduces searching, but touches
much more of the fixed arena and increases collection work. Arena extent is neither
live payload nor process RSS; the reservation was already 1 MiB in every case.

The fourfold budget candidate demonstrates why fewer collections alone are not a
sufficient objective: more surviving/uncollected blocks increase allocation search.
Its four-owner screening CPU/response is worse, and its apparent mean throughput
advantage is contaminated by the deteriorated baseline. No default budget change is
justified. Append-first is an informative ablation, not a qualified replacement: it
also changes internal address reuse and needs fragmentation/pressure/ownership tests.

## Candidate confirmation

ThinLTO applies to the C native adapters/runtime. The Cranelift application object
and prebuilt Rust archives are unchanged machine code; this is **not** LTO across all
managed methods or a replacement compiler backend. O3 affects C compilation only.

| Variant | Owners | Mean requests/s | Mean CPU µs/response | p99 ms (two trials) |
| --- | ---: | ---: | ---: | --- |
| baseline | 1 | 1,785.6 | 554.6 | 24.84; 34.68 |
| baseline | 4 | 5,193.4 | 693.3 | 8.81; 11.18 |
| thinlto | 1 | 1,869.8 | 530.3 | 27.57; 29.06 |
| thinlto | 4 | 5,486.1 | 658.2 | 7.30; 8.23 |
| appendfirst | 1 | 1,934.8 | 512.9 | 33.16; 30.27 |
| appendfirst | 4 | 5,376.1 | 669.9 | 8.25; 7.05 |

ThinLTO improves means by about 4.7%/5.6% throughput and 4.4%/5.1% CPU/response at
one/four owners. Append-first improves about 8.4%/3.5% throughput and 7.5%/3.4% CPU,
but four-owner trials overlap the baseline and arena extent rises roughly elevenfold.
All raw latency values are retained; tail results are mixed. Neither candidate is
integrated automatically. O3 has no clean confirmation and no demonstrated advantage
over the more promising ThinLTO candidate in the screening CPU results.

ThinLTO is additionally checked by relinking five existing compiled async consumers
in optimized/sanitized, reuse/fallback configurations: 20 executions / 40 invocations
cover nested awaits, cancellation/disposal, forced GC, cache overflow and fault
teardown/re-entry. This is macOS evidence, not Windows/native-kit/JIT/shared-worker
qualification. Link-time optimization can change symbol visibility, layout and build
cost; normal host, exported native ABI and package/cache checks remain integration gates.

## Generational GC remains an option

The comparison baseline is [.NET GC design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/garbage-collection.md):
thread allocation contexts and generational collection reduce common allocation and
young-collection work. This is architectural guidance, not a requirement to reproduce
CoreCLR internals. A nursery could address both the search loop and repeated tracing
of long-lived server state. The current experiment does not measure survival rates,
so it cannot predict a generational speedup.

This continues the [2026-10-13 generational evaluation](../../../../docs/runtime-gc.md#generational-collection-evaluation--2026-10-13).
The author's subsequent [concurrent-GC direction](../../../../docs/design/concurrent-gc-evaluation.md)
remains in force: the question about generations does not cancel that work or select
a new implementation order. Barrier/store auditing and allocation-locality work are
shared prerequisites; generations and concurrent tracing can be evaluated independently.
The [existing snapshot-collector experiment](../../serve/memory-gc-20261019/README.md)
already has bounded worker-tracing and owner-reclamation evidence. Its older cohort
favored bulk handling but lost tiny-response throughput; those timings cannot be
compared directly with this newer object/runtime. A fresh cohort should reuse its
safety oracle and measure snapshot copying, discarded cycles, synchronization and
collector-worker CPU separately. This investigation has not rerun that collector;
concurrent GC is not being rejected on the basis of the current synchronous profile.

Two viable directions deserve explicit comparison:

| Direction | Benefit | Cost / prerequisite |
| --- | --- | --- |
| Nonmoving young/old regions with remembered sets | Keeps native addresses stable; avoids scanning all old objects on minor collections | Fragmentation and promotion bookkeeping remain; still needs complete old-to-young tracking |
| Evacuating nursery with older nonmoving or compacting regions | Cheap bump allocation and densely packed survivors | Precise relocation of roots/fields, handles/interior references, pinning and native-boundary discipline |

Generational does not imply moving, concurrent, or a larger object header. Age and
remembered-set metadata could be held in side tables/regions; costs must be measured.
Current conservative candidates, borrowed/interior references and raw native holders
make movement a separate prerequisite, not something to assume away.

Proposed staged investigation (not implemented):

1. Add allocation-age/survival histograms, live/allocated bytes per collection,
   remembered-set candidate store rates, root/heap scan counts and pause distributions.
   Track native socket/resource retention separately from managed payload.
2. Inventory every reference store: fields, arrays, static storage, closure captures,
   native helpers, bulk copies and shared-worker writes. Introduce a checked barrier
   contract and test old objects receiving young references, cycles, overwritten
   references and repeated collections. Preserve ordering under worker coordination.
3. Prototype a bounded nonmoving young region first if it preserves current native
   contracts; compare it against the indexed non-generational allocator under the same
   memory limits. A copying nursery remains open pending precise/pinned-root support.
4. Validate pressure fallback, promotion failure, fault/cancellation retention, native
   handles, finalization/resource cleanup and cross-worker mutation/GC. Compare CPU,
   throughput, pause tails and peak resident memory on plaintext then mixed lifetimes.

The [Go GC guide](https://go.dev/doc/gc-guide) also separates allocation rate, live
memory, tracing cost and memory/CPU tradeoffs; its stated scope describes Go 1.19,
so it is not evidence about every current implementation detail. Generations are a
candidate, not a prerequisite for competitive HTTP performance.

## Implementation priorities and research basis

| Priority | Concrete change to investigate | Required evidence / risk |
| --- | --- | --- |
| 1 | Bounded free-space bins or address-aware free index in owned arenas | Search-step distribution, fragmentation, failed fits, zeroing, pressure retry, stale-base rejection and concurrent/shared-owner tests; preserve external arena validation |
| 2 | Native ThinLTO build option, then qualified target defaults | Exported ABI and native-kit/JIT/source-host tests, artifact/cache identity, build cost and longer paired runs |
| 3 | Compiler effect analysis for non-allocating/non-suspending leaf calls; bulk byte parsing/encoding | Preserve fault/static-init/native callback behavior and worker rendezvous; fragmented headers, CRLF/UTF-8 validation, limits, cancellation and forced GC |
| 4 | Typed immutable image-owned GC maps and cheaper thread-state access | Remove repeated length/marker parsing only behind explicit ownership/version contracts; retain mutable-plan rejection, unload/reuse safety and root metadata diagnostics |
| Parallel design | Generational allocation/collection | Survival and barrier audit above; do not confuse current whole-heap tracing with generations |
| Later workload scaling | Persistent connections, readiness queues, bounded completion batching | Protocol framing, fairness, backpressure, deadlines, shutdown; instrument waits and wakeups before scheduler replacement |

The [Go allocator](https://go.dev/src/runtime/malloc.go) uses size classes and local
span caches; [HotSpot TLAB allocation](https://github.com/openjdk/jdk/blob/master/src/hotspot/share/gc/shared/threadLocalAllocBuffer.inline.hpp)
uses a bounded top/end fast path. These support investigating cheap allocation paths,
not importing their collectors unchanged. A free-space index can improve the existing
nonmoving collector and remain useful for an older generation later.

[CoreCLR GC info](https://github.com/dotnet/runtime/blob/main/src/coreclr/inc/gcinfo.h)
uses versioned metadata and tracks its version at a method-range/image level rather
than repeating it per method. neoCLR currently selects publication points for ordinary
calls, constructors, arrays, static operations and text materialization; it does not
prove small getters non-collecting. The parser byte loop calls length/item/header
accessors repeatedly, and response encoding appends many individual bytes. Existing
[site attribution](../site-profile/README.md) assigns 29.7% of publications to three
parsing/encoding methods. That is a count, not an estimate of removable CPU cost.

[LLVM ThinLTO](https://clang.llvm.org/docs/ThinLTO.html) enables cross-module analysis
and importing, explaining why it is worth testing across C helper boundaries. It
cannot inline opaque generated machine-code bodies. Previous private TLS grouping
[failed to establish a reliable benefit](../gc-thread-state/README.md); do not repeat
that change on the strength of TLS samples alone.

[Kestrel IOQueue](https://github.com/dotnet/aspnetcore/blob/main/src/Servers/Kestrel/Transport.Sockets/src/Internal/IOQueue.cs)
schedules a drain on the thread pool and processes queued callbacks;
[SocketConnection](https://github.com/dotnet/aspnetcore/blob/main/src/Servers/Kestrel/Transport.Sockets/src/Internal/SocketConnection.cs)
uses pipes, pooled memory and separate send/receive logic. [Go netpoll](https://go.dev/src/runtime/netpoll.go)
coordinates readiness, parking, deadlines and close races. neoCLR already runs four
isolated owners; its bounded operation table is scanned and idle waiting uses poll
with a 1 ms cap for services without a common wakeup event. Replacing it with a proper
readiness backend may help idle/high-connection workloads, but this active plaintext
profile does not identify it as the leading cost. Shared-heap async execution and
channels remain separate safety work; they cannot be inferred from isolated-owner runs.

## Evidence and reproduction

`profile-build.json`, `capture.json`, `sample.txt.gz`, `sample-summary.json.gz` and
`analysis-validation.json` retain the fresh profile and byte-verified attribution
with unchanged parser-test evidence reused by source hash.
`screen.json.gz` / `screen-runner.py.gz` preserve the original screening version;
`confirmation.json.gz` records the longer follow-up. `diagnostic-*.json.gz` plus
`diagnostic-*.c.gz` retain exact diagnostic sources and counts. `validation.json.gz`
records counter/ThinLTO checks. All reports include commands and input identities.
The initial capture preparation failed to locate `-o` by assuming it was the last
argument; the corrected manifest uses its actual argument position. It never entered
a timed trial. Preliminary diagnostic reruns are superseded by the final source-matched
reports; no unfavorable capacity trial is discarded.

```sh
python3 benchmarks/http-server/primitive-comparison/cost-investigation/measure.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-owned-root/candidate.o \
  --client target/primitive-comparison-refresh/client \
  --variants baseline thinlto appendfirst --seconds 10 --output target/cost-confirmation
python3 benchmarks/http-server/primitive-comparison/cost-investigation/diagnose.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-owned-root/candidate.o \
  --client target/primitive-comparison-refresh/client \
  --mode timed --workers 4 --output target/cost-diagnostic
python3 benchmarks/http-server/primitive-comparison/cost-investigation/validate.py \
  --diagnostic target/cost-diagnostic/profile.json \
  --qualification target/context-reuse-default-consumers/qualification.json \
  --output target/cost-validation
```

Use the recorded Xcode SDK environment and new output directories. Optional
`--paced4` and `--append-first` select diagnostic candidates; capacity variants are
selected separately. Do not overlap diagnostics/builds with throughput trials.
No public API, Raven compiler bridge, installed package or website capability changes.
