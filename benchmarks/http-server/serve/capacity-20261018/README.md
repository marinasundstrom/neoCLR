# Serve sustained-rate investigation — 2026-10-18

Author direction: continue profiling Serve, explain admission misses and latency
variation, establish a reliable tested rate, and commit each focused slice. This
continues the [matched Serve implementation](../README.md), not a new HTTP contract
or change to the default stress collector. The .NET-style task/callback behavior
is unchanged; this work concerns measurement and implementation costs.

## First diagnostic slice

Same retained final Serve executable, experimental paced GC, 16 server lanes,
Apple M1/macOS loopback, ten-byte response, connection close. Four sequential
20-second runs with opt-in client phase timestamps:

| Offered/sec | Client slots | Success/offered | Offered p99 ms | Dispatch lag p99 ms |
| ---: | ---: | ---: | ---: | ---: |
| 150 | 16 | 3000/3000 | 12.81 | 1.87 |
| 300 | 16 | 5993/6000 | 19.47 | 0.96 |
| 150 | 64 | 3000/3000 | 200.79 | 68.01 |
| 300 | 64 | 5779/6000 | 426.61 | 66.90 |

All failures here are generator in-flight admission misses; all admitted responses
validate and server errors remain zero. The seven misses in the 16-slot 300/sec run
occur in second 16 (generator lag maximum 47 ms). The 64-slot 300/sec run loses
89/104/28 arrivals in seconds 4/5/6. Increasing the cap permits more queued work
and does not resolve pauses. These sequential runs do not isolate cap causality:
host conditions vary, and generator dispatch itself is delayed.

Response-wait p99 is 17.66 ms for 300/sec with 16 slots and 423.96 ms with 64 slots.
Connection p99 is only 0.99/4.04 ms, respectively, although individual connection
and client-close outliers also occur. Response wait includes server work, network
queues and delayed client observation. It is not proof of a particular server phase.
CPU/completion remains 2.58–2.87 ms across these trials, despite latency variation.

[Summary and full compressed report](diagnostic-sweep.json) retain all 18,000
arrivals and executable/build/input hashes. [Per-second/phase analysis](diagnostic-analysis.json)
retains the bursts and slowest requests. Four harness contract tests pass, including
an independent delayed peer that verifies phase ordering and response-wait attribution.
No reliable sustained rate follows from this short sweep. Native sampling and an
independent trivial peer are the next attribution checks; longer unprofiled runs
must retain all failed trials rather than selecting the fastest sample.

## Tools

- `load/run.py --diagnostics`: optional client phase timings; normal fixed offered
  schedule and loss denominator remain unchanged.
- `load/analyze.py`: per-second outcomes and diagnostic phase distributions.
- `serve/diagnose.py --executable APP --sample`: 5-second macOS native sample during
  a separate offered-load run; instrumented results are not capacity measurements.
- `serve/diagnose.py` without an executable: starts `generator-control.py` in a
  separate Python process to assess generator/shared-host headroom. This is a trivial
  peer control, not a framework performance ranking.
- `serve/archive-report.py`: lossless gzip evidence with hashes and compact summary.

## Native cost and generator control

The [independent trivial-peer control](generator-control.json) validates all
18,000 arrivals at 600/sec over 30 seconds: offered p99 4.46 ms, generator-lag p99
1.56 ms, maximum offered latency 64.55 ms. This demonstrates generator headroom
in that window; it does not eliminate intermittent host contention in other windows.
The peer runs in a separate process on the same machine. No comparison with Python
as a production HTTP framework is intended.

A separate [5-second native sample](sample-paced-summary.json) during
[300/sec diagnostic traffic](sample-paced.json) has 3,531 main-thread stack samples.
Top-of-stack counts include:

| Function | Samples | Share of observed stacks |
| --- | ---: | ---: |
| `neoclr_gc_allocate_v1` | 749 | 21.2% |
| `collect_indexed` | 729 | 20.6% |
| `neoclr_gc_stack_roots_v1` | 576 | 16.3% |
| `strlen` | 244 | 6.9% |
| `bzero` | 203 | 5.7% |
| `strstr` | 182 | 5.2% |

These are sampled wall-stack shares, not exact CPU attribution or speedup forecasts.
The complete compressed stack tree and executable identity are retained. Socket
syscalls do not dominate this sample. Inspection of the matching source explains
concrete repeated costs:

- `runtime/native/gc/native-gc.c`: every allocation validates every heap span while
  remembering the first free fit. It intentionally detects malformed trailing spans
  before mutating heap/output; merely stopping at the first fit breaks that contract.
- `runtime/native/gc/root-probe.c`: every root publication checks JSON plan length
  with `strlen` and the required marker with `strstr`, then validates storage across
  the active frame chain. Native plans are image metadata, but any cache/hoisting
  change must preserve descriptor validation, image lifetime and mutable-test cases.
- Paced GC still collects after 16 allocations or 64 published boundaries, and idle
  acquisition collects too. No policy defaults or validation checks were removed.

The [GC timing-only relink](gc-link.json), run separately at 150/sec for 15 seconds,
records [131,101 collections, 1.736 seconds cumulative GC time, maximum 16.326 ms,
7,555,240 root publications and 874,832 allocations](gc-counters.json). All allocations
are reclaimed; failures are zero; peak live payload is 48,931 bytes and arena extent
75,808 bytes. These are whole-process totals, including warmup, idle and shutdown.
The [load report](gc-measurement.json) retains all 2,250 measured responses, a 95.3 ms
p99 and 12.9 ms generator-lag p99. Window server CPU is 6.26 seconds; the whole-process
GC total must not be divided by that window CPU as an exact GC percentage.

A single observed GC pause does not explain the largest 163 ms end-to-end latency
in this run. Multiple collections, queueing, OS scheduling and delayed client
observation remain contributors. Root/heap validation is the next concrete CPU
optimization candidate; the current evidence does not prove which mechanism causes
every tail event. A .NET-compatible async API does not require these particular
JSON-validation and arena-scanning implementation costs. Optimizing them needs
focused GC/root correctness evidence, independently of HTTP capacity qualification.

## Prospective sustained-rate procedure

The [recorded plan](qualification-plan.json) fixes targets before long trials:
100% valid offered responses, successful offered p99 <=250 ms, generator-lag p99
<=5 ms, zero server errors and clean shutdown. Start with two unprofiled 120-second
paced trials at 150/sec and two 60-second default-stress trials at 5/sec. Follow
paced success with two 60-second 200/sec trials; on failure, test 100/sec twice for
120 seconds. Retain every trial. No builds or sampling profilers run during timing.
Use 64 client slots so brief stalls can queue, and retain offered latency and misses.
Report RSS first/last/peak and first/last ten-second medians; neither these short
windows nor RSS alone prove absence of leaks. A passing rate is a local tested
operating point for this fixture and policy, not a maximum or production guarantee.
