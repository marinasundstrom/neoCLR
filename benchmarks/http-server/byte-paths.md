# HTTP byte parsing and response writing — 2026-10-13

Development measurement on macOS ARM64 (Apple M1), following the collector change.
This does not replace the published Preview 14 baseline. The implementation accepts
received blocks at the decoder boundary, uses internal byte windows, and constructs
response framing as bytes. It removes temporary strings and copies while retaining
buffered request/response ownership. See [implementation and protocol checks](../../docs/experiments/http-byte-paths/README.md).

## Allocations per request

Diagnostic AOT counters snapshot cumulative successful allocations at accepted
connection closes 8 and 32. Differences cover 24 warm requests, including HTTP,
handler and JSON work. Counts exclude the first eight requests and teardown.
TEXT counts are native text-storage blocks, **not a count of all managed String
objects**. Bytes are requested allocation bytes, not resident memory or heap capacity.

| Workload | Metric | Before | After | Reduction |
| --- | --- | ---: | ---: | ---: |
| Plain text | Allocations/request | 559 | 519 | 7.16% |
| Plain text | Bytes/request | 22,346 | 20,913 | 6.41% |
| Plain text | TEXT blocks/request | 152 | 137 | 9.87% |
| Plain text | TEXT bytes/request | 1,909 | 1,344 | 29.60% |
| JSON | Allocations/request | 1,256 | 1,210 | 3.66% |
| JSON | Bytes/request | 38,829 | 37,294 | 3.95% |
| JSON | TEXT blocks/request | 528 | 507 | 3.98% |
| JSON | TEXT bytes/request | 9,032 | 8,377 | 7.25% |

Raw captures: [plain before](byte-path-allocation-baseline-plain-20261013.json),
[plain after](byte-path-allocation-candidate-plain-20261013.json),
[JSON before](byte-path-allocation-baseline-json-20261013.json),
[JSON after](byte-path-allocation-candidate-json-20261013.json).
Matching `-stacks.txt.gz` files retain sampled stacks. Instrumented timings are not
used as throughput evidence. These allocation counts were not separately measured
in JIT or interpreter mode.

All four captures validated 32 responses, reported zero allocation failures and
reclaimed every allocated object by process completion (17,908/16,628 plain;
40,214/38,742 JSON). Peak live bytes stayed 8,119 plain and 12,542 JSON.
Peak JSON arena bytes increased slightly, 19,640 to 19,688; reduced allocation does
not imply every heap metric improves. No collection policy, root tracking or native
collector code changed. Previous sanitized collector/retained-stack evidence remains
applicable; this slice additionally exercises managed ownership under collection.

## Uninstrumented throughput

[Raw matched results](byte-path-comparison-20261013.json) retain commands, component
hashes, all samples and summaries. Both variants use the same optimized JIT host and
current AOT collector; only System.Web differs. Both compile the same application
sources with the same compiler and other libraries. Builds finish before timing.

Each process serves 32 sequential loopback requests over separate connections,
with the first eight discarded. Eight warmup processes precede five alternating
rounds across two workloads, two native modes and two variants: 48 processes,
1,536 checked responses, 960 measured requests. No profiler runs during timing.

| Workload / mode | Before median req/s (range) | After median req/s (range) | Change |
| --- | ---: | ---: | ---: |
| Plain / JIT | 66.23 (65.49–66.71) | 69.20 (65.36–74.24) | +4.5% |
| Plain / AOT | 69.31 (66.76–69.82) | 77.29 (76.13–77.47) | +11.5% |
| JSON / JIT | 36.31 (35.47–36.66) | 38.51 (38.29–38.57) | +6.1% |
| JSON / AOT | 37.89 (37.15–38.40) | 40.01 (38.86–40.68) | +5.6% |

Plain JIT ranges overlap; repeat on a quieter host before treating that smaller
gain as stable. These short concurrency-one runs are exploratory, not saturation
results or comparisons with Go/.NET. Do not combine their ratios with unrelated
published runs. Interpreter received functional checks, not a new timing series.
Windows, TLS and SQL throughput were not requalified here.

## Reproduction and next work

Use `prepare-http-foundation-bundle.py` to build baseline/candidate libraries, then
`benchmark.py --prepare-only --workload plain|json` for each. Run
`compare-library.py` with the four prepared `results.json` paths, the identical
`--jit-host`, and a fresh `--output`. For allocation captures, run `profile.py` with
each prepared `native/build.json`, its `--workload`, and a fresh output directory.
Exact invocations and input hashes are retained in the reports.

The remaining paths still allocate public strings, encode constant framing bytes,
copy response bodies and append bytes individually. Profile those costs before
introducing buffer pooling or a public view abstraction: views retained across await
must have explicit ownership, and pooled buffers must not be reused while borrowed.
Collector/root-boundary overhead remains a separate measured opportunity.
