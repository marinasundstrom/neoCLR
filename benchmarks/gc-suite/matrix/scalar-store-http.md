# Typed scalar-store HTTP qualification — 2026-10-25

Typed scalar stores are now the normal path within the private concurrent
candidate. `prepare-http.py` rebuilds the collector/provider from current sources
and records compiler settings; `--generic-scalar-stores` selects the former
generic lowering for comparison. The old `--scalar-array-stores` spelling remains
an alias for the normal path. Ambient selection is cleared before compilation. `--reuse-assembly`
allows the second build to use the exact same compiled benchmark assembly.

`pair-scalar-http.py` validates current input hashes, requires control/candidate
selection to differ as declared, and checks byte-identical generated collector and
provider sources plus sync objects. It pairs their recipes for interleaved runs
without rewriting the source build reports. Both collectors retain their separately
tuned budgets; this does not impose equal CPU/memory policy or change defaults.

The fixture and wire checks are unchanged from the standard HTTP matrix: plain
text and typed JSON, response bytes/status/framing, per-connection request ordering,
shutdown counts, completed concurrent epochs and zero invalid-state reports.
Close, keep-alive and pipeline coverage are distinct. One owner/four clients does
not establish multi-owner scaling, shared-heap safety, saturation or peer parity.

The measurements below predate default integration and retain their original
opt-in flags and input hashes. See [integration validation](scalar-store-integration.md)
for the selection change; no new performance result is inferred from it.

## Reproduce

```sh
python3 benchmarks/gc-suite/matrix/prepare-http.py --generic-scalar-stores --output target/new-http-control
python3 benchmarks/gc-suite/matrix/prepare-http.py --reuse-assembly \
  --scalar-array-stores --output target/new-http-candidate
python3 benchmarks/gc-suite/matrix/pair-scalar-http.py \
  --control target/new-http-control --candidate target/new-http-candidate \
  --output target/new-http-pair
python3 benchmarks/gc-suite/matrix/http.py --build target/new-http-pair \
  --output target/new-http-smoke --requests 64 --rounds 1 --smoke \
  --modes sync-aot live-aot scalar-aot sync-jit live-jit scalar-jit
```

Native AOT/JIT only: no interpreter implementation changes arise from this
optimization. Diagnostic/profile runs remain separate from throughput runs.
No public API or Raven compiler/CLI bridge change is involved. The previous
[CoreCLR typed-store comparison and safety evidence](scalar-store-follow-up.md)
remain the design basis.

## Qualification result

All 36 cells pass: 2,880 checked responses including warmup, across two payloads,
three connection policies and six native variants. There are 598 completed
concurrent epochs and zero invalid-state reports. Every paired live/scalar cell
has identical allocation counts and allocated bytes. `nm -u` confirms the scalar
helper is imported by the candidate HTTP object and absent in the control.

After that cohort, the HTTP driver was hardened to clear ambient scalar selection
before applying each recorded recipe, as it already did for stack/byref flags.
An explicit `NEOCLR_LIVE_SCALAR_ARRAY_STORES=1` launch passes two additional JIT
pipeline qualification cases (40 responses including warmup). The control host
does not export the scalar helper, so leaked selection would fail to link/execute.
The original cohort preserves its original driver hash; the ambient-control report
records the updated driver. This is a harness isolation fix, not a runtime change.

Reports under `scalar-store-evidence/` retain `http-control-build`,
`http-candidate-build`, `http-pair-build`, `http-symbols`, `http-smoke` and
`http-ambient` JSON gzip files. Both native builds use current generated collector
and provider sources. Source/recipe hashes are retained; not every transitive
header is separately hashed. The fixture and standard sample definitions are
unchanged. No website build/publication was needed for these prose/data updates.

## Next measurement

Run the paired build without instrumentation, interleaving sync, concurrent control
and scalar candidate over close/pipeline plain/JSON traffic, with repeated rounds
and the standard 4,096-request window. Keep allocation/epoch/output checks and
report both throughput and whole-invocation CPU/memory; invocation includes warmup
and cleanup. Treat small mixed effects honestly. Profile pause distributions
separately. These qualification timings do not establish an HTTP gain, and no
collector/default setting changes follow from passing the wire checks.

## Repeated HTTP comparison

All 72 native invocations pass 296,064 responses (4,096 measured plus 16 warmup
per invocation), with 32,174 completed concurrent epochs and no invalid-state
reports. Three interleaved rounds cover both payloads, close/pipeline protocols,
and all six variants. This one-owner/four-client fixed-work cohort is not a
saturation or external-framework ranking. JIT compilation is excluded from
invocation CPU; traffic-window throughput and whole-invocation CPU have different
boundaries. Raw results: `scalar-store-evidence/http-cost.json.gz`.

Median requests/second:

| Payload | Protocol | Execution | Sync | Concurrent control | Scalar candidate |
| --- | --- | --- | ---: | ---: | ---: |
| plain | close | AOT | 5388 | 4115 | 4198 |
| plain | close | JIT | 3195 | 2584 | 2493 |
| plain | pipeline | AOT | 7539 | 6124 | 6174 |
| plain | pipeline | JIT | 4314 | 3711 | 3732 |
| json | close | AOT | 3594 | 2783 | 2780 |
| json | close | JIT | 2040 | 1742 | 1791 |
| json | pipeline | AOT | 4107 | 3493 | 3495 |
| json | pipeline | JIT | 2426 | 2201 | 2191 |

Median invocation CPU milliseconds:

| Payload | Protocol | Execution | Sync | Concurrent control | Scalar candidate |
| --- | --- | --- | ---: | ---: | ---: |
| plain | close | AOT | 754.35 | 1003.68 | 985.21 |
| plain | close | JIT | 1276.29 | 1559.22 | 1647.42 |
| plain | pipeline | AOT | 545.58 | 692.88 | 686.99 |
| plain | pipeline | JIT | 954.18 | 1123.03 | 1117.21 |
| json | close | AOT | 1143.02 | 1473.81 | 1471.73 |
| json | close | JIT | 1970.82 | 2343.51 | 2292.83 |
| json | pipeline | AOT | 1005.90 | 1201.73 | 1201.69 |
| json | pipeline | JIT | 1690.94 | 1888.49 | 1895.77 |

Scalar effects on HTTP are small and mixed, unlike the large standalone
buffer-reuse gain. Plain close JIT is 5.7% higher CPU and 3.5% lower throughput in
this cohort; do not hide that regression or declare it proven noise. Other CPU
changes range from about 2.2% better to 0.4% worse. Sync remains faster in every
measured configuration. These results support keeping sync as the release default,
not abandoning continued concurrent/runtime optimization. Configuration and run
variation remain limitations of three short rounds.

## Separate owner-operation profile

Twelve pipeline plain/JSON AOT/JIT cases pass 12,480 responses. The existing
owner-operation instrumenter wraps complete collect/begin/finish/cancel calls and
guarded writes in copied collectors. Its deterministic nested/TLS/histogram and
negative controls pass with UB/bounds sanitizers. The summarizer now keeps protocol
in group keys to avoid pooling different connection policies. Begin/finish/cancel
counts match independent collector counters; all histogram consistency checks pass.

Outermost-GC p99 bucket bounds are 16.384–32.768 microseconds for sync except JSON
JIT at 32.768–65.536; concurrent control and candidate are 65.536–131.072 throughout.
Concurrent has fewer outer operations and lower aggregate owner-GC wall time in
these cases, but longer per-operation p99 intervals. Maxima include scheduler noise
and one live JSON-JIT outlier of 563 microseconds. This is one diagnostic run per
cell, not a pause-regression verdict or an HTTP tail-latency comparison.

These measurements exclude background marker CPU and nonwrapped runtime work;
scalar writes bypass the guarded-write path because primitive payloads are not
traced. Inclusive rows overlap: use outermost-GC for disjoint selected GC operations,
never sum all rows. Instrumentation overhead is included. Reports `http-pause-build`,
`http-pauses` and `http-pause-summary` preserve raw evidence separately from timing.
The wider multi-owner/shared-worker pause/default qualification remains open.
