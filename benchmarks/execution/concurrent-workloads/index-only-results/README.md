# Simplify concurrent store lookup — 2026-10-25

The author asked to remove unnecessary bookkeeping and investigate simpler
structures serving the same purpose. The first cache-reinsertion screen found no
meaningful gain. This follow-up removes the entire recent-range cache from the
store adapter, using the existing owned allocation-start index for interior writes.
It uses the existing `NEOCLR_LIVE_NO_STORE_CACHE` contract; no GC or compiler
semantics have changed. The measurement tool now exposes this configuration and
pairs it with each selected live pacing policy via `--compare-index-only`.

## Structures and obligations

| Structure | Purpose | Candidate treatment |
|---|---|---|
| Owned allocation-start bitmap and summary | Current allocated bases and predecessor lookup | Retained, including sweep/reset/release revocation |
| Exact-base cache | Fast positive base admission across runtime services | Retained; this experiment does not remove it globally |
| Four recent store range hints and replacement cursor | Avoid repeated allocation walks | Removed from store state and store updates |
| Detached epoch index and headers | Locate old snapshot objects and validate immutable identity | Retained, including mutation gate/logging |

Recent-range hints were valuable before the persistent allocation index replaced
linear fallback walks. They are not inherently necessary for safety: the checked
index lookup followed by block-state and payload-extent validation supplies the
same admission proof. Marker snapshot ownership, barriers and published shape
checks remain unchanged. This is owner-local only, not shared-heap admission.

## Five interleaved rounds

The same qualified allocation-pattern objects, CPU compiler options, 1024/16384
pacing and 50% log watermark run with both native providers. All hosts compile
before timing. Alternating modes produce 60 correct invocations; allocation counts
and bytes match. Invocation CPU includes cleanup and marker work.

| Sample | Four-range CPU ms | Index-only CPU ms | Change |
|---|---:|---:|---:|
| compute | 61.148 | 61.684 | +0.9% |
| reuse-buffer | 104.950 | 105.304 | +0.3% |
| steady | 158.783 | 145.463 | -8.4% |
| bursts | 86.090 | 78.098 | -9.3% |
| retained-window | 329.535 | 317.546 | -3.6% |
| mixed-phases | 161.973 | 146.415 | -9.6% |

The allocation-heavy reductions reproduce the earlier sequential screen. Compute
and reusable-buffer differences below 1% are not meaningful gains. Native optimized
and UBSan/bounds validation passes 34 commands, including differential liveness,
mutation, extent, lifecycle and deletion-barrier negative controls. Cache-specific
negative controls are inapplicable with cache code removed. These are not TSan or
shared-heap qualifications.

## Reproduction and next gate

```sh
python3 benchmarks/execution/concurrent-workloads/measure.py   --build target/gc-patterns-standard-build --output target/index-only-new   --rounds 5 --allocation-index --compare-exact-allocation-index   --compare-inline-base --live-budgets 1024:16384 --finish-watermark 50   --compare-index-only --modes live-budget-1024-16384-hybrid-inline   live-budget-1024-16384-index-only-hybrid-inline
```

Use the standard suite builder to prepare missing qualified objects. Compressed
reports preserve commands, compiler flags, inputs and raw measurements. The previous
[reinsertion control](../../concurrent-live/repeat-base-screen/README.md) provides
the initial sequential comparison baseline.

The follow-up below qualifies interleaved plaintext/JSON HTTP, owner and JIT paths. Current defaults and website baseline remain unchanged;
this evidence supports a candidate, not concurrent superiority over synchronous GC.
The tradeoff is potentially slower repeated same-object writes versus eliminating
range-cache management for allocation-heavy work. This reuses the existing
[.NET/Go allocation-index research](../allocation-index-results.md).

## HTTP and JIT follow-up

Five interleaved fixed-work AOT rounds, 4096 measured responses plus 16 warmup,
four clients, identical generated objects and native policy except store hints:

| Handler | Four-range requests/s | Index-only requests/s | Four-range CPU s | Index-only CPU s |
|---|---:|---:|---:|---:|
| Plaintext | 4018.4 | 4126.2 | 1.044540 | 1.016097 |
| JSON | 2644.5 | 2725.9 | 1.561444 | 1.522124 |

All 82,240 responses pass body/EOF validation. Median throughput increases 2.7% and
3.1%; CPU falls 2.7% and 2.5%. Median arena peaks remain about 309 KB
for plaintext and 353 KB for JSON; precise values are in reports.
Allocation counts differ by at most small scheduling-dependent amounts (plaintext
medians 855841/855840, JSON both 1879753). No allocation reduction is claimed.

Fresh managed JIT executions pass 512 plaintext and 512 JSON requests with current
immutable-byref and stack-forwarding options, completed epochs and cleanup. These
are correctness runs, not JIT performance measurements.

The first HTTP run stopped because the harness expected cache hits in the newly
cache-free mode. The run is retained as `http-initial-harness-failure.json.gz`;
qualification now explicitly requires zero hint hits for index-only stores.

## Isolated-owner follow-up

Three rotated 5-second windows per configuration, one-second warmup, 32 Go client
lanes, one or four isolated owner heaps; 520,953 total validated responses:

| Owners | Four-range requests/s | Index-only requests/s | Four-range CPU µs/response | Index-only CPU µs/response |
|---|---:|---:|---:|---:|
| 1 | 4158.4 | 4155.2 | 245.68 | 240.80 |
| 4 | 10170.2 | 10569.2 | 343.92 | 338.54 |

One-owner throughput is flat; CPU falls about 2.0%/1.6%. Four-owner index-only
throughput ranges 9226–10764 requests/s, so its median advantage is provisional.
Median maximum RSS remains about 12 MB/25 MB for one/four owners. No shared-heap
writers, sanitizer timing, cross-framework ranking or production-default claim.

Use `gc-suite/run.py benchmark --index-only` or `profile --index-only` to exercise
the policy through the standard suite and preserve same-collector comparisons.
The default suite policy is unchanged so the original baseline remains reproducible.

## Pause and phase accounting

The standard index-only profile passes HTTP plaintext/JSON (4,160 responses) and
all six standalone cases (12 invocations), including owner-pause, begin, finish
and collection-part validation. A separate three-round paired diagnostic checks
12,480 responses: both concurrent policies have outermost-GC p99 in
[65,536, 131,072) ns for both handlers. Maximum observed durations are 168/166 µs
for plaintext and 132/292 µs for JSON (four-range/index-only). These instrumented
runs do not establish improved pauses; the higher JSON maximum requires caution.
The earlier one-round profile had a higher index-only JSON p99 bucket, reinforcing
that short diagnostic runs are insufficient for a latency improvement claim.

The standard profile also tests end-to-end policy forwarding and records
`indexOnly: true`. Same-collector comparison tests and 26 website tooling tests
pass. Website prose reports the measured gains; no deployment was performed.
Remaining work: longer owner/tail-latency measurements and further reduction of
repeated ownership/extent processing, with the existing safety proofs preserved.
