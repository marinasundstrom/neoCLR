# Reusing qualified stack-argument elision in isolated HTTP owners

2026-10-25. The call-site investigation exposed an integration omission: the
isolated-owner builder selected the qualified optimization-level/native options,
but did not enable the existing complete-closure stack-argument elision options.
The earlier [forwarding qualification](../concurrent-workloads/forward-elision-results.md)
compared forwarding against direct-argument elision; its smaller incremental gains
are not comparable to restoring both on this previously unelided baseline.

`build.py --elide-stack-arguments` now explicitly enables direct and forwarded
stack-argument elision for live compilation and records that choice. It clears
ambient values of those three switches before selecting either path, including
sync compilation. Future manifests also record compiler option values per invocation.
The default/control remains available. No compiler semantics, native ownership,
provider contract or runtime defaults change; this reuses the previously qualified
proof. The reconstructed synchronous object is byte-identical to the baseline.

## Matched cost

Three alternating one-owner pairs validate **89,922 responses** through EOF. Both
builds use identical native options and source; only the live managed object differs.
The compiler removes 13,883 statically guarded stores under the complete-call-closure
proof. Results use 32 lanes, one-second warmup and four-second windows. CPU and
allocations include startup/warmup/drain/shutdown; rates count window completions.

| Live path | Median requests/s | CPU us/response | Managed allocations/response |
| --- | ---: | ---: | ---: |
| Previously unelided | 2,741.00 | 377.30 | 208.039 |
| Direct and forwarded elision | 3,241.75 | 321.59 | 208.034 |

This cohort improves throughput **18.3%** and reduces CPU/response **14.8%**.
Out-of-arena adapter calls fall from about 9,942 to 4,057 per response. The small
allocation difference is amortization, not an allocation-saving claim. All live
trials complete epochs, balance shutdown and release index metadata. The synchronous
collector remains faster in prior comparisons; this does not establish concurrent
default acceptance or pause superiority.

A separate four-owner sync/live check validates **95,412 responses** and clean
per-owner teardown. Observed rates are 14,983/8,990 requests/s and CPU 247/432 us per
response. These single trials are execution checks, not stable capacity claims:
earlier repeated cohorts exposed transport stalls. No Kestrel parity is established.

## Dynamic caller attribution

`profile-stores.py` reuses the bounded owner-thread histogram, reporting from the
owning HTTP thread before teardown. The older instrumentation expected one textual
store declaration; generated borrowed-wrapper conditionals now contain two. Instrument
only the ordinary entry and explicitly reject borrowed-store profiling. Validate
that the retained sync/live compiler selection maps agree before symbol mapping.

The baseline diagnostic run validates 5,236 responses and accounts for exactly
52,058,334 out-of-arena stores, with zero histogram overflow. A separate optimized
profile confirms the reduction. Counts reconcile with the adapter totals and exact
executable symbols; a changed-count negative is rejected. Instrumented timings are
not used in the table. The measurement tool now propagates diagnostic build status;
a further checked short run verifies that marker. Earlier raw instrumented reports
are explicitly classified by their manifests and normalized summary inputs.

After elision, `Result` constructors account for **59.4% of remaining out-of-arena
store calls**. This is a call-count share, not CPU share. Concrete leading symbols
are `neoclr_value_855`, `818`, `791`, `796` and `919` in this exact selection.
Next inspect their destination provenance and caller mixtures; avoid another blind
native micro-optimization. Any specialized path must prove stack extent at its call
site while preserving general heap callers and error/root lifetimes.

## Scope and reproduction

The reused proof already has native/JIT/AOT, JSON and standalone qualification in
the linked report; the latest equal-join extension passes 52 focused tests and emits
byte-identical objects for those five retained workloads. This integration adds fresh
isolated-owner plaintext evidence. It does not establish new JSON/multi-owner JIT,
shared-heap, retention or pause results. Raven emission, Runtime Contract configuration
and CLI/native metadata encoding remain unchanged. Website/public API claims do not
change; this remains experimental native-GC benchmarking.

```sh
python3 benchmarks/execution/concurrent-http-owners/build.py \
  --runner target/release/deps/neoclr_codegen-12f33f26d2ff4df0 --sample plain \
  --elide-stack-arguments --output target/http-forward-new
python3 benchmarks/execution/concurrent-http-owners/profile-stores.py \
  --build target/http-forward-new --client target/http-close-client \
  --output target/http-sites-new
```

Use fresh paths. Alternate uninstrumented baseline/candidate measurement runs with
`measure.py --owners 1 --modes live --seconds 4 --rounds 1`; retain every pair.
[forward-evidence](forward-evidence/) contains builds, paired and four-owner reports,
profile manifests/counts/summaries and the count-mismatch negative. The initial
profiler attempts exposed an obsolete declaration assertion and an incorrect way
of identifying the live selection; both were fixed before the final mapped profiles.
