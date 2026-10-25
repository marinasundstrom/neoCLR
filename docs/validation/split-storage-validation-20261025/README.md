# Split small-frame storage validation — 2026-10-25

The [current profiles](../current-runtime-stacks-20261025/README.md) identify common
frame-entry/root-state costs in sync and concurrent execution. Retained-stack TLS
grouping was already rejected; this slice instead follows the existing
[small-frame counts and scalar-loop experiment](../../../benchmarks/execution/concurrent-workloads/scalar-frame-results.md).
Most measured activations have at most four descriptors. Existing .NET/CLR root-map
research remains applicable: immutable layout facts do not prove dynamic slot
addresses. There is no API or metadata semantic divergence in this change.

## Candidate and safety

`NEOCLR_GC_SPLIT_STORAGE_VALIDATION` is opt-in. Zero through four slots use a scalar
loop; larger tables use a non-inlined helper retaining the existing vectorizable
loop. Every address, read width and applicable flags predicate remains checked on
each admitted lifetime. Root values, ownership, transient/ancestor rules, stack
bounds and rejection behavior are unchanged. The original loop remains the default.
This is a native code-generation experiment, not a metadata cache or GC exemption.

ARM64 disassembly confirms that owned-frame entry shrinks from 160 to 96 stack
bytes and no longer saves eight SIMD registers. Large tables incur an extra call;
they are not presumed faster. Native owned-chain/checked-root qualification passes
138 commands across worker, pacing and sanitizer settings. An additional 136
commands compare original/split validators at 0, 1, 4, 5 and 64 slots, with invalid
first/last addresses, widths and flags plus the legitimate unchecked-flags mode.
UB/bounds sanitizer failures are rejected. No new Windows qualification is claimed.

## Matched AOT HTTP screen

Three interleaved rounds validate 98,688 responses, plain and JSON, sync and live,
original and split, four clients and pipeline depth 16. Generated objects, native
policy, host configuration and source are identical within each pair; only the
split-validation define changes. Builds finish before timing.

| Handler | Collector | Invocation CPU change | Request throughput change |
| --- | --- | ---: | ---: |
| Plain | Sync | −4.27% | +4.46% |
| Plain | Concurrent | −1.55% | +1.57% |
| JSON | Sync | −2.57% | +2.72% |
| JSON | Concurrent | −1.58% | +1.56% |

These are descriptive changes in three-round medians, not confidence intervals or
saturated capacity. CPU includes the complete invocation; client throughput covers
measured traffic. Sync remains faster. This small screen does not establish a broad
concurrent advantage, lower pauses, or an allocation improvement. Keep the switch
experimental until standalone and further JIT performance evidence support promotion.

## Reproduction

`owned-root-chain/validate.py --split-validation` and
`owned-root-chain/validate-storage-boundary.py` retain safety controls.
`matrix/prepare-root-variant.py` validates a fresh standard HTTP build's inputs and
relinks the same objects with the candidate define, keeping original and split
AOT/JIT recipes. `matrix/http.py` runs those modes with the usual response/lifecycle
checks. `matrix/compare-root-validation.py` relinks the six standard standalone
samples from retained qualified objects, records current native source hashes,
interleaves five rounds and excludes the first as warmup. These paths are relative
to `benchmarks/execution` or `benchmarks/gc-suite` respectively.

Raw reports and native entry disassembly are compressed alongside this record.

JIT smoke qualification also passes eight cases and 640 responses across both
collectors and validator variants. These short runs are correctness evidence,
not JIT performance measurements.

## Standalone AOT follow-up

Sixty concurrent invocations pass exact output, lifecycle and matched allocation
checks across all six standard samples. Five interleaved rounds include one warmup
round excluded from the medians. This is concurrent-versus-itself evidence.

| Sample | Original CPU ms | Split CPU ms | Change |
| --- | ---: | ---: | ---: |
| compute | 65.400 | 65.356 | -0.07% |
| reuse-buffer | 55.586 | 55.728 | +0.26% |
| steady | 156.403 | 149.730 | -4.27% |
| bursts | 83.156 | 78.286 | -5.86% |
| retained-window | 330.881 | 317.728 | -3.97% |
| mixed-phases | 158.060 | 151.047 | -4.44% |

Managed allocations and allocated bytes match within every sample. These short
cohorts do not establish statistical significance or large-frame performance.
Further JIT timing and cross-platform validation remain open before integration
into the selected policy; defaults are unchanged.
