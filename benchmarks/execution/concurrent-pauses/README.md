# Outermost collector-operation pauses — 2026-10-24

The automatic concurrent candidate's original GC timer excludes background waits
before remark and splits nested collector passes into separate histogram samples.
This diagnostic instruments the outer operation so that a nested finish, synchronous
recheck and post-idle capture form one blocking interval. It also measures context
completion and heap release, including cancellation and worker shutdown.

The instrumentation is generated out of tree for both the synchronous collector
and the automatic candidate. A thread-local nesting counter prevents double
counting; failures still record their duration. Reports do not mutate counters.
The schema is 64 logarithmic histogram buckets, operation count, total ns, max ns,
failure count, five operation-category counts and one reserved zero. Categories
are requested collection, idle/paced collection, direct conditional collection,
invocation completion and heap release. Nested operations belong only to their
outer category. Root publication/descriptor validation before collector entry and
HTTP/network time are outside this measurement. These are owner-thread intervals,
not shared-heap safepoint rendezvous or application latency distributions.

## Validation

Optimized and undefined/bounds-sanitized tests pass for both variants. Seven outer
operations produce exactly seven histogram entries and the expected category and
failure counts despite the candidate's nested work. Repeated reads are identical.
A test-only 5 ms delay before the candidate's background wait must appear in the
outer maximum; it lies outside the original inner timer. HTTP builds omit that
macro. Final generated sources, compiler flags and raw reports are retained.

Final sequential-client cohorts pass 1,024 responses for every collector/sample/
mode combination: 8,192 responses covering plain/JSON and native JIT/AOT. No other
qualification jobs run alongside the final cohorts. Initial profiles and pre-delay
tests are retained separately. This is one diagnostic cohort per combination,
not a capacity trial or statistical confidence interval.

## Findings

| Sample/mode | Sync outer µs/request | Candidate outer µs/request | Candidate inner µs/request | Candidate p99 bucket upper µs |
| --- | ---: | ---: | ---: | ---: |
| Plain AOT | 25.53 | 81.04 | 79.34 | 262.143 |
| Plain JIT | 21.98 | 69.68 | 68.30 | 524.287 |
| JSON AOT | 45.35 | 86.54 | 84.65 | 131.071 |
| JSON JIT | 61.01 | 79.87 | 78.25 | 131.071 |

Candidate outer-minus-inner time is 1.4–1.9 µs/request. Most blocking time remains
inside capture/remark/synchronous collection work in these runs; hidden waits do
not explain most of the regression. The difference also includes wrapper/timer,
invocation and release overhead, so it is **not a pure waiting-time measurement**.
Wall time includes descheduling. For example, synchronous JSON JIT has a 3.071 ms
maximum outlier; maxima and bucket quantiles must not be treated as CPU cost or
HTTP tail latency. Startup and shutdown are included in totals and histograms.

These data make the extra synchronous idle recheck the next concrete target.
Removing it changes floating-garbage and resource-cleanup timing, so that change
requires lifecycle/pressure/shutdown tests and memory measurements. This report
does not establish that rechecks can be removed safely or that concurrent GC
should become the default. Background-worker CPU, full standalone workload and
worker/platform qualification remain part of the original goal.

## Reproduction

Run `validate.py --output <fresh-directory>` to generate and qualify both sources.
Use `measure.py --synchronous --gc-source <sync-instrumented.c>` for the baseline,
and omit `--synchronous` for the candidate; select `--sample plain|json`,
`--requests 1024` and a fresh `--output` directory. Existing qualified managed
objects and the owned-host recipe are reused as documented in concurrent-integration.
`summarize.py` accepts the four final gzip reports and `--output summary.json`.
