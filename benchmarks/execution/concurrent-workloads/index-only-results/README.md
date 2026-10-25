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

Next: interleaved plaintext/JSON HTTP, then owner/JIT qualification if the HTTP
results justify selection. Current defaults and website baseline remain unchanged;
this evidence supports a candidate, not concurrent superiority over synchronous GC.
The tradeoff is potentially slower repeated same-object writes versus eliminating
range-cache management for allocation-heavy work. This reuses the existing
[.NET/Go allocation-index research](../allocation-index-results.md).
