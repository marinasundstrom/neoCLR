# Serve admission stalls: GC and scheduler correlation

Diagnostic evidence, 2026-10-18. Fixed guest and library from the callback-reservation
candidate; no runtime behavior change. This instrumentation is **not capacity evidence**.
Thread CPU clock sampling on every collection/acquire can itself cause saturation.

| 20-second run, 400 offered/sec | Valid / 8,000 | Misses | Offered p99 | Generator lag p99 |
| --- | ---: | ---: | ---: | ---: |
| Initial, shared wait hook only | 7,910 | 90 | 167.53 ms | 129.46 ms |
| Expanded, includes async pump pause | 5,611 | 2,389 | 365.31 ms | 3.93 ms |

Both runs and every arrival are retained. Different instrumentation and host conditions
prevent comparing throughput between these runs. Neither establishes a sustainable rate.

## Findings

The initial longest GC interval was 75.412 ms wall time but only 0.049 ms thread CPU.
It coincided with a generator dispatch gap and a subsequent miss cluster: attributing
75 ms of execution to the collector would be incorrect. Other misses remain outside
that cluster. Clock alignment shows correlation, not the cause of descheduling.

The initial shared scheduler-wait hook recorded zero calls because the async pump
uses its own pause path. The expanded capture fixes that coverage gap. It records
1,518 waits across startup, warmup and shutdown, but **only two waits totaling
2.977 ms inside the measurement window**. Persistent backlog in that run cannot be
explained by sleeping on each loop. The longest measured collection was 4.802 ms
wall / 0.069 ms CPU. Whole-process GC CPU totals were 4.878 s and 4.689 s respectively;
these include warmup/shutdown, and must not be divided by measurement completions.

The current native collector runs synchronously on the runtime owner thread. Its
mark/sweep walks block managed progress on that thread. See [GC architecture and
research](../../../../docs/runtime-gc.md). Moving it to a background thread requires
mutation barriers, root coordination and safe reclamation; a separate process would
also require a shared-memory/ownership design. No concurrent collector is implemented
by this diagnostic change. Avoidable Promise result-list/array allocations remain a
separately measured candidate for reducing GC work.

## Capture and validation

`capture.py --build BUILD --output NEW_DIRECTORY` relinks the existing guest object
with bounded wrappers, then runs the existing diagnostic request recorder. Defaults
are 20 seconds at 400/sec. Run `analyze.py NEW_DIRECTORY` afterwards. Use the repository
Xcode SDK environment required by the underlying build. This is a POSIX diagnostic.

Hooks have fixed storage for 131,072 intervals, no allocation/I/O during capture,
and explicit dropped-event accounting. All waits are retained; GC/acquire intervals
under 100 microseconds contribute to totals but not interval records. Acquire includes
collection, so categories must not be summed as exclusive CPU or wall time.
Unix/monotonic origins align independent processes; endpoint drift was -1 and -3
microseconds, below the 1 ms rejection bound. The 100 ms bins preserve every offered
arrival and miss exactly. Alignment is approximate, not an atomic clock snapshot.

`test_trace.py` checks wrapper forwarding, statuses, both wait paths and clock ranges.
The four load tests check diagnostic origins and unchanged request accounting.
Both pass. Raw trace/build/request data and generated native sources are retained
compressed alongside summaries. The initial manifest identifies the earlier hook
revision that omitted pump waits; the current capture script reproduces expanded
coverage. Ordinary uninstrumented runs do not acquire the diagnostic clock origins.
