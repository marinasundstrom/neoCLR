# Native fiber transfer profile — 2026-10-25

Sixteen native HTTP configurations pass 16,640 responses: plain/JSON, close/pipeline,
AOT/JIT and sync/concurrent, 1,024 measured requests plus 16 warmup each. This is
one diagnostic run per cell, not a new throughput cohort. Compiler and collector
defaults are unchanged.

## Measurement boundary

The profiler copies `native-runtime.c` and timestamps immediately before each
`swapcontext` and immediately on arrival in the destination context, including
first entry and cached reuse. Guest execution, root parking/resumption, callback
admission, queue dispatch and stack cleanup lie outside this interval. Thread CPU
and wall clocks have deliberately different sampling endpoints; clock overhead
is included. Do not treat their difference as precise descheduling time or subtract
these durations from uninstrumented results.

Every shutdown requires balanced begin/arrival counts and exactly two transfers
per successful resume. The sanitized owned-body test passes 321 entries, 160
resumptions, 161 abandoned bodies and zero reserved bytes at shutdown. Deliberately
removing first-entry arrival accounting aborts. This checks instrumentation coverage
and existing lifetime behavior; it is not a new interleaving proof for all fibers.

| Payload | Connection | Mode | Transfers | Transfer thread CPU ms | Fraction of invocation CPU |
| --- | --- | --- | ---: | ---: | ---: |
| plain | close | sync-aot | 12,496 | 12.10 | 5.5% |
| plain | close | live-aot | 12,496 | 12.16 | 4.2% |
| plain | close | sync-jit | 12,496 | 11.01 | 3.4% |
| plain | close | live-jit | 12,496 | 11.18 | 2.9% |
| plain | pipeline | sync-aot | 4,976 | 4.46 | 3.1% |
| plain | pipeline | live-aot | 4,976 | 4.41 | 2.5% |
| plain | pipeline | sync-jit | 4,976 | 4.49 | 1.9% |
| plain | pipeline | live-jit | 4,976 | 4.51 | 1.6% |
| json | close | sync-aot | 12,496 | 10.80 | 3.7% |
| json | close | live-aot | 12,496 | 11.02 | 3.0% |
| json | close | sync-jit | 12,496 | 11.20 | 2.3% |
| json | close | live-jit | 12,496 | 11.36 | 2.0% |
| json | pipeline | sync-aot | 4,976 | 4.56 | 1.7% |
| json | pipeline | live-aot | 4,976 | 4.47 | 1.5% |
| json | pipeline | sync-jit | 4,976 | 4.59 | 1.1% |
| json | pipeline | live-jit | 4,976 | 4.65 | 1.0% |

Close-per-request performs 6,248 resumes / 12,496 transfers; pipelining performs
2,488 / 4,976, approximately 60% fewer transfers for the same checked work. Counts
match across both payloads and all native modes. Startup, warmup and cleanup are
included. Native invocation CPU includes collector threads; transfer CPU is on
the application owner. This is not OS context-switch or task queue-delay accounting.

## Architectural implication

Raw carrier switching is a measurable but comparatively small component here:
roughly 1–5.5% of instrumented invocation CPU including clock overhead. These data
do not justify replacing the carrier or adding virtual threads as the next HTTP
optimization. Reducing dispatch frequency and repeated work around resumptions is
a stronger next investigation, alongside the demonstrated shared-pool handoff gain.
This is an inference limited to the tested workloads. Root parking, host-root
admission, queue scheduling and readiness costs remain to be attributed separately.

## Reproduce

```sh
python3 benchmarks/gc-suite/concurrency/prepare-fiber-profile.py \
  --build target/gc-matrix-http-build-v3 --output target/new-fiber-build
python3 benchmarks/gc-suite/matrix/http.py \
  --build target/new-fiber-build --output target/new-fiber-profile \
  --modes sync-aot live-aot sync-jit live-jit --protocols close pipeline \
  --requests 1024 --rounds 1
```

The build is explicitly diagnostic and the matrix labels it `profile`, never
`benchmark`. The retained native guest objects are relinked with current native
host sources and the copied instrumented carrier; no guest recompile or proof-flag
change is involved. This private tool targets macOS ARM64/ucontext only. Commands,
raw diagnostics, positive/negative checks and hashes are in `fiber-build.json.gz`
and `fiber-http.json.gz`. No website build or publication is implied.
