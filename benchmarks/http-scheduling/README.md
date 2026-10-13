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

## Boundary and collector phase attribution — 2026-10-13

Add `--gc-phases` to `profile.py` to build diagnostic copies of `root-probe.c` and
`native-gc.c`. Exact source markers are checked before insertion; source drift fails
rather than silently moving a measurement. No shipping runtime or collector policy
changes. The existing profile ABI remains unchanged.

The tool separates pre-collection root-boundary validation from five collector
phases: heap index/owner validation; initial root seeding; transitive tracing plus
conditional-root closure; owner/resource cleanup; sweeping/coalescing. Phase elapsed
sums exclude some wrapper overhead and must not be treated as CPU samples. Added
clock calls especially perturb short phases. Boundary time includes the observer's
whole-frame-chain reads; it excludes generated spilling and other enter/leave work.

| Diagnostic workload | Collections/process | No intervening allocation | Boundary validation | Root seeding | Transitive tracing | Sweep/coalescing |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Plain text, 32 sequential requests | 177,095–177,098 | 90.8% | 132–143 ms | 77–82 ms | 101–107 ms | 39–42 ms |
| JSON, 32 sequential requests | 271,943–271,945 | 85.9% | 273–298 ms | 153–163 ms | 195–207 ms | 79–82 ms |
| Runtime-suspension application, 16 concurrent requests plus cancellation/shutdown | 166,921–167,225 | 91.1–91.2% | 123–133 ms | 125–136 ms | 272–293 ms | 88–95 ms |

[Plain evidence](gc-phases-plain-20261013.json), [JSON evidence](gc-phases-json-20261013.json)
and [runtime evidence](gc-phases-runtime-20261013.json) cover eight successful
processes and 192 validated responses, with complete allocation reclamation and no
collector failures. They reuse the previously identified guest/library versions;
they are not a cross-lowering or throughput comparison. The
[diagnostic accounting contract](gc-phases-contract-20261013.json) runs the existing
collector-profile test with sanitizer instrumentation, including an invalid-context
collection: four attempts, one expected failure, three without a new allocation.

The current native boundary observer actually performs a full collection at each
root boundary. The no-allocation counter means no intervening successful managed
allocation on the thread; it does **not** prove unchanged roots, unchanged object
edges, no dead objects or safe omission of a collection. In particular dependent
owners, abandoned fibers and resource cleanup can change without new allocation.

The evidence supports testing collection pacing and reducing repeated root work
before selecting a new collector. A pressure-policy experiment must retain explicit
collection, bounded heap/failure behavior, stress mode, dependent-owner correctness,
and a defined quiescent/idle reclamation path. It must measure retained memory and
resource cleanup latency alongside speed. Allocation itself currently cannot
collect, so allocation-failure retry needs a verified published-root boundary; merely
skipping calls or inserting collection into allocation would not establish correctness.

## Generations and ZGC research — 2026-10-13

The author explicitly requested considering .NET's generations and Java's ZGC.
.NET's small-object heap uses generations 0, 1 and 2, allowing young collections
without revisiting the entire older heap ([Microsoft fundamentals](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/fundamentals)).
Oracle's [Generational ZGC explanation](https://inside.java/2023/11/28/gen-zgc-explainer/)
describes the JDK 21 addition of young/old generations to its largely concurrent
collector. ZGC targets low pauses, not zero GC work; concurrency consumes resources
that would otherwise be available to the application. These are compatible design
axes: generational does not mean stop-the-world, and concurrent does not mean
non-generational. Neither requires neoCLR to copy a specific generation count.

For neoCLR, first measure allocation lifetimes/survival, old-to-young writes and
root scanning under realistic long-lived HTTP servers. A generational nonmoving
collector is one candidate; moving young objects is another. Remembered sets/barriers
must cover interpreter, JIT, AOT and host writes before collecting only young objects.
Moving objects additionally require precise relocation-capable references and a
native pinning/interior-reference contract; the current conservative object-word
scan cannot simply become a relocating scan. Concurrent collection further requires
barrier and safepoint coordination with parked stacks and host services.

This is a research comparison, not a chosen architecture or an implementation of
generations. Sources accessed 2026-10-13. The Oracle explanation is pinned to JDK 21;
no claim about the latest JVM defaults is inferred from it. Direct JEP retrieval was
unavailable during this review, so release-history claims beyond that source are
not used. The next bounded experiment remains allocation-pressure pacing with
correctness gates, while gathering the survival/barrier evidence needed to choose
whether generations justify their complexity.

## Collection-pacing follow-up

The [isolated pacing experiment](../http-gc-pacing/README.md) measures large HTTP
improvements but rejects its candidate on a reproducible large-allocation admission
counterexample. It identifies the next correctness contract; it does not change the
shipping collector or establish the value of generational collection.

## Windows wait attribution

The Windows JIT workflow's `benchmark_roots` scope now follows uninstrumented
measurements with two diagnostic AOT runs each for plain text and JSON.
`windows-profile.py` consumes the passing native build record and relinks identical
guest objects with private clock hooks. `windows-counters.c` uses QPC wall intervals
and process CPU counters; it records socket polling, host scheduler pauses and GC.
These are whole-worker counters including startup, warmup and shutdown. Poll time
can include nested runtime work; categories must not be blindly summed. Response
validation, poll accounting, zero collector failures and final reclamation are
required. Results will determine whether collector work or waiting warrants the
next change. This extends existing diagnostic tooling, not the public profiler ABI
or managed Scheduler contract; .NET/CLR comparison and collector research above
remain applicable. No production scheduling policy changes in this slice.

### Windows readiness-wait candidate

The host can now wait on pending accept/read/write/connect descriptors before
polling again, instead of sleeping regardless of readiness. This is a private
owner-thread integration hook used by JIT and AOT; it never consumes a completion,
changes roots or invokes guest code. Existing arbitration and fault delivery stay
in the poller. The requested timeout is capped at 1 ms and shortened for pending
deadlines. DNS-only, timer-only and legacy native TLS waits retain the old pause;
these sources still lack a shared readiness event. OS scheduling can overrun either
requested timeout. macOS retains its prior wait policy in this slice.

This is a bounded optimization of the existing 64-operation adapter, not a scalable
reactor replacement. Unlike .NET's Windows IOCP implementation, it uses Winsock
`select`; eventual completion-port integration remains open. Benefit to validate:
socket arrival can wake the owner immediately. Costs: rebuilding descriptor sets,
provider restrictions and continued polling for other sources. It does not change
managed async semantics. Primary sources consulted 2026-10-13:
[Winsock select](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-select),
[Sleep](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-sleep),
and [.NET Socket source](https://source.dot.net/System.Net.Sockets/System/Net/Sockets/Socket.cs.html).

`benchmark_waits=true` selects five alternating uninstrumented JIT/AOT rounds using
identical sources/guests, differing only by the private diagnostic build switch
`NEOCLR_SCHEDULER_LEGACY_WAIT`. Separate instrumented AOT runs qualify both policies,
including final reclamation. The old policy is a benchmark control, not an SDK option.
Results determine acceptance; no gain is assumed from the implementation alone.

### Matched Windows results — 2026-10-13

[Run 38410199440](https://github.com/marinasundstrom/neoCLR/actions/runs/38410199440)
passes at `58279fe05` on Windows Server 2022 x64, GitHub-hosted AMD virtual machine.
[Raw uninstrumented report](windows-waits-20261013.json) retains all five alternating
rounds, eight warmup processes, input hashes and build commands. Same guest objects
and source bytes within each pair; only the legacy-wait compile switch differs.
Each process validates 32 sequential loopback responses on fresh connections and
excludes the first eight from timing. All 1,536 responses pass, including 960 timed
requests. Startup/JIT compilation is outside request timings.

| Workload | Mode | Sleep requests/s (range) | Readiness requests/s (range) | Median ratio |
| --- | --- | ---: | ---: | ---: |
| Plain text | JIT | 40.10 (37.07–42.30) | 89.30 (82.63–93.46) | 2.23× |
| Plain text | AOT | 41.77 (37.47–47.44) | 87.19 (58.55–91.94) | 2.09× |
| Object JSON | JIT | 29.95 (26.94–33.21) | 47.36 (45.84–49.51) | 1.58× |
| Object JSON | AOT | 28.45 (25.12–33.21) | 46.73 (46.25–47.74) | 1.64× |

Ranges do not overlap in these runs. The plain AOT candidate has a slower round,
which remains in the report. This is a short, concurrency-one CI comparison, not
sustained throughput or a cross-runtime/platform ranking. In particular, compare
within this run, not with the slower prior Windows VM's absolute results.

Separate AOT diagnostics, two runs per policy/workload, validate another 256
responses. All 16,340 plain / 38,358 JSON allocations per process are reclaimed;
all collector and poll error counts are zero.

| Whole-process elapsed interval | Plain sleep | Plain readiness | JSON sleep | JSON readiness |
| --- | ---: | ---: | ---: | ---: |
| Scheduler waits | 424–443 ms | 24–27 ms | 386–489 ms | 51–63 ms |
| Collector | 191–196 ms | 195–198 ms | 359–361 ms | 355–356 ms |
| Socket polling | 1.95–2.13 ms | 1.66–1.77 ms | 2.11–2.20 ms | 1.79–1.82 ms |
| Worker wall time | 776–810 ms | 386–387 ms | 1,023–1,118 ms | 675–678 ms |

Raw profiles: [plain sleep](windows-plain-sleep-20261013.json),
[plain readiness](windows-plain-readiness-20261013.json),
[JSON sleep](windows-json-sleep-20261013.json),
[JSON readiness](windows-json-readiness-20261013.json).
These clocks measure elapsed intervals, not an exclusive CPU breakdown. Polling
may nest runtime work; process CPU accounting on this VM is coarse and is retained
as diagnostic data rather than used to infer utilization. Instrumented variants
run separately, not in alternating throughput rounds. The uninstrumented experiment
is the speedup evidence.

The result identifies unconditional sleeping as the dominant avoidable waiting cost
in these Windows cases. Collection time and allocation volume are essentially
unchanged while waits fall sharply. Collection remains the largest measured work
interval after this fix; complete helper-root coverage remains prerequisite to
safer pressure-based collection. Socket-table scanning is small here. DNS/TLS
readiness integration, idle efficiency and concurrent saturation still need their
own measurements; this result does not select a new collector or scheduler design.

Correctness evidence: [Windows socket contracts](windows-wait-socket-validation-20261013.json),
[Windows GC/root contracts](windows-wait-gc-validation-20261013.json), both from
[run 38410204548](https://github.com/marinasundstrom/neoCLR/actions/runs/38410204548),
which also passes macOS socket checks. The [macOS profiler regression](macos-wait-hook-profile-20261013.json)
passes 64 responses with complete reclamation; its wait behavior is unchanged.
The private Windows wait does not invoke callbacks, consume results or move roots;
the accept contract verifies that published callback ownership survives waiting
and an intervening collection.

The earlier [diagnostic run 38409478850](https://github.com/marinasundstrom/neoCLR/actions/runs/38409478850)
also passes at `10727135f`: [plain](windows-initial-plain-profile-38409478850.json)
and [JSON](windows-initial-json-profile-38409478850.json) show 684–791 / 676–748 ms
in sleeps versus 349–350 / 650 ms in collection. It ran on a different hosted VM;
do not subtract its intervals from the candidate run. Its accompanying
[root-observation repeat](windows-root-repeat-38409478850.json) is retained, including
smaller median changes and slower samples. This repeat reinforces the limitation
of short hosted measurements and is not the readiness-wait comparison.

The focused [HTTP/SSL gate 38410209858](https://github.com/marinasundstrom/neoCLR/actions/runs/38410209858)
also passes at `58279fe05`: [eight default-JIT HTTP cases](windows-wait-http-20261013.json)
include fragmentation, duplicate lengths, handler error/fault, DNS, timeout cleanup
and delayed success; [managed SSL streams](windows-wait-tls-20261013.json) exchange
nested JSON with GC in interpreter/JIT/AOT. The [gate report](windows-wait-http-gate-20261013.json)
is explicitly focused, not a new full-library-suite result.
