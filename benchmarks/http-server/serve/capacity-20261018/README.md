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
