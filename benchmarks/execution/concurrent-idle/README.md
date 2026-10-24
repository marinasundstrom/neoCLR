# Concurrent idle completion without synchronous recheck — 2026-10-24

This candidate finishes an in-flight cycle at idle without immediately tracing the
whole current graph again. It then begins the next capture. Explicit/managed and
pressure collections retain the synchronous recheck; context change, invocation
completion and heap release retain cancellation and shutdown behavior. The first
idle with no pending epoch still collects synchronously before beginning capture.
The synchronous default and previous candidate remain available for comparison.

## Semantics and .NET comparison

This deliberately changes internal cleanup timing: newly allocated or changed
unreachable objects can survive the current epoch and release their associated
native resource in a later cycle. There is no wall-clock finalization guarantee.
Explicit disposal and pressure recovery are unchanged. The benefit sought is less
repeated tracing; the costs are delayed cleanup and more retained/copied memory.

[.NET background GC](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/background-gc)
combines background generation-2 work with foreground ephemeral collections when
needed. This experimental nonmoving, non-generational owned-heap collector does
not reproduce that design: its synchronous pressure fallback preserves progress,
but copying and conservative retention still add work. [.NET collection fundamentals](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/fundamentals)
also distinguish heap reclamation from the application-visible root graph. This
experiment does not introduce a public promise about the exact collection cycle
in which an unreachable native resource is released.

## Qualification

Optimized and undefined/bounds-sanitized tests verify that a newly allocated
unreachable resource survives the first idle completion and is released at the
next. A 1,000-cycle allocation workload stabilizes at at most two batches (6,144
arena bytes); explicit collection then empties the heap. This bound describes the
fixture, not arbitrary continuously mutating graphs or total process memory.
Existing forced collector/seed tests pass using their original configuration.

Plain/JSON JIT/AOT short runs pass normal shutdown. The longer matched profiles
pass 1,024 responses per collector/sample/mode, 8,192 total. Candidate counters
confirm zero synchronous rechecks in these HTTP profiles and no epoch left active
at exit. Resource pressure, descriptor retirement and cancellation mechanisms are
unchanged from the previous integrated candidate. Cross-platform, shared-worker,
race-detector and broader managed workload qualification remain open.

## Costs

| Sample/mode | Sync outer µs/request | Candidate outer µs/request | Sync peak arena bytes | Candidate peak arena bytes |
| --- | ---: | ---: | ---: | ---: |
| Plain AOT | 20.51 | 49.88 | 83,064 | 105,232 |
| Plain JIT | 17.21 | 42.83 | 81,752 | 104,688 |
| JSON AOT | 42.55 | 74.07 | 92,768 | 101,288 |
| JSON JIT | 43.57 | 59.90 | 92,768 | 100,344 |

These sequential-client profiles include startup/shutdown and scheduling noise.
Arena extent excludes copied snapshots, queues, indexes and thread stacks; it is
not RSS. Compared with the previous cohort, absolute blocking totals are lower,
but that is not a matched before/after performance claim: the synchronous totals
also changed. Use the fresh synchronous pairs to assess this candidate.

Four one-owner AOT plaintext capacity trials use the same managed object, matched
profiled hosts, 32 connections, 2-second warmup and 10-second windows. Synchronous
throughput is 3,978–4,060 responses/s; candidate throughput is 3,758–3,866. CPU per
response is 236–237 µs synchronous and 260–266 µs candidate. Paired means show about
5.2% lower throughput and 11.4% more CPU/response for the candidate. Latency tails
are noisy and overlap. These local pairs are not confidence intervals or evidence
for changing the default.

The retained set currently includes **all** new/changed blocks, even if neither
the initial nor current roots reach them. That causes additional scanning and
floating garbage. The next candidate should retain the required closures without
blanket marking, preserving conservative interior-pointer interpretation, weak/
conditional exposure and references to newly allocated storage. That change needs
its own proof and differential tests; this report does not implement it.

## Reproduction

`generate.py --output <file>` layers this policy over concurrent-integration.
`validate.py --output <fresh-directory>` runs the focused tests. Reuse
`concurrent-integration/measure.py` for wire/lifecycle qualification and
`concurrent-pauses/instrument.py` plus `measure.py` for complete outer-operation
profiles. `concurrent-integration/capacity.py` accepts this generated collector via
`--candidate-source`, with the same objects and host recipes documented there.
Raw reports, generated sources, pause summaries and capacity pairs are retained.
