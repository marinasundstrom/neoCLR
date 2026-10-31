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

## Reproduction and validation

Use the 7de5ada65 cancellation-qualified libraries/runtime and Raven 813dad20c.
The build record retains exact input hashes and commands. Build once, then run
without competing repository builds:

```sh
python3 scripts/build-native-project.py --profile http --bundle /path/to/bundle \
  --project benchmarks/neo-web/NeoWebBenchmark.rvnproj \
  --aot target/release/neoclr-codegen --output target/web-profile-build
python3 scripts/profile-neo-web.py --build target/web-profile-build \
  --output target/web-policy --requests 30 --warmup 5 --repetitions 3
python3 scripts/profile-neo-web.py --build target/web-profile-build \
  --output target/web-paced --policy paced --requests 1000 --warmup 20 \
  --repetitions 3 --sample-seconds 0
python3 docs/validation/neo-web-performance-20261031/spot-check.py \
  --bundle /path/to/bundle --build target/web-profile-build \
  --host-recipe /path/to/passing-jit-preparation.json --output target/web-spot
```

The 54 matched policy trials, 27 longer paced trials and four spot checks validate
29,725 responses, including warmups/first requests. Separate sampling validates
another 2,454 responses. Timing assertions check status and exact body, including
query parsing, service invocation and cancellable-token admission. Server processes
are terminated after measurement; lifecycle correctness is covered separately by
the request-abort framework suite, not inferred from benchmark termination.

Compressed profiles retain full native stack samples; profiles.json identifies the
uncompressed bytes/hashes. No compiler/runtime implementation or public API changes
are part of this commit. This work fixes benchmark attribution and adds the
measurement tools needed to choose a subsequent bounded optimization.

The final benchmark source was compiled successfully after whitespace/comment-only
formatting of the listener-selection branch. `final-source-validation.json` retains
both source and assembly hashes: the assembly bytes differ, and timings remain
attributed to the original measured build. No behavioral edit followed measurement.
The website build passed, checking 6,943 pages; its log retains existing API-summary
warnings. Both Python tools executed successfully in the measurements above.
