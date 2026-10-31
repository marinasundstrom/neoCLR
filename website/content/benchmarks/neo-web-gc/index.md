# Neo.Web collector-policy profiling — 2026-10-31

Development measurement on Apple M1, macOS ARM64. This follow-up corrects the
interpretation of the earlier native throughput figures: installed native hosts
use **stress GC**, with full collection at frequent generated boundaries. That
is useful for lifetime validation; it does not characterize the opt-in paced host.
The interpreter has its own policy. No runtime default is changed by this work.

## Matched policy comparison

Both AOT hosts use the same object file, 1 MiB heap budget and build recipe. Only
`NEOCLR_GC_PACED` differs. The private policy currently budgets 256 allocations or
4096 progressing boundaries; idle/pending scheduler collection still applies.
There are three trials per case/policy, five warmups and thirty measured fresh
loopback connections, with policy order alternated and case order rotated.

The median direct-request case increases from **22.3 to 739.5 req/s** (about 33×),
and the generic handler from **17.6 to 621.5 req/s** (about 35×). Every response is
validated. These are short same-program policy comparisons, not production
capacity or a .NET comparison. No other repository build ran during measurement;
workstation background activity was not controlled.

Separate five-second stack samples place 3561/4041 stress samples (88.1%) directly
in collect_indexed or the collection entry routine, with another 302 in mark_frames.
Under pacing, collect_indexed accounts for 75/3258 samples (2.3%); allocation has
766, heap-span validation 647, and poll 912. Sample fractions are observations,
not exact CPU accounting. Profiling runs are excluded from throughput statistics.

## Longer paced characterization

Three trials of 1000 measured requests after twenty warmups per case. Values are
median throughput and the observed range across trials, not confidence intervals.

| Handler | Median req/s | Trial range req/s |
| --- | ---: | ---: |
| direct | 711.4 | 690.0–777.0 |
| direct-context | 655.2 | 448.5–705.8 |
| direct-scope | 692.4 | 385.3–751.9 |
| context | 627.1 | 452.0–634.7 |
| context-middleware | 575.0 | 486.2–578.9 |
| generic | 585.0 | 558.0–587.9 |
| service | 539.1 | 483.1–595.4 |
| query | 551.3 | 506.9–555.9 |
| cancellation | 586.4 | 545.6–589.7 |

The ranges overlap and include substantial outliers. Cancellation has no resolved
additional cost in this screen; that does not make monitoring free. Middleware,
service activation and query parsing are useful next targets, but this evidence
does not support precise additive overhead percentages. Direct variants retain
an identical empty provider/handler setup; direct-scope additionally creates and
disposes a scope per request. Query binding also formats the integer result.
All cases share one image and use a sequential external Python HTTP consumer with
fresh connections. Keep-alive, concurrency, pause distributions, allocation counts
and sustained production workloads are outside this measurement.

## Interpreter/JIT spot checks

One process per row on the expanded comparison image. Interpreter and JIT stress
use 30 measured requests; paced JIT uses 100. All have five warmups. JIT hosts share
32 MiB heap / 4 MiB array budgets; AOT above uses 1 MiB / 64 KiB. Thus the JIT pair
is internally matched, but JIT/AOT are not a controlled same-budget comparison.

| Mode/policy and endpoint | Req/s | Median latency ms | Readiness seconds |
| --- | ---: | ---: | ---: |
| interpreter/interpreter, generic | 3.35 | 282.36 | 4.31 |
| interpreter/interpreter, cancellation | 3.66 | 245.67 | 4.22 |
| jit/stress, cancellation | 16.83 | 58.12 | 67.46 |
| jit/paced, cancellation | 56.68 | 6.35 | 132.05 |

Paced JIT has lower median latency but large spikes (p95 82.9 ms, maximum 210.6 ms).
The single runs do not explain the readiness variation or establish that pacing
caused it. JIT does not reach the AOT screen's throughput; keep this unresolved gap
visible. Do not extrapolate native pacing to interpreter performance.

## Conclusions and next work

The earlier native numbers primarily exposed stress-collection cost. The paced
screen moves the native investigation toward allocation/span validation and
framework invocation. Prepared endpoint invocation remains open. JIT startup and
interpreter execution need their own profiling; switching native policy does not
solve either. Pacing remains opt-in pending broader correctness, memory-pressure,
lifetime, pause and cross-platform qualification. Preserve the stress host as a
correctness configuration rather than silently replacing it in old results.

[Raw evidence and reproduction](https://github.com/marinasundstrom/neoCLR/tree/main/docs/validation/neo-web-performance-20261031)
· [Historical stress-host results](/benchmarks/neo-web/)


## JIT startup follow-up

A subsequent investigation separates metadata/backend preparation from request-time
allocation and heap-span validation. A shared JIT/AOT candidate computes the dense
static-owner layout once during lowering instead of rescanning it for each
static-field access. It remains unselected because no decisive startup gain was
established. The rebuilt Neo.Web native object is byte-for-byte identical;
this change targets preparation cost and does not promise faster request execution.

[Paired JIT measurements, static initialization tests and remaining work](https://github.com/marinasundstrom/neoCLR/tree/main/docs/validation/neo-web-jit-layout-20261031)
record the bounded fix. GC policy and execution-mode defaults remain unchanged.

The final repeated layout comparison records median readiness of 56.09 s before
and 56.47 s after, with overlapping ranges: no decisive whole-process startup gain
is established. Warm throughput is about 578 versus 587 req/s; the emitted native
object is unchanged. The [standalone HttpServer baseline](/benchmarks/http/) now
owns the primitive-server performance question; framework overhead is separate.
