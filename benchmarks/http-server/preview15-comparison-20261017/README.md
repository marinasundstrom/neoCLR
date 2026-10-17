# Preview 15 HTTP comparison — 2026-10-17

The published Preview 15 macOS ARM64 bundle runs the same plain/typed-JSON
workloads and current harness on the same Apple M1/macOS 27.0 host as the
[refreshed candidate](../candidate-refreshed-20261017/README.md).
Each cohort validates 1,152 responses across interpreter/JIT/AOT with zero errors.
Five measured rounds and one process warmup per mode use 32 requests per process,
discarding the first eight from timing. Builds and startup are outside throughput.

Preview 15 uses inclusive `0..32`, so its JSON fixture alone changes that spelling
to `0..<32`, preserving exactly 32 requests. The [fixture](JsonServer.rvn) is retained
and hashed. The [unadapted failure](json-unadapted-failure.json) timed out awaiting
`DONE` because the old server awaited a 33rd request; it is not a passing run.
The plain-text fixture uses a while loop and requires no adaptation.

| Workload | Mode | Preview 15 requests/s | Candidate requests/s | Candidate / baseline |
| --- | --- | ---: | ---: | ---: |
| plain | interpreter | 2.20 | 2.22 | 1.009 |
| plain | jit | 153.29 | 150.52 | 0.982 |
| plain | aot | 158.81 | 156.91 | 0.988 |
| json | interpreter | 0.97 | 1.20 | 1.232 |
| json | jit | 64.23 | 81.42 | 1.268 |
| json | aot | 66.58 | 84.65 | 1.271 |

Plain-text medians are close. JSON medians are higher for the candidate in this
run; this is a complete compiler/runtime/library comparison, not an isolated
optimization measurement. Sequential cohorts, active host indexing and interactive
load prevent firm speedup or regression claims from small differences. Raw samples,
load snapshots, startup and exact artifact/input hashes are in [plain](plain.json)
and [JSON](json.json). No concurrent local builds or benchmark runs overlapped timing.
Windows measurements remain separate. These data do not establish a cross-platform
ranking or a universal regression threshold.
