# Concurrent remark restricted to reachable closures — 2026-10-24

This experiment layers on concurrent-idle and stops marking every new or changed
allocation at remark. It retains the initial live graph, rescans changed members
of that graph, then traces current roots and conditional edges through the current
heap. New or initially unreachable objects survive only when reached through these
closures or conservative candidates. Original allocation identity validation,
pressure/explicit synchronous recovery and cancellation remain in place.
Production GC is unchanged. This is still full immutable copying and owner-thread
comparison/reclamation, not a live-heap concurrent collector.

## Conservative pointer safety

An unchanged reachable payload can contain a numeric word naming a free hole or
storage beyond the old used extent. If allocation makes that address a valid
payload before remark, skipping the unchanged payload must not lose the reference.
The marker therefore saves unresolved in-arena words from reachable snapshot
payloads and conditional processing. After joining the marker, remark resolves
them through the current heap index. Headers/padding remain invalid unless they
have become a valid current payload. The marker never dereferences live storage.

This adds a temporary candidate vector and may retain conservative false positives.
Entries are not deduplicated; repeated conditional passes can increase storage.
Allocation failure still aborts in this prototype. Initial live objects can remain
floating garbage; this does not implement precise current-only reachability.

## Design comparison and author guidance

[.NET background GC](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/background-gc)
combines background generation-2 collection with foreground ephemeral collections.
This prototype has no generations and does not reproduce that implementation.
[Go's GC guide](https://go.dev/doc/gc-guide) provides another nonmoving tracing
reference and emphasizes CPU/memory tradeoffs. Neither establishes a performance
benefit for this implementation. The author's external ChatGPT summary is recorded
in the development timeline as input, not a mandated progression through parallel
GC before concurrent marking. A safe mutation/root protocol, suspended-task state
and shared-worker qualification remain essential work for a live-heap design.

## Validation

Optimized and undefined/bounds-sanitized builds exercise 16 future-address cases,
100 delayed mutation/closure cycles, conditional-to-strong exposure, and 20,000
block checks against an independently computed randomized graph closure. Original
collector, scan, dense and seed fixtures run through explicit snapshot begin/finish,
including conditional fixed points and 142,272 scanner configurations per build.
A negative control omitting saved future addresses aborts at the expected liveness
check. A paced idle fixture checks immediate cleanup of a newly unreachable resource
and 1,000 churn cycles bounded by one 3,072-byte batch, followed by explicit cleanup.
These bounds describe the fixture, not arbitrary workload memory.

Matched plain/JSON JIT/AOT profiles check 8,192 responses, complete shutdown, no GC
failures and no pending epoch. Candidate runs complete 4,130 concurrent epochs;
these HTTP profiles require no extra synchronous recheck. The following are outer
collector-operation times, including waits/capture; not end-to-end request latency.

| Sample/mode | Sync µs/request | Candidate µs/request | Sync peak arena | Candidate peak arena |
| --- | ---: | ---: | ---: | ---: |
| Plain AOT | 21.23 | 44.97 | 86,192 | 95,520 |
| Plain JIT | 17.76 | 34.54 | 81,752 | 90,928 |
| JSON AOT | 44.26 | 59.35 | 92,808 | 92,728 |
| JSON JIT | 41.40 | 57.01 | 92,768 | 92,904 |

Arena extent excludes snapshots, indexes, queues and thread stacks; it is not RSS.
The preceding idle candidate retained more arena memory, but its older measurements
are not a matched performance comparison. Use the fresh synchronous pairs here.

Four one-owner AOT capacity trials use the same managed object and profiled hosts,
32 connections, a two-second warmup and ten-second measurement windows in reversed
order. Synchronous throughput is 4,221–4,232 responses/s; candidate throughput is
4,087–4,127. Mean throughput falls 2.8% and process CPU/response rises 7.5%
(229.1 to 246.3 µs). CPU includes marker work, warmup and shutdown and is divided
by all completed responses. p99 ranges overlap (sync 9.10–10.81 ms, candidate
8.96–12.35 ms). Two observations per variant are not confidence intervals.
The reused capacity runner's generic scope mentions rechecks; this candidate's
idle path omits them, as the separate lifecycle profiles establish.

## Reproduction and limits

Run `generate.py --output <file>` and `validate.py --sparse-index --output <fresh>`.
Reuse concurrent-pauses instrument/measure/summarize for profiles and
concurrent-integration/capacity.py with this generated source and the existing
plain managed object. Reports retain exact commands, hashes and raw observations.

The synchronous default remains. Race-detector qualification is still blocked by
the previously reproduced sanitizer startup problem; UBSan does not replace it.
Shared-worker builds are rejected. Windows, broader managed standalone workloads,
multiple owners, full process memory and peer comparisons remain unqualified.
The next architectural task is to avoid full copying/comparison through an explicit
safe mutation/read protocol, rather than assuming a collector thread makes this
prototype a throughput improvement.
