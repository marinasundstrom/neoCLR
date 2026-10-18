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

## Activity transition and repeated qualification

The original two 120-second 150/sec trials both fail: 29/85 admission misses,
offered p99 290.9/132.7 ms, and generator-lag p99 45.6/40.4 ms. The second trial
includes a 762 ms dispatch delay followed by catch-up arrivals filling all 64 slots.
The first also shows sustained occupied slots before its miss burst; not every
miss can be reduced to one delayed generator wakeup. All admitted responses validate.

The author then said, “OK. I stop side activities”. The in-progress 100/sec pair
spans that transition and is retained as such: the first trial validates all 12,000
responses but fails the lag target at 5.30 ms; the second validates all 12,000 with
8.62 ms offered p99 and 1.87 ms lag p99, passing. Subsequent runs start fresh after
the activity change. This is a changed host condition, not an implementation speedup.
The machine remains a shared desktop, with background services and resource sampling.

| Policy | Offered/sec | Duration per trial | Validated, trial 1 / 2 | Offered p99 ms, trial 1 / 2 | Lag p99 ms, trial 1 / 2 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Experimental paced | 150 | 120 s | 18,000 / 18,000 | 7.22 / 6.98 | 1.05 / 1.04 |
| Experimental paced | 200 | 60 s | 12,000 / 12,000 | 8.54 / 5.78 | 0.79 / 0.58 |
| Default stress | 5 | 60 s | 300 / 300 | 64.04 / 66.80 | 2.75 / 2.54 |

All six trials pass the recorded criteria, with zero admission misses, invalid
responses, server errors or shutdown errors. Timed completions equal offered rates.
Paced peak sampled RSS is 12,864–12,896 KiB; final samples are 11,152–11,216 KiB.
Stress peak/final RSS is 12,896 KiB. There is no growing RSS trend in these windows;
this is not a leak proof or coverage of slow-client/body/keep-alive/TLS workloads.
CPU per completion is 2.69–2.91 ms paced, versus 53.4–58.2 ms stress at its control
rate. The stress 5/sec point is a conservative control, not its capacity ceiling.

The first 200/sec run reaches 23 client operations in flight, above the old cap of
16, while its rare maximum offered latency is 111 ms. A finite generator cap is
part of the experiment: at 200 arrivals/sec, a 250 ms pause can produce 50 scheduled
arrivals. The old 16-slot cap has less room for such a transient. This arithmetic
illustrates the budget choice; it is not a proof that all requests share one delay.
Do not translate generator admission misses into HTTP rejection counts.

## Coincident control and higher-rate diagnostic

`coincident-control.py` drives Serve at 400/sec and the trivial peer at 100/sec from
one generator loop for 30 seconds. This is deliberately separate from qualification:
the extra peer changes the workload. All 12,000 Serve and 3,000 control responses
validate. Serve offered p99 is 45.41 ms, versus 3.08 ms for control; lag p99 is
0.68 ms for both. Serve uses about 28.89 sampled CPU seconds of the 30-second window
(~96% of one core), while control uses 0.50 seconds. Peak client in-flight is 20/2.
This supports a server CPU cost near this higher rate while the generator and peer
remain responsive; it does not prove a universal 400/sec ceiling.

A separate GC-timed 400/sec run validates all 6,000 measured requests over 15 seconds,
with offered p99 90.98 ms and lag p99 0.65 ms. Whole-process counters include 20,003,480
root publications, 320,575 collections, 4.148 seconds collection time and a maximum
observed collection of 0.732 ms. All 2,295,142 allocations are reclaimed, failures
are zero, peak live payload remains 48,931 bytes and arena extent is 77,040 bytes.
Higher latency therefore need not imply growth in the managed live graph; TCP and
client queues are outside that metric. Different host conditions and instrumentation
mean this is not a controlled speed comparison with the earlier 150/sec GC run.

The successful 400/sec diagnostic triggers a separately recorded extension: two
unprofiled 120-second 400/sec trials, followed by a 30-second exploratory 500/sec
boundary check. Their results determine whether 400/sec can join the repeat-qualified
points. A short success alone does not change the sustained-rate conclusion.

## Sustained result and overload boundary

Both fresh **400 offered/sec, 120-second** trials pass: each validates all **48,000**
responses, with no misses or server errors and clean shutdown. Offered p99 is
**115.79 / 98.59 ms**; generator-lag p99 **0.652 / 0.669 ms**. Timed completion
rates are **399.983 / 399.992/sec**; two/one responses complete during drain and
are not added to timed throughput. Server CPU totals are 113.96 / 114.83 seconds
(~95–96% of one core). Sampled peak RSS is 12,880 / 12,896 KiB.

The **500/sec, 30-second** exploratory check fails: 12,394 of 15,000 offered
responses validate, with **2,606 client admission misses**. Timed throughput is
**411.0/sec**, successful offered p99 163.39 ms, lag p99 0.515 ms and server CPU
29.96 seconds (~100% of one core). Every admitted response validates; server errors
are zero. Low generator lag plus saturated server CPU and a throughput plateau
supports a server CPU bottleneck in this fixture. Successful latency alone would
hide the overload losses. 500/sec is not qualified.

**400/sec is the highest repeat-qualified local point in this investigation**, under
experimental paced GC and the quieter host conditions, with a 64-slot client budget.
It is close to a single-core CPU limit and has little headroom. The 150/200 points
have substantially lower tail latency. This is not a production guarantee, a default-GC
claim, or an exact maximum; 401–499/sec, long-duration memory behavior, other hosts,
keep-alive, JSON, TLS, large bodies and slow clients remain unqualified.

The next concrete CPU work is root metadata/ancestor validation and arena allocation
scans, followed by collection itself. Preserve malformed-descriptor/heap detection,
root ownership and stress-GC tests when optimizing. No runtime code or GC policy was
changed to obtain these rate differences.

Evidence: [earlier failed 150/sec trials](paced-150-long.json),
[100/sec transition trials](paced-100-long.json), [author activity transition](author-quiet-transition.json),
[quieter-run plan](quiet-plan.json), [400/sec extension plan](extended-plan.json),
[150/sec qualification](quiet-paced-150.json), [200/sec qualification](quiet-paced-200.json),
[default stress control](quiet-stress-5.json), [400/sec qualification](quiet-paced-400.json),
[500/sec boundary](quiet-paced-500-boundary.json), [coincident peer](coincident-overload.json),
and [higher-rate GC counters](gc-overload-counters.json)/[load](gc-overload.json).
Each load summary links a complete hashed gzip report; corresponding `*-analysis.json`
files retain per-second outcomes, slowest requests and beginning/end RSS medians.
The [admission burst excerpts](admission-bursts.json) and [busy-host snapshot](host-during.json)
retain the earlier failure context. Six focused harness/analysis tests pass.

The author subsequently reaffirmed that comparison with a web application in Go
remains the goal. This qualification supplies neoCLR's matched load contract and
local baseline; the historical 32-request Go pilot is a different workload. A new
Go `net/http` fixture is measured in the [following matched comparison](../go/README.md).
This capacity slice remains neoCLR-only evidence; cross-runtime ratios belong to that
separate alternating series.
