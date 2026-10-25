# Concurrent-specific tuning and scratch reuse

2026-10-25. Author direction: tune concurrent GC against its own baseline; sync is
an informative reference, not a requirement for identical CPU, memory or pacing.
This refines the earlier concurrent-over-sync wrap-up target. No default changes.

## Where the handoff occurs

The owner captures roots and detached headers, signals a persistent marker through
its condition variable, and continues managed execution. The marker traces live
payloads through the mutation gate. Finish waits for completion, drains mutation
logs, remarks current roots and may redispatch marking. The owner still validates
identities, traces new objects, handles resources, sweeps/coalesces, and publishes
the free list. Completion of a job is not termination of its persistent thread.

The [phase evidence](../concurrent-workloads/stable-byref-pause-results.md) puts
roughly 93% of JSON finish CPU in final collection, not marker waiting. That includes
seeding, roots and index work; it is **not** 93% sweep. Merely yielding or adding a
thread cannot remove those costs. Go's background/allocation-driven span sweeping,
.NET's allocation budgets and G1's adaptive starts are useful
[primary-source comparisons](../concurrent-workloads/industry-gc-comparison.md).
A future bounded-sweep prototype must separate mark completion, cleanup ownership,
unswept spans and allocation/free-list publication before overlapping reclamation.
This is proposed work, not implemented concurrent sweeping.

## Independently tune the concurrent policy

The HTTP and ordinary drivers accept `--live-budgets A:P ...`, producing interleaved
concurrent variants while leaving sync and the concurrent baseline unchanged.
HTTP also accepts `--live-heap-bytes`; existing `--gate-words` selects log capacity.
Explicit command lines, hashes, policy maps and checked lifetime counters are saved.
The existing allocation/publication thresholds still toggle begin/finish; this is
not yet an adaptive controller or independently paced sweep.

Four clients, three rounds, 2,048 requests plus 16 warmups per run; fixed workload
and generated objects. 74,304 HTTP responses pass across two six-mode cohorts.
Heap capacity is 1 MiB and log capacity 4,096 words in this first sweep. Values are
medians; CPU is whole-server process CPU including worker, startup and warmup.

| Concurrent A/P budgets | Plain requests/s | Plain CPU s | JSON requests/s | JSON CPU s |
| --- | ---: | ---: | ---: | ---: |
| 256 / 4096 baseline | 3,623 | .6054 | 2,324 | .9351 |
| 256 / 16384 | 3,656 | .5991 | 2,328 | .9359 |
| 1024 / 4096 | 3,669 | .5976 | 2,491 | .8778 |
| 1024 / 16384 | 4,079 | .5161 | 2,728 | .7749 |
| 4096 / 65536 | 4,424 | .4766 | 2,889 | .7145 |

Raising just one threshold leaves the other trigger active. The moderate joint
setting improves plaintext throughput **12.6%**, CPU **14.7%**; JSON throughput
**17.4%**, CPU **17.1%**. Arena peaks rise from about 147 to 472 thousand bytes for plain,
and 145 to 354 thousand bytes for
JSON. These are arena occupancy peaks, not allocation volume or process RSS.
Plain managed allocation counts/bytes remain identical; JSON varies slightly with
scheduling. See raw reports rather than claiming zero allocation differences.
Sync reference medians are 5,077/3,080 requests/s; superiority is not established
or required to recognize concurrent tuning gains.

**Largest budgets need a coupled log policy.** At 4096/65536 the 1 MiB arena fills,
and median completed/canceled epochs are 19/97 plain and 27/172 JSON. All but one
cancellation per run matches the store cancellation counter, with zero native-scope
cancellations. These are safe fallbacks, not successful concurrent marking cycles.

A further 37,152 checked responses screen heap/log combinations sequentially at
4096/65536; these are sensitivity results, not fully interleaved estimates:

| Heap / log words | Plain rps / CPU s | JSON rps / CPU s | Plain / JSON store cancellations |
| --- | --- | --- | --- |
| 1 MiB / 4096 (prior cohort) | 4,424 / .4766 | 2,889 / .7145 | 96 / 171 |
| 1 MiB / 16384 | 4,288 / .4880 | 2,786 / .7417 | 0 / 0 |
| 4 MiB / 4096 | 4,202 / .5012 | 2,612 / .7976 | 103 / 205 |
| 4 MiB / 16384 | 4,203 / .4942 | 2,707 / .7603 | 0 / 0 |

With the larger log, completed cycles rise to 89/152 at 1 MiB and 80/142 at 4 MiB.
Only one shutdown cancellation remains. Larger heap alone worsens this screen;
larger log costs additional native storage but prevents abandoned marker work.
Do not promote the fastest cancellation-heavy setting as a successful concurrency
policy. Measure queue occupancy/overflow and memory headroom together next.

120 standalone AOT runs (tasks, collections, arrays; ten rounds per mode) also pass.
At 1024/16384, concurrent CPU falls 1.8%, 7.4%, 2.8% respectively. At 4096/65536 it
falls 2.9%, 9.0%, 4.0%, with substantially larger arena peaks. The ordinary driver
still uses its existing arena capacity. The different sensitivities argue for a
measured workload policy rather than globally multiplying constants.

## Bounded scratch-storage experiment: no demonstrated HTTP benefit

Opt-in `NEOCLR_LIVE_REUSE_SCRATCH` retains six owner-local array buffers: detached
headers, offsets, worklist, mark bits, expected bits and sparse index. Total cached
capacity is capped at 1 MiB; oversized jobs use disposable storage. Every acquisition
zeros its requested extent. Completion/detachment precedes release; active leases
reject reuse/shutdown, and worker shutdown releases all cached buffers. Gates,
snapshot objects, edges and managed payload lifetimes are unchanged. Reuse avoids
repeated allocation, not header copying or required bitmap reset.

Two initial five-round HTTP cohorts pass 123,360 responses but are flat/slightly
worse: plaintext CPU +0.1%, JSON +0.3%. After strengthening spill lease checks,
37,008 additional plaintext responses show CPU +1.5%, throughput -1.2%. Ninety
standalone runs are approximately flat (arrays CPU -0.8%). Keep this an opt-in
experiment; **do not adopt it as a performance improvement**. Actual workload native
allocation volume and retained scratch peaks are not yet instrumented. The unit
fixture proves 100 repeated acquisitions use one backing allocation, not a general
workload allocation-saving magnitude.

Optimized and UB/bounds live fixtures pass deletion/insertion, delayed mutation,
future addresses, overflow recovery, record/object/array scanning, and 12,000
randomized differential blocks per lifetime run. Deliberately missing deletion logs
and invalid cache proofs fail. Scratch tests cover reset, growth, aggregate cap,
spill, empty allocation, release and rejection of duplicate/active leases.
AddressSanitizer stalled before main during shadow-memory initialization; the stack
is archived, the process was terminated, and ASan is unqualified. UB/bounds passes.
No new JIT or shared-mutator qualification is claimed for the optional scratch path.

## Reproduction and limits

Use `concurrent-store/measure-plaintext.py` with the retained sync/live objects and
`--compare-allocation-index --compare-exact-allocation-index --compare-inline-base
--live-budgets 256:16384 1024:4096 1024:16384 4096:65536`. Select the desired modes
with `--modes`; exact commands and generated-source hashes are archived in gzip JSON.
`concurrent-workloads/measure.py` supports the same budget pairs. `--compare-scratch`
adds a control/candidate mode without changing ordinary defaults.
Run `concurrent-live/test-scratch.py` and `concurrent-live/validate.py --reuse-scratch
--sparse-index --allocation-index --exact-allocation-index --inline-base --output NEW`.
`check-evidence.py` verifies final HTTP build defines and lifecycle accounting.
It correctly rejects the first headroom report's duplicate equal heap define; a
subsequent different-value build failed under `-Werror`, prompting replacement of
the inherited define. Failed build evidence is retained. Five later headroom cohorts
pass the exact-one-define check. The initial equal defines did select the stated
1 MiB capacity; that measurement is retained as historical evidence.

No cross-framework capacity, steady-state latency or concurrent-default claim.
Managed allocation pressure, barrier failures, explicit collection, native ownership
and shutdown rules remain intact. Website/public API descriptions require no change:
these are private experimental knobs, not new supported runtime APIs.
