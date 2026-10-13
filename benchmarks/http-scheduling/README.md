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
