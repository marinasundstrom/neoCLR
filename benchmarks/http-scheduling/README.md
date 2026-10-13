# HTTP scheduling diagnosis

Development evidence, 2026-10-13, macOS ARM64. These instrumented runs diagnose
where work occurs; they are not published throughput results or a scheduler speedup.

The retained tool relinks qualified guest objects with diagnostic copies of the
socket adapter and entry host. It counts socket polls, ready/empty/error outcomes,
consecutive empty polls and host sleeps, alongside process CPU and existing GC
counters. Production runtime sources and guest objects are unchanged. Exact commands,
input hashes and original build manifests identify each executable. Timings include
instrumentation and desktop scheduling noise; poll/GC/sleep clocks measure elapsed
time, whereas process user/system counters measure CPU. Do not divide these into
an exact CPU breakdown or sum overlapping scopes.

## Findings

| AOT workload | Runs | Socket polling per process | GC elapsed per process | Validation |
| --- | ---: | ---: | ---: | --- |
| Concurrent plain text, no intentional idle | 2 | 1.23–1.37 ms | 577–596 ms | 16 responses/run plus cancellation/shutdown |
| Concurrent plain text, four 100 ms idle holds | 2 | 2.24–2.35 ms | 613–628 ms | Same assertions; fast peers finish before releasing slow peer |
| Sequential plain text | 2 | 1.38–2.07 ms | 262–397 ms | 32 complete responses/run |
| Sequential JSON | 2 | 1.41–1.81 ms | 493–573 ms | 32 complete responses/run, decoded JSON checked |

[Concurrent results](concurrent-20261013.json), [plain results](plain-20261013.json),
and [JSON results](json-20261013.json) retain every sample. All 192 responses pass;
all eight runs have zero scheduler errors and zero GC failures, with every allocation
reclaimed. This establishes correctness for these runs, not all collector interleavings.
The concurrent test uses the incoming-stream slice's buffered control guest; the
sequential tests reuse the previous concurrent-admission library builds. They are
separate diagnostics, not a matched comparison between those library versions.

During four intentional idle holds, empty polls rise from 6–9 to 272–275. The host
sleeps 270–273 times for an elapsed 339–344 ms. It therefore does not tight-spin in
this case. Both `benchmarks/native-web/http-host.c` and the separate JIT entry host
use the platform's 1 ms pause on pending work. A readiness-driven wait is a sensible
latency/idle-efficiency candidate, but these counts do not measure OS readiness time,
wakeup latency or prove its effect on throughput. Timers, cancellation, DNS completion
and TLS readiness must participate in any replacement; reducing a sleep interval is
not a substitute for a correctly signalled wait.

The measured socket-table scans are small relative to instrumented collector work.
This does not measure all scheduling costs: managed continuation dispatch, root
publication, retained stacks and GC barriers occur outside the socket-poll wrapper.
Consequently the evidence does not justify changing callback batching or fairness.
The first investigation target is the cost of those execution/GC boundaries, with
separate readiness-to-resume measurement. Preserve root ownership and cancellation
correctness when optimizing either path.

## Execution-model limit

These retained guest builds use compiler-generated state machines; their link commands
do not include the runtime-async stack carrier. They are **not** a runtime-suspension
versus state-machine comparison. The VM scheduler separately has a 10 ms fallback
because socket readiness does not signal its wake latch (`src/scheduler.rs` and
`src/invocation.rs`). Neither that path nor JIT is instrumented by this first tool.

The next controlled matrix needs equivalent plain/JSON concurrent consumers, matched
runtime-async and state-machine builds, completed/pending awaits, slow peers, CPU,
allocation and latency observations. Qualify each mode's actual lowering and host
before attributing differences to suspension. Compare with the retained .NET/Go/Node
workloads only after equivalent protocol and concurrency conditions are established.
No runtime scheduler policy or public API changes in this slice.

## Reproduce

Use an existing successful `scripts/build-native-project.py --profile http` build
with the matching guest for each workload. The profiler reuses its link command and
checks input hashes again after execution. Output directories must be new.

```sh
python3 benchmarks/http-scheduling/profile.py --build CONCURRENT_BUILD/build.json --output NEW_DIRECTORY
python3 benchmarks/http-scheduling/profile.py --build PLAIN_BUILD/build.json --workload plain --output NEW_DIRECTORY
python3 benchmarks/http-scheduling/profile.py --build JSON_BUILD/build.json --workload json --output NEW_DIRECTORY
```

The concurrent controller's optional delay defaults to zero, preserving its ordinary
validation behavior. The profiler explicitly adds 100 ms only after the three fast
peers have completed and before releasing the fourth request. Timing it does not
establish starvation bounds or fairness under sustained load. No website publication
is performed; the publication hold remains in force.

## Matched application lowering — 2026-10-13

`compare-lowerings.py --recipe PASSING_HTTP_BUILD_JSON --bundle BUNDLE --output NEW_DIR`
now builds the same concurrent plain-text consumer twice against identical libraries.
It verifies backend runtime-async bindings are present only for the opted-in application.
Both hosts link the same retained-stack carrier; the state-machine guest never uses it.
Only application bodies change lowering: this is not a runtime-async rebuild of System.Web.

[Qualification](lowerings-20261013.json) passes four 16-response processes including
cancellation/shutdown and slow-peer holds. [State-machine counters](lowering-state-profile-20261013.json)
and [runtime counters](lowering-runtime-profile-20261013.json) add four instrumented
processes per lowering, all passing with full reclamation and zero collector errors.
These are grouped diagnostic runs, not alternating throughput samples. Historical
reports retain the script hashes used then; current profiler revisions add fiber counters.

Runtime suspension creates 93–95 drain fibers with no intentional idle and 358–359
with four 100 ms holds, whereas state-machine runs create none. The native pump
currently allocates a drain fiber for every loop iteration even when the preceding
iteration did not execute guest code. Skipping these redundant drains is a focused
optimization candidate. It must still drain after callbacks and resumed bodies, and
preserve suspended drain fibers, roots, faults and cancellation. JIT and interpreter
comparisons and direct readiness timestamps remain open.

## Collector research direction — 2026-10-13

The author proposed investigating other collectors alongside scheduling. This is
research guidance, not a decision to replace neoCLR's collector. The current native
probe deliberately collects at root boundaries; its stress policy must not be
mistaken for an unavoidable cost of nonmoving collection.

| Reference | What to investigate for neoCLR | Cost or prerequisite to validate |
| --- | --- | --- |
| [.NET/CoreCLR collector design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/garbage-collection.md) | Allocation contexts, generational collection and collecting short-lived objects without repeatedly tracing the whole live graph | Generations require remembered references; movement requires updating roots/interior references and native interop rules |
| [Go GC guide](https://go.dev/doc/gc-guide) | Allocation-driven pacing, heap/CPU tradeoffs and nonmoving mark/sweep as an alternative to movement | Concurrent tracing requires barriers and coordination with suspended stacks; memory headroom and assist costs need measurement |
| [V8 collector design](https://chromium.googlesource.com/v8/v8/+/main/docs/heap/garbage-collection.md) | Young-generation treatment, incremental/concurrent marking and remembered-set discipline | Barrier cost, moving-object support and platform/compiler integration cannot be assumed free |

Sources accessed 2026-10-13. These are architecture references, not benchmark results
for current releases; the Go guide explicitly identifies its described baseline as
Go 1.19. The applicability and costs above are neoCLR investigation points, not
claims that adopting a particular design will improve this runtime.

Start by separating boundary checks from actual collection cycles and measuring
root publication, root/heap scanning, sweeping and allocation separately. Then test
an allocation-pressure policy while retaining explicit/stress collection and tight
memory-limit behavior. Compare CPU, pause distributions, allocation rate, live heap,
peak reserved memory and request latency together. Only after that evidence should
we select between optimizing the existing nonmoving design, adding generations or
adding concurrent phases. Parked stacks, dependent owners, cancellation, JIT/AOT root
maps and future native interop are correctness constraints throughout.

## Idle drain correction — 2026-10-13

The macOS retained-stack pump now drains initially and after resumed bodies or
scheduler callbacks can produce managed work. Empty polls alone do not create a new
drain fiber. An existing suspended drain remains owned and resumes normally. This
relies on the carrier's existing single-owner-thread contract; future cross-thread
managed queue producers must signal ready work rather than rely on periodic draining.
No scheduler delivery, root ownership, timer or cancellation API changed.

[Candidate diagnostics](idle-drain-profile-20261013.json) reduce drain fibers from
358–359 to 90 with four 100 ms holds, and from 93–95 to 89 without holds. All four
16-response processes pass; each fully reclaims its 14,760 or 14,823 allocations.
The reduction is native fiber setup work, not fewer managed allocations.

[Focused sanitizer checks](idle-drain-gc-validation-20261013.json) pass collector,
root ownership, 320 retained-body lifetimes and the new idle-drain regression. That
regression exercises pending polls, callback-generated work and a drain that itself
suspends. It [fails against the old carrier](idle-drain-baseline-regression-20261013.json).
Reproduce with `validate_conditional_roots.py --owned-bodies` and its existing recipe.

[Uninstrumented comparison](idle-drain-comparison-20261013.json) relinks identical
runtime-async guest objects with the old and new carrier. Four alternating rounds
follow warmups for each hold/variant combination: 320 responses across 20 processes
all validate. No builds or profiler runs were active during these measured rounds;
this remains an uncontrolled desktop-host experiment.

| Case | Baseline median process time (range) | Candidate median process time (range) |
| --- | --- | --- |
| No intentional idle | 724 ms (712–760) | 727 ms (709–747) |
| Four 100 ms holds | 1,114 ms (1,105–1,116) | 1,131 ms (1,114–1,184) |

The ranges overlap and candidate medians are slightly higher. This establishes no
HTTP speedup and does not exclude a small regression. The stable result is removal
of redundant native fiber creation; broader throughput and tail-latency qualification
remain open. Whole-process CPU medians are 711/715 ms without holds and 769/765 ms
with holds, baseline/candidate respectively. Do not turn these into request throughput.

Reproduce the comparison with a saved baseline carrier source and a passing matched
runtime build from `compare-lowerings.py`:

```sh
python3 benchmarks/http-scheduling/compare-carrier.py --build RUNTIME_BUILD/build.json --baseline OLD_NATIVE_RUNTIME_C --output NEW_DIRECTORY
```
