# Current integrated runtime stack profiles — 2026-10-25

The canonical concurrent source integration is commit `d7389954a`. A fresh native
build uses the pinned qualified compiler runner, typed scalar stores and selected
owned policy. Eight one-second macOS captures after warmup cover plain/JSON,
sync/concurrent and AOT/JIT with four clients and pipeline depth 16. All 262,272
responses (32,768 + 16 warmup per case), output, lifecycle and GC checks pass.
This is diagnostic evidence, not a throughput comparison or default qualification.

## Current observations

| Handler/mode | TLS | Frame entry | Stack roots | Generic store | Header check |
| --- | ---: | ---: | ---: | ---: | ---: |
| plain sync-aot | 78 | 108 | 109 | — | — |
| plain live-aot | 106 | 92 | 41 | 25 | 9 |
| plain sync-jit | 68 | 63 | 48 | — | — |
| plain live-jit | 74 | 40 | 40 | 27 | 11 |
| json sync-aot | 110 | 76 | 64 | — | — |
| json live-aot | 94 | 75 | 41 | 27 | 11 |
| json sync-jit | 69 | 54 | 38 | — | — |
| json live-jit | 81 | 52 | 34 | 33 | 6 |

These are top-of-stack wall observations, not CPU percentages. Dashes mean below
the five-observation reporting threshold or absent, not zero. Sleeping marker and
JIT test-runner threads are included in raw reports; do not count their waits as
busy CPU. Sample counts and scheduling vary, so these columns do not establish
before/after gains or exact cost fractions. JIT generated code is less completely
symbolized than AOT; native service names are still visible.

TLS caller extraction has zero unattributed observations in every case. Native
stack checks and owned frame entry recur in both collectors, while store admission
and allocation-index lookup add concurrent-specific work. Root-state entry and
publication remain visible in both handlers and modes. This supports investigating
common runtime services alongside the collector, rather than attributing the
entire remaining HTTP gap to background marking or its synchronization gate.

## Next bounded change

Inspect native stack-check/TLS and small-frame entry code generation with current
source and call-site counts. Preserve dynamic stack bounds, current owner identity,
frame lifetime, mutable root values and malformed-descriptor rejection. Existing
[small-frame experiments](../../../benchmarks/execution/concurrent-workloads/scalar-frame-results.md)
already establish that per-activation validation is not redundant image metadata:
immutable layout and dynamic slot addresses must be distinguished. Select a change
only after that inspection, then compare unprofiled plain/JSON and standalone
workloads against this same integrated candidate and retain sync controls.

## Reproduction and controls

```
python3 benchmarks/gc-suite/matrix/http.py \
  --build target/gc-current-stack-build --output target/gc-current-stack-profile \
  --modes sync-aot live-aot sync-jit live-jit --protocols pipeline \
  --requests 32768 --rounds 1 --sample-stacks
python3 benchmarks/execution/concurrent-workloads/summarize-cpu.py \
  --measurement target/gc-current-stack-profile/results.json \
  --output target/gc-current-stack-profile/summary.json
```

Reports retain input/capture hashes and exact commands; compressed raw captures
retain call trees and offsets. The parser accepts the legacy single-handler schema
and rejects a modified capture hash before writing a summary. The harness rejects
short sampling runs before creating output or launching a server. No runtime or
collector semantics change in this slice. Whole-process child CPU includes the
sampler; server-reported invocation CPU remains subject to sampling interference.
Run unprofiled cohorts separately for performance claims.
