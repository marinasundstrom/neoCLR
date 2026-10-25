# Typed scalar-store HTTP qualification — 2026-10-25

The standalone optimization remains opt-in. `prepare-http.py` now rebuilds the
concurrent collector/provider from current sources and records compiler settings;
`--scalar-array-stores` selects the same compiler and native-host flag. It explicitly
clears ambient scalar selection for control/sync compilation. `--reuse-assembly`
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

## Reproduce

```sh
python3 benchmarks/gc-suite/matrix/prepare-http.py --output target/new-http-control
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
